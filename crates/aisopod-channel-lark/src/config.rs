//! Configuration for Lark/Feishu channel.
//!
//! This module provides the configuration structure for connecting
//! to the Lark Open Platform API.

use serde::{Deserialize, Serialize};

/// Configuration for a Lark/Feishu channel account.
///
/// This struct contains all the necessary credentials and settings
/// for connecting to the Lark Open Platform API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LarkConfig {
    /// App ID from Lark Developer Console
    pub app_id: String,
    /// App Secret from Lark Developer Console
    pub app_secret: String,
    /// Verification token for event subscriptions
    pub verification_token: String,
    /// Encrypt key for event encryption (optional)
    pub encrypt_key: Option<String>,
    /// Webhook port for event subscriptions
    pub webhook_port: u16,
    /// Use Feishu domain instead of Lark (for China region)
    pub use_feishu: bool,
}

impl Default for LarkConfig {
    fn default() -> Self {
        Self {
            app_id: String::new(),
            app_secret: String::new(),
            verification_token: String::new(),
            encrypt_key: None,
            webhook_port: 8080,
            use_feishu: false,
        }
    }
}

impl LarkConfig {
    /// Returns the base URL for the Lark/Feishu API.
    pub fn base_url(&self) -> String {
        if self.use_feishu {
            "https://open.feishu.cn".to_string()
        } else {
            "https://open.larksuite.com".to_string()
        }
    }

    /// Returns the webhook URL for event subscriptions.
    pub fn webhook_url(&self, hostname: &str) -> String {
        format!("https://{hostname}:{}/lark/events", self.webhook_port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = LarkConfig::default();
        assert!(config.app_id.is_empty());
        assert!(config.app_secret.is_empty());
        assert!(config.verification_token.is_empty());
        assert_eq!(config.webhook_port, 8080);
        assert!(!config.use_feishu);
    }

    #[test]
    fn test_base_url() {
        let mut config = LarkConfig::default();
        config.use_feishu = false;
        assert_eq!(config.base_url(), "https://open.larksuite.com");

        config.use_feishu = true;
        assert_eq!(config.base_url(), "https://open.feishu.cn");
    }
}
