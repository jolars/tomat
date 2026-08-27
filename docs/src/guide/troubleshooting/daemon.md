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
   runtime_dir="${XDG_RUNTIME_DIR:-${TMPDIR:-/tmp}/tomat-$(id -u)}"
   ls -la "$runtime_dir"/tomat*
   # Should show socket and PID files with your user ownership
   ```

3. **Run daemon in foreground** to see errors:

   ```bash
   tomat daemon run
   # This shows all output directly
   ```

4. **Check runtime directory**:

   ```bash
   echo "${XDG_RUNTIME_DIR:-${TMPDIR:-/tmp}/tomat-$(id -u)}"
   ```

   Linux normally supplies `$XDG_RUNTIME_DIR`. macOS uses a Tomat directory
   beneath the per-user `$TMPDIR`. Set `TOMAT_RUNTIME_DIR` to override both.
   Tomat creates the directory with mode `700` when it is missing, but it never
   changes the permissions of a directory it did not create: if the daemon
   reports that the runtime directory is writable by other users, tighten it
   yourself with `chmod go-w` or point `TOMAT_RUNTIME_DIR` somewhere private.

## Daemon Stops Unexpectedly

### Problem

Daemon process dies or becomes unresponsive.

### Solution

1. **Check system logs**:

   ```bash
   journalctl --user -u tomat.service -f  # If using systemd
   tail -f ~/Library/Logs/tomat.log        # If using launchd on macOS
   ```

2. **Check for multiple instances**:

   ```bash
   ps aux | grep tomat
   # Kill any duplicate processes
   ```

3. **Clean up stale files**:

   ```bash
   runtime_dir="${XDG_RUNTIME_DIR:-${TMPDIR:-/tmp}/tomat-$(id -u)}"
   rm -f "$runtime_dir/tomat.sock" "$runtime_dir/tomat.pid"
   tomat daemon start
   ```

## Permission Errors

### Problem

"Permission denied" when accessing socket or PID files.

### Solution

1. **Check file ownership**:

   ```bash
   runtime_dir="${XDG_RUNTIME_DIR:-${TMPDIR:-/tmp}/tomat-$(id -u)}"
   ls -la "$runtime_dir"/tomat*
   # Files should be owned by your user
   ```

2. **Ensure runtime directory exists**:

   ```bash
   runtime_dir="${XDG_RUNTIME_DIR:-${TMPDIR:-/tmp}/tomat-$(id -u)}"
   mkdir -p "$runtime_dir"
   chmod 700 "$runtime_dir"
   ```

3. **Restart daemon**:

   ```bash
   tomat daemon stop
   tomat daemon start
   ```
