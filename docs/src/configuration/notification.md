# Notification Settings

The `[notification]` section controls desktop notifications shown during phase
transitions. Notifications are displayed by the native notification system on
macOS or by a notification daemon such as dunst or mako on Linux.

```toml
[notification]
enabled = true
icon = "auto"
timeout = 5000
urgency = "normal"
```

## Options

`enabled`
  : Whether to show desktop notifications. 

`icon`
  : Controls notification icon.

    `"auto"` (default)
    : Uses embedded icon, cached to `$XDG_CACHE_HOME/tomat/icon.png` (mako-compatible)

    `"theme"`
    : Uses system theme icon (`"timer"`)

    `<path>`
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

> [!NOTE]
>
> macOS controls notification icons and display duration. The current macOS
> backend also ignores `urgency`; these options continue to apply on Linux.

`work_message`
  : The message shown when transitioning from work to break. 

    Default
    : `"Break time! Take a short rest ☕"`

`break_message`
  : The message sown when transitioning from break to work. 

    Default
    : `"Back to work! Let's focus 🍅"`

`long_break_message`
  : The message shown when transitioning to a long break.

    Default
    : `"Long break time! Take a well-deserved rest 🏖️"`

## Examples

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
