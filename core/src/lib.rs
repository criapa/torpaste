//! TorChat-Paste Core Library
//! Anonymous P2P Messenger with Tor integration

pub mod tor_manager;
pub mod crypto;
pub mod protocol;
pub mod storage;
pub mod config;

pub use tor_manager::{TorManager, TorStatus};
pub use crypto::{Crypto, IdentityKeyPair as KeyPair, SessionKey, Fingerprint};
pub use protocol::{Message, MessageType, ChatProtocol};
pub use storage::{SecureStorage, StoredContact};
pub use config::AppConfig;

use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

/// Main application state
pub struct AppState {
    pub tor_manager: Arc<RwLock<TorManager>>,
    pub crypto: Arc<Crypto>,
    pub storage: Arc<SecureStorage>,
    pub contacts: Arc<RwLock<HashMap<String, Contact>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            tor_manager: Arc::new(RwLock::new(TorManager::new())),
            crypto: Arc::new(Crypto::new()),
            storage: Arc::new(SecureStorage::new()),
            contacts: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// Contact representation (only stores public address)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Contact {
    /// The .onion address of the contact
    pub address: String,
    /// Local nickname (not synced)
    pub nickname: String,
    /// Whether the contact is currently online
    pub online: bool,
    /// Last seen timestamp (session only)
    pub last_seen: Option<i64>,
}

impl Contact {
    pub fn new(address: String, nickname: String) -> Self {
        Self {
            address,
            nickname,
            online: false,
            last_seen: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contact_creation() {
        let contact = Contact::new(
            "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890.onion".to_string(),
            "Test User".to_string(),
        );
        assert!(!contact.online);
        assert_eq!(contact.nickname, "Test User");
    }
}
