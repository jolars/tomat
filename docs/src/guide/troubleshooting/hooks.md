# Hook Issues

## Hook Not Executing

### Problem

Configured hook command does not run when expected.

### Solution

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

## Permission Denied

### Problem

Hook fails with permission denied error.

### Solution

Check that the command has execute permissions:

```bash
chmod +x /home/user/scripts/my-script.sh
```

## Hook Timing Out

### Problem

Hook command is killed before completing.

### Solution

If your command takes longer than the default 5 seconds, increase the timeout:

```toml
[hooks.on_work_start]
cmd = "/home/user/scripts/slow-script.sh"
timeout = 10
```

Check if the command hangs when run manually. Ensure it doesn't require
interactive input.

## No Error Output

### Problem

Hook fails silently with no error messages.

### Solution

Enable output capture to see error messages:

```toml
[hooks.on_work_start]
cmd = "/usr/bin/my-command"
capture_output = true
```

## Working Directory Errors

### Problem

Hook fails because working directory is invalid or inaccessible.

### Solution

Verify the working directory exists and has correct permissions:

```bash
ls -ld /path/to/working/directory
```

Use absolute paths for both commands and working directories:

```toml
[hooks.on_work_start]
cmd = "/usr/bin/my-command"
cwd = "/home/user/project"
```

## Hook Not Triggered

### Problem

Hook is configured but never fires on the expected event.

### Solution

Check that the hook name is spelled correctly (e.g., `on_work_start`, not
`on_start_work`). After changing your config, restart the daemon:

```bash
tomat daemon stop
tomat daemon start
```

## Environment Variables Not Working

### Problem

Hook command cannot access `TOMAT_*` environment variables.

### Solution

Test your hook manually with environment variables:

```bash
TOMAT_EVENT=work_start TOMAT_PHASE=work /path/to/command
```

Verify your script or command is reading the correct variable names.
