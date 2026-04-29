---
phase: 02-connect-sync
plan: 02
subsystem: sync, storage, imap
tags: [imap, sync-engine, incremental-sync, email-metadata, sqlite, offline-first]

# Dependency graph
requires:
  - phase: 02-connect-sync
    provides: AccountConfig data model, Storage SQLite layer, connect_imap function
provides:
  - SyncEngine with sync_account and sync_all_accounts for multi-account incremental email sync
  - EmailMetadata struct bridging IMAP data and SQLite storage
  - Storage email metadata CRUD (save, list, get_max_uid, mark_read, list_folders)
  - IMAP fetch_folders, fetch_message_headers, fetch_message_body functions
  - Offline email browsing via SQLite queries
affects: [03-ui, 04-compose-send]

# Tech tracking
tech-stack:
  added: []
  patterns: [incremental-UID-sync, body-files-on-disk-not-in-SQLite, SyncStatus-state-enum]

key-files:
  created: [src/sync.rs]
  modified: [src/account.rs, src/imap_conn.rs, src/storage.rs, src/main.rs]

key-decisions:
  - "EmailMetadata struct placed in account.rs (not separate file) — keeps import graph simple for CXX-Qt bridge later"
  - "Email bodies stored as individual files in data directory, not SQLite BLOBs — keeps DB small and queryable"
  - "incremental sync via UID ranges: 1:* for first sync, N+1:* for subsequent — avoids re-downloading all emails"
  - "Foreign key constraint on email_metadata.account_id enforced in tests — accounts must exist before emails"
  - "imap crate envelope() returns Option<&Envelope> — handled with early continue on None"

patterns-established:
  - "UID-range incremental sync: first sync fetches 1:*, subsequent fetches max_uid+1:*"
  - "Email body storage: files on disk at dirs::data_dir()/remailable/bodies/{account_id}/{email_id}.txt"
  - "SyncStatus enum: state machine for sync progress (Idle/Syncing/Synced/Offline/Error)"
  - "FK-gated test pattern: create parent records before child records in SQLite tests"

requirements-completed: [OFFL-01, OFFL-02, OFFL-04]

# Metrics
duration: 16min
completed: 2026-04-29
---

# Phase 2 Plan 2: Email Sync Engine Summary

**IMAP sync engine with incremental UID-based email fetching, SQLite email metadata persistence, and offline query methods**

## Performance

- **Duration:** 16 min
- **Started:** 2026-04-29T21:04:58Z
- **Completed:** 2026-04-29T21:21:11Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- EmailMetadata struct bridging IMAP envelope data to SQLite storage with all required fields (id, account_id, folder, uid, subject, from_addr, date, read, body_path)
- IMAP folder listing (fetch_folders) and message header/body fetching (fetch_message_headers, fetch_message_body) 
- SyncEngine with incremental sync using UID ranges — only new emails fetched on subsequent syncs
- Storage extended with email metadata CRUD (save_email_metadata, list_emails_by_folder, get_max_uid, mark_email_read, list_folders)
- Email bodies stored as individual files on disk for size efficiency, not as SQLite BLOBs
- SyncStatus enum for tracking per-account sync state

## Task Commits

Each task was committed atomically:

1. **Task 1: Extend IMAP connection with folder listing and message fetching** - `3576304` (feat)
2. **Task 2: Implement sync engine and extend storage for email metadata** - `4c82c16` (feat)

## Files Created/Modified
- `src/account.rs` - Added EmailMetadata struct with all email header fields + body_path
- `src/imap_conn.rs` - Added fetch_folders, fetch_message_headers, fetch_message_body functions; fixed imap crate API differences
- `src/storage.rs` - Extended with save_email_metadata, list_emails_by_folder, get_max_uid, mark_email_read, list_folders; added 5 new tests
- `src/sync.rs` - New file: SyncEngine with sync_account, sync_all_accounts, sync_folder; incremental UID sync; tests
- `src/main.rs` - Added `pub mod sync;` module declaration

## Decisions Made
- EmailMetadata in account.rs (not a separate file) — keeps import graph simple for CXX-Qt bridge in Plan 03
- Email bodies stored as individual files at `dirs::data_dir()/remailable/bodies/{account_id}/{email_id}.txt` — keeps SQLite DB small and queryable
- Incremental sync via UID ranges (1:* for first, N+1:* for subsequent) — avoids re-downloading all emails on each sync
- FK constraint on email_metadata.account_id — enforced even in tests, requires parent account to exist first

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed imap crate API mismatches**
- **Found during:** Task 1 (IMAP extension implementation)
- **Issue:** Plan specified IMAP API patterns that don't match the actual imap crate v2.4.1. Five compilation errors: (1) `name()` returns `&str` not an iterator, (2) `envelope()` returns `Option<&Envelope>` not `&Envelope` directly, (3) `from` is `Option<Vec<Address>>` not `&Vec<Address>`, (4) Flag comparison needed deref `*f == Flag::Seen`, (5) `msg.uid` is a field not a method
- **Fix:** Updated all API calls: used `.name().to_string()` directly, handled envelope with `match msg.envelope()` and `continue` on None, used `.as_ref().and_then()` chain for from field, dereferenced Flag in comparison, accessed uid as field
- **Files modified:** src/imap_conn.rs
- **Verification:** All 20 unit tests pass, cargo check succeeds
- **Committed in:** 3576304 (Task 1 commit)

**2. [Rule 1 - Bug] Fixed foreign key constraint in email metadata tests**
- **Found during:** Task 2 (Storage email metadata methods)
- **Issue:** Tests inserting email_metadata with account_id="acct-1" failed with FK constraint violation — no matching account exists in the accounts table
- **Fix:** Updated all storage email tests to create and save an AccountConfig first, then use its generated id as the account_id for email metadata
- **Files modified:** src/storage.rs
- **Verification:** All 20 unit tests pass
- **Committed in:** 4c82c16 (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (2 bugs)
**Impact on plan:** Both auto-fixes required for correctness — API adaptation for real crate behavior and proper FK enforcement. No scope creep.

## Issues Encountered
- imap crate API differs significantly from plan assumptions — envelope() is Option, name() returns &str, from field requires .as_ref(), and Flag comparison needs deref. All handled inline.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Sync engine ready for Plan 03 (UI) — SyncEngine can be driven from QML via CXX-Qt bridge
- Email metadata queryable offline via Storage methods (list_emails_by_folder, get_max_uid)
- SyncStatus enum ready for UI state display
- IMAP fetch functions ready for live server integration testing

---
*Phase: 02-connect-sync*
*Completed: 2026-04-29*

## Self-Check: PASSED

- [x] src/account.rs exists (EmailMetadata struct)
- [x] src/imap_conn.rs exists (fetch_folders, fetch_message_headers, fetch_message_body)
- [x] src/storage.rs exists (email metadata CRUD methods)
- [x] src/sync.rs exists (SyncEngine, SyncStatus)
- [x] src/main.rs exists (pub mod sync)
- [x] Commit 3576304 found
- [x] Commit 4c82c16 found
- [x] SUMMARY.md exists