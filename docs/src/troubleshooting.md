# Troubleshooting Guide

This guide covers common issues and their solutions when using tomat.

## Installation Issues

### ALSA Library Missing

**Problem**: Audio notifications don't work, or you get ALSA-related errors.

**Solution**:

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

### Cargo Install Fails

**Problem**: `cargo install tomat` fails with compilation errors.

**Solutions**:

1. **Update Rust**: `rustup update stable`
2. **Check toolchain**: Ensure you're using Rust stable
3. **Install ALSA**: See ALSA section above
4. **Clear cache**: `cargo clean` if building from source

## Daemon Issues

### Daemon Won't Start

**Problem**: `tomat daemon start` fails or daemon exits immediately.

**Troubleshooting steps**:

1. **Check if already running**:

   ```bash
   tomat daemon status
   # If running, stop first: tomat daemon stop
   ```

2. **Check socket permissions**:

   ```bash
   ls -la $XDG_RUNTIME_DIR/tomat*
   # Should show socket and PID files with your user ownership
   ```

3. **Run daemon in foreground** to see errors:

   ```bash
   tomat daemon run
   # This shows all output directly
   ```

4. **Check runtime directory**:
   ```bash
   echo $XDG_RUNTIME_DIR
   # Should output something like /run/user/1000
   # If empty, daemon will fail to start
   ```

### Daemon Stops Unexpectedly

**Problem**: Daemon process dies or becomes unresponsive.

**Solutions**:

1. **Check system logs**:

   ```bash
   journalctl --user -u tomat.service -f  # If using systemd
   ```

2. **Check for multiple instances**:

   ```bash
   ps aux | grep tomat
   # Kill any duplicate processes
   ```

3. **Clean up stale files**:
   ```bash
   rm -f $XDG_RUNTIME_DIR/tomat.sock $XDG_RUNTIME_DIR/tomat.pid
   tomat daemon start
   ```

### Permission Errors

**Problem**: "Permission denied" when accessing socket or PID files.

**Solutions**:

1. **Check file ownership**:

   ```bash
   ls -la $XDG_RUNTIME_DIR/tomat*
   # Files should be owned by your user
   ```

2. **Ensure runtime directory exists**:

   ```bash
   mkdir -p $XDG_RUNTIME_DIR
   chmod 700 $XDG_RUNTIME_DIR
   ```

3. **Restart daemon**:
   ```bash
   tomat daemon stop
   tomat daemon start
   ```

## Configuration Issues

#### Config file not loading

Check that your config file is at the correct location:
`~/.config/tomat/config.toml`. You can verify the path exists:

```bash
ls -l ~/.config/tomat/config.toml
```

#### Syntax errors

TOML is whitespace-sensitive and requires proper quoting. Use a TOML validator
or check that brackets, quotes, and equal signs are balanced.

#### Permission denied

Ensure the config file is readable by your user:

```bash
chmod 644 ~/.config/tomat/config.toml
```

### Audio Issues

#### No sound playing

On Linux, audio requires ALSA (Advanced Linux Sound Architecture). If ALSA is
not available, tomat will automatically disable audio or fall back to the system
beep.

#### Custom sounds not working

Verify that your sound files exist and are in WAV format:

```bash
file /path/to/your/sound.wav
```

#### Volume too low or too high

Adjust the `volume` setting in your config file. Valid range is 0.0 (silent) to
1.0 (full volume):

```toml
[sound]
volume = 0.8
```

#### Fallback to system beep

If you're hearing the system beep instead of audio files, your system may not
have ALSA available. You can explicitly enable this behavior:

```toml
[sound]
system_beep = true
```

### Notification Issues

#### No notifications appearing

Check if your notification daemon is running. Common daemons include `dunst` and
`mako`. You can test with:

```bash
notify-send "Test" "Notification test"
```

#### Icon not showing

Try different icon modes. The `"auto"` mode works best with mako:

```toml
[notification]
icon = "auto"
```

Alternatively, use the system theme icon:

```toml
[notification]
icon = "theme"
```

#### Timeout too short

Increase the timeout value (in milliseconds):

```toml
[notification]
timeout = 10000
```

### Hook Issues

#### Hook not executing

Verify the command exists and is executable:

```bash
which playerctl
ls -l /home/user/scripts/my-script.sh
```

Use absolute paths in your hook configuration:

```toml
[hooks.on_work_start]
cmd = "/usr/bin/playerctl"
```

#### Permission denied

Check that the command has execute permissions:

```bash
chmod +x /home/user/scripts/my-script.sh
```

#### Hook timing out

If your command takes longer than the default 5 seconds, increase the timeout:

```toml
[hooks.on_work_start]
cmd = "/home/user/scripts/slow-script.sh"
timeout = 10
```

Check if the command hangs when run manually. Ensure it doesn't require
interactive input.

#### No error output

Enable output capture to see error messages:

```toml
[hooks.on_work_start]
cmd = "/usr/bin/my-command"
capture_output = true
```

#### Working directory errors

Verify the working directory exists and has correct permissions:

```bash
ls -ld /path/to/working/directory
```

Use absolute paths for both commands and working directories.

#### Hook not triggered

Check that the hook name is spelled correctly (e.g., `on_work_start`, not
`on_start_work`). After changing your config, restart the daemon:

```bash
tomat daemon stop
tomat daemon start
```

#### Environment variables not working

Test your hook manually with environment variables:

```bash
TOMAT_EVENT=work_start TOMAT_PHASE=work /path/to/command
```

Verify your script or command is reading the correct variable names.

## Audio Issues

### No Sound Notifications

**Problem**: Timer works but no audio plays during transitions.

**Solutions**:

1. **Check audio configuration**:

   ```toml
   # ~/.config/tomat/config.toml
   [sound]
   enabled = true  # Must be true
   ```

2. **Test system audio**:

   ```bash
   # Test if ALSA works
   aplay /usr/share/sounds/alsa/Front_Left.wav

   # Or try speaker-test
   speaker-test -t sine -f 1000 -l 1
   ```

3. **Check volume levels**:
   - System volume (alsamixer, pavucontrol)
   - Tomat volume in config (0.0-1.0)

4. **Try different audio modes**:
   ```toml
   [sound]
   system_beep = true  # Use system beep instead
   ```

### Wrong Audio Device

**Problem**: Audio plays on wrong device or not audible.

**Solutions**:

1. **Check default ALSA device**:

   ```bash
   aplay -l  # List audio devices
   cat ~/.asoundrc  # Check ALSA configuration
   ```

2. **Use system beep as fallback**:
   ```toml
   [sound]
   system_beep = true
   ```

### Custom Sound Files Not Working

**Problem**: Custom sound files don't play.

**Solutions**:

1. **Check file paths and existence**:

   ```bash
   ls -la /path/to/your/sound.wav
   ```

2. **Verify file format** (must be WAV):

   ```bash
   file /path/to/your/sound.wav
   # Should show: RIFF (little-endian) data, WAVE audio
   ```

3. **Test file with system player**:

   ```bash
   aplay /path/to/your/sound.wav
   ```

4. **Use absolute paths**:
   ```toml
   [sound]
   work_to_break = "/home/user/sounds/work-done.wav"  # Absolute path
   ```

## Notification Issues

### No Desktop Notifications

**Problem**: Timer works but no notifications appear.

**Solutions**:

1. **Check notification daemon**:

   ```bash
   # For mako (Wayland)
   ps aux | grep mako

   # For dunst (X11)
   ps aux | grep dunst

   # Test notifications
   notify-send "Test" "Notification test"
   ```

2. **Check tomat notification config**:

   ```toml
   [notification]
   enabled = true  # Must be true
   ```

3. **Try different icon modes**:
   ```toml
   [notification]
   icon = "theme"  # Try system theme icon
   ```

### Icon Not Showing in Notifications

**Problem**: Notifications appear but without icons.

**Solutions for different notification daemons**:

1. **Mako**: Use auto mode (default)

   ```toml
   [notification]
   icon = "auto"  # Uses cached embedded icon
   ```

2. **Dunst**: Try theme mode

   ```toml
   [notification]
   icon = "theme"  # Uses system "timer" icon
   ```

3. **Custom icon**:

   ```toml
   [notification]
   icon = "/usr/share/icons/hicolor/48x48/apps/timer.png"
   ```

4. **Check icon cache location**:
   ```bash
   ls -la ~/.cache/tomat/icon.png
   # Should exist if using "auto" mode
   ```

## Waybar Integration Issues

### Status Not Updating

**Problem**: Waybar shows outdated or no tomat status.

**Solutions**:

1. **Check daemon status**:

   ```bash
   tomat daemon status
   # Should show "Daemon is running"
   ```

2. **Test status command directly**:

   ```bash
   tomat status
   # Should return JSON with current status
   ```

3. **Check waybar configuration**:

   ```json
   {
     "custom/tomat": {
       "exec": "tomat status",
       "interval": 1, // Update every second
       "return-type": "json" // Required for proper parsing
     }
   }
   ```

4. **Restart waybar**:
   ```bash
   killall waybar && waybar &
   ```

### JSON Parsing Errors

**Problem**: Waybar shows parsing errors for tomat output.

**Solutions**:

1. **Verify JSON output**:

   ```bash
   tomat status | jq .
   # Should show properly formatted JSON
   ```

2. **Check for daemon errors**:

   ```bash
   tomat daemon stop
   tomat daemon run  # Run in foreground to see errors
   ```

3. **Update waybar config**:
   ```json
   {
     "custom/tomat": {
       "exec": "tomat status 2>/dev/null || echo '{\"text\":\"🍅 Error\"}'",
       "return-type": "json"
     }
   }
   ```

### Styling Not Applied

**Problem**: Waybar shows tomat status but CSS styling doesn't work.

**Solutions**:

1. **Check CSS class names**:

   ```bash
   tomat status | jq .class
   # Should return: "work", "work-paused", "break", etc.
   ```

2. **Verify CSS selectors** in waybar style:

   ```css
   #custom-tomat.work {
     background-color: #ff6b6b;
   }

   #custom-tomat.work-paused {
     background-color: #ff9999;
   }
   ```

3. **Test with simple styling**:
   ```css
   #custom-tomat {
     background-color: red; /* Should always apply */
   }
   ```

## Systemd Integration Issues

### Service Fails to Start

**Problem**: `systemctl --user start tomat.service` fails.

**Solutions**:

1. **Check service file location**:

   ```bash
   ls -la ~/.config/systemd/user/tomat.service
   ```

2. **Verify service file content**:

   ```bash
   cat ~/.config/systemd/user/tomat.service
   # Should contain: ExecStart=%h/.cargo/bin/tomat daemon run
   ```

3. **Check service status**:

   ```bash
   systemctl --user status tomat.service
   journalctl --user -u tomat.service -f
   ```

4. **Reload systemd configuration**:
   ```bash
   systemctl --user daemon-reload
   systemctl --user restart tomat.service
   ```

### Service Starts But Daemon Not Accessible

**Problem**: Service is running but `tomat status` fails.

**Solutions**:

1. **Check if daemon is actually running**:

   ```bash
   ps aux | grep tomat
   ```

2. **Verify socket creation**:

   ```bash
   ls -la $XDG_RUNTIME_DIR/tomat.sock
   ```

3. **Check service logs**:
   ```bash
   journalctl --user -u tomat.service --no-pager
   ```

## General Debugging

### Enable Debug Output

Run daemon in foreground to see all output:

```bash
tomat daemon stop
tomat daemon run  # Shows all debug output
```

### Check File Permissions

Ensure all tomat files have correct permissions:

```bash
# Runtime files
ls -la $XDG_RUNTIME_DIR/tomat*

# Config files
ls -la ~/.config/tomat/

# Cache files
ls -la ~/.cache/tomat/
```

### Clean State Reset

To completely reset tomat state:

```bash
# Stop daemon
tomat daemon stop

# Remove all tomat files
rm -f $XDG_RUNTIME_DIR/tomat.*
rm -rf ~/.cache/tomat/

# Restart
tomat daemon start
```

## Getting Help

If you're still experiencing issues:

1. **Check GitHub Issues**: Search existing issues for your problem
2. **Create New Issue**: Include:
   - Operating system and version
   - Tomat version (`tomat --version`)
   - Error messages and logs
   - Configuration file content
   - Steps to reproduce

3. **Provide Debug Information**:
   ```bash
   # Include this output in your issue
   echo "=== System Info ==="
   uname -a
   echo "=== Tomat Version ==="
   tomat --version
   echo "=== Runtime Dir ==="
   echo $XDG_RUNTIME_DIR
   ls -la $XDG_RUNTIME_DIR/tomat* 2>/dev/null || echo "No tomat files"
   echo "=== Config ==="
   cat ~/.config/tomat/config.toml 2>/dev/null || echo "No config file"
   echo "=== Daemon Status ==="
   tomat daemon status 2>&1
   ```
