# Phase 1: Bootstrap & CI - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-29
**Phase:** 1-bootstrap-ci
**Areas discussed:** Cross-compilation toolchain

---

## Cross-compilation toolchain

### Toolchain source

| Option | Description | Selected |
|--------|-------------|----------|
| Yocto SDK | Use reMarkable's official Yocto SDK for cross-compiler and sysroot. Most reliable library version matching. | ✓ |
| Generic ARM + musl | Generic aarch64-unknown-linux-gnu target with musl static linking. Lighter but Qt linkage issues likely. | |
| Docker + Yocto SDK | Docker container with SDK pre-installed. Reproducible but heavier CI setup. | |

**User's choice:** Yocto SDK
**Notes:** Recommended for reliability of exact library version match with device

### Qt/QML linking

| Option | Description | Selected |
|--------|-------------|----------|
| SDK sysroot Qt | Link dynamically against Qt libraries in Yocto SDK sysroot. Standard approach for reMarkable. | ✓ |
| Static Qt linkage | Statically link Qt into binary. Larger binary, more complex build. | |
| Hybrid approach | cxx/qml-rust generates C++ bridging code compiled with the rest. | |

**User's choice:** SDK sysroot Qt
**Notes:** Recommended — avoids static linking complexity, matches device's installed Qt

### CI SDK availability

| Option | Description | Selected |
|--------|-------------|----------|
| CI installs SDK | Install Yocto SDK in CI via shell step, with caching. Straightforward. | ✓ |
| Custom Docker image | Pre-built Docker image with SDK. Faster after first setup, but maintenance overhead. | |
| Manual toolchain | Ubuntu runner + manual assembly. Fragile, not recommended. | |

**User's choice:** CI installs SDK
**Notes:** Recommended for simplicity — avoids maintaining a custom Docker image

### Build system wiring

| Option | Description | Selected |
|--------|-------------|----------|
| Custom Cargo target | .cargo/config.toml pointing to SDK linker and sysroot. Standard Rust approach. | ✓ |
| CMake + Corrosion | CMake finds cross-compiler and Qt, builds Rust via Corrosion. More complex. | |
| Makefile wrapper | Makefile rules setting up cross-compilation environment. Less maintainable. | |

**User's choice:** Custom Cargo target
**Notes:** Recommended — standard, clean, portable across local dev and CI

## the agent's Discretion

- Exact Rust crate structure (binary crate, workspace layout, QML file location)
- GitHub Actions workflow details and caching strategy
- AppLoad package assembly process (format to be researched)
- Rust-Qt binding choice (cxx, qml-rust, or other — to be researched)
- Local development workflow for testing without a device

## Deferred Ideas

None — discussion stayed within phase scope