# Tomat <img src='https://raw.githubusercontent.com/jolars/tomat/refs/heads/main/assets/logo.svg' align="right" width="139" />

[![Build Status](https://github.com/jolars/tomat/workflows/Build%20and%20Test/badge.svg)](https://github.com/jolars/tomat/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Tomat ("tomato" in Swedish 🇸🇪) is a Pomodoro timer with daemon support designed
for waybar and other status bars.

## Features

- **🍅 Pomodoro Technique**: Work/break cycles with configurable durations
- **⚙️ TOML Configuration**: Persistent defaults via XDG config directory
- **⚡ Daemon Architecture**: Robust background service that survives restarts
- **📊 Waybar Integration**: JSON output with CSS classes for seamless integration
- **🎮 Visual Indicators**: Play ▶ and pause ⏸ symbols for clear state indication
- **🔧 Auto-advance Control**: Choose between manual or automatic phase transitions
- **🔄 Process Management**: Built-in daemon start/stop/status commands
- **🖥️ Unix Sockets**: Fast, secure local communication
- **🌙 Systemd Integration**: Auto-start with user session
- **📱 Desktop Notifications**: Phase transition alerts
- **🔊 Sound Notifications**: Audio alerts enabled by default with embedded sounds and customization
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

# Optional: Configure defaults in ~/.config/tomat/config.toml
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

**Note**: Audio will be automatically disabled if ALSA is not available. The timer will still work normally with desktop notifications only.

### Quick Install (Recommended)

```bash
# Install from crates.io
cargo install tomat

# Set up systemd service for auto-start
mkdir -p ~/.config/systemd/user
curl -o ~/.config/systemd/user/tomat.service https://raw.githubusercontent.com/jolars/tomat/main/tomat.service
systemctl --user daemon-reload
systemctl --user enable tomat.service
systemctl --user start tomat.service
```

**Note**: Ensure `~/.cargo/bin` is in your PATH.

### Install from Source

```bash
# Clone and install
git clone https://github.com/jolars/tomat.git
cd tomat
./install.sh
```

This installs the binary and sets up the systemd service automatically.

### Manual Installation

```bash
# Install from crates.io or build from source
cargo install tomat
# OR
cargo install --path .

# Set up systemd service
mkdir -p ~/.config/systemd/user
cp tomat.service ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable tomat.service
systemctl --user start tomat.service
```

**Note**: Ensure `~/.cargo/bin` is in your PATH.

## Configuration

Tomat can be configured using a TOML file located at
`~/.config/tomat/config.toml`. This allows you to set default values for timer
durations and behaviors without specifying them on every command.

### Sound Notifications

By default, tomat plays audio notifications when transitioning between work/break phases:

- **Embedded sounds**: High-quality WAV files built into the application
- **Linux requirement**: Requires ALSA (Advanced Linux Sound Architecture)
- **Automatic fallback**: If audio system unavailable, falls back to system beep or disables audio
- **Customizable**: Override with your own sound files or disable entirely
- **Volume control**: Adjustable volume level (0.0 to 1.0)

To disable audio notifications:

```toml
[sound]
enabled = false
```

To use custom sound files:

```toml
[sound]
work_to_break = "/path/to/your/work-end-sound.wav"
break_to_work = "/path/to/your/break-end-sound.wav"
work_to_long_break = "/path/to/your/long-break-start.wav"
```

### Example Configuration

Create `~/.config/tomat/config.toml`:

```toml
[timer]
work = 25.0          # Work duration in minutes (default: 25)
break_time = 5.0     # Break duration in minutes (default: 5)
long_break = 15.0    # Long break duration in minutes (default: 15)
sessions = 4         # Sessions until long break (default: 4)
auto_advance = false # Auto-advance between phases (default: false)

[sound]
enabled = true       # Enable sound notifications (default: true)
system_beep = false  # Use system beep instead of WAV files (default: false)
use_embedded = true  # Use embedded sounds (default: true)
volume = 0.5         # Volume level 0.0-1.0 (default: 0.5)
# Custom sound files (optional - override embedded sounds)
# work_to_break = "/path/to/work-to-break.wav"
# break_to_work = "/path/to/break-to-work.wav"
# work_to_long_break = "/path/to/work-to-long-break.wav"
```

### Priority Order

Settings are applied in this order (later overrides earlier):

1. **Built-in defaults**: 25min work, 5min break, 15min long break, 4 sessions
2. **Config file**: Values from `~/.config/tomat/config.toml`
3. **CLI arguments**: Flags passed to `tomat start`

### Partial Configuration

You can specify only the values you want to override:

```toml
[timer]
work = 30.0
auto_advance = true
# Other values will use built-in defaults

[sound]
enabled = false  # Disable all sound notifications
# Other sound settings will use built-in defaults
```

## Usage

### Daemon Management

```bash
# Start daemon in background
tomat daemon start

# Check daemon status
tomat daemon status

# Stop daemon
tomat daemon stop
```

### Timer Control

```bash
# Start Pomodoro (25min work, 5min break by default)
tomat start

# Start with custom durations and auto-advance
tomat start --work 30 --break-time 10 --auto-advance

# Toggle pause/resume
tomat toggle

# Skip to next phase
tomat skip

# Stop current session
tomat stop

# Get status (JSON for waybar)
tomat status
```

### Auto-advance Behavior

By default (`--auto-advance=false`):

- Timer transitions to next phase but **pauses**
- You manually resume with `tomat toggle` or `tomat start`
- Gives you control over when breaks start/end

With `--auto-advance=true`:

- Timer automatically continues through all phases
- No manual intervention needed
- Traditional Pomodoro timer behavior

## Waybar Integration

### Configuration

Add to your waybar config:

```json
"custom/pomodoro": {
    "format": "{}",
    "exec": "tomat status",
    "return-type": "json",
    "interval": 1,
    "on-click": "tomat toggle",
    "on-click-right": "tomat skip",
    "on-click-middle": "tomat start"
}
```

### Styling

The status output provides CSS classes for styling:

```css
#custom-pomodoro.work {
  background: #2d5a27;
  color: #ffffff;
}

#custom-pomodoro.work-paused {
  background: #1a3a16;
  color: #cccccc;
}

#custom-pomodoro.break {
  background: #8b4513;
  color: #ffffff;
}

#custom-pomodoro.break-paused {
  background: #5c2e09;
  color: #cccccc;
}

#custom-pomodoro.long-break {
  background: #1e3a8a;
  color: #ffffff;
}

#custom-pomodoro.long-break-paused {
  background: #0f1e5c;
  color: #cccccc;
}
```

## Output Format

The `tomat status` command returns JSON optimized for status bars:

```json
{
  "text": "🍅 24:30 ▶",
  "tooltip": "Work (1/4) - 25.0min",
  "class": "work",
  "percentage": 2.0
}
```

### Visual Indicators

- **Icons**: 🍅 (work), ☕ (break), 🏖️ (long break)
- **State**: ▶ (running), ⏸ (paused)
- **Format**: `{icon} {time} {state}`

### CSS Classes

- `work` / `work-paused` - Work session running/paused
- `break` / `break-paused` - Break session running/paused
- `long-break` / `long-break-paused` - Long break running/paused

## Architecture

```
┌─────────────────┐    Unix Socket     ┌──────────────────┐
│   tomat start   │ ──────────────────▶│   tomat daemon   │
│   tomat status  │                    │                  │
│   tomat toggle  │ ◀──────────────────│  Timer State     │
│   tomat stop    │    JSON Response   │  Notifications   │
└─────────────────┘                    └──────────────────┘
     (Client)                              (Background)
```

- **Daemon**: Runs continuously, manages timer state and notifications
- **Client**: Sends commands via Unix socket (`$XDG_RUNTIME_DIR/tomat.sock`)
- **Persistence**: Timer survives waybar restarts and system suspend/resume

## Examples

### Basic Workflow

```bash
# Start daemon
tomat daemon start

# Begin a Pomodoro session
tomat start --work 25 --break-time 5

# Status shows: 🍅 24:30 ▶
tomat status

# Pause for interruption
tomat toggle
# Status shows: 🍅 24:30 ⏸

# Resume work
tomat toggle
# Status shows: 🍅 24:29 ▶

# When work completes (auto_advance=false default)
# Status shows: ☕ 05:00 ⏸

# Start break manually
tomat toggle
# Status shows: ☕ 04:59 ▶
```

### Automatic Mode

```bash
# Start with auto-advance (no manual resume needed)
tomat start --work 25 --break-time 5 --auto-advance

# Timer automatically flows: Work ▶ → Break ▶ → Work ▶ → ...
# No manual intervention required
```

## Development

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed development documentation.

```bash
# Quick development workflow
cargo build
tomat daemon start
tomat start --work 0.1 --break-time 0.05  # Fast testing
cargo test  # Run 11 integration tests
```

## License

MIT License - see the [LICENSE](LICENSE) file.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make changes with tests
4. Run `cargo test` and `cargo clippy`
5. Submit a pull request

Bug reports and feature requests welcome via [GitHub Issues](https://github.com/jolars/tomat/issues).
