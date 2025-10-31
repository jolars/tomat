# Development Guide

This guide covers contributing to tomat, understanding the architecture, and
development workflows.

## Contributing

We welcome contributions to tomat! This project follows standard open-source
practices with a focus on code quality and thorough testing.

### Quick Contribution Checklist

Before submitting any changes:

- [ ] **Formatting**: `cargo fmt -- --check` (MUST exit with code 0)
- [ ] **Linting**: `cargo clippy --all-targets --all-features -- -D warnings`
      (MUST exit with code 0, no warnings allowed)
- [ ] **Compilation**: `cargo check` (MUST pass)
- [ ] **Tests**: `cargo test` (all integration tests must pass)

### Getting Started

1. **Fork and clone** the repository
2. **Install prerequisites**:

   ```bash
   # Rust toolchain (specified in rust-toolchain.toml)
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

   # ALSA development libraries (for audio)
   sudo apt-get install libasound2-dev  # Ubuntu/Debian
   sudo dnf install alsa-lib-devel      # Fedora/RHEL
   sudo pacman -S alsa-lib              # Arch Linux
   ```

3. **Quick development check**:
   ```bash
   task dev  # Runs: cargo check → cargo test → cargo clippy
   ```

### Development Workflow

#### Essential Build Commands

**Always run commands from the repository root.**

```bash
# Quick development check (recommended)
task dev

# Individual commands
cargo check                    # Check compilation without building
cargo test                     # Run all tests (19 integration tests)
cargo clippy --all-targets --all-features -- -D warnings  # Lint
cargo fmt                      # Format code
cargo fmt -- --check          # Check formatting

# Build commands
cargo build                    # Development build
cargo build --release          # Release build
```

#### Testing Your Changes

```bash
# Build and start daemon for testing
cargo build && ./target/debug/tomat daemon start

# Test with short durations for fast feedback
./target/debug/tomat start --work 0.1 --break 0.05  # 6s work, 3s break
./target/debug/tomat status
./target/debug/tomat toggle

# Stop daemon when done
./target/debug/tomat daemon stop
```

### Code Quality Standards

#### Mandatory Requirements

All code changes MUST pass these checks before commit:

1. **Zero clippy warnings**:
   `cargo clippy --all-targets --all-features -- -D warnings`
2. **Proper formatting**: `cargo fmt -- --check`
3. **All tests pass**: `cargo test`
4. **Compilation success**: `cargo check`

#### Code Style

- **Error handling**: Uses `Box<dyn std::error::Error>` for simplicity
- **Comments**: Only add comments when they match existing style or explain
  complex logic
- **Dependencies**: Use existing libraries when possible, avoid adding new
  dependencies unless absolutely necessary
- **Commit style**: Use
  [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`,
  `docs:`, `test:`, `refactor:`)

### Architecture Overview

Tomat is designed as a small, focused Rust project (~800 lines) with a
client-server architecture.

#### Module Structure

- **`src/main.rs`** (133 lines): CLI parsing with clap, command dispatching
- **`src/config.rs`** (289 lines): Configuration system with timer, sound, and
  notification settings
- **`src/server.rs`** (950 lines): Unix socket server, daemon lifecycle, client
  request handling, PID file management
- **`src/timer.rs`** (870 lines): Timer state machine, phase transitions, status
  output formatting, desktop notifications
- **`tests/cli.rs`** (636 lines): 19 comprehensive integration tests

#### Communication Flow

```
Client CLI Commands
       ↓
Unix Socket ($XDG_RUNTIME_DIR/tomat.sock)
       ↓
Daemon Process (background)
       ↓
TimerState (Work/Break/LongBreak phases)
       ↓
JSON Status Output (optimized for waybar)
```

#### Key Design Decisions

**Client-Server Architecture:**

- Single binary with subcommands: `daemon start|stop|status|run`, `start`,
  `stop`, `status`, `skip`, `toggle`
- Daemon listens on Unix socket at `$XDG_RUNTIME_DIR/tomat.sock`
- PID file tracking at `$XDG_RUNTIME_DIR/tomat.pid` with exclusive file locking
- Line-delimited JSON protocol for communication

**Timer State Machine:**

- Phases: Work → Break → Work → ... → LongBreak (after N sessions)
- Two modes controlled by `--auto-advance` flag:
  - `false` (default): Timer transitions to next phase but **pauses**, requiring
    manual resume
  - `true`: Timer automatically continues through all phases
- Timer starts in paused work state, never returns to "idle"

### Testing Infrastructure

#### Integration Test Pattern

All tests use the `TestDaemon` helper struct for isolated testing:

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

- **Isolated environments**: Each test uses `tempfile::tempdir()` with custom
  `XDG_RUNTIME_DIR`
- **Fast execution**: Fractional minutes (0.05 = 3 seconds) for rapid testing
- **Notification suppression**: `TOMAT_TESTING=1` env var disables desktop
  notifications
- **Automatic cleanup**: `TestDaemon` Drop impl kills daemon process

#### Test Categories

1. **Auto-advance behavior**: Verify `auto_advance=false` pauses after
   transitions, `auto_advance=true` continues automatically
2. **Timer control**: Toggle pause/resume, stop/start
3. **Daemon lifecycle**: Start, stop, status, duplicate detection
4. **Edge cases**: Manual skip, fractional minutes
5. **Configuration**: Timer, sound, and notification configuration parsing
6. **Icon management**: Embedded icon caching and different icon modes

### Adding New Features

#### Adding a New Command

1. Add enum variant to `Commands` in `src/main.rs`
2. Add command handling in `handle_client()` in `src/server.rs`
3. Add match arm in `main()` in `src/main.rs`
4. Write integration tests in `tests/cli.rs` using `TestDaemon`

#### Modifying Timer Behavior

1. Update `TimerState` struct in `src/timer.rs` if new fields needed
2. Modify state machine logic in `next_phase()`, `start_work()`, etc.
3. Update status output in `get_status_output()`
4. Test both `auto_advance=true` and `auto_advance=false` modes

#### Adding Configuration Options

1. Update appropriate config struct in `src/config.rs`
2. Add default value functions
3. Update `Default` implementation
4. Add comprehensive tests for new configuration options
5. Update documentation and examples

### Technical Implementation Details

#### Process Management

- **Daemon lifecycle**: SIGTERM with 5-second timeout, then SIGKILL
- **PID file locking**: Uses `fs2::FileExt::try_lock_exclusive()` to prevent
  race conditions
- **Socket cleanup**: Automatic cleanup of socket and PID files on graceful
  shutdown

#### Notification System

- **Desktop notifications**: Via `notify-rust` with embedded icon system
- **Icon caching**: Embedded icon automatically cached to
  `~/.cache/tomat/icon.png`
- **Mako compatibility**: Default "auto" icon mode works with mako out of the
  box

#### Configuration System

- **TOML-based**: Configuration loaded from `~/.config/tomat/config.toml`
- **Hierarchical**: Built-in defaults → config file → CLI arguments
- **Partial configuration**: Users can override only specific values

### Debugging Tips

```bash
# Run daemon in foreground (see output directly)
cargo build && ./target/debug/tomat daemon run

# Test output (see println! statements)
cargo test -- --nocapture

# Check socket status
ss -lx | grep tomat

# Inspect PID file
cat $XDG_RUNTIME_DIR/tomat.pid && ps -p <PID>

# Check logs (if using systemd)
journalctl --user -u tomat.service -f
```

### Areas for Contribution

Priority areas where contributions are welcome:

- **Performance**: Optimization opportunities
- **Platform support**: Windows/macOS compatibility exploration
- **Error handling**: Better error messages and recovery
- **Testing**: Additional test coverage
- **Integration**: Additional status bar support
- **Documentation**: Examples, tutorials, troubleshooting guides

### Backward Compatibility

When contributing, ensure:

- **No breaking changes**: Existing waybar configurations must continue to work
- **CLI stability**: Existing command-line interfaces are preserved
- **Configuration compatibility**: Existing config files remain valid
- **API consistency**: JSON output format remains stable for waybar integration

### Release Process

The project uses automated semantic versioning:

1. **Conventional Commits**: Commit messages determine version bumps
2. **Automated CI**: GitHub Actions handle testing and releases
3. **Semantic Versioning**: `feat:` → minor, `fix:` → patch, `BREAKING CHANGE:`
   → major

### Getting Help

- **Issues**: Open GitHub issues for bugs or feature requests
- **Discussions**: Use GitHub Discussions for questions
- **Code review**: All pull requests receive thorough review
- **Testing**: Comprehensive CI pipeline ensures quality

The project maintainer is responsive and welcoming to new contributors!

