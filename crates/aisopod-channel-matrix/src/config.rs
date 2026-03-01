//! Matrix channel configuration.
//!
//! This module defines the configuration types for the Matrix channel,
//! including homeserver settings, authentication methods, and encryption options.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for a Matrix account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatrixAccountConfig {
    /// Homeserver URL (e.g., "https://matrix.org" or "https://matrix.example.com")
    pub homeserver_url: String,
    /// Authentication method
    pub auth: MatrixAuth,
    /// Enable end-to-end encryption
    #[serde(default = "default_true")]
    pub enable_e2ee: bool,
    /// Rooms to join (e.g., ["!room:matrix.org", "#general:matrix.org"])
    #[serde(default)]
    pub rooms: Vec<String>,
    /// Path to store encryption keys and sync state
    #[serde(default)]
    pub state_store_path: Option<PathBuf>,
    /// Optional list of allowed user IDs
    #[serde(default)]
    pub allowed_users: Vec<String>,
    /// Whether the bot must be mentioned in group chats
    #[serde(default)]
    pub requires_mention_in_group: bool,
}

impl Default for MatrixAccountConfig {
    fn default() -> Self {
        Self {
            homeserver_url: "https://matrix.org".to_string(),
            auth: MatrixAuth::Password {
                username: String::new(),
                password: String::new(),
            },
            enable_e2ee: true,
            rooms: Vec::new(),
            state_store_path: None,
            allowed_users: Vec::new(),
            requires_mention_in_group: false,
        }
    }
}

/// Authentication method for Matrix.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MatrixAuth {
    /// Username and password authentication
    #[serde(rename = "password")]
    Password { username: String, password: String },
    /// Access token authentication
    #[serde(rename = "token")]
    AccessToken { access_token: String },
    /// SSO authentication with token
    #[serde(rename = "sso")]
    SSO { token: String },
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_account_config_default() {
        let config = MatrixAccountConfig::default();
        assert_eq!(config.homeserver_url, "https://matrix.org");
        assert!(config.enable_e2ee);
        assert!(config.rooms.is_empty());
    }

    #[test]
    fn test_matrix_auth_password_serialization() {
        let config = MatrixAccountConfig {
            homeserver_url: "https://matrix.org".to_string(),
            auth: MatrixAuth::Password {
                username: "user".to_string(),
                password: "pass".to_string(),
            },
            ..Default::default()
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains(r#""type":"password""#));
        assert!(json.contains(r#""username":"user""#));
    }

    #[test]
    fn test_matrix_auth_token_serialization() {
        let config = MatrixAccountConfig {
            homeserver_url: "https://matrix.org".to_string(),
            auth: MatrixAuth::AccessToken {
                access_token: "token123".to_string(),
            },
            ..Default::default()
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains(r#""type":"token""#));
        assert!(json.contains(r#""access_token":"token123""#));
    }

    #[test]
    fn test_matrix_auth_sso_serialization() {
        let config = MatrixAccountConfig {
            homeserver_url: "https://matrix.org".to_string(),
            auth: MatrixAuth::SSO {
                token: "sso_token".to_string(),
            },
            ..Default::default()
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains(r#""type":"sso""#));
        assert!(json.contains(r#""token":"sso_token""#));
    }

    #[test]
    fn test_matrix_config_with_rooms() {
        let config = MatrixAccountConfig {
            homeserver_url: "https://matrix.org".to_string(),
            auth: MatrixAuth::Password {
                username: "user".to_string(),
                password: "pass".to_string(),
            },
            rooms: vec![
                "!room:matrix.org".to_string(),
                "#general:matrix.org".to_string(),
            ],
            ..Default::default()
        };

        assert_eq!(config.rooms.len(), 2);
        assert_eq!(config.rooms[0], "!room:matrix.org");
        assert_eq!(config.rooms[1], "#general:matrix.org");
    }

    #[test]
    fn test_matrix_config_e2ee_disabled() {
        let config = MatrixAccountConfig {
            enable_e2ee: false,
            ..Default::default()
        };

        assert!(!config.enable_e2ee);
    }
}
