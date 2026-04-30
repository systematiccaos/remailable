# Cross-compilation environment for reMarkable Paper Pro
# The SDK is mounted at runtime via -v flag
FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive

# Install build dependencies + Rust in single layer
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    python3 \
    curl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
    && /root/.cargo/bin/rustup target add aarch64-unknown-linux-gnu

ENV PATH="/root/.cargo/bin:${PATH}"

# The reMarkable Yocto SDK hardcodes /usr/local/oe-sdk-hardcoded-buildpath as its
# installation prefix in ELF binaries (INTERP, RPATH). When we mount the SDK at a
# different path, those binaries can't find their dynamic linker. We create the
# expected directory as a symlink to wherever the SDK is actually mounted so the
# cross-compiler and qmake can execute.
RUN mkdir -p /usr/local/oe-sdk-hardcoded-buildpath && \
    ln -sf /opt/codex/rm-ferrari/sysroots /usr/local/oe-sdk-hardcoded-buildpath/sysroots

WORKDIR /build
CMD ["bash"]