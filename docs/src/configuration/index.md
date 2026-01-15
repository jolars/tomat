# Configuration Guide

Tomat supports comprehensive configuration via a TOML file located at
`$XDG_CONFIG_HOME/tomat/config.toml` (typically `~/.config/tomat/config.toml`).
This allows you to set default values for timer durations and behaviors without
specifying them on every command.

The configuration file is organized into five main sections: 

[`[timer]`](timer.md)
: timer durations and behavior

[`[sound]`](sound.md)
: for audio notification settings

[`[notification]`](notification.md)
: for desktop notification settings [`[display]`](display.md) for output

[`[display]`](display.md)
: for output formatting

[`[hooks]`](hooks.md)
: for custom commands triggered by timer events


