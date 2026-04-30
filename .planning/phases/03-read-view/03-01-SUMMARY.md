---
phase: 03-read-view
plan: 01
subsystem: storage, imap, sync
tags: [rusqlite, sqlite, imap, regex, html-parser, threading, search, attachments]

# Dependency graph
requires:
  - phase: 02-connect-sync
    provides: EmailMetadata struct, Storage layer, IMAP connection, SyncEngine
provides:
  - Extended EmailMetadata with content_type, in_reply_to, thread_id, has_attachments
  - AttachmentMetadata struct for attachment persistence
  - Storage methods: get_email, search_emails, list_thread, save_attachment, list_attachments, get_attachment_by_id, mark_downloaded
  - IMAP BODYSTRUCTURE parsing (ParsedStructure, AttachmentPart)
  - IMAP thread header fetching (fetch_message_headers_for_thread)
  - IMAP attachment part download (fetch_attachment_part)
  - HTML-to-e-ink plain text converter (html_to_eink)
  - Thread ID calculation (calculate_thread_id)
  - Extended sync engine storing content_type, threading, and attachment metadata
affects: [03-read-view, 04-compose-reply]

# Tech tracking
tech-stack:
  added: [regex-1]
  patterns: [PRAGMA-migration-for-schema-evolution, FK-enforcement-via-PRAGMA, HTML-to-plain-text-for-eink, BODYSTRUCTURE-regex-parsing, thread-id-from-in-reply-to]

key-files:
  created: []
  modified: [src/account.rs, src/storage.rs, src/imap_conn.rs, src/sync.rs, Cargo.toml]

key-decisions:
  - "regex crate for HTML stripping in html_to_eink — simple approach for e-ink fallback"
  - "BODYSTRUCTURE parsed via regex (not full MIME parser) — sufficient for common email patterns"
  - "HTML entities decoded before tag stripping to preserve angle brackets in content"
  - "PRAGMA foreign_keys = ON enabled for proper FK constraint enforcement"
  - "Schema migration via PRAGMA table_info + ALTER TABLE ADD COLUMN for backward compat"

patterns-established:
  - "Schema migration pattern: check column existence with PRAGMA table_info before ALTER TABLE"
  - "FK-safe deletion: query associated records first, delete in correct order (attachments → emails → accounts)"
  - "E-ink HTML conversion: strip <style>/<script> first, decode entities, replace block elements, strip tags, collapse whitespace"
  - "Thread ID calculation: use In-Reply-To header normalized (strip angle brackets), fall back to email's own ID"

requirements-completed: [READ-01, READ-03, READ-04, READ-05, READ-06, READ-07, ATCH-01, ATCH-02, ATCH-03]

# Metrics
duration: 17min
completed: 2026-04-30
---

# Phase 3 Plan 1: Rust Backend Extensions Summary

**Extended EmailMetadata with content_type/threading/attachments, Storage CRUD for search/threads/attachments, IMAP BODYSTRUCTURE parsing, HTML-to-eink conversion, and sync engine updates**

## Performance

- **Duration:** 17 min
- **Started:** 2026-04-30T07:07:49Z
- **Completed:** 2026-04-30T07:24:38Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- Extended EmailMetadata with content_type, in_reply_to, thread_id, has_attachments fields for e-ink rendering, thread grouping, and attachment indication
- Added AttachmentMetadata struct with full CRUD via Storage (save, list, get_by_id, mark_downloaded)
- Implemented search_emails with case-insensitive LIKE matching on subject and from_addr
- Implemented get_email by ID and list_thread by thread_id with date ordering
- Added IMAP BODYSTRUCTURE parsing to extract content type and attachment parts via regex
- Added fetch_message_headers_for_thread for MESSAGE-ID/IN-REPLY-TO threading headers
- Added fetch_attachment_part for downloading individual MIME parts to disk
- Implemented html_to_eink converter: strips CSS/scripts, decodes entities, replaces block elements, produces high-contrast plain text
- Updated sync engine to fetch and store content_type, in_reply_to, thread_id, has_attachments, and attachment metadata during sync
- Schema migration handles existing databases via PRAGMA table_info + ALTER TABLE ADD COLUMN
- All 48 unit tests pass (5 ignored live-IMAP integration tests)

## Task Commits

Each task was committed atomically:

1. **Task 1: Extend data model and storage for reading, threads, search, and attachments** - `1f4e393` (feat)
2. **Task 2: Extend IMAP for BODYSTRUCTURE, attachment download, HTML conversion; update sync** - `964d9d1` (feat)

## Files Created/Modified

- `src/account.rs` - Added AttachmentMetadata struct, extended EmailMetadata with content_type, in_reply_to, thread_id, has_attachments
- `src/storage.rs` - Added attachments table, migrations, get_email, search_emails, list_thread, save_attachment, list_attachments, get_attachment_by_id, mark_downloaded; FK enforcement; 12 new tests
- `src/imap_conn.rs` - Added ParsedStructure, AttachmentPart, fetch_bodystructure, fetch_message_headers_for_thread, fetch_attachment_part, html_to_eink, calculate_thread_id, parse_bodystructure; 13 new tests
- `src/sync.rs` - Extended sync_folder to fetch and store content_type, threading, attachment metadata; 3 new tests
- `Cargo.toml` - Added regex dependency

## Decisions Made

- **regex crate for HTML stripping** — Simple regex-based approach for html_to_eink is sufficient for the e-ink fallback renderer. QML WebView can handle full HTML rendering; Rust-side conversion is just the plain-text fallback.
- **BODYSTRUCTURE via regex, not full MIME parser** — IMAP BODYSTRUCTURE responses are complex S-expressions, but a regex-based extractor handles common patterns (multi-part with PDF/image attachments). Over-engineering a full parser is unnecessary for v1.
- **HTML entities decoded before tag stripping** — Decodes `&lt;` `&gt;` to unicode placeholders before stripping tags, then restores to actual characters. This prevents `<3>` being stripped as an HTML tag.
- **PRAGMA foreign_keys = ON** — SQLite doesn't enforce foreign keys by default. Enabling this ensures data integrity (e.g., can't delete an account with orphaned attachments).
- **Schema migration via PRAGMA table_info** — Instead of blindly running ALTER TABLE (which fails if columns exist), we check column existence first. This handles both fresh installs and upgrades from Phase 2 databases.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed HTML entity stripping destroying `<3>` as HTML tag**
- **Found during:** Task 2 (html_to_eink implementation)
- **Issue:** Decoded `&lt;3&gt;` to `<3>` before tag stripping, then `<3>` was stripped as an HTML tag by `<[^>]*>` regex
- **Fix:** Changed entity decoding to use Unicode placeholders (`‹` and `›`) during tag stripping, then replace back to `<` and `>` after. Also added `<style>` and `<script>` block removal before entity decoding.
- **Files modified:** src/imap_conn.rs
- **Verification:** test_html_to_eink_entities passes, test_html_to_eink_strips_css passes

**2. [Rule 1 - Bug] Fixed BODYSTRUCTURE regex matching combined vs separate type strings**
- **Found during:** Task 2 (parse_bodystructure implementation)
- **Issue:** Initial regex `"(text|application|...)/([^"]+)"` expected combined MIME types like `"text/plain"`, but IMAP BODYSTRUCTURE encodes them as separate quoted strings `"text" "plain"`. Test assertion on body_content_type failed.
- **Fix:** Changed regex to `"(text|application|...)"\s+"([^"]+)"` to match separate type/subtype strings
- **Files modified:** src/imap_conn.rs
- **Verification:** test_parse_bodystructure_multipart and test_parse_bodystructure_plain_only pass

**3. [Rule 1 - Bug] Fixed SQLite foreign key violation on delete_account**
- **Found during:** Task 1 (storage tests)
- **Issue:** Deleting an account violated FK constraint because attachments reference email_metadata, which references accounts. The original code deleted in wrong order.
- **Fix:** Added PRAGMA foreign_keys = ON, restructured delete_account to query email IDs first, then delete attachments → email_metadata → accounts. Also added account_id-based attachment cleanup.
- **Files modified:** src/storage.rs
- **Verification:** test_delete_account_cascades_attachments passes

**4. [Rule 3 - Blocking] Fixed CREATE INDEX referencing columns not yet added during migration**
- **Found during:** Task 1 (storage schema migration)
- **Issue:** CREATE INDEX idx_email_thread ON email_metadata(thread_id) ran before migration added the thread_id column, causing "no such column" error on existing databases.
- **Fix:** Split init_tables into: (1) CREATE TABLE statements, (2) migration, (3) indexes that depend on migrated columns. This ensures columns exist before indexes are created.
- **Files modified:** src/storage.rs
- **Verification:** test_migration_adds_columns passes

**5. [Rule 3 - Blocking] Fixed Rust lifetime issue with statement in delete_account**
- **Found during:** Task 1 (compilation fix)
- **Issue:** Rust borrow checker rejected `stmt.query_map(...).collect()` because the temporary MappedRows owned the statement reference and statement was dropped while still borrowed
- **Fix:** Assigned collect result to a variable via separate `let rows = ...; rows.collect()` pattern
- **Files modified:** src/storage.rs
- **Verification:** cargo test compiles and passes

---

**Total deviations:** 5 auto-fixed (3 bug, 2 blocking)
**Impact on plan:** All auto-fixes necessary for correctness and compilation. No scope creep.

## Issues Encountered

None — all issues found and fixed as deviations above.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Backend data model and storage ready for Plan 03-02 (CXX-Qt bridge QObjects for folder nav, email list, email reader, threads)
- IMAP BODYSTRUCTURE and attachment download ready for Plan 03-03 (search, attachment handling, inline PDF viewer)
- HTML-to-eink conversion ready for QML email reader fallback rendering
- Thread grouping logic ready for QML thread view

---
*Phase: 03-read-view*
*Completed: 2026-04-30*

## Self-Check: PASSED

- [x] src/account.rs exists
- [x] src/storage.rs exists
- [x] src/imap_conn.rs exists
- [x] src/sync.rs exists
- [x] Cargo.toml exists
- [x] Commit 1f4e393 found
- [x] Commit 964d9d1 found