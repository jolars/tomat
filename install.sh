#!/bin/bash
set -e

echo "Installing tomat..."

# Install the binary
cargo install --path .

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
echo "Note: Make sure ~/.cargo/bin is in your PATH"
echo "Add this to your shell profile if needed:"
echo "  export PATH=\"\$HOME/.cargo/bin:\$PATH\""