<!-- GSD:project-start source:PROJECT.md -->
## Project

**remailable**

An offline-first email client for the reMarkable Paper Pro tablet, deployed via AppLoad. Built in Rust + Qt, it supports multiple IMAP/SMTP accounts with a focus on reading emails and replying, inline PDF/image attachment rendering, and CI builds via GitHub Actions. Designed for the reMarkable's e-ink display and Type Folio keyboard accessory.

**Core Value:** Reading and replying to email on a reMarkable Paper Pro tablet, offline-first, with native-quality e-ink UX.

### Constraints

- **Tech Stack:** Rust + Qt/QML — user's explicit choice for safety + native UI
- **Platform:** reMarkable Paper Pro (specific model, not generic reMarkable 2)
- **Deployment:** Must work with AppLoad packaging format
- **CI:** GitHub Actions — build on push to main, manual release
- **E-ink UX:** Must optimize for slow refresh rates, touch input, high-contrast monochrome rendering
- **Offline:** Local-first architecture — emails must be readable without network
- **No on-screen keyboard:** Composition via Type Folio / system keyboard only
<!-- GSD:project-end -->

<!-- GSD:stack-start source:STACK.md -->
## Technology Stack

Technology stack not yet documented. Will populate after codebase mapping or first phase.
<!-- GSD:stack-end -->

<!-- GSD:conventions-start source:CONVENTIONS.md -->
## Conventions

Conventions not yet established. Will populate as patterns emerge during development.
<!-- GSD:conventions-end -->

<!-- GSD:architecture-start source:ARCHITECTURE.md -->
## Architecture

Architecture not yet mapped. Follow existing patterns found in the codebase.
<!-- GSD:architecture-end -->

<!-- GSD:workflow-start source:GSD defaults -->
## GSD Workflow Enforcement

Before using Edit, Write, or other file-changing tools, start work through a GSD command so planning artifacts and execution context stay in sync.

Use these entry points:
- `/gsd:quick` for small fixes, doc updates, and ad-hoc tasks
- `/gsd:debug` for investigation and bug fixing
- `/gsd:execute-phase` for planned phase work

Do not make direct repo edits outside a GSD workflow unless the user explicitly asks to bypass it.
<!-- GSD:workflow-end -->



<!-- GSD:profile-start -->
## Developer Profile

> Profile not yet configured. Run `/gsd:profile-user` to generate your developer profile.
> This section is managed by `generate-claude-profile` -- do not edit manually.
<!-- GSD:profile-end -->
