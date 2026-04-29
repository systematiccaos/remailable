# Phase 1: Bootstrap & CI - Context

**Gathered:** 2026-04-29
**Status:** Ready for planning

<domain>
## Phase Boundary

The project builds and produces a deployable AppLoad package for reMarkable Paper Pro. This phase delivers: Rust+Qt project scaffolding, ARM cross-compilation toolchain configuration, AppLoad-compatible packaging, and a GitHub Actions CI pipeline. The end result is a blank-window app that launches on a reMarkable Paper Pro.

</domain>

<decisions>
## Implementation Decisions

### Cross-compilation toolchain
- **D-01:** Use the reMarkable official Yocto SDK for both the cross-compiler and sysroot — ensures exact library version match with the device and is the standard approach for reMarkable apps
- **D-02:** Qt/QML is linked dynamically against the Qt libraries provided in the Yocto SDK sysroot — avoids complexity of static Qt linking and matches the device's installed libraries
- **D-03:** GitHub Actions CI installs the Yocto SDK via a shell step (with download caching) — straightforward setup, avoids maintaining a custom Docker image, ~3-5 min overhead per run that can be cached
- **D-04:** Cross-compilation is wired into Rust via a custom Cargo target configuration (`.cargo/config.toml`) pointing to the SDK's linker and sysroot — standard Rust cross-compilation approach, clean and portable across local dev and CI

### the agent's Discretion
- Exact structure of the Rust crate (binary crate, workspace layout, where QML files live)
- Specific GitHub Actions workflow steps and caching strategy
- How to assemble the AppLoad package (research needed on AppLoad format)
- Whether to use cxx, qml-rust, or another Rust-Qt binding (research needed)
- Local development workflow for testing without a device

</decisions>

<specifics>
## Specific Ideas

- No specific product references — this is infrastructure/tooling setup, not a visual feature
- The user is new to AppLoad but aware of its purpose — documentation and discovery of the packaging format is expected

</specifics>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### AppLoad packaging
- AppLoad documentation and packaging format — must be researched; the user is new to AppLoad and no local docs exist yet

### reMarkable Paper Pro SDK
- reMarkable Yocto SDK setup and cross-compilation guide — must be researched for correct target triple, sysroot paths, and Qt version

### Rust cross-compilation
- `.cargo/config.toml` cross-compilation documentation — standard Rust approach for custom targets with custom linkers

No local spec/ADR files exist yet — this is the first phase of a greenfield project. Research should focus on AppLoad format and reMarkable SDK documentation.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- None — greenfield project, no existing code

### Established Patterns
- None — patterns will be established in this phase

### Integration Points
- This phase creates the foundation that all subsequent phases build on
- The Cargo target config, build scripts, and CI workflow establish patterns for the entire project

</code_context>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 01-bootstrap-ci*
*Context gathered: 2026-04-29*