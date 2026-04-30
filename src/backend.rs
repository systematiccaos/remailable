//! remailable backend — headless process for AppLoad
//!
//! Runs as a backend service, communicating with the QML frontend
//! via the AppLoad SEQPACKET socket protocol.
//! No Qt/QML dependency — pure Rust.

pub mod appload;
pub mod account;
pub mod storage;

use std::sync::{Arc, Mutex};

use account::AccountConfig;
use serde_json::{json};

use appload::{AppLoadClient, MSG_REQUEST, SYS_NEW_FRONTEND, SYS_TERMINATE};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let socket_path = args.get(1).expect("Usage: remailable-backend <socket_path>");

    // Initialize storage
    let db_path = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("/home/root"))
        .join("remailable")
        .join("remailable.db");
    std::fs::create_dir_all(db_path.parent().unwrap()).ok();
    let db = storage::Storage::open(db_path.to_str().unwrap_or(":memory:"))
        .expect("Failed to open database");
    let storage = Arc::new(Mutex::new(db));

    // Connect to AppLoad socket
    let client = AppLoadClient::connect(socket_path)
        .expect("Failed to connect to AppLoad socket");
    let client = Arc::new(Mutex::new(client));

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
                    "current_view": if accounts.is_empty() { "account_list" } else { "folder_list" },
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
                    handle_request(&client, &storage, msg);
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
    msg: appload::AppLoadMessage,
) {
    let id = msg.id.unwrap_or(0);
    let action = msg.action.as_deref().unwrap_or("");

    match action {
        "get_accounts" => {
            let accounts = storage.lock().unwrap().list_accounts().unwrap_or_default();
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