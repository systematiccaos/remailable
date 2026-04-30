#!/bin/bash
# Cross-compile remailable for reMarkable Paper Pro inside Docker
set -euo pipefail

MODE="${1:-release}"
SDK_PATH="${HOME}/rm-sdk"

if [ ! -d "$SDK_PATH/sysroots" ]; then
    echo "Error: SDK not found at $SDK_PATH"
    echo "Download and extract the reMarkable SDK first."
    exit 1
fi

echo "=== Building remailable for reMarkable Paper Pro (aarch64) ==="
echo "Mode: $MODE"

# Build the Docker image if it doesn't exist
if ! docker image inspect remailable-cross >/dev/null 2>&1; then
    echo "Building Docker image..."
    docker build -t remailable-cross docker/
fi

# Run the cross-compilation inside the container.
#
# Key setup:
# 1. /usr/local/oe-sdk-hardcoded-buildpath -> /opt/codex/rm-ferrari (symlink in Docker image)
#    This lets the SDK's x86-64 binaries find their dynamic linker.
# 2. Fix the env-setup script paths (sed) so variables point to the right sysroot.
# 3. Source the environment, then use a linker wrapper that adds --sysroot flags
#    that cargo doesn't normally pass.
docker run --rm \
    -v "$(pwd)":/build \
    -v "$SDK_PATH":/opt/codex/rm-ferrari \
    remailable-cross \
    bash -c '
        # Fix the env script: replace hardcoded installation prefix with actual mount path
        # The SDK installer was supposed to do this but we manually extracted on macOS
        sed -i "s|/usr/local/oe-sdk-hardcoded-buildpath|/opt/codex/rm-ferrari|g" \
            /opt/codex/rm-ferrari/environment-setup-cortexa53-crypto-remarkable-linux 2>/dev/null || true
        sed -i "s|/usr/local/oe-sdk-hardcoded-buildpath|/opt/codex/rm-ferrari|g" \
            /opt/codex/rm-ferrari/site-config-cortexa53-crypto-remarkable-linux 2>/dev/null || true

        # Source the SDK environment (sets CC, CXX, PKG_CONFIG_*, PATH, etc.)
        unset LD_LIBRARY_PATH
        source /opt/codex/rm-ferrari/environment-setup-cortexa53-crypto-remarkable-linux

        SYSROOT=/opt/codex/rm-ferrari/sysroots/cortexa53-crypto-remarkable-linux

        # cxx-qt-build needs QMAKE and PKG_CONFIG
        export QMAKE=/opt/codex/rm-ferrari/sysroots/x86_64-codexsdk-linux/usr/bin/qmake
        export PKG_CONFIG_PATH=$SYSROOT/usr/lib/pkgconfig:$SYSROOT/usr/share/pkgconfig
        export PKG_CONFIG_SYSROOT_DIR=$SYSROOT

        # Use our linker wrapper that adds --sysroot and -L flags
        export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=/build/scripts/linker-wrapper.sh

        echo "=== Cross-compilation environment ==="
        echo "CC=$CC"
        echo "CXX=$CXX"
        echo "QMAKE=$QMAKE"
        echo "SYSROOT=$SYSROOT"
        echo "LINKER=$CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER"

        which aarch64-remarkable-linux-g++ || { echo "ERROR: cross-compiler not on PATH"; exit 1; }

        echo ""
        echo "=== Building remailable-backend (AppLoad headless) ==="
        cd /build
        REMAILABLE_NO_QT=1 cargo build --bin remailable-backend --target aarch64-unknown-linux-gnu --release
    '

echo ""
echo "=== Build complete ==="
echo "Backend binary: target/aarch64-unknown-linux-gnu/${MODE}/remailable-backend"