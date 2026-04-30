#!/bin/bash
set -euo pipefail

# Assemble AppLoad-compatible package for reMarkable Paper Pro
# Output: build/remailable/ directory ready for deployment
#
# Usage: ./scripts/package.sh [path-to-binary]
#   If no binary path is given, defaults to the cross-compiled release binary.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BINARY_PATH="${1:-$PROJECT_ROOT/target/aarch64-unknown-linux-gnu/release/remailable}"
PKG_DIR="$PROJECT_ROOT/build/remailable"

echo "Assembling AppLoad package..."
rm -rf "$PKG_DIR"
mkdir -p "$PKG_DIR"

# Copy the cross-compiled binary
cp "$BINARY_PATH" "$PKG_DIR/remailable"
chmod +x "$PKG_DIR/remailable"

# Copy AppLoad manifest (external app format)
cp "$PROJECT_ROOT/packaging/external.manifest.json" "$PKG_DIR/external.manifest.json"

# Copy icon
cp "$PROJECT_ROOT/packaging/icon.png" "$PKG_DIR/icon.png"

# Copy QML files (loaded by binary at runtime from working directory)
mkdir -p "$PKG_DIR/qml"
cp "$PROJECT_ROOT/qml/"*.qml "$PKG_DIR/qml/"
cp "$PROJECT_ROOT/qml/"*.js "$PKG_DIR/qml/" 2>/dev/null || true

# Create tarball for easy deployment
cd "$PROJECT_ROOT/build"
tar czf remailable-appload.tar.gz -C . remailable/

echo "Package assembled at: $PKG_DIR"
echo "Tarball: $PROJECT_ROOT/build/remailable-appload.tar.gz"
echo ""
echo "To install on device, copy the remailable/ directory to:"
echo "  /home/root/xovi/exthome/appload/remailable/"