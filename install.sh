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

# Install systemd service using tomat's built-in command
echo ""
echo "Installing systemd service..."
tomat daemon install

echo ""
echo "Installation complete!"
echo ""
echo "To view the manual:"
echo "  man tomat"
echo ""
echo "Note: Make sure ~/.cargo/bin is in your PATH"
echo "Add this to your shell profile if needed:"
echo "  export PATH=\"\$HOME/.cargo/bin:\$PATH\""