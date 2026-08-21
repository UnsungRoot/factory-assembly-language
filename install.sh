#!/bin/sh
# Factory Assembly Language (FAL) & FALZ Universal Installer
# Compatible with Linux, macOS, and POSIX Unix systems.

set -e

FAL_HOME="${FALZ_HOME:-$HOME/.falz}"
FAL_BIN_DIR="$FAL_HOME/bin"

echo "========================================================"
echo "   Factory Assembly Language (FAL) Universal Installer   "
echo "========================================================"

# 1. Detect Operating System & CPU Architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

echo "[1/4] Detecting host hardware environment..."
echo "      Operating System : $OS"
echo "      CPU Architecture : $ARCH"

case "$ARCH" in
    x86_64|amd64)
        FAL_ARCH="x86_64"
        ;;
    aarch64|arm64)
        FAL_ARCH="aarch64"
        ;;
    *)
        echo "Warning: Untested architecture $ARCH, proceeding with generic build."
        FAL_ARCH="$ARCH"
        ;;
esac

# 2. Create FALZ Environment Directories
echo "[2/4] Initializing FALZ global storage path at $FAL_HOME..."
mkdir -p "$FAL_BIN_DIR"
mkdir -p "$FAL_HOME/cache"
mkdir -p "$FAL_HOME/storeroom"
mkdir -p "$FAL_HOME/logs"

# 3. Build or Install the FAL Binary
echo "[3/4] Installing FAL executable into $FAL_BIN_DIR/fal..."

# If building from local repo:
if [ -f "./Cargo.toml" ] && command -v cargo >/dev/null 2>&1; then
    cargo build --release --quiet
    cp ./target/release/fal "$FAL_BIN_DIR/fal"
elif [ -f "./fal" ]; then
    cp ./fal "$FAL_BIN_DIR/fal"
else
    echo "Error: FAL binary or Cargo build system not found!"
    exit 1
fi

chmod +x "$FAL_BIN_DIR/fal"

# 4. Configure Shell PATH
echo "[4/4] Configuring system PATH..."

SHELL_PROFILE=""
if [ -n "$ZSH_VERSION" ] || [ "$(basename "$SHELL")" = "zsh" ]; then
    SHELL_PROFILE="$HOME/.zshrc"
elif [ -n "$BASH_VERSION" ] || [ "$(basename "$SHELL")" = "bash" ]; then
    if [ -f "$HOME/.bashrc" ]; then
        SHELL_PROFILE="$HOME/.bashrc"
    else
        SHELL_PROFILE="$HOME/.bash_profile"
    fi
else
    SHELL_PROFILE="$HOME/.profile"
fi

EXPORT_LINE="export PATH=\"\$HOME/.falz/bin:\$PATH\""

if [ -f "$SHELL_PROFILE" ]; then
    if ! grep -q ".falz/bin" "$SHELL_PROFILE" 2>/dev/null; then
        echo "" >> "$SHELL_PROFILE"
        echo "# Factory Assembly Language (FAL)" >> "$SHELL_PROFILE"
        echo "$EXPORT_LINE" >> "$SHELL_PROFILE"
        echo "      Added $FAL_BIN_DIR to $SHELL_PROFILE"
    else
        echo "      PATH already configured in $SHELL_PROFILE"
    fi
fi

# Run dynamic hardware detection verification
export PATH="$FAL_BIN_DIR:$PATH"
echo ""
"$FAL_BIN_DIR/fal" env

echo "========================================================"
echo "  Installation Complete! FAL & FALZ are ready to use.   "
echo "========================================================"
echo ""
echo "To start coding right away:"
echo "   1. Refresh your shell:"
echo "      source $SHELL_PROFILE"
echo "   2. Create a new factory project:"
echo "      fal new my_first_app"
echo "   3. Run it:"
echo "      cd my_first_app && fal run factory.fal"
echo ""
