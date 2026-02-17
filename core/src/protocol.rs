//! P2P Messaging Protocol

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use thiserror::Error;
//use log::{info};

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("Connection not established")]
    NotConnected,
    #[error("Handshake failed: {0}")]
    HandshakeFailed(String),
    #[error("Send failed: {0}")]
    SendFailed(String),
    #[error("Receive failed: {0}")]
    ReceiveFailed(String),
    #[error("Invalid message format")]
    InvalidFormat,
    #[error("Encryption error: {0}")]
    EncryptionError(String),
}

/// Message types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageType {
    /// Handshake - key exchange
    Handshake,
    /// Text message
    Text,
    /// File transfer
    File,
    /// Acknowledgment
    Ack,
    /// Keep-alive
    KeepAlive,
    /// Disconnect notification
    Disconnect,
}

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Message ID
    pub id: String,
    /// Message type
    pub msg_type: MessageType,
    /// Sender address
    pub sender: String,
    /// Timestamp (Unix epoch)
    pub timestamp: i64,
    /// Content (encrypted in transit)
    pub content: String,
    /// Sequence number for ordering
    pub sequence: u64,
    /// File metadata (for file transfers)
    pub file_metadata: Option<FileMetadata>,
}

/// File transfer metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    /// File name
    pub name: String,
    /// File size in bytes
    pub size: u64,
    /// MIME type
    pub mime_type: String,
}

/// Handshake message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeMessage {
    /// Protocol version
    pub version: u16,
    /// Public key
    pub public_key: String,
    /// Nonce for key exchange
    pub nonce: String,
}

/// Protocol frame for transmission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolFrame {
    /// Frame type
    pub frame_type: FrameType,
    /// Payload (JSON serialized message)
    pub payload: String,
    /// Message checksum
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FrameType {
    /// Single message
    Single,
    /// First fragment
    FirstFragment,
    /// Middle fragment
    MiddleFragment,
    /// Last fragment
    LastFragment,
}

/// Chat protocol handler
pub struct ChatProtocol {
    /// Protocol version
    version: u16,
    /// Max fragment size
    max_fragment_size: usize,
    /// Sequence counter
    sequence: u64,
    /// Pending fragments
    pending_fragments: VecDeque<ProtocolFrame>,
}

impl ChatProtocol {
    pub fn new() -> Self {
        Self {
            version: 1,
            max_fragment_size: 1024,
            sequence: 0,
            pending_fragments: VecDeque::new(),
        }
    }

    /// Get next sequence number
    pub fn next_sequence(&mut self) -> u64 {
        let seq = self.sequence;
        self.sequence += 1;
        seq
    }

    /// Create a handshake message
    pub fn create_handshake(&self, public_key: &str, nonce: &str) -> HandshakeMessage {
        HandshakeMessage {
            version: self.version,
            public_key: public_key.to_string(),
            nonce: nonce.to_string(),
        }
    }

    /// Serialize a message to bytes for transmission
    pub fn serialize_message(&self, msg: &Message) -> Result<String, ProtocolError> {
        serde_json::to_string(msg)
            .map_err(|_| ProtocolError::InvalidFormat)
    }

    /// Deserialize a message from bytes
    pub fn deserialize_message(&self, data: &str) -> Result<Message, ProtocolError> {
        serde_json::from_str(data)
            .map_err(|_| ProtocolError::InvalidFormat)
    }

    /// Serialize a handshake message
    pub fn serialize_handshake(&self, handshake: &HandshakeMessage) -> Result<String, ProtocolError> {
        serde_json::to_string(handshake)
            .map_err(|_| ProtocolError::InvalidFormat)
    }

    /// Deserialize a handshake message
    pub fn deserialize_handshake(&self, data: &str) -> Result<HandshakeMessage, ProtocolError> {
        serde_json::from_str(data)
            .map_err(|_| ProtocolError::InvalidFormat)
    }

    /// Create a protocol frame
    pub fn create_frame(&self, payload: String) -> ProtocolFrame {
        use sodiumoxide::crypto::hash::sha256;
        use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

        let checksum = BASE64.encode(sha256::hash(payload.as_bytes()).as_ref());

        ProtocolFrame {
            frame_type: FrameType::Single,
            payload,
            checksum,
        }
    }

    /// Serialize a frame for transmission
    pub fn serialize_frame(&self, frame: &ProtocolFrame) -> Result<String, ProtocolError> {
        serde_json::to_string(frame)
            .map_err(|_| ProtocolError::InvalidFormat)
    }

    /// Deserialize a frame from transmission
    pub fn deserialize_frame(&self, data: &str) -> Result<ProtocolFrame, ProtocolError> {
        serde_json::from_str(data)
            .map_err(|_| ProtocolError::InvalidFormat)
    }

    /// Create a text message
    pub fn create_text_message(
        &mut self,
        sender: &str,
        content: String,
    ) -> Message {
        let timestamp = chrono::Utc::now().timestamp();

        Message {
            id: Self::generate_message_id(),
            msg_type: MessageType::Text,
            sender: sender.to_string(),
            timestamp,
            content,
            sequence: self.next_sequence(),
            file_metadata: None,
        }
    }

    /// Create a file message
    pub fn create_file_message(
        &mut self,
        sender: &str,
        content: String,
        metadata: FileMetadata,
    ) -> Message {
        let timestamp = chrono::Utc::now().timestamp();

        Message {
            id: Self::generate_message_id(),
            msg_type: MessageType::File,
            sender: sender.to_string(),
            timestamp,
            content,
            sequence: self.next_sequence(),
            file_metadata: Some(metadata),
        }
    }

    /// Generate a unique message ID
    fn generate_message_id() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bytes: Vec<u8> = (0..16).map(|_| rng.gen()).collect();
        hex::encode(bytes)
    }
}

impl Default for ChatProtocol {
    fn default() -> Self {
        Self::new()
    }
}

/// Connection state
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    /// Not connected
    Disconnected,
    /// Performing handshake
    Handshake,
    /// Connected and encrypted
    Connected,
    /// Connection closing
    Closing,
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self::Disconnected
    }
}

/// P2P Connection
pub struct P2PConnection {
    /// Remote address
    pub remote_address: String,
    /// Connection state
    pub state: ConnectionState,
    /// Established session key (if any)
    pub session_key: Option<crate::crypto::SessionKey>,
    /// Messages pending acknowledgment
    pub pending_acks: Vec<String>,
    /// Last activity timestamp
    pub last_activity: i64,
}

impl P2PConnection {
    pub fn new(address: String) -> Self {
        Self {
            remote_address: address,
            state: ConnectionState::Disconnected,
            session_key: None,
            pending_acks: Vec::new(),
            last_activity: chrono::Utc::now().timestamp(),
        }
    }

    pub fn set_connected(&mut self, session_key: crate::crypto::SessionKey) {
        self.state = ConnectionState::Connected;
        self.session_key = Some(session_key);
    }

    pub fn is_connected(&self) -> bool {
        self.state == ConnectionState::Connected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let mut protocol = ChatProtocol::new();
        let msg = protocol.create_text_message(
            "test.onion",
            "Hello".to_string(),
        );
        assert_eq!(msg.msg_type, MessageType::Text);
        assert_eq!(msg.content, "Hello");
    }

    #[test]
    fn test_frame_serialization() {
        let protocol = ChatProtocol::new();
        let frame = protocol.create_frame("test payload".to_string());
        let serialized = protocol.serialize_frame(&frame).unwrap();
        let deserialized = protocol.deserialize_frame(&serialized).unwrap();
        assert_eq!(deserialized.payload, "test payload");
    }
}
