# Service Management Troubleshooting

## Service Fails to Start

### Problem

`systemctl --user start tomat.service` fails.

### Solution

1. **Check service file location**:

   ```bash
   ls -la $XDG_CONFIG_HOME/systemd/user/tomat.service
   ```

2. **Verify service file content**:

   ```bash
   cat $XDG_CONFIG_HOME/systemd/user/tomat.service
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

## Service Starts But Daemon Not Accessible

### Problem

Service is running but `tomat status` fails.

### Solution

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

## Enable Debug Output

### Problem

Need to see detailed daemon output for debugging.

### Solution

Run daemon in foreground to see all output:

```bash
tomat daemon stop
tomat daemon run  # Shows all debug output
```
