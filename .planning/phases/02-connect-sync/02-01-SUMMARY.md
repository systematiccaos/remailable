---
phase: 02-connect-sync
plan: 01
subsystem: storage, imap, smtp
tags: [rusqlite, sqlite, imap, lettre, native-tls, serde, uuid, account-model]

# Dependency graph
requires:
  - phase: 01-bootstrap-ci
    provides: CXX-Qt project scaffold, build.rs, Cargo.toml with CXX-Qt deps
provides:
  - AccountConfig data model with UUID v4 id, IMAP/SMTP fields
  - SQLite storage layer with accounts table and email_metadata table
  - IMAP connection validation returning ConnectionResult enum
  - SMTP connection validation via lettre test_connection()
  - Live IMAP session connection (connect_imap) for sync engine
  - SMTP transport builder (build_smtp_transport) for sending
affects: [02-connect-sync, 03-ui, 04-compose-send]

# Tech tracking
tech-stack:
  added: [rusqlite-0.32-bundled, serde-1, serde_json-1, uuid-1-v4, dirs-5, imap-2, native-tls-0.2, lettre-0.11]
  patterns: [ConnectionResult-enum-for-categorized-errors, SQLite-offline-storage, UUID-v4-for-account-ids]

key-files:
  created: [src/account.rs, src/storage.rs, src/imap_conn.rs, src/smtp_conn.rs]
  modified: [Cargo.toml, src/main.rs]

key-decisions:
  - "rusqlite with bundled feature for cross-compilation simplicity (no system SQLite dependency)"
  - "UUID v4 for account IDs (no external service, no sequential ID leakage)"
  - "email_metadata table created now to avoid future schema migration"
  - "imap crate login error returns (Error, Client) tuple — not (Client, Error)"
  - "imap::connect takes ToSocketAddrs directly — no try_into needed"
  - "lettre test_connection returns Result<bool, Error> — not bool"
  - "ConnectionResult enum categorizes errors for UI display"

patterns-established:
  - "ConnectionResult enum: pattern for categorizing network errors into ConnectionFailed, AuthFailed, TlsFailed, Timeout"
  - "Storage::open with :memory: for tests: pattern for testable SQLite layers"
  - "AccountConfig with new() constructor auto-generating UUIDs"

requirements-completed: [CONN-01, CONN-02, CONN-05, OFFL-01]

# Metrics
duration: 18min
completed: 2026-04-29
---

# Phase 2 Plan 1: Account Model & Connection Validation Summary

**Account data model with SQLite persistence, IMAP/SMTP connection validation with categorized error types**

## Performance

- **Duration:** 18 min
- **Started:** 2026-04-29T20:39:21Z
- **Completed:** 2026-04-29T20:57:40Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments
- AccountConfig struct with UUID v4 auto-generation, IMAP and SMTP configuration fields
- SQLite storage layer with full CRUD (save, list, get, delete) for accounts
- email_metadata table schema ready for Plan 02 sync engine
- IMAP connection validation with categorized error results (ConnectionFailed, AuthFailed, TlsFailed)
- SMTP connection validation via lettre's test_connection() method
- Live IMAP session provider (connect_imap) for the sync engine
- SMTP transport builder for Phase 4 email sending
- Database initialization at app startup using platform data directory

## Task Commits

Each task was committed atomically:

1. **Task 1: Define account data model and SQLite storage layer** - `39d180c` (feat)
2. **Task 2: Implement IMAP and SMTP connection validation** - `9bcc9c9` (feat)

## Files Created/Modified
- `src/account.rs` - AccountConfig struct with all IMAP/SMTP fields and UUID id generation
- `src/storage.rs` - SQLite-backed Storage with CRUD operations and email_metadata table
- `src/imap_conn.rs` - IMAP validation (validate_imap, connect_imap) with ConnectionResult enum
- `src/smtp_conn.rs` - SMTP validation (validate_smtp, build_smtp_transport)
- `Cargo.toml` - Added rusqlite, serde, uuid, dirs, imap, native-tls, lettre dependencies
- `src/main.rs` - Added module declarations and database initialization at startup

## Decisions Made
- rusqlite with `bundled` feature so SQLite compiles inline — no system dependency for cross-compilation
- UUID v4 for account IDs — no external service needed, no sequential leakage
- email_metadata table created in this plan to avoid schema migration later (OFFL-01 foundation)
- ConnectionResult enum used by both IMAP and SMTP validation for consistent error categorization

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed imap crate API mismatches**
- **Found during:** Task 2 (IMAP/SMTP connection validation)
- **Issue:** Plan specified `imap::types::Error` (private) and `(&addr).try_into()` pattern. The actual `imap` crate API differs: Error is at `imap::error::Error` but is not needed since `imap::connect` takes `ToSocketAddrs` directly; login error tuple is `(Error, Client)` not `(Client, Error)`; `session.logout()` requires mut binding; `lettre::test_connection()` returns `Result<bool, Error>` not `bool`
- **Fix:** Used `&addr` directly for `imap::connect`, swapped tuple destructuring to `(e, _client)`, added `mut` to session, pattern matched `Ok(true)/Ok(false)/Err(e)` for SMTP test_connection
- **Files modified:** src/imap_conn.rs, src/smtp_conn.rs
- **Verification:** All 8 unit tests pass, cargo check succeeds

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Minimal — API adaptation required for crate versions. No scope creep.

## Issues Encountered
- Yocto SDK cross-compilation config in `.cargo/config.toml` sets QMAKE and PKG_CONFIG_SYSROOT_DIR env vars that break native builds. Needed to override with `QMAKE=/opt/homebrew/bin/qmake PKG_CONFIG_PATH="" PKG_CONFIG_SYSROOT_DIR=""` for local development/testing.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Account model and storage ready for Plan 02 (sync engine to populate email_metadata)
- IMAP connection module ready for Plan 02 (connect_imap returns live sessions)
- SMTP transport builder ready for Phase 04 (compose and send)
- ConnectionResult enum ready to be exposed to QML via CXX-Qt bridge in Plan 03

---
*Phase: 02-connect-sync*
*Completed: 2026-04-29*

## Self-Check: PASSED

- [x] src/account.rs exists
- [x] src/storage.rs exists
- [x] src/imap_conn.rs exists
- [x] src/smtp_conn.rs exists
- [x] Commit 39d180c found
- [x] Commit 9bcc9c9 found
- [x] Commit 8951113 found
- [x] SUMMARY.md exists