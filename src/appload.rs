//! AppLoad IPC protocol for reMarkable Paper Pro
//!
//! Uses Unix SEQPACKET sockets via the `uds` crate for
//! communication between QML frontend (xochitl) and Rust backend.
//!
//! Protocol (derived from movewriter's protocol.py):
//!   - Messages are sent as SEQPACKET datagrams
//!   - App messages: header (8 bytes: u32 msg_type + u32 length) then
//!     a separate SEQPACKET datagram with the payload (JSON bytes)
//!   - System messages (msg_type >= 0xFFFFFFFE):
//!     SYS_NEW_FRONTEND: 8-byte header only, may have 1-byte payload datagram
//!     SYS_TERMINATE: 8-byte header only
//!
//! Message types:
//!   - MSG_REQUEST  (1): Frontend → Backend request
//!   - MSG_RESPONSE (2): Backend → Frontend response  
//!   - MSG_EVENT    (3): Backend → Frontend event
//!
//! System message types:
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
    ///
    /// AppLoad protocol: each message consists of two SEQPACKET datagrams:
    ///   1. 8-byte header: u32 msg_type + u32 length (little-endian)
    ///   2. Payload bytes (length from header)
    ///
    /// System messages (msg_type >= 0xFFFFFFFE) have a 1-byte payload
    /// datagram that must be consumed (discarded).
    pub fn recv(&self) -> io::Result<(u32, Option<AppLoadMessage>)> {
        let mut buf = [0u8; 65536];

        // Read header SEQPACKET datagram
        // May need to skip stray short messages (e.g., leftover payload from system messages)
        loop {
            let n = self.sock.recv(&mut buf)?;

            if n == 0 {
                return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Connection closed"));
            }

            if n < HEADER_SIZE {
                // Short datagram — stray payload from a system message. Skip and read next.
                eprintln!("Skipping short datagram: {} bytes", n);
                continue;
            }

            // Got a proper header
            break;
        }

        let msg_type = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
        let length = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);

        // System messages — consume their payload datagram (typically 1 byte)
        if msg_type >= SYS_NEW_FRONTEND {
            if length > 0 {
                let mut discard = [0u8; 65536];
                let _ = self.sock.recv(&mut discard); // consume and discard payload
            }
            eprintln!("System message: type=0x{:08X} length={}", msg_type, length);
            return Ok((msg_type, None));
        }

        // App message — payload comes as a separate SEQPACKET datagram
        if length > 0 {
            let mut payload_buf = vec![0u8; length as usize];
            let payload_n = self.sock.recv(&mut payload_buf)?;
            let payload_str = String::from_utf8_lossy(&payload_buf[..payload_n.min(length as usize)]);
            eprintln!("App message: type={} length={} payload={}", msg_type, length, &payload_str[..payload_str.len().min(200)]);
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