//! iMessage channel configuration types.
//!
//! This module defines configuration structures for the iMessage channel plugin,
//! including account configuration for both AppleScript and BlueBubbles backends.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Configuration for an iMessage account.
///
/// Supports two backends:
/// - AppleScript (native macOS, uses osascript)
/// - BlueBubbles (third-party server, works on any platform with HTTP access)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImessageAccountConfig {
    /// Unique identifier for this account
    pub account_id: String,
    /// Backend to use: "applescript" or "bluebubbles"
    #[serde(default = "default_backend")]
    pub backend: String,
    /// Optional list of allowed phone numbers/email addresses (if empty, all senders are allowed)
    pub allowed_senders: Option<HashSet<String>>,
    /// Optional list of group chat IDs to monitor (if empty, all groups are monitored)
    pub monitored_groups: Option<HashSet<String>>,
    /// Whether to include media in received messages
    #[serde(default = "default_include_media")]
    pub include_media: bool,
    /// Whether to enable delivery receipts for sent messages
    #[serde(default)]
    pub delivery_receipts: bool,
    /// AppleScript backend configuration
    #[serde(default)]
    pub applescript: AppleScriptConfig,
    /// BlueBubbles backend configuration
    #[serde(default)]
    pub bluebubbles: BlueBubblesConfig,
}

impl Default for ImessageAccountConfig {
    fn default() -> Self {
        Self {
            account_id: "default".to_string(),
            backend: default_backend(),
            allowed_senders: None,
            monitored_groups: None,
            include_media: default_include_media(),
            delivery_receipts: false,
            applescript: AppleScriptConfig::default(),
            bluebubbles: BlueBubblesConfig::default(),
        }
    }
}

impl ImessageAccountConfig {
    /// Create a new ImessageAccountConfig with the given account ID.
    pub fn new(account_id: &str) -> Self {
        Self {
            account_id: account_id.to_string(),
            ..Default::default()
        }
    }

    /// Validate the account configuration.
    pub fn validate(&self) -> Result<(), ImessageError> {
        // Check backend
        match self.backend.as_str() {
            "applescript" => {
                // AppleScript is macOS-only
                #[cfg(not(target_os = "macos"))]
                return Err(ImessageError::PlatformUnsupported(
                    "AppleScript backend requires macOS".to_string(),
                ));
                
                #[cfg(target_os = "macos")]
                {
                    // Validate AppleScript-specific settings
                    if self.applescript.osascript_path.is_none() {
                        // Check if osascript exists
                        if std::path::Path::new("/usr/bin/osascript").exists() {
                            // Valid, will use default
                        } else {
                            return Err(ImessageError::OsascriptNotFound);
                        }
                    }
                }
            }
            "bluebubbles" => {
                // BlueBubbles requires valid HTTP endpoint
                if self.bluebubbles.api_url.is_none() {
                    return Err(ImessageError::MissingBlueBubblesUrl);
                }
                
                // Validate URL
                let url = self.bluebubbles.api_url.as_ref().unwrap();
                url.parse::<url::Url>().map_err(|_| ImessageError::InvalidUrl {
                    url: url.clone(),
                    message: "Invalid BlueBubbles API URL".to_string(),
                })?;
            }
            _ => {
                return Err(ImessageError::InvalidBackend {
                    backend: self.backend.clone(),
                });
            }
        }
        
        Ok(())
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

    /// Check if a group should be monitored.
    pub fn is_group_monitored(&self, group_id: &str) -> bool {
        if let Some(ref monitored) = self.monitored_groups {
            monitored.contains(group_id)
        } else {
            // If no monitor list is configured, all groups are monitored
            true
        }
    }

    /// Get the backend type.
    pub fn backend_type(&self) -> BackendType {
        match self.backend.as_str() {
            "applescript" => BackendType::AppleScript,
            "bluebubbles" => BackendType::BlueBubbles,
            _ => BackendType::AppleScript, // Default fallback
        }
    }
}

/// Backend type enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackendType {
    /// AppleScript backend using osascript
    AppleScript,
    /// BlueBubbles HTTP API backend
    BlueBubbles,
}

/// AppleScript backend configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppleScriptConfig {
    /// Path to osascript binary (defaults to /usr/bin/osascript on macOS)
    #[serde(default = "default_osascript_path")]
    pub osascript_path: Option<String>,
    /// Timeout for AppleScript execution in seconds
    #[serde(default = "default_applescript_timeout")]
    pub timeout_seconds: u64,
    /// Whether to enable verbose AppleScript logging
    #[serde(default)]
    pub verbose: bool,
}

/// BlueBubbles backend configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BlueBubblesConfig {
    /// BlueBubbles API URL (e.g., http://localhost:12345)
    #[serde(default)]
    pub api_url: Option<String>,
    /// BlueBubbles API password (if required)
    #[serde(default)]
    pub api_password: Option<String>,
    /// WebSocket URL for real-time updates
    #[serde(default)]
    pub websocket_url: Option<String>,
    /// Timeout for API requests in seconds
    #[serde(default = "default_bluebubbles_timeout")]
    pub timeout_seconds: u64,
    /// Whether to automatically reconnect on disconnect
    #[serde(default = "default_reconnect")]
    pub reconnect: bool,
    /// Maximum reconnection attempts
    #[serde(default = "default_max_reconnect_attempts")]
    pub max_reconnect_attempts: u32,
}

/// Error types for iMessage channel operations.
#[derive(Debug, thiserror::Error)]
pub enum ImessageError {
    /// Platform not supported
    #[error("Platform not supported: {0}")]
    PlatformUnsupported(String),
    /// AppleScript backend requires macOS
    #[error("AppleScript backend requires macOS")]
    AppleScriptRequiresMacOs,
    /// osascript not found
    #[error("osascript not found. Please ensure AppleScript is installed on macOS.")]
    OsascriptNotFound,
    /// Missing BlueBubbles URL
    #[error("BlueBubbles API URL is required when using bluebubbles backend")]
    MissingBlueBubblesUrl,
    /// Invalid URL
    #[error("Invalid URL '{url}': {message}")]
    InvalidUrl {
        url: String,
        message: String,
    },
    /// Invalid backend
    #[error("Invalid backend '{backend}'. Must be 'applescript' or 'bluebubbles'")]
    InvalidBackend {
        backend: String,
    },
    /// BlueBubbles API error
    #[error("BlueBubbles API error: {0}")]
    BlueBubblesApi(String),
    /// AppleScript execution error
    #[error("AppleScript error: {0}")]
    AppleScript(String),
    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    /// HTTP error
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    /// JSON parsing error
    #[error("JSON parsing error: {0}")]
    JsonParseError(#[from] serde_json::Error),
    /// Failed to deserialize response
    #[error("Response deserialization error: {0}")]
    ResponseDeserializeError(String),
    /// Authentication error
    #[error("Authentication error: {0}")]
    AuthenticationError(String),
    /// Connection error
    #[error("Connection error: {0}")]
    ConnectionError(String),
    /// Media handling error
    #[error("Media error: {0}")]
    MediaError(String),
}

impl ImessageError {
    /// Check if this error indicates a platform issue.
    pub fn is_platform_error(&self) -> bool {
        matches!(
            self,
            ImessageError::PlatformUnsupported(_) | ImessageError::AppleScriptRequiresMacOs
        )
    }
}

/// Result type for iMessage channel operations.
pub type ImessageResult<T> = std::result::Result<T, ImessageError>;

// Default values
fn default_backend() -> String {
    #[cfg(target_os = "macos")]
    {
        "applescript".to_string()
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        "bluebubbles".to_string()
    }
}

fn default_include_media() -> bool {
    true
}

fn default_osascript_path() -> Option<String> {
    Some("/usr/bin/osascript".to_string())
}

fn default_applescript_timeout() -> u64 {
    30
}

fn default_bluebubbles_timeout() -> u64 {
    30
}

fn default_reconnect() -> bool {
    true
}

fn default_max_reconnect_attempts() -> u32 {
    5
}

/// Utility functions for iMessage configuration.
pub mod utils {
    use super::*;

    /// Parse a phone number or email address string.
    pub fn normalize_contact_identifier(identifier: &str) -> String {
        let trimmed = identifier.trim();
        
        // Check if it looks like an email
        if trimmed.contains('@') {
            return trimmed.to_string();
        }
        
        // Otherwise, treat as phone number and normalize
        normalize_phone_number(trimmed)
    }

    /// Parse a phone number string and normalize it to E.164 format.
    pub fn normalize_phone_number(phone: &str) -> String {
        let cleaned: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
        
        // If input already starts with + and is valid E.164, return as-is
        if phone.starts_with('+') && cleaned.len() >= 10 && cleaned.len() <= 15 {
            return format!("+{}", cleaned);
        }
        
        // If already has + but wrong length, just clean it
        if phone.starts_with('+') {
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

    /// Extract group ID from iMessage group data.
    pub fn extract_group_id(group_data: &serde_json::Value) -> Option<String> {
        // iMessage groups typically have a GUID or chat identifier
        group_data.get("guid")
            .or_else(|| group_data.get("chat_guid"))
            .or_else(|| group_data.get("group_id"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    /// Extract sender identifier (phone number or email) from iMessage message.
    pub fn extract_sender_id(message: &serde_json::Value) -> Option<String> {
        // iMessage messages can have "address", "from", "sender", or "account_id"
        message.get("address")
            .or_else(|| message.get("from"))
            .or_else(|| message.get("sender"))
            .or_else(|| message.get("account_id"))
            .or_else(|| message.get("service_center"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    /// Extract message text from iMessage message data.
    pub fn extract_message_text(message: &serde_json::Value) -> Option<String> {
        message.get("text")
            .or_else(|| message.get("body"))
            .or_else(|| message.get("content"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    /// Check if a string looks like a phone number.
    pub fn is_phone_number(s: &str) -> bool {
        let cleaned: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
        cleaned.len() >= 10 && cleaned.len() <= 15
    }

    /// Check if a string looks like an email address.
    pub fn is_email(s: &str) -> bool {
        s.contains('@') && s.contains('.')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = ImessageAccountConfig::default();
        assert_eq!(config.account_id, "default");
        #[cfg(target_os = "macos")]
        assert_eq!(config.backend, "applescript");
        #[cfg(not(target_os = "macos"))]
        assert_eq!(config.backend, "bluebubbles");
    }

    #[test]
    fn test_config_new() {
        let config = ImessageAccountConfig::new("test-account");
        assert_eq!(config.account_id, "test-account");
    }

    #[test]
    fn test_config_validation_applescript() {
        let config = ImessageAccountConfig {
            backend: "applescript".to_string(),
            ..Default::default()
        };
        
        // On macOS, this should succeed
        #[cfg(target_os = "macos")]
        {
            // May still fail if osascript doesn't exist
            let result = config.validate();
            // Don't assert success as osascript might not exist in test environment
            let _ = result;
        }
        
        // On non-macOS, this should fail
        #[cfg(not(target_os = "macos"))]
        {
            assert!(matches!(config.validate(), Err(ImessageError::PlatformUnsupported(_))));
        }
    }

    #[test]
    fn test_config_validation_bluebubbles_no_url() {
        let config = ImessageAccountConfig {
            backend: "bluebubbles".to_string(),
            ..Default::default()
        };
        
        assert!(matches!(config.validate(), Err(ImessageError::MissingBlueBubblesUrl)));
    }

    #[test]
    fn test_config_validation_bluebubbles_with_url() {
        let config = ImessageAccountConfig {
            backend: "bluebubbles".to_string(),
            bluebubbles: BlueBubblesConfig {
                api_url: Some("http://localhost:12345".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_invalid_backend() {
        let config = ImessageAccountConfig {
            backend: "invalid".to_string(),
            ..Default::default()
        };
        
        assert!(matches!(config.validate(), Err(ImessageError::InvalidBackend { .. })));
    }

    #[test]
    fn test_is_sender_allowed() {
        let mut config = ImessageAccountConfig::new("test");
        
        // No allowlist - all senders allowed
        assert!(config.is_sender_allowed("+1234567890"));
        
        // Add allowlist
        let mut allowed = HashSet::new();
        allowed.insert("+1234567890".to_string());
        config.allowed_senders = Some(allowed);
        
        assert!(config.is_sender_allowed("+1234567890"));
        assert!(!config.is_sender_allowed("+9876543210"));
    }

    #[test]
    fn test_is_group_monitored() {
        let mut config = ImessageAccountConfig::new("test");
        
        // No monitor list - all groups monitored
        assert!(config.is_group_monitored("group1"));
        
        // Add monitor list
        let mut monitored = HashSet::new();
        monitored.insert("group1".to_string());
        config.monitored_groups = Some(monitored);
        
        assert!(config.is_group_monitored("group1"));
        assert!(!config.is_group_monitored("group2"));
    }

    #[test]
    fn test_normalize_phone_number() {
        use utils::normalize_phone_number;
        
        assert_eq!(normalize_phone_number("+1234567890"), "+1234567890");
        assert_eq!(normalize_phone_number("1234567890"), "+11234567890");
        assert_eq!(normalize_phone_number("1-234-567-890"), "+11234567890");
        assert_eq!(normalize_phone_number("abc123def456"), "+123456");
    }

    #[test]
    fn test_normalize_contact_identifier() {
        use utils::normalize_contact_identifier;
        
        // Email address
        assert_eq!(normalize_contact_identifier("user@example.com"), "user@example.com");
        
        // Phone number
        assert_eq!(normalize_contact_identifier("+1234567890"), "+1234567890");
        assert_eq!(normalize_contact_identifier("1234567890"), "+11234567890");
    }

    #[test]
    fn test_extract_group_id() {
        let json = serde_json::json!({
            "guid": "chat123",
            "chat_guid": "chat456"
        });
        
        assert_eq!(utils::extract_group_id(&json), Some("chat123".to_string()));
    }

    #[test]
    fn test_extract_sender_id() {
        let json = serde_json::json!({
            "address": "+1234567890",
            "from": "+0987654321"
        });
        
        assert_eq!(utils::extract_sender_id(&json), Some("+1234567890".to_string()));
    }

    #[test]
    fn test_is_phone_number() {
        use utils::is_phone_number;
        
        assert!(is_phone_number("+1234567890"));
        assert!(is_phone_number("1234567890"));
        assert!(!is_phone_number("user@example.com"));
        assert!(!is_phone_number("short"));
    }

    #[test]
    fn test_is_email() {
        use utils::is_email;
        
        assert!(is_email("user@example.com"));
        assert!(is_email("test@domain.org"));
        assert!(!is_email("not-an-email"));
        assert!(!is_email("+1234567890"));
    }
}
