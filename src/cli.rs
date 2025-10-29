use clap::{ArgAction, Args, Parser, Subcommand};

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
    /// Work duration in minutes (default: from config or 25)
    #[arg(short, long)]
    pub work: Option<f32>,
    /// Break duration in minutes (default: from config or 5)
    #[arg(short, long)]
    pub break_time: Option<f32>,
    /// Long break duration in minutes (default: from config or 15)
    #[arg(short, long)]
    pub long_break: Option<f32>,
    /// Sessions until long break (default: from config or 4)
    #[arg(short, long)]
    pub sessions: Option<u32>,
    /// Automatically advance between timer states (default: from config or false)
    #[arg(short, long, action = ArgAction::SetTrue)]
    pub auto_advance: bool,
    /// Enable sound notifications
    #[arg(long, action = ArgAction::SetTrue)]
    pub sound: bool,
    /// Use system beep instead of sound files
    #[arg(long, action = ArgAction::SetTrue)]
    pub beep: bool,
    /// Volume level (0.0-1.0)
    #[arg(long)]
    pub volume: Option<f32>,
}

impl TimerArgs {
    /// Get work duration with fallback
    pub fn get_work(&self, default: f32) -> f32 {
        self.work.unwrap_or(default)
    }

    /// Get break duration with fallback
    pub fn get_break_time(&self, default: f32) -> f32 {
        self.break_time.unwrap_or(default)
    }

    /// Get long break duration with fallback
    pub fn get_long_break(&self, default: f32) -> f32 {
        self.long_break.unwrap_or(default)
    }

    /// Get sessions with fallback
    pub fn get_sessions(&self, default: u32) -> u32 {
        self.sessions.unwrap_or(default)
    }

    /// Get auto_advance with fallback
    pub fn get_auto_advance(&self, default: bool) -> bool {
        // If flag is set, it's true; otherwise use default
        if self.auto_advance { true } else { default }
    }

    /// Get sound enabled with fallback
    pub fn get_sound(&self, default: bool) -> bool {
        if self.sound { true } else { default }
    }

    /// Get beep enabled with fallback
    pub fn get_beep(&self, default: bool) -> bool {
        if self.beep { true } else { default }
    }

    /// Get volume with fallback
    pub fn get_volume(&self, default: f32) -> f32 {
        match self.volume {
            Some(v) if (0.0..=1.0).contains(&v) => v,
            Some(v) => {
                eprintln!(
                    "Warning: Volume {} out of range (0.0-1.0), using {}",
                    v, default
                );
                default
            }
            None => default,
        }
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
    /// Pause the current timer
    Pause,
    /// Resume a paused timer
    Resume,
    /// Toggle timer (start if stopped, stop if running)
    Toggle,
}
