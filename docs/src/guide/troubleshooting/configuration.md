# Configuration Issues

## Config File Not Loading

### Problem

Configuration file is not being loaded or recognized.

### Solution

Check that your config file is at the correct location:
`$XDG_CONFIG_HOME/tomat/config.toml` (typically `~/.config/tomat/config.toml`).

Verify the path exists:

```bash
ls -l $XDG_CONFIG_HOME/tomat/config.toml
# Or: ls -l ~/.config/tomat/config.toml
```

## Syntax Errors

### Problem

Configuration file has TOML syntax errors.

### Solution

TOML is whitespace-sensitive and requires proper quoting. Use a TOML validator
or check that brackets, quotes, and equal signs are balanced.

Test your config:

```bash
# Using tomat to validate
tomat status
# Will show errors if config is invalid
```

## Permission Denied

### Problem

Cannot read configuration file due to permission errors.

### Solution

Ensure the config file is readable by your user:

```bash
chmod 644 $XDG_CONFIG_HOME/tomat/config.toml
# Or: chmod 644 ~/.config/tomat/config.toml
```
