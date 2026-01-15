# Display Settings

The `[display]` section controls how timer status is formatted in the output.

```toml
[display]
text_format = "{icon} {time} {state}"
```

## Options

`text_format`
: Template string for formatting timer display.
  The field supports the following placeholders enclosed in curly braces `{}`:

  `{icon}`
  : Phase icon (`🍅` `☕` `🏖️`)

  `{time}`
  : Remaining time (e.g., `25:00`)

  `{state}`
  : Play/pause symbol (`▶` `⏸`)

  `{phase}`
  : Phase name (`Work`, `Break`, `Long Break`)

  `{session}`
  : Session progress (e.g., `1/4`; empty for breaks)

  Default
  : `"{icon} {time} {state}"`

  Example
  : `"{phase}: {time} {state}"`

## Examples

Minimal format (time only):

```toml
[display]
text_format = "{time}"
```

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
