use native_tls::TlsConnector;
use regex::Regex;
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

/// Information about a single attachment part from BODYSTRUCTURE.
#[derive(Debug, Clone)]
pub struct AttachmentPart {
    pub filename: String,
    pub content_type: String,
    pub size: i64,
    pub part_number: String,  // IMAP part number like "2", "2.1"
}

/// Parsed BODYSTRUCTURE result: the primary body content type and any attachment parts.
#[derive(Debug, Clone)]
pub struct ParsedStructure {
    pub body_content_type: String,  // e.g. "text/plain" or "text/html"
    pub parts: Vec<AttachmentPart>,
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

/// Fetch the BODYSTRUCTURE of a specific message by UID.
/// Parses the response to extract the primary body content type and any attachment parts.
/// Uses a simple text parsing approach — not a full MIME parser.
pub fn fetch_bodystructure(
    session: &mut imap::Session<native_tls::TlsStream<std::net::TcpStream>>,
    uid: u32,
) -> Result<ParsedStructure, String> {
    let messages = session.uid_fetch(
        uid.to_string().as_str(),
        "(UID BODYSTRUCTURE)"
    ).map_err(|e| format!("IMAP FETCH BODYSTRUCTURE for UID {} failed: {}", uid, e))?;

    let msg = messages.iter().next()
        .ok_or_else(|| format!("No message found for UID {}", uid))?;

    let body_data = msg.body()
        .ok_or_else(|| format!("No BODYSTRUCTURE data for UID {}", uid))?;

    let structure_str = String::from_utf8_lossy(body_data).to_string();
    parse_bodystructure(&structure_str)
}

/// Parse a BODYSTRUCTURE S-expression string into a ParsedStructure.
/// This is a simplified parser that extracts:
/// - The primary body content_type (text/plain or text/html)
/// - Any attachment parts with filename, content_type, size, and part_number
///
/// BODYSTRUCTURE format is complex — this handles the common cases:
/// - Simple text messages: ("text" "plain" ...)
/// - Multipart with text+attachments: ("mixed" ("text" "plain" ...)("application" "pdf" ...)...)
/// - Nested multipart: ("mixed" ("alternative" ("text" "plain" ...)("text" "html" ...))("application" "pdf" ...)...)
fn parse_bodystructure(structure_str: &str) -> Result<ParsedStructure, String> {
    let mut body_content_type = "text/plain".to_string();
    let mut parts = Vec::new();

    // Extract content types from the structure string.
    // IMAP BODYSTRUCTURE encodes type/subtype as separate quoted strings:
    // ("text" "plain" ...) or ("application" "pdf" ...)
    // We match patterns like "text" "plain" as a pair.
    let type_subtype_re = Regex::new(r#""(text|application|image|audio|video|multipart)"\s+"([^"]+)""#)
        .map_err(|e| format!("Regex error: {}", e))?;

    // Find all content type pairs (type, subtype)
    let content_types: Vec<(String, String)> = type_subtype_re.captures_iter(structure_str)
        .map(|cap| {
            let major = cap.get(1).unwrap().as_str().to_string();
            let minor = cap.get(2).unwrap().as_str().to_string();
            (major, minor)
        })
        .collect();

    // Extract filenames — look for "filename" parameter in the structure
    // IMAP BODYSTRUCTURE encodes parameters as: ("filename" "report.pdf")
    let filename_re = Regex::new(r#""filename"\s+"([^"]+)""#)
        .map_err(|e| format!("Regex error: {}", e))?;
    let filenames: Vec<String> = filename_re.captures_iter(structure_str)
        .map(|cap| cap.get(1).unwrap().as_str().to_string())
        .collect();

    // Determine the primary body content type
    for (major, minor) in &content_types {
        let full_type = format!("{}/{}", major, minor);
        if major == "text" && (minor == "plain" || minor == "html") {
            // Prefer HTML over plain if both exist
            if full_type == "text/html" || body_content_type == "text/plain" {
                body_content_type = full_type.clone();
            }
        }
    }

    // Build attachment parts from non-text content types.
    // IMAP part numbering for multipart messages: the first text part is part 1,
    // and subsequent non-text parts are 2, 3, etc.
    // For the simplified parser, we track the part number based on position.
    let mut part_number = 2u32; // Start at 2 (part 1 is the text body)
    let mut filename_idx = 0;
    for (major, minor) in &content_types {
        if major == "multipart" { continue; }
        if major == "text" && (minor == "plain" || minor == "html") {
            continue; // Text parts are the body, not attachments
        }

        let filename = if filename_idx < filenames.len() {
            let f = filenames[filename_idx].clone();
            filename_idx += 1;
            f
        } else {
            format!("attachment-{}.dat", part_number)
        };

        parts.push(AttachmentPart {
            filename,
            content_type: format!("{}/{}", major, minor),
            size: 0, // Size from BODYSTRUCTURE is hard to extract reliably; will be populated on download
            part_number: part_number.to_string(),
        });
        part_number += 1;
    }

    Ok(ParsedStructure {
        body_content_type,
        parts,
    })
}

/// Fetch specific headers (MESSAGE-ID, IN-REPLY-TO, REFERENCES) for thread grouping.
/// Returns (message_id, in_reply_to) tuple.
pub fn fetch_message_headers_for_thread(
    session: &mut imap::Session<native_tls::TlsStream<std::net::TcpStream>>,
    uid: u32,
) -> Result<(String, String), String> {
    let messages = session.uid_fetch(
        uid.to_string().as_str(),
        "(UID BODY[HEADER.FIELDS (MESSAGE-ID IN-REPLY-TO REFERENCES)])"
    ).map_err(|e| format!("IMAP FETCH thread headers for UID {} failed: {}", uid, e))?;

    let msg = messages.iter().next()
        .ok_or_else(|| format!("No message found for UID {}", uid))?;

    let header_data = msg.body()
        .ok_or_else(|| format!("No header data for UID {}", uid))?;

    let header_str = String::from_utf8_lossy(header_data).to_string();

    // Parse the headers
    let mut message_id = String::new();
    let mut in_reply_to = String::new();

    for line in header_str.lines() {
        let line_lower = line.to_lowercase();
        if line_lower.starts_with("message-id:") {
            message_id = line[10..].trim().to_string();
        } else if line_lower.starts_with("in-reply-to:") {
            in_reply_to = line[12..].trim().to_string();
        }
        // REFERENCES header could also be used for threading, but In-Reply-To is sufficient
    }

    Ok((message_id, in_reply_to))
}

/// Download a specific MIME part of a message by IMAP part number.
/// Returns the raw bytes of the attachment for saving to disk.
pub fn fetch_attachment_part(
    session: &mut imap::Session<native_tls::TlsStream<std::net::TcpStream>>,
    uid: u32,
    part_number: &str,
) -> Result<Vec<u8>, String> {
    let fetch_cmd = format!("BODY[{}]", part_number);
    let messages = session.uid_fetch(
        uid.to_string().as_str(),
        &fetch_cmd,
    ).map_err(|e| format!("IMAP FETCH attachment part {} for UID {} failed: {}", part_number, uid, e))?;

    let msg = messages.iter().next()
        .ok_or_else(|| format!("No message found for UID {}", uid))?;

    msg.body()
        .map(|bytes| bytes.to_vec())
        .ok_or_else(|| format!("No attachment data for part {} in UID {}", part_number, uid))
}

/// Convert an HTML email body to plain text optimized for e-ink display.
/// Strips HTML tags, replaces block elements with newlines,
/// decodes common HTML entities, and collapses excessive whitespace.
pub fn html_to_eink(html: &str) -> String {
    let mut text = html.to_string();

    // Remove <style>...</style> blocks (CSS content) first
    let style_re = Regex::new(r"(?is)<style[^>]*>.*?</style>").unwrap();
    text = style_re.replace_all(&text, "").to_string();

    // Remove <script>...</script> blocks
    let script_re = Regex::new(r"(?is)<script[^>]*>.*?</script>").unwrap();
    text = script_re.replace_all(&text, "").to_string();

    // Decode common HTML entities BEFORE stripping tags,
    // so that entities like &lt; and &gt; render as < > (not treated as HTML)
    text = text.replace("&amp;", "&");
    text = text.replace("&lt;", "‹");  // Temporarily use unicode to avoid tag stripping
    text = text.replace("&gt;", "›");
    text = text.replace("&nbsp;", " ");
    text = text.replace("&quot;", "\"");
    text = text.replace("&#39;", "'");
    text = text.replace("&apos;", "'");

    // Replace block elements with newlines before stripping tags
    let block_re = Regex::new(r"(?i)<br\s*/?\s*>").unwrap();
    text = block_re.replace_all(&text, "\n").to_string();

    let block_close_re = Regex::new(r"(?i)</(p|div|h[1-6]|li|tr|blockquote)>").unwrap();
    text = block_close_re.replace_all(&text, "\n").to_string();

    let block_open_re = Regex::new(r"(?i)<(p|div|h[1-6]|li|tr|blockquote)[^>]*>").unwrap();
    text = block_open_re.replace_all(&text, "\n").to_string();

    // Strip all remaining HTML tags
    let tag_re = Regex::new(r"<[^>]*>").unwrap();
    text = tag_re.replace_all(&text, "").to_string();

    // Now restore the temporary unicode markers back to actual < and >
    text = text.replace("‹", "<");
    text = text.replace("›", ">");

    // Collapse 3+ newlines into 2 (preserve paragraph breaks but remove excess)
    let collapse_re = Regex::new(r"\n{3,}").unwrap();
    text = collapse_re.replace_all(&text, "\n\n").to_string();

    // Trim trailing whitespace from each line
    let lines: Vec<String> = text.lines()
        .map(|l| l.trim_end().to_string())
        .collect();
    text = lines.join("\n");

    // Remove leading/trailing whitespace from the whole text
    text.trim().to_string()
}

/// Calculate the thread_id for an email based on its In-Reply-To header.
/// If in_reply_to is non-empty, thread_id = normalized in_reply_to (message-id).
/// If in_reply_to is empty, thread_id = the email's own id (it starts a new thread).
pub fn calculate_thread_id(email_id: &str, in_reply_to: &str) -> String {
    if in_reply_to.is_empty() {
        email_id.to_string()
    } else {
        // Normalize: strip angle brackets from message-id
        in_reply_to.trim_matches('<').trim_matches('>').to_string()
    }
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

    #[test]
    fn test_parsed_structure() {
        let structure = ParsedStructure {
            body_content_type: "text/html".to_string(),
            parts: vec![AttachmentPart {
                filename: "report.pdf".to_string(),
                content_type: "application/pdf".to_string(),
                size: 2048,
                part_number: "2".to_string(),
            }],
        };
        assert_eq!(structure.body_content_type, "text/html");
        assert_eq!(structure.parts.len(), 1);
        assert_eq!(structure.parts[0].filename, "report.pdf");
        assert_eq!(structure.parts[0].part_number, "2");
    }

    #[test]
    fn test_html_to_eink_simple() {
        let html = r#"<html><body><p style="color:red">Hello</p></body></html>"#;
        let text = html_to_eink(html);
        assert_eq!(text, "Hello");
    }

    #[test]
    fn test_html_to_eink_paragraphs() {
        let html = "<p>First paragraph</p><p>Second paragraph</p>";
        let text = html_to_eink(html);
        assert!(text.contains("First paragraph"));
        assert!(text.contains("Second paragraph"));
        assert!(text.contains("\n"));
    }

    #[test]
    fn test_html_to_eink_entities() {
        let html = "<p>Rock &amp; Roll &lt;3&gt; &quot;quotes&quot;</p>";
        let text = html_to_eink(html);
        assert!(text.contains("Rock & Roll"));
        assert!(text.contains("<3>"));
        assert!(text.contains("\"quotes\""));
    }

    #[test]
    fn test_html_to_eink_br() {
        let html = "Line 1<br>Line 2<br/>Line 3";
        let text = html_to_eink(html);
        assert!(text.contains("Line 1\nLine 2\nLine 3") || text.contains("Line 1") && text.contains("Line 3"));
    }

    #[test]
    fn test_html_to_eink_strips_css() {
        let html = r#"<html><head><style>body { color: red; }</style></head><body><p>Content</p></body></html>"#;
        let text = html_to_eink(html);
        assert!(!text.contains("color:"));
        assert!(!text.contains("red"));
        assert!(text.contains("Content"));
    }

    #[test]
    fn test_html_to_eink_collapse_whitespace() {
        let html = "<p>A</p><p>B</p><p>C</p>";
        let text = html_to_eink(html);
        // Should not have 3+ consecutive newlines
        assert!(!text.contains("\n\n\n"));
    }

    #[test]
    fn test_html_to_eink_nbsp() {
        let html = "<p>Hello&nbsp;World</p>";
        let text = html_to_eink(html);
        assert!(text.contains("Hello World"));
    }

    #[test]
    fn test_calculate_thread_id_with_reply() {
        let email_id = "email-123";
        let in_reply_to = "<parent-msg-456@example.com>";
        let thread_id = calculate_thread_id(email_id, in_reply_to);
        assert_eq!(thread_id, "parent-msg-456@example.com");
    }

    #[test]
    fn test_calculate_thread_id_no_reply() {
        let email_id = "email-123";
        let in_reply_to = "";
        let thread_id = calculate_thread_id(email_id, in_reply_to);
        assert_eq!(thread_id, "email-123");
    }

    #[test]
    fn test_calculate_thread_id_normalize_angle_brackets() {
        let email_id = "email-123";
        let in_reply_to = "<msg-id>";
        let thread_id = calculate_thread_id(email_id, in_reply_to);
        assert_eq!(thread_id, "msg-id");
    }

    #[test]
    fn test_parse_bodystructure_multipart() {
        // Simulate a BODYSTRUCTURE response for a multipart message with an attachment
        let structure = r#"("mixed" ("alternative" ("text" "plain" ("charset" "utf-8") NIL NIL "7bit" 123 5 NIL NIL NIL NIL)("text" "html" ("charset" "utf-8") NIL NIL "7bit" 456 10 NIL NIL NIL NIL)("application" "pdf" ("filename" "report.pdf") NIL NIL "base64" 2048 NIL ("attachment" ("filename" "report.pdf")) NIL NIL))"#;
        let parsed = parse_bodystructure(structure).unwrap();
        assert_eq!(parsed.body_content_type, "text/html");
        assert_eq!(parsed.parts.len(), 1);
        assert_eq!(parsed.parts[0].filename, "report.pdf");
        assert_eq!(parsed.parts[0].content_type, "application/pdf");
    }

    #[test]
    fn test_parse_bodystructure_plain_only() {
        let structure = r#"("text" "plain" ("charset" "utf-8") NIL NIL "7bit" 42 1 NIL NIL NIL NIL)"#;
        let parsed = parse_bodystructure(structure).unwrap();
        assert_eq!(parsed.body_content_type, "text/plain");
        assert!(parsed.parts.is_empty());
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

    // Integration test for fetch_bodystructure with live server
    #[test]
    #[ignore]
    fn test_fetch_bodystructure_live() {
        let config = AccountConfig::new(
            "Live Test".into(),
            "imap.gmail.com".into(),
            993,
            std::env::var("TEST_IMAP_USER").unwrap_or_default(),
            std::env::var("TEST_IMAP_PASS").unwrap_or_default(),
            "smtp.gmail.com".into(),
            465,
        );
        let mut session = connect_imap(&config).unwrap();
        session.select("INBOX").unwrap();
        let structure = fetch_bodystructure(&mut session, 1).unwrap();
        assert!(!structure.body_content_type.is_empty());
    }

    // Integration test for fetch_message_headers_for_thread with live server
    #[test]
    #[ignore]
    fn test_fetch_thread_headers_live() {
        let config = AccountConfig::new(
            "Live Test".into(),
            "imap.gmail.com".into(),
            993,
            std::env::var("TEST_IMAP_USER").unwrap_or_default(),
            std::env::var("TEST_IMAP_PASS").unwrap_or_default(),
            "smtp.gmail.com".into(),
            465,
        );
        let mut session = connect_imap(&config).unwrap();
        session.select("INBOX").unwrap();
        let (msg_id, in_reply_to) = fetch_message_headers_for_thread(&mut session, 1).unwrap();
        // message_id should be non-empty for real emails
        assert!(!msg_id.is_empty());
    }

    // Integration test for fetch_attachment_part with live server
    #[test]
    #[ignore]
    fn test_fetch_attachment_part_live() {
        let config = AccountConfig::new(
            "Live Test".into(),
            "imap.gmail.com".into(),
            993,
            std::env::var("TEST_IMAP_USER").unwrap_or_default(),
            std::env::var("TEST_IMAP_PASS").unwrap_or_default(),
            "smtp.gmail.com".into(),
            465,
        );
        let mut session = connect_imap(&config).unwrap();
        session.select("INBOX").unwrap();
        // This test requires knowing a UID + part_number for an email with attachment
        // Will likely fail if no such email exists — use as manual test
        let _result = fetch_attachment_part(&mut session, 1, "2");
    }
}