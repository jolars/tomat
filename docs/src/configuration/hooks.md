# Hooks

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

## Available Hooks

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

## Options

`cmd`
: Command to execute

  Example
  : `"/usr/bin/notify-send"`

`args`
: Command arguments (list of strings)

  Default
  : `[]` (no arguments)

  Example
  : `["-u", "critical"]`

`timeout`
: Timeout in seconds (0 = no timeout)

  Default
  : `5` seconds

`cwd`
: Working directory

  Default
  : User's home directory (`$HOME`)

  Example
  : `"/home/user/scripts"`

`capture_output`
: Capture stdout/stderr for debugging

  Default
  : `false`

## Environment Variables

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

These can be used in scripts to customize behavior based on the timer state.

## Examples

Pause music during work sessions:

```toml
[hooks.on_work_start]
cmd = "playerctl"
args = ["pause"]
timeout = 2

[hooks.on_break_start]
cmd = "playerctl"
args = ["play"]
```

Adjust screen brightness:

```toml
[hooks.on_pause]
cmd = "brightnessctl"
args = ["set", "30%"]

[hooks.on_resume]
cmd = "brightnessctl"
args = ["set", "100%"]
```

Log completed sessions:

```toml
[hooks.on_complete]
cmd = "sh"
args = ["-c", "echo \"$(date): $TOMAT_PHASE completed\" >> ~/tomat.log"]
capture_output = true
```

Custom notifications:

```toml
[hooks.on_work_start]
cmd = "notify-send"
args = ["🍅 Focus Time", "Let's get things done!", "-u", "critical"]
timeout = 3

[hooks.on_long_break_start]
cmd = "notify-send"
args = ["🏖️ Long Break", "You've earned it! Take 15 minutes."]
```

Execute custom scripts:

```toml
[hooks.on_work_start]
cmd = "/home/user/scripts/start-focus-mode.sh"
cwd = "/home/user/scripts"
timeout = 10

[hooks.on_stop]
cmd = "/home/user/scripts/end-focus-mode.sh"
cwd = "/home/user/scripts"
```

## Security Considerations

Hooks execute with the daemon's user privileges. Follow these security best
practices:

- **Only configure trusted commands**: Hooks can execute any command your user
  can run.
- **Use absolute paths**: Prefer `/usr/bin/notify-send` over `notify-send` to
  avoid PATH manipulation.
- **Never run the daemon as root**: Always use the `--user` systemd service.
- **Verify config file ownership**: Ensure your config file is owned by your user.
- **No shell injection**: Commands are executed directly without a shell,
  preventing injection attacks.
- **Timeout protection**: Hooks are automatically killed after the timeout to
  prevent hanging processes.

**_Note:_** _If an attacker controls your `$XDG_CONFIG_HOME` directory, they already
have code execution via shell rc files. Hooks don't introduce new attack
vectors beyond standard Unix permissions._
