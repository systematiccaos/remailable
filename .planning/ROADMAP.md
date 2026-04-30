# Roadmap: remailable

**Core Value:** Reading and replying to email on a reMarkable Paper Pro tablet, offline-first, with native-quality e-ink UX
**Granularity:** Coarse (3-5 phases)
**Created:** 2026-04-29

## Phases

- [x] **Phase 1: Bootstrap & CI** — Project scaffolding, cross-compilation, AppLoad packaging, and GitHub Actions pipeline
- [x] **Phase 2: Connect & Sync** — IMAP/SMTP account setup, multi-account management, local sync, and offline access
- [ ] **Phase 3: Read & View** — Folder navigation, email reading (plain + HTML), threads, search, and attachment handling
- [ ] **Phase 4: Compose & Reply** — Writing and sending emails using the Type Folio keyboard

## Phase Details

### Phase 1: Bootstrap & CI
**Goal**: The project builds and produces a deployable AppLoad package for reMarkable Paper Pro
**Depends on**: Nothing (first phase)
**Requirements**: DEPL-01, DEPL-02, DEPL-03, DEPL-04
**Success Criteria** (what must be TRUE):
  1. `cargo build` succeeds for the reMarkable Paper Pro ARM target with Qt/QML linked
  2. A push to main triggers GitHub Actions CI that cross-compiles the project for the reMarkable target
  3. The CI pipeline produces an AppLoad-compatible package as a build artifact
  4. The AppLoad package can be installed on a reMarkable Paper Pro and the app launches to a blank window
**Plans**: 2 plans
- [x] 01-01 — Scaffold Rust+Qt project with CXX-Qt and ARM cross-compilation config
- [x] 01-02 — AppLoad packaging scripts and GitHub Actions CI pipeline

### Phase 2: Connect & Sync
**Goal**: Users can configure email accounts and have emails synced locally for offline access
**Depends on**: Phase 1
**Requirements**: CONN-01, CONN-02, CONN-03, CONN-04, CONN-05, OFFL-01, OFFL-02, OFFL-03, OFFL-04
**Success Criteria** (what must be TRUE):
  1. User can add an IMAP/SMTP account with hostname, port, username, and password — and settings are validated before saving
  2. User can add, switch between, and remove multiple email accounts via a settings UI
  3. Emails from all configured accounts sync to local storage when network is available
  4. User can browse and read cached emails while offline (no network)
  5. App displays sync status (syncing / synced / offline) to the user
**Plans**: 3 plans
- [x] 02-01 — Account data model, IMAP/SMTP connection validation, and SQLite storage
- [x] 02-02 — Email sync engine with incremental sync and offline storage
- [x] 02-03 — CXX-Qt bridge, QML account management UI, and sync status indicator
**UI hint**: yes

### Phase 3: Read & View
**Goal**: Users can browse, read, and interact with emails and attachments optimized for e-ink display
**Depends on**: Phase 2
**Requirements**: READ-01, READ-02, READ-03, READ-04, READ-05, READ-06, READ-07, ATCH-01, ATCH-02, ATCH-03
**Success Criteria** (what must be TRUE):
  1. User can navigate email folders (inbox, sent, drafts, trash, custom) and see email lists with subject, sender, and date
  2. User can read plain text and HTML emails rendered in an e-ink-friendly format (high contrast, minimal refresh)
  3. User can see read/unread status synced with the server and view emails grouped into conversation threads
  4. User can search emails by subject and sender
  5. User can see attachment lists with filename and size, download attachments to device, and view PDFs inline
**Plans**: 3 plans
- [ ] 03-01 — Rust data layer: MIME parsing, HTML sanitization, search, threads, attachment metadata, IMAP flag sync
- [ ] 03-02 — CXX-Qt bridge + core reading UI: folder navigation, email list, plain text viewing, read/unread sync
- [ ] 03-03 — Advanced features: HTML rendering, thread grouping, search, attachments, PDF inline viewing
**UI hint**: yes

## Progress

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Bootstrap & CI | 2/2 | Complete | 2026-04-29 |
| 2. Connect & Sync | 3/3 | Complete | 2026-04-29 |
| 3. Read & View | 1/3 | In Progress | - |
| 4. Compose & Reply | 0/0 | Not started | - |