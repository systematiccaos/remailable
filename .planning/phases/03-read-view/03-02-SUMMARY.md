---
phase: 03-read-view
plan: 02
subsystem: ui, bridge
tags: [cxx-qt, qml, e-ink, folder-navigation, email-list, email-reader, threading]

# Dependency graph
requires:
  - phase: 02-connect-sync
    provides: CXX-Qt 0.8 pattern, AppModel, AccountListModel, Lazy<Mutex<Storage>> bridge
  - phase: 03-read-view/01
    provides: Extended EmailMetadata, Storage methods (list_folders, list_emails_by_folder, get_email, mark_email_read, list_thread), AttachmentMetadata
provides:
  - FolderListModel QObject for folder navigation
  - EmailListModel QObject for email list browsing with read/unread/attachment indicators
  - EmailReaderModel QObject for email body reading and thread view
  - Extended AppModel with selected_folder, selected_email_id, active_account_name
  - FolderList.qml screen for folder navigation
  - EmailList.qml screen for email list with subject/sender/date/read-status
  - EmailReader.qml screen for email reading (plain text + HTML via RichText) and thread view
  - Updated main.qml navigation: account_list → folder_list → email_list → email_reader
affects: [03-read-view, 04-compose-reply]

# Tech tracking
tech-stack:
  added: []
  patterns: [index-based-get-model-invokables, Loader-source-navigation, RichText-HTML-fallback-for-eink]

key-files:
  created: [qml/FolderList.qml, qml/EmailList.qml, qml/EmailReader.qml]
  modified: [src/cxxqt.rs, qml/main.qml, qml/AccountList.qml, build.rs]

key-decisions:
  - "TextArea with RichText format as universal HTML renderer for e-ink (QtWebView may not be available on Paper Pro)"
  - "select_folder returns void — QML reads get_folder_name(index) and sets appModel.selected_folder"
  - "toggle_email_read uses two-phase borrow: read from rust() then write via rust_mut()"

patterns-established:
  - "CXX-Qt multi-model bridge: each QML screen gets its own QObject model with index-based getters"
  - "E-ink QML pattern: white bg (#ffffff), black text, no animations, 44px+ touch targets, pressed state #f0f0f0"
  - "Email body dual-mode: TextEdit.RichText for HTML, TextEdit.PlainText for plain text via content_type check"
  - "Thread view: load_thread() called alongside load_email(), thread emails shown in bottom section when count > 1"

requirements-completed: [READ-01, READ-02, READ-03, READ-04, READ-05, READ-06]

# Metrics
duration: 13min
completed: 2026-04-30
---

# Phase 3 Plan 2: CXX-Qt Bridge and QML Screens Summary

**CXX-Qt bridge QObjects (FolderListModel, EmailListModel, EmailReaderModel) and three QML screens (FolderList, EmailList, EmailReader) wired into main.qml navigation for folder-based email browsing and reading**

## Performance

- **Duration:** 13 min
- **Started:** 2026-04-30T07:32:30Z
- **Completed:** 2026-04-30T07:45:30Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- Three new CXX-Qt QObjects: FolderListModel (folder navigation), EmailListModel (email browsing with read/unread/attachment/thread access), EmailReaderModel (email body loading, disk reading, auto-mark-read, thread view)
- Extended AppModel with selected_folder, selected_email_id, active_account_name properties for navigation state
- Full navigation flow: AccountList → FolderList → EmailList → EmailReader with back buttons at each level
- EmailReader dual-mode body rendering: RichText for HTML emails, PlainText for plain text emails, using TextArea (avoids QtWebView dependency on Paper Pro)
- Thread view at bottom of EmailReader showing related emails with tap-to-navigate
- Read/unread indicator toggle in email list (tap circle icon), auto-mark-read on email open
- All screens follow e-ink design: 1620x2160, 14-28px fonts, 44px+ touch targets, high contrast, no animations

## Task Commits

Each task was committed atomically:

1. **Task 1: Add FolderListModel, EmailListModel, and EmailReaderModel CXX-Qt QObjects** - `9d314b3` (feat)
2. **Task 2: Create QML screens for folder navigation, email list, and email reading** - `20b3780` (feat)

## Files Created/Modified

- `src/cxxqt.rs` - Added FolderListModel, EmailListModel, EmailReaderModel QObjects with all invokables; extended AppModel with 3 new qproperties
- `qml/FolderList.qml` - Folder navigation screen with back button, account name, folder list tap-to-select
- `qml/EmailList.qml` - Email list screen with subject/sender/date, read/unread circle toggle, attachment indicator, unread count
- `qml/EmailReader.qml` - Email reader with header, dual-mode body (RichText/PlainText), thread section with tap-to-navigate
- `qml/main.qml` - Added 3 new model instances, extended Loader navigation switch with folder_list/email_list/email_reader
- `qml/AccountList.qml` - Added Folders button per account, sets active_account_id and active_account_name on tap
- `build.rs` - Added 3 new QML file registrations

## Decisions Made

- **TextArea with RichText as universal HTML renderer** — QtWebView may not be available on the reMarkable Paper Pro's Qt6 build. Using `textFormat: TextEdit.RichText` as a fallback gives basic HTML rendering that works well on e-ink. If WebView is later confirmed available, we can add a conditional Loader.
- **select_folder returns void, QML reads get_folder_name** — The CXX-Qt bridge can't directly access AppModel from FolderListModel. Instead, the QML layer calls `get_folder_name(index)` and sets `appModel.selected_folder`. This keeps models decoupled.
- **Two-phase borrow pattern for toggle_email_read** — CXX-Qt's `rust()` returns an immutable reference and `rust_mut()` returns a mutable reference. To toggle read state, we first read the email ID and current state via `rust()`, drop the borrow, then update via `rust_mut()`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed CXX-Qt compile error in toggle_email_read with nested borrows**
- **Found during:** Task 1 (cargo check failed)
- **Issue:** Original implementation used `if let Ok(())` with nested mutable borrow `rust_mut()` inside a closure, causing CXX-Qt codegen error on line 626 (`expected ','`)
- **Fix:** Split into two-phase borrow: read email_id and new_read via `rust()` block, then write via `rust_mut()` after the block ends. Also replaced `if let Ok(())` with `.is_ok()` check.
- **Files modified:** src/cxxqt.rs
- **Verification:** cargo check passes
- **Committed in:** 9d314b3 (Task 1 commit)

**2. [Rule 1 - Bug] Fixed drop(rust) warning for reference type in select_folder**
- **Found during:** Task 1 (cargo check warning)
- **Issue:** `drop(rust)` on a `&FolderListModelRust` reference does nothing — Rust warned about dropping a reference
- **Fix:** Simplified select_folder to just validate index and use `let _folder_name` pattern, removing the unnecessary drop
- **Files modified:** src/cxxqt.rs
- **Verification:** cargo check passes with zero warnings for remailable crate
- **Committed in:** 9d314b3 (Task 1 commit)

---

**Total deviations:** 2 auto-fixed (2 bug)
**Impact on plan:** Both auto-fixes were compile correctness issues. No scope creep.

## Issues Encountered

None — all issues found and fixed as deviations above.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Full navigation flow working: accounts → folders → email list → email reader with back navigation
- Email list displays subject, sender, date, read/unread indicator, attachment indicator
- Email reader renders plain text and HTML (via RichText), shows thread emails
- Read/unread status is togglable and auto-marked on email open
- Ready for Plan 03-03: search, attachment handling, inline PDF viewer
- Future consideration: if QtWebView is confirmed on Paper Pro, EmailReader can be enhanced with WebView for full HTML rendering

---
*Phase: 03-read-view*
*Completed: 2026-04-30*

## Self-Check: PASSED

- [x] src/cxxqt.rs exists
- [x] qml/FolderList.qml exists
- [x] qml/EmailList.qml exists
- [x] qml/EmailReader.qml exists
- [x] qml/main.qml exists
- [x] qml/AccountList.qml exists
- [x] build.rs exists
- [x] Commit 9d314b3 found
- [x] Commit 20b3780 found