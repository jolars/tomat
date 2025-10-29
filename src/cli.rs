use clap::{Args, Parser, Subcommand};

#[derive(Subcommand)]
pub enum DaemonAction {
    /// Start the daemon in the background
    Start,
    /// Stop the running daemon
    Stop,
    /// Check daemon status
    Status,
    /// Run the daemon in the foreground (internal use)
    #[command(hide = true)]
    Run,
}

#[derive(Parser)]
#[command(name = "tomat")]
#[command(about = "A Pomodoro timer with daemon support for waybar")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Args)]
pub struct TimerArgs {
    /// Work duration in minutes (default: 25)
    #[arg(short, long, default_value = "25")]
    pub work: f32,
    /// Break duration in minutes (default: 5)
    #[arg(short, long, default_value = "5")]
    pub break_time: f32,
    /// Long break duration in minutes (default: 15)
    #[arg(short, long, default_value = "15")]
    pub long_break: f32,
    /// Sessions until long break (default: 4)
    #[arg(short, long, default_value = "4")]
    pub sessions: u32,
    /// Automatically advance between timer states (default: false)
    #[arg(short, long, default_value = "false")]
    pub auto_advance: bool,
}

impl TimerArgs {
    #[cfg(not(doc))]
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "work": self.work,
            "break": self.break_time,
            "long_break": self.long_break,
            "sessions": self.sessions,
            "auto_advance": self.auto_advance
        })
    }
}

#[derive(Subcommand)]
pub enum Commands {
    /// Daemon management
    Daemon {
        #[command(subcommand)]
        action: DaemonAction,
    },
    /// Start a new Pomodoro session
    Start {
        #[command(flatten)]
        timer: TimerArgs,
    },
    /// Stop the current session
    Stop,
    /// Get current status as JSON
    Status,
    /// Skip to next phase
    Skip,
    /// Toggle timer (start if stopped, stop if running)
    Toggle {
        #[command(flatten)]
        timer: TimerArgs,
    },
}
