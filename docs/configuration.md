# Configuration Guide

Tomat supports comprehensive configuration via a TOML file located at `~/.config/tomat/config.toml`. This allows you to set default values for timer durations and behaviors without specifying them on every command.

**💡 Quick Start**: Copy the [complete example config](../config-example.toml):

```bash
mkdir -p ~/.config/tomat
cp config-example.toml ~/.config/tomat/config.toml
# Then edit ~/.config/tomat/config.toml as needed
```

## Configuration File Structure

### Timer Settings

```toml
[timer]
work = 25.0          # Work session duration in minutes (default: 25)
break = 5.0          # Break duration in minutes (default: 5)
long_break = 15.0    # Long break duration in minutes (default: 15)
sessions = 4         # Sessions until long break (default: 4)
auto_advance = false # Auto-advance between phases (default: false)
```

### Sound Notifications

By default, tomat plays audio notifications when transitioning between work/break phases:

- **Embedded sounds**: High-quality WAV files built into the application
- **Linux requirement**: Requires ALSA (Advanced Linux Sound Architecture)
- **Automatic fallback**: If audio system unavailable, falls back to system beep or disables audio
- **Customizable**: Override with your own sound files or disable entirely
- **Volume control**: Adjustable volume level (0.0 to 1.0)

```toml
[sound]
enabled = true        # Enable sound notifications (default: true)
system_beep = false  # Use system beep instead of WAV files (default: false)
use_embedded = true  # Use embedded sounds (default: true)
volume = 0.5         # Volume level 0.0-1.0 (default: 0.5)
# Custom sound files (optional - override embedded sounds)
# work_to_break = "/path/to/work-to-break.wav"
# break_to_work = "/path/to/break-to-work.wav"
# work_to_long_break = "/path/to/work-to-long-break.wav"
```

To disable audio notifications:

```toml
[sound]
enabled = false
```

To use custom sound files:

```toml
[sound]
use_embedded = false
work_to_break = "/home/user/sounds/work-done.wav"
break_to_work = "/home/user/sounds/break-over.wav"
work_to_long_break = "/home/user/sounds/long-break.wav"
```

### Desktop Notifications

Desktop notifications are shown when transitioning between phases. The notification system supports three icon modes:

- **`"auto"` (default)**: Uses embedded icon, automatically cached to `~/.cache/tomat/icon.png` (works with mako and other notification daemons)
- **`"theme"`**: Uses system theme icon (`"timer"`)
- **Custom path**: Specify a custom icon file, e.g., `"/home/user/my-icon.png"`

```toml
[notification]
enabled = true        # Enable desktop notifications (default: true)
icon = "auto"         # Icon mode: "auto", "theme", or path (default: "auto")
timeout = 10000        # Notification timeout in milliseconds (default: 3000)
```

To disable desktop notifications:

```toml
[notification]
enabled = false
```

To use a custom icon:

```toml
[notification]
icon = "/path/to/custom/icon.png"
```

To use longer notification timeout:

```toml
[notification]
timeout = 10000  # 10 seconds
```

## Priority Order

Settings are applied in this order (later overrides earlier):

1. **Built-in defaults**: 25min work, 5min break, 15min long break, 4 sessions
2. **Config file**: Values from `~/.config/tomat/config.toml`
3. **CLI arguments**: Flags passed to `tomat start`

## Partial Configuration

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

## Complete Example

Here's a complete configuration file with all options:

```toml
# ~/.config/tomat/config.toml

[timer]
work = 25.0          # Work session duration in minutes
break = 5.0          # Break duration in minutes
long_break = 15.0    # Long break duration in minutes
sessions = 4         # Number of work sessions before long break
auto_advance = false # Whether to automatically continue to next phase

[sound]
enabled = true       # Enable sound notifications
system_beep = false  # Use system beep instead of sound files
use_embedded = true  # Use embedded sound files
volume = 0.5         # Volume level (0.0 to 1.0)
# Custom sound files (optional - will override embedded sounds)
# work_to_break = "/path/to/custom/work-to-break.wav"
# break_to_work = "/path/to/custom/break-to-work.wav"
# work_to_long_break = "/path/to/custom/work-to-long-break.wav"

[notification]
enabled = true  # Enable desktop notifications
icon = "auto"   # Icon to use: "auto" (embedded), "theme" (system), or "/path/to/icon.png" (custom)
timeout = 10000 # Notification timeout in milliseconds
```

## Troubleshooting Configuration

### Config File Not Loading

1. **Check file location**: Ensure config is at `~/.config/tomat/config.toml`
2. **Verify syntax**: Use `toml` syntax validation tools
3. **Check permissions**: Ensure the file is readable

### Audio Issues

1. **ALSA not available**: Audio will be automatically disabled if ALSA is not available
2. **Custom sound files**: Ensure files exist and are in WAV format
3. **Volume issues**: Check system volume and tomat volume setting

### Notification Issues

1. **No notifications**: Check if notification daemon is running
2. **Icon not showing**: Try different icon modes (`"auto"`, `"theme"`, custom path)
3. **Mako compatibility**: Use `icon = "auto"` for best mako support
