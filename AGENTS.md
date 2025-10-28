# Copilot Instructions for tomat

## Repository Overview

**tomat** is a Pomodoro timer with daemon support designed for waybar and other status bars. It's a small Rust project (~580 lines across multiple modules) that implements a server/client architecture using Unix sockets for inter-process communication.

**Key Details:**

- **Language:** Rust (2024 edition)
- **Architecture:** Client/server with Unix socket communication
- **Target:** Linux systems with systemd user services
- **Purpose:** Lightweight Pomodoro timer for waybar integration
- **Dependencies:** Standard Rust ecosystem (tokio, clap, serde, chrono)

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

   # Run tests (currently no tests in codebase)
   cargo test

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
4. **Tests:** `cargo test` (currently no tests, but command must succeed)

**Pre-commit hooks are configured** in `.pre-commit-config.yaml` and will run clippy and rustfmt automatically if using the Nix devenv.

## Project Layout & Architecture

### File Structure

```
/
├── src/
│   ├── main.rs               # CLI parsing and command dispatching (163 lines)
│   ├── server.rs             # Unix socket server and daemon logic (215 lines)
│   └── timer.rs              # Timer state management and phase transitions (201 lines)
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

The project is organized into three main modules:

- **`main.rs`**: CLI parsing with clap and command dispatching to server/client functions
- **`server.rs`**: Unix socket server implementation, client communication handling, and daemon event loop
- **`timer.rs`**: Timer state management, phase transitions, status output formatting, and desktop notifications

**Communication flow:**
- **Single binary** with subcommands: `daemon`, `start`, `stop`, `status`, `skip`, `toggle`
- **Daemon mode:** Runs continuously, listens on Unix socket at `$XDG_RUNTIME_DIR/tomat.sock`
- **Client mode:** All other commands send requests to daemon via socket
- **Timer state:** Manages work/break phases with automatic transitions
- **JSON output:** Formatted for waybar consumption with CSS classes

### Key Dependencies

- `tokio`: Async runtime for socket handling and timers
- `clap`: Command-line argument parsing with derive macros
- `serde`/`serde_json`: Serialization for client/server communication
- `chrono`: Time handling with serialization support
- `dirs`: Standard directory discovery
- `libc`: Unix user ID access
- `notify-rust`: Desktop notifications for phase transitions

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
   # Build and run daemon in background
   cargo build && ./target/debug/tomat daemon &

   # Test client commands
   ./target/debug/tomat status
   ./target/debug/tomat start --work 1 --break-time 1  # Short durations for testing
   ./target/debug/tomat toggle  # Toggle timer on/off
   ```

2. **Essential validation before commit:**

   ```bash
   # Run full development workflow
   task dev

   # Or individual steps
   cargo fmt
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test
   ```

3. **Test systemd integration:**
   ```bash
   ./install.sh
   systemctl --user status tomat.service
   ```

### Common Gotchas

- **Socket path:** Uses `$XDG_RUNTIME_DIR/tomat.sock` or `/run/user/$UID/tomat.sock`
- **Daemon cleanup:** Remove socket file on startup in case of unclean shutdown
- **Dependencies:** Clean build downloads ~60 crates, takes ~10 seconds
- **No tests:** Project currently has no unit tests, but `cargo test` must still pass
- **Systemd:** Service expects `tomat` binary in PATH (typically `~/.cargo/bin`)

### Build Timing

- **Incremental build:** ~0.3s
- **Clean build:** ~10s (dependency compilation)
- **Release build:** ~1.2s (optimized compilation)

## Key Implementation Notes

- **Error handling:** Uses `Box<dyn std::error::Error>` for simplicity
- **Communication:** Line-delimited JSON over Unix sockets
- **Timer precision:** 1-second resolution with tokio timers
- **Signal handling:** None implemented (relies on systemd restart)
- **Logging:** Uses `println!`/`eprintln!` for output
- **State persistence:** None - state lost on daemon restart
- **Notifications:** Desktop notifications sent automatically on phase transitions via `notify-rust`

## Trust These Instructions

These instructions have been validated by running all commands and testing the build pipeline. Only perform additional exploration if you encounter errors not covered here or if instructions appear outdated. The project structure is simple and well-contained - avoid over-engineering solutions.
