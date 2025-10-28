use notify_rust::Notification;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Clone)]
pub struct TimerState {
    pub phase: Phase,
    pub start_time: u64,
    pub duration_minutes: u32,
    pub work_duration: u32,
    pub break_duration: u32,
    pub long_break_duration: u32,
    pub sessions_until_long_break: u32,
    pub current_session_count: u32,
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
    Idle,
}

impl TimerState {
    pub fn new(work: u32, break_time: u32, long_break: u32, sessions: u32) -> Self {
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

    pub fn start_work(&mut self) {
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

    pub fn is_finished(&self) -> bool {
        self.get_remaining_seconds() <= 0 && !matches!(self.phase, Phase::Idle)
    }

    pub fn next_phase(&mut self) -> Result<(), Box<dyn std::error::Error>> {
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

    pub fn stop(&mut self) {
        self.phase = Phase::Idle;
        self.start_time = 0;
        self.duration_minutes = 0;
        self.current_session_count = 0;
    }

    pub fn get_status_output(&self) -> StatusOutput {
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
