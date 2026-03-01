//! Microsoft Teams configuration types.
//!
//! This module provides configuration structures for Microsoft Teams channel,
//! including Azure AD authentication settings and Bot Framework configuration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for a Microsoft Teams channel account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MsTeamsAccountConfig {
    /// Unique identifier for this account
    pub id: String,
    /// Azure AD tenant ID
    pub tenant_id: String,
    /// Azure AD client ID (application ID)
    pub client_id: String,
    /// Azure AD client secret
    pub client_secret: String,
    /// Optional bot framework app ID (Microsoft App ID)
    #[serde(default)]
    pub bot_app_id: Option<String>,
    /// Optional bot framework app password
    #[serde(default)]
    pub bot_app_password: Option<String>,
    /// List of allowed user IDs (if empty, all users are allowed)
    #[serde(default)]
    pub allowed_users: Vec<String>,
    /// List of allowed channel IDs (if empty, all channels are allowed)
    #[serde(default)]
    pub allowed_channels: Vec<String>,
}

impl Default for MsTeamsAccountConfig {
    fn default() -> Self {
        Self {
            id: String::new(),
            tenant_id: String::new(),
            client_id: String::new(),
            client_secret: String::new(),
            bot_app_id: None,
            bot_app_password: None,
            allowed_users: Vec::new(),
            allowed_channels: Vec::new(),
        }
    }
}

/// Microsoft Teams channel configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MsTeamsConfig {
    /// List of Microsoft Teams accounts
    #[serde(default)]
    pub accounts: Vec<MsTeamsAccountConfig>,
    /// Webhook endpoint configuration
    #[serde(default)]
    pub webhook: WebhookConfig,
}

impl Default for MsTeamsConfig {
    fn default() -> Self {
        Self {
            accounts: Vec::new(),
            webhook: WebhookConfig::default(),
        }
    }
}

/// Webhook endpoint configuration for receiving Bot Framework activities.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WebhookConfig {
    /// Enable webhook mode for incoming activities
    #[serde(default)]
    pub enabled: bool,
    /// Port to listen on for webhook requests
    #[serde(default = "default_webhook_port")]
    pub port: u16,
    /// Optional path for webhook endpoint
    #[serde(default)]
    pub path: String,
    /// Microsoft app ID for webhook validation
    #[serde(default)]
    pub microsoft_app_id: Option<String>,
    /// Microsoft app password for webhook validation
    #[serde(default)]
    pub microsoft_app_password: Option<String>,
}

fn default_webhook_port() -> u16 {
    3978
}

/// Microsoft Teams channel configuration with flattened account structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MsTeamsFlattenedConfig {
    /// Azure AD tenant ID
    pub tenant_id: String,
    /// Azure AD client ID (application ID)
    pub client_id: String,
    /// Azure AD client secret
    pub client_secret: String,
    /// Bot framework app ID (Microsoft App ID)
    #[serde(default)]
    pub bot_app_id: Option<String>,
    /// Bot framework app password
    #[serde(default)]
    pub bot_app_password: Option<String>,
    /// List of allowed user IDs
    #[serde(default)]
    pub allowed_users: Vec<String>,
    /// List of allowed channel IDs
    #[serde(default)]
    pub allowed_channels: Vec<String>,
    /// Webhook configuration
    #[serde(default)]
    pub webhook: WebhookConfig,
}

impl From<MsTeamsFlattenedConfig> for MsTeamsConfig {
    fn from(config: MsTeamsFlattenedConfig) -> Self {
        let account = MsTeamsAccountConfig {
            id: "default".to_string(),
            tenant_id: config.tenant_id,
            client_id: config.client_id,
            client_secret: config.client_secret,
            bot_app_id: config.bot_app_id,
            bot_app_password: config.bot_app_password,
            allowed_users: config.allowed_users,
            allowed_channels: config.allowed_channels,
        };

        Self {
            accounts: vec![account],
            webhook: config.webhook,
        }
    }
}

/// Configuration helper methods for Microsoft Teams.
impl MsTeamsAccountConfig {
    /// Creates a new Microsoft Teams account configuration.
    pub fn new(id: &str, tenant_id: &str, client_id: &str, client_secret: &str) -> Self {
        Self {
            id: id.to_string(),
            tenant_id: tenant_id.to_string(),
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            ..Default::default()
        }
    }

    /// Creates a new Microsoft Teams account configuration with bot framework support.
    pub fn with_bot(
        id: &str,
        tenant_id: &str,
        client_id: &str,
        client_secret: &str,
        bot_app_id: &str,
        bot_app_password: &str,
    ) -> Self {
        Self {
            id: id.to_string(),
            tenant_id: tenant_id.to_string(),
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            bot_app_id: Some(bot_app_id.to_string()),
            bot_app_password: Some(bot_app_password.to_string()),
            ..Default::default()
        }
    }

    /// Check if a user ID is allowed.
    pub fn is_user_allowed(&self, user_id: &str) -> bool {
        self.allowed_users.is_empty() || self.allowed_users.contains(&user_id.to_string())
    }

    /// Check if a channel ID is allowed.
    pub fn is_channel_allowed(&self, channel_id: &str) -> bool {
        self.allowed_channels.is_empty() || self.allowed_channels.contains(&channel_id.to_string())
    }

    /// Get the bot app ID, or return the client ID if not set.
    pub fn bot_app_id_or_client_id(&self) -> &str {
        self.bot_app_id.as_deref().unwrap_or(&self.client_id)
    }
}

/// Configuration validation methods.
impl MsTeamsConfig {
    /// Validates the configuration and returns an error if invalid.
    pub fn validate(&self) -> Result<(), anyhow::Error> {
        if self.accounts.is_empty() {
            return Err(anyhow::anyhow!(
                "At least one Microsoft Teams account is required"
            ));
        }

        for account in &self.accounts {
            if account.tenant_id.is_empty() {
                return Err(anyhow::anyhow!("Tenant ID is required for each account"));
            }
            if account.client_id.is_empty() {
                return Err(anyhow::anyhow!("Client ID is required for each account"));
            }
            if account.client_secret.is_empty() {
                return Err(anyhow::anyhow!(
                    "Client secret is required for each account"
                ));
            }
        }

        Ok(())
    }

    /// Get an account by its ID.
    pub fn get_account(&self, id: &str) -> Option<&MsTeamsAccountConfig> {
        self.accounts.iter().find(|a| a.id == id)
    }

    /// Get an account by its ID (mutable).
    pub fn get_account_mut(&mut self, id: &str) -> Option<&mut MsTeamsAccountConfig> {
        self.accounts.iter_mut().find(|a| a.id == id)
    }

    /// Get all account IDs.
    pub fn get_account_ids(&self) -> Vec<String> {
        self.accounts.iter().map(|a| a.id.clone()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ms_teams_account_config_default() {
        let config = MsTeamsAccountConfig::default();
        assert!(config.id.is_empty());
        assert!(config.tenant_id.is_empty());
        assert!(config.client_id.is_empty());
        assert!(config.client_secret.is_empty());
    }

    #[test]
    fn test_ms_teams_account_config_new() {
        let config = MsTeamsAccountConfig::new("test", "tenant123", "client123", "secret123");
        assert_eq!(config.id, "test");
        assert_eq!(config.tenant_id, "tenant123");
        assert_eq!(config.client_id, "client123");
        assert_eq!(config.client_secret, "secret123");
    }

    #[test]
    fn test_ms_teams_account_config_with_bot() {
        let config = MsTeamsAccountConfig::with_bot(
            "test",
            "tenant123",
            "client123",
            "secret123",
            "bot123",
            "bot_secret123",
        );
        assert_eq!(config.bot_app_id, Some("bot123".to_string()));
        assert_eq!(config.bot_app_password, Some("bot_secret123".to_string()));
    }

    #[test]
    fn test_user_allowed() {
        let config = MsTeamsAccountConfig {
            id: "test".to_string(),
            tenant_id: "tenant123".to_string(),
            client_id: "client123".to_string(),
            client_secret: "secret123".to_string(),
            allowed_users: vec!["user1".to_string(), "user2".to_string()],
            ..Default::default()
        };

        assert!(config.is_user_allowed("user1"));
        assert!(!config.is_user_allowed("user3"));

        // Empty allowed_users means all users are allowed
        let config_empty = MsTeamsAccountConfig::default();
        assert!(config_empty.is_user_allowed("any_user"));
    }

    #[test]
    fn test_channel_allowed() {
        let config = MsTeamsAccountConfig {
            id: "test".to_string(),
            tenant_id: "tenant123".to_string(),
            client_id: "client123".to_string(),
            client_secret: "secret123".to_string(),
            allowed_channels: vec!["channel1".to_string()],
            ..Default::default()
        };

        assert!(config.is_channel_allowed("channel1"));
        assert!(!config.is_channel_allowed("channel2"));
    }

    #[test]
    fn test_bot_app_id_or_client_id() {
        let config = MsTeamsAccountConfig::with_bot(
            "test",
            "tenant123",
            "client123",
            "secret123",
            "bot123",
            "bot_secret123",
        );
        assert_eq!(config.bot_app_id_or_client_id(), "bot123");

        let config_no_bot =
            MsTeamsAccountConfig::new("test", "tenant123", "client123", "secret123");
        assert_eq!(config_no_bot.bot_app_id_or_client_id(), "client123");
    }

    #[test]
    fn test_ms_teams_config_validate() {
        let config = MsTeamsConfig::default();
        assert!(config.validate().is_err());

        let config = MsTeamsConfig {
            accounts: vec![MsTeamsAccountConfig::new(
                "test",
                "tenant123",
                "client123",
                "secret123",
            )],
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }
}
