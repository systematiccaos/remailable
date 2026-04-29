use native_tls::TlsConnector;
use crate::account::AccountConfig;

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