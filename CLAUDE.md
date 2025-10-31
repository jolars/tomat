# Agent instructions

This file provides guidance to LLM agents on how to work with the `tomat`
project, a Pomodoro timer with daemon support. It includes an overview of the
project, essential commands for building and testing, architectural details,
configuration options, testing patterns, protocol specifications, and common
development tasks.

## Project Overview

**tomat** is a Pomodoro timer with daemon support designed for waybar and other
status bars. It's a small Rust project (~800 lines) implementing a client-server
architecture using Unix sockets for inter-process communication.

## Essential Commands

### Building and Testing

```bash
# Quick development check (recommended)
task dev
# Runs: cargo check → cargo test → cargo clippy --all-targets --all-features -- -D warnings

# Build
cargo build                    # Development build
cargo build --release          # Release build

# Run tests (27 integration tests across 6 modules)
cargo test

# Run specific test categories by module
cargo test --test cli integration::timer      # Timer behavior tests
cargo test --test cli integration::daemon     # Daemon management tests
cargo test --test cli integration::formats    # Output format tests
cargo test --test cli integration::commands   # Command validation tests

# Run with output for debugging
cargo test -- --nocapture
```

### Linting and Formatting

```bash
# CRITICAL: Must pass before commit with zero warnings
cargo clippy --all-targets --all-features -- -D warnings

# Format code
cargo fmt

# Check formatting
cargo fmt -- --check
```

### Manual Testing

```bash
# Build and start daemon
cargo build && ./target/debug/tomat daemon start

# Test with short durations (fractional minutes for fast testing)
./target/debug/tomat start --work 0.1 --break 0.05  # 6s work, 3s break
./target/debug/tomat status
./target/debug/tomat status --output waybar   # JSON output for waybar
./target/debug/tomat status --output plain    # Plain text output
./target/debug/tomat toggle

# Stop daemon
./target/debug/tomat daemon stop
```

## Architecture

### Module Structure

- **`src/main.rs`** (133 lines): CLI parsing with clap, command dispatching
- **`src/config.rs`** (289 lines): Configuration system with timer, sound, and
  notification settings
- **`src/server.rs`** (950 lines): Unix socket server, daemon lifecycle, client
  request handling, PID file management with file locking
- **`src/timer.rs`** (870 lines): Timer state machine, phase transitions, status
  output formatting with Format enum (waybar JSON and plain text), desktop
  notifications with icon management
- **`tests/`**: Modular integration test suite (27 tests across 6 modules)
  - **`cli.rs`**: Integration test entry point
  - **`integration/common.rs`** (171 lines): Shared TestDaemon helper and utilities
  - **`integration/timer.rs`** (300 lines): Timer behavior and auto-advance tests
  - **`integration/daemon.rs`** (88 lines): Daemon lifecycle tests
  - **`integration/formats.rs`** (223 lines): Output format tests
  - **`integration/commands.rs`** (86 lines): Command validation tests

### Communication Flow

```
Client CLI Commands
       ↓
Unix Socket ($XDG_RUNTIME_DIR/tomat.sock)
       ↓
Daemon Process (background)
       ↓
TimerState (Work/Break/LongBreak phases)
       ↓
Status Output
```

### Key Design Decisions

**Client-Server Architecture:**

- Single binary with subcommands: `daemon start|stop|status|run`, `start`,
  `stop`, `status`, `skip`, `toggle`
- Daemon listens on Unix socket at `$XDG_RUNTIME_DIR/tomat.sock`
- PID file tracking at `$XDG_RUNTIME_DIR/tomat.pid` with exclusive file locking
- Line-delimited JSON protocol for communication
- File locking prevents multiple daemon instances and race conditions

**Timer State Machine:**

- Phases: Work → Break → Work → ... → LongBreak (after N sessions)
- Two modes controlled by `--auto-advance` flag:
  - `false` (default): Timer transitions to next phase but **pauses**, requiring
    manual resume
  - `true`: Timer automatically continues through all phases
- Timer starts in paused work state, never returns to "idle"
- Checked every 1 second in daemon loop (see `server.rs:207-228`)

**Auto-advance Implementation:**

- Phase transitions in `timer.rs:90-153` (`next_phase()` method)
- When `auto_advance=false`: calls
  `self.phase = Phase::X; self.is_paused = true`
- When `auto_advance=true`: calls `self.start_X()` which sets
  `self.is_paused = false`
- Manual skip command (`tomat skip`) respects the current auto-advance setting

**Status Output Formats:**

- Supports `waybar` (JSON) and `plain` (text) formats
- Specified via `--output` option on `status` command (e.g.,
  `tomat status --output plain`)
- **Waybar format:** JSON with `text`, `tooltip`, `class`, `percentage` fields
- **Plain format:** Simple text string (e.g., "🍅 24:30 ▶")
- Visual indicators: 🍅 (work), ☕ (break), 🏖️ (long break), ▶ (running), ⏸
  (paused)
- CSS classes (waybar only): `work`, `work-paused`, `break`, `break-paused`,
  `long-break`, `long-break-paused`
- **Note:** Format infrastructure in place to support additional formats
  (polybar, i3bar) in the future

**Notification System:**

- Desktop notifications via `notify-rust` with configurable icons
- Embedded icon system with automatic caching to `~/.cache/tomat/icon.png`
- Three icon modes: `"auto"` (embedded), `"theme"` (system), or custom path
- Configurable timeout and enable/disable options

## Configuration

### Config File Structure

Configuration is loaded from `~/.config/tomat/config.toml`:

```toml
[timer]
work = 25.0           # Work session duration in minutes
break = 5.0          # Break duration in minutes
long_break = 15.0    # Long break duration in minutes
sessions = 4         # Number of work sessions before long break
auto_advance = false # Whether to automatically continue to next phase

[sound]
enabled = true        # Enable sound notifications
system_beep = false  # Use system beep instead of sound files
use_embedded = true  # Use embedded sound files
volume = 0.5         # Volume level (0.0 to 1.0)
# Custom sound files (optional - will override embedded sounds)
# work_to_break = "/path/to/custom/work-to-break.wav"
# break_to_work = "/path/to/custom/break-to-work.wav"
# work_to_long_break = "/path/to/custom/work-to-long-break.wav"

[notification]
enabled = true        # Enable desktop notifications
icon = "auto"         # Icon mode: "auto" (embedded), "theme" (system), or path
timeout = 5000        # Notification timeout in milliseconds
```

### Icon Configuration

The notification system supports three icon modes:

- **`"auto"` (default)**: Uses embedded icon, cached to
  `~/.cache/tomat/icon.png`
- **`"theme"`**: Uses system theme icon (`"timer"`)
- **Custom path**: e.g., `"/path/to/custom/icon.png"`

## Testing

### Integration Test Pattern

All tests use the `TestDaemon` helper struct (`tests/cli.rs:8-148`):

```rust
// Start isolated test daemon with temporary socket
let daemon = TestDaemon::start()?;

// Send commands
daemon.send_command(&["start", "--work", "0.05", "--break", "0.05"])?;

// Get status
let status = daemon.get_status()?;
assert_eq!(status["class"], "work");

// Wait for timer completion
daemon.wait_for_completion(10)?;
```

**Key testing features:**

- **Modular architecture:** Tests organized into logical modules by functionality
- **TestDaemon helper:** Shared utility in `common.rs` for daemon lifecycle management
- Isolated environments: Each test uses `tempfile::tempdir()` with custom
  `XDG_RUNTIME_DIR`
- Fast execution: Fractional minutes (0.05 = 3 seconds) for rapid testing
- Notification suppression: `TOMAT_TESTING=1` env var disables desktop
  notifications
- Automatic cleanup: `TestDaemon` Drop impl kills daemon process

### Test Categories (27 tests total)

1. **Timer behavior** (`integration::timer`): Auto-advance logic, phase transitions, pause/resume
2. **Daemon lifecycle** (`integration::daemon`): Start, stop, status, duplicate detection  
3. **Output formats** (`integration::formats`): Waybar JSON, plain text, i3status-rs
4. **Command validation** (`integration::commands`): Parameter validation, error handling
5. **Configuration** (unit tests): Timer, sound, and notification configuration parsing
6. **Icon management** (unit tests): Embedded icon caching and different icon modes

## Protocol Details

### ClientMessage (client → daemon)

```json
{
  "command": "start" | "stop" | "status" | "skip" | "toggle",
  "args": {
    "work": 25.0,
    "break": 5.0,
    "long_break": 15.0,
    "sessions": 4,
    "auto_advance": false,
    "output": "waybar" | "plain"  // for status command
  }
}
```

### ServerResponse (daemon → client)

```json
{
  "success": true,
  "data": {...},  // StatusOutput for "status" command
  "message": "Timer stopped"
}
```

### StatusOutput

**Waybar format** (JSON object):

```json
{
  "text": "🍅 24:30 ▶",
  "tooltip": "Work (1/4) - 25.0min",
  "class": "work",
  "percentage": 2.0
}
```

**Plain format** (text string):

```
"🍅 24:30 ▶"
```

## Common Development Tasks

### Adding a New Command

1. Add enum variant to `Commands` in `src/main.rs:37-86`
2. Add command handling in `handle_client()` in `src/server.rs:53-172`
3. Add match arm in `main()` in `src/main.rs:92-197`
4. Write integration tests in appropriate module in `tests/integration/`

### Modifying Timer Behavior

1. Update `TimerState` struct in `src/timer.rs:5-17` if new fields needed
2. Modify state machine logic in `next_phase()` (`timer.rs:90-153`),
   `start_work()`, etc.
3. Update status output in `get_status_output()` (`timer.rs:170-270`)
4. Test both `auto_advance=true` and `auto_advance=false` modes

### Adding New Output Formats

1. Add new variant to `Format` enum in `src/timer.rs:11-16`
2. Update `FromStr` implementation to parse new format in `src/timer.rs:18-30`
3. Modify `get_status_output()` to handle format-specific output in
   `src/timer.rs:413+`
4. Update `StatusOutput` struct if new fields are needed for specific formats
5. Update CLI and documentation to reflect newly supported formats
6. Test format parsing and output with various status bar configurations

### Debugging Tips

- Run daemon in foreground: `cargo build && ./target/debug/tomat daemon run`
  (see output directly)
- Test output: `cargo test -- --nocapture` (see println! statements)
- Check socket: `ss -lx | grep tomat` (verify daemon is listening)
- Inspect PID: `cat $XDG_RUNTIME_DIR/tomat.pid` and `ps -p <PID>`

## Important Notes

- **Platform:** Linux-only (uses Unix sockets, libc signals)
- **Error handling:** Uses `Box<dyn std::error::Error>` for simplicity
- **State persistence:** None - timer state is lost on daemon restart
- **Timer precision:** 1-second resolution with tokio::time::sleep
- **Process management:** SIGTERM with 5-second timeout, then SIGKILL
- **Systemd integration:** Service uses `tomat daemon run` command (not just
  `tomat daemon`)
- **Notifications:** Desktop notifications with embedded icon system for mako
  compatibility
- **Icon caching:** Embedded icon automatically cached to
  `~/.cache/tomat/icon.png`
- **Commit style:** Use Conventional Commits (feat:, fix:, docs:, test:,
  refactor:)
- **CI requirements:** Must pass clippy with `-D warnings` (zero warnings
  allowed)

## Dependencies

- `tokio`: Async runtime for socket handling and timers
- `clap`: Command-line argument parsing with derive macros
- `serde`/`serde_json`: Serialization for client/server communication
- `chrono`: Time handling
- `libc`: Unix process management (getuid, kill)
- `notify-rust`: Desktop notifications on phase transitions
- `fs2`: File locking for daemon instance prevention
- `rodio`: Audio playbook for sound notifications (optional)
- `tempfile` (dev): Temporary directories for integration tests
