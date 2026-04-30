---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: completed
last_updated: "2026-04-30T08:25:31.403Z"
progress:
  total_phases: 3
  completed_phases: 3
  total_plans: 8
  completed_plans: 8
---

# STATE: remailable

**Last updated:** 2026-04-30

## Project Reference

**Core Value:** Reading and replying to email on a reMarkable Paper Pro tablet, offline-first, with native-quality e-ink UX
**Current Focus:** Phase 03 — read-view (COMPLETE)

## Current Position

Phase: 03 (read-view) — COMPLETED
Plan: 3 of 3 (ALL COMPLETE)
**Phase:** 03
**Plan:** Not started
**Status:** Milestone complete

## Performance Metrics

| Metric | Value |
|--------|-------|
| Phases completed | 3 / 4 |
| Requirements shipped | 12 / 26 |
| Plans executed | 9 |
| Sessions on project | 8 |
| Phase 01-bootstrap-ci P01 | 14 min | 2 tasks | 11 files |
| Phase 01-bootstrap-ci P02 | 7 min | 2 tasks | 6 files |
| Phase 02-connect-sync P01 | 18 min | 2 tasks | 6 files |
| Phase 02-connect-sync P02 | 16 min | 2 tasks | 4 files |
| Phase 02-connect-sync P03 | 12 min | 2 tasks | 9 files |
| Phase 03-read-view P01 | 17 min | 2 tasks | 5 files |
| Phase 03-read-view P02 | 13 min | 2 tasks | 6 files |
| Phase 03-read-view P03 | 27 min | 2 tasks | 8 files |
| Phase 03-read-view P03 | 27min | 2 tasks | 8 files |

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
- Plan 03-03: EmailListModel caches account_id internally for search/clear operations (avoids re-passing from QML)
- Plan 03-03: refresh_threaded loads flat list with thread_id — QML visualizes threading via indent prefix
- Plan 03-03: HTML/plain text toggle via show_html()/show_plain_text() invokables toggling email_content_type
- Plan 03-03: PDF fallback with file path display (Qt.labs.pdf may not be on Paper Pro)
- Plan 03-03: Attachment download copies from attachments/ to downloads/ dir with DB status update
- Plan 03-03: SearchBar is a standalone QML component embedded in EmailList for reusability
- [Phase 03-read-view]: EmailListModel caches account_id internally for search/clear operations (avoids re-passing from QML)
- [Phase 03-read-view]: PDF fallback with file path display (Qt.labs.pdf may not be on Paper Pro, system viewer recommended)

### Active Todos

- [x] Plan 01-01: Project scaffold + ARM cross-compilation config
- [x] Plan 01-02: GitHub Actions CI pipeline
- [x] Plan 02-01: Account model, SQLite storage, IMAP/SMTP connection validation
- [x] Plan 02-02: Sync engine with incremental IMAP email sync to local storage
- [x] Plan 02-03: CXX-Qt bridge and QML UI for account management and sync status
- [x] Plan 03-01: Extend Rust backend for reading, threads, search, and attachments
- [x] Plan 03-02: CXX-Qt bridge QObjects and QML screens for folder nav, email list, reader
- [x] Plan 03-03: Search, thread grouping, HTML toggle, attachment handling, PDF view

### Blockers

- (none)

## Session Continuity

**Last action:** Completed 03-03-PLAN.md (search, thread grouping, HTML toggle, attachment handling, PDF view)
**Next step:** Phase 04 — compose-reply (email composition and SMTP sending)
**Carry-forward:** CXX-Qt 0.8 API, Qt 6 for local dev, Lazy<Mutex<Storage>> bridge pattern, EmailListModel with search/thread modes, AttachmentListModel with download, EmailReaderModel with HTML toggle, SearchBar/AttachmentList QML components, e-ink design patterns (28px headers, 20px body, 44px touch, high contrast, monochrome)
