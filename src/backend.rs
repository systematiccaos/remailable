//! remailable backend — headless process for AppLoad
//!
//! Runs as a backend service, communicating with the QML frontend
//! via the AppLoad SEQPACKET socket protocol.
//! No Qt/QML dependency — pure Rust.

pub mod appload;
pub mod account;
pub mod storage;

use std::sync::{Arc, Mutex};

use account::{AccountConfig, EmailMetadata};
use serde_json::{json};

use appload::{AppLoadClient, MSG_REQUEST, SYS_NEW_FRONTEND, SYS_TERMINATE};
use base64::Engine;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let socket_path = args.get(1).expect("Usage: remailable-backend <socket_path>");

    // Initialize storage
    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("/home/root/.local/share"))
        .join("remailable");
    std::fs::create_dir_all(&data_dir).ok();
    let db_path = data_dir.join("remailable.db");
    let db = storage::Storage::open(db_path.to_str().unwrap_or(":memory:"))
        .expect("Failed to open database");
    let storage = Arc::new(Mutex::new(db));
    let data_dir_arc = Arc::new(data_dir);

    // Connect to AppLoad socket — retry for up to 5 seconds
    let mut client: Option<AppLoadClient> = None;
    for attempt in 0..50 {
        eprintln!("Connecting to AppLoad socket: {} (attempt {})", socket_path, attempt + 1);
        match AppLoadClient::connect(socket_path) {
            Ok(c) => {
                eprintln!("Connected to AppLoad socket");
                client = Some(c);
                break;
            }
            Err(e) => {
                eprintln!("Connect failed: {}, retrying in 100ms...", e);
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }
    }
    let client = Arc::new(Mutex::new(client.expect("Failed to connect to AppLoad socket after 50 retries")));

    let running = Arc::new(Mutex::new(true));

    // Main message loop
    while *running.lock().unwrap() {
        let (msg_type, payload) = match client.lock().unwrap().recv() {
            Ok(result) => result,
            Err(e) => {
                eprintln!("recv error: {}", e);
                std::thread::sleep(std::time::Duration::from_millis(100));
                continue;
            }
        };

        match msg_type {
            SYS_NEW_FRONTEND => {
                eprintln!("New frontend connected, sending initial state");
                let accounts = storage.lock().unwrap().list_accounts().unwrap_or_default();
                let c = client.lock().unwrap();
                let _ = c.send_event("initial_state", json!({
                    "account_count": accounts.len(),
                    "current_view": "account_list",
                    "sync_status": "idle",
                    "accounts": accounts.iter().map(|a| json!({
                        "id": a.id,
                        "display_name": a.display_name,
                        "email": a.email,
                        "imap_host": a.imap_host,
                    })).collect::<Vec<_>>()
                }));
            }
            SYS_TERMINATE => {
                eprintln!("Terminate received, shutting down");
                *running.lock().unwrap() = false;
            }
            MSG_REQUEST => {
                if let Some(msg) = payload {
                    handle_request(&client, &storage, &data_dir_arc, msg);
                }
            }
            _ => {
                eprintln!("Unknown message type: {}", msg_type);
            }
        }
    }

    eprintln!("Backend exiting");
}

fn handle_request(
    client: &Arc<Mutex<AppLoadClient>>,
    storage: &Arc<Mutex<storage::Storage>>,
    data_dir: &Arc<std::path::PathBuf>,
    msg: appload::AppLoadMessage,
) {
    let id = msg.id.unwrap_or(0);
    let action = msg.action.as_deref().unwrap_or("");

    eprintln!("handle_request: action={} id={}", action, id);

    match action {
        "get_accounts" => {
            let accounts = storage.lock().unwrap().list_accounts().unwrap_or_default();
            eprintln!("get_accounts: found {} accounts", accounts.len());
            let list: Vec<serde_json::Value> = accounts.iter().map(|a| json!({
                "id": a.id,
                "display_name": a.display_name,
                "email": a.email,
                "imap_host": a.imap_host,
            })).collect();
            let c = client.lock().unwrap();
            let _ = c.send_response(id, json!({ "accounts": list }));
        }
        "add_account" => {
            if let Some(params) = &msg.params {
                let config = AccountConfig {
                    id: uuid::Uuid::new_v4().to_string(),
                    display_name: params.get("display_name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    email: params.get("email").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    imap_host: params.get("imap_host").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    imap_port: params.get("imap_port").and_then(|v| v.as_u64()).unwrap_or(993) as u16,
                    smtp_host: params.get("smtp_host").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    smtp_port: params.get("smtp_port").and_then(|v| v.as_u64()).unwrap_or(587) as u16,
                    username: params.get("username").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    password: params.get("password").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                };
                let result = storage.lock().unwrap().save_account(&config);
                let c = client.lock().unwrap();
                match result {
                    Ok(_) => {
                        let _ = c.send_response(id, json!({"success": true}));
                        let _ = c.send_event("accounts_changed", json!({}));
                    }
                    Err(e) => {
                        let _ = c.send_response(id, json!({"error": e.to_string(), "success": false}));
                    }
                }
            }
        }
        "remove_account" => {
            if let Some(params) = &msg.params {
                let account_id = params.get("id").and_then(|v| v.as_str()).unwrap_or("");
                let result = storage.lock().unwrap().delete_account(account_id);
                let c = client.lock().unwrap();
                let _ = c.send_response(id, json!({"success": result.is_ok()}));
                let _ = c.send_event("accounts_changed", json!({}));
            }
        }
        "sync" => {
            // Sync all accounts (or a specific one if account_id param given)
            let account_id = msg.params.as_ref()
                .and_then(|p| p.get("account_id"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            // Send syncing status event
            {
                let c = client.lock().unwrap();
                let _ = c.send_event("sync_status", json!({"status": "syncing"}));
            }

            let accounts = if let Some(ref aid) = account_id {
                storage.lock().unwrap().get_account(aid).ok().flatten()
                    .into_iter().collect::<Vec<_>>()
            } else {
                storage.lock().unwrap().list_accounts().unwrap_or_default()
            };

            let mut total_new = 0u32;
            let mut errors = Vec::new();

            for acct in &accounts {
                eprintln!("sync: connecting to {} for account {}", acct.imap_host, acct.display_name);
                match imap_sync(acct, storage, data_dir) {
                    Ok(new_count) => {
                        eprintln!("sync: {} new emails for {}", new_count, acct.display_name);
                        total_new += new_count;
                    }
                    Err(e) => {
                        eprintln!("sync: error for {}: {}", acct.display_name, e);
                        errors.push(format!("{}: {}", acct.display_name, e));
                    }
                }
            }

            let c = client.lock().unwrap();
            if errors.is_empty() {
                let _ = c.send_response(id, json!({"success": true, "new_emails": total_new}));
                let _ = c.send_event("sync_status", json!({"status": "idle", "new_emails": total_new}));
            } else {
                let _ = c.send_response(id, json!({"success": false, "errors": errors, "new_emails": total_new}));
                let _ = c.send_event("sync_status", json!({"status": "error", "errors": errors}));
            }
        }
        "get_folders" => {
            if let Some(params) = &msg.params {
                let account_id = params.get("account_id").and_then(|v| v.as_str()).unwrap_or("");
                // First try stored folders from previous syncs
                let folders = storage.lock().unwrap().list_folders(account_id).unwrap_or_default();
                let c = client.lock().unwrap();
                if folders.is_empty() {
                    // No stored folders yet — return a default list for the account
                    let _ = c.send_response(id, json!({
                        "folders": [
                            {"name": "INBOX", "account_id": account_id}
                        ]
                    }));
                } else {
                    let _ = c.send_response(id, json!({
                        "folders": folders.iter().map(|f| json!({
                            "name": f,
                            "account_id": account_id,
                        })).collect::<Vec<_>>()
                    }));
                }
            }
        }
        "get_emails" => {
            if let Some(params) = &msg.params {
                let account_id = params.get("account_id").and_then(|v| v.as_str()).unwrap_or("");
                let folder = params.get("folder").and_then(|v| v.as_str()).unwrap_or("INBOX");
                let emails = storage.lock().unwrap()
                    .list_emails_by_folder(account_id, folder)
                    .unwrap_or_default();
                let c = client.lock().unwrap();
                let _ = c.send_response(id, json!({
                    "emails": emails.iter().map(|e| json!({
                        "id": e.id,
                        "subject": e.subject,
                        "from": e.from_addr,
                        "date": e.date,
                        "read": e.read,
                        "has_attachments": e.has_attachments,
                        "folder": e.folder,
                        "account_id": e.account_id,
                    })).collect::<Vec<_>>()
                }));
            }
        }
        "get_email_body" => {
            if let Some(params) = &msg.params {
                let email_id = params.get("id").and_then(|v| v.as_str()).unwrap_or("");
                let _account_id = params.get("account_id").and_then(|v| v.as_str()).unwrap_or("");
                let _folder = params.get("folder").and_then(|v| v.as_str()).unwrap_or("INBOX");
                let email = storage.lock().unwrap().get_email(email_id).ok().flatten();
                let c = client.lock().unwrap();
                if let Some(e) = email {
                    let raw = std::fs::read_to_string(&e.body_path).unwrap_or_default();
                    let parsed = parse_email_body(&raw);
                    
                    let mut body = parsed.body;
                    let mut content_type = parsed.content_type;
                    
                    // If we only have headers (content is empty), try to fetch the full body from IMAP
                    if body.trim().is_empty() || body.starts_with("* ") {
                        // Try to fetch full body from IMAP
                        if let Some(account) = storage.lock().unwrap().get_account(&e.account_id).ok().flatten() {
                            match imap_fetch_body(&account, &e.folder, e.uid, data_dir) {
                                Ok(fetched) => {
                                    body = fetched.body;
                                    content_type = fetched.content_type;
                                }
                                Err(err) => {
                                    eprintln!("get_email_body: IMAP fetch failed: {}", err);
                                }
                            }
                        }
                    }
                    
                    // Sanitize content_type — strip charset and other parameters
                    if let Some(idx) = content_type.find(';') {
                        content_type = content_type[..idx].trim().to_string();
                    }
                    // If we extracted HTML from multipart, mark as text/html
                    if content_type.starts_with("multipart/") && (body.contains("<html") || body.contains("<body") || body.contains("<p>")) {
                        content_type = "text/html".to_string();
                    }
                    
                    // Sanitize HTML for e-ink rendering
                    if content_type == "text/html" {
                        body = sanitize_html(&body);
                    }
                   
                    let _ = c.send_response(id, json!({
                        "id": e.id,
                        "subject": e.subject,
                        "from": e.from_addr,
                        "date": e.date,
                        "body": body,
                        "content_type": content_type,
                        "has_attachments": e.has_attachments,
                    }));
                } else {
                    let _ = c.send_response(id, json!({"error": "Email not found"}));
                }
            }
        }
        "ping" => {
            let c = client.lock().unwrap();
            let _ = c.send_response(id, json!({"pong": true}));
        }
        _ => {
            let c = client.lock().unwrap();
            let _ = c.send_response(id, json!({"error": format!("Unknown action: {}", action)}));
        }
    }
}

/// Connect to IMAP server, fetch new emails since last sync, store them locally.
/// Returns the number of new emails fetched.
fn imap_sync(
    account: &AccountConfig,
    storage: &Arc<Mutex<storage::Storage>>,
    data_dir: &std::path::PathBuf,
) -> Result<u32, String> {
    eprintln!("imap_sync: connecting to {}:{}", account.imap_host, account.imap_port);

    // Connect via TLS
    let tls = native_tls::TlsConnector::new()
        .map_err(|e| format!("TLS init failed: {}", e))?;

    let client = imap::connect(
        (account.imap_host.as_str(), account.imap_port),
        account.imap_host.as_str(),
        &tls,
    ).map_err(|e| format!("IMAP connect failed: {}", e))?;

    // Login
    let mut session = client
        .login(&account.username, &account.password)
        .map_err(|e| format!("IMAP login failed: {}", e.0))?;

    // Fetch folders (LIST)
    let folders_list = session.list(Some(""), Some("*"))
        .map_err(|e| format!("IMAP LIST failed: {}", e))?;

    let folder_names: Vec<String> = folders_list.iter()
        .map(|f| f.name().to_string())
        .collect();
    eprintln!("imap_sync: found {} folders", folder_names.len());

    let mut total_new = 0u32;

    // Only sync INBOX for now, limit to recent 50 emails
    let folders_to_sync = vec!["INBOX"];

    for folder_name in &folders_to_sync {
        eprintln!("imap_sync: syncing folder {}", folder_name);

        // Select the folder
        let mailbox = match session.select(folder_name) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("imap_sync: cannot select folder {}: {}", folder_name, e);
                continue;
            }
        };

        let exists = mailbox.exists as u32;
        eprintln!("imap_sync: folder {} has {} messages", folder_name, exists);
        
        if exists == 0 {
            continue;
        }

        // Find the highest UID we've already stored
        let max_uid = storage.lock().unwrap()
            .get_max_uid(&account.id, folder_name)
            .unwrap_or(None)
            .unwrap_or(0);

        // Use sequence numbers to fetch last N messages
        // This avoids the slow UID SEARCH ALL on large mailboxes
        let fetch_count = 50u32.min(exists);
        let start_seq = exists.saturating_sub(fetch_count) + 1;
        let seq_set = format!("{}:{}", start_seq, exists);
        eprintln!("imap_sync: fetching seq {} for folder {} ({} messages total)", seq_set, folder_name, exists);

        let body_dir = data_dir.join("bodies").join(&account.id).join(folder_name);
        std::fs::create_dir_all(&body_dir).ok();

        // Fetch each email individually using raw IMAP commands
        // to work around the imap crate's broken FETCH parser
        for seq_num in start_seq..=exists {
            // Use raw command to fetch UID + RFC822.HEADER
            let command = format!("FETCH {} (UID RFC822.HEADER)", seq_num);
            let raw_response = match session.run_command_and_read_response(&command) {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("imap_sync: raw FETCH {} failed: {}", seq_num, e);
                    continue;
                }
            };

            // Parse the raw response to extract UID and headers
            let response_str = String::from_utf8_lossy(&raw_response);
            let uid = parse_imap_uid(&response_str).unwrap_or(0);
            if uid == 0 {
                eprintln!("imap_sync: could not parse UID from FETCH response for seq {}", seq_num);
                continue;
            }
            if uid <= max_uid {
                continue; // Skip already-synced
            }

            let headers = parse_imap_fetch_headers(&response_str);

            let email_id = uuid::Uuid::new_v4().to_string();
            let body_filename = format!("{}.txt", email_id);
            let body_path = body_dir.join(&body_filename);

            // Save the raw headers as body file
            std::fs::write(&body_path, &raw_response).ok();

            let email = EmailMetadata {
                id: email_id,
                account_id: account.id.clone(),
                folder: folder_name.to_string(),
                uid,
                subject: headers.subject,
                from_addr: headers.from,
                date: headers.date,
                read: false,
                body_path: body_path.to_string_lossy().to_string(),
                content_type: "text/plain".to_string(),
                in_reply_to: headers.in_reply_to,
                thread_id: uuid::Uuid::new_v4().to_string(),
                has_attachments: false,
            };

            storage.lock().unwrap().save_email_metadata(&email).ok();
            total_new += 1;
        }

        eprintln!("imap_sync: folder {} done, {} new emails total", folder_name, total_new);
    }

    // Logout
    let _ = session.logout();

    Ok(total_new)
}

/// Parse UID from a raw IMAP FETCH response like:
/// * 10627 FETCH (UID 85803 RFC822.HEADER {12513}
fn parse_imap_uid(response: &str) -> Option<u32> {
    // Find "UID " followed by digits
    let uid_marker = "UID ";
    if let Some(pos) = response.find(uid_marker) {
        let after = &response[pos + uid_marker.len()..];
        let uid_str: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
        return uid_str.parse().ok();
    }
    None
}

/// Parse email headers from a raw IMAP FETCH response
fn parse_imap_fetch_headers(response: &str) -> ParsedHeaders {
    // The IMAP response looks like:
    // * 1 FETCH (UID 123 RFC822.HEADER {size}\r\n<headers>\r\n)\r\n
    // We need to extract just the headers section
    
    // Find the header data — it's between {size}\r\n and the closing )\r\n
    // Look for RFC822.HEADER in the response, then find the literal size marker
    let mut subject = String::new();
    let mut from = String::new();
    let mut date = String::new();
    let mut in_reply_to = String::new();

    // Find literal section in the response: {N}\r\n...\r\n)
    if let Some(lit_start) = response.find("{") {
        if let Some(lit_end) = response[lit_start..].find("}\r\n") {
            let size_str = &response[lit_start+1..lit_start+lit_end];
            let _size: usize = size_str.parse().unwrap_or(0);
            let data_start = lit_start + lit_end + 3; // skip "}\r\n"
            let header_data = &response[data_start..];
            
            // Parse standard email headers from the literal data
            let parsed = parse_email_headers(header_data.as_bytes());
            subject = parsed.subject;
            from = parsed.from;
            date = parsed.date;
            in_reply_to = parsed.in_reply_to;
        }
    }

    // Fallback: try to find Subject:/From:/Date: in the raw IMAP response
    if subject.is_empty() {
        if let Some(pos) = response.find("Subject: ") {
            let line_start = &response[pos..];
            if let Some(end) = line_start.find('\r') {
                subject = line_start[9..end.min(line_start.len())].to_string();
            }
        }
    }
    if from.is_empty() {
        if let Some(pos) = response.find("From: ") {
            let line_start = &response[pos..];
            if let Some(end) = line_start.find('\r') {
                from = line_start[6..end.min(line_start.len())].to_string();
            }
        }
    }
    if date.is_empty() {
        if let Some(pos) = response.find("Date: ") {
            let line_start = &response[pos..];
            if let Some(end) = line_start.find('\r') {
                date = line_start[6..end.min(line_start.len())].to_string();
            }
        }
    }

    ParsedHeaders { subject, from, date, in_reply_to }
}

/// Parsed email headers from raw RFC822 header bytes
struct ParsedHeaders {
    subject: String,
    from: String,
    date: String,
    in_reply_to: String,
}

/// Parse email headers from raw RFC822 header bytes
fn parse_email_headers(raw: &[u8]) -> ParsedHeaders {
    let text = String::from_utf8_lossy(raw);
    let mut subject = String::new();
    let mut from = String::new();
    let mut date = String::new();
    let mut in_reply_to = String::new();

    // Simple header parsing — handle folded lines (continuation lines start with whitespace)
    let mut current_header = String::new();
    let mut current_value = String::new();

    for line in text.lines() {
        if line.starts_with(char::is_whitespace) {
            // Continuation line — append to current value
            current_value.push(' ');
            current_value.push_str(line.trim());
        } else {
            // Save previous header
            if !current_header.is_empty() {
                let val = current_value.trim().to_string();
                match current_header.to_lowercase().as_str() {
                    "subject" => subject = val,
                    "from" => from = val,
                    "date" => date = val,
                    "in-reply-to" => in_reply_to = val,
                    _ => {}
                }
            }
            // Start new header
            if let Some((name, value)) = line.split_once(':') {
                current_header = name.to_string();
                current_value = value.to_string();
            } else {
                current_header.clear();
                current_value.clear();
            }
        }
    }
    // Save last header
    if !current_header.is_empty() {
        let val = current_value.trim().to_string();
        match current_header.to_lowercase().as_str() {
            "subject" => subject = val,
            "from" => from = val,
            "date" => date = val,
            "in-reply-to" => in_reply_to = val,
            _ => {}
        }
    }

    ParsedHeaders { subject, from, date, in_reply_to }
}

/// Parsed email body from raw file data
struct ParsedBody {
    body: String,
    content_type: String, // "text/html" or "text/plain"
}

/// Parse email body from stored raw data.
/// The stored data could be:
/// 1. Raw IMAP FETCH response with RFC822.HEADER only (no body — need on-demand fetch)
/// 2. Raw IMAP FETCH response with full RFC822 (headers + body)
/// 3. Raw RFC822 email (headers + body, no IMAP wrapper)
fn parse_email_body(raw: &str) -> ParsedBody {
    // Case 1 & 2: Raw IMAP response
    if raw.starts_with("* ") && raw.contains("FETCH") {
        // Check if this is a header-only response (RFC822.HEADER) or full (RFC822 or BODY[])
        let is_header_only = raw.contains("RFC822.HEADER") && !raw.contains("RFC822 ") && !raw.contains("BODY[]");
        
        // Find the literal marker {size}\r\n
        // For IMAP FETCH, the literal marker is right after the data item name
        // We need the FIRST { that's part of the IMAP literal, not ones inside the email
        let lit_start = if let Some(fetch_pos) = raw.find("FETCH") {
            raw[fetch_pos..].find('{').map(|pos| fetch_pos + pos)
        } else {
            raw.find('{')
        };
        
        if let Some(lit_start) = lit_start {
            if let Some(lit_end) = raw[lit_start..].find("}\r\n") {
                let size_str = &raw[lit_start + 1..lit_start + lit_end];
                let literal_size: usize = size_str.parse().unwrap_or(0);
                let data_start = lit_start + lit_end + 3;
                if data_start >= raw.len() {
                    return ParsedBody { body: String::new(), content_type: "text/plain".to_string() };
                }
                let literal_end = (data_start + literal_size).min(raw.len());
                let literal_data = &raw[data_start..literal_end];
                
                if is_header_only {
                    // We only have headers, no body content.
                    let headers = literal_data;
                    let is_html = headers.to_lowercase().contains("content-type: text/html")
                        || headers.to_lowercase().contains("content-type: multipart/");
                    return ParsedBody {
                        body: String::new(), // No body content available
                        content_type: if is_html { "text/html".to_string() } else { "text/plain".to_string() },
                    };
                }
                
                // Full RFC822 message: headers + \r\n\r\n + body
                let body_start = match literal_data.find("\r\n\r\n") {
                    Some(pos) => pos + 4,
                    None => match literal_data.find("\n\n") {
                        Some(pos) => pos + 2,
                        None => 0,
                    }
                };
                
                let headers = &literal_data[..body_start.min(literal_data.len())];
                let raw_body = if body_start < literal_data.len() {
                    literal_data[body_start..].to_string()
                } else {
                    String::new()
                };
                
                let content_type = detect_content_type_from_headers(headers);
                let body_text = maybe_extract_multipart(&raw_body, headers);
                
                
                return ParsedBody { body: body_text, content_type };
            }
        }
        
        // Fallback: couldn't parse literal
        return ParsedBody { body: String::new(), content_type: "text/plain".to_string() };
    }
    
    // Case 3: Raw email (headers + body, no IMAP wrapper)
    let body_start = match raw.find("\r\n\r\n") {
        Some(pos) => pos + 4,
        None => match raw.find("\n\n") {
            Some(pos) => pos + 2,
            None => 0,
        }
    };
    
    let headers = &raw[..body_start.min(raw.len())];
    let raw_body = if body_start < raw.len() {
        raw[body_start..].to_string()
    } else {
        String::new()
    };
    
    let content_type = detect_content_type_from_headers(headers);
    let body_text = maybe_extract_multipart(&raw_body, headers);
    
    ParsedBody { body: body_text, content_type }
}

/// If the body is multipart, extract the best part (HTML preferred).
/// Otherwise, decode content-transfer-encoding and return as-is.
fn maybe_extract_multipart(body: &str, headers: &str) -> String {
    let lower_headers = headers.to_lowercase();
    
    // Check if body is multipart
    if let Some(boundary) = extract_boundary(headers) {
        return extract_multipart_body(body, &boundary);
    }
    
    // Check content-transfer-encoding for single-part messages
    if lower_headers.contains("content-transfer-encoding: base64") {
        return decode_base64(body);
    } else if lower_headers.contains("content-transfer-encoding: quoted-printable") {
        return decode_quoted_printable(body);
    }
    
    body.to_string()
}

/// Detect content type from email headers
fn detect_content_type_from_headers(headers: &str) -> String {
    let lower = headers.to_lowercase();
    if lower.contains("content-type: text/html") || lower.contains("content-type: multipart/") {
        "text/html".to_string()
    } else if lower.contains("<html") || lower.contains("<!doctype html") || lower.contains("<body") {
        "text/html".to_string()
    } else {
        "text/plain".to_string()
    }
}

/// Extract boundary string from Content-Type header (preserves original case)
fn extract_boundary(headers: &str) -> Option<String> {
    let lower = headers.to_lowercase();
    if let Some(pos) = lower.find("boundary=") {
        // Use the position from the lowercase version to index into the original headers
        let after = &headers[pos + 9..];
        // Strip quotes if present
        let after = after.trim_start_matches('"');
        let boundary: String = after.chars().take_while(|c| *c != ';' && *c != '"' && *c != '\r' && *c != '\n' && *c != ' ').collect();
        if !boundary.is_empty() {
            return Some(boundary);
        }
    }
    None
}

/// Extract the best body part from a MIME multipart body.
/// Prefers text/html over text/plain.
fn extract_multipart_body(body: &str, boundary: &str) -> String {
    let delimiter = format!("--{}", boundary);
    let end_delimiter = format!("--{}--", boundary);
    
    // Split by boundary
    let parts: Vec<&str> = body.split(&delimiter).collect();
    
    let mut html_part = String::new();
    let mut plain_part = String::new();
    
    for part in &parts[1..] { // Skip preamble before first boundary
        if part.starts_with("--") || part.starts_with(&end_delimiter[2..]) {
            continue; // End boundary
        }
        
        // Each part has headers and body separated by blank line
        let (part_headers, part_body) = if let Some(pos) = part.find("\r\n\r\n") {
            (&part[..pos], &part[pos + 4..])
        } else if let Some(pos) = part.find("\n\n") {
            (&part[..pos], &part[pos + 2..])
        } else {
            ("", *part)
        };
        
        // Strip trailing \r\n from part body
        let part_body = part_body.trim_end_matches("\r\n").trim_end_matches('\n');
        
        let part_lower = part_headers.to_lowercase();
        let is_html = part_lower.contains("content-type: text/html");
        let is_plain = part_lower.contains("content-type: text/plain");
        
        // Decode content-transfer-encoding to raw bytes
        let decoded_bytes = if part_lower.contains("content-transfer-encoding: base64") {
            decode_base64_to_bytes(part_body)
        } else if part_lower.contains("content-transfer-encoding: quoted-printable") {
            decode_quoted_printable_to_bytes(part_body)
        } else {
            part_body.as_bytes().to_vec()
        };
        
        // Convert bytes to string using the charset from Content-Type
        let charset = extract_charset(part_headers);
        let decoded = charset_decode(&decoded_bytes, &charset);
        
        if is_html && html_part.is_empty() {
            html_part = decoded;
        } else if is_plain && plain_part.is_empty() {
            plain_part = decoded;
        }
    }
    
    // Prefer HTML, fall back to plain
    if !html_part.is_empty() {
        html_part
    } else {
        plain_part
    }
}

/// Decode base64 content to raw bytes
fn decode_base64_to_bytes(input: &str) -> Vec<u8> {
    let clean: String = input.chars().filter(|c| !c.is_whitespace()).collect();
    base64::engine::general_purpose::STANDARD.decode(&clean).unwrap_or_else(|_| input.as_bytes().to_vec())
}

/// Decode quoted-printable content to raw bytes
fn decode_quoted_printable_to_bytes(input: &str) -> Vec<u8> {
    let mut result = Vec::new();
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '=' {
            if chars.peek().map_or(false, |nc| *nc == '\r' || *nc == '\n') {
                if chars.peek() == Some(&'\r') { chars.next(); }
                if chars.peek() == Some(&'\n') { chars.next(); }
                continue;
            }
            let hex1 = chars.next();
            let hex2 = chars.next();
            if let (Some(h1), Some(h2)) = (hex1, hex2) {
                if let (Some(d1), Some(d2)) = (h1.to_digit(16), h2.to_digit(16)) {
                    result.push((d1 << 4 | d2) as u8);
                    continue;
                }
            }
            result.push(b'=');
            if let Some(h1) = hex1 { result.push(h1 as u8); }
            if let Some(h2) = hex2 { result.push(h2 as u8); }
        } else {
            result.push(c as u8);
        }
    }
    result
}

/// Extract charset from Content-Type header (e.g. "text/html; charset=iso-8859-1")
fn extract_charset(headers: &str) -> String {
    let lower = headers.to_lowercase();
    if let Some(pos) = lower.find("charset=") {
        let after = &headers[pos + 8..];
        let after = after.trim_start_matches(&['"', '\''] as &[_]);
        after.chars()
            .take_while(|c| !c.is_whitespace() && *c != ';' && *c != '"' && *c != '\'' && *c != '\r' && *c != '\n')
            .collect()
    } else {
        String::from("utf-8")
    }
}

/// Decode bytes to UTF-8 string using the given charset label
fn charset_decode(bytes: &[u8], charset: &str) -> String {
    let label = charset.to_lowercase();
    if label == "utf-8" || label == "utf8" {
        return String::from_utf8_lossy(bytes).to_string();
    }
    if label == "iso-8859-1" || label == "latin1" || label == "iso8859-1" {
        return bytes.iter().map(|&b| b as char).collect();
    }
    if label == "iso-8859-15" || label == "latin9" {
        let mut s = String::with_capacity(bytes.len());
        for &b in bytes {
            s.push(match b {
                0xA4 => '\u{20AC}', // €
                0xA6 => '\u{0160}', // Š
                0xA8 => '\u{0161}', // š
                0xB4 => '\u{017D}', // Ž
                0xB8 => '\u{017E}', // ž
                0xBC => '\u{0152}', // Œ
                0xBD => '\u{0153}', // œ
                0xBE => '\u{0178}', // Ÿ
                _ => b as char,
            });
        }
        return s;
    }
    if label == "windows-1252" || label == "cp1252" || label == "windows1252" {
        let mut s = String::with_capacity(bytes.len());
        for &b in bytes {
            s.push(match b {
                0x80 => '\u{20AC}', 0x82 => '\u{201A}', 0x83 => '\u{0192}',
                0x84 => '\u{201E}', 0x85 => '\u{2026}', 0x86 => '\u{2020}',
                0x87 => '\u{2021}', 0x88 => '\u{02C6}', 0x89 => '\u{2030}',
                0x8A => '\u{0160}', 0x8B => '\u{2039}', 0x8C => '\u{0152}',
                0x8E => '\u{017D}', 0x91 => '\u{2018}', 0x92 => '\u{2019}',
                0x93 => '\u{201C}', 0x94 => '\u{201D}', 0x95 => '\u{2022}',
                0x96 => '\u{2013}', 0x97 => '\u{2014}', 0x98 => '\u{02DC}',
                0x99 => '\u{2122}', 0x9A => '\u{0161}', 0x9B => '\u{203A}',
                0x9C => '\u{0153}', 0x9E => '\u{017E}', 0x9F => '\u{0178}',
                _ => b as char,
            });
        }
        return s;
    }
    String::from_utf8_lossy(bytes).to_string()
}

/// Decode base64 content (kept for backward compat)
fn decode_base64(input: &str) -> String {
    let bytes = decode_base64_to_bytes(input);
    String::from_utf8_lossy(&bytes).to_string()
}

/// Decode quoted-printable content
fn decode_quoted_printable(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '=' {
            // Soft line break
            if chars.peek().map_or(false, |nc| *nc == '\r' || *nc == '\n') {
                // Skip \r\n or \n
                if chars.peek() == Some(&'\r') {
                    chars.next();
                }
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                continue;
            }
            // Encoded byte =XX
            let hex1 = chars.next();
            let hex2 = chars.next();
            if let (Some(h1), Some(h2)) = (hex1, hex2) {
                if let (Some(d1), Some(d2)) = (h1.to_digit(16), h2.to_digit(16)) {
                    let byte = (d1 << 4 | d2) as u8;
                    result.push(byte as char);
                    continue;
                }
            }
            // If hex parsing failed, just append the characters
            result.push('=');
            if let Some(h1) = hex1 { result.push(h1); }
            if let Some(h2) = hex2 { result.push(h2); }
        } else {
            result.push(c);
        }
    }
    result
}

/// Fetch the full body of a single email from IMAP
fn imap_fetch_body(
    account: &AccountConfig,
    folder: &str,
    uid: u32,
    _data_dir: &std::path::PathBuf,
) -> Result<ParsedBody, String> {
    eprintln!("imap_fetch_body: fetching UID {} from {}/{}", uid, account.email, folder);
    
    let tls = native_tls::TlsConnector::new()
        .map_err(|e| format!("TLS init failed: {}", e))?;
    
    let client = imap::connect(
        (account.imap_host.as_str(), account.imap_port),
        account.imap_host.as_str(),
        &tls,
    ).map_err(|e| format!("IMAP connect failed: {}", e))?;
    
    let mut session = client
        .login(&account.username, &account.password)
        .map_err(|e| format!("IMAP login failed: {}", e.0))?;
    
    session.select(folder)
        .map_err(|e| format!("IMAP SELECT failed: {}", e))?;
    
    // Fetch full RFC822 message using raw command
    let command = format!("UID FETCH {} (RFC822)", uid);
    let raw_response = session.run_command_and_read_response(&command)
        .map_err(|e| format!("IMAP FETCH failed: {}", e))?;
    
    let _ = session.logout();
    
    // Parse the full email
    let response_str = String::from_utf8_lossy(&raw_response);
    let parsed = parse_email_body(&response_str);
    
    Ok(parsed)
}

/// Sanitize HTML for e-ink rendering.
/// Qt's Text.RichText does NOT support <style> blocks — they render as visible text.
/// Instead we strip them and inject inline styles + attribute cleanup.
fn sanitize_html(html: &str) -> String {
    let mut result = html.to_string();
    
    let script_re = regex::Regex::new(r"(?is)<script[^>]*>.*?</script>").unwrap();
    result = script_re.replace_all(&result, "").to_string();
    let iframe_re = regex::Regex::new(r"(?is)<iframe[^>]*>.*?</iframe>").unwrap();
    result = iframe_re.replace_all(&result, "").to_string();
    let object_re = regex::Regex::new(r"(?is)<object[^>]*>.*?</object>").unwrap();
    result = object_re.replace_all(&result, "").to_string();
    let embed_re = regex::Regex::new(r"(?is)<embed[^>]*>").unwrap();
    result = embed_re.replace_all(&result, "").to_string();
    let applet_re = regex::Regex::new(r"(?is)<applet[^>]*>.*?</applet>").unwrap();
    result = applet_re.replace_all(&result, "").to_string();
    
    let style_block_re = regex::Regex::new(r"(?is)<style[^>]*>.*?</style>").unwrap();
    result = style_block_re.replace_all(&result, "").to_string();
    let link_css_re = regex::Regex::new(r#"(?i)<link[^>]*stylesheet[^>]*>"#).unwrap();
    result = link_css_re.replace_all(&result, "").to_string();
    
    let style_attr_re = regex::Regex::new(r#"(?i)\sstyle\s*=\s*"([^"]*)"|\sstyle\s*=\s*'([^']*)'"#).unwrap();
    result = style_attr_re.replace_all(&result, |caps: &regex::Captures| {
        let val = caps.get(1).or_else(|| caps.get(2)).map(|m| m.as_str()).unwrap_or("");
        let fixed = fix_inline_style(val);
        format!(r#" style="{}""#, fixed)
    }).to_string();

    result
}

fn fix_inline_style(style: &str) -> String {
    let lower = style.to_lowercase();
    if lower.contains("white") || lower.contains("#fff") || lower.contains("#ffffff") {
        let color_re = regex::Regex::new(r#"(?i)color\s*:\s*[^;]+"#).unwrap();
        color_re.replace_all(style, "color:#222").to_string()
    } else {
        style.to_string()
    }
}