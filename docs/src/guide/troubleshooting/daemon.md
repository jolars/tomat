# Daemon Issues

## Daemon Won't Start

### Problem

`tomat daemon start` fails or daemon exits immediately.

### Solution

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

## Daemon Stops Unexpectedly

### Problem

Daemon process dies or becomes unresponsive.

### Solution

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

## Permission Errors

### Problem

"Permission denied" when accessing socket or PID files.

### Solution

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
