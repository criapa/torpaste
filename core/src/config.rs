//! Application Configuration

use serde::{Deserialize, Serialize};

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Tor configuration
    pub tor: TorConfig,
    /// Pastebin configuration (Legacy/Optional)
    pub pastebin: PastebinConfig,
    /// Protocol configuration
    pub protocol: ProtocolConfig,
    /// Security settings
    pub security: SecurityConfig,
}

/// Tor-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorConfig {
    /// SOCKS5 proxy port
    pub socks_port: u16,
    /// Control port (not used with Arti usually, but kept for compatibility)
    pub control_port: u16,
    /// Use bundled Tor binary (false usually means using Arti library)
    pub use_bundled: bool,
    /// Data directory for Tor
    pub data_dir: String,
    /// Enable logging
    pub enable_logging: bool,
}

impl Default for TorConfig {
    fn default() -> Self {
        Self {
            socks_port: 9050,
            control_port: 9051,
            use_bundled: true,
            data_dir: "tor_data".to_string(),
            enable_logging: false,
        }
    }
}

/// Pastebin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PastebinConfig {
    /// API dev key
    pub api_key: String,
    /// Default expiration (N = minutes, H = hours, D = days, M = months, W = weeks)
    pub default_expiration: String,
    /// Make pastes private by default
    pub default_private: bool,
}

impl Default for PastebinConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            default_expiration: "10M".to_string(),
            default_private: true,
        }
    }
}

/// Protocol configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolConfig {
    /// Protocol version
    pub version: u16,
    /// Maximum message size in bytes
    pub max_message_size: usize,
    /// Connection timeout in seconds
    pub connection_timeout: u64,
    /// Keep-alive interval in seconds
    pub keepalive_interval: u64,
    /// Maximum retry attempts
    pub max_retries: u32,
    /// Fragment size for large messages
    pub fragment_size: usize,
}

impl Default for ProtocolConfig {
    fn default() -> Self {
        Self {
            version: 1,
            max_message_size: 10 * 1024 * 1024, // 10MB
            connection_timeout: 30,
            keepalive_interval: 60,
            max_retries: 3,
            fragment_size: 1024,
        }
    }
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable screenshots (should be false for privacy)
    pub allow_screenshots: bool,
    /// Show notifications content
    pub show_notification_content: bool,
    /// Require authentication on app open
    pub require_auth: bool,
    /// Clear clipboard after copy timeout (seconds, 0 = disabled)
    pub clipboard_timeout: u32,
    /// Enable debug logging in release
    pub debug_in_release: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            allow_screenshots: false,
            show_notification_content: false,
            require_auth: false,
            clipboard_timeout: 30,
            debug_in_release: false,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            tor: TorConfig::default(),
            pastebin: PastebinConfig::default(),
            protocol: ProtocolConfig::default(),
            security: SecurityConfig::default(),
        }
    }
}
