#!/bin/bash
set -euo pipefail

# Assemble AppLoad-compatible package for reMarkable Paper Pro
# Output: build/remailable/ directory ready for deployment
#
# AppLoad QML app format:
#   manifest.json       — QML app configuration
#   icon.png             — App icon
#   qml/main.qml        — QML frontend entry point
#   qml/*.qml           — Additional QML components
#   backend/remailable-backend — Headless Rust backend
#   backend/entry       — Shell wrapper that passes socket path

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BACKEND_PATH="${1:-$PROJECT_ROOT/target/aarch64-unknown-linux-gnu/release/remailable-backend}"
PKG_DIR="$PROJECT_ROOT/build/remailable"

echo "Assembling AppLoad package..."
rm -rf "$PKG_DIR"
mkdir -p "$PKG_DIR" "$PKG_DIR/qml" "$PKG_DIR/backend"

# Copy AppLoad manifest (QML app format)
cp "$PROJECT_ROOT/packaging/manifest.json" "$PKG_DIR/manifest.json"

# Copy icon
cp "$PROJECT_ROOT/packaging/icon.png" "$PKG_DIR/icon.png"

# Copy QML frontend files (new AppLoad frontend, not CXX-Qt)
cp "$PROJECT_ROOT/qml/frontend/"*.qml "$PKG_DIR/qml/"

# Copy backend binary
cp "$BACKEND_PATH" "$PKG_DIR/backend/remailable-backend"
chmod +x "$PKG_DIR/backend/remailable-backend"

# Copy backend entry script
cp "$PROJECT_ROOT/packaging/backend-entry.sh" "$PKG_DIR/backend/entry"
chmod +x "$PKG_DIR/backend/entry"

# Keep the native binary for direct launch testing (optional)
if [ -f "$PROJECT_ROOT/target/aarch64-unknown-linux-gnu/release/remailable" ]; then
    cp "$PROJECT_ROOT/target/aarch64-unknown-linux-gnu/release/remailable" "$PKG_DIR/remailable"
    chmod +x "$PKG_DIR/remailable"
    cp "$PROJECT_ROOT/packaging/external.manifest.json" "$PKG_DIR/external.manifest.json"
fi

# Create tarball for easy deployment
cd "$PROJECT_ROOT/build"
tar czf remailable-appload.tar.gz -C . remailable/

echo "Package assembled at: $PKG_DIR"
echo "Tarball: $PROJECT_ROOT/build/remailable-appload.tar.gz"
echo ""
echo "To install on device, copy the remailable/ directory to:"
echo "  /home/root/xovi/exthome/appload/remailable/"