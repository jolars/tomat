# Overview

**tomat** is a Pomodoro timer with daemon support designed for seamless integration with waybar and other status bars. It uses a client-daemon architecture with Unix socket communication to ensure timer state persists across restarts and system suspend/resume.

## Architecture

Tomat consists of:

- **Daemon** - Runs continuously, manages timer state and notifications
- **Client** - Sends commands via Unix socket at $XDG_RUNTIME_DIR/tomat.sock
- **Notifications** - Desktop alerts and optional sound notifications on phase transitions

The timer supports three phases:

- **Work** - Focus session (default: 25 minutes)
- **Break** - Short rest (default: 5 minutes)
- **Long Break** - Extended rest after N work sessions (default: 15 minutes after 4 sessions)

## Quick Start

## Installation

```bash
# Install from crates.io
cargo install tomat

# Install systemd user service
tomat daemon install
systemctl --user start tomat.service
```

## Basic Usage

```bash
# Start daemon (if not using systemd)
tomat daemon start

# Begin a Pomodoro session
tomat start

# Check status
tomat status

# Toggle pause/resume
tomat toggle

# Skip to next phase
tomat skip

# Stop timer
tomat stop
```

## Custom Sessions

```bash
# Custom durations
tomat start --work 45 --break 15

# Auto-advance through all phases
tomat start --auto-advance all

# Enforced breaks (auto-advance only to break)
tomat start --auto-advance to-break

# Self-paced work (auto-advance only to work)
tomat start --auto-advance to-work
```

# WAYBAR INTEGRATION

## Configuration

Add to **~/.config/waybar/config**:

```json
{
  "modules-right": ["custom/tomat"],
  "custom/tomat": {
    "exec": "tomat status",
    "interval": 1,
    "return-type": "json",
    "format": "{}",
    "on-click": "tomat toggle",
    "on-click-right": "tomat skip"
  }
}
```

## Continuous Updates (Alternative)

For reduced CPU usage, use the watch command:

```json
{
  "custom/tomat": {
    "exec": "tomat watch --interval 1",
    "return-type": "json",
    "format": "{}",
    "on-click": "tomat toggle",
    "on-click-right": "tomat skip"
  }
}
```

## Styling

Add to **~/.config/waybar/style.css**:

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

## JSON Output

Tomat provides waybar-optimized JSON:

```json
{
  "text": "🍅 24:30 ▶",
  "tooltip": "Work (1/4) - 25.0min",
  "class": "work",
  "percentage": 2.0
}
```

# OTHER STATUS BARS

## i3status-rust

```toml
[[block]]
block = "custom"
command = "tomat status --output i3status-rs"
interval = 1
json = true

[[block.click]]
button = "left"
cmd = "tomat toggle"

[[block.click]]
button = "right"
cmd = "tomat skip"
```

## Polybar

```ini
[module/tomat]
type = custom/script
exec = tomat status --output plain
interval = 1
click-left = tomat toggle
click-right = tomat skip
```

## i3blocks

```ini
[tomat]
command=tomat status --output plain
interval=1
```

# SYSTEMD INTEGRATION

## Automatic Startup

```bash
# Install service
tomat daemon install

# Enable and start
systemctl --user enable tomat.service
systemctl --user start tomat.service
```

## Service Management

```bash
# Check status
systemctl --user status tomat.service

# View logs
journalctl --user -u tomat.service -f

# Restart
systemctl --user restart tomat.service

# Disable
systemctl --user disable tomat.service
```

## Manual Service File

Create **~/.config/systemd/user/tomat.service**:

```ini
[Unit]
Description=Tomat Pomodoro Timer Daemon
After=graphical-session.target

[Service]
Type=simple
ExecStart=%h/.cargo/bin/tomat daemon run
Restart=on-failure
RestartSec=5

[Install]
WantedBy=default.target
```

# NOTIFICATION SYSTEMS

## Mako (Wayland)

Works out of the box with embedded icon. Optional custom styling in **~/.config/mako/config**:

```ini
[app-name="Tomat"]
background-color=#2d3748
text-color=#ffffff
border-color=#4a5568
default-timeout=5000
```

## Dunst (X11)

Works with default configuration. Optional custom rules in **~/.config/dunst/dunstrc**:

```ini
[tomat]
appname = "Tomat"
background = "#2d3748"
foreground = "#ffffff"
timeout = 5000
```

# HOOKS AND AUTOMATION

## Media Player Control

```toml
[hooks.on_work_start]
cmd = "playerctl"
args = ["pause"]

[hooks.on_break_start]
cmd = "playerctl"
args = ["play"]
```

## Screen Brightness

```toml
[hooks.on_pause]
cmd = "brightnessctl"
args = ["set", "30%"]

[hooks.on_resume]
cmd = "brightnessctl"
args = ["set", "100%"]
```

## Focus Mode (i3/Sway)

```toml
[hooks.on_work_start]
cmd = "swaymsg"
args = ["fullscreen", "enable"]

[hooks.on_break_start]
cmd = "swaymsg"
args = ["fullscreen", "disable"]
```

## Productivity Logging

```toml
[hooks.on_complete]
cmd = "sh"
args = ["-c", "echo \"$(date): $TOMAT_PHASE completed\" >> ~/tomat.log"]
capture_output = true
```

## Custom Focus Script

Create **~/.local/bin/tomat-focus.sh**:

```bash
#!/bin/bash
case "$TOMAT_EVENT" in
    work_start)
        dunstctl set-paused true
        playerctl pause
        brightnessctl set 100%
        ;;
    break_start|long_break_start)
        dunstctl set-paused false
        brightnessctl set 80%
        ;;
esac
```

Configure:

```toml
[hooks.on_work_start]
cmd = "/home/user/.local/bin/tomat-focus.sh"

[hooks.on_break_start]
cmd = "/home/user/.local/bin/tomat-focus.sh"

[hooks.on_long_break_start]
cmd = "/home/user/.local/bin/tomat-focus.sh"
```

# TROUBLESHOOTING

## Daemon Issues

**Daemon won't start:**

```bash
# Check if already running
tomat daemon status

# Check socket file
ls -l $XDG_RUNTIME_DIR/tomat.sock

# View systemd logs
journalctl --user -u tomat.service -n 50
```

**Multiple instances:**

```bash
# Stop daemon
tomat daemon stop

# Clean up stale files
rm -f $XDG_RUNTIME_DIR/tomat.sock
rm -f $XDG_RUNTIME_DIR/tomat.pid
```

## Configuration Issues

**Config not loading:**

```bash
# Verify file location
ls -l ~/.config/tomat/config.toml

# Check syntax with TOML validator
# Ensure proper quoting and bracket matching
```

**Partial config not working:**

Only specify values you want to override. Unspecified options use defaults.

## Audio Issues

**No sound:**

On Linux, audio requires ALSA development libraries:

```bash
# Ubuntu/Debian
sudo apt-get install libasound2-dev

# Fedora/RHEL
sudo dnf install alsa-lib-devel

# Arch Linux
sudo pacman -S alsa-lib
```

**Custom sounds not working:**

Verify WAV file format:

```bash
file /path/to/sound.wav
```

## Notification Issues

**No notifications:**

```bash
# Test notification daemon
notify-send "Test" "Notification test"

# Check tomat config
[notification]
enabled = true
```

**Icon not showing:**

Try different icon modes:

```toml
[notification]
icon = "auto"    # or "theme" or "/path/to/icon.png"
```

## Waybar Issues

**Module not updating:**

- Check daemon is running: `tomat daemon status`
- Verify waybar config syntax
- Test command manually: `tomat status`
- Check waybar logs: `journalctl --user -u waybar.service -f`

**Wrong output format:**

Ensure `return-type` is set to `"json"` in waybar config.

# FILES

**~/.config/tomat/config.toml**
:   Configuration file

**~/.config/systemd/user/tomat.service**
:   Systemd user service file

**~/.cache/tomat/icon.png**
:   Cached notification icon

**$XDG_RUNTIME_DIR/tomat.sock**
:   Unix socket for client-daemon communication

**$XDG_RUNTIME_DIR/tomat.pid**
:   Daemon PID file

# EXAMPLES

## Minimal Workflow

```bash
# One-time setup
cargo install tomat
tomat daemon install
systemctl --user start tomat.service

# Daily usage
tomat start          # Begin work session
tomat status         # Check time remaining
tomat toggle         # Pause/resume
tomat skip           # Move to break early
```

## Power User Setup

Configuration file with hooks:

```toml
[timer]
work = 45.0
break = 15.0
auto_advance = "to-break"

[sound]
enabled = true
volume = 0.7

[display]
text_format = "[{session}] {icon} {time}"

[hooks.on_work_start]
cmd = "/home/user/scripts/focus-mode.sh"
args = ["enable"]

[hooks.on_break_start]
cmd = "/home/user/scripts/focus-mode.sh"
args = ["disable"]
```

Waybar with custom format:

```json
{
  "custom/tomat": {
    "exec": "tomat watch",
    "return-type": "json",
    "on-click": "tomat toggle",
    "on-click-middle": "tomat stop",
    "on-click-right": "tomat skip"
  }
}
```

## Additional Resources

- [CLI Reference](cli-reference.md) - Complete command-line reference
- [Configuration Reference](configuration.md) - Detailed configuration options
- Project homepage: https://github.com/jolars/tomat
