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

    /// Save config to file
    #[allow(dead_code)]
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::config_path().ok_or("Could not determine config directory")?;

        // Create config directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let contents = toml::to_string_pretty(self)?;
        fs::write(&path, contents)?;
        Ok(())
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
}
