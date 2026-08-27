# Service Management

Tomat uses a daemon (server) to manage its timers in the background. Most users
will want Tomat to start automatically when they log in. The built-in installer
uses systemd user services on Linux and LaunchAgents on macOS.

## Automatic Setup

Install and enable the service for the current platform:

```bash
tomat daemon install
```

Remove it again with:

```bash
tomat daemon uninstall
```

Pass `--force` to overwrite an existing service file without being asked; this
is required when the command runs without a terminal, such as from a
provisioning script.

The generated service file records `TOMAT_RUNTIME_DIR` if it is set in the
environment at install time, because the service manager starts the daemon with
a minimal environment and would otherwise put the daemon and its clients on
different sockets. Re-run `tomat daemon install --force` after changing the
variable.

## Systemd

### Setup

Install the service file and enable auto-start:

```bash
# Copy service file
curl -o ~/.config/systemd/user/tomat.service https://raw.githubusercontent.com/jolars/tomat/main/assets/tomat.service

# Enable auto-start
systemctl --user enable tomat.service
systemctl --user start tomat.service
```

If you prefer to create the service file manually
(`~/.config/systemd/user/tomat.service`):

```ini
[Unit]
Description=Tomat Pomodoro Timer Daemon
After=graphical-session.target

[Service]
Type=simple
ExecStart=%h/.cargo/bin/tomat daemon run
Restart=on-failure
RestartSec=5

[Install]
WantedBy=default.target
```

### Management

The service is managed using standard `systemctl` commands:

```bash
# Check status
systemctl --user status tomat.service

# View logs
journalctl --user -u tomat.service -f

# Restart service
systemctl --user restart tomat.service

# Disable auto-start
systemctl --user disable tomat.service
```

The systemd user manager is torn down at logout, so the service will not start
at boot or before you log in graphically unless the account allows lingering:

```bash
loginctl enable-linger $USER
```

The unit uses `Restart=on-failure`, so `tomat daemon stop` (or
`systemctl --user stop tomat.service`) keeps the daemon stopped, while a crash
still brings it back.

## launchd

On macOS, `tomat daemon install` writes
`~/Library/LaunchAgents/io.github.jolars.tomat.plist`, loads it into the current
GUI session, and starts the daemon. The agent uses `KeepAlive` with
`SuccessfulExit` set to false, so it restarts Tomat after a crash but leaves it
alone after `tomat daemon stop`.

Daemon errors are appended to `~/Library/Logs/tomat.log`. Nothing rotates that
file, so truncate it yourself if a repeatedly failing daemon lets it grow.

Use `launchctl` to inspect or restart it:

```bash
# Check status and recent exit information
launchctl print gui/$UID/io.github.jolars.tomat

# Restart the daemon managed by launchd
launchctl kickstart -k gui/$UID/io.github.jolars.tomat

# Inspect daemon errors
tail -f ~/Library/Logs/tomat.log
```

Use `tomat daemon uninstall` to unload the LaunchAgent and remove its plist.
