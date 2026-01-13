# Integration Guide

This guide covers integrating tomat with various status bars and notification
systems.

## Status Bars

### Waybar Integration

Tomat is designed specifically for waybar integration with rich JSON output and
CSS styling support.

#### Basic Configuration

Add this to your waybar config (`~/.config/waybar/config`):

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

#### Styling

Add CSS styling (`~/.config/waybar/style.css`):

```css
#custom-tomat {
  padding: 0 10px;
  margin: 0 5px;
  border-radius: 5px;
}

#custom-tomat.work {
  background-color: #ff6b6b;
  color: #ffffff;
}

#custom-tomat.work-paused {
  background-color: #ff9999;
  color: #ffffff;
}

#custom-tomat.break {
  background-color: #4ecdc4;
  color: #ffffff;
}

#custom-tomat.break-paused {
  background-color: #7dd3db;
  color: #ffffff;
}

#custom-tomat.long-break {
  background-color: #45b7d1;
  color: #ffffff;
}

#custom-tomat.long-break-paused {
  background-color: #74c0db;
  color: #ffffff;
}
```

#### JSON Output Format

Tomat provides waybar-optimized JSON output:

```json
{
  "text": "🍅 24:30 ▶",
  "tooltip": "Work (1/4) - 25.0min",
  "class": "work",
  "percentage": 2.0
}
```

**Fields:**

- **text**: Display text with icon and status symbols
- **tooltip**: Detailed information for hover
- **class**: CSS class for styling
- **percentage**: Progress percentage (0-100)

**CSS Classes:**

- `work` / `work-paused` - Work session running/paused
- `break` / `break-paused` - Break session running/paused
- `long-break` / `long-break-paused` - Long break running/paused

**Visual Indicators:**

- **Icons**: 🍅 (work), ☕ (break), 🏖️ (long break)
- **State**: ▶ (running), ⏸ (paused)

### i3status-rust

Tomat provides native support for i3status-rust with a dedicated output format:

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

The i3status-rs format provides:

- `text`: Display text with timer and status icons
- `short_text`: Same as text (for abbreviated display)
- `state`: Timer state mapping (Critical=work, Good=break, Info=paused)

### i3blocks

i3blocks works perfectly with tomat's existing formats:

#### Simple Integration

```ini
[tomat]
command=tomat status --output plain
interval=1
```

#### With Click Support

```ini
[tomat]
command=tomat status --output plain
interval=1
signal=10
```

Add click handling with environment variables:

```bash
#!/bin/bash
# ~/.config/i3blocks/scripts/tomat-click
case $BLOCK_BUTTON in
    1) tomat toggle ;;     # Left click: toggle
    3) tomat skip ;;       # Right click: skip
esac
pkill -RTMIN+10 i3blocks   # Refresh the block
```

Then set as the command: `command=~/.config/i3blocks/scripts/tomat-click`

### i3bar/i3status

For i3bar integration or i3status, you can use a helper script. First, add this
to your i3status config:

```
order += "read_file tomat"

read_file tomat {
    path = "/tmp/tomat-status"
    format = "%content"
}
```

Helper script:

```bash
#!/bin/bash
while true; do
    tomat status --output plain > /tmp/tomat-status
    sleep 1
done
```

### Polybar

```ini
[module/tomat]
type = custom/script
exec = tomat status --output plain
interval = 1
click-left = tomat toggle
click-right = tomat skip
format-prefix = " "
format-foreground = #ffffff
```

## Systemd Integration

For automatic daemon startup with your desktop session.

### Installation

```bash
# Copy service file
curl -o ~/.config/systemd/user/tomat.service https://raw.githubusercontent.com/jolars/tomat/main/examples/systemd.service

# Enable auto-start
systemctl --user enable tomat.service
systemctl --user start tomat.service
```

### Manual Service File

If you prefer to create the service file manually
(`~/.config/systemd/user/tomat.service`):

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

### Service Management

```bash
# Check status
systemctl --user status tomat.service

# View logs
journalctl --user -u tomat.service -f

# Restart service
systemctl --user restart tomat.service

# Disable auto-start
systemctl --user disable tomat.service
```

## Notification Systems

### Mako (Wayland)

Tomat works perfectly with mako out of the box using the embedded icon system:

```bash
# Default configuration works automatically
# Icon will be cached to ~/.cache/tomat/icon.png
```

For custom mako styling, add to `~/.config/mako/config`:

```ini
[app-name="Tomat"]
background-color=#2d3748
text-color=#ffffff
border-color=#4a5568
default-timeout=5000
```

### Dunst (X11)

For dunst notification daemon:

```bash
# Works with default configuration
# Uses embedded icon or theme icon as fallback
```

Custom dunst rules (`~/.config/dunst/dunstrc`):

```ini
[tomat]
appname = "Tomat"
background = "#2d3748"
foreground = "#ffffff"
timeout = 5000
```

### Custom Notification Icons

To use a custom notification icon:

```toml
# ~/.config/tomat/config.toml
[notification]
icon = "/path/to/your/custom-icon.png"
```

## Hooks Integration

Hooks allow tomat to integrate with external tools and workflows by executing
custom commands on timer events. This enables powerful automation scenarios.

### Common Integration Patterns

#### Media Player Control

Automatically pause/resume music during work sessions:

```toml
# ~/.config/tomat/config.toml
[hooks.on_work_start]
cmd = "playerctl"
args = ["pause"]

[hooks.on_break_start]
cmd = "playerctl"
args = ["play"]
```

Works with any MPRIS-compatible media player (Spotify, VLC, Firefox, etc.).

#### Screen Brightness Control

Adjust brightness based on timer state:

```toml
[hooks.on_pause]
cmd = "brightnessctl"
args = ["set", "30%"]

[hooks.on_resume]
cmd = "brightnessctl"
args = ["set", "100%"]
```

#### Focus Mode with i3/Sway

Enter fullscreen or hide distractions during work:

```toml
[hooks.on_work_start]
cmd = "swaymsg"
args = ["fullscreen", "enable"]

[hooks.on_break_start]
cmd = "swaymsg"
args = ["fullscreen", "disable"]
```

#### Discord/Slack Status

Update your online status automatically:

```toml
[hooks.on_work_start]
cmd = "/home/user/scripts/set-discord-status.sh"
args = ["🍅 Focusing", "dnd"]
cwd = "/home/user/scripts"

[hooks.on_break_start]
cmd = "/home/user/scripts/set-discord-status.sh"
args = ["☕ On break", "idle"]
```

#### Productivity Logging

Track your work sessions in external systems:

```toml
[hooks.on_work_end]
cmd = "sh"
args = ["-c", "echo \"$(date +%Y-%m-%d\\ %H:%M:%S),work,$TOMAT_SESSION_COUNT\" >> ~/productivity.csv"]
capture_output = true
```

Creates a CSV log file: `2026-01-09 14:05:00,work,1`

#### Notification Enhancement

Send notifications to multiple systems:

```toml
[hooks.on_work_start]
cmd = "/home/user/scripts/multi-notify.sh"
args = ["Work started!", "Focus for 25 minutes"]

[hooks.on_long_break_start]
cmd = "notify-send"
args = ["🏖️ Long Break", "Take 15 minutes. You earned it!", "-u", "critical", "-t", "0"]
```

#### Time Tracking Integration

Integrate with external time tracking tools:

```toml
[hooks.on_work_start]
cmd = "watson"
args = ["start", "pomodoro"]

[hooks.on_break_start]
cmd = "watson"
args = ["stop"]
```

Or with Toggl CLI:

```toml
[hooks.on_work_start]
cmd = "toggl"
args = ["start", "Pomodoro session"]

[hooks.on_stop]
cmd = "toggl"
args = ["stop"]
```

### Tips for Hook Integration

1. **Test commands manually first** - Verify commands work before adding to config
2. **Use absolute paths** - More reliable than relying on PATH
3. **Enable capture_output for debugging** - See what went wrong
4. **Set appropriate timeouts** - Longer for network calls, shorter for local commands
5. **Use shell scripts for complex logic** - Keep config simple, put complexity in scripts
6. **Check environment variables** - Scripts can use `$TOMAT_*` variables for context

### Example Integration Script

Create a comprehensive focus mode script:

```bash
#!/bin/bash
# ~/.local/bin/tomat-focus-mode.sh

EVENT="$TOMAT_EVENT"
PHASE="$TOMAT_PHASE"

case "$EVENT" in
    work_start)
        # Enter focus mode
        dunstctl set-paused true           # Pause notifications
        playerctl pause                     # Pause music
        brightnessctl set 100%             # Full brightness
        swaymsg "workspace 1"              # Switch to work workspace
        echo "Focus mode activated"
        ;;
    break_start|long_break_start)
        # Exit focus mode
        dunstctl set-paused false
        brightnessctl set 80%
        echo "Break time!"
        ;;
    pause)
        # Gentle mode
        brightnessctl set 50%
        ;;
    resume)
        brightnessctl set 100%
        ;;
esac
```

Configure it:

```toml
[hooks.on_work_start]
cmd = "/home/user/.local/bin/tomat-focus-mode.sh"
cwd = "/home/user"
timeout = 10

[hooks.on_break_start]
cmd = "/home/user/.local/bin/tomat-focus-mode.sh"

[hooks.on_long_break_start]
cmd = "/home/user/.local/bin/tomat-focus-mode.sh"

[hooks.on_pause]
cmd = "/home/user/.local/bin/tomat-focus-mode.sh"

[hooks.on_resume]
cmd = "/home/user/.local/bin/tomat-focus-mode.sh"
```

See the [Configuration Reference](configuration.md#hooks) for complete hooks documentation.
