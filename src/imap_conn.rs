use native_tls::TlsConnector;
use crate::account::{AccountConfig, EmailMetadata};

/// Result of a connection validation attempt.
#[derive(Debug)]
pub enum ConnectionResult {
    Success,
    ConnectionFailed(String),
    AuthFailed(String),
    TlsFailed(String),
    Timeout(String),
}

impl std::fmt::Display for ConnectionResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionResult::Success => write!(f, "Connection successful"),
            ConnectionResult::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            ConnectionResult::AuthFailed(msg) => write!(f, "Authentication failed: {}", msg),
            ConnectionResult::TlsFailed(msg) => write!(f, "TLS error: {}", msg),
            ConnectionResult::Timeout(msg) => write!(f, "Connection timed out: {}", msg),
        }
    }
}

/// Validate an IMAP connection by connecting, starting TLS, and logging in.
/// Returns Ok(ConnectionResult::Success) on success, or Ok with specific failure variant.
pub fn validate_imap(config: &AccountConfig) -> Result<ConnectionResult, String> {
    let tls = TlsConnector::new()
        .map_err(|e| format!("Failed to create TLS connector: {}", e))?;

    // Connect with TLS
    let addr = format!("{}:{}", config.imap_host, config.imap_port);

    let client = match imap::connect(&addr, &config.imap_host, &tls) {
        Ok(c) => c,
        Err(e) => {
            return Ok(ConnectionResult::ConnectionFailed(e.to_string()));
        }
    };

    // Login
    match client.login(&config.username, &config.password) {
        Ok(mut session) => {
            // Logout cleanly
            session.logout().ok();
            Ok(ConnectionResult::Success)
        }
        Err((e, _client)) => {
            Ok(ConnectionResult::AuthFailed(e.to_string()))
        }
    }
}

/// Connect to IMAP and return a session (for sync engine use in Plan 02).
/// This is separate from validate — it returns the live session.
pub fn connect_imap(config: &AccountConfig) -> Result<imap::Session<native_tls::TlsStream<std::net::TcpStream>>, String> {
    let tls = TlsConnector::new()
        .map_err(|e| format!("Failed to create TLS connector: {}", e))?;
    let addr = format!("{}:{}", config.imap_host, config.imap_port);
    let client = imap::connect(&addr, &config.imap_host, &tls)
        .map_err(|e| format!("IMAP connection failed: {}", e))?;
    client.login(&config.username, &config.password)
        .map_err(|(e, _client)| {
            format!("IMAP login failed: {}", e)
        })
}

/// Fetch the list of folders from the IMAP server.
/// Returns a Vec of folder names (e.g., ["INBOX", "Sent", "Drafts", "Trash"]).
pub fn fetch_folders(
    session: &mut imap::Session<native_tls::TlsStream<std::net::TcpStream>>,
) -> Result<Vec<String>, String> {
    let list = session.list(Some(""), Some("*"))
        .map_err(|e| format!("IMAP LIST failed: {}", e))?;
    let folders: Vec<String> = list.iter()
        .map(|name| name.name().to_string())
        .collect();
    Ok(folders)
}

/// Fetch message UIDs and headers from a specific IMAP folder.
/// Returns a Vec of EmailMetadata with uid, subject, from, date populated.
/// body_path and id are set later by the sync engine when saving.
pub fn fetch_message_headers(
    session: &mut imap::Session<native_tls::TlsStream<std::net::TcpStream>>,
    folder: &str,
    account_id: &str,
) -> Result<Vec<EmailMetadata>, String> {
    session.select(folder)
        .map_err(|e| format!("IMAP SELECT {} failed: {}", folder, e))?;

    // Fetch all message UIDs with ENVELOPE (subject, from, date) and FLAGS
    let messages = session.uid_fetch("1:*", "(UID ENVELOPE FLAGS)")
        .map_err(|e| format!("IMAP FETCH failed on {}: {}", folder, e))?;

    let mut results = Vec::new();
    for msg in messages.iter() {
        let uid = msg.uid.unwrap_or(0);
        if uid == 0 { continue; }

        // envelope() returns Option<&Envelope> — extract fields if present
        let envelope = match msg.envelope() {
            Some(env) => env,
            None => continue, // skip messages without envelope
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

        results.push(EmailMetadata {
            id: String::new(),        // Set by sync engine (UUID)
            account_id: account_id.to_string(),
            folder: folder.to_string(),
            uid,
            subject,
            from_addr,
            date,
            read,
            body_path: String::new(), // Set by sync engine after saving body
            content_type: String::new(), // Set by sync engine from BODYSTRUCTURE
            in_reply_to: String::new(),  // Set by sync engine from thread headers
            thread_id: String::new(),    // Set by sync engine from thread headers
            has_attachments: false,       // Set by sync engine from BODYSTRUCTURE
        });
    }

    Ok(results)
}

/// Fetch the full body (RFC822.TEXT) of a specific message by UID.
/// Returns the body as a String.
pub fn fetch_message_body(
    session: &mut imap::Session<native_tls::TlsStream<std::net::TcpStream>>,
    uid: u32,
) -> Result<String, String> {
    let messages = session.uid_fetch(
        uid.to_string().as_str(),
        "RFC822.TEXT"
    ).map_err(|e| format!("IMAP FETCH body for UID {} failed: {}", uid, e))?;

    let msg = messages.iter().next()
        .ok_or_else(|| format!("No message found for UID {}", uid))?;

    msg.body()
        .map(|bytes| String::from_utf8_lossy(bytes).to_string())
        .ok_or_else(|| format!("No body in message UID {}", uid))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account::AccountConfig;

    fn bad_host_config() -> AccountConfig {
        AccountConfig::new(
            "Bad Host".into(),
            "nonexistent.invalid.host.example.com".into(), // DNS will fail
            993,
            "user@test.com".into(),
            "password".into(),
            "smtp.invalid.host.example.com".into(),
            587,
        )
    }

    #[test]
    fn test_validate_imap_bad_host() {
        let config = bad_host_config();
        let result = validate_imap(&config);
        // Should return Ok with a ConnectionFailed variant (not panic or Err)
        assert!(result.is_ok());
        match result.unwrap() {
            ConnectionResult::ConnectionFailed(_) | ConnectionResult::TlsFailed(_) => {},
            other => panic!("Expected connection failure, got: {:?}", other),
        }
    }

    #[test]
    fn test_fetch_folders_bad_host() {
        // connect_imap should fail on bad host, so fetch_folders can't even be called
        let config = bad_host_config();
        let result = connect_imap(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_email_metadata_fields() {
        let meta = crate::account::EmailMetadata {
            id: "test-id".into(),
            account_id: "acct-id".into(),
            folder: "INBOX".into(),
            uid: 42,
            subject: "Test".into(),
            from_addr: "a@b.com".into(),
            date: "2026-01-01".into(),
            read: false,
            body_path: "/tmp/body.txt".into(),
            content_type: "text/plain".into(),
            in_reply_to: String::new(),
            thread_id: "test-id".into(),
            has_attachments: false,
        };
        assert_eq!(meta.uid, 42);
        assert_eq!(meta.folder, "INBOX");
        assert!(!meta.read);
    }

    // Integration test requiring a live IMAP server
    // Run with: cargo test test_validate_imap_live -- --ignored
    #[test]
    #[ignore]
    fn test_validate_imap_live() {
        let config = AccountConfig::new(
            "Live Test".into(),
            "imap.gmail.com".into(),
            993,
            std::env::var("TEST_IMAP_USER").unwrap_or_default(),
            std::env::var("TEST_IMAP_PASS").unwrap_or_default(),
            "smtp.gmail.com".into(),
            465,
        );
        let result = validate_imap(&config);
        assert!(result.is_ok());
        match result.unwrap() {
            ConnectionResult::Success => {},
            other => panic!("Expected success, got: {}", other),
        }
    }
}