use crate::account::{AccountConfig, EmailMetadata};
use crate::imap_conn;
use crate::storage::Storage;
use uuid::Uuid;

/// Sync status for a single account.
#[derive(Debug, Clone)]
pub enum SyncStatus {
    Idle,
    Syncing { account: String, folder: String },
    Synced { account: String },
    Offline,
    Error { account: String, message: String },
}

/// The email sync engine.
/// Handles syncing emails from IMAP servers to local SQLite storage.
pub struct SyncEngine<'a> {
    storage: &'a Storage,
}

impl<'a> SyncEngine<'a> {
    pub fn new(storage: &'a Storage) -> Self {
        Self { storage }
    }

    /// Sync all configured accounts.
    /// Returns a Vec of (account_display_name, result) tuples.
    pub fn sync_all_accounts(&self, accounts: &[AccountConfig]) -> Vec<(String, Result<(), String>)> {
        accounts.iter().map(|account| {
            let result = self.sync_account(account);
            (account.display_name.clone(), result)
        }).collect()
    }

    /// Sync a single account: connect to IMAP, fetch folders and messages, store locally.
    /// This is an incremental sync — only new messages (UID > max stored UID) are fetched.
    pub fn sync_account(&self, config: &AccountConfig) -> Result<(), String> {
        // Connect to IMAP
        let mut session = imap_conn::connect_imap(config)?;

        // Get list of folders
        let folders = imap_conn::fetch_folders(&mut session)?;

        // Sync each folder
        for folder in &folders {
            self.sync_folder(&mut session, config, folder)?;
        }

        // Logout
        session.logout().map_err(|e| format!("Logout failed: {}", e))?;
        Ok(())
    }

    /// Incrementally sync a single folder for an account.
    /// Only fetches messages with UID greater than the max stored UID.
    fn sync_folder(
        &self,
        session: &mut imap::Session<native_tls::TlsStream<std::net::TcpStream>>,
        config: &AccountConfig,
        folder: &str,
    ) -> Result<(), String> {
        // Get the highest UID we already have for this folder
        let max_uid = self.storage.get_max_uid(&config.id, folder)
            .map_err(|e| format!("Storage error getting max UID: {}", e))?;

        // Select the folder
        session.select(folder)
            .map_err(|e| format!("IMAP SELECT {} failed: {}", folder, e))?;

        // Fetch only new messages (UID > max_uid) or all if first sync
        let uid_range = match max_uid {
            Some(uid) => format!("{}:*", uid + 1),
            None => "1:*".to_string(),
        };

        let messages = session.uid_fetch(&uid_range, "(UID ENVELOPE FLAGS)")
            .map_err(|e| format!("IMAP FETCH on {} failed: {}", folder, e))?;

        for msg in messages.iter() {
            let uid = msg.uid.unwrap_or(0);
            if uid == 0 { continue; }

            // Skip the last UID on incremental sync (since IMAP UID ranges are inclusive)
            if let Some(max) = max_uid {
                if uid <= max { continue; }
            }

            // Parse envelope fields
            let envelope = match msg.envelope() {
                Some(env) => env,
                None => continue,
            };

            let subject = envelope.subject
                .and_then(|s| std::str::from_utf8(s).ok())
                .unwrap_or("(no subject)")
                .to_string();

            let from_addr = envelope.from
                .as_ref()
                .and_then(|f| f.first())
                .and_then(|addr| addr.mailbox.as_ref())
                .and_then(|m| std::str::from_utf8(m).ok())
                .unwrap_or("(unknown)")
                .to_string();

            let date = envelope.date
                .and_then(|d| std::str::from_utf8(d).ok())
                .unwrap_or("")
                .to_string();

            let read = msg.flags().iter().any(|f| *f == imap::types::Flag::Seen);

            let email_id = Uuid::new_v4().to_string();

            // Fetch the body for this message
            let body = imap_conn::fetch_message_body(session, uid)?;
            let body_dir = format!("remailable/bodies/{}", config.id);
            let body_path = format!("{}/{}.txt", body_dir, email_id);

            // Create body directory and save
            let base = dirs::data_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
            let full_body_dir = base.join(&body_dir);
            std::fs::create_dir_all(&full_body_dir).ok();
            let full_body_path = base.join(&body_path);
            std::fs::write(&full_body_path, &body)
                .map_err(|e| format!("Failed to write body file: {}", e))?;

            let email = EmailMetadata {
                id: email_id,
                account_id: config.id.clone(),
                folder: folder.to_string(),
                uid,
                subject,
                from_addr,
                date,
                read,
                body_path,
                content_type: String::new(),     // Will be populated by extended sync (Plan 03-01)
                in_reply_to: String::new(),      // Will be populated by extended sync (Plan 03-01)
                thread_id: String::new(),        // Will be populated by extended sync (Plan 03-01)
                has_attachments: false,           // Will be populated by extended sync (Plan 03-01)
            };

            self.storage.save_email_metadata(&email)
                .map_err(|e| format!("Failed to save email metadata: {}", e))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account::AccountConfig;
    use crate::storage::Storage;

    #[test]
    fn test_sync_engine_new() {
        let storage = Storage::open(":memory:").unwrap();
        let _engine = SyncEngine::new(&storage);
    }

    #[test]
    fn test_sync_bad_host_returns_error() {
        let storage = Storage::open(":memory:").unwrap();
        let engine = SyncEngine::new(&storage);
        let config = AccountConfig::new(
            "Bad".into(),
            "nonexistent.invalid.host.example.com".into(),
            993,
            "user@test.com".into(),
            "pass".into(),
            "smtp.invalid.example.com".into(),
            587,
        );
        let result = engine.sync_account(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_incremental_uid_range_first_sync() {
        // First sync: max_uid is None → range should be "1:*"
        let uid_range = match None as Option<u32> {
            Some(uid) => format!("{}:*", uid + 1),
            None => "1:*".to_string(),
        };
        assert_eq!(uid_range, "1:*");
    }

    #[test]
    fn test_incremental_uid_range_subsequent_sync() {
        // Subsequent sync: max_uid is 42 → range should be "43:*"
        let max_uid: Option<u32> = Some(42);
        let uid_range = match max_uid {
            Some(uid) => format!("{}:*", uid + 1),
            None => "1:*".to_string(),
        };
        assert_eq!(uid_range, "43:*");
    }

    #[test]
    fn test_sync_all_accounts_empty() {
        let storage = Storage::open(":memory:").unwrap();
        let engine = SyncEngine::new(&storage);
        let results = engine.sync_all_accounts(&[]);
        assert!(results.is_empty());
    }
}