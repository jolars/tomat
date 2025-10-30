# Copilot Instructions for tomat

## Repository Overview

**tomat** is a Pomodoro timer with daemon support designed for waybar and other status bars. It's a small Rust project (~800 lines across multiple modules) that implements a server/client architecture using Unix sockets for inter-process communication.

**Key Details:**

- **Language:** Rust (2024 edition)
- **Architecture:** Client/server with Unix socket communication
- **Target:** Linux systems with systemd user services
- **Purpose:** Lightweight Pomodoro timer for waybar integration
- **Dependencies:** Standard Rust ecosystem (tokio, clap, serde, chrono, notify-rust, fs2, rodio)
- **Testing:** Comprehensive integration tests (19 tests covering all functionality)

## Build & Development Environment

### Prerequisites

- Rust stable toolchain (specified in `rust-toolchain.toml`)
- Cargo for building and dependency management
- Optional: Task runner (`go-task`) for development workflows
- Optional: Nix/devenv for reproducible development environment

### Essential Build Commands

**Always run commands from the repository root (`/home/jola/projects/tomat`).**

1. **Quick development check:**

   ```bash
   task dev
   ```

   This runs: `cargo check` → `cargo test` → `cargo clippy --all-targets --all-features -- -D warnings`

2. **Individual commands:**

   ```bash
   # Check compilation without building
   cargo check

   # Run tests (comprehensive integration test suite)
   cargo test

   # Run specific test categories
   cargo test --test cli test_auto_advance    # Auto-advance functionality
   cargo test --test cli test_daemon         # Daemon management

   # Lint with clippy - MUST pass with zero warnings
   cargo clippy --all-targets --all-features -- -D warnings

   # Check code formatting
   cargo fmt -- --check

   # Format code
   cargo fmt
   ```

3. **Build commands:**

   ```bash
   # Development build (fast)
   cargo build

   # Release build (optimized, ~1.2s from clean)
   cargo build --release

   # Clean build (from clean state takes ~10s for dependencies)
   cargo clean && cargo build
   ```

4. **Installation:**

   ```bash
   # Quick install with systemd service setup
   ./install.sh

   # Manual install
   cargo install --path .
   ```

### Pre-commit Validation

**CRITICAL:** All code changes MUST pass these checks before commit:

1. **Formatting:** `cargo fmt -- --check` (MUST exit with code 0)
2. **Linting:** `cargo clippy --all-targets --all-features -- -D warnings` (MUST exit with code 0, no warnings allowed)
3. **Compilation:** `cargo check` (MUST pass)
4. **Tests:** `cargo test` (19 integration tests must pass)

**Pre-commit hooks are configured** in `.pre-commit-config.yaml` and will run clippy and rustfmt automatically if using the Nix devenv.

## Project Layout & Architecture

### File Structure

```
/
├── src/
│   ├── main.rs               # CLI parsing and command dispatching
│   ├── config.rs             # Configuration system (timer, sound, notification settings)
│   ├── server.rs             # Unix socket server, daemon logic, and process management
│   └── timer.rs              # Timer state management, phase transitions, and notification system
├── tests/
│   └── cli.rs                # Integration tests (19 tests)
├── assets/
│   ├── icon.png              # Embedded notification icon
│   └── sounds/               # Embedded audio files
├── Cargo.toml               # Dependencies and metadata, includes cargo-deb config
├── Cargo.lock               # Dependency lockfile
├── Taskfile.yml             # Task runner commands (dev, lint, build-release, test-*)
├── rust-toolchain.toml      # Rust version specification (stable)
├── tomat.service            # Systemd user service file
├── install.sh               # Installation script with systemd setup
├── .github/
│   ├── workflows/
│   │   ├── build-and-test.yml    # CI: tests on Ubuntu/Windows/macOS + security audit
│   │   └── release.yml           # Semantic release workflow
│   └── dependabot.yml            # Dependency updates
├── .pre-commit-config.yaml  # Pre-commit hooks for formatting and linting
├── .releaserc.json          # Semantic release configuration
└── devenv.*                 # Nix development environment files
```

### Code Architecture

The project is organized into four main modules:

- **`main.rs`**: CLI parsing with clap and command dispatching to server/client functions
- **`config.rs`**: Configuration system with timer, sound, and notification settings loaded from TOML
- **`server.rs`**: Unix socket server implementation, client communication handling, daemon process management (PID files, graceful shutdown), timer event loop, and configuration loading
- **`timer.rs`**: Timer state management, phase transitions, status output formatting, desktop notifications with embedded icon system, and auto-advance logic
- **`tests/cli.rs`**: Comprehensive integration tests covering all functionality

**Communication flow:**

- **Single binary** with subcommands: `daemon start|stop|status|run`, `start`, `stop`, `status`, `skip`, `toggle`
- **Daemon mode:** Runs continuously, listens on Unix socket at `$XDG_RUNTIME_DIR/tomat.sock`
- **Client mode:** All other commands send requests to daemon via socket
- **Timer state:** Manages work/break/long-break phases with configurable auto-advance behavior
- **JSON output:** Formatted for waybar consumption with CSS classes and visual indicators (play ▶/pause ⏸ symbols)

### Key Dependencies

- `tokio`: Async runtime for socket handling and timers
- `clap`: Command-line argument parsing with derive macros
- `serde`/`serde_json`: Serialization for client/server communication
- `chrono`: Time handling with serialization support
- `dirs`: Standard directory discovery
- `libc`: Unix user ID access and process management
- `notify-rust`: Desktop notifications for phase transitions
- `fs2`: File locking for daemon instance prevention (prevents race conditions)
- `toml`: Configuration file parsing
- `tempfile` (dev-dependency): Temporary directories for integration tests

## Continuous Integration

### GitHub Actions Workflows

1. **build-and-test.yml** (runs on PR, push to main):
   - **Multi-platform:** Ubuntu, Windows, macOS
   - **Steps:** Build → Test → Clippy → Format check
   - **Security:** RustSec security audit
   - **Caching:** Cargo registry and target directory

2. **release.yml** (manual trigger):
   - **Semantic release** with conventional commits
   - **Automated:** Version bumping, changelog, GitHub releases

### Validation Pipeline

Your changes will be validated against:

1. **Compilation** on all three platforms
2. **Zero clippy warnings** (with `-D warnings` flag)
3. **Proper formatting** (rustfmt)
4. **Security vulnerabilities** (cargo audit)

## Development Workflow

### Making Changes

1. **Start daemon for testing:**

   ```bash
   # Build and start daemon in background (modern approach)
   cargo build && ./target/debug/tomat daemon start

   # Check daemon status
   ./target/debug/tomat daemon status

   # Test client commands with short durations
   ./target/debug/tomat start --work 0.1 --break-time 0.05  # 6s work, 3s break
   ./target/debug/tomat status
   ./target/debug/tomat toggle  # Toggle timer pause/resume

   # Stop daemon when done
   ./target/debug/tomat daemon stop
   ```

2. **Essential validation before commit:**

   ```bash
   # Run full development workflow
   task dev

   # Or individual steps
   cargo fmt
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test  # Runs 19 integration tests
   ```

3. **Test systemd integration:**
   ```bash
   ./install.sh
   systemctl --user status tomat.service
   ```

### Common Gotchas

- **Socket path:** Uses `$XDG_RUNTIME_DIR/tomat.sock` or `/run/user/$UID/tomat.sock`
- **PID files:** Daemon creates `$XDG_RUNTIME_DIR/tomat.pid` for process management
- **Daemon cleanup:** Automatic cleanup of socket and PID files on graceful shutdown
- **Dependencies:** Clean build downloads ~60 crates, takes ~10 seconds
- **Testing:** 19 integration tests validate all functionality including daemon management
- **Systemd:** Service expects `tomat daemon run` command (updated from plain `tomat daemon`)
- **Notifications:** Automatically disabled during testing via `TOMAT_TESTING` environment variable

### Build Timing

- **Incremental build:** ~0.3s
- **Clean build:** ~10s (dependency compilation)
- **Release build:** ~1.2s (optimized compilation)

## Key Implementation Notes

### Timer Behavior

- **No Idle phase:** Timer starts in paused work state, never returns to "idle"
- **Auto-advance:** Configurable via `--auto-advance` flag (default: false)
  - `false`: Timer transitions to next phase but pauses (requires manual resume)
  - `true`: Timer continues automatically through all phases
- **Visual indicators:** Play symbol ▶ when running, pause symbol ⏸ when paused
- **Phase transitions:** Work → Break → Work → ... → Long Break (after N sessions)

### Technical Details

- **Error handling:** Uses `Box<dyn std::error::Error>` for simplicity
- **Communication:** Line-delimited JSON over Unix sockets
- **Timer precision:** 1-second resolution with tokio timers
- **Process management:** SIGTERM → SIGKILL graceful shutdown with 5-second timeout
- **Logging:** Uses `println!`/`eprintln!` for output
- **State persistence:** None - state lost on daemon restart
- **Notifications:** Desktop notifications sent automatically on phase transitions via `notify-rust`
- **Icon system:** Embedded icon with automatic caching to `~/.cache/tomat/icon.png` for mako compatibility
- **Configuration:** TOML-based configuration for timer, sound, and notification settings

### Daemon Management

- **Manual control:** `tomat daemon start|stop|status` for development and user convenience
- **Systemd integration:** `tomat daemon run` for production deployment  
  (Note: systemd service file updated from `tomat daemon` to `tomat daemon run`)
- **Process safety:** PID file tracking with exclusive file locking, duplicate instance prevention, stale file cleanup
- **File locking:** Uses `fs2::FileExt::try_lock_exclusive()` on PID file to prevent race conditions
- **Background operation:** Detached process with stdio redirection

### Status Output Format

The timer provides JSON output optimized for waybar and other status bars:

```json
{
  "class": "work-paused", // CSS class for styling
  "percentage": 0.0, // Progress percentage (0-100)
  "text": "🍅 25:00 ⏸", // Display text with icon and play/pause symbol
  "tooltip": "Work (1/4) - 25.0min (Paused)" // Detailed tooltip information
}
```

**CSS Classes:**

- `work` / `work-paused` - Work session running/paused
- `break` / `break-paused` - Break session running/paused
- `long-break` / `long-break-paused` - Long break running/paused

**Visual Symbols:**

- **Icons:** 🍅 (work), ☕ (break), 🏖️ (long break)
- **State:** ▶ (playing/running), ⏸ (paused)
- **Format:** `{icon} {time} {state_symbol}`

### Configuration System

The application uses a TOML configuration file located at `~/.config/tomat/config.toml` with three main sections:

**Timer Configuration:**
```toml
[timer]
work = 25.0           # Work duration in minutes
break = 5.0          # Break duration in minutes  
long_break = 15.0    # Long break duration in minutes
sessions = 4         # Sessions until long break
auto_advance = false # Auto-continue to next phase
```

**Sound Configuration:**
```toml
[sound]
enabled = true        # Enable sound notifications
system_beep = false  # Use system beep
use_embedded = true  # Use embedded sound files
volume = 0.5         # Volume level (0.0-1.0)
```

**Notification Configuration:**
```toml
[notification]
enabled = true        # Enable desktop notifications
icon = "auto"         # Icon mode: "auto", "theme", or path
timeout = 3000        # Timeout in milliseconds
```

**Icon Modes:**
- `"auto"`: Uses embedded icon, cached to `~/.cache/tomat/icon.png` (mako-compatible)
- `"theme"`: Uses system theme icon (`"timer"`)
- Custom path: e.g., `"/path/to/custom/icon.png"`

### Testing Infrastructure

- **Integration tests:** 19 comprehensive tests covering all functionality
- **Configuration tests:** Validate TOML parsing and defaults for all config sections
- **Icon system tests:** Test embedded icon caching and different icon modes
- **Isolated environments:** Each test uses temporary directories and custom socket paths
- **Timing handling:** Tests use fractional minutes (0.05 = 3 seconds) for fast execution
- **Notification suppression:** Tests automatically disable desktop notifications
- **Daemon lifecycle:** Tests cover start, stop, status, and error conditions

## Trust These Instructions

These instructions have been validated by running all commands and testing the build pipeline. Only perform additional exploration if you encounter errors not covered here or if instructions appear outdated. The project structure is simple and well-contained - avoid over-engineering solutions.
