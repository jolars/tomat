# Notification Systems

Tomat uses the `notify-rust` crate to send desktop notifications, which
supports a variety of notification daemons on Linux systems. Below are
instructions for integrating Tomat with popular notification systems.

## Mako (Wayland)

Tomat works perfectly with mako out of the box using the embedded icon system:

```bash
# Default configuration works automatically
# Icon will be cached to ~/.cache/tomat/icon.png
```

For custom mako styling, add to `~/.config/mako/config`:

```ini
[app-name="Tomat"]
background-color=#2d3748
text-color=#ffffff
border-color=#4a5568
default-timeout=5000
```

## Dunst (X11)

For dunst notification daemon:

```bash
# Works with default configuration
# Uses embedded icon or theme icon as fallback
```

Custom dunst rules (`~/.config/dunst/dunstrc`):

```ini
[tomat]
appname = "Tomat"
background = "#2d3748"
foreground = "#ffffff"
timeout = 5000
```

## Custom Notification Icons

To use a custom notification icon:

```toml
# ~/.config/tomat/config.toml
[notification]
icon = "/path/to/your/custom-icon.png"
```

