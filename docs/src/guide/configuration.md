# Configuration Guide

Tomat supports comprehensive configuration via a TOML file. On Linux, the path
is `$XDG_CONFIG_HOME/tomat/config.toml` (typically
`~/.config/tomat/config.toml`); on macOS, it is
`~/Library/Application Support/tomat/config.toml`. The `TOMAT_CONFIG`
environment variable can override either location.

The configuration file is organized into five main sections, which
are documented in detail in [the configuration reference](../configuration/index.md).

## Example

Here's a minimal example of the `[timer]` section, showing the default values:

```toml
[timer]
work = 25.0
break = 5.0
long_break = 15.0
sessions = 4
```

> [!TIP]
>
> Ready-to-use configuration files are available in the
> [`examples/`](https://github.com/jolars/tomat/tree/main/examples) directory.
> Copy the
> [complete example config](https://github.com/jolars/tomat/blob/main/examples/config.toml)
> to get started quickly.
> 
> ```bash
> mkdir -p "${XDG_CONFIG_HOME:-$HOME/.config}/tomat"
> cp examples/config.toml "${XDG_CONFIG_HOME:-$HOME/.config}/tomat/config.toml"
> # Then edit the config file as needed
> ```

## Configuration Priority

Settings are applied in the following order, with later sources overriding
earlier ones:

1. **Built-in defaults**
   
   Standard Pomodoro timings (25min work, 5min break,
   15min long break, 4 sessions).

2. **Config file**

   Values from `$XDG_CONFIG_HOME/tomat/config.toml`.

3. **CLI arguments**

   Flags passed to `tomat start` command.

This means you can set your preferred defaults in the config file and still
override them on a per-session basis using command-line flags.

## Environment Variables

`TOMAT_CONFIG`
: Path to the configuration file, overriding the platform default.

`TOMAT_RUNTIME_DIR`
: Directory holding the daemon's socket, PID, and state files. It takes
  precedence over `XDG_RUNTIME_DIR`, which in turn takes precedence over the
  platform default (`/run/user/$UID` on Linux, a Tomat directory beneath the
  per-user `$TMPDIR` on macOS). The directory must be owned by you and must not
  be writable by anyone else; Tomat creates it with mode `700` when it does not
  exist yet, and refuses to modify a directory it did not create.

The daemon and its clients each resolve the runtime directory on their own, so
they have to agree. If you export `TOMAT_RUNTIME_DIR` from your shell profile,
run `tomat daemon install` from a shell where it is set: the generated service
file records the value so the service-managed daemon uses the same socket. See
[Service Management](integration/service-management.md).

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
