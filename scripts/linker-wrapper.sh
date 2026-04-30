#!/bin/bash
# Linker wrapper for reMarkable Paper Pro cross-compilation
# Adds --sysroot and library search paths that the SDK's CC/CXX normally provide,
# but which cargo doesn't pass when it uses the linker directly.
#
# Usage: Set CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER to point to this script.

SYSROOT="/opt/codex/rm-ferrari/sysroots/cortexa53-crypto-remarkable-linux"

exec aarch64-remarkable-linux-g++ \
    -mcpu=cortex-a53 \
    -march=armv8-a+crc+crypto \
    --sysroot="$SYSROOT" \
    -L"$SYSROOT/usr/lib" \
    -L"$SYSROOT/usr/lib/aarch64-remarkable-linux/11.4.0" \
    "$@"