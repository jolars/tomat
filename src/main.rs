mod server;

use clap::{Parser, Subcommand};
use notify_rust::Notification;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::server::{run_daemon, send_command};

#[derive(Serialize, Deserialize)]
struct ServerResponse {
    success: bool,
    data: serde_json::Value,
    message: String,
}

#[derive(Parser)]
#[command(name = "tomat")]
#[command(about = "A Pomodoro timer with daemon support for waybar")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the daemon (usually run by systemd)
    Daemon,
    /// Start a new Pomodoro session
    Start {
        /// Work duration in minutes (default: 25)
        #[arg(short, long, default_value = "25")]
        work: u32,
        /// Break duration in minutes (default: 5)
        #[arg(short, long, default_value = "5")]
        break_time: u32,
        /// Long break duration in minutes (default: 15)
        #[arg(short, long, default_value = "15")]
        long_break: u32,
        /// Sessions until long break (default: 4)
        #[arg(short, long, default_value = "4")]
        sessions: u32,
    },
    /// Stop the current session
    Stop,
    /// Get current status as JSON
    Status,
    /// Skip to next phase
    Skip,
    /// Toggle timer (start if stopped, stop if running)
    Toggle {
        /// Work duration in minutes (default: 25)
        #[arg(short, long, default_value = "25")]
        work: u32,
        /// Break duration in minutes (default: 5)
        #[arg(short, long, default_value = "5")]
        break_time: u32,
        /// Long break duration in minutes (default: 15)
        #[arg(short, long, default_value = "15")]
        long_break: u32,
        /// Sessions until long break (default: 4)
        #[arg(short, long, default_value = "4")]
        sessions: u32,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
enum Phase {
    Work,
    Break,
    LongBreak,
    Idle,
}

#[derive(Serialize, Deserialize, Clone)]
struct TimerState {
    phase: Phase,
    start_time: u64,
    duration_minutes: u32,
    work_duration: u32,
    break_duration: u32,
    long_break_duration: u32,
    sessions_until_long_break: u32,
    current_session_count: u32,
}

#[derive(Serialize)]
struct StatusOutput {
    text: String,
    tooltip: String,
    class: String,
    percentage: f64,
}

impl TimerState {
    fn new(work: u32, break_time: u32, long_break: u32, sessions: u32) -> Self {
        Self {
            phase: Phase::Idle,
            start_time: 0,
            duration_minutes: 0,
            work_duration: work,
            break_duration: break_time,
            long_break_duration: long_break,
            sessions_until_long_break: sessions,
            current_session_count: 0,
        }
    }

    fn start_work(&mut self) {
        self.phase = Phase::Work;
        self.duration_minutes = self.work_duration;
        self.start_time = current_timestamp();
    }

    fn start_break(&mut self) {
        self.phase = Phase::Break;
        self.duration_minutes = self.break_duration;
        self.start_time = current_timestamp();
    }

    fn start_long_break(&mut self) {
        self.phase = Phase::LongBreak;
        self.duration_minutes = self.long_break_duration;
        self.start_time = current_timestamp();
    }

    fn get_remaining_seconds(&self) -> i64 {
        if matches!(self.phase, Phase::Idle) {
            return 0;
        }

        let elapsed = current_timestamp() - self.start_time;
        let total_duration = self.duration_minutes as u64 * 60;

        if elapsed >= total_duration {
            0
        } else {
            (total_duration - elapsed) as i64
        }
    }

    fn is_finished(&self) -> bool {
        self.get_remaining_seconds() <= 0 && !matches!(self.phase, Phase::Idle)
    }

    fn next_phase(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let (message, _icon) = match self.phase {
            Phase::Work => {
                self.current_session_count += 1;

                // Check if it's time for a long break
                if self.current_session_count >= self.sessions_until_long_break {
                    self.current_session_count = 0;
                    self.start_long_break();
                    ("Long break time! Take a well-deserved rest 🏖️", "🍅")
                } else {
                    self.start_break();
                    ("Break time! Take a rest ☕", "🍅")
                }
            }
            Phase::Break => {
                self.start_work();
                ("Back to work! Stay focused 💪", "🍅")
            }
            Phase::LongBreak => {
                self.start_work();
                ("Back to work! You're refreshed and ready 🚀", "🍅")
            }
            Phase::Idle => {
                self.start_work();
                ("Pomodoro started! Let's focus 🍅", "🍅")
            }
        };

        // Send desktop notification (synchronous to avoid cross-platform issues)
        if let Err(e) = Notification::new()
            .summary("Pomodoro Timer")
            .body(message)
            .icon("timer")
            .timeout(3000)
            .show()
        {
            eprintln!("Failed to send notification: {}", e);
        }

        Ok(())
    }

    fn stop(&mut self) {
        self.phase = Phase::Idle;
        self.start_time = 0;
        self.duration_minutes = 0;
        self.current_session_count = 0;
    }

    fn get_status_output(&self) -> StatusOutput {
        let remaining = self.get_remaining_seconds();

        if matches!(self.phase, Phase::Idle) {
            return StatusOutput {
                text: "🍅 Idle".to_string(),
                tooltip: "Pomodoro timer idle".to_string(),
                class: "idle".to_string(),
                percentage: 0.0,
            };
        }

        let total_duration = self.duration_minutes as i64 * 60;
        let elapsed = total_duration - remaining;
        let percentage = if total_duration > 0 {
            (elapsed as f64 / total_duration as f64) * 100.0
        } else {
            100.0
        };

        let (icon, class) = match self.phase {
            Phase::Work => ("🍅", "work"),
            Phase::Break => ("☕", "break"),
            Phase::LongBreak => ("🏖️", "long-break"),
            Phase::Idle => ("🍅", "idle"),
        };

        let time_str = if remaining <= 0 {
            "Done!".to_string()
        } else {
            format!("{:02}:{:02}", remaining / 60, remaining % 60)
        };

        let phase_name = match self.phase {
            Phase::Work => "Work",
            Phase::Break => "Break",
            Phase::LongBreak => "Long Break",
            Phase::Idle => "Idle",
        };

        let sessions_info = if matches!(self.phase, Phase::Work) {
            format!(
                " ({}/{})",
                self.current_session_count + 1,
                self.sessions_until_long_break
            )
        } else {
            String::new()
        };

        StatusOutput {
            text: format!("{} {}", icon, time_str),
            tooltip: format!(
                "{}{} - {}min",
                phase_name, sessions_info, self.duration_minutes
            ),
            class: class.to_string(),
            percentage,
        }
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Daemon => {
            run_daemon().await?;
        }

        Commands::Start {
            work,
            break_time,
            long_break,
            sessions,
        } => {
            let args = serde_json::json!({
                "work": work,
                "break": break_time,
                "long_break": long_break,
                "sessions": sessions
            });

            match send_command("start", args).await {
                Ok(response) => {
                    if response.success {
                        println!(
                            "Pomodoro started: {}min work, {}min break, {}min long break every {} sessions",
                            work, break_time, long_break, sessions
                        );
                    } else {
                        eprintln!("Error: {}", response.message);
                    }
                }
                Err(e) => eprintln!("Failed to connect to daemon: {}", e),
            }
        }

        Commands::Stop => match send_command("stop", serde_json::Value::Null).await {
            Ok(response) => {
                if response.success {
                    println!("Timer stopped");
                } else {
                    eprintln!("Error: {}", response.message);
                }
            }
            Err(e) => eprintln!("Failed to connect to daemon: {}", e),
        },

        Commands::Status => match send_command("status", serde_json::Value::Null).await {
            Ok(response) => {
                if response.success {
                    println!("{}", serde_json::to_string(&response.data)?);
                } else {
                    eprintln!("Error: {}", response.message);
                }
            }
            Err(e) => eprintln!("Failed to connect to daemon: {}", e),
        },

        Commands::Skip => match send_command("skip", serde_json::Value::Null).await {
            Ok(response) => {
                if response.success {
                    println!("Skipped to next phase");
                } else {
                    eprintln!("Error: {}", response.message);
                }
            }
            Err(e) => eprintln!("Failed to connect to daemon: {}", e),
        },

        Commands::Toggle {
            work,
            break_time,
            long_break,
            sessions,
        } => {
            let args = serde_json::json!({
                "work": work,
                "break": break_time,
                "long_break": long_break,
                "sessions": sessions
            });

            match send_command("toggle", args).await {
                Ok(response) => {
                    if response.success {
                        println!("{}", response.message);
                    } else {
                        eprintln!("Error: {}", response.message);
                    }
                }
                Err(e) => eprintln!("Failed to connect to daemon: {}", e),
            }
        }
    }

    Ok(())
}
