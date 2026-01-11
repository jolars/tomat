# Configuration Guide

Tomat supports comprehensive configuration via a TOML file located at
`~/.config/tomat/config.toml`. This allows you to set default values for timer
durations and behaviors without specifying them on every command.

**💡 Quick Start**: Copy the [complete example config](../examples/config.toml):

```bash
mkdir -p ~/.config/tomat
cp examples/config.toml ~/.config/tomat/config.toml
# Then edit ~/.config/tomat/config.toml as needed
```

## Configuration File Structure

### Timer Settings

```toml
[timer]
work = 25.0            # Work session duration in minutes (default: 25)
break = 5.0            # Break duration in minutes (default: 5)
long_break = 15.0      # Long break duration in minutes (default: 15)
sessions = 4           # Sessions until long break (default: 4)
auto_advance = "none"  # Auto-advance mode (default: "none")
                       # Options: "none", "all", "to-break", "to-work"
                       # (boolean true/false also supported for backwards compatibility)
```

#### Auto-Advance Modes

The `auto_advance` setting controls how the timer transitions between phases:

- **`"none"`** (default): Pause after every phase transition, requiring manual resume. Gives you full control over your schedule.
- **`"all"`**: Automatically continue through all phases without pausing. Perfect for uninterrupted Pomodoro sessions.
- **`"to-break"`**: Auto-advance only from work to break/long-break. Enforces regular breaks while letting you choose when to resume work.
- **`"to-work"`**: Auto-advance only from break/long-break to work. Allows self-paced breaks while ensuring work sessions start promptly.

**Backwards compatibility**: Boolean values `true` and `false` are still supported and will be automatically converted to `"all"` and `"none"` respectively.

### Sound Notifications

By default, tomat plays audio notifications when transitioning between
work/break phases:

- **Embedded sounds**: High-quality WAV files built into the application
- **Linux requirement**: Requires ALSA (Advanced Linux Sound Architecture)
- **Automatic fallback**: If audio system unavailable, falls back to system beep
  or disables audio
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

Desktop notifications are shown when transitioning between phases. The
notification system supports three icon modes:

- **`"auto"` (default)**: Uses embedded icon, automatically cached to
  `~/.cache/tomat/icon.png` (works with mako and other notification daemons)
- **`"theme"`**: Uses system theme icon (`"timer"`)
- **Custom path**: Specify a custom icon file, e.g., `"/home/user/my-icon.png"`

```toml
[notification]
enabled = true        # Enable desktop notifications (default: true)
icon = "auto"         # Icon mode: "auto", "theme", or path (default: "auto")
timeout = 5000        # Notification timeout in milliseconds (default: 3000)
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
auto_advance = "all"  # Auto-advance through all phases
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
work = 25.0           # Work session duration in minutes
break = 5.0           # Break duration in minutes
long_break = 15.0     # Long break duration in minutes
sessions = 4          # Number of work sessions before long break
auto_advance = "none" # Auto-advance mode: "none", "all", "to-break", "to-work"
                      # (boolean true/false also supported for backwards compatibility)

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
enabled = true                           # Enable desktop notifications
icon = "auto"                           # Icon to use: "auto" (embedded), "theme" (system), or "/path/to/icon.png" (custom)
timeout = 5000                          # Notification timeout in milliseconds
```

## Hooks

Hooks allow you to execute custom commands when timer events occur. This enables
integration with external tools, automation workflows, and custom notifications.

### Basic Syntax

```toml
[hooks.on_work_start]
cmd = "notify-send"
args = ["🍅 Work Time", "Focus for 25 minutes"]

[hooks.on_break_start]
cmd = "playerctl"
args = ["pause"]
```

### Available Hooks

- `on_work_start` - Triggered when a work session starts
- `on_break_start` - Triggered when a break starts
- `on_long_break_start` - Triggered when a long break starts
- `on_pause` - Triggered when timer is paused
- `on_resume` - Triggered when timer is resumed
- `on_stop` - Triggered when timer is stopped manually
- `on_complete` - Triggered when a phase completes naturally (auto or manual
  transition)
- `on_skip` - Triggered when user skips to next phase

### Hook Configuration Fields

Each hook supports these optional fields:

```toml
[hooks.on_work_start]
cmd = "/usr/bin/notify-send"        # Command to execute (required)
args = ["Title", "Message"]         # Command arguments (optional, default: [])
timeout = 5                         # Timeout in seconds (optional, default: 5)
cwd = "/home/user"                  # Working directory (optional, default: $HOME)
capture_output = false              # Capture stdout/stderr (optional, default: false)
```

### Environment Variables

All hooks receive these environment variables:

- `TOMAT_EVENT` - Event name (e.g., `"work_start"`, `"pause"`)
- `TOMAT_PHASE` - Current phase (`"work"`, `"break"`, `"long_break"`)
- `TOMAT_REMAINING_SECONDS` - Seconds remaining in current phase
- `TOMAT_SESSION_COUNT` - Current session number (e.g., `1`, `2`, `3`)
- `TOMAT_AUTO_ADVANCE` - Auto-advance mode (`"none"`, `"all"`, `"to-break"`, `"to-work"`)

### Example Use Cases

**Pause music during work sessions:**

```toml
[hooks.on_work_start]
cmd = "playerctl"
args = ["pause"]
timeout = 2

[hooks.on_break_start]
cmd = "playerctl"
args = ["play"]
```

**Adjust screen brightness:**

```toml
[hooks.on_pause]
cmd = "brightnessctl"
args = ["set", "30%"]

[hooks.on_resume]
cmd = "brightnessctl"
args = ["set", "100%"]
```

**Log completed sessions:**

```toml
[hooks.on_complete]
cmd = "sh"
args = ["-c", "echo \"$(date): $TOMAT_PHASE completed\" >> ~/tomat.log"]
capture_output = true
```

**Custom notifications:**

```toml
[hooks.on_work_start]
cmd = "notify-send"
args = ["🍅 Focus Time", "Let's get things done!", "-u", "critical"]
timeout = 3

[hooks.on_long_break_start]
cmd = "notify-send"
args = ["🏖️ Long Break", "You've earned it! Take 15 minutes."]
```

**Execute custom scripts:**

```toml
[hooks.on_work_start]
cmd = "/home/user/scripts/start-focus-mode.sh"
cwd = "/home/user/scripts"
timeout = 10

[hooks.on_stop]
cmd = "/home/user/scripts/end-focus-mode.sh"
cwd = "/home/user/scripts"
```

### Security Considerations

**Hooks execute with daemon's user privileges.** Follow these security
guidelines:

- ✅ **Only configure trusted commands** - hooks can execute any command
- ✅ **Use absolute paths** - e.g., `/usr/bin/notify-send` instead of
  `notify-send`
- ✅ **Never run daemon as root** - always use `--user` systemd service
- ✅ **Verify config file ownership** - ensure `~/.config/tomat/config.toml` is
  owned by your user
- ✅ **No shell injection** - commands are executed directly (not via shell),
  preventing injection attacks
- ✅ **Timeout protection** - hooks are killed after timeout to prevent hanging

**Threat model**: If an attacker controls your `~/.config` directory, they
already have code execution via shell rc files. Hooks don't introduce new attack
vectors beyond standard Unix permissions.

## Troubleshooting Configuration

### Config File Not Loading

1. **Check file location**: Ensure config is at `~/.config/tomat/config.toml`
2. **Verify syntax**: Use `toml` syntax validation tools
3. **Check permissions**: Ensure the file is readable

### Audio Issues

1. **ALSA not available**: Audio will be automatically disabled if ALSA is not
   available
2. **Custom sound files**: Ensure files exist and are in WAV format
3. **Volume issues**: Check system volume and tomat volume setting

### Notification Issues

1. **No notifications**: Check if notification daemon is running
2. **Icon not showing**: Try different icon modes (`"auto"`, `"theme"`, custom
   path)
3. **Mako compatibility**: Use `icon = "auto"` for best mako support

### Hook Issues

1. **Hook not executing**:
   - Check command path with `which <command>` or use absolute path
   - Verify command is executable: `ls -l /path/to/command`
   - Enable `capture_output = true` to see error messages
2. **Hook timing out**:
   - Increase timeout: `timeout = 10`
   - Check if command hangs when run manually
   - Ensure command doesn't require interactive input
3. **Working directory errors**:
   - Verify `cwd` path exists: `ls -ld /path/to/cwd`
   - Use absolute paths for commands and files
   - Check permissions on working directory
4. **Environment variable issues**:
   - Test hook manually:
     `TOMAT_EVENT=work_start TOMAT_PHASE=work /path/to/command`
   - Check if command expects different variable names
5. **Hook not triggered**:
   - Verify hook name is spelled correctly
   - Check config file syntax with TOML validator
   - Restart daemon after config changes:
     `tomat daemon stop && tomat daemon start`
