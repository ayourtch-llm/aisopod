//! Signal channel configuration types.
//!
//! This module defines configuration structures for the Signal channel plugin,
//! including account configuration and signal-cli daemon settings.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Configuration for a Signal account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalAccountConfig {
    /// The phone number for this Signal account in E.164 format (e.g., +1234567890)
    pub phone_number: String,
    /// Optional device name for identification
    pub device_name: Option<String>,
    /// Optional list of allowed phone numbers (if empty, all senders are allowed)
    pub allowed_senders: Option<HashSet<String>>,
    /// Optional list of group IDs to monitor (if empty, all groups are monitored)
    pub monitored_groups: Option<HashSet<String>>,
    /// Whether disappearing messages are enabled for this account
    #[serde(default = "default_disappearing_enabled")]
    pub disappearing_enabled: bool,
    /// Disappearing message timer duration in seconds (default: 2592000 = 30 days)
    #[serde(default = "default_disappearing_timer")]
    pub disappearing_timer: u64,
    /// Whether to include media in received messages
    #[serde(default = "default_include_media")]
    pub include_media: bool,
    /// Path to signal-cli binary (defaults to "signal-cli")
    pub signal_cli_path: Option<String>,
    /// Path to signal-cli data directory
    pub signal_cli_data_dir: Option<String>,
}

impl Default for SignalAccountConfig {
    fn default() -> Self {
        Self {
            phone_number: String::new(),
            device_name: None,
            allowed_senders: None,
            monitored_groups: None,
            disappearing_enabled: default_disappearing_enabled(),
            disappearing_timer: default_disappearing_timer(),
            include_media: default_include_media(),
            signal_cli_path: None,
            signal_cli_data_dir: None,
        }
    }
}

impl SignalAccountConfig {
    /// Create a new SignalAccountConfig with the given phone number.
    pub fn new(phone_number: String) -> Self {
        Self {
            phone_number,
            ..Default::default()
        }
    }

    /// Validate the phone number format.
    pub fn validate_phone_number(&self) -> std::result::Result<(), SignalError> {
        // Signal phone numbers should be in E.164 format
        let re = regex::Regex::new(r"^\+[1-9]\d{1,14}$")
            .map_err(|e| SignalError::InvalidPhoneNumber {
                phone: self.phone_number.clone(),
                message: format!("Invalid regex: {}", e),
            })?;

        if re.is_match(&self.phone_number) {
            Ok(())
        } else {
            Err(SignalError::InvalidPhoneNumber {
                phone: self.phone_number.clone(),
                message: "Phone number must be in E.164 format (e.g., +1234567890)".to_string(),
            })
        }
    }

    /// Check if a sender is allowed based on the allowed_senders list.
    pub fn is_sender_allowed(&self, sender: &str) -> bool {
        if let Some(ref allowed) = self.allowed_senders {
            allowed.contains(sender)
        } else {
            // If no allowlist is configured, all senders are allowed
            true
        }
    }
}

/// Configuration for the signal-cli daemon.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalDaemonConfig {
    /// Path to signal-cli binary
    pub signal_cli_path: String,
    /// Path to signal-cli data directory
    pub signal_cli_data_dir: Option<String>,
    /// JSON-RPC server port (default: 8080)
    pub json_rpc_port: u16,
    /// Timeout for daemon operations in seconds
    pub operation_timeout_seconds: u64,
    /// Maximum number of connection retries
    pub max_retries: u32,
    /// Delay between retries in milliseconds
    pub retry_delay_ms: u64,
}

impl Default for SignalDaemonConfig {
    fn default() -> Self {
        Self {
            signal_cli_path: "signal-cli".to_string(),
            signal_cli_data_dir: None,
            json_rpc_port: 8080,
            operation_timeout_seconds: 30,
            max_retries: 3,
            retry_delay_ms: 1000,
        }
    }
}

/// Error types for Signal channel operations.
#[derive(Debug, thiserror::Error)]
pub enum SignalError {
    /// Invalid phone number format
    #[error("Invalid phone number '{phone}': {message}")]
    InvalidPhoneNumber {
        phone: String,
        message: String,
    },
    /// Signal-cli daemon not found or not executable
    #[error("Signal-cli not found: {0}")]
    SignalCliNotFound(String),
    /// Signal-cli daemon connection failed
    #[error("Signal-cli daemon connection failed: {0}")]
    DaemonConnectionFailed(String),
    /// Failed to spawn signal-cli process
    #[error("Failed to spawn signal-cli: {0}")]
    SpawnFailed(String),
    /// Failed to send command to signal-cli
    #[error("Failed to send command: {0}")]
    SendCommandFailed(String),
    /// Failed to receive response from signal-cli
    #[error("Failed to receive response: {0}")]
    ReceiveResponseFailed(String),
    /// JSON parsing error
    #[error("JSON parsing error: {0}")]
    JsonParseError(#[from] serde_json::Error),
    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    /// Failed to deserialize JSON-RPC response
    #[error("JSON-RPC response error: {0}")]
    JsonRpcError(String),
    /// Media handling error
    #[error("Media error: {0}")]
    MediaError(String),
    /// Failed to decode base64 data
    #[error("Base64 decode error: {0}")]
    Base64DecodeError(#[from] base64::DecodeError),
}

/// Result type for Signal channel operations.
pub type Result<T> = std::result::Result<T, SignalError>;

// Default values
fn default_disappearing_enabled() -> bool {
    false
}

fn default_disappearing_timer() -> u64 {
    2592000 // 30 days in seconds
}

fn default_include_media() -> bool {
    true
}

/// Utility functions for Signal channel configuration.
pub mod utils {
    use super::*;

    /// Parse a phone number string and normalize it to E.164 format.
    pub fn normalize_phone_number(phone: &str) -> String {
        let cleaned: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
        
        // If input already starts with + and is valid E.164, return as-is
        if phone.starts_with('+') && cleaned.len() >= 10 && cleaned.len() <= 15 {
            return format!("+{}", cleaned);
        }
        
        if cleaned.starts_with('1') && cleaned.len() == 11 {
            // US number with country code
            format!("+{}", cleaned)
        } else if cleaned.len() >= 10 {
            // Assume US number if 10-11 digits
            format!("+1{}", cleaned)
        } else {
            // Already in correct format or unknown
            format!("+{}", cleaned)
        }
    }

    /// Extract group ID from a Signal group object.
    pub fn extract_group_id(group_data: &serde_json::Value) -> Option<String> {
        // Signal groups typically have an "id" or "groupId" field
        group_data.get("id")
            .or_else(|| group_data.get("groupId"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    /// Extract phone number from a Signal message sender.
    pub fn extract_sender_id(message: &serde_json::Value) -> Option<String> {
        // Signal messages typically have "source" or "sourceNumber" field
        message.get("source")
            .or_else(|| message.get("sourceNumber"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phone_number_validation_valid() {
        let config = SignalAccountConfig::new("+1234567890".to_string());
        assert!(config.validate_phone_number().is_ok());
    }

    #[test]
    fn test_phone_number_validation_invalid() {
        let config = SignalAccountConfig::new("1234567890".to_string());
        assert!(config.validate_phone_number().is_err());
    }

    #[test]
    fn test_phone_number_validation_invalid_format() {
        let config = SignalAccountConfig::new("abc123".to_string());
        assert!(config.validate_phone_number().is_err());
    }

    #[test]
    fn test_sender_allowed() {
        let mut allowed = HashSet::new();
        allowed.insert("+1234567890".to_string());
        
        let config = SignalAccountConfig {
            phone_number: "+0987654321".to_string(),
            allowed_senders: Some(allowed),
            ..Default::default()
        };

        assert!(config.is_sender_allowed("+1234567890"));
        assert!(!config.is_sender_allowed("+1111111111"));
    }

    #[test]
    fn test_sender_allowed_no_list() {
        let config = SignalAccountConfig::new("+1234567890".to_string());
        
        // When no allowed_senders is set, all senders should be allowed
        assert!(config.is_sender_allowed("+1111111111"));
        assert!(config.is_sender_allowed("+2222222222"));
    }

    #[test]
    fn test_normalize_phone_number() {
        assert_eq!(utils::normalize_phone_number("+1234567890"), "+1234567890");
        // For 10-digit numbers, it prepends +1
        assert_eq!(utils::normalize_phone_number("1234567890"), "+11234567890");
        assert_eq!(utils::normalize_phone_number("1-234-567-890"), "+11234567890");
    }
}
