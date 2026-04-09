#!/usr/bin/env sh
set -e

INSTALL_DIR="/usr/local/bin"
BINARY="$INSTALL_DIR/clawsh"

echo "Building clawsh..."
cargo build --release

echo "Installing to $BINARY..."
sudo cp target/release/clawsh "$BINARY"
sudo chmod +x "$BINARY"

# Register as a valid login shell
if ! grep -qxF "$BINARY" /etc/shells; then
    echo "Registering clawsh in /etc/shells..."
    echo "$BINARY" | sudo tee -a /etc/shells > /dev/null
fi

echo ""
echo "clawsh installed successfully!"
echo ""
echo "To set clawsh as your default shell:"
echo "  chsh -s $BINARY"
echo ""
echo "Or just run it now:"
echo "  clawsh"
