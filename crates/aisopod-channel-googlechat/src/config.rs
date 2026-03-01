//! Google Chat channel configuration.

use serde::{Deserialize, Serialize};

/// Configuration for a Google Chat bot account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleChatAccountConfig {
    /// Authentication type: "oauth2" or "service_account"
    #[serde(rename = "auth_type")]
    pub auth_type: AuthType,
    /// OAuth 2.0 configuration
    #[serde(default)]
    pub oauth2: Option<OAuth2Config>,
    /// Service account configuration
    #[serde(default)]
    pub service_account: Option<ServiceAccountConfig>,
    /// Optional list of allowed user IDs (if empty, all users are allowed)
    #[serde(default)]
    pub allowed_users: Option<Vec<String>>,
    /// Optional list of allowed space IDs (if empty, all spaces are allowed)
    #[serde(default)]
    pub allowed_spaces: Option<Vec<String>>,
    /// Whether messages require a bot mention to be received
    #[serde(default = "default_mention_required")]
    pub mention_required: bool,
}

impl Default for GoogleChatAccountConfig {
    fn default() -> Self {
        Self {
            auth_type: AuthType::OAuth2,
            oauth2: None,
            service_account: None,
            allowed_users: None,
            allowed_spaces: None,
            mention_required: false,
        }
    }
}

impl GoogleChatAccountConfig {
    /// Create a new GoogleChatAccountConfig with OAuth 2.0 authentication.
    pub fn oauth2(oauth2: OAuth2Config) -> Self {
        Self {
            auth_type: AuthType::OAuth2,
            oauth2: Some(oauth2),
            service_account: None,
            allowed_users: None,
            allowed_spaces: None,
            mention_required: false,
        }
    }

    /// Create a new GoogleChatAccountConfig with service account authentication.
    pub fn service_account(service_account: ServiceAccountConfig) -> Self {
        Self {
            auth_type: AuthType::ServiceAccount,
            service_account: Some(service_account),
            oauth2: None,
            allowed_users: None,
            allowed_spaces: None,
            mention_required: false,
        }
    }
}

fn default_mention_required() -> bool {
    false
}

/// Authentication type for Google Chat API.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuthType {
    /// OAuth 2.0 authentication
    #[serde(rename = "oauth2")]
    OAuth2,
    /// Service account authentication
    #[serde(rename = "service_account")]
    ServiceAccount,
}

/// OAuth 2.0 configuration for Google Chat API.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OAuth2Config {
    /// Client ID from Google Cloud Console
    pub client_id: String,
    /// Client secret from Google Cloud Console
    pub client_secret: String,
    /// Refresh token for accessing Google Chat API
    pub refresh_token: String,
    /// Optional token URI (defaults to Google's OAuth endpoint)
    #[serde(default = "default_token_uri")]
    pub token_uri: String,
    /// Optional project number (for service account impersonation)
    pub project_number: Option<String>,
}

fn default_token_uri() -> String {
    "https://oauth2.googleapis.com/token".to_string()
}

impl Default for OAuth2Config {
    fn default() -> Self {
        Self {
            client_id: String::new(),
            client_secret: String::new(),
            refresh_token: String::new(),
            token_uri: default_token_uri(),
            project_number: None,
        }
    }
}

/// Service account configuration for Google Chat API.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServiceAccountConfig {
    /// Path to the service account key file (JSON)
    pub key_file: String,
    /// Email address of the service account
    pub client_email: String,
    /// Private key from the service account key file
    pub private_key: String,
    /// Optional project number (for service account impersonation)
    pub project_number: Option<String>,
}

impl Default for ServiceAccountConfig {
    fn default() -> Self {
        Self {
            key_file: String::new(),
            client_email: String::new(),
            private_key: String::new(),
            project_number: None,
        }
    }
}

/// Configuration for webhook endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    /// The secret token for webhook verification
    pub verify_token: String,
    /// Optional path for webhook endpoint (defaults to /googlechat/webhook)
    #[serde(default = "default_webhook_path")]
    pub path: String,
}

fn default_webhook_path() -> String {
    "/googlechat/webhook".to_string()
}

impl Default for WebhookConfig {
    fn default() -> Self {
        Self {
            verify_token: String::new(),
            path: default_webhook_path(),
        }
    }
}

/// Complete Google Chat channel configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleChatConfig {
    /// List of Google Chat account configurations
    #[serde(default)]
    pub accounts: Vec<GoogleChatAccountConfig>,
    /// Webhook configuration
    #[serde(default)]
    pub webhook: WebhookConfig,
}

impl Default for GoogleChatConfig {
    fn default() -> Self {
        Self {
            accounts: Vec::new(),
            webhook: WebhookConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_google_chat_account_config_default() {
        let config = GoogleChatAccountConfig::default();
        assert_eq!(config.auth_type, AuthType::OAuth2);
        assert!(config.oauth2.is_none());
        assert!(config.service_account.is_none());
        assert!(config.allowed_users.is_none());
        assert!(config.allowed_spaces.is_none());
        assert!(!config.mention_required);
    }

    #[test]
    fn test_google_chat_account_config_oauth2() {
        let oauth2 = OAuth2Config {
            client_id: "test-client-id".to_string(),
            client_secret: "test-client-secret".to_string(),
            refresh_token: "test-refresh-token".to_string(),
            ..Default::default()
        };

        let config = GoogleChatAccountConfig::oauth2(oauth2.clone());

        assert_eq!(config.auth_type, AuthType::OAuth2);
        assert_eq!(config.oauth2, Some(oauth2));
        assert!(config.service_account.is_none());
    }

    #[test]
    fn test_google_chat_account_config_service_account() {
        let service_account = ServiceAccountConfig {
            key_file: "/path/to/key.json".to_string(),
            client_email: "test@test.iam.gserviceaccount.com".to_string(),
            private_key: "-----BEGIN PRIVATE KEY-----\ntest\n-----END PRIVATE KEY-----\n"
                .to_string(),
            ..Default::default()
        };

        let config = GoogleChatAccountConfig::service_account(service_account.clone());

        assert_eq!(config.auth_type, AuthType::ServiceAccount);
        assert!(config.oauth2.is_none());
        assert_eq!(config.service_account, Some(service_account));
    }

    #[test]
    fn test_google_chat_account_config_custom_settings() {
        let config = GoogleChatAccountConfig {
            auth_type: AuthType::OAuth2,
            oauth2: Some(OAuth2Config {
                client_id: "client123".to_string(),
                ..Default::default()
            }),
            service_account: None,
            allowed_users: Some(vec!["user1".to_string(), "user2".to_string()]),
            allowed_spaces: Some(vec!["space1".to_string()]),
            mention_required: true,
        };

        assert!(config.allowed_users.is_some());
        assert_eq!(config.allowed_users.as_ref().unwrap().len(), 2);
        assert!(config.allowed_spaces.is_some());
        assert!(config.mention_required);
    }

    #[test]
    fn test_google_chat_config_default() {
        let config = GoogleChatConfig::default();
        assert!(config.accounts.is_empty());
        assert_eq!(config.webhook.path, "/googlechat/webhook");
    }

    #[test]
    fn test_google_chat_config_custom() {
        let config = GoogleChatConfig {
            accounts: vec![GoogleChatAccountConfig::default()],
            webhook: WebhookConfig {
                verify_token: "my-secret-token".to_string(),
                path: "/custom/webhook".to_string(),
            },
        };

        assert_eq!(config.accounts.len(), 1);
        assert_eq!(config.webhook.verify_token, "my-secret-token");
        assert_eq!(config.webhook.path, "/custom/webhook");
    }

    #[test]
    fn test_auth_type_serialization() {
        let json = serde_json::to_string(&AuthType::OAuth2).unwrap();
        assert_eq!(json, "\"oauth2\"");

        let json = serde_json::to_string(&AuthType::ServiceAccount).unwrap();
        assert_eq!(json, "\"service_account\"");
    }

    #[test]
    fn test_auth_type_deserialization() {
        let auth: AuthType = serde_json::from_str("\"oauth2\"").unwrap();
        assert_eq!(auth, AuthType::OAuth2);

        let auth: AuthType = serde_json::from_str("\"service_account\"").unwrap();
        assert_eq!(auth, AuthType::ServiceAccount);
    }
}
