# Systemd Service Integration Example

This example shows how the new `tomat daemon install` command works.

## Before: Manual Installation

Previously, users had to manually set up the systemd service:

```bash
# Install tomat
cargo install tomat

# Manual systemd setup (error-prone)
mkdir -p ~/.config/systemd/user
curl -o ~/.config/systemd/user/tomat.service https://raw.githubusercontent.com/jolars/tomat/main/tomat.service
systemctl --user daemon-reload
systemctl --user enable tomat.service
systemctl --user start tomat.service
```

## After: Built-in Installation

Now it's just two commands:

```bash
# Install tomat
cargo install tomat

# Automatic systemd setup
tomat daemon install
systemctl --user start tomat.service
```

## What the install command does

1. **Detects current executable path** - Works with any installation method
2. **Generates service file** - Uses the actual installed binary path
3. **Creates systemd directory** - `~/.config/systemd/user/`
4. **Installs service file** - `tomat.service` with correct ExecStart path
5. **Reloads systemd** - `systemctl --user daemon-reload`
6. **Enables service** - `systemctl --user enable tomat.service`

## Generated service file

The command generates a service file like this:

```ini
[Unit]
Description=Tomat Pomodoro Timer Daemon
After=graphical-session.target

[Service]
Type=simple
ExecStart=/home/user/.cargo/bin/tomat daemon run
Restart=always
RestartSec=5

[Install]
WantedBy=default.target
```

## Benefits

- **No hardcoded paths** - Works with any installation location
- **Fewer steps** - Reduces user error
- **Self-contained** - No external dependencies or scripts
- **Easy uninstall** - `tomat daemon uninstall` removes everything
- **Error handling** - Graceful fallback if systemctl isn't available

## Usage with different package managers

### Cargo
```bash
cargo install tomat
tomat daemon install
```

### Future: AUR, Homebrew, etc.
```bash
# Any package manager that installs to PATH
pacman -S tomat  # (hypothetical)
tomat daemon install  # Works regardless of install location
```