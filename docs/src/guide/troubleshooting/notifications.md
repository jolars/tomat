# Notification Issues

## No Desktop Notifications

### Problem

Timer works but no notifications appear.

### Solution

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

## Icon Not Showing in Notifications

### Problem

Notifications appear but without icons.

### Solution

Different solutions for different notification daemons:

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
   ls -la $XDG_CACHE_HOME/tomat/icon.png
   # Should exist if using "auto" mode
   ```
