//! AppLoad IPC protocol for reMarkable Paper Pro
//!
//! Uses Unix SEQPACKET sockets via the `uds` crate for
//! communication between QML frontend (xochitl) and Rust backend.
//!
//! Protocol:
//!   - 8-byte header: u32 msg_type + u32 payload_length (little-endian)
//!   - Separate SEQPACKET message for payload (JSON bytes)
//!
//! Message types:
//!   - MSG_REQUEST  (1): Frontend → Backend request
//!   - MSG_RESPONSE (2): Backend → Frontend response
//!   - MSG_EVENT    (3): Backend → Frontend event
//!
//! System messages (msg_type >= 0xFFFFFFFE):
//!   - SYS_NEW_FRONTEND (0xFFFFFFFE): New frontend connected
//!   - SYS_TERMINATE   (0xFFFFFFFF): Backend should terminate

use std::io;
use uds::UnixSeqpacketConn;
use serde::{Deserialize, Serialize};

// Message types
pub const MSG_REQUEST: u32 = 1;
pub const MSG_RESPONSE: u32 = 2;
pub const MSG_EVENT: u32 = 3;

// System message types
pub const SYS_NEW_FRONTEND: u32 = 0xFFFFFFFE;
pub const SYS_TERMINATE: u32 = 0xFFFFFFFF;

const HEADER_SIZE: usize = 8;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppLoadMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// AppLoad IPC client using Unix SEQPACKET
pub struct AppLoadClient {
    sock: UnixSeqpacketConn,
}

impl AppLoadClient {
    /// Connect to the AppLoad socket (path passed as $1 to the backend)
    pub fn connect(socket_path: &str) -> io::Result<Self> {
        let sock = UnixSeqpacketConn::connect(socket_path)?;
        Ok(Self { sock })
    }

    /// Receive a message from the frontend.
    /// Returns (msg_type, Option<payload>) — system messages have no payload dict.
    pub fn recv(&self) -> io::Result<(u32, Option<AppLoadMessage>)> {
        let mut buf = [0u8; 65536];
        let n = self.sock.recv(&mut buf)?;

        if n < HEADER_SIZE {
            if n >= 4 {
                let msg_type = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
                if msg_type >= SYS_NEW_FRONTEND {
                    return Ok((msg_type, None));
                }
            }
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Short header"));
        }

        let msg_type = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
        let length = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);

        // System messages — consume payload if present in same message
        // SEQPACKET delivers entire datagrams, so header+payload may come together or separately
        if msg_type >= SYS_NEW_FRONTEND {
            return Ok((msg_type, None));
        }

        // App message — payload may follow header
        // In SEQPACKET, the header and payload are sent as separate messages
        // So we need to recv again for the payload
        if length > 0 {
            let mut payload_buf = vec![0u8; length as usize];
            let payload_n = self.sock.recv(&mut payload_buf)?;
            let payload_str = String::from_utf8_lossy(&payload_buf[..payload_n.min(length as usize)]);
            let msg: Option<AppLoadMessage> = serde_json::from_str(&payload_str).ok();
            Ok((msg_type, msg))
        } else {
            Ok((msg_type, None))
        }
    }

    /// Send a response to the frontend
    pub fn send_response(&self, id: u64, data: serde_json::Value) -> io::Result<()> {
        let payload = AppLoadMessage {
            id: Some(id),
            action: None,
            params: None,
            event: None,
            data: Some(data),
        };
        self.send(MSG_RESPONSE, &payload)
    }

    /// Send an event to the frontend
    pub fn send_event(&self, event: &str, data: serde_json::Value) -> io::Result<()> {
        let payload = AppLoadMessage {
            id: None,
            action: None,
            params: None,
            event: Some(event.to_string()),
            data: Some(data),
        };
        self.send(MSG_EVENT, &payload)
    }

    fn send(&self, msg_type: u32, payload: &AppLoadMessage) -> io::Result<()> {
        let body = serde_json::to_vec(payload).unwrap_or_default();
        let mut header = [0u8; HEADER_SIZE];
        header[0..4].copy_from_slice(&msg_type.to_le_bytes());
        header[4..8].copy_from_slice(&(body.len() as u32).to_le_bytes());

        // SEQPACKET preserves message boundaries — send header then payload as separate messages
        self.sock.send(&header)?;
        if !body.is_empty() {
            self.sock.send(&body)?;
        }
        Ok(())
    }
}