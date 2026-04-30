---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: executing
last_updated: "2026-04-30T07:48:22.341Z"
progress:
  total_phases: 3
  completed_phases: 2
  total_plans: 8
  completed_plans: 7
---

# STATE: remailable

**Last updated:** 2026-04-30

## Project Reference

**Core Value:** Reading and replying to email on a reMarkable Paper Pro tablet, offline-first, with native-quality e-ink UX
**Current Focus:** Phase 03 — read-view

## Current Position

Phase: 03 (read-view) — EXECUTING
Plan: 3 of 3
**Phase:** 3
**Plan:** 03-02 COMPLETED → Next: 03-03
**Status:** Executing Phase 03

## Performance Metrics

| Metric | Value |
|--------|-------|
| Phases completed | 2 / 4 |
| Requirements shipped | 6 / 26 |
| Plans executed | 6 |
| Sessions on project | 7 |
| Phase 01-bootstrap-ci P01 | 14 min | 2 tasks | 11 files |
| Phase 01-bootstrap-ci P02 | 7 min | 2 tasks | 6 files |
| Phase 02-connect-sync P01 | 18 min | 2 tasks | 6 files |
| Phase 02-connect-sync P02 | 16 min | 2 tasks | 4 files |
| Phase 02-connect-sync P03 | 12 min | 2 tasks | 9 files |
| Phase 03-read-view P01 | 17 min | 2 tasks | 5 files |
| Phase 03-read-view P02 | 13 min | 2 tasks | 6 files |
| Phase 03-read-view P02 | 13min | 2 tasks | 6 files |

## Accumulated Context

### Decisions

- Phase 1: Yocto SDK for cross-compilation (official reMarkable toolchain)
- Phase 1: Dynamic Qt linking against SDK sysroot libraries
- Phase 1: CI installs SDK via shell step with caching
- Phase 1: Custom Cargo target config (.cargo/config.toml) for cross-compilation
- Plan 01-01: Used CXX-Qt 0.8 (not 0.7) with adapted API patterns
- Plan 01-01: QML module URI is io.remailable.Remailable
- Plan 01-01: Qt6 for local dev, Yocto SDK for target
- Plan 01-02: AppLoad external app format with eglfs/KMS Qt platform
- Plan 01-02: build-rcc.sh simplified to copy QML (CXX-Qt uses qrc bundling)
- Plan 01-02: CI SDK environment sourced in single build step (no step persistence)
- Plan 02-01: rusqlite with bundled feature for cross-compilation simplicity
- Plan 02-01: UUID v4 for account IDs (no external service, no sequential ID leakage)
- Plan 02-01: email_metadata table created now to avoid future schema migration
- Plan 02-01: ConnectionResult enum categorizes network errors for UI
- Plan 02-02: EmailMetadata in account.rs (not separate file) for simpler CXX-Qt bridge import
- Plan 02-02: Email bodies as files on disk, not SQLite BLOBs — keeps DB small
- Plan 02-02: Incremental UID-range sync (1:* first, N+1:* subsequent)
- Plan 02-03: CXX-Qt 0.8 qproperty uses QString backing fields (not String)
- Plan 02-03: Lazy<Mutex<Storage>> global for bridge access (not Rc<RefCell<Storage>>)
- Plan 02-03: Index-based get_account_* invokables instead of QAbstractListModel for v1
- Plan 02-03: Synchronous validation on main thread (background threading needed later)
- Plan 02-03: Loader source switching for navigation (not StackView)
- Plan 03-01: regex crate for HTML stripping in html_to_eink (simple approach for e-ink fallback)
- Plan 03-01: BODYSTRUCTURE parsed via regex (not full MIME parser) — sufficient for common patterns
- Plan 03-01: HTML entities decoded before tag stripping with Unicode placeholders to preserve angle brackets
- Plan 03-01: PRAGMA foreign_keys = ON for proper FK constraint enforcement
- Plan 03-01: Schema migration via PRAGMA table_info + ALTER TABLE ADD COLUMN for backward compat
- Plan 03-02: TextArea with RichText format as universal HTML renderer for e-ink (QtWebView may not be available on Paper Pro)
- Plan 03-02: select_folder returns void, QML reads get_folder_name and sets appModel.selected_folder
- Plan 03-02: Two-phase borrow pattern for toggle_email_read (rust() then rust_mut())
- [Phase 03-read-view]: TextArea with RichText format as universal HTML renderer for e-ink (QtWebView may not be available on Paper Pro)
- [Phase 03-read-view]: select_folder returns void — QML reads get_folder_name and sets appModel.selected_folder (decoupled models)
- [Phase 03-read-view]: Two-phase borrow pattern for toggle_email_read: rust() for read then rust_mut() for write

### Active Todos

- [x] Plan 01-01: Project scaffold + ARM cross-compilation config
- [x] Plan 01-02: GitHub Actions CI pipeline
- [x] Plan 02-01: Account model, SQLite storage, IMAP/SMTP connection validation
- [x] Plan 02-02: Sync engine with incremental IMAP email sync to local storage
- [x] Plan 02-03: CXX-Qt bridge and QML UI for account management and sync status
- [x] Plan 03-01: Extend Rust backend for reading, threads, search, and attachments
- [x] Plan 03-02: CXX-Qt bridge QObjects and QML screens for folder nav, email list, reader

### Blockers

- (none)

## Session Continuity

**Last action:** Completed 03-02-PLAN.md (CXX-Qt bridge QObjects and QML screens for folder nav, email list, reader)
**Next step:** Plan 03-03 (search UI, attachment handling, inline PDF viewer)
**Carry-forward:** CXX-Qt 0.8 API, Qt 6 for local dev, Lazy<Mutex<Storage>> bridge pattern, EmailMetadata with content_type/in_reply_to/thread_id/has_attachments, AttachmentMetadata, index-based model getters (FolderListModel, EmailListModel, EmailReaderModel), RichText HTML fallback for e-ink, select_folder QML-side pattern, two-phase borrow for toggle_email_read, Loader source navigation pattern
