# XDG Directory Support Test

This demonstrates that tomat now properly respects XDG Base Directory specifications for systemd service installation.

## Before (hardcoded paths)
```rust
let home = std::env::var("HOME")?;
let systemd_dir = format!("{}/.config/systemd/user", home);
```

## After (XDG-compliant)
```rust
let systemd_dir = if let Some(config_dir) = dirs::config_dir() {
    config_dir.join("systemd").join("user")
} else {
    // Fallback to HOME/.config if XDG config dir is not available
    let home = std::env::var("HOME")?;
    std::path::PathBuf::from(home).join(".config").join("systemd").join("user")
};
```

## Test Cases

### Default behavior (no XDG vars set)
```bash
tomat daemon install
# Uses ~/.config/systemd/user/tomat.service
```

### Custom XDG_CONFIG_HOME
```bash
export XDG_CONFIG_HOME=/custom/config/path
tomat daemon install
# Uses /custom/config/path/systemd/user/tomat.service
```

### Environment without dirs crate support
```bash
# Falls back to $HOME/.config gracefully
```

## Benefits

1. **XDG Compliance**: Respects user's XDG configuration directories
2. **Consistent**: Uses same directory resolution as config.rs
3. **Robust**: Graceful fallback if XDG directories aren't available
4. **Portable**: Works across different Linux distributions and setups

## Verification

The fix ensures both install and uninstall operations use the same directory resolution logic, preventing issues where files might be created in one location but searched for in another.