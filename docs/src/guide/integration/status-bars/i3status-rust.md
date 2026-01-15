# i3status-rust

Tomat provides native support for i3status-rust with a dedicated output format:

```toml
[[block]]
block = "custom"
command = "tomat status --output i3status-rs"
interval = 1
json = true

[[block.click]]
button = "left"
cmd = "tomat toggle"

[[block.click]]
button = "right"
cmd = "tomat skip"
```

The i3status-rs format provides:

`text`
: Display text with timer and status icons

`short_text`
: Same as text (for abbreviated display)

`state`
: Timer state mapping (Critical=work, Good=break, Info=paused)
