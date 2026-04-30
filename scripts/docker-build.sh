#!/bin/bash
# Cross-compile remailable for reMarkable Paper Pro inside Docker
# Usage: ./scripts/docker-build.sh [release|debug]
set -euo pipefail

MODE="${1:-release}"
SDK_PATH="${HOME}/rm-sdk"

if [ ! -d "$SDK_PATH/sysroots" ]; then
    echo "Error: SDK not found at $SDK_PATH"
    echo "Download and extract the reMarkable SDK first:"
    echo "  See docker/Dockerfile for instructions"
    exit 1
fi

echo "=== Building remailable for reMarkable Paper Pro (aarch64) ==="
echo "Mode: $MODE"

# Build the Docker image if it doesn't exist
if ! docker image inspect remailable-cross >/dev/null 2>&1; then
    echo "Building Docker image..."
    docker build -t remailable-cross docker/
fi

# Run the cross-compilation inside the container
docker run --rm \
    -v "$(pwd)":/build \
    -v "$SDK_PATH":/opt/codex/rm-ferrari \
    remailable-cross \
    bash -c "
        source /opt/codex/rm-ferrari/environment-setup-cortexa53-crypto-remarkable-linux && \
        export QMAKE=\"/opt/codex/rm-ferrari/sysroots/x86_64-codexsdk-linux/usr/bin/qmake\" && \
        export PKG_CONFIG_PATH=\"/opt/codex/rm-ferrari/sysroots/cortexa53-crypto-remarkable-linux/usr/lib/pkgconfig:/opt/codex/rm-ferrari/sysroots/cortexa53-crypto-remarkable-linux/usr/share/pkgconfig\" && \
        export PKG_CONFIG_SYSROOT_DIR=\"/opt/codex/rm-ferrari/sysroots/cortexa53-crypto-remarkable-linux\" && \
        cd /build && \
        cargo build --target aarch64-unknown-linux-gnu ${MODE:+--$MODE}
    "

echo ""
echo "=== Build complete ==="
echo "Binary: target/aarch64-unknown-linux-gnu/${MODE}/remailable"