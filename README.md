# Tomat <img src='https://raw.githubusercontent.com/jolars/tomat/refs/heads/main/images/logo.svg' align="right" width="139" />

[![Build Status](https://github.com/jolars/tomat/workflows/Build%20and%20Test/badge.svg)](https://github.com/jolars/tomat/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/crates/v/tomat.svg)](https://crates.io/crates/tomat)

Tomat ("tomato" in Swedish 🇸🇪) is a Pomodoro timer with daemon support designed
for waybar and other status bars.

## Features

- **🍅 Pomodoro Technique**: Work/break cycles with configurable durations
- **⚙️ TOML Configuration**: Persistent defaults via XDG config directory
- **⚡ Daemon Architecture**: Robust background service that survives restarts
- **📊 Waybar Integration**: JSON output with CSS classes for seamless
  integration
- **🎮 Visual Indicators**: Play ▶ and pause ⏸ symbols for clear state
  indication
- **🔧 Auto-advance Control**: Choose between manual or automatic phase
  transitions
- **🔄 Process Management**: Built-in daemon start/stop/status commands
- **🖥️ Unix Sockets**: Fast, secure local communication
- **🌙 Systemd Integration**: Auto-start with user session
- **📱 Desktop Notifications**: Phase transition alerts with configurable icons
- **🖼️ Icon System**: Embedded icon with mako compatibility and custom icon
  support
- **🔊 Sound Notifications**: Audio alerts with embedded sounds and
  customization
- **💾 Minimal Resources**: Lightweight and efficient

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

```bash
# Set up systemd service for auto-start
mkdir -p ~/.config/systemd/user
curl -o ~/.config/systemd/user/tomat.service https://raw.githubusercontent.com/jolars/tomat/main/tomat.service
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

# Auto-advance between phases
tomat start --auto-advance
```

### Control Timer

```bash
tomat status    # Get current status (JSON for waybar)
tomat toggle    # Pause/resume timer
tomat skip      # Skip to next phase
tomat stop      # Stop timer and return to idle
```

### Daemon Management

```bash
tomat daemon start    # Start background daemon
tomat daemon stop     # Stop daemon
tomat daemon status   # Check daemon status
```

## Configuration

Create `~/.config/tomat/config.toml` to customize defaults:

```toml
[timer]
work = 25.0          # Work duration in minutes
break = 5.0          # Break duration in minutes
long_break = 15.0    # Long break duration in minutes
sessions = 4         # Sessions until long break
auto_advance = false # Auto-continue to next phase

[sound]
enabled = true       # Enable sound notifications
volume = 0.5         # Volume level (0.0-1.0)

[notification]
enabled = true       # Enable desktop notifications
icon = "auto"        # Icon mode: "auto", "theme", or custom path
timeout = 10000      # Notification timeout in milliseconds
```

**💡 Tip**: Copy the complete example config:

```bash
mkdir -p ~/.config/tomat
cp examples/config.toml ~/.config/tomat/config.toml
# Edit as needed
```

## Waybar Integration

Add to your waybar config (`~/.config/waybar/config`):

```json
{
  "modules-right": ["custom/tomat"],
  "custom/tomat": {
    "exec": "tomat status",
    "interval": 1,
    "return-type": "json",
    "format": "{text}",
    "tooltip": true,
    "on-click": "tomat toggle",
    "on-click-right": "tomat skip"
  }
}
```

Add CSS styling (`~/.config/waybar/style.css`):

```css
#custom-tomat.work {
  background-color: #ff6b6b;
}
#custom-tomat.work-paused {
  background-color: #ff9999;
}
#custom-tomat.break {
  background-color: #4ecdc4;
}
#custom-tomat.break-paused {
  background-color: #7dd3db;
}
#custom-tomat.long-break {
  background-color: #45b7d1;
}
#custom-tomat.long-break-paused {
  background-color: #74c0db;
}
```

**💡 Tip**: See [`examples/`](examples/) for complete waybar config and styling
examples.

## JSON Output

Tomat provides waybar-optimized JSON output:

```json
{
  "text": "🍅 24:30 ▶",
  "tooltip": "Work (1/4) - 25.0min",
  "class": "work",
  "percentage": 2.0
}
```

**Visual Indicators:**

- **Icons**: 🍅 (work), ☕ (break), 🏖️ (long break)
- **State**: ▶ (running), ⏸ (paused)
- **CSS Classes**: `work`, `work-paused`, `break`, `break-paused`, `long-break`,
  `long-break-paused`

## Documentation

For detailed guides and advanced configuration:

- **[📋 Documentation Index](https://github.com/jolars/tomat/blob/main/docs/index.md)** -
  Complete documentation overview
- **[📁 Examples](https://github.com/jolars/tomat/tree/main/examples)** -
  Ready-to-use configurations (waybar, systemd, etc.)
- **[📖 Configuration Guide](https://github.com/jolars/tomat/blob/main/docs/configuration.md)** -
  Complete configuration options
- **[🔗 Integration Guide](https://github.com/jolars/tomat/blob/main/docs/integration.md)** -
  Waybar, systemd, and notification setup
- **[👨‍💻 Development Guide](https://github.com/jolars/tomat/blob/main/docs/development.md)** -
  Contributing and architecture
- **[🐛 Troubleshooting](https://github.com/jolars/tomat/blob/main/docs/troubleshooting.md)** -
  Common issues and solutions

## Examples

### Basic Workflow

```bash
# One-time setup
cargo install tomat

# Daily usage
tomat daemon start
tomat start          # Begin 25min work session
# ... work on your task ...
tomat status         # Check remaining time
tomat toggle         # Take a quick pause
tomat skip           # Move to break early
# ... enjoy your break ...
# Timer automatically suggests when to return to work
```

### Custom Sessions

```bash
# Long focus session
tomat start --work 45 --break 15

# Sprint session
tomat start --work 15 --break 5 --auto-advance

# Deep work (no interruptions)
tomat start --work 90 --break 30 --sessions 2
```

### Integration Examples

```bash
# Check if currently working
tomat status | jq -r '.class' | grep -q work && echo "Focus time!"

# Get remaining time
tomat status | jq -r '.tooltip'

# Waybar click handlers
tomat toggle    # Left click to pause/resume
tomat skip      # Right click to skip phase
```

## Architecture

```
Client Commands  →  Unix Socket  →  Daemon Process  →  Timer State  →  JSON Output
     ↓                  ↓               ↓              ↓              ↓
tomat start      $XDG_RUNTIME_DIR/  Background     Work/Break/    {"text": "🍅 25:00 ▶",
tomat status     tomat.sock         Service        LongBreak       "class": "work"}
tomat toggle                                       Phases
```

- **Daemon**: Runs continuously, manages timer state and notifications
- **Client**: Sends commands via Unix socket for fast communication
- **Persistence**: Timer survives waybar restarts and system suspend/resume
- **Notifications**: Desktop alerts and optional sound notifications on phase
  transitions

## License

MIT License - see [LICENSE](https://github.com/jolars/tomat/blob/main/LICENSE)
for details.

## Contributing

Contributions welcome! See the
[Development Guide](https://github.com/jolars/tomat/blob/main/docs/DEVELOPMENT.md)
for details on:

- Setting up the development environment
- Code quality standards
- Testing infrastructure
- Architecture overview

---

**Happy focusing! 🍅**
