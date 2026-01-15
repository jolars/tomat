## Polybar

To integrate Tomat with Polybar, you can create a custom module that executes
the `tomat status` command to display the current timer status. Below is an
example configuration snippet that you can add to your Polybar configuration
file (usually located at `~/.config/polybar/config`).

```ini
[module/tomat]
type = custom/script
exec = tomat status --output plain
interval = 1
click-left = tomat toggle
click-right = tomat skip
format-prefix = " "
format-foreground = #ffffff
```
