//! Mattermost channel configuration.

use serde::{Deserialize, Serialize};

/// Configuration for connecting to a Mattermost server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MattermostConfig {
    /// Mattermost server URL (e.g., "https://mattermost.example.com")
    pub server_url: String,
    /// Authentication method to use
    pub auth: MattermostAuth,
    /// Channels to join (by channel name or ID)
    pub channels: Vec<String>,
    /// Team name or ID (required for some operations)
    #[serde(default)]
    pub team: Option<String>,
}

/// Authentication method for Mattermost API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MattermostAuth {
    /// Bot token authentication
    #[serde(rename = "bot")]
    BotToken { token: String },
    /// Personal access token authentication
    #[serde(rename = "personal")]
    PersonalToken { token: String },
    /// Username/password authentication
    #[serde(rename = "password")]
    Password { username: String, password: String },
}

impl MattermostConfig {
    /// Create a new MattermostConfig with the given server URL.
    pub fn new(server_url: String) -> Self {
        Self {
            server_url,
            auth: MattermostAuth::BotToken {
                token: String::new(),
            },
            channels: Vec::new(),
            team: None,
        }
    }

    /// Set the authentication method.
    pub fn with_auth(mut self, auth: MattermostAuth) -> Self {
        self.auth = auth;
        self
    }

    /// Set the channels to join.
    pub fn with_channels(mut self, channels: Vec<String>) -> Self {
        self.channels = channels;
        self
    }

    /// Set the team name or ID.
    pub fn with_team(mut self, team: impl Into<String>) -> Self {
        self.team = Some(team.into());
        self
    }
}

impl Default for MattermostConfig {
    fn default() -> Self {
        Self::new("https://mattermost.example.com".to_string())
    }
}

/// Error types for Mattermost configuration.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// Server URL is missing or invalid
    #[error("Invalid server URL: {0}")]
    InvalidServerUrl(String),
    /// Authentication configuration is missing or invalid
    #[error("Invalid authentication: {0}")]
    InvalidAuth(String),
    /// Channel configuration is invalid
    #[error("Invalid channel configuration: {0}")]
    InvalidChannel(String),
}

impl MattermostConfig {
    /// Validate the configuration.
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate server URL
        if self.server_url.is_empty() {
            return Err(ConfigError::InvalidServerUrl(
                "Server URL cannot be empty".to_string(),
            ));
        }

        // Parse and validate URL format
        let url = url::Url::parse(&self.server_url)
            .map_err(|e| ConfigError::InvalidServerUrl(format!("Failed to parse URL: {}", e)))?;

        if url.scheme() != "http" && url.scheme() != "https" {
            return Err(ConfigError::InvalidServerUrl(
                "Server URL must use http or https scheme".to_string(),
            ));
        }

        // Validate authentication
        match &self.auth {
            MattermostAuth::BotToken { token } => {
                if token.is_empty() {
                    return Err(ConfigError::InvalidAuth(
                        "Bot token cannot be empty".to_string(),
                    ));
                }
            }
            MattermostAuth::PersonalToken { token } => {
                if token.is_empty() {
                    return Err(ConfigError::InvalidAuth(
                        "Personal token cannot be empty".to_string(),
                    ));
                }
            }
            MattermostAuth::Password { username, password } => {
                if username.is_empty() || password.is_empty() {
                    return Err(ConfigError::InvalidAuth(
                        "Username and password cannot be empty".to_string(),
                    ));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = MattermostConfig::default();
        assert_eq!(config.server_url, "https://mattermost.example.com");
    }

    #[test]
    fn test_new_config() {
        let config = MattermostConfig::new("https://mattermost.example.com".to_string());
        assert_eq!(config.server_url, "https://mattermost.example.com");
    }

    #[test]
    fn test_with_auth() {
        let config = MattermostConfig::new("https://mattermost.example.com".to_string()).with_auth(
            MattermostAuth::BotToken {
                token: "test-token".to_string(),
            },
        );
        assert!(matches!(
            config.auth,
            MattermostAuth::BotToken { token } if token == "test-token"
        ));
    }

    #[test]
    fn test_with_channels() {
        let config = MattermostConfig::new("https://mattermost.example.com".to_string())
            .with_channels(vec!["general".to_string(), "random".to_string()]);
        assert_eq!(config.channels.len(), 2);
        assert_eq!(config.channels[0], "general");
        assert_eq!(config.channels[1], "random");
    }

    #[test]
    fn test_with_team() {
        let config =
            MattermostConfig::new("https://mattermost.example.com".to_string()).with_team("myteam");
        assert_eq!(config.team, Some("myteam".to_string()));
    }

    #[test]
    fn test_validate_success_bot_token() {
        let config = MattermostConfig::new("https://mattermost.example.com".to_string()).with_auth(
            MattermostAuth::BotToken {
                token: "test-token".to_string(),
            },
        );
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_success_personal_token() {
        let config = MattermostConfig::new("https://mattermost.example.com".to_string()).with_auth(
            MattermostAuth::PersonalToken {
                token: "test-token".to_string(),
            },
        );
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_success_password() {
        let config = MattermostConfig::new("https://mattermost.example.com".to_string()).with_auth(
            MattermostAuth::Password {
                username: "user".to_string(),
                password: "pass".to_string(),
            },
        );
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_error_empty_url() {
        let config = MattermostConfig::new("".to_string());
        assert!(matches!(
            config.validate(),
            Err(ConfigError::InvalidServerUrl(_))
        ));
    }

    #[test]
    fn test_validate_error_invalid_url() {
        let config = MattermostConfig::new("not-a-url".to_string());
        assert!(matches!(
            config.validate(),
            Err(ConfigError::InvalidServerUrl(_))
        ));
    }

    #[test]
    fn test_validate_error_bot_token_empty() {
        let config = MattermostConfig::new("https://mattermost.example.com".to_string()).with_auth(
            MattermostAuth::BotToken {
                token: "".to_string(),
            },
        );
        assert!(matches!(
            config.validate(),
            Err(ConfigError::InvalidAuth(_))
        ));
    }

    #[test]
    fn test_validate_error_password_empty() {
        let config = MattermostConfig::new("https://mattermost.example.com".to_string()).with_auth(
            MattermostAuth::Password {
                username: "".to_string(),
                password: "pass".to_string(),
            },
        );
        assert!(matches!(
            config.validate(),
            Err(ConfigError::InvalidAuth(_))
        ));
    }

    #[test]
    fn test_deserialize_bot_token() {
        let json = r#"{
            "server_url": "https://mattermost.example.com",
            "auth": {
                "type": "bot",
                "token": "test-token"
            },
            "channels": ["general"],
            "team": "myteam"
        }"#;
        let config: MattermostConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.server_url, "https://mattermost.example.com");
        assert!(matches!(config.auth, MattermostAuth::BotToken { .. }));
        assert_eq!(config.channels, vec!["general"]);
        assert_eq!(config.team, Some("myteam".to_string()));
    }

    #[test]
    fn test_deserialize_personal_token() {
        let json = r#"{
            "server_url": "https://mattermost.example.com",
            "auth": {
                "type": "personal",
                "token": "test-token"
            },
            "channels": []
        }"#;
        let config: MattermostConfig = serde_json::from_str(json).unwrap();
        assert!(matches!(config.auth, MattermostAuth::PersonalToken { .. }));
    }

    #[test]
    fn test_deserialize_password() {
        let json = r#"{
            "server_url": "https://mattermost.example.com",
            "auth": {
                "type": "password",
                "username": "user",
                "password": "pass"
            },
            "channels": ["general"]
        }"#;
        let config: MattermostConfig = serde_json::from_str(json).unwrap();
        assert!(matches!(config.auth, MattermostAuth::Password { .. }));
    }
}
