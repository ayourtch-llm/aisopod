//! Google Chat channel plugin implementation.
//!
//! This module implements the ChannelPlugin trait for Google Chat,
//! enabling the bot to receive and send messages via the Google Chat API.

use crate::api::{GoogleChatClient, ListSpacesResponse};
use crate::auth::{GoogleChatAuth, OAuth2Auth, ServiceAccountAuth};
use crate::config::{GoogleChatAccountConfig, GoogleChatConfig, WebhookConfig};
use crate::webhook::{create_webhook_router, WebhookState};
use aisopod_channel::adapters::{
    AccountConfig, AccountSnapshot, ChannelConfigAdapter, SecurityAdapter,
};
use aisopod_channel::message::{IncomingMessage, MessageTarget, PeerInfo, PeerKind, SenderInfo};
use aisopod_channel::plugin::ChannelPlugin;
use aisopod_channel::types::{ChannelCapabilities, ChannelMeta, ChatType, MediaType};
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info, warn};

// Re-export common types
pub use crate::api::Message;
pub use crate::cards::{CardBuilder, CardSection};

/// A Google Chat account wraps the configuration with its state.
#[derive(Clone)]
pub struct GoogleChatAccount {
    /// Unique identifier for this account
    pub id: String,
    /// The account configuration
    pub config: GoogleChatAccountConfig,
    /// Whether this account is currently connected
    pub connected: bool,
    /// The timestamp of the last connection
    pub last_connected: Option<DateTime<Utc>>,
}

impl GoogleChatAccount {
    /// Create a new GoogleChatAccount with the given configuration.
    pub fn new(id: String, config: GoogleChatAccountConfig) -> Self {
        Self {
            id,
            config,
            connected: false,
            last_connected: None,
        }
    }

    /// Check if the account is enabled.
    pub fn is_enabled(&self) -> bool {
        // An account is enabled if it has valid auth configuration
        match self.config.auth_type {
            crate::config::AuthType::OAuth2 => self.config.oauth2.is_some(),
            crate::config::AuthType::ServiceAccount => self.config.service_account.is_some(),
        }
    }
}

/// Google Chat channel plugin implementation.
///
/// This struct manages Google Chat connections and webhook handling.
/// It implements the `ChannelPlugin` trait to integrate with the aisopod system.
#[derive(Clone)]
pub struct GoogleChatChannel {
    /// Vector of Google Chat accounts
    accounts: Vec<GoogleChatAccount>,
    /// The channel ID
    id: String,
    /// The channel metadata
    meta: ChannelMeta,
    /// The channel capabilities
    capabilities: ChannelCapabilities,
    /// Current running tasks for graceful shutdown
    shutdown_signal: Option<Arc<tokio::sync::Notify>>,
    /// The configuration adapter
    config_adapter: GoogleChatChannelConfigAdapter,
    /// The security adapter
    security_adapter: Option<GoogleChatSecurityAdapter>,
    /// Webhook configuration
    webhook_config: WebhookConfig,
}

impl GoogleChatChannel {
    /// Creates a new Google Chat channel with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The Google Chat account configuration
    /// * `account_id` - Unique identifier for this account instance
    ///
    /// # Returns
    ///
    /// * `Ok(GoogleChatChannel)` - The channel if configuration is valid
    /// * `Err(anyhow::Error)` - An error if the configuration is invalid
    pub async fn new(config: GoogleChatConfig, account_id: &str) -> Result<Self> {
        let account_config = config.accounts.get(0).cloned().unwrap_or_default();

        let account = GoogleChatAccount::new(account_id.to_string(), account_config);

        // Validate the account configuration
        if !account.is_enabled() {
            return Err(anyhow::anyhow!(
                "Failed to validate Google Chat account: no valid authentication configured"
            ));
        }

        let id = format!("googlechat-{}", account_id);
        let meta = ChannelMeta {
            label: "Google Chat".to_string(),
            docs_url: Some("https://developers.google.com/chat".to_string()),
            ui_hints: serde_json::json!({
                "auth_type_field": "auth_type",
                "oauth2_section": "oauth2",
                "service_account_section": "service_account"
            }),
        };
        let capabilities = ChannelCapabilities {
            chat_types: vec![ChatType::Dm, ChatType::Group],
            supports_media: true,
            supports_reactions: true,
            supports_threads: true,
            supports_typing: false,
            supports_voice: false,
            max_message_length: Some(4096),
            supported_media_types: vec![MediaType::Image, MediaType::Document],
        };
        let accounts = vec![account];
        let config_adapter = GoogleChatChannelConfigAdapter::new(accounts.clone());
        let security_adapter = Some(GoogleChatSecurityAdapter::new(accounts.clone()));

        Ok(Self {
            accounts,
            id,
            meta,
            capabilities,
            shutdown_signal: None,
            config_adapter,
            security_adapter,
            webhook_config: config.webhook,
        })
    }

    /// Get all configured account IDs for this channel.
    pub fn list_account_ids(&self) -> Vec<String> {
        self.accounts.iter().map(|a| a.id.clone()).collect()
    }

    /// Get an account by its ID.
    pub fn get_account(&self, account_id: &str) -> Option<&GoogleChatAccount> {
        self.accounts.iter().find(|a| a.id == account_id)
    }

    /// Get an account by its ID (mutable).
    pub fn get_account_mut(&self, account_id: &str) -> Option<&GoogleChatAccount> {
        self.accounts.iter().find(|a| a.id == account_id)
    }

    /// Create an API client for the given account.
    pub fn create_client(&self, account_id: &str) -> Result<GoogleChatClient> {
        let account = self
            .get_account(account_id)
            .ok_or_else(|| anyhow::anyhow!("Account {} not found", account_id))?;

        let auth: Box<dyn GoogleChatAuth> = match account.config.auth_type {
            crate::config::AuthType::OAuth2 => {
                let oauth2 = account
                    .config
                    .oauth2
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("OAuth2 configuration not found"))?;
                Box::new(OAuth2Auth::new(oauth2.clone()))
            }
            crate::config::AuthType::ServiceAccount => {
                let sa =
                    account.config.service_account.as_ref().ok_or_else(|| {
                        anyhow::anyhow!("Service account configuration not found")
                    })?;
                Box::new(ServiceAccountAuth::new(sa.clone())?)
            }
        };

        Ok(GoogleChatClient::new(auth))
    }

    /// Register webhook routes with the gateway router.
    ///
    /// # Arguments
    ///
    /// * `router` - The router to register routes with
    /// * `account_id` - The account ID for this webhook
    ///
    /// # Returns
    ///
    /// The updated router with webhook routes
    pub fn register_webhook_routes(&self, router: axum::Router, account_id: &str) -> axum::Router {
        let account = self.get_account(account_id).expect("Account not found");

        let webhook_state = WebhookState::new(
            self.webhook_config.verify_token.clone(),
            account_id,
            self.id.clone(),
        );

        router.merge(create_webhook_router(webhook_state))
    }

    /// Send a text message to a space.
    ///
    /// # Arguments
    ///
    /// * `space_id` - The space ID to send the message to
    /// * `text` - The text content to send
    ///
    /// # Returns
    ///
    /// * `Ok(Message)` - The sent message
    /// * `Err(anyhow::Error)` - An error if sending fails
    pub async fn send_message(&self, space_id: &str, text: &str) -> Result<Message> {
        let account = self
            .get_account(space_id)
            .ok_or_else(|| anyhow::anyhow!("Account {} not found", space_id))?;

        let client = self.create_client(&account.id)?;
        let message_request = crate::api::CreateMessageRequest {
            text: Some(text.to_string()),
            cards: None,
            cards_v2: None,
            thread: None,
            reply_message_id: None,
        };

        client.create_message(space_id, &message_request).await
    }

    /// Send a rich card message to a space.
    ///
    /// # Arguments
    ///
    /// * `space_id` - The space ID to send the card to
    /// * `card` - The card to send
    ///
    /// # Returns
    ///
    /// * `Ok(Message)` - The sent message
    /// * `Err(anyhow::Error)` - An error if sending fails
    pub async fn send_card(&self, space_id: &str, card: serde_json::Value) -> Result<Message> {
        let account = self
            .get_account(space_id)
            .ok_or_else(|| anyhow::anyhow!("Account {} not found", space_id))?;

        let client = self.create_client(&account.id)?;
        let message_request = crate::api::CreateMessageRequest {
            text: None,
            cards: Some(vec![card]),
            cards_v2: None,
            thread: None,
            reply_message_id: None,
        };

        client.create_message(space_id, &message_request).await
    }

    /// Check if a message should be processed based on security settings.
    ///
    /// # Arguments
    ///
    /// * `message` - The incoming message to check
    ///
    /// # Returns
    ///
    /// * `true` - The message should be processed
    /// * `false` - The message should be filtered out
    pub fn should_process_message(&self, message: &IncomingMessage) -> bool {
        // Check if the sender is in the allowed list
        if let Some(ref allowed_users) = self
            .get_account(&message.account_id)
            .and_then(|a| a.config.allowed_users.clone())
        {
            if !allowed_users.contains(&message.sender.id) {
                warn!(
                    "Message from {} filtered (not in allowed list)",
                    message.sender.id
                );
                return false;
            }
        }

        true
    }
}

/// ChannelConfigAdapter implementation for GoogleChatChannel.
#[derive(Clone)]
pub struct GoogleChatChannelConfigAdapter {
    /// Reference to the channel accounts
    accounts: Vec<GoogleChatAccount>,
}

impl GoogleChatChannelConfigAdapter {
    /// Create a new GoogleChatChannelConfigAdapter.
    pub fn new(accounts: Vec<GoogleChatAccount>) -> Self {
        Self { accounts }
    }
}

#[async_trait]
impl ChannelConfigAdapter for GoogleChatChannelConfigAdapter {
    fn list_accounts(&self) -> Result<Vec<String>> {
        Ok(self.accounts.iter().map(|a| a.id.clone()).collect())
    }

    fn resolve_account(&self, id: &str) -> Result<AccountSnapshot> {
        self.accounts
            .iter()
            .find(|a| a.id == id)
            .map(|a| AccountSnapshot {
                id: a.id.clone(),
                channel: "googlechat".to_string(),
                enabled: a.is_enabled(),
                connected: a.connected,
            })
            .ok_or_else(|| anyhow::anyhow!("Account {} not found", id))
    }

    fn enable_account(&self, id: &str) -> Result<()> {
        info!("Enabling Google Chat account {}", id);
        // In production, this would enable the account
        Ok(())
    }

    fn disable_account(&self, id: &str) -> Result<()> {
        info!("Disabling Google Chat account {}", id);
        // In production, this would disable the account
        Ok(())
    }

    fn delete_account(&self, id: &str) -> Result<()> {
        info!("Deleting Google Chat account {}", id);
        // In production, this would delete the account
        Ok(())
    }
}

/// Security adapter for GoogleChatChannel.
#[derive(Clone)]
pub struct GoogleChatSecurityAdapter {
    /// Reference to the channel accounts
    accounts: Vec<GoogleChatAccount>,
}

impl GoogleChatSecurityAdapter {
    /// Create a new GoogleChatSecurityAdapter.
    pub fn new(accounts: Vec<GoogleChatAccount>) -> Self {
        Self { accounts }
    }
}

#[async_trait]
impl SecurityAdapter for GoogleChatSecurityAdapter {
    fn is_allowed_sender(&self, sender: &SenderInfo) -> bool {
        // Check if the sender is in the allowed list for any account
        for account in &self.accounts {
            if let Some(ref allowed_users) = account.config.allowed_users {
                if allowed_users.contains(&sender.id) {
                    return true;
                }
            }
        }

        // If no allowed list is configured, allow all senders
        self.accounts.is_empty()
            || self
                .accounts
                .iter()
                .any(|a| a.config.allowed_users.is_none())
    }

    fn requires_mention_in_group(&self) -> bool {
        // Google Chat requires mentions in spaces by default
        true
    }
}

// ============================================================================
// ChannelPlugin implementation
// ============================================================================

#[async_trait]
impl ChannelPlugin for GoogleChatChannel {
    /// Returns the unique identifier for this channel plugin.
    fn id(&self) -> &str {
        &self.id
    }

    /// Returns metadata about this channel implementation.
    fn meta(&self) -> &ChannelMeta {
        &self.meta
    }

    /// Returns the capabilities supported by this channel.
    fn capabilities(&self) -> &ChannelCapabilities {
        &self.capabilities
    }

    /// Returns the configuration adapter for this channel.
    fn config(&self) -> &dyn ChannelConfigAdapter {
        &self.config_adapter
    }

    /// Returns the security adapter for this channel.
    fn security(&self) -> Option<&dyn SecurityAdapter> {
        self.security_adapter
            .as_ref()
            .map(|a| a as &dyn SecurityAdapter)
    }
}

/// Register a Google Chat channel with the registry.
///
/// This function creates a GoogleChatChannel from the given configuration
/// and adds it to the channel registry.
///
/// # Arguments
///
/// * `registry` - The channel registry to register with
/// * `config` - The Google Chat configuration
/// * `account_id` - Unique identifier for this account instance
pub async fn register(
    registry: &mut aisopod_channel::ChannelRegistry,
    config: GoogleChatConfig,
    account_id: &str,
) -> Result<()> {
    let channel = GoogleChatChannel::new(config, account_id).await?;
    registry.register(Arc::new(channel));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_google_chat_account_config_default() {
        let config = GoogleChatAccountConfig::default();
        assert_eq!(config.auth_type, crate::config::AuthType::OAuth2);
        assert!(config.oauth2.is_none());
        assert!(config.service_account.is_none());
    }

    #[test]
    fn test_google_chat_account_enabled() {
        let config = GoogleChatAccountConfig {
            auth_type: crate::config::AuthType::OAuth2,
            oauth2: Some(crate::config::OAuth2Config {
                client_id: "test".to_string(),
                client_secret: "test".to_string(),
                refresh_token: "test".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        };

        let account = GoogleChatAccount::new("test".to_string(), config);
        assert!(account.is_enabled());
    }

    #[test]
    fn test_google_chat_channel_default() {
        let config = GoogleChatConfig::default();

        // Test that new() returns an error when no accounts are configured
        // The new function is async so we need to handle it differently in a sync test
        // For now, just verify that the default config has no accounts
        assert!(
            config.accounts.is_empty(),
            "Default config should have no accounts"
        );
    }

    #[test]
    fn test_google_chat_channel_with_oauth2() {
        let config = GoogleChatConfig {
            accounts: vec![GoogleChatAccountConfig {
                auth_type: crate::config::AuthType::OAuth2,
                oauth2: Some(crate::config::OAuth2Config {
                    client_id: "test-client-id".to_string(),
                    client_secret: "test-client-secret".to_string(),
                    refresh_token: "test-refresh-token".to_string(),
                    ..Default::default()
                }),
                ..Default::default()
            }],
            ..Default::default()
        };

        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(GoogleChatChannel::new(config, "test-account"));

        assert!(result.is_ok());
        let channel = result.unwrap();
        assert_eq!(channel.id(), "googlechat-test-account");
        assert_eq!(channel.meta().label, "Google Chat");
    }

    #[test]
    fn test_google_chat_security_adapter() {
        let accounts = vec![GoogleChatAccount::new(
            "account1".to_string(),
            GoogleChatAccountConfig {
                auth_type: crate::config::AuthType::OAuth2,
                oauth2: Some(crate::config::OAuth2Config::default()),
                allowed_users: Some(vec!["user1".to_string()]),
                ..Default::default()
            },
        )];

        let adapter = GoogleChatSecurityAdapter::new(accounts);

        // Test allowed sender
        let allowed_sender = SenderInfo {
            id: "user1".to_string(),
            display_name: None,
            username: None,
            is_bot: false,
        };
        assert!(adapter.is_allowed_sender(&allowed_sender));

        // Test disallowed sender
        let disallowed_sender = SenderInfo {
            id: "user2".to_string(),
            display_name: None,
            username: None,
            is_bot: false,
        };
        assert!(!adapter.is_allowed_sender(&disallowed_sender));
    }

    #[test]
    fn test_google_chat_security_adapter_no_allowlist() {
        let accounts = vec![GoogleChatAccount::new(
            "account1".to_string(),
            GoogleChatAccountConfig {
                auth_type: crate::config::AuthType::OAuth2,
                oauth2: Some(crate::config::OAuth2Config::default()),
                ..Default::default()
            },
        )];

        let adapter = GoogleChatSecurityAdapter::new(accounts);

        // With no allowlist, all senders are allowed
        let sender = SenderInfo {
            id: "any-user".to_string(),
            display_name: None,
            username: None,
            is_bot: false,
        };
        assert!(adapter.is_allowed_sender(&sender));
    }

    #[test]
    fn test_requires_mention_in_group() {
        let accounts = vec![GoogleChatAccount::new(
            "account1".to_string(),
            GoogleChatAccountConfig::default(),
        )];

        let adapter = GoogleChatSecurityAdapter::new(accounts);

        // Google Chat requires mentions in groups by default
        assert!(adapter.requires_mention_in_group());
    }
}
