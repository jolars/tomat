# Configuration Guide

Tomat supports comprehensive configuration via a TOML file located at
`~/.config/tomat/config.toml`. This allows you to set default values for timer
durations and behaviors without specifying them on every command.

The configuration file is organized into five main sections: `[timer]` for timer
durations and behavior, `[sound]` for audio notification settings,
`[notification]` for desktop notification settings, `[display]` for output
formatting, and `[hooks]` for custom commands triggered by timer events.

```admonish tip
Copy the
[complete example config](https://github.com/jolars/tomat/blob/main/examples/config.toml)
to get started quickly.
```

```bash
mkdir -p ~/.config/tomat
cp examples/config.toml ~/.config/tomat/config.toml
# Then edit ~/.config/tomat/config.toml as needed
```

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

| Mode         | Behavior                                                    |
| ------------ | ----------------------------------------------------------- |
| `"none"`     | Pause after every phase transition (requires manual resume) |
| `"all"`      | Automatically continue through all phases without pausing   |
| `"to-break"` | Auto-advance only from work to break/long-break             |
| `"to-work"`  | Auto-advance only from break/long-break to work             |

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
enabled = true
system_beep = false
use_embedded = true
volume = 0.5
```

### Quick Reference

| Option               | Default |
| -------------------- | ------- |
| `enabled`            | `true`  |
| `system_beep`        | `false` |
| `use_embedded`       | `true`  |
| `volume`             | `0.5`   |
| `work_to_break`      | -       |
| `break_to_work`      | -       |
| `work_to_long_break` | -       |

### Using Custom Sounds

To use your own sound files, set `use_embedded = false` and specify the paths to
your WAV files:

```toml
[sound]
use_embedded = false
work_to_break = "/home/user/sounds/work-done.wav"
break_to_work = "/home/user/sounds/break-over.wav"
work_to_long_break = "/home/user/sounds/long-break.wav"
```

To disable all audio:

```toml
[sound]
enabled = false
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

| Option               | Default                                           |
| -------------------- | ------------------------------------------------- |
| `enabled`            | `true`                                            |
| `icon`               | `"auto"`                                          |
| `timeout`            | `5000`                                            |
| `urgency`            | `"normal"`                                        |
| `work_message`       | `"Break time! Take a short rest ☕"`              |
| `break_message`      | `"Back to work! Let's focus 🍅"`                  |
| `long_break_message` | `"Long break time! Take a well-deserved rest 🏖️"` |

### Icon Modes

| Mode        | Description                                                               |
| ----------- | ------------------------------------------------------------------------- |
| `"auto"`    | Uses embedded icon, cached to `~/.cache/tomat/icon.png` (mako-compatible) |
| `"theme"`   | Uses system theme icon (`"timer"`)                                        |
| Custom path | Specify a file path (e.g., `"/home/user/my-icon.png"`)                    |

### Urgency Levels

| Level        | Behavior                                            |
| ------------ | --------------------------------------------------- |
| `"low"`      | Minimal interruption, typically shown without sound |
| `"normal"`   | Standard notification priority                      |
| `"critical"` | High priority, may bypass do-not-disturb settings   |

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
2. **Config file**: Values from `~/.config/tomat/config.toml`.
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
# ~/.config/tomat/config.toml

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

| Hook                  | Triggered When                                          |
| --------------------- | ------------------------------------------------------- |
| `on_work_start`       | A work session starts                                   |
| `on_break_start`      | A break starts                                          |
| `on_long_break_start` | A long break starts                                     |
| `on_pause`            | Timer is paused                                         |
| `on_resume`           | Timer is resumed                                        |
| `on_stop`             | Timer is stopped manually                               |
| `on_complete`         | A phase completes naturally (auto or manual transition) |
| `on_skip`             | User skips to next phase                                |

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

| Variable                  | Description                                                      |
| ------------------------- | ---------------------------------------------------------------- |
| `TOMAT_EVENT`             | Event name (e.g., `"work_start"`, `"pause"`)                     |
| `TOMAT_PHASE`             | Current phase (`"work"`, `"break"`, `"long_break"`)              |
| `TOMAT_REMAINING_SECONDS` | Seconds remaining in current phase                               |
| `TOMAT_SESSION_COUNT`     | Current session number (1, 2, 3, ...)                            |
| `TOMAT_AUTO_ADVANCE`      | Auto-advance mode (`"none"`, `"all"`, `"to-break"`, `"to-work"`) |

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

```admonish warning

Hooks execute with the daemon's user privileges. Follow these security best
practices:

- **Only configure trusted commands**: Hooks can execute any command your user
  can run.
- **Use absolute paths**: Prefer `/usr/bin/notify-send` over `notify-send` to
  avoid PATH manipulation.
- **Never run the daemon as root**: Always use the `--user` systemd service.
- **Verify config file ownership**: Ensure `~/.config/tomat/config.toml` is
  owned by your user.
- **No shell injection**: Commands are executed directly without a shell,
  preventing injection attacks.
- **Timeout protection**: Hooks are automatically killed after the timeout to
  prevent hanging processes.

**_Note:_** _If an attacker controls your `~/.config` directory, they already
have code execution via shell rc files. Hooks don't introduce new attack vectors
beyond standard Unix permissions._

```

## Troubleshooting

### Config File Issues

#### Config file not loading

Check that your config file is at the correct location:
`~/.config/tomat/config.toml`. You can verify the path exists:

```bash
ls -l ~/.config/tomat/config.toml
```

#### Syntax errors

TOML is whitespace-sensitive and requires proper quoting. Use a TOML validator
or check that brackets, quotes, and equal signs are balanced.

#### Permission denied

Ensure the config file is readable by your user:

```bash
chmod 644 ~/.config/tomat/config.toml
```

### Audio Issues

#### No sound playing

On Linux, audio requires ALSA (Advanced Linux Sound Architecture). If ALSA is
not available, tomat will automatically disable audio or fall back to the system
beep.

#### Custom sounds not working

Verify that your sound files exist and are in WAV format:

```bash
file /path/to/your/sound.wav
```

#### Volume too low or too high

Adjust the `volume` setting in your config file. Valid range is 0.0 (silent) to
1.0 (full volume):

```toml
[sound]
volume = 0.8
```

#### Fallback to system beep

If you're hearing the system beep instead of audio files, your system may not
have ALSA available. You can explicitly enable this behavior:

```toml
[sound]
system_beep = true
```

### Notification Issues

#### No notifications appearing

Check if your notification daemon is running. Common daemons include `dunst` and
`mako`. You can test with:

```bash
notify-send "Test" "Notification test"
```

#### Icon not showing

Try different icon modes. The `"auto"` mode works best with mako:

```toml
[notification]
icon = "auto"
```

Alternatively, use the system theme icon:

```toml
[notification]
icon = "theme"
```

#### Timeout too short

Increase the timeout value (in milliseconds):

```toml
[notification]
timeout = 10000
```

### Hook Issues

#### Hook not executing

Verify the command exists and is executable:

```bash
which playerctl
ls -l /home/user/scripts/my-script.sh
```

Use absolute paths in your hook configuration:

```toml
[hooks.on_work_start]
cmd = "/usr/bin/playerctl"
```

#### Permission denied

Check that the command has execute permissions:

```bash
chmod +x /home/user/scripts/my-script.sh
```

#### Hook timing out

If your command takes longer than the default 5 seconds, increase the timeout:

```toml
[hooks.on_work_start]
cmd = "/home/user/scripts/slow-script.sh"
timeout = 10
```

Check if the command hangs when run manually. Ensure it doesn't require
interactive input.

#### No error output

Enable output capture to see error messages:

```toml
[hooks.on_work_start]
cmd = "/usr/bin/my-command"
capture_output = true
```

#### Working directory errors

Verify the working directory exists and has correct permissions:

```bash
ls -ld /path/to/working/directory
```

Use absolute paths for both commands and working directories.

#### Hook not triggered

Check that the hook name is spelled correctly (e.g., `on_work_start`, not
`on_start_work`). After changing your config, restart the daemon:

```bash
tomat daemon stop
tomat daemon start
```

#### Environment variables not working

Test your hook manually with environment variables:

```bash
TOMAT_EVENT=work_start TOMAT_PHASE=work /path/to/command
```

Verify your script or command is reading the correct variable names.
