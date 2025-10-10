#!/bin/bash
# Installation script for Ramble Daemon

set -e

echo "Ramble Daemon Installation"
echo "==========================="
echo ""

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust/Cargo is not installed."
    echo "Please install Rust from https://rustup.rs/"
    exit 1
fi

# Build the daemon
echo "Building ramble-daemon..."
cargo build --release

# Determine installation directory
if [ -w "/usr/local/bin" ]; then
    INSTALL_DIR="/usr/local/bin"
else
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
fi

# Copy binary
echo "Installing to $INSTALL_DIR..."
cp target/release/ramble-daemon "$INSTALL_DIR/"

# Make executable
chmod +x "$INSTALL_DIR/ramble-daemon"

# Setup config
echo ""
echo "Running setup..."
"$INSTALL_DIR/ramble-daemon" setup

echo ""
echo "✓ Installation complete!"
echo ""
echo "Next steps:"
echo "1. Edit the config file at: ~/.config/ramble/config.toml"
echo "2. Set your username and server URL"
echo "3. Run 'ramble-daemon' to start the daemon"
echo ""
echo "Optional: Set up auto-start with systemd"
echo "See README.md for instructions"
