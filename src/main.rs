mod server;
mod timer;

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

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
