use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Config {
    #[serde(default)]
    pub timer: TimerConfig,
    #[serde(default)]
    pub sound: SoundConfig,
    #[serde(default)]
    pub notification: NotificationConfig,
    #[serde(default)]
    pub display: DisplayConfig,
    #[serde(default)]
    pub hooks: HooksConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimerConfig {
    /// Work duration in minutes (default: 25)
    #[serde(default = "default_work")]
    pub work: f32,
    /// Break duration in minutes (default: 5)
    #[serde(default = "default_break", rename = "break")]
    pub break_time: f32,
    /// Long break duration in minutes (default: 15)
    #[serde(default = "default_long_break")]
    pub long_break: f32,
    /// Sessions until long break (default: 4)
    #[serde(default = "default_sessions")]
    pub sessions: u32,
    /// Automatically advance between timer states (default: false)
    #[serde(default)]
    pub auto_advance: bool,
}

fn default_work() -> f32 {
    25.0
}

fn default_break() -> f32 {
    5.0
}

fn default_long_break() -> f32 {
    15.0
}

fn default_sessions() -> u32 {
    4
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NotificationConfig {
    /// Enable desktop notifications (default: true)
    #[serde(default = "default_notification_enabled")]
    pub enabled: bool,
    /// Icon to use for notifications (default: "auto")
    /// "auto" = use embedded icon, "theme" = use system theme icon, or path to custom icon
    #[serde(default = "default_icon")]
    pub icon: String,
    /// Notification timeout in milliseconds (default: 5000)
    #[serde(default = "default_timeout")]
    pub timeout: u32,
}

fn default_notification_enabled() -> bool {
    true
}

fn default_icon() -> String {
    "auto".to_string()
}

fn default_timeout() -> u32 {
    5000
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DisplayConfig {
    /// Text format template (default: "{icon} {time} {state}")
    /// Available placeholders: {icon}, {time}, {state}, {phase}, {session}
    #[serde(default = "default_text_format")]
    pub text_format: String,
}

fn default_text_format() -> String {
    "{icon} {time} {state}".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SoundConfig {
    /// Enable sound notifications (default: true)
    #[serde(default)]
    pub enabled: bool,
    /// Use system beep instead of any sound files (default: false)
    #[serde(default)]
    pub system_beep: bool,
    /// Use embedded sounds (default: true)
    #[serde(default = "default_use_embedded")]
    pub use_embedded: bool,
    /// Volume level 0.0-1.0 (default: 0.5)
    #[serde(default = "default_volume")]
    pub volume: f32,
    /// Custom sound file for work->break transition (overrides embedded)
    pub work_to_break: Option<String>,
    /// Custom sound file for break->work transition (overrides embedded)
    pub break_to_work: Option<String>,
    /// Custom sound file for work->long_break transition (overrides embedded)
    pub work_to_long_break: Option<String>,
}

fn default_use_embedded() -> bool {
    true
}

fn default_volume() -> f32 {
    0.5
}

impl Default for SoundConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            system_beep: false,
            use_embedded: true,
            volume: 0.5,
            work_to_break: None,
            break_to_work: None,
            work_to_long_break: None,
        }
    }
}

impl Default for TimerConfig {
    fn default() -> Self {
        Self {
            work: default_work(),
            break_time: default_break(),
            long_break: default_long_break(),
            sessions: default_sessions(),
            auto_advance: false,
        }
    }
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            enabled: default_notification_enabled(),
            icon: default_icon(),
            timeout: default_timeout(),
        }
    }
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            text_format: default_text_format(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct HooksConfig {
    #[serde(default)]
    pub on_work_start: Option<HookCommand>,
    #[serde(default)]
    pub on_break_start: Option<HookCommand>,
    #[serde(default)]
    pub on_long_break_start: Option<HookCommand>,
    #[serde(default)]
    pub on_pause: Option<HookCommand>,
    #[serde(default)]
    pub on_resume: Option<HookCommand>,
    #[serde(default)]
    pub on_stop: Option<HookCommand>,
    #[serde(default)]
    pub on_complete: Option<HookCommand>,
    #[serde(default)]
    pub on_skip: Option<HookCommand>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HookCommand {
    /// Command to execute
    pub cmd: String,
    /// Command arguments (default: empty)
    #[serde(default)]
    pub args: Vec<String>,
    /// Timeout in seconds (default: 5)
    #[serde(default = "default_hook_timeout")]
    pub timeout: u64,
    /// Working directory (default: user's home directory)
    #[serde(default)]
    pub cwd: Option<String>,
    /// Capture output for debugging (default: false, redirects to /dev/null)
    #[serde(default)]
    pub capture_output: bool,
}

fn default_hook_timeout() -> u64 {
    5
}

impl HookCommand {
    /// Execute the hook command asynchronously
    pub async fn execute(
        &self,
        event: &str,
        phase: &str,
        remaining_seconds: u64,
        session_count: u32,
        auto_advance: bool,
    ) {
        use std::process::Stdio;
        use tokio::process::Command;

        let mut cmd = Command::new(&self.cmd);
        cmd.args(&self.args);

        // Set environment variables
        cmd.env("TOMAT_EVENT", event);
        cmd.env("TOMAT_PHASE", phase);
        cmd.env("TOMAT_REMAINING_SECONDS", remaining_seconds.to_string());
        cmd.env("TOMAT_SESSION_COUNT", session_count.to_string());
        cmd.env("TOMAT_AUTO_ADVANCE", auto_advance.to_string());

        // Set working directory
        if let Some(cwd) = &self.cwd {
            cmd.current_dir(cwd);
        } else if let Some(home) = dirs::home_dir() {
            cmd.current_dir(home);
        }

        // Configure output handling
        if self.capture_output {
            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());
        } else {
            cmd.stdout(Stdio::null());
            cmd.stderr(Stdio::null());
        }

        // Spawn the command
        let mut child = match cmd.spawn() {
            Ok(child) => child,
            Err(e) => {
                eprintln!("Failed to spawn hook command '{}': {}", self.cmd, e);
                return;
            }
        };

        // Wait for command to complete with timeout
        let timeout_duration = std::time::Duration::from_secs(self.timeout);
        match tokio::time::timeout(timeout_duration, child.wait()).await {
            Ok(Ok(status)) => {
                if !status.success() {
                    eprintln!("Hook command '{}' exited with status: {}", self.cmd, status);
                }
            }
            Ok(Err(e)) => {
                eprintln!("Hook command '{}' failed: {}", self.cmd, e);
            }
            Err(_) => {
                eprintln!(
                    "Hook command '{}' timed out after {} seconds",
                    self.cmd, self.timeout
                );
                let _ = child.kill().await;
            }
        }
    }
}

impl HooksConfig {
    /// Execute a hook by event name
    pub async fn execute_hook(
        &self,
        event: &str,
        phase: &str,
        remaining_seconds: u64,
        session_count: u32,
        auto_advance: bool,
    ) {
        let hook = match event {
            "work_start" => &self.on_work_start,
            "break_start" => &self.on_break_start,
            "long_break_start" => &self.on_long_break_start,
            "pause" => &self.on_pause,
            "resume" => &self.on_resume,
            "stop" => &self.on_stop,
            "complete" => &self.on_complete,
            "skip" => &self.on_skip,
            _ => return,
        };

        if let Some(hook_cmd) = hook {
            hook_cmd
                .execute(event, phase, remaining_seconds, session_count, auto_advance)
                .await;
        }
    }
}

impl Config {
    /// Get the config file path
    pub fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|dir| dir.join("tomat").join("config.toml"))
    }

    /// Load config from file, falling back to defaults if not found
    pub fn load() -> Self {
        Self::config_path()
            .and_then(|path| {
                if path.exists() {
                    fs::read_to_string(&path)
                        .ok()
                        .and_then(|contents| toml::from_str(&contents).ok())
                } else {
                    None
                }
            })
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.timer.work, 25.0);
        assert_eq!(config.timer.break_time, 5.0);
        assert_eq!(config.timer.long_break, 15.0);
        assert_eq!(config.timer.sessions, 4);
        assert!(!config.timer.auto_advance);

        // Test notification defaults
        assert!(config.notification.enabled);
        assert_eq!(config.notification.icon, "auto");
        assert_eq!(config.notification.timeout, 5000);

        // Test display defaults
        assert_eq!(config.display.text_format, "{icon} {time} {state}");
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&toml_str).unwrap();

        assert_eq!(deserialized.timer.work, config.timer.work);
        assert_eq!(deserialized.timer.break_time, config.timer.break_time);
        assert_eq!(deserialized.timer.long_break, config.timer.long_break);
        assert_eq!(deserialized.timer.sessions, config.timer.sessions);
        assert_eq!(deserialized.timer.auto_advance, config.timer.auto_advance);

        // Test notification serialization
        assert_eq!(
            deserialized.notification.enabled,
            config.notification.enabled
        );
        assert_eq!(deserialized.notification.icon, config.notification.icon);
        assert_eq!(
            deserialized.notification.timeout,
            config.notification.timeout
        );
    }

    #[test]
    fn test_partial_config() {
        let toml_str = r#"
            [timer]
            work = 30.0
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.timer.work, 30.0);
        // Other fields should use defaults
        assert_eq!(config.timer.break_time, 5.0);
        assert_eq!(config.timer.long_break, 15.0);
        assert_eq!(config.timer.sessions, 4);
    }

    #[test]
    fn test_empty_config() {
        let toml_str = "";
        let config: Config = toml::from_str(toml_str).unwrap();

        // Should all be defaults
        assert_eq!(config.timer.work, 25.0);
        assert_eq!(config.timer.break_time, 5.0);
        assert_eq!(config.timer.long_break, 15.0);
        assert_eq!(config.timer.sessions, 4);
        assert!(!config.timer.auto_advance);
    }

    #[test]
    fn test_config_load_returns_default_when_no_file() {
        // This should not panic and should return defaults
        let config = Config::load();
        assert_eq!(config.timer.work, 25.0);
    }

    #[test]
    fn test_config_uses_break_not_break_time() {
        let config = Config::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();

        // Should use "break" in the TOML output, not "break_time"
        assert!(
            toml_str.contains("break = "),
            "Config should serialize with 'break' field"
        );
        assert!(
            !toml_str.contains("break_time"),
            "Config should not serialize with 'break_time' field"
        );
    }

    #[test]
    fn test_config_can_parse_break_field() {
        let toml_str = r#"
            [timer]
            work = 30
            break = 7.0
            long_break = 20.0
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.timer.work, 30.0);
        assert_eq!(config.timer.break_time, 7.0);
        assert_eq!(config.timer.long_break, 20.0);
    }

    #[test]
    fn test_notification_config() {
        let toml_str = r#"
            [notification]
            enabled = false
            icon = "/path/to/custom/icon.png"
            timeout = 5000
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert!(!config.notification.enabled);
        assert_eq!(config.notification.icon, "/path/to/custom/icon.png");
        assert_eq!(config.notification.timeout, 5000);

        // Timer should still use defaults
        assert_eq!(config.timer.work, 25.0);
    }

    #[test]
    fn test_partial_notification_config() {
        let toml_str = r#"
            [notification]
            icon = "theme"
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert!(config.notification.enabled); // Should use default
        assert_eq!(config.notification.icon, "theme");
        assert_eq!(config.notification.timeout, 5000); // Should use default
    }

    #[test]
    fn test_hooks_config_default() {
        let config = Config::default();
        assert!(config.hooks.on_work_start.is_none());
        assert!(config.hooks.on_pause.is_none());
        assert!(config.hooks.on_resume.is_none());
    }

    #[test]
    fn test_hooks_config_parsing() {
        let toml_str = r#"
            [hooks.on_work_start]
            cmd = "notify-send"
            args = ["Work time!", "Let's focus"]
            timeout = 3
            cwd = "/tmp"
            capture_output = true

            [hooks.on_pause]
            cmd = "playerctl"
            args = ["pause"]
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();

        let work_hook = config.hooks.on_work_start.as_ref().unwrap();
        assert_eq!(work_hook.cmd, "notify-send");
        assert_eq!(work_hook.args, vec!["Work time!", "Let's focus"]);
        assert_eq!(work_hook.timeout, 3);
        assert_eq!(work_hook.cwd.as_deref(), Some("/tmp"));
        assert!(work_hook.capture_output);

        let pause_hook = config.hooks.on_pause.as_ref().unwrap();
        assert_eq!(pause_hook.cmd, "playerctl");
        assert_eq!(pause_hook.args, vec!["pause"]);
        assert_eq!(pause_hook.timeout, 5); // Default
        assert!(!pause_hook.capture_output); // Default
    }

    #[test]
    fn test_hooks_defaults() {
        let toml_str = r#"
            [hooks.on_resume]
            cmd = "echo"
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        let hook = config.hooks.on_resume.as_ref().unwrap();

        assert_eq!(hook.cmd, "echo");
        assert!(hook.args.is_empty()); // Default
        assert_eq!(hook.timeout, 5); // Default
        assert!(hook.cwd.is_none()); // Default
        assert!(!hook.capture_output); // Default
    }
}
