# Configuration Guide

Tomat supports comprehensive configuration via a TOML file located at
`$XDG_CONFIG_HOME/tomat/config.toml` (typically `~/.config/tomat/config.toml`).
This allows you to set default values for timer durations and behaviors without
specifying them on every command.

The configuration file is organized into five main sections: `[timer]` for timer
durations and behavior, `[sound]` for audio notification settings,
`[notification]` for desktop notification settings, `[display]` for output
formatting, and `[hooks]` for custom commands triggered by timer events.

## File Locations

Tomat follows the [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html):

**Configuration**
  : `$XDG_CONFIG_HOME/tomat/config.toml` (default: `~/.config/tomat/config.toml`)

**Cache**
  : `$XDG_CACHE_HOME/tomat/` (default: `~/.cache/tomat/`)
    - Stores cached notification icon

**Runtime**
  : `$XDG_RUNTIME_DIR/` (default: `/run/user/$UID/`)
    - Unix socket: `tomat.sock`
    - PID file: `tomat.pid`

You can override these locations by setting the respective environment variables.

> [!TIP]
>
> Ready-to-use configuration files are available in the
> [`examples/`](https://github.com/jolars/tomat/tree/main/examples) directory.
> Copy the
> [complete example config](https://github.com/jolars/tomat/blob/main/examples/config.toml)
> to get started quickly.

````

```bash
mkdir -p $XDG_CONFIG_HOME/tomat  # or: mkdir -p ~/.config/tomat
cp examples/config.toml $XDG_CONFIG_HOME/tomat/config.toml
# Then edit the config file as needed
````

## Timer Settings

The `[timer]` section controls timer durations and phase transition behavior.

```toml
[timer]
work = 25.0
break = 5.0
long_break = 15.0
sessions = 4
auto_advance = "none"
```

### Auto-Advance Modes

The `auto_advance` setting controls how the timer transitions between phases:

`"none"`
  : Pause after every phase transition (requires manual resume)

`"all"`
  : Automatically continue through all phases without pausing

`"to-break"`
  : Auto-advance only from work to break/long-break

`"to-work"`
  : Auto-advance only from break/long-break to work

---

**_Note:_** _Boolean values `true` and `false` are still supported for backwards
compatibility and will be automatically converted to `"all"` and `"none"`
respectively._

## Sound Settings

The `[sound]` section controls audio notifications when transitioning between
work/break phases. By default, tomat plays high-quality WAV files built into the
application. On Linux, this requires ALSA (Advanced Linux Sound Architecture).
If the audio system is unavailable, it will automatically fall back to the
system beep or disable audio.

```toml
[sound]
mode = "embedded"  # Options: "embedded", "system-beep", "none"
volume = 0.5
```

### Options

`mode`
  : Sound notification mode. Controls how phase transitions are announced:
    - `"embedded"` (default): Use built-in audio files
    - `"system-beep"`: Use system beep (terminal bell)
    - `"none"`: No sound notifications

`volume`
  : Audio volume level for embedded and custom sounds (0.0-1.0). Default: `0.5`

`work_to_break`
  : Path to custom sound file for work→break transitions. Overrides embedded sound. Optional.

`break_to_work`
  : Path to custom sound file for break→work transitions. Overrides embedded sound. Optional.

`work_to_long_break`
  : Path to custom sound file for work→long break transitions. Overrides embedded sound. Optional.

**Deprecated options** (kept for backwards compatibility):
- `enabled`: Use `mode = "none"` instead
- `system_beep`: Use `mode = "system-beep"` instead  
- `use_embedded`: Use `mode = "embedded"` instead

### Using Custom Sounds

To use your own sound files, keep `mode = "embedded"` and specify paths to your
audio files. Custom sounds override the built-in ones:

```toml
[sound]
mode = "embedded"
work_to_break = "/home/user/sounds/work-done.ogg"
break_to_work = "/home/user/sounds/break-over.ogg"
work_to_long_break = "/home/user/sounds/long-break.ogg"
volume = 0.7
```

To disable all audio:

```toml
[sound]
mode = "none"
```

To use system beep only:

```toml
[sound]
mode = "system-beep"
```

## Notification Settings

The `[notification]` section controls desktop notifications shown during phase
transitions. Notifications are displayed by your system's notification daemon
(such as dunst or mako).

```toml
[notification]
enabled = true
icon = "auto"
timeout = 5000
urgency = "normal"
```

### Quick Reference

`enabled`
  : Whether to show desktop notifications. 

`icon`
  : Controls notification icon.

    `"auto"` (default)
    : Uses embedded icon, cached to `$XDG_CACHE_HOME/tomat/icon.png` (mako-compatible)

    `"theme"`
    : Uses system theme icon (`"timer"`)

    path
    : Specify a file path (e.g., `"/home/user/my-icon.png"`)

`timeout`
  : Default: `5000`

`urgency`
  : Controls notification priority level.

    `"normal"` (default)
    : Standard notification priority

    `"low"`
    : Minimal interruption, typically shown without sound

    `"critical"`
    : High priority, may bypass do-not-disturb settings

`work_message`
  : Default: `"Break time! Take a short rest ☕"`

`break_message`
  : Default: `"Back to work! Let's focus 🍅"`

`long_break_message`
  : Default: `"Long break time! Take a well-deserved rest 🏖️"`

### Icon Modes


### Urgency Levels


### Examples

To disable notifications:

```toml
[notification]
enabled = false
```

To use a custom icon:

```toml
[notification]
icon = "/path/to/custom/icon.png"
```

To customize notification messages:

```toml
[notification]
work_message = "Break time! Step away from the screen."
break_message = "Back to work! Let's get things done."
long_break_message = "Long break! You've earned it."
```

## Display Settings

The `[display]` section controls how timer status is formatted in the output.

```toml
[display]
text_format = "{icon} {time} {state}"
```

### Available Placeholders

| Placeholder | Description                                      |
| ----------- | ------------------------------------------------ |
| `{icon}`    | Phase icon (`🍅` `☕` `🏖️`)                      |
| `{time}`    | Remaining time (e.g., `25:00`)                   |
| `{state}`   | Play/pause symbol (`▶` `⏸`)                    |
| `{phase}`   | Phase name (`Work`, `Break`, `Long Break`)       |
| `{session}` | Session progress (e.g., `1/4`; empty for breaks) |

### Examples

**Minimal format (time only):**

```toml
[display]
text_format = "{time}"
```

**With session counter:**

```toml
[display]
text_format = "[{session}] {icon} {time}"
```

**Verbose format:**

```toml
[display]
text_format = "{phase}: {time} {state}"
```

## Configuration Priority

Settings are applied in the following order, with later sources overriding
earlier ones:

1. **Built-in defaults**: Standard Pomodoro timings (25min work, 5min break,
   15min long break, 4 sessions).
2. **Config file**: Values from `$XDG_CONFIG_HOME/tomat/config.toml`.
3. **CLI arguments**: Flags passed to `tomat start` command.

This means you can set your preferred defaults in the config file and still
override them on a per-session basis using command-line flags.

## Partial Configuration

You only need to specify the values you want to override. Any unspecified
options will use their built-in defaults:

```toml
[timer]
work = 30.0
auto_advance = "all"

[sound]
enabled = false
```

In this example, `break`, `long_break`, and `sessions` will use their default
values, and all other sound settings besides `enabled` will also use defaults.

## Complete Configuration Example

Here's a complete configuration file with all available options:

```toml
# $XDG_CONFIG_HOME/tomat/config.toml

# Timer durations and behavior
[timer]
work = 25.0           # Work session duration in minutes
break = 5.0           # Break duration in minutes
long_break = 15.0     # Long break duration in minutes
sessions = 4          # Number of work sessions before long break
auto_advance = "none" # Auto-advance mode: "none", "all", "to-break", "to-work"

# Sound notifications
[sound]
enabled = true        # Enable sound notifications
system_beep = false   # Use system beep instead of sound files
use_embedded = true   # Use embedded sound files
volume = 0.5          # Volume level (0.0 to 1.0)
# Optional: Custom sound files (will override embedded sounds)
# work_to_break = "/path/to/custom/work-to-break.wav"
# break_to_work = "/path/to/custom/break-to-work.wav"
# work_to_long_break = "/path/to/custom/work-to-long-break.wav"

# Desktop notifications
[notification]
enabled = true        # Enable desktop notifications
icon = "auto"         # Icon mode: "auto" (embedded), "theme" (system), or "/path/to/icon.png"
timeout = 5000        # Notification timeout in milliseconds
urgency = "normal"    # Urgency level: "low", "normal", "critical"
# Optional: Custom notification messages
# work_message = "Break time! Take a short rest ☕"
# break_message = "Back to work! Let's focus 🍅"
# long_break_message = "Long break time! Take a well-deserved rest 🏖️"

# Display formatting
[display]
text_format = "{icon} {time} {state}"  # Text display template
```

## Hooks

Hooks allow you to execute custom commands when timer events occur. This enables
integration with external tools, automation workflows, and custom notifications.

```toml
[hooks.on_work_start]
cmd = "playerctl"
args = ["pause"]

[hooks.on_break_start]
cmd = "playerctl"
args = ["play"]
```

### Available Hooks

`on_work_start`
  : A work session starts

`on_break_start`
  : A break starts

`on_long_break_start`
  : A long break starts

`on_pause`
  : Timer is paused

`on_resume`
  : Timer is resumed

`on_stop`
  : Timer is stopped manually

`on_complete`
  : A phase completes naturally (auto or manual transition)

`on_skip`
  : User skips to next phase

### Hook Configuration Fields

| Field            | Default | Required |
| ---------------- | ------- | -------- |
| `cmd`            | -       | Yes      |
| `args`           | `[]`    | No       |
| `timeout`        | `5`     | No       |
| `cwd`            | `$HOME` | No       |
| `capture_output` | `false` | No       |

- **`cmd`**: Command to execute
- **`args`**: Command arguments (list of strings)
- **`timeout`**: Timeout in seconds (0 = no timeout)
- **`cwd`**: Working directory
- **`capture_output`**: Capture stdout/stderr for debugging

### Environment Variables

All hooks receive these environment variables:

`TOMAT_EVENT`
  : Event name (e.g., `"work_start"`, `"pause"`)

`TOMAT_PHASE`
  : Current phase (`"work"`, `"break"`, `"long_break"`)

`TOMAT_REMAINING_SECONDS`
  : Seconds remaining in current phase

`TOMAT_SESSION_COUNT`
  : Current session number (1, 2, 3, ...)

`TOMAT_AUTO_ADVANCE`
  : Auto-advance mode (`"none"`, `"all"`, `"to-break"`, `"to-work"`)

### Example Configurations

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

> [!WARNING]
>
> Hooks execute with the daemon's user privileges. Follow these security best
> practices:
>
> - **Only configure trusted commands**: Hooks can execute any command your user
>   can run.
> - **Use absolute paths**: Prefer `/usr/bin/notify-send` over `notify-send` to
>   avoid PATH manipulation.
> - **Never run the daemon as root**: Always use the `--user` systemd service.
> - **Verify config file ownership**: Ensure your config file is owned by your user.
> - **No shell injection**: Commands are executed directly without a shell,
>   preventing injection attacks.
> - **Timeout protection**: Hooks are automatically killed after the timeout to
>   prevent hanging processes.
>
> **_Note:_** _If an attacker controls your `$XDG_CONFIG_HOME` directory, they already
> have code execution via shell rc files. Hooks don't introduce new attack
> vectors beyond standard Unix permissions._
