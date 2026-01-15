# Status Bar Integration Troubleshooting

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
