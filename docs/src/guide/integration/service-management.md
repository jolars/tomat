# Service Management

Tomat uses a daemon (server) to manager its timers in the background. 
Most users will want to set up tomat to start automatically when they log in.
This is typically done using `systemd` user services on Linux systems.

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
