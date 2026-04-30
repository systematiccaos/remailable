---
phase: 03-read-view
plan: 03
subsystem: ui, bridge
tags: [cxx-qt, qml, e-ink, search, threading, attachments, pdf, html-rendering]

# Dependency graph
requires:
  - phase: 02-connect-sync
    provides: CXX-Qt 0.8 pattern, AppModel, AccountListModel, Lazy<Mutex<Storage>> bridge
  - phase: 03-read-view/01
    provides: Extended EmailMetadata, Storage methods (search_emails, list_threads, list_attachments, mark_downloaded)
  - phase: 03-read-view/02
    provides: FolderListModel, EmailListModel, EmailReaderModel, FolderList.qml, EmailList.qml, EmailReader.qml, main.qml navigation
provides:
  - EmailListModel search_emails/clear_search/refresh_threaded invokables with is_searching/thread_mode qproperties
  - AttachmentListModel with load_attachments/download_attachment and metadata getters
  - EmailReaderModel HTML/plain toggle (show_html/show_plain_text) and attachment_email_id qproperty
  - SearchBar.qml component for subject/sender filtering
  - AttachmentList.qml component with download and PDF viewing
  - Updated EmailList.qml with search bar, thread toggle, improved e-ink styling
  - Updated EmailReader.qml with HTML/plain toggle and attachment section
  - Updated main.qml with AttachmentListModel and pdf_view navigation
  - Updated build.rs with new QML file registrations
affects: [03-read-view, 04-compose-reply]

# Tech tracking
tech-stack:
  added: []
  patterns: [search-via-like-query, thread-mode-toggle, attachment-download-to-data-dir, html-plain-text-toggle, pdf-fallback-with-file-path]

key-files:
  created: [qml/SearchBar.qml, qml/AttachmentList.qml]
  modified: [src/cxxqt.rs, qml/EmailList.qml, qml/EmailReader.qml, qml/main.qml, build.rs]

key-decisions:
  - "EmailListModel caches account_id internally for search/clear operations (avoids re-passing account_id from QML)"
  - "refresh_threaded loads flat list with thread_id — QML visualizes threading via indent prefix and thread indicator"
  - "HTML/plain text toggle uses EmailReaderModel.show_html()/show_plain_text() invokables toggling email_content_type qproperty"
  - "PDF inline viewing uses fallback message with file path — Qt.labs.pdf may not be on Paper Pro, system viewer is recommended"
  - "AttachmentListModel download_attachment copies from attachments/ to downloads/ dir and updates DB"
  - "SearchBar is a standalone QML component embedded in EmailList for reusability"

patterns-established:
  - "Two-phase borrow pattern for search_emails/clear_search: extract data via rust() in block, then rust_mut() separately"
  - "E-ink search UX: search bar at top, search status indicator bar with Clear button when is_searching"
  - "Thread mode toggle button in header: Threads/List switching refresh_emails vs refresh_threaded"
  - "Thread visual: indented replies with ▶ prefix when thread_mode is true"
  - "HTML toggle: button in email reader header switches between RichText and PlainText textFormat"

requirements-completed: [READ-04, READ-06, READ-07, ATCH-01, ATCH-02, ATCH-03]

# Metrics
duration: 27min
completed: 2026-04-30
---

# Phase 3 Plan 3: Search, Threads, HTML Rendering, and Attachments Summary

**Search by subject/sender, thread grouping toggle, HTML/plain text toggle, and full attachment handling (list, download, PDF fallback) via CXX-Qt bridge and e-ink QML screens**

## Performance

- **Duration:** 27 min
- **Started:** 2026-04-30T07:50:33Z
- **Completed:** 2026-04-30T08:17:54Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- New CXX-Qt bridge models: search_emails/clear_search/refresh_threaded on EmailListModel, AttachmentListModel with full CRUD + download, show_html/show_plain_text on EmailReaderModel
- EmailListModel tracks is_searching and thread_mode state, caches account_id for search/refresh without re-passing from QML
- AttachmentListModel download_attachment copies files from attachments/ dir to downloads/ dir and updates DB download status
- EmailReaderModel tracks attachment_email_id for QML to wire AttachmentList
- SearchBar.qml: standalone search input component with clear button, search trigger, and e-ink-friendly large touch targets
- AttachmentList.qml: attachment list with filename, formatted size (B/KB/MB), content type, download button, PDF view button, and download status indicator
- EmailList.qml: integrated SearchBar, thread/List toggle button, search status indicator, thread indent and indicator, enhanced unread background tint (#f0f5f0)
- EmailReader.qml: HTML/plain text toggle button in header, AttachmentList embedded in body, PDF view fallback page with file path display
- All screens use e-ink-optimized styling: 44px+ touch targets, high contrast black on white, large fonts

## Task Commits

Each task was committed atomically:

1. **Task 1: Add search, thread grouping, attachment, and HTML bridge models to CXX-Qt** - `269d66b` (feat)
2. **Task 2: Create QML screens for search, threads, HTML email, and attachments** - `96e6c46` (feat)

## Files Created/Modified

- `src/cxxqt.rs` - Added search_emails/clear_search/refresh_threaded to EmailListModel, AttachmentListModel with download, show_html/show_plain_text and attachment_email_id to EmailReaderModel, updated backing structs
- `qml/SearchBar.qml` - New standalone search input component with Search/Clear functionality
- `qml/AttachmentList.qml` - New attachment list with download, file size formatting, PDF view button, and download status
- `qml/EmailList.qml` - Added SearchBar integration, thread/List toggle, search status indicator, thread indent, improved e-ink styling
- `qml/EmailReader.qml` - Added HTML/plain toggle, AttachmentList integration, PDF view fallback, improved layout
- `qml/main.qml` - Added AttachmentListModel instance, pdf_view navigation case
- `build.rs` - Added SearchBar.qml and AttachmentList.qml registrations

## Decisions Made

- **EmailListModel caches account_id internally** — QML passes account_id on refresh_emails, which stores it for subsequent search/clear operations, avoiding re-passing from QML state
- **refresh_threaded loads flat list with thread_id** — Rather than restructuring the list into nested thread groups, the backend provides the same flat list and QML uses thread_id to visually indicate threading (indent, prefix). This keeps the model simple and consistent
- **HTML/plain text toggle via show_html()/show_plain_text() invokables** — These toggle the email_content_type qproperty between "text/html" and "text/plain", which the QML TextArea watches via textFormat binding. Simple and e-ink-friendly
- **PDF fallback with file path display** — Since Qt.labs.pdf may not be available on the Paper Pro, the PDF viewer shows a fallback message with the download path. Users can open PDFs in the system viewer. This is documented as a future enhancement point
- **Attachment download copies to downloads/ dir** — download_attachment copies from remailable/attachments/{account_id}/{email_id}/{filename} to remailable/downloads/{email_id}/{filename}, then marks the DB record as downloaded with the path

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed drop(rust) warning for reference type in CXX-Qt bridge**
- **Found during:** Task 1 (cargo check warning)
- **Issue:** `drop(rust)` on `&EmailListModelRust` reference does nothing — Rust warned about dropping a reference in search_emails and clear_search
- **Fix:** Replaced with block-scoped borrows using tuple destructuring pattern: `let (account_id, current_folder) = { let rust = ...; (rust.account_id.clone(), ...) };`
- **Files modified:** src/cxxqt.rs
- **Verification:** cargo check passes with zero warnings for remailable crate
- **Committed in:** 96e6c46 (part of Task 1 commit, applied during overall Task 1 work)

**2. [Rule 3 - Blocking] Kept EmailReader.qml filename instead of creating EmailView.qml**
- **Found during:** Task 2 (QML file naming)
- **Issue:** Plan referenced `EmailView.qml` but the existing file is `EmailReader.qml` and navigation references `email_reader` view. Creating a new file and removing the old one would break without updating main.qml correctly
- **Fix:** Updated `EmailReader.qml` in-place instead of creating `EmailView.qml`. All the planned functionality (HTML toggle, attachments) was added to the existing file
- **Files modified:** qml/EmailReader.qml (updated instead of creating new EmailView.qml)
- **Verification:** cargo build and cargo test pass
- **Committed in:** 96e6c46

---

**Total deviations:** 2 (1 bug auto-fix, 1 blocking issue resolution)
**Impact on plan:** Both deviations were minimal. The drop fix eliminates a compiler warning, and keeping EmailReader.qml avoids unnecessary file renames.

## Issues Encountered

None — all issues found and fixed as deviations above.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Complete email reading features: search by subject/sender, thread grouping, HTML rendering toggle, attachment handling
- All Phase 3 requirements (READ-04, READ-06, READ-07, ATCH-01, ATCH-02, ATCH-03) fulfilled
- Ready for Phase 4: Compose and Reply (new email composition, reply, SMTP send)
- Future consideration: if Qt.labs.pdf is confirmed on Paper Pro, inline PDF rendering can replace the fallback message

---
*Phase: 03-read-view*
*Completed: 2026-04-30*

## Self-Check: PASSED

- [x] src/cxxqt.rs exists
- [x] qml/SearchBar.qml exists
- [x] qml/AttachmentList.qml exists
- [x] qml/EmailList.qml exists
- [x] qml/EmailReader.qml exists
- [x] qml/main.qml exists
- [x] build.rs exists
- [x] Commit 269d66b found
- [x] Commit 96e6c46 found