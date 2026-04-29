#!/bin/bash
set -euo pipefail

# Copy QML files to the package directory for AppLoad external app format.
#
# CXX-Qt compiles QML into the binary via qrc at build time, so this step
# is not strictly required for the binary to find its QML. However, copying
# the QML source files into the package allows:
#   - Runtime loading if the qrc path is unavailable
#   - Future QML hot-reload during development
#   - Inspection of QML sources on the device
#
# For the AppLoad external app format, the binary loads QML from qrc by default.
# The copied qml/ directory can serve as a fallback if the binary is modified
# to load from the filesystem instead.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
PKG_DIR="$PROJECT_ROOT/build/remailable"

mkdir -p "$PKG_DIR/qml"
cp -r "$PROJECT_ROOT/qml/"*.qml "$PKG_DIR/qml/"
cp "$PROJECT_ROOT/qml/"*.js "$PKG_DIR/qml/" 2>/dev/null || true

echo "Copied QML files to $PKG_DIR/qml"