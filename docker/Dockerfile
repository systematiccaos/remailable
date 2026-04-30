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

# SDK will be mounted at /opt/codex/rm-ferrari at runtime
RUN echo 'if [ -f /opt/codex/rm-ferrari/environment-setup-cortexa53-crypto-remarkable-linux ]; then' >> /root/.bashrc \
    && echo '  source /opt/codex/rm-ferrari/environment-setup-cortexa53-crypto-remarkable-linux' >> /root/.bashrc \
    && echo 'fi' >> /root/.bashrc

WORKDIR /build
CMD ["bash"]