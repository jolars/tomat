# i3blocks

i3blocks works perfectly with Tomat's plain text output mode.

## Simple Integration

```ini
[tomat]
command=tomat status --output plain
interval=1
```

## With Click Support

```ini
[tomat]
command=tomat status --output plain
interval=1
signal=10
```

Add click handling with environment variables:

```bash
#!/bin/bash
# ~/.config/i3blocks/scripts/tomat-click
case $BLOCK_BUTTON in
    1) tomat toggle ;;     # Left click: toggle
    3) tomat skip ;;       # Right click: skip
esac
pkill -RTMIN+10 i3blocks   # Refresh the block
```

Then set as the command: `command=~/.config/i3blocks/scripts/tomat-click`
