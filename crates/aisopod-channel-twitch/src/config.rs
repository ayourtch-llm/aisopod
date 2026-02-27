//! Twitch channel configuration types.
//!
//! This module defines configuration structures for the Twitch channel plugin,
//! including connection settings and authentication options.

use serde::{Deserialize, Serialize};

/// Configuration for the Twitch channel.
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct TwitchConfig {
    /// Bot username on Twitch
    pub username: String,
    /// OAuth token (e.g., "oauth:abc123...")
    pub oauth_token: String,
    /// Channels to join (e.g., ["#channel1", "#channel2"])
    pub channels: Vec<String>,
    /// Enable whisper support (requires verified bot)
    #[serde(default)]
    pub enable_whispers: bool,
    /// Client ID for Twitch API calls
    pub client_id: Option<String>,
}

impl TwitchConfig {
    /// Validate the configuration.
    pub fn validate(&self) -> Result<(), anyhow::Error> {
        if self.username.is_empty() {
            return Err(anyhow::anyhow!("Twitch username cannot be empty"));
        }
        if self.oauth_token.is_empty() {
            return Err(anyhow::anyhow!("Twitch OAuth token cannot be empty"));
        }
        if self.channels.is_empty() {
            return Err(anyhow::anyhow!("At least one channel must be configured"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_twitch_config_validation() {
        let config = TwitchConfig {
            username: "testbot".to_string(),
            oauth_token: "oauth:abc123".to_string(),
            channels: vec!["#test".to_string()],
            enable_whispers: false,
            client_id: None,
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_twitch_config_validation_empty_username() {
        let config = TwitchConfig {
            username: "".to_string(),
            oauth_token: "oauth:abc123".to_string(),
            channels: vec!["#test".to_string()],
            enable_whispers: false,
            client_id: None,
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_twitch_config_validation_empty_oauth() {
        let config = TwitchConfig {
            username: "testbot".to_string(),
            oauth_token: "".to_string(),
            channels: vec!["#test".to_string()],
            enable_whispers: false,
            client_id: None,
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_twitch_config_validation_no_channels() {
        let config = TwitchConfig {
            username: "testbot".to_string(),
            oauth_token: "oauth:abc123".to_string(),
            channels: vec![],
            enable_whispers: false,
            client_id: None,
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_twitch_config_serialization() {
        let config = TwitchConfig {
            username: "testbot".to_string(),
            oauth_token: "oauth:abc123".to_string(),
            channels: vec!["#test".to_string(), "#another".to_string()],
            enable_whispers: true,
            client_id: Some("client123".to_string()),
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("testbot"));
        assert!(json.contains("oauth:abc123"));
        assert!(json.contains("#test"));
        assert!(json.contains("#another"));
        assert!(json.contains("true"));
        assert!(json.contains("client123"));

        let deserialized: TwitchConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.username, config.username);
        assert_eq!(deserialized.oauth_token, config.oauth_token);
        assert_eq!(deserialized.channels.len(), config.channels.len());
        assert!(deserialized.enable_whispers);
        assert_eq!(deserialized.client_id, config.client_id);
    }
}
