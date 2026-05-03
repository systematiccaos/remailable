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
                let email = storage.lock().unwrap().get_email(email_id).ok().flatten();
                let c = client.lock().unwrap();
                if let Some(e) = email {
                    let body = std::fs::read_to_string(&e.body_path).unwrap_or_else(|_| "(body not available)".to_string());
                    let _ = c.send_response(id, json!({
                        "id": e.id,
                        "subject": e.subject,
                        "from": e.from_addr,
                        "date": e.date,
                        "body": body,
                        "content_type": e.content_type,
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