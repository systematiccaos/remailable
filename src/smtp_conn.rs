use crate::account::AccountConfig;
use lettre::SmtpTransport;
use lettre::transport::smtp::authentication::Credentials;
use super::imap_conn::ConnectionResult;

/// Validate SMTP connection by connecting and authenticating.
/// Does NOT send an actual email — just verifies the connection works.
pub fn validate_smtp(config: &AccountConfig) -> Result<ConnectionResult, String> {
    let creds = Credentials::new(config.username.clone(), config.password.clone());

    // Build SMTP transport based on port
    // Port 465 = SMTPS (implicit TLS), Port 587 = STARTTLS
    let transport_result = if config.smtp_port == 465 {
        SmtpTransport::relay(&config.smtp_host)
            .map_err(|e| format!("SMTP relay build failed: {}", e))
            .map(|builder| builder.credentials(creds).port(config.smtp_port).build())
    } else {
        // STARTTLS (port 587 or other)
        SmtpTransport::starttls_relay(&config.smtp_host)
            .map_err(|e| format!("SMTP STARTTLS build failed: {}", e))
            .map(|builder| builder.credentials(creds).port(config.smtp_port).build())
    };

    let transport = match transport_result {
        Ok(t) => t,
        Err(e) => return Ok(ConnectionResult::TlsFailed(e)),
    };

    // Test the connection by calling .test_connection()
    match transport.test_connection() {
        Ok(true) => Ok(ConnectionResult::Success),
        Ok(false) => Ok(ConnectionResult::ConnectionFailed(
            "SMTP test_connection returned false".into(),
        )),
        Err(e) => Ok(ConnectionResult::ConnectionFailed(e.to_string())),
    }
}

/// Build an SMTP transport for sending (used by Phase 4).
/// Returns the transport ready for use, does NOT test the connection.
pub fn build_smtp_transport(config: &AccountConfig) -> Result<SmtpTransport, String> {
    let creds = Credentials::new(config.username.clone(), config.password.clone());
    if config.smtp_port == 465 {
        SmtpTransport::relay(&config.smtp_host)
            .map_err(|e| format!("SMTP relay build failed: {}", e))
            .map(|builder| builder.credentials(creds).port(config.smtp_port).build())
    } else {
        SmtpTransport::starttls_relay(&config.smtp_host)
            .map_err(|e| format!("SMTP STARTTLS build failed: {}", e))
            .map(|builder| builder.credentials(creds).port(config.smtp_port).build())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account::AccountConfig;

    #[test]
    fn test_validate_smtp_bad_host() {
        let config = AccountConfig::new(
            "Bad Host".into(),
            "imap.invalid.example.com".into(),
            993,
            "user@test.com".into(),
            "password".into(),
            "nonexistent.invalid.smtp.example.com".into(),
            587,
        );
        let result = validate_smtp(&config);
        assert!(result.is_ok());
        match result.unwrap() {
            ConnectionResult::ConnectionFailed(_) |
            ConnectionResult::TlsFailed(_) => {},
            other => panic!("Expected failure, got: {:?}", other),
        }
    }

    #[test]
    #[ignore]
    fn test_validate_smtp_live() {
        let config = AccountConfig::new(
            "Live Test".into(),
            "imap.gmail.com".into(),
            993,
            std::env::var("TEST_IMAP_USER").unwrap_or_default(),
            std::env::var("TEST_IMAP_PASS").unwrap_or_default(),
            "smtp.gmail.com".into(),
            465,
        );
        let result = validate_smtp(&config);
        assert!(result.is_ok());
        match result.unwrap() {
            ConnectionResult::Success => {},
            other => panic!("Expected success, got: {}", other),
        }
    }
}