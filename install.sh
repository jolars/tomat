#!/bin/bash
set -e

echo "Installing tomat..."

# Build with release profile (which also generates man pages)
cargo build --release

# Install the binary
cargo install --path .

# Install man page
MAN_DIR="$HOME/.local/share/man/man1"
mkdir -p "$MAN_DIR"
if [ -f "target/man/tomat.1" ]; then
    cp target/man/tomat.1 "$MAN_DIR/"
    echo "Man page installed to $MAN_DIR/tomat.1"
fi

# Ensure ~/.config/systemd/user directory exists
mkdir -p ~/.config/systemd/user

# Install systemd service
cp tomat.service ~/.config/systemd/user/

# Reload systemd and enable the service
systemctl --user daemon-reload
systemctl --user enable tomat.service

echo "Installation complete!"
echo ""
echo "To start the daemon:"
echo "  systemctl --user start tomat.service"
echo ""
echo "To check status:"
echo "  systemctl --user status tomat.service"
echo ""
echo "To view the manual:"
echo "  man tomat"
echo ""
echo "Note: Make sure ~/.cargo/bin is in your PATH"
echo "Add this to your shell profile if needed:"
echo "  export PATH=\"\$HOME/.cargo/bin:\$PATH\""