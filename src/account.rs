use serde::{Deserialize, Serialize};

/// Email metadata stored locally for offline access.
/// The email body is stored separately (body_path) for size efficiency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailMetadata {
    pub id: String,           // UUID v4
    pub account_id: String,  // Foreign key to AccountConfig.id
    pub folder: String,       // IMAP folder name (e.g., "INBOX", "Sent")
    pub uid: u32,            // IMAP UID (unique within folder+account)
    pub subject: String,
    pub from_addr: String,
    pub date: String,        // ISO 8601 or IMAP date format
    pub read: bool,
    pub body_path: String,   // Path to locally stored email body file
    pub content_type: String, // MIME content type (e.g. "text/plain", "text/html") — READ-03, READ-04
    pub in_reply_to: String,  // Message-ID of parent email for threading — READ-06
    pub thread_id: String,    // Groups emails in same conversation — READ-06
    pub has_attachments: bool, // Flag for attachment indicator in list view — ATCH-01
}

/// Metadata for an email attachment, persisted in the attachments table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentMetadata {
    pub id: String,           // UUID v4
    pub email_id: String,     // FK to email_metadata.id
    pub account_id: String,   // For storage directory organization
    pub filename: String,      // Attachment filename from Content-Disposition
    pub content_type: String,  // MIME type (e.g. "application/pdf")
    pub size: i64,             // Size in bytes
    pub part_number: String,   // IMAP part number for fetching (e.g. "2", "2.1")
    pub downloaded: bool,      // Whether file is saved to disk
    pub local_path: String,    // Path to downloaded file, empty if not downloaded
}

/// Full account configuration including IMAP and SMTP settings.
/// This is what the user provides when adding an account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountConfig {
    pub id: String,              // UUID v4
    pub display_name: String,    // User-facing name like "Work" or "Personal"
    pub email: String,           // Email address (e.g. "user@example.com")
    pub imap_host: String,
    pub imap_port: u16,          // Typically 993 for IMAPS
    pub username: String,
    pub password: String,        // Stored in SQLite — future phases may encrypt
    pub smtp_host: String,
    pub smtp_port: u16,          // Typically 587 for SMTPS or 465 for SMTP over TLS
}

impl AccountConfig {
    #[allow(dead_code)]
    pub fn new(
        display_name: String,
        email: String,
        imap_host: String,
        imap_port: u16,
        username: String,
        password: String,
        smtp_host: String,
        smtp_port: u16,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            display_name,
            email,
            imap_host,
            imap_port,
            username,
            password,
            smtp_host,
            smtp_port,
        }
    }
}