# Installation Issues

## ALSA Library Missing

### Problem

Audio notifications don't work, or you get ALSA-related errors.

### Solution

```bash
# Ubuntu/Debian
sudo apt-get install libasound2-dev

# Fedora/RHEL
sudo dnf install alsa-lib-devel

# Arch Linux
sudo pacman -S alsa-lib
```

**Note**: Audio will be automatically disabled if ALSA is not available. The
timer will still work normally with desktop notifications only.

## Cargo Install Fails

### Problem 

`cargo install tomat` fails with compilation errors.

### Solution

Try the following steps:

1. **Update Rust**: `rustup update stable`
2. **Check toolchain**: Ensure you're using Rust stable
3. **Install ALSA**: See ALSA section above
4. **Clear cache**: `cargo clean` if building from source
