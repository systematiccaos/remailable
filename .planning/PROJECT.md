# remailable

## What This Is

An offline-first email client for the reMarkable Paper Pro tablet, deployed via AppLoad. Built in Rust + Qt, it supports multiple IMAP/SMTP accounts with a focus on reading emails and replying, inline PDF/image attachment rendering, and CI builds via GitHub Actions. Designed for the reMarkable's e-ink display and Type Folio keyboard accessory.

## Core Value

Reading and replying to email on a reMarkable Paper Pro tablet, offline-first, with native-quality e-ink UX.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] Connect to multiple IMAP accounts and sync emails locally for offline reading
- [ ] Browse and read emails with a full-featured folder-based layout
- [ ] Compose new emails and reply using the reMarkable Type Folio / system keyboard
- [ ] Send emails via SMTP for each configured account
- [ ] Render PDF and image attachments inline, leveraging reMarkable's display strengths
- [ ] Deploy as a standalone app via AppLoad on reMarkable Paper Pro
- [ ] Build and package via GitHub Actions (build on push to main)

### Out of Scope

- PGP/encrypted email support — not needed for v1, adds significant complexity
- Calendar/contacts integration — separate concern, not part of an email client's core
- Push notifications — reMarkable doesn't support background app notifications well
- Rich text / HTML email composing — plain text replies are sufficient for e-ink context
- Advanced search (full-text index) — start with basic folder/subject search, add later if needed

## Context

- **Target device:** reMarkable Paper Pro — an e-ink tablet with a 11.8" color display, touch input, and optional Type Folio keyboard case
- **Deployment:** AppLoad is the mechanism for loading custom apps onto reMarkable tablets; the user is new to AppLoad but aware of its purpose
- **Tech ecosystem:** reMarkable's official apps are built with Qt/C++ on top of a custom Linux OS (based on Yocto). The user chose Rust + Qt as the implementation stack — Rust for safety and performance, Qt/QML for the UI layer matching the native reMarkable look and feel
- **Use case:** The user primarily reads emails on the reMarkable with occasional replies, not heavy composition
- **Connectivity:** reMarkable tablets have WiFi but are often used offline; local-first caching is critical
- **Input:** Touch for navigation, Type Folio keyboard for composition; no on-screen keyboard built into the app

## Constraints

- **Tech Stack:** Rust + Qt/QML — user's explicit choice for safety + native UI
- **Platform:** reMarkable Paper Pro (specific model, not generic reMarkable 2)
- **Deployment:** Must work with AppLoad packaging format
- **CI:** GitHub Actions — build on push to main, manual release
- **E-ink UX:** Must optimize for slow refresh rates, touch input, high-contrast monochrome rendering
- **Offline:** Local-first architecture — emails must be readable without network
- **No on-screen keyboard:** Composition via Type Folio / system keyboard only

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Rust + Qt/QML | Rust for safety/performance on embedded Linux; Qt for native reMarkable UI patterns | — Pending |
| Local-first storage | reMarkable is frequently offline; emails must be accessible without network | — Pending |
| Type Folio for composition | No on-screen keyboard; reMarkable's Type Folio is the natural input method | — Pending |
| Multiple accounts from v1 | User needs more than one email account | — Pending |
| AppLoad deployment | Standard mechanism for sideloading custom apps on reMarkable | — Pending |
| GitHub Actions build-on-push | Simple CI; manual release keeps control over what ships | — Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-04-29 after initialization*