# Overview

**tomat** is a Pomodoro timer with daemon support designed for seamless
integration with waybar and other status bars. It uses a client-daemon
architecture with Unix socket communication to ensure timer state persists
across restarts and system suspend/resume.

## Architecture

Tomat consists of:

- **Daemon** - Runs continuously, manages timer state and notifications
- **Client** - Sends commands via Unix socket at $XDG_RUNTIME_DIR/tomat.sock
- **Notifications** - Desktop alerts and optional sound notifications on phase
  transitions

The timer supports three phases:

- **Work** - Focus session (default: 25 minutes)
- **Break** - Short rest (default: 5 minutes)
- **Long Break** - Extended rest after N work sessions (default: 15 minutes
  after 4 sessions)

## Quick Start

```bash
# Install from crates.io
cargo install tomat

# Start daemon and begin working
tomat daemon start
tomat start

# Check status (perfect for waybar)
tomat status
```

## Installation

### Prerequisites

On Linux systems, audio notifications require ALSA development libraries:

```bash
# Ubuntu/Debian
sudo apt-get install libasound2-dev

# Fedora/RHEL
sudo dnf install alsa-lib-devel

# Arch Linux
sudo pacman -S alsa-lib
```

**Note**: Audio will be automatically disabled if ALSA is not available. The
timer will still work normally with desktop notifications only.

### Install from Crates.io

```bash
cargo install tomat
```

### Quick Setup with Systemd

After installing tomat, you can set up the systemd service with a single
command:

```bash
# Install systemd user service (recommended)
tomat daemon install

# Start the daemon
systemctl --user start tomat.service
```

**Alternative manual setup:**

```bash
# Manual systemd setup (if you prefer)
mkdir -p ~/.config/systemd/user
curl -o ~/.config/systemd/user/tomat.service https://raw.githubusercontent.com/jolars/tomat/main/examples/systemd.service
systemctl --user daemon-reload
systemctl --user enable tomat.service
systemctl --user start tomat.service
```

## Basic Usage

### Start Timer

```bash
# Start with defaults (25min work, 5min break)
tomat start

# Custom durations
tomat start --work 30 --break 10 --long-break 20 --sessions 3

# Auto-advance through all phases
tomat start --auto-advance all

# Auto-advance only from work to break (forced breaks)
tomat start --auto-advance to-break

# Auto-advance only from break to work (self-paced work)
tomat start --auto-advance to-work
```

### Control Timer

```bash
tomat status    # Get current status (JSON for waybar)
tomat watch     # Continuously output status updates
tomat toggle    # Pause/resume timer
tomat skip      # Skip to next phase
tomat stop      # Stop timer and return to idle
```

### Daemon Management

```bash
tomat daemon start     # Start background daemon
tomat daemon stop      # Stop daemon
tomat daemon status    # Check daemon status
tomat daemon install   # Install systemd user service
tomat daemon uninstall # Remove systemd user service
```

## Uninstall

To completely remove tomat:

```bash
# Stop and remove systemd service
tomat daemon uninstall

# Remove the binary
cargo uninstall tomat

# Remove configuration (optional)
rm -rf ~/.config/tomat
```
