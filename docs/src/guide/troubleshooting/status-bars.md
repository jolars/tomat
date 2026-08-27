# Status Bar Integration Troubleshooting

## General Issues

### No Status Showing

#### Problem

The status bar does not display any tomat status.

#### Solution

1. **Ensure daemon is running**:

   ```bash
   tomat daemon status
   # Should show "Daemon is running"
   ```

2. Check that the PATH includes the directory where tomat is installed. See
   <https://github.com/jolars/tomat/issues/21> for instance.

## Waybar Integration Issues

### Status Not Updating

#### Problem

Waybar shows outdated or no tomat status.

#### Solution

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

#### Problem

Waybar shows parsing errors for tomat output.

#### Solution

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

3. **Check the disconnected status**:

   When the daemon is not running, `tomat status` returns valid JSON with the
   `disconnected` class. It does not write a connection error to the Waybar log.

   ```json
   {
     "text": "",
     "tooltip": "Tomat daemon is not running",
     "class": "disconnected",
     "percentage": 0.0
   }
   ```

### Styling Not Applied

#### Problem

Waybar shows tomat status but CSS styling doesn't work.

#### Solution

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
