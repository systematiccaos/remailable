# Requirements: remailable

**Defined:** 2026-04-29
**Core Value:** Reading and replying to email on a reMarkable Paper Pro tablet, offline-first, with native-quality e-ink UX

## v1 Requirements

Requirements for initial release. Each maps to roadmap phases.

### Email Connectivity

- [x] **CONN-01**: User can connect to an IMAP server using hostname, port, username, and password
- [x] **CONN-02**: User can send emails via SMTP with the same credentials
- [x] **CONN-03**: User can configure and switch between multiple email accounts
- [x] **CONN-04**: User can add and remove email accounts through a settings UI
- [x] **CONN-05**: App validates IMAP/SMTP connection settings before saving

### Email Reading

- [x] **READ-01**: User can list and navigate email folders (inbox, sent, drafts, trash, custom)
- [ ] **READ-02**: User can view an email list with subject, sender, and date
- [x] **READ-03**: User can read plain text email bodies
- [x] **READ-04**: User can read HTML email bodies rendered in an e-ink-friendly way
- [x] **READ-05**: User can see read/unread status on emails and it syncs with the server
- [x] **READ-06**: User can view emails grouped into conversation threads
- [x] **READ-07**: User can search emails by subject and sender

### Email Composition

- [ ] **COMP-01**: User can compose a new email using the Type Folio / system keyboard
- [ ] **COMP-02**: User can reply to an existing email
- [ ] **COMP-03**: User can specify recipient (To), subject, and body for outgoing emails

### Attachments

- [x] **ATCH-01**: User can see a list of attachments on an email with filename and size
- [x] **ATCH-02**: User can download attachments to the device
- [x] **ATCH-03**: User can view PDF attachments inline using the reMarkable's display

### Offline & Sync

- [x] **OFFL-01**: Emails are cached locally for offline reading when previously synced
- [x] **OFFL-02**: App syncs emails from the server when network becomes available
- [x] **OFFL-03**: App indicates sync status (syncing, synced, offline) to the user
- [x] **OFFL-04**: User can browse and read cached emails while offline

### Deployment

- [x] **DEPL-01**: App is packaged in AppLoad-compatible format for reMarkable Paper Pro
- [x] **DEPL-02**: GitHub Actions workflow builds the project on push to main
- [x] **DEPL-03**: Cross-compilation for reMarkable's ARM target is configured in CI
- [x] **DEPL-04**: Build artifacts (AppLoad package) are available as CI output

## v2 Requirements

### Email Composition (Extended)

- **COMP-04**: User can forward emails to other recipients
- **COMP-05**: User can add CC and BCC fields to outgoing emails
- **COMP-06**: User can save emails as drafts before sending

### Attachments (Extended)

- **ATCH-04**: User can view image attachments inline in the email view
- **ATCH-05**: User can attach files to outgoing emails

### Authentication

- **AUTH-01**: User can authenticate via OAuth2 with Gmail
- **AUTH-02**: User can authenticate via OAuth2 with Outlook

### Connectivity (Extended)

- **CONN-06**: App receives push email notifications via IMAP IDLE

### Offline (Extended)

- **OFFL-05**: User can flag/star emails offline and sync flags when online
- **OFFL-06**: App performs background sync while open and connected

### Deployment (Extended)

- **DEPL-05**: Auto-release on tag push
- **DEPL-06**: CI cross-compilation caching for faster builds

## Out of Scope

| Feature | Reason |
|---------|--------|
| PGP/encrypted email | Significant complexity, not needed for v1 |
| Calendar/contacts integration | Separate concern outside email client scope |
| Push notifications | reMarkable doesn't support background app notifications well |
| On-screen virtual keyboard | Composition limited to Type Folio / system keyboard |
| Rich text/HTML composing | Plain text sufficient for e-ink context |
| Full-text search index | Basic search suffices for v1; full-text index is heavy for embedded device |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| DEPL-01 | Phase 1 | Complete |
| DEPL-02 | Phase 1 | Complete |
| DEPL-03 | Phase 1 | Complete |
| DEPL-04 | Phase 1 | Complete |
| CONN-01 | Phase 2 | Complete |
| CONN-02 | Phase 2 | Complete |
| CONN-03 | Phase 2 | Complete |
| CONN-04 | Phase 2 | Complete |
| CONN-05 | Phase 2 | Complete |
| OFFL-01 | Phase 2 | Complete |
| OFFL-02 | Phase 2 | Complete |
| OFFL-03 | Phase 2 | Complete |
| OFFL-04 | Phase 2 | Complete |
| READ-01 | Phase 3 | Complete |
| READ-02 | Phase 3 | Pending |
| READ-03 | Phase 3 | Complete |
| READ-04 | Phase 3 | Complete |
| READ-05 | Phase 3 | Complete |
| READ-06 | Phase 3 | Complete |
| READ-07 | Phase 3 | Complete |
| ATCH-01 | Phase 3 | Complete |
| ATCH-02 | Phase 3 | Complete |
| ATCH-03 | Phase 3 | Complete |
| COMP-01 | Phase 4 | Pending |
| COMP-02 | Phase 4 | Pending |
| COMP-03 | Phase 4 | Pending |

**Coverage:**
- v1 requirements: 26 total
- Mapped to phases: 26
- Unmapped: 0 ✓

---
*Requirements defined: 2026-04-29*
*Last updated: 2026-04-29 after roadmap creation*