# Display Settings

The `[display]` section controls how timer status is formatted in the output.

```toml
[display]
text_format = "{icon} {time} {state}"
# text_format_idle = "" # Optional: format for idle state (defaults to text_format)
```

## Options

`text_format`

:   Template string for formatting timer display when timer is active
    (work/break phases). The field supports the following placeholders enclosed
    in curly braces `{}`:

    `{icon}`
    :   Phase icon (`🍅` work/idle, `☕` break, `🏖️` long break)

    `{time}`https://github.com/jolars/dprint-plugin-badness/actions/runs/33058798128
    :   Remaining time (e.g., `25:00`). When idle, shows upcoming work duration.

    `{state}`
    :   Play/pause/stop symbol (`▶` running, `⏸` paused, `⏹` idle)

    `{phase}`
    :   Phase name (`Work`, `Break`, `Long Break`, `Idle`)

    `{session}`
    :   Session progress (e.g., `1/4`; empty for breaks and idle)

    Default
    :   `"{icon} {time} {state}"`

    Example
    :   `"{phase}: {time} {state}"`

`text_format_idle`

:   Template string for formatting timer display when timer is idle (stopped).
    Uses the same placeholders as `text_format`.

    If not specified, defaults to the value of `text_format`. Set to empty string
    `""` to hide the widget when timer is idle.

    Default
    :   Same as `text_format` (omit this option to use default)

    Examples

    :   ````toml
        # Hide widget when idle
        text_format_idle = ""

        # Show custom idle message
        text_format_idle = "⏹ Ready to start"

        # Use default (same as text_format) - simply omit the option:
        # text_format_idle = ...
        ```

        ## Examples

        Minimal format (time only):

        ```toml
        [display]
        text_format = "{time}"
        ````

With session counter:

```toml
[display]
text_format = "[{session}] {icon} {time}"
```

Verbose format:

```toml
[display]
text_format = "{phase}: {time} {state}"
```

Show only icon when in idle:

```toml
[display]
text_format = "{icon} {time} {state}"
text_format_idle = "{icon}"
```

## Icon Customization

The `[display.icons]` subsection allows you to customize the emoji/text symbols
used for different phases and states.

```toml
[display.icons]
work = "🍅"          # Work/Idle phase icon (default: 🍅)
break = "☕"         # Break phase icon (default: ☕)
long_break = "🏖️"   # Long break phase icon (default: 🏖️)
play = "▶"          # Playing state symbol (default: ▶)
pause = "⏸"         # Paused state symbol (default: ⏸)
stop = "⏹"          # Stopped/Idle state symbol (default: ⏹)
```

### Options

`work`

:   Icon shown during work sessions and idle phase (when using `{icon}`
    placeholder).

    Default: `"🍅"`

`break`

:   Icon shown during short break sessions.

    Default: `"☕"`

`long_break`

:   Icon shown during long break sessions.

    Default: `"🏖️"`

`play`

:   Symbol shown when timer is running (when using `{state}` placeholder).

    Default: `"▶"`

`pause`

:   Symbol shown when timer is paused.

    Default: `"⏸"`

`stop`

:   Symbol shown when timer is in idle state.

    Default: `"⏹"`

### Examples

ASCII-only symbols:

```toml
[display.icons]
work = "W"
break = "B"
long_break = "L"
play = ">"
pause = "||"
stop = "X"
```

Alternative emoji set:

```toml
[display.icons]
work = "💼"
break = "🎮"
long_break = "🌴"
```

Minimal text symbols:

```toml
[display.icons]
work = "[W]"
break = "[B]"
long_break = "[LB]"
play = "▸"
pause = "❙❙"
stop = "■"
```
