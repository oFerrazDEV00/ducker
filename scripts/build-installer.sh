#!/bin/bash
# =============================================================================
# Ducker Installer Build Script (Linux)
# "Be Simple, Be Duck."
#
# This script:
# 1. Builds the CLI binary in release mode
# 2. Copies the CLI binary to the Tauri resources directory
# 3. Builds the Tauri desktop app with AppImage and .deb packages
# 4. Creates a standalone CLI tarball
#
# Usage:
#   scripts/build-installer.sh            (release build)
#   scripts/build-installer.sh --debug    (debug build for testing)
#
# Prerequisites:
#   - Rust toolchain
#   - Node.js 20+
#   - System deps: libwebkit2gtk-4.1-dev, libgtk-3-dev, libayatana-appindicator3-dev
# =============================================================================

set -euo pipefail

echo ""
echo " ===================================="
echo "  Ducker Installer Builder (Linux)"
echo "  Be Simple, Be Duck."
echo " ===================================="
echo ""

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BUILD_MODE="release"
BUILD_FLAG="--release"
TAURI_BUILD_FLAG=""

# Parse arguments
if [[ "${1:-}" == "--debug" ]]; then
    BUILD_MODE="debug"
    BUILD_FLAG=""
    TAURI_BUILD_FLAG="--debug"
    echo "[INFO] Building in DEBUG mode"
else
    echo "[INFO] Building in RELEASE mode"
fi

# Step 1: Build the CLI binary
echo ""
echo "[1/4] Building Ducker CLI..."
cargo build -p ducker-cli $BUILD_FLAG
echo "[OK] CLI built successfully"

# Step 2: Copy CLI binary to Tauri resources
echo ""
echo "[2/4] Preparing installer resources..."

CLI_SOURCE="$PROJECT_ROOT/target/$BUILD_MODE/ducker"
RESOURCES_DIR="$PROJECT_ROOT/app/src-tauri/resources"

mkdir -p "$RESOURCES_DIR"

if [[ ! -f "$CLI_SOURCE" ]]; then
    echo "[ERROR] CLI binary not found: $CLI_SOURCE"
    exit 1
fi

cp "$CLI_SOURCE" "$RESOURCES_DIR/ducker"
chmod +x "$RESOURCES_DIR/ducker"
echo "[OK] CLI binary copied to resources"

cp "$PROJECT_ROOT/LICENSE" "$RESOURCES_DIR/LICENSE"
echo "[OK] LICENSE copied to resources"

# Step 3: Build the Tauri app with packages
echo ""
echo "[3/4] Building Ducker Desktop packages..."
echo "      (this may take several minutes on first build)"
echo ""

cd "$PROJECT_ROOT/app"

# Check if npm dependencies are installed
if [[ ! -d "node_modules" ]]; then
    echo "[INFO] Installing npm dependencies..."
    npm install
fi

# Build with Tauri
npx tauri build $TAURI_BUILD_FLAG

# Step 4: Create standalone CLI tarball
echo ""
echo "[4/4] Creating standalone CLI tarball..."

CLI_DIST_DIR="$PROJECT_ROOT/target/$BUILD_MODE/bundle/cli"
mkdir -p "$CLI_DIST_DIR"

# Determine architecture
ARCH=$(uname -m)
VERSION=$(grep '^version' "$PROJECT_ROOT/cli/Cargo.toml" | head -1 | sed 's/.*"\(.*\)"/\1/')

TARBALL_NAME="ducker-cli-${VERSION}-linux-${ARCH}.tar.gz"
TARBALL_DIR=$(mktemp -d)
mkdir -p "$TARBALL_DIR/ducker-cli"

cp "$CLI_SOURCE" "$TARBALL_DIR/ducker-cli/ducker"
chmod +x "$TARBALL_DIR/ducker-cli/ducker"
cp "$PROJECT_ROOT/LICENSE" "$TARBALL_DIR/ducker-cli/"
cp "$PROJECT_ROOT/README.md" "$TARBALL_DIR/ducker-cli/"

# Create install script for CLI
cat > "$TARBALL_DIR/ducker-cli/install.sh" << 'INSTALL_EOF'
#!/bin/bash
# Ducker CLI Installer — Be Simple, Be Duck.
set -e

PREFIX="${1:-$HOME/.local}"
BIN_DIR="$PREFIX/bin"

echo "Installing Ducker CLI to $BIN_DIR..."
mkdir -p "$BIN_DIR"
cp ducker "$BIN_DIR/ducker"
chmod +x "$BIN_DIR/ducker"

echo ""
echo "Ducker CLI installed successfully!"
echo ""
echo "Make sure $BIN_DIR is in your PATH:"
echo "  export PATH=\"$BIN_DIR:\$PATH\""
echo ""
echo "Then run: ducker --help"
echo ""
echo "🦆 Be Simple, Be Duck."
INSTALL_EOF
chmod +x "$TARBALL_DIR/ducker-cli/install.sh"

tar -czf "$CLI_DIST_DIR/$TARBALL_NAME" -C "$TARBALL_DIR" ducker-cli
rm -rf "$TARBALL_DIR"
echo "[OK] CLI tarball created: $CLI_DIST_DIR/$TARBALL_NAME"

# Summary
echo ""
echo " ===================================="
echo "  Build Complete!"
echo " ===================================="
echo ""
echo "[OUTPUT] Generated artifacts:"
echo ""

BUNDLE_DIR="$PROJECT_ROOT/target/$BUILD_MODE/bundle"

if [[ -d "$BUNDLE_DIR/appimage" ]]; then
    echo " AppImage:"
    ls -lh "$BUNDLE_DIR/appimage/"*.AppImage 2>/dev/null || echo "   (none found)"
fi

if [[ -d "$BUNDLE_DIR/deb" ]]; then
    echo " Debian package:"
    ls -lh "$BUNDLE_DIR/deb/"*.deb 2>/dev/null || echo "   (none found)"
fi

echo " CLI tarball:"
ls -lh "$CLI_DIST_DIR/$TARBALL_NAME"

echo ""
echo " 🦆 Be Simple, Be Duck."
echo ""
