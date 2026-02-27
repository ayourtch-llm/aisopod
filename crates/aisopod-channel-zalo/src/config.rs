//! Configuration for Zalo channel.
//!
//! This module provides the configuration structure for connecting
//! to the Zalo Official Account API.

use serde::{Deserialize, Serialize};

/// Configuration for a Zalo Official Account.
///
/// This struct contains all the necessary credentials and settings
/// for connecting to the Zalo OA API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZaloConfig {
    /// Zalo OA App ID from Zalo Developer Console
    pub app_id: String,
    /// Zalo OA App Secret from Zalo Developer Console
    pub app_secret: String,
    /// OAuth refresh token for authentication
    pub refresh_token: String,
    /// Webhook port for event subscriptions
    pub webhook_port: u16,
    /// OA Secret Key for webhook verification
    pub oa_secret_key: String,
    /// Optional: Webhook path (default: /zalo/webhook)
    #[serde(default = "default_webhook_path")]
    pub webhook_path: String,
}

fn default_webhook_path() -> String {
    "/zalo/webhook".to_string()
}

impl Default for ZaloConfig {
    fn default() -> Self {
        Self {
            app_id: String::new(),
            app_secret: String::new(),
            refresh_token: String::new(),
            webhook_port: 8080,
            oa_secret_key: String::new(),
            webhook_path: default_webhook_path(),
        }
    }
}

impl ZaloConfig {
    /// Returns the base URL for the Zalo OA API.
    pub fn base_url(&self) -> String {
        "https://openapi.zalo.me/v3.0/oa".to_string()
    }

    /// Returns the webhook URL for event subscriptions.
    pub fn webhook_url(&self, hostname: &str) -> String {
        format!(
            "https://{hostname}{}",
            self.webhook_path
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ZaloConfig::default();
        assert!(config.app_id.is_empty());
        assert!(config.app_secret.is_empty());
        assert!(config.refresh_token.is_empty());
        assert_eq!(config.webhook_port, 8080);
        assert!(config.oa_secret_key.is_empty());
        assert_eq!(config.webhook_path, "/zalo/webhook");
    }

    #[test]
    fn test_base_url() {
        let config = ZaloConfig::default();
        assert_eq!(config.base_url(), "https://openapi.zalo.me/v3.0/oa");
    }

    #[test]
    fn test_webhook_url() {
        let mut config = ZaloConfig::default();
        config.webhook_path = "/zalo/webhook".to_string();
        assert_eq!(
            config.webhook_url("example.com"),
            "https://example.com/zalo/webhook"
        );
    }
}
