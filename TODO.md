# TODO

This document tracks known issues and planned improvements for tomat.

## High Priority - Bugs

### Timer Completion Race Condition

**Location:** `src/server.rs:221-226`

**Issue:** Timer checks `is_finished()` only every 1 second in the daemon loop. Phase transitions can be delayed by up to 1 second after the timer actually finishes.

**Impact:** Minor - phase transitions may be ~1 second late.

**Proposed Fix:**

- Calculate exact time until completion
- Use `tokio::time::sleep_until()` with precise timestamp
- Check immediately when timer should finish

## Medium Priority - Improvements

### Add Explicit Pause/Resume Commands

**Location:** `src/cli.rs`, `src/server.rs`

**Issue:**

- Only `toggle` command exists (which pauses/resumes)
- Users may want explicit `pause` and `resume` commands
- More intuitive for scripting and automation

**Proposed Fix:**

- Add `tomat pause` command
- Add `tomat resume` command
- Keep `toggle` for convenience
- Update documentation

### Improve Error Handling in Phase Transitions

**Location:** `src/server.rs:130-131`, `src/server.rs:223-224`

**Issue:** Errors during phase transitions are only logged to stderr, not visible to user.

**Proposed Fix:**

- Consider how to surface notification errors to user
- Maybe add status field for "last error"
- Log to file in addition to stderr

## Low Priority - Features

### History and Statistics Tracking

**Issue:** No way to track completed Pomodoros or view productivity statistics.

**Proposed Features:**

- Store completed Pomodoros in `~/.local/share/tomat/history.json`
- Add `tomat stats` command showing:
  - Pomodoros completed today
  - Pomodoros completed this week/month
  - Average session length
  - Streak tracking
- Integration with waybar tooltip to show daily stats

**Dependencies:**

- Need to decide on data format
- Consider privacy implications (rotation, optional feature?)

### Sound Notifications

**Issue:** Only desktop notifications currently supported, which can be easily missed.

**Proposed Features:**

- Add `--sound` CLI flag or config option
- Support system beep or audio file playback
- Allow custom sound files per phase transition
- Volume control

**Dependencies:** May need to add audio playback crate (e.g., `rodio`).

### Custom Notification Messages

**Issue:** Notification messages are hardcoded in `src/timer.rs:105-136`.

**Proposed Features:**

- Allow customization via config file:
  ```toml
  [notifications]
  work_to_break = "Break time! Step away from the screen."
  break_to_work = "Back to work! Let's get things done."
  work_to_long_break = "Long break! You've earned it."
  ```
- Support notification urgency levels
- Option to disable notifications entirely

## Testing

### Add Stress Tests

Test daemon stability under:

- Rapid command sequences
- Concurrent connections
- Long-running sessions (days/weeks)
- System suspend/resume cycles
