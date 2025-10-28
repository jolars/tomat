# tomat

A Pomodoro timer with daemon support for waybar and other status bars.

## Features

- **Server/Client Architecture**: Robust daemon that survives waybar restarts
- **JSON Output**: Perfect for waybar integration
- **Unix Socket Communication**: Fast, secure local IPC
- **Systemd Integration**: Auto-start with user session
- **Minimal Resource Usage**: Lightweight daemon

## Installation

### Quick Install (Recommended)

```bash
git clone https://github.com/jolars/tomat.git
cd tomat
./install.sh
```

This will install the binary and set up the systemd service automatically.

### Manual Installation

#### From source

```bash
git clone https://github.com/jolars/tomat.git
cd tomat
cargo install --path .

# Set up systemd service
mkdir -p ~/.config/systemd/user
cp tomat.service ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable tomat.service
systemctl --user start tomat.service
```

#### Via package managers (when available)

```bash
# Debian/Ubuntu (future)
# sudo apt install tomat

# Arch Linux (future)
# yay -S tomat
```

**Note**: Ensure `~/.cargo/bin` is in your PATH. Add this to your shell profile if needed:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

### Creating a Debian package (for maintainers)

```bash
cargo install cargo-deb
cargo deb
```

This creates a `.deb` package that properly installs the systemd service.

## Usage

### Basic Commands

```bash
# Start a Pomodoro session (25min work, 5min break by default)
tomat start

# Start with custom durations
tomat start --work 30 --break-time 10

# Get current status (JSON format for waybar)
tomat status

# Skip to next phase (work -> break -> work)
tomat skip

# Stop current session
tomat stop
```

### Waybar Integration

Add this module to your waybar config:

```json
"custom/pomodoro": {
    "format": "{}",
    "exec": "tomat status",
    "return-type": "json",
    "interval": 1,
    "on-click": "tomat skip",
    "on-click-right": "tomat stop",
    "on-click-middle": "tomat start"
}
```

### CSS Classes

The status output includes these CSS classes for styling:

- `idle`: Timer not running
- `work`: Work session active
- `break`: Break session active

Example waybar CSS:

```css
#custom-pomodoro.work {
  background: #2d5a27;
  color: #ffffff;
}

#custom-pomodoro.break {
  background: #8b4513;
  color: #ffffff;
}

#custom-pomodoro.idle {
  background: #404040;
  color: #888888;
}
```

## JSON Output Format

```json
{
  "text": "🍅 24:30",
  "tooltip": "Work - 25min",
  "class": "work",
  "percentage": 2.0
}
```

## Architecture

- **Daemon** (`tomat daemon`): Runs continuously, manages timer state
- **Client** (`tomat <command>`): Sends commands to daemon via Unix socket
- **Socket**: Located at `$XDG_RUNTIME_DIR/tomat.sock` (typically `/run/user/$UID/tomat.sock`)

This architecture ensures the timer survives waybar restarts, system suspend/resume, and provides accurate timing.
