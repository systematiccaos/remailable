use rusqlite::{params, Connection};
use crate::account::AccountConfig;

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
}