# Integration Guide

This guide covers integrating tomat with various status bars and notification
systems.

## Waybar Integration

Tomat is designed specifically for waybar integration with rich JSON output and
CSS styling support.

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

## Other Status Bars

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

For continuous updates, use the watch command:

```toml
[[block]]
block = "custom"
command = "tomat watch --output i3status-rs --interval 1"
interval = "once"
json = true
```

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

#### Rich Integration with Colors

```ini
[tomat]
command=~/.config/i3blocks/scripts/tomat-rich
interval=1
```

Parser script using waybar JSON:

```bash
#!/bin/bash
# ~/.config/i3blocks/scripts/tomat-rich
OUTPUT=$(tomat status --output waybar 2>/dev/null)
if [ $? -eq 0 ]; then
    TEXT=$(echo "$OUTPUT" | jq -r '.text')
    CLASS=$(echo "$OUTPUT" | jq -r '.class')

    echo "$TEXT"           # Full text
    echo "$TEXT"           # Short text

    # Set color based on timer state
    case "$CLASS" in
        "work") echo "#ff6b6b" ;;
        "work-paused") echo "#ff9999" ;;
        "break") echo "#4ecdc4" ;;
        "break-paused") echo "#7dd3db" ;;
        "long-break") echo "#45b7d1" ;;
        "long-break-paused") echo "#74c0db" ;;
    esac
else
    echo "🍅 Not running"
    echo ""
    echo "#888888"
fi
```

### i3bar/i3status

For direct i3bar integration or i3status, you can use the plain text format:

```bash
# Direct in i3 config
bar {
    status_command exec tomat watch --output plain --interval 1
}
```

Or with i3status using a helper script:

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
