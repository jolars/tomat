use clap::{ArgAction, Args, Parser, Subcommand};

#[derive(Subcommand)]
pub enum DaemonAction {
    /// Start the daemon in the background
    #[command(
        long_about = "Start the tomat daemon as a background process. The daemon \
        manages timer state and handles client requests via a Unix socket at \
        $XDG_RUNTIME_DIR/tomat.sock. Only one daemon instance can run at a time."
    )]
    Start,
    /// Stop the running daemon
    #[command(
        long_about = "Stop the running tomat daemon gracefully. This will terminate \
        any active timer session. The daemon will clean up its socket and PID files."
    )]
    Stop,
    /// Check daemon status
    #[command(
        long_about = "Check if the tomat daemon is currently running and report its \
        process ID."
    )]
    Status,
    /// Install systemd user service
    #[command(
        long_about = "Install and enable the tomat systemd user service. This allows \
        the daemon to start automatically on login and restart if it crashes. The service \
        file is installed to ~/.config/systemd/user/tomat.service."
    )]
    #[command(
        after_help = "After installation, manage the service with systemctl:\n    \
        systemctl --user start tomat.service\n    \
        systemctl --user status tomat.service\n    \
        systemctl --user stop tomat.service"
    )]
    Install,
    /// Uninstall systemd user service
    #[command(
        long_about = "Stop and remove the tomat systemd user service. This removes \
        the service file and disables automatic startup."
    )]
    Uninstall,
    /// Run the daemon in the foreground (internal use)
    #[command(hide = true)]
    Run,
}

#[derive(Parser)]
#[command(name = "tomat")]
#[command(
    author,
    version,
    about = "A Pomodoro timer with daemon support for waybar"
)]
#[command(
    long_about = "Tomat is a Pomodoro timer with a daemon-based architecture, designed for \
    seamless integration with waybar and other status bars. It uses a Unix socket for \
    client-server communication, ensuring your timer state persists across waybar restarts \
    and system suspend/resume."
)]
#[command(after_help = "\
EXAMPLES:

    # Start daemon and begin a session
    tomat daemon start
    tomat start

    # Custom session durations
    tomat start --work 45 --break 15

    # Check status (outputs JSON for waybar)
    tomat status

    # Toggle pause/resume
    tomat toggle

For more information, visit: https://github.com/jolars/tomat")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Args)]
pub struct TimerArgs {
    /// Work session duration in minutes
    #[arg(short, long)]
    #[arg(help = "Work duration in minutes (default: from config or 25)")]
    #[arg(
        long_help = "Duration of work sessions in minutes. If not specified, uses the value \
        from ~/.config/tomat/config.toml or the built-in default of 25 minutes."
    )]
    pub work: Option<f32>,
    /// Break duration in minutes
    #[arg(short, long = "break")]
    #[arg(help = "Break duration in minutes (default: from config or 5)")]
    #[arg(
        long_help = "Duration of short breaks in minutes. If not specified, uses the value \
        from ~/.config/tomat/config.toml or the built-in default of 5 minutes."
    )]
    pub break_time: Option<f32>,
    /// Long break duration in minutes
    #[arg(short, long = "long-break")]
    #[arg(help = "Long break duration in minutes (default: from config or 15)")]
    #[arg(
        long_help = "Duration of long breaks in minutes. Long breaks occur after completing \
        the configured number of work sessions. If not specified, uses the value from \
        ~/.config/tomat/config.toml or the built-in default of 15 minutes."
    )]
    pub long_break: Option<f32>,
    /// Number of work sessions before a long break
    #[arg(short, long)]
    #[arg(help = "Sessions until long break (default: from config or 4)")]
    #[arg(
        long_help = "Number of work/break cycles before taking a long break. If not specified, \
        uses the value from ~/.config/tomat/config.toml or the built-in default of 4 sessions."
    )]
    pub sessions: Option<u32>,
    /// Automatically advance to the next phase
    #[arg(short, long)]
    #[arg(help = "Auto-advance mode: all, none, to-break, to-work (default: from config)")]
    #[arg(long_help = "Control automatic phase transitions:\n  \
        all      - Auto-advance through all phases\n  \
        none     - Never auto-advance (pause at transitions)\n  \
        to-break - Auto-advance from work to break only\n  \
        to-work  - Auto-advance from break to work only\n\n\
        If not specified, uses the value from ~/.config/tomat/config.toml or the \
        built-in default of 'none'.")]
    pub auto_advance: Option<String>,
    /// Sound notification mode
    #[arg(long)]
    #[arg(help = "Sound mode: embedded, system-beep, none (default: from config)")]
    #[arg(long_help = "Control sound notifications:\n  \
        embedded    - Use built-in audio files (default)\n  \
        system-beep - Use system beep (terminal bell)\n  \
        none        - No sound notifications\n\n\
        If not specified, uses the value from ~/.config/tomat/config.toml or the \
        built-in default of 'embedded'.")]
    pub sound_mode: Option<String>,
    /// DEPRECATED: Enable sound notifications for this session
    #[arg(long, action = ArgAction::SetTrue, hide = true)]
    pub sound: bool,
    /// DEPRECATED: Use system beep instead of audio files
    #[arg(long, action = ArgAction::SetTrue, hide = true)]
    pub beep: bool,
    /// Audio volume level (0.0 to 1.0)
    #[arg(long)]
    #[arg(help = "Volume level (0.0-1.0)")]
    #[arg(
        long_help = "Set the audio volume for sound notifications, from 0.0 (silent) to 1.0 \
        (maximum). Values outside this range will be clamped. If not specified, uses the \
        value from ~/.config/tomat/config.toml or the built-in default of 0.5."
    )]
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

    /// Get auto_advance with fallback - returns string to avoid circular dependency
    pub fn get_auto_advance_str(&self, default_str: &str) -> String {
        self.auto_advance
            .clone()
            .unwrap_or_else(|| default_str.to_string())
    }

    /// Get sound mode with fallback, considering deprecated fields
    pub fn get_sound_mode(&self, default_str: &str) -> String {
        // Check for new sound_mode first
        if let Some(ref mode) = self.sound_mode {
            return mode.clone();
        }

        // Handle deprecated flags for backwards compatibility
        if self.beep {
            return "system-beep".to_string();
        }
        if self.sound {
            return "embedded".to_string();
        }

        default_str.to_string()
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
    /// Manage the background daemon
    #[command(
        long_about = "Manage the tomat daemon, which runs in the background and \
        maintains timer state. The daemon must be running for timer commands to work."
    )]
    Daemon {
        #[command(subcommand)]
        action: DaemonAction,
    },
    /// Start a new Pomodoro session
    #[command(
        long_about = "Start a new Pomodoro timer session with the specified durations. \
        If no options are provided, uses defaults from ~/.config/tomat/config.toml or \
        built-in defaults (25min work, 5min break, 15min long break, 4 sessions). \
        Custom durations only apply to the current session."
    )]
    #[command(after_help = "\
EXAMPLES:

    # Start with defaults
    tomat start

    # Custom work/break durations
    tomat start --work 45 --break 15

    # Auto-advance between phases
    tomat start --auto-advance")]
    Start {
        #[command(flatten)]
        timer: TimerArgs,
    },
    /// Stop the current session
    #[command(long_about = "Stop the current Pomodoro session and return the timer to idle state.")]
    Stop,
    /// Get current timer status
    #[command(
        long_about = "Display the current timer status. Output format can be customized \
        for different status bars (waybar, i3status-rs) or plain text. Text appearance can be \
        customized using format templates."
    )]
    #[command(after_help = "\
OUTPUT FORMATS:

`waybar`
  : JSON output for waybar (default)

`i3status-rs`
  : JSON output for i3status-rs

`plain`
  : Plain text output

FORMAT PLACEHOLDERS:

`{icon}`
  : Phase icon (🍅 work, ☕ break, 🏖️ long break)

`{time}`
  : Remaining time (MM:SS)

`{state}`
  : Play/pause symbol (▶/⏸)

`{phase}`
  : Phase name (Work/Break/Long Break)

`{session}`
  : Session progress (e.g. 1/4)

EXAMPLES:

    tomat status
    tomat status --output plain
    tomat status --format \"{time}\"
    tomat status --format \"{phase}: {time} {state}\"")]
    Status {
        /// Output format: waybar, i3status-rs, or plain
        #[arg(short, long, default_value = "waybar")]
        #[arg(value_parser = ["waybar", "i3status-rs", "plain"])]
        output: String,
        /// Text format template
        #[arg(short = 'f', long)]
        #[arg(help = "Custom text format (e.g. \"{icon} {time}\")")]
        #[arg(long_help = "Customize the text display using placeholders:\n\
            {icon}    - Phase icon\n\
            {time}    - Remaining time (MM:SS)\n\
            {state}   - Play/pause symbol\n\
            {phase}   - Phase name\n\
            {session} - Session progress")]
        format: Option<String>,
    },
    /// Continuously output status updates
    #[command(
        long_about = "Continuously watch and output timer status updates. This maintains \
        a single connection to the daemon and updates at the specified interval. Automatically \
        exits when the daemon stops. More efficient than polling with 'status' command."
    )]
    #[command(after_help = "\
EXAMPLES:

    # Watch with default interval (0.25 seconds)
    tomat watch

    # Watch with 5-second updates
    tomat watch --interval 5

    # Watch with plain text output
    tomat watch --output plain")]
    Watch {
        /// Output format: waybar, i3status-rs, or plain
        #[arg(short, long, default_value = "waybar")]
        #[arg(value_parser = ["waybar", "i3status-rs", "plain"])]
        output: String,
        /// Text format template
        #[arg(short = 'f', long)]
        #[arg(help = "Custom text format (e.g. \"{icon} {time}\")")]
        format: Option<String>,
        /// Update interval in seconds
        #[arg(short, long, default_value = "0.25")]
        interval: f64,
    },
    /// Skip to the next phase
    #[command(
        long_about = "Skip the current phase and immediately transition to the next phase \
        (work → break → work → ... → long break). The timer will start in the new phase if \
        auto-advance is enabled, otherwise it will be paused."
    )]
    Skip,
    /// Pause the current timer
    #[command(
        long_about = "Pause the currently running timer. Use 'resume' or 'toggle' to \
        continue."
    )]
    Pause,
    /// Resume a paused timer
    #[command(long_about = "Resume a paused timer from where it left off.")]
    Resume,
    /// Toggle timer pause/resume
    #[command(
        long_about = "Toggle the timer state: pause if running, resume if paused. This is \
        useful for waybar click handlers."
    )]
    Toggle,
}
