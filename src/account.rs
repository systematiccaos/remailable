use serde::{Deserialize, Serialize};

/// Full account configuration including IMAP and SMTP settings.
/// This is what the user provides when adding an account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountConfig {
    pub id: String,              // UUID v4
    pub display_name: String,    // User-facing name like "Work" or "Personal"
    pub imap_host: String,
    pub imap_port: u16,          // Typically 993 for IMAPS
    pub username: String,
    pub password: String,        // Stored in SQLite — future phases may encrypt
    pub smtp_host: String,
    pub smtp_port: u16,          // Typically 587 for SMTPS or 465 for SMTP over TLS
}

impl AccountConfig {
    pub fn new(
        display_name: String,
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
            imap_host,
            imap_port,
            username,
            password,
            smtp_host,
            smtp_port,
        }
    }
}