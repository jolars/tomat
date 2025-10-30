# Integration Guide

This guide covers integrating tomat with various status bars, notification systems, and desktop environments.

## Waybar Integration

Tomat is designed specifically for waybar integration with rich JSON output and CSS styling support.

### Basic Waybar Configuration

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

### Waybar Styling

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

### JSON Output Format

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

## Systemd Integration

For automatic daemon startup with your desktop session.

### Installation

```bash
# Copy service file
curl -o ~/.config/systemd/user/tomat.service https://raw.githubusercontent.com/jolars/tomat/main/tomat.service

# Enable auto-start
systemctl --user enable tomat.service
systemctl --user start tomat.service
```

### Manual Service File

If you prefer to create the service file manually (`~/.config/systemd/user/tomat.service`):

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
default-timeout=3000
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
timeout = 3000
```

### Custom Notification Icons

To use a custom notification icon:

```toml
# ~/.config/tomat/config.toml
[notification]
icon = "/path/to/your/custom-icon.png"
```

## Other Status Bars

### Polybar

```ini
[module/tomat]
type = custom/script
exec = tomat status 2>/dev/null | jq -r '.text // "🍅 Not running"'
interval = 1
click-left = tomat toggle
click-right = tomat skip
format-prefix = " "
format-foreground = #ffffff
```

### i3status-rust

```toml
[[block]]
block = "custom"
command = "tomat status 2>/dev/null | jq -r '.text // \"🍅 Not running\"'"
interval = 1
click = [
  { button = "left"; cmd = "tomat toggle"; },
  { button = "right"; cmd = "tomat skip"; }
]
```

### i3status

```
order += "read_file tomat"

read_file tomat {
    path = "/tmp/tomat-status"
    format = "%content"
}
```

With a helper script that runs periodically:

```bash
#!/bin/bash
# Update tomat status for i3status
tomat status 2>/dev/null | jq -r '.text // "🍅 Not running"' > /tmp/tomat-status
```

## Desktop Environment Integration

### GNOME

For GNOME Shell integration, tomat works with:
- **Desktop notifications**: Via `notify-rust`
- **System tray**: Via waybar or other status bars
- **Keyboard shortcuts**: Set up custom shortcuts for `tomat toggle`, `tomat skip`

### KDE Plasma

Tomat integrates well with KDE:
- **Panel widgets**: Use command output widget with `tomat status`
- **Notifications**: Works with KDE notification system
- **Global shortcuts**: Set up via System Settings

### Window Managers (i3, sway, etc.)

Perfect for minimal setups:
- **Status bars**: Waybar, polybar, i3status
- **Keybindings**: Direct tomat commands
- **Notifications**: Works with any notification daemon

## Audio System Integration

### ALSA (Default)

Works out of the box on most Linux systems:

```bash
# Check ALSA availability
aplay -l
```

### PulseAudio/PipeWire

Tomat works through ALSA compatibility layer:

```bash
# Usually works automatically
# No additional configuration needed
```

### Troubleshooting Audio

1. **No sound**: Check ALSA installation and permissions
2. **Wrong device**: Audio automatically uses default ALSA device
3. **Volume issues**: Adjust system volume and tomat volume in config

## Development Integration

### Editor Integration

Use tomat with your development workflow:

```bash
# Start timer when beginning work
alias work="tomat start && echo 'Focus time! 🍅'"

# Quick status check
alias focus="tomat status | jq -r '.tooltip'"
```

### Git Hooks

Integrate with git workflow:

```bash
# .git/hooks/pre-commit
#!/bin/bash
# Pause timer during commits
tomat toggle 2>/dev/null || true
```

### IDE Plugins

While tomat doesn't have official IDE plugins, you can integrate via:
- **Terminal commands**: Most IDEs support terminal integration
- **Status bar**: Display tomat status in IDE status bar
- **Notifications**: Desktop notifications work across all applications