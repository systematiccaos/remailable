use rusqlite::{params, Connection};
use crate::account::{AccountConfig, EmailMetadata};

pub struct Storage {
    conn: Connection,
}

impl Storage {
    /// Open (or create) the SQLite database at the given path.
    /// Pass ":memory:" for tests.
    pub fn open(path: &str) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(path)?;
        let storage = Self { conn };
        storage.init_tables()?;
        Ok(storage)
    }

    fn init_tables(&self) -> Result<(), rusqlite::Error> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS accounts (
                id TEXT PRIMARY KEY,
                display_name TEXT NOT NULL,
                imap_host TEXT NOT NULL,
                imap_port INTEGER NOT NULL,
                username TEXT NOT NULL,
                password TEXT NOT NULL,
                smtp_host TEXT NOT NULL,
                smtp_port INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS email_metadata (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                folder TEXT NOT NULL,
                uid INTEGER,
                subject TEXT,
                from_addr TEXT,
                date TEXT,
                read INTEGER DEFAULT 0,
                body_path TEXT,
                FOREIGN KEY (account_id) REFERENCES accounts(id)
            );

            CREATE INDEX IF NOT EXISTS idx_email_account_folder
                ON email_metadata(account_id, folder);

            CREATE INDEX IF NOT EXISTS idx_email_uid
                ON email_metadata(account_id, uid);"
        )?;
        Ok(())
    }

    pub fn save_account(&self, account: &AccountConfig) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "INSERT OR REPLACE INTO accounts (id, display_name, imap_host, imap_port, username, password, smtp_host, smtp_port)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![account.id, account.display_name, account.imap_host, account.imap_port,
                    account.username, account.password, account.smtp_host, account.smtp_port],
        )?;
        Ok(())
    }

    pub fn list_accounts(&self) -> Result<Vec<AccountConfig>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, display_name, imap_host, imap_port, username, password, smtp_host, smtp_port FROM accounts"
        )?;
        let accounts = stmt.query_map([], |row| {
            Ok(AccountConfig {
                id: row.get(0)?,
                display_name: row.get(1)?,
                imap_host: row.get(2)?,
                imap_port: row.get(3)?,
                username: row.get(4)?,
                password: row.get(5)?,
                smtp_host: row.get(6)?,
                smtp_port: row.get(7)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        Ok(accounts)
    }

    pub fn get_account(&self, id: &str) -> Result<Option<AccountConfig>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, display_name, imap_host, imap_port, username, password, smtp_host, smtp_port FROM accounts WHERE id = ?1"
        )?;
        let mut accounts = stmt.query_map(params![id], |row| {
            Ok(AccountConfig {
                id: row.get(0)?,
                display_name: row.get(1)?,
                imap_host: row.get(2)?,
                imap_port: row.get(3)?,
                username: row.get(4)?,
                password: row.get(5)?,
                smtp_host: row.get(6)?,
                smtp_port: row.get(7)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        Ok(accounts.pop())
    }

    pub fn delete_account(&self, id: &str) -> Result<(), rusqlite::Error> {
        self.conn.execute("DELETE FROM accounts WHERE id = ?1", params![id])?;
        // Also delete associated email metadata
        self.conn.execute("DELETE FROM email_metadata WHERE account_id = ?1", params![id])?;
        Ok(())
    }

    /// Save (insert or replace) email metadata to the local store.
    pub fn save_email_metadata(&self, email: &EmailMetadata) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "INSERT OR REPLACE INTO email_metadata
             (id, account_id, folder, uid, subject, from_addr, date, read, body_path)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![email.id, email.account_id, email.folder, email.uid,
                    email.subject, email.from_addr, email.date, email.read as i32, email.body_path],
        )?;
        Ok(())
    }

    /// List all emails for a given account+folder, ordered by date descending.
    pub fn list_emails_by_folder(&self, account_id: &str, folder: &str) -> Result<Vec<EmailMetadata>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, account_id, folder, uid, subject, from_addr, date, read, body_path
             FROM email_metadata WHERE account_id = ?1 AND folder = ?2
             ORDER BY date DESC"
        )?;
        let emails = stmt.query_map(params![account_id, folder], |row| {
            Ok(EmailMetadata {
                id: row.get(0)?,
                account_id: row.get(1)?,
                folder: row.get(2)?,
                uid: row.get(3)?,
                subject: row.get(4)?,
                from_addr: row.get(5)?,
                date: row.get(6)?,
                read: row.get::<_, i32>(7)? != 0,
                body_path: row.get(8)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        Ok(emails)
    }

    /// Get the maximum IMAP UID stored for a given account+folder.
    /// Returns None if no emails are stored for that folder.
    pub fn get_max_uid(&self, account_id: &str, folder: &str) -> Result<Option<u32>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT MAX(uid) FROM email_metadata WHERE account_id = ?1 AND folder = ?2"
        )?;
        let result: Option<u32> = stmt.query_row(params![account_id, folder], |row| row.get(0))?;
        Ok(result)
    }

    /// Mark an email as read or unread.
    pub fn mark_email_read(&self, email_id: &str, read: bool) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "UPDATE email_metadata SET read = ?1 WHERE id = ?2",
            params![read as i32, email_id],
        )?;
        Ok(())
    }

    /// List all folders that have emails stored for a given account.
    pub fn list_folders(&self, account_id: &str) -> Result<Vec<String>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT folder FROM email_metadata WHERE account_id = ?1"
        )?;
        let folders = stmt.query_map(params![account_id], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(folders)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account::AccountConfig;

    fn test_account() -> AccountConfig {
        AccountConfig::new(
            "Test Account".into(),
            "imap.example.com".into(),
            993,
            "user@example.com".into(),
            "password123".into(),
            "smtp.example.com".into(),
            587,
        )
    }

    #[test]
    fn test_account_config_new() {
        let config = test_account();
        assert!(!config.id.is_empty(), "ID should be generated");
        assert_eq!(config.imap_host, "imap.example.com");
        assert_eq!(config.imap_port, 993);
        assert_eq!(config.username, "user@example.com");
        assert_eq!(config.password, "password123");
        assert_eq!(config.smtp_host, "smtp.example.com");
        assert_eq!(config.smtp_port, 587);
    }

    #[test]
    fn test_open_memory() {
        let storage = Storage::open(":memory:").unwrap();
        // Tables should be created
        let accounts = storage.list_accounts().unwrap();
        assert!(accounts.is_empty());
    }

    #[test]
    fn test_save_and_list_accounts() {
        let storage = Storage::open(":memory:").unwrap();
        let account = test_account();
        storage.save_account(&account).unwrap();
        let accounts = storage.list_accounts().unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].display_name, "Test Account");
        assert_eq!(accounts[0].imap_host, "imap.example.com");
    }

    #[test]
    fn test_get_account() {
        let storage = Storage::open(":memory:").unwrap();
        let account = test_account();
        storage.save_account(&account).unwrap();
        let fetched = storage.get_account(&account.id).unwrap().unwrap();
        assert_eq!(fetched.id, account.id);
        assert_eq!(fetched.username, "user@example.com");
    }

    #[test]
    fn test_delete_account() {
        let storage = Storage::open(":memory:").unwrap();
        let account = test_account();
        storage.save_account(&account).unwrap();
        storage.delete_account(&account.id).unwrap();
        let accounts = storage.list_accounts().unwrap();
        assert!(accounts.is_empty());
    }

    #[test]
    fn test_get_nonexistent_account() {
        let storage = Storage::open(":memory:").unwrap();
        let result = storage.get_account("nonexistent-id").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_save_and_list_emails() {
        let storage = Storage::open(":memory:").unwrap();
        // Must save an account first due to foreign key constraint
        let account = AccountConfig::new(
            "Test".into(),
            "imap.example.com".into(),
            993,
            "user@example.com".into(),
            "pass".into(),
            "smtp.example.com".into(),
            587,
        );
        storage.save_account(&account).unwrap();
        let email = EmailMetadata {
            id: "email-1".into(),
            account_id: account.id.clone(),
            folder: "INBOX".into(),
            uid: 42,
            subject: "Hello".into(),
            from_addr: "a@b.com".into(),
            date: "2026-01-01T00:00:00Z".into(),
            read: false,
            body_path: "/tmp/email-1.txt".into(),
        };
        storage.save_email_metadata(&email).unwrap();
        let emails = storage.list_emails_by_folder(&account.id, "INBOX").unwrap();
        assert_eq!(emails.len(), 1);
        assert_eq!(emails[0].subject, "Hello");
        assert_eq!(emails[0].uid, 42);
    }

    #[test]
    fn test_get_max_uid_empty() {
        let storage = Storage::open(":memory:").unwrap();
        let max = storage.get_max_uid("acct-1", "INBOX").unwrap();
        assert!(max.is_none());
    }

    #[test]
    fn test_get_max_uid_with_data() {
        let storage = Storage::open(":memory:").unwrap();
        let account = AccountConfig::new(
            "Test".into(),
            "imap.example.com".into(),
            993,
            "user@example.com".into(),
            "pass".into(),
            "smtp.example.com".into(),
            587,
        );
        storage.save_account(&account).unwrap();
        for uid in [10u32, 20, 30] {
            let email = EmailMetadata {
                id: format!("email-{}", uid),
                account_id: account.id.clone(),
                folder: "INBOX".into(),
                uid,
                subject: "Test".into(),
                from_addr: "a@b.com".into(),
                date: "2026-01-01".into(),
                read: false,
                body_path: String::new(),
            };
            storage.save_email_metadata(&email).unwrap();
        }
        let max = storage.get_max_uid(&account.id, "INBOX").unwrap();
        assert_eq!(max, Some(30));
    }

    #[test]
    fn test_mark_email_read() {
        let storage = Storage::open(":memory:").unwrap();
        let account = AccountConfig::new(
            "Test".into(),
            "imap.example.com".into(),
            993,
            "user@example.com".into(),
            "pass".into(),
            "smtp.example.com".into(),
            587,
        );
        storage.save_account(&account).unwrap();
        let email = EmailMetadata {
            id: "email-1".into(),
            account_id: account.id.clone(),
            folder: "INBOX".into(),
            uid: 1,
            subject: "Test".into(),
            from_addr: "a@b.com".into(),
            date: "2026-01-01".into(),
            read: false,
            body_path: String::new(),
        };
        storage.save_email_metadata(&email).unwrap();
        storage.mark_email_read("email-1", true).unwrap();
        let emails = storage.list_emails_by_folder(&account.id, "INBOX").unwrap();
        assert!(emails[0].read);
    }

    #[test]
    fn test_list_folders() {
        let storage = Storage::open(":memory:").unwrap();
        let account = AccountConfig::new(
            "Test".into(),
            "imap.example.com".into(),
            993,
            "user@example.com".into(),
            "pass".into(),
            "smtp.example.com".into(),
            587,
        );
        storage.save_account(&account).unwrap();
        for (folder, uid) in [("INBOX", 1u32), ("Sent", 2u32), ("INBOX", 3u32)] {
            let email = EmailMetadata {
                id: format!("email-{}-{}", folder, uid),
                account_id: account.id.clone(),
                folder: folder.into(),
                uid,
                subject: "Test".into(),
                from_addr: "a@b.com".into(),
                date: "2026-01-01".into(),
                read: false,
                body_path: String::new(),
            };
            storage.save_email_metadata(&email).unwrap();
        }
        let folders = storage.list_folders(&account.id).unwrap();
        assert_eq!(folders.len(), 2);
        assert!(folders.contains(&"INBOX".to_string()));
        assert!(folders.contains(&"Sent".to_string()));
    }
}