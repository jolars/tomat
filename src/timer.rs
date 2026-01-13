use notify_rust::Notification;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::audio::{AudioPlayer, SoundType};
use crate::config::{AutoAdvanceMode, NotificationConfig, SoundConfig};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    #[default]
    Waybar,
    Plain,
    I3statusRs,
}

impl std::str::FromStr for Format {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "waybar" => Ok(Format::Waybar),
            "plain" => Ok(Format::Plain),
            "i3status-rs" => Ok(Format::I3statusRs),
            _ => Err(format!(
                "Unknown format: '{}'. Supported formats: waybar, plain, i3status-rs",
                s
            )),
        }
    }
}

// Embed the icon file at compile time
static ICON_DATA: &[u8] = include_bytes!("../assets/icon.png");

/// Get the appropriate icon for notifications based on configuration
fn get_notification_icon(
    config: &NotificationConfig,
) -> Result<String, Box<dyn std::error::Error>> {
    match config.icon.as_str() {
        "auto" => {
            // Use embedded icon
            let icon_path = get_cached_icon_path()?;
            icon_path
                .to_str()
                .ok_or("Icon path contains invalid UTF-8".into())
                .map(|s| s.to_string())
        }
        "theme" => {
            // Use system theme icon
            Ok("timer".to_string())
        }
        custom_path => {
            // Use custom icon path
            let path = PathBuf::from(custom_path);
            if path.exists() {
                Ok(custom_path.to_string())
            } else {
                // Fall back to embedded icon if custom path doesn't exist
                eprintln!(
                    "Warning: Custom icon path '{}' not found, falling back to embedded icon",
                    custom_path
                );
                let icon_path = get_cached_icon_path()?;
                icon_path
                    .to_str()
                    .ok_or("Icon path contains invalid UTF-8".into())
                    .map(|s| s.to_string())
            }
        }
    }
}

/// Get the path to the cached icon file, creating it if necessary
fn get_cached_icon_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Use XDG cache directory
    let cache_dir = match dirs::cache_dir() {
        Some(dir) => dir.join("tomat"),
        None => {
            // Fallback to ~/.cache/tomat if XDG cache dir is not available
            match dirs::home_dir() {
                Some(home) => home.join(".cache").join("tomat"),
                None => return Err("Could not determine cache directory".into()),
            }
        }
    };

    // Create cache directory if it doesn't exist
    fs::create_dir_all(&cache_dir)?;

    let icon_path = cache_dir.join("icon.png");

    // Write icon file if it doesn't exist or if it's outdated
    if !icon_path.exists() || is_icon_outdated(&icon_path)? {
        let mut file = fs::File::create(&icon_path)?;
        file.write_all(ICON_DATA)?;
    }

    Ok(icon_path)
}

/// Check if the cached icon file is outdated compared to the embedded data
fn is_icon_outdated(icon_path: &PathBuf) -> Result<bool, Box<dyn std::error::Error>> {
    let existing_data = fs::read(icon_path)?;
    Ok(existing_data != ICON_DATA)
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TimerState {
    pub phase: Phase,
    pub start_time: u64,
    pub duration_minutes: f32,
    pub work_duration: f32,
    pub break_duration: f32,
    pub long_break_duration: f32,
    pub sessions_until_long_break: u32,
    pub current_session_count: u32,
    pub auto_advance: AutoAdvanceMode,
    pub is_paused: bool,
    /// Elapsed seconds when timer was paused (to preserve progress on resume)
    #[serde(default)]
    pub paused_elapsed_seconds: Option<u64>,
    /// Hook that should be executed when timer resumes from paused state after phase transition
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pending_hook: Option<String>,
}

/// Raw timer status data - pure state, no presentation
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TimerStatus {
    pub phase: Phase,                   // Work, Break, or LongBreak
    pub is_paused: bool,                // Whether timer is paused
    pub remaining_seconds: u64,         // Time remaining in current phase
    pub duration_minutes: f32,          // Total duration of current phase
    pub current_session: u32,           // Current session number (1-based)
    pub sessions_until_long_break: u32, // Total sessions before long break
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum StatusOutput {
    Waybar {
        text: String,
        tooltip: String,
        class: String,
        percentage: f64,
    },
    I3statusRs {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        short_text: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        icon: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        state: Option<String>,
    },
    Plain(String),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Phase {
    Work,
    Break,
    LongBreak,
}

impl std::fmt::Display for Phase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Phase::Work => write!(f, "work"),
            Phase::Break => write!(f, "break"),
            Phase::LongBreak => write!(f, "long_break"),
        }
    }
}

impl TimerState {
    pub fn new(work: f32, break_time: f32, long_break: f32, sessions: u32) -> Self {
        Self {
            phase: Phase::Work,
            start_time: 0,
            duration_minutes: work,
            work_duration: work,
            break_duration: break_time,
            long_break_duration: long_break,
            sessions_until_long_break: sessions,
            current_session_count: 0,
            auto_advance: AutoAdvanceMode::None,
            is_paused: true, // Start in paused state
            paused_elapsed_seconds: None,
            pending_hook: None,
        }
    }

    pub fn start_work(&mut self) {
        self.phase = Phase::Work;
        self.duration_minutes = self.work_duration;
        self.start_time = current_timestamp();
        self.is_paused = false;
    }

    fn start_break(&mut self) {
        self.phase = Phase::Break;
        self.duration_minutes = self.break_duration;
        self.start_time = current_timestamp();
        self.is_paused = false;
    }

    fn start_long_break(&mut self) {
        self.phase = Phase::LongBreak;
        self.duration_minutes = self.long_break_duration;
        self.start_time = current_timestamp();
        self.is_paused = false;
    }

    pub fn get_remaining_seconds(&self) -> u64 {
        if self.is_paused {
            // If we have stored elapsed time, calculate remaining from that
            if let Some(elapsed) = self.paused_elapsed_seconds {
                let total_duration = (self.duration_minutes * 60.0) as i64;
                let remaining = total_duration - elapsed as i64;
                return remaining.max(0) as u64;
            }
            // Otherwise return full duration (initial paused state)
            return (self.duration_minutes * 60.0) as u64;
        }

        let elapsed = current_timestamp() - self.start_time;
        let total_duration = (self.duration_minutes * 60.0) as u64;

        total_duration.saturating_sub(elapsed)
    }

    pub fn is_finished(&self) -> bool {
        !self.is_paused && self.get_remaining_seconds() == 0
    }

    /// Get the exact timestamp when the timer will finish, or None if paused
    pub fn get_finish_time(&self) -> Option<u64> {
        if self.is_paused {
            None
        } else {
            let total_duration = (self.duration_minutes * 60.0) as u64;
            Some(self.start_time + total_duration)
        }
    }

    pub fn next_phase(
        &mut self,
        sound_config: &SoundConfig,
        notification_config: &NotificationConfig,
        audio_player: Option<&AudioPlayer>,
        hooks_config: &crate::config::HooksConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (message, sound_type, hook_event) = match self.phase {
            Phase::Work => {
                self.current_session_count += 1;

                let (sound_type, hook_event, message) =
                    if self.current_session_count >= self.sessions_until_long_break {
                        self.current_session_count = 0;
                        if self.auto_advance.should_advance(true) {
                            self.start_long_break();
                        } else {
                            self.phase = Phase::LongBreak;
                            self.duration_minutes = self.long_break_duration;
                            self.is_paused = true;
                        }
                        (
                            SoundType::WorkToLongBreak,
                            "long_break_start",
                            &notification_config.long_break_message,
                        )
                    } else {
                        if self.auto_advance.should_advance(true) {
                            self.start_break();
                        } else {
                            self.phase = Phase::Break;
                            self.duration_minutes = self.break_duration;
                            self.is_paused = true;
                        }
                        (
                            SoundType::WorkToBreak,
                            "break_start",
                            &notification_config.work_message,
                        )
                    };

                (message, sound_type, hook_event)
            }
            Phase::Break => {
                if self.auto_advance.should_advance(false) {
                    self.start_work();
                } else {
                    self.phase = Phase::Work;
                    self.duration_minutes = self.work_duration;
                    self.is_paused = true;
                }
                (
                    &notification_config.break_message,
                    SoundType::BreakToWork,
                    "work_start",
                )
            }
            Phase::LongBreak => {
                if self.auto_advance.should_advance(false) {
                    self.start_work();
                } else {
                    self.phase = Phase::Work;
                    self.duration_minutes = self.work_duration;
                    self.is_paused = true;
                }
                (
                    &notification_config.break_message,
                    SoundType::BreakToWork,
                    "work_start",
                )
            }
        };

        // Play sound if enabled and not testing
        if sound_config.enabled
            && !is_testing()
            && let Some(player) = audio_player
        {
            self.play_transition_sound(sound_config, player, sound_type)?;
        }

        // Send notification (existing code)
        if !is_testing() && notification_config.enabled {
            self.send_notification(message, notification_config)?;
        }

        // Execute hook asynchronously only if timer is running (not paused)
        // If paused, store the hook to be executed when user resumes
        if !self.is_paused {
            // Timer is running, execute hook immediately
            // Only spawn async task if we have a Tokio runtime (not in unit tests)
            if tokio::runtime::Handle::try_current().is_ok() {
                let hooks = hooks_config.clone();
                let phase_str = self.phase.to_string();
                let remaining = self.get_remaining_seconds();
                let session_count = self.current_session_count;
                let auto_advance = format!("{:?}", self.auto_advance).to_lowercase();
                let event = hook_event.to_string();

                tokio::spawn(async move {
                    hooks
                        .execute_hook(&event, &phase_str, remaining, session_count, &auto_advance)
                        .await;
                });
            }
        } else {
            // Timer is paused, store hook for later execution on resume
            self.pending_hook = Some(hook_event.to_string());
        }

        Ok(())
    }

    fn play_transition_sound(
        &self,
        config: &SoundConfig,
        player: &AudioPlayer,
        sound_type: SoundType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if config.system_beep {
            player.play_system_beep();
            return Ok(());
        }

        // Check for custom sound file first
        let custom_file = match sound_type {
            SoundType::WorkToBreak => &config.work_to_break,
            SoundType::BreakToWork => &config.break_to_work,
            SoundType::WorkToLongBreak => &config.work_to_long_break,
        };

        if let Some(file_path) = custom_file {
            // Try custom file first
            if let Err(e) = player.play_custom_file(file_path, config.volume) {
                eprintln!("Failed to play custom sound '{}': {}", file_path, e);
                // Fallback to embedded sound
                self.try_embedded_sound(config, player, sound_type)?;
            }
        } else if config.use_embedded {
            // Use embedded sound
            self.try_embedded_sound(config, player, sound_type)?;
        } else {
            // Fallback to system beep
            player.play_system_beep();
        }

        Ok(())
    }

    fn try_embedded_sound(
        &self,
        config: &SoundConfig,
        player: &AudioPlayer,
        sound_type: SoundType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Err(e) = player.play_embedded_sound(sound_type, config.volume) {
            eprintln!("Failed to play embedded sound: {}", e);
            // Final fallback to system beep
            player.play_system_beep();
        }
        Ok(())
    }

    fn send_notification(
        &self,
        message: &str,
        config: &NotificationConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Skip notifications during testing
        if is_testing() {
            return Ok(());
        }

        let mut notification = Notification::new();
        notification
            .summary("Tomat")
            .body(message)
            .timeout(config.timeout as i32)
            .urgency(config.urgency.clone().into());

        // Use configured icon
        match get_notification_icon(config) {
            Ok(icon) => {
                notification.icon(&icon);
            }
            Err(e) => {
                eprintln!(
                    "Warning: Failed to get notification icon: {}, falling back to 'timer'",
                    e
                );
                notification.icon("timer");
            }
        }

        if let Err(e) = notification.show() {
            eprintln!("Failed to send notification: {}", e);
        }

        Ok(())
    }

    pub fn resume(&mut self) -> Option<String> {
        if self.is_paused {
            // If we have stored elapsed time from a pause, restore it
            if let Some(elapsed) = self.paused_elapsed_seconds {
                // Set start_time so that the elapsed time is preserved
                self.start_time = current_timestamp() - elapsed;
                self.paused_elapsed_seconds = None;
            } else {
                // First time starting from paused state
                self.start_time = current_timestamp();
            }
            self.is_paused = false;

            // Return and clear any pending hook
            self.pending_hook.take()
        } else {
            None
        }
    }

    pub fn pause(&mut self) {
        if !self.is_paused {
            // Store elapsed time so we can restore it on resume
            let elapsed = current_timestamp() - self.start_time;
            self.paused_elapsed_seconds = Some(elapsed);
            self.is_paused = true;
        }
    }

    pub fn stop(&mut self) {
        self.phase = Phase::Work;
        self.start_time = 0;
        self.duration_minutes = self.work_duration;
        self.current_session_count = 0;
        self.is_paused = true;
        self.paused_elapsed_seconds = None;
        self.pending_hook = None;
    }

    /// Get raw timer status data for client-side formatting
    pub fn get_timer_status(&self) -> TimerStatus {
        TimerStatus {
            phase: self.phase.clone(),
            is_paused: self.is_paused,
            remaining_seconds: self.get_remaining_seconds(),
            duration_minutes: self.duration_minutes,
            current_session: self.current_session_count + 1,
            sessions_until_long_break: self.sessions_until_long_break,
        }
    }

    /// Convert TimerStatus to StatusOutput with given format and text template
    /// This handles all presentation logic: icons, state symbols, tooltips, CSS classes
    pub fn format_status(
        status: &TimerStatus,
        format: &Format,
        text_template: &str,
    ) -> StatusOutput {
        // Derive presentation data from raw state
        let (icon, phase_name, class) = match status.phase {
            Phase::Work => (
                "🍅",
                "Work",
                if status.is_paused {
                    "work-paused"
                } else {
                    "work"
                },
            ),
            Phase::Break => (
                "☕",
                "Break",
                if status.is_paused {
                    "break-paused"
                } else {
                    "break"
                },
            ),
            Phase::LongBreak => (
                "🏖️",
                "Long Break",
                if status.is_paused {
                    "long-break-paused"
                } else {
                    "long-break"
                },
            ),
        };

        let state_symbol = if status.is_paused { "⏸" } else { "▶" };

        let time_str = format!(
            "{:02}:{:02}",
            status.remaining_seconds / 60,
            status.remaining_seconds % 60
        );

        let session_str = if matches!(status.phase, Phase::Work) {
            format!(
                "{}/{}",
                status.current_session, status.sessions_until_long_break
            )
        } else {
            String::new()
        };

        let sessions_info = if matches!(status.phase, Phase::Work) {
            format!(
                " ({}/{})",
                status.current_session, status.sessions_until_long_break
            )
        } else {
            String::new()
        };

        // Calculate percentage for progress bars
        let total_duration = (status.duration_minutes * 60.0) as u64;
        let elapsed = total_duration.saturating_sub(status.remaining_seconds);
        let percentage = if status.is_paused {
            0.0
        } else if total_duration > 0 {
            (elapsed as f64 / total_duration as f64) * 100.0
        } else {
            100.0
        };

        // Build tooltip
        let tooltip = if status.is_paused {
            format!(
                "{}{} - {:.1}min (Paused)",
                phase_name, sessions_info, status.duration_minutes
            )
        } else {
            format!(
                "{}{} - {:.1}min",
                phase_name, sessions_info, status.duration_minutes
            )
        };

        // Apply text template
        let display_text = text_template
            .replace("{icon}", icon)
            .replace("{time}", &time_str)
            .replace("{state}", state_symbol)
            .replace("{phase}", phase_name)
            .replace("{session}", &session_str);

        match format {
            Format::Waybar => StatusOutput::Waybar {
                text: display_text,
                tooltip,
                class: class.to_string(),
                percentage,
            },
            Format::I3statusRs => {
                // Map timer states to i3status-rs states
                let i3status_state = if status.is_paused {
                    "Info"
                } else {
                    match status.phase {
                        Phase::Work => "Critical",
                        _ => "Good",
                    }
                };

                StatusOutput::I3statusRs {
                    text: display_text.clone(),
                    short_text: Some(display_text),
                    icon: None,
                    state: Some(i3status_state.to_string()),
                }
            }
            Format::Plain => StatusOutput::Plain(display_text),
        }
    }
}

fn is_testing() -> bool {
    std::env::var("TOMAT_TESTING").is_ok()
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_env() {
        // SAFETY: Setting environment variable during tests is safe as tests are run sequentially
        // in the same process and we only set this once per test.
        unsafe {
            std::env::set_var("TOMAT_TESTING", "1");
        }
    }

    #[test]
    fn test_new_timer_starts_in_paused_work_state() {
        let timer = TimerState::new(25.0, 5.0, 15.0, 4);

        assert!(matches!(timer.phase, Phase::Work));
        assert!(timer.is_paused);
        assert_eq!(timer.work_duration, 25.0);
        assert_eq!(timer.break_duration, 5.0);
        assert_eq!(timer.long_break_duration, 15.0);
        assert_eq!(timer.sessions_until_long_break, 4);
        assert_eq!(timer.current_session_count, 0);
        assert_eq!(timer.auto_advance, AutoAdvanceMode::None);
    }

    #[test]
    fn test_start_work_sets_running_state() {
        let mut timer = TimerState::new(25.0, 5.0, 15.0, 4);

        timer.start_work();

        assert!(matches!(timer.phase, Phase::Work));
        assert!(!timer.is_paused);
        assert_eq!(timer.duration_minutes, 25.0);
        assert!(timer.start_time > 0);
    }

    #[test]
    fn test_paused_timer_shows_full_duration() {
        let timer = TimerState::new(25.0, 5.0, 15.0, 4);

        let remaining = timer.get_remaining_seconds();

        assert_eq!(remaining, 25 * 60); // 25 minutes in seconds
    }

    #[test]
    fn test_running_timer_calculates_remaining_time() {
        let mut timer = TimerState::new(25.0, 5.0, 15.0, 4);
        timer.start_work();

        // Timer just started, should have close to full duration
        let remaining = timer.get_remaining_seconds();
        assert!(remaining > 24 * 60 && remaining <= 25 * 60);
    }

    #[test]
    fn test_is_finished_false_when_paused() {
        let timer = TimerState::new(25.0, 5.0, 15.0, 4);

        assert!(!timer.is_finished());
    }

    #[test]
    fn test_is_finished_false_when_time_remaining() {
        let mut timer = TimerState::new(25.0, 5.0, 15.0, 4);
        timer.start_work();

        assert!(!timer.is_finished());
    }

    #[test]
    fn test_resume_unpauses_timer() {
        let mut timer = TimerState::new(25.0, 5.0, 15.0, 4);
        assert!(timer.is_paused);

        timer.resume();

        assert!(!timer.is_paused);
        assert!(timer.start_time > 0);
    }

    #[test]
    fn test_stop_resets_to_paused_work() {
        let mut timer = TimerState::new(25.0, 5.0, 15.0, 4);
        timer.auto_advance = AutoAdvanceMode::All;
        timer.current_session_count = 2;
        timer.start_work();

        timer.stop();

        assert!(matches!(timer.phase, Phase::Work));
        assert!(timer.is_paused);
        assert_eq!(timer.current_session_count, 0);
        assert_eq!(timer.duration_minutes, timer.work_duration);
    }

    #[test]
    fn test_next_phase_work_to_break_auto_advance_false() {
        setup_test_env();
        let mut timer = TimerState::new(25.0, 5.0, 15.0, 4);
        timer.auto_advance = AutoAdvanceMode::None;
        timer.phase = Phase::Work;
        timer.current_session_count = 0;

        timer
            .next_phase(
                &SoundConfig::default(),
                &NotificationConfig::default(),
                None,
                &crate::config::HooksConfig::default(),
            )
            .unwrap();

        assert!(matches!(timer.phase, Phase::Break));
        assert!(timer.is_paused);
        assert_eq!(timer.duration_minutes, 5.0);
        assert_eq!(timer.current_session_count, 1);
    }

    #[test]
    fn test_next_phase_work_to_break_auto_advance_true() {
        setup_test_env();
        let mut timer = TimerState::new(25.0, 5.0, 15.0, 4);
        timer.auto_advance = AutoAdvanceMode::All;
        timer.phase = Phase::Work;
        timer.current_session_count = 0;

        timer
            .next_phase(
                &SoundConfig::default(),
                &NotificationConfig::default(),
                None,
                &crate::config::HooksConfig::default(),
            )
            .unwrap();

        assert!(matches!(timer.phase, Phase::Break));
        assert!(!timer.is_paused);
        assert_eq!(timer.duration_minutes, 5.0);
        assert_eq!(timer.current_session_count, 1);
        assert!(timer.start_time > 0);
    }

    #[test]
    fn test_next_phase_work_to_long_break_after_sessions() {
        setup_test_env();
        let mut timer = TimerState::new(25.0, 5.0, 15.0, 4);
        timer.auto_advance = AutoAdvanceMode::None;
        timer.phase = Phase::Work;
        timer.current_session_count = 3; // Fourth work session

        timer
            .next_phase(
                &SoundConfig::default(),
                &NotificationConfig::default(),
                None,
                &crate::config::HooksConfig::default(),
            )
            .unwrap();

        assert!(matches!(timer.phase, Phase::LongBreak));
        assert!(timer.is_paused);
        assert_eq!(timer.duration_minutes, 15.0);
        assert_eq!(timer.current_session_count, 0); // Reset after long break
    }

    #[test]
    fn test_next_phase_break_to_work() {
        setup_test_env();
        let mut timer = TimerState::new(25.0, 5.0, 15.0, 4);
        timer.auto_advance = AutoAdvanceMode::None;
        timer.phase = Phase::Break;

        timer
            .next_phase(
                &SoundConfig::default(),
                &NotificationConfig::default(),
                None,
                &crate::config::HooksConfig::default(),
            )
            .unwrap();

        assert!(matches!(timer.phase, Phase::Work));
        assert!(timer.is_paused);
        assert_eq!(timer.duration_minutes, 25.0);
    }

    #[test]
    fn test_next_phase_long_break_to_work() {
        setup_test_env();
        let mut timer = TimerState::new(25.0, 5.0, 15.0, 4);
        timer.auto_advance = AutoAdvanceMode::None;
        timer.phase = Phase::LongBreak;

        timer
            .next_phase(
                &SoundConfig::default(),
                &NotificationConfig::default(),
                None,
                &crate::config::HooksConfig::default(),
            )
            .unwrap();

        assert!(matches!(timer.phase, Phase::Work));
        assert!(timer.is_paused);
        assert_eq!(timer.duration_minutes, 25.0);
    }

    #[test]
    fn test_get_status_output_paused_work() {
        let timer = TimerState::new(25.0, 5.0, 15.0, 4);

        let timer_status = timer.get_timer_status();
        let status =
            TimerState::format_status(&timer_status, &Format::default(), "{icon} {time} {state}");

        match status {
            StatusOutput::Waybar {
                text,
                class,
                tooltip,
                percentage,
            } => {
                assert_eq!(text, "🍅 25:00 ⏸");
                assert_eq!(class, "work-paused");
                assert!(tooltip.contains("Work (1/4)"));
                assert!(tooltip.contains("Paused"));
                assert_eq!(percentage, 0.0);
            }
            _ => panic!("Expected Waybar format for default"),
        }
    }

    #[test]
    fn test_get_status_output_running_work() {
        let mut timer = TimerState::new(25.0, 5.0, 15.0, 4);
        timer.start_work();

        let timer_status = timer.get_timer_status();
        let status =
            TimerState::format_status(&timer_status, &Format::default(), "{icon} {time} {state}");

        match status {
            StatusOutput::Waybar {
                text,
                class,
                tooltip,
                percentage,
            } => {
                assert!(text.starts_with("🍅"));
                assert!(text.ends_with("▶"));
                assert_eq!(class, "work");
                assert!(tooltip.contains("Work (1/4)"));
                assert!(!tooltip.contains("Paused"));
                assert!((0.0..=100.0).contains(&percentage));
            }
            _ => panic!("Expected Waybar format for default"),
        }
    }

    #[test]
    fn test_get_status_output_paused_break() {
        let mut timer = TimerState::new(25.0, 5.0, 15.0, 4);
        timer.phase = Phase::Break;
        timer.duration_minutes = 5.0;
        timer.is_paused = true;

        let timer_status = timer.get_timer_status();
        let status =
            TimerState::format_status(&timer_status, &Format::default(), "{icon} {time} {state}");

        match status {
            StatusOutput::Waybar {
                text,
                class,
                tooltip,
                ..
            } => {
                assert_eq!(text, "☕ 05:00 ⏸");
                assert_eq!(class, "break-paused");
                assert!(tooltip.contains("Break"));
                assert!(tooltip.contains("Paused"));
                // Breaks don't show session count (no parentheses for session number)
            }
            _ => panic!("Expected Waybar format for default"),
        }
    }

    #[test]
    fn test_get_status_output_paused_long_break() {
        let mut timer = TimerState::new(25.0, 5.0, 15.0, 4);
        timer.phase = Phase::LongBreak;
        timer.duration_minutes = 15.0;
        timer.is_paused = true;

        let timer_status = timer.get_timer_status();
        let status =
            TimerState::format_status(&timer_status, &Format::default(), "{icon} {time} {state}");

        match status {
            StatusOutput::Waybar {
                text,
                class,
                tooltip,
                ..
            } => {
                assert_eq!(text, "🏖️ 15:00 ⏸");
                assert_eq!(class, "long-break-paused");
                assert!(tooltip.contains("Long Break"));
            }
            _ => panic!("Expected Waybar format for default"),
        }
    }

    #[test]
    fn test_session_count_increments_correctly() {
        setup_test_env();
        let mut timer = TimerState::new(25.0, 5.0, 15.0, 4);
        timer.auto_advance = AutoAdvanceMode::None;
        timer.phase = Phase::Work;

        // Complete 3 work sessions
        for i in 0..3 {
            assert_eq!(timer.current_session_count, i);
            timer
                .next_phase(
                    &SoundConfig::default(),
                    &NotificationConfig::default(),
                    None,
                    &crate::config::HooksConfig::default(),
                )
                .unwrap(); // Work -> Break
            assert!(matches!(timer.phase, Phase::Break));
            timer
                .next_phase(
                    &SoundConfig::default(),
                    &NotificationConfig::default(),
                    None,
                    &crate::config::HooksConfig::default(),
                )
                .unwrap(); // Break -> Work
            assert!(matches!(timer.phase, Phase::Work));
        }

        assert_eq!(timer.current_session_count, 3);

        // Fourth session should trigger long break
        timer
            .next_phase(
                &SoundConfig::default(),
                &NotificationConfig::default(),
                None,
                &crate::config::HooksConfig::default(),
            )
            .unwrap();
        assert!(matches!(timer.phase, Phase::LongBreak));
        assert_eq!(timer.current_session_count, 0); // Reset
    }

    #[test]
    fn test_fractional_minutes() {
        let timer = TimerState::new(0.5, 0.1, 0.2, 4);

        assert_eq!(timer.work_duration, 0.5);
        assert_eq!(timer.break_duration, 0.1);
        assert_eq!(timer.long_break_duration, 0.2);

        let remaining = timer.get_remaining_seconds();
        assert_eq!(remaining, 30); // 0.5 minutes = 30 seconds
    }

    #[test]
    fn test_custom_text_format() {
        let mut timer = TimerState::new(25.0, 5.0, 15.0, 4);
        timer.start_work();

        // Get timer status
        let timer_status = timer.get_timer_status();

        // Test with custom format
        let status =
            TimerState::format_status(&timer_status, &Format::default(), "{time} - {phase}");

        match status {
            StatusOutput::Waybar { text, .. } => {
                assert!(text.contains("25:00"));
                assert!(text.contains("Work"));
                assert!(!text.contains("🍅")); // Icon should not be present
            }
            _ => panic!("Expected Waybar format"),
        }

        // Test with another custom format
        let status =
            TimerState::format_status(&timer_status, &Format::default(), "[{session}] {icon}");

        match status {
            StatusOutput::Waybar { text, .. } => {
                assert!(text.contains("[1/4]"));
                assert!(text.contains("🍅"));
                assert!(!text.contains(":")); // Time should not be present
            }
            _ => panic!("Expected Waybar format"),
        }
    }

    #[test]
    fn test_auto_advance_persists_through_phases() {
        setup_test_env();
        let mut timer = TimerState::new(25.0, 5.0, 15.0, 4);
        timer.auto_advance = AutoAdvanceMode::All;
        timer.phase = Phase::Work;

        // Transition to break
        timer
            .next_phase(
                &SoundConfig::default(),
                &NotificationConfig::default(),
                None,
                &crate::config::HooksConfig::default(),
            )
            .unwrap();
        assert!(!timer.is_paused); // Should still be running

        // Transition back to work
        timer
            .next_phase(
                &SoundConfig::default(),
                &NotificationConfig::default(),
                None,
                &crate::config::HooksConfig::default(),
            )
            .unwrap();
        assert!(!timer.is_paused); // Should still be running
    }

    #[test]
    fn test_pause_preserves_remaining_time() {
        let mut timer = TimerState::new(25.0, 5.0, 15.0, 4);
        timer.start_work();

        // Simulate some time passing (let's say 5 minutes = 300 seconds)
        timer.start_time = current_timestamp() - 300;

        // Get remaining time before pause
        let remaining_before = timer.get_remaining_seconds();
        assert_eq!(remaining_before, 25 * 60 - 300); // Should be 20 minutes

        // Pause the timer using the pause method
        timer.pause();

        // Resume the timer
        timer.resume();

        // Get remaining time after resume
        let remaining_after = timer.get_remaining_seconds();

        // The fix should make this assertion pass
        assert!(
            remaining_after.abs_diff(remaining_before) <= 1,
            "Expected remaining time to be preserved after pause/resume. Before: {}, After: {}",
            remaining_before,
            remaining_after
        );
    }

    #[test]
    fn test_icon_path_creation() {
        // Test that the icon path function works and creates the cache directory
        let icon_path = get_cached_icon_path().expect("Should be able to get icon path");

        // The icon file should exist after calling get_cached_icon_path
        assert!(icon_path.exists(), "Icon file should be created");

        // The icon file should have the correct extension
        assert_eq!(icon_path.extension().unwrap(), "png");

        // The icon file should contain the embedded data
        let file_data = std::fs::read(&icon_path).expect("Should be able to read icon file");
        assert_eq!(
            file_data, ICON_DATA,
            "Icon file should contain the embedded data"
        );

        // Calling get_cached_icon_path again should not change the file
        let icon_path2 = get_cached_icon_path().expect("Should be able to get icon path again");
        assert_eq!(icon_path, icon_path2, "Icon path should be consistent");
    }

    #[test]
    fn test_notification_icon_config() {
        use crate::config::NotificationConfig;

        // Test "auto" mode
        let config = NotificationConfig {
            icon: "auto".to_string(),
            ..Default::default()
        };
        let icon = get_notification_icon(&config).expect("Should get auto icon");
        assert!(
            icon.ends_with("icon.png"),
            "Auto icon should be cached icon path"
        );

        // Test "theme" mode
        let config = NotificationConfig {
            icon: "theme".to_string(),
            ..Default::default()
        };
        let icon = get_notification_icon(&config).expect("Should get theme icon");
        assert_eq!(icon, "timer", "Theme icon should be 'timer'");

        // Test custom path mode (with existing file)
        let temp_icon = std::env::temp_dir().join("test_icon.png");
        std::fs::write(&temp_icon, b"fake icon data").expect("Should create temp icon");

        let config = NotificationConfig {
            icon: temp_icon.to_str().unwrap().to_string(),
            ..Default::default()
        };
        let icon = get_notification_icon(&config).expect("Should get custom icon");
        assert_eq!(
            icon,
            temp_icon.to_str().unwrap(),
            "Custom icon should match path"
        );

        // Clean up
        std::fs::remove_file(&temp_icon).ok();
    }

    #[test]
    fn test_granular_auto_advance_to_break() {
        setup_test_env();
        let mut timer = TimerState::new(25.0, 5.0, 15.0, 4);
        timer.auto_advance = AutoAdvanceMode::ToBreak;
        timer.phase = Phase::Work;

        // Work -> Break should auto-advance
        timer
            .next_phase(
                &SoundConfig::default(),
                &NotificationConfig::default(),
                None,
                &crate::config::HooksConfig::default(),
            )
            .unwrap();

        assert!(matches!(timer.phase, Phase::Break));
        assert!(!timer.is_paused); // Should be running

        // Break -> Work should NOT auto-advance
        timer
            .next_phase(
                &SoundConfig::default(),
                &NotificationConfig::default(),
                None,
                &crate::config::HooksConfig::default(),
            )
            .unwrap();

        assert!(matches!(timer.phase, Phase::Work));
        assert!(timer.is_paused); // Should be paused
    }

    #[test]
    fn test_granular_auto_advance_to_work() {
        setup_test_env();
        let mut timer = TimerState::new(25.0, 5.0, 15.0, 4);
        timer.auto_advance = AutoAdvanceMode::ToWork;
        timer.phase = Phase::Work;

        // Work -> Break should NOT auto-advance
        timer
            .next_phase(
                &SoundConfig::default(),
                &NotificationConfig::default(),
                None,
                &crate::config::HooksConfig::default(),
            )
            .unwrap();

        assert!(matches!(timer.phase, Phase::Break));
        assert!(timer.is_paused); // Should be paused

        // Break -> Work should auto-advance
        timer
            .next_phase(
                &SoundConfig::default(),
                &NotificationConfig::default(),
                None,
                &crate::config::HooksConfig::default(),
            )
            .unwrap();

        assert!(matches!(timer.phase, Phase::Work));
        assert!(!timer.is_paused); // Should be running
    }

    #[test]
    fn test_granular_auto_advance_to_long_break() {
        setup_test_env();
        let mut timer = TimerState::new(25.0, 5.0, 15.0, 4);
        timer.auto_advance = AutoAdvanceMode::ToBreak;
        timer.phase = Phase::Work;
        timer.current_session_count = 3; // Fourth session

        // Work -> Long Break should auto-advance (ToBreak mode)
        timer
            .next_phase(
                &SoundConfig::default(),
                &NotificationConfig::default(),
                None,
                &crate::config::HooksConfig::default(),
            )
            .unwrap();

        assert!(matches!(timer.phase, Phase::LongBreak));
        assert!(!timer.is_paused); // Should be running
    }
}
