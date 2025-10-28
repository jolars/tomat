use notify_rust::Notification;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

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
    pub auto_advance: bool,
    pub is_paused: bool,
}

#[derive(Serialize)]
pub struct StatusOutput {
    text: String,
    tooltip: String,
    class: String,
    percentage: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Phase {
    Work,
    Break,
    LongBreak,
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
            auto_advance: false,
            is_paused: true, // Start in paused state
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

    fn get_remaining_seconds(&self) -> i64 {
        if self.is_paused {
            return (self.duration_minutes * 60.0) as i64;
        }

        let elapsed = current_timestamp() - self.start_time;
        let total_duration = (self.duration_minutes * 60.0) as u64;

        if elapsed >= total_duration {
            0
        } else {
            (total_duration - elapsed) as i64
        }
    }

    pub fn is_finished(&self) -> bool {
        !self.is_paused && self.get_remaining_seconds() <= 0
    }

    pub fn next_phase(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let (message, _icon) = match self.phase {
            Phase::Work => {
                self.current_session_count += 1;

                // Check if it's time for a long break
                if self.current_session_count >= self.sessions_until_long_break {
                    self.current_session_count = 0;
                    if self.auto_advance {
                        self.start_long_break();
                    } else {
                        self.phase = Phase::LongBreak;
                        self.duration_minutes = self.long_break_duration;
                        self.is_paused = true;
                    }
                    ("Long break time! Take a well-deserved rest 🏖️", "🍅")
                } else {
                    if self.auto_advance {
                        self.start_break();
                    } else {
                        self.phase = Phase::Break;
                        self.duration_minutes = self.break_duration;
                        self.is_paused = true;
                    }
                    ("Break time! Take a rest ☕", "🍅")
                }
            }
            Phase::Break => {
                if self.auto_advance {
                    self.start_work();
                } else {
                    self.phase = Phase::Work;
                    self.duration_minutes = self.work_duration;
                    self.is_paused = true;
                }
                ("Back to work! Stay focused 💪", "🍅")
            }
            Phase::LongBreak => {
                if self.auto_advance {
                    self.start_work();
                } else {
                    self.phase = Phase::Work;
                    self.duration_minutes = self.work_duration;
                    self.is_paused = true;
                }
                ("Back to work! You're refreshed and ready 🚀", "🍅")
            }
        };

        // Send desktop notification (synchronous to avoid cross-platform issues)
        // Skip notifications during testing
        if std::env::var("TOMAT_TESTING").is_err()
            && let Err(e) = Notification::new()
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

    pub fn resume(&mut self) {
        if self.is_paused {
            self.start_time = current_timestamp();
            self.is_paused = false;
        }
    }

    pub fn stop(&mut self) {
        self.phase = Phase::Work;
        self.start_time = 0;
        self.duration_minutes = self.work_duration;
        self.current_session_count = 0;
        self.is_paused = true;
    }

    pub fn get_status_output(&self) -> StatusOutput {
        let (icon, class) = match self.phase {
            Phase::Work => (
                "🍅",
                if self.is_paused {
                    "work-paused"
                } else {
                    "work"
                },
            ),
            Phase::Break => (
                "☕",
                if self.is_paused {
                    "break-paused"
                } else {
                    "break"
                },
            ),
            Phase::LongBreak => (
                "🏖️",
                if self.is_paused {
                    "long-break-paused"
                } else {
                    "long-break"
                },
            ),
        };

        if self.is_paused {
            let sessions_info = if matches!(self.phase, Phase::Work) {
                format!(
                    " ({}/{})",
                    self.current_session_count + 1,
                    self.sessions_until_long_break
                )
            } else {
                String::new()
            };

            let phase_name = match self.phase {
                Phase::Work => "Work",
                Phase::Break => "Break",
                Phase::LongBreak => "Long Break",
            };

            // Show the full duration time with a pause indicator
            let time_str = format!(
                "{:02}:{:02}",
                (self.duration_minutes * 60.0) as i64 / 60,
                (self.duration_minutes * 60.0) as i64 % 60
            );

            return StatusOutput {
                text: format!("{} {} ⏸", icon, time_str),
                tooltip: format!(
                    "{}{} - {:.1}min (Paused)",
                    phase_name, sessions_info, self.duration_minutes
                ),
                class: class.to_string(),
                percentage: 0.0,
            };
        }

        let remaining = self.get_remaining_seconds();
        let total_duration = (self.duration_minutes * 60.0) as i64;
        let elapsed = total_duration - remaining;
        let percentage = if total_duration > 0 {
            (elapsed as f64 / total_duration as f64) * 100.0
        } else {
            100.0
        };

        // Always show remaining time with play symbol when running
        let time_str = format!("{:02}:{:02}", remaining / 60, remaining % 60);

        let phase_name = match self.phase {
            Phase::Work => "Work",
            Phase::Break => "Break",
            Phase::LongBreak => "Long Break",
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
            text: format!("{} {} ▶", icon, time_str),
            tooltip: format!(
                "{}{} - {:.1}min",
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
