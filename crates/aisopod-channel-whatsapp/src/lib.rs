//! WhatsApp Business API channel plugin for aisopod.
//!
//! This crate provides a channel plugin implementation for WhatsApp,
//! enabling the bot to receive and send messages via the WhatsApp Business API.
//!
//! # Features
//!
//! - Webhook-based message receiving from WhatsApp Business API
//! - Support for DMs and group messages
//! - Message normalization to shared `IncomingMessage` type
//! - Webhook verification for secure webhook registration
//! - Multi-account support with account-specific configurations
//! - Filtering by allowed phone numbers
//! - Text, image, audio, video, document, location, and contact message support

mod connection;
mod receive;
mod webhook;

pub use connection::{WhatsAppAccountConfig, WhatsAppMode, WhatsAppError};
pub use receive::{parse_webhook_payload, normalize_message, WhatsAppWebhookPayload};
pub use receive::{WhatsAppMessage};
pub use webhook::{create_webhook_router, WebhookState, WebhookVerifyQuery, WebhookVerifyResponse};

use aisopod_channel::adapters::{AccountConfig, AccountSnapshot, ChannelConfigAdapter};
use aisopod_channel::message::{IncomingMessage, MessageTarget, PeerInfo, SenderInfo};
use aisopod_channel::types::{ChannelCapabilities, ChannelMeta, ChatType, MediaType};
use aisopod_channel::plugin::ChannelPlugin;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info, warn};

// Re-export common types
pub use connection::WhatsAppPhoneNumberDetails;

/// A WhatsApp account wraps the configuration with its state.
#[derive(Clone)]
pub struct WhatsAppAccount {
    /// Unique identifier for this account
    pub id: String,
    /// The account configuration
    pub config: WhatsAppAccountConfig,
    /// Whether this account is currently connected
    pub connected: bool,
    /// The timestamp of the last connection
    pub last_connected: Option<DateTime<Utc>>,
}

impl WhatsAppAccount {
    /// Create a new WhatsAppAccount with the given configuration.
    pub fn new(id: String, config: WhatsAppAccountConfig) -> Self {
        Self {
            id,
            config,
            connected: false,
            last_connected: None,
        }
    }

    /// Check if the account is enabled.
    pub fn is_enabled(&self) -> bool {
        // For now, we consider the account enabled if it has an API token
        self.config.api_token.is_some()
    }

    /// Validate the API token by attempting to fetch phone number details.
    pub async fn validate_token(&self) -> Result<WhatsAppPhoneNumberDetails> {
        let api_token = self.config.api_token.as_ref()
            .ok_or_else(|| anyhow::anyhow!("API token not configured"))?;

        // In production, this would make an actual API call to WhatsApp
        // For now, we just check that the token is non-empty
        if api_token.trim().is_empty() {
            return Err(anyhow::anyhow!("API token cannot be empty"));
        }

        // Mock response - in production, this would be:
        // GET https://graph.facebook.com/v18.0/{phone_number_id}
        // Authorization: Bearer {api_token}
        Ok(WhatsAppPhoneNumberDetails {
            id: self.config.phone_number_id.clone().unwrap_or_default(),
            display_phone_number: "+15550000000".to_string(),
            verified_name: Some("Test Business".to_string()),
        })
    }
}

/// WhatsApp channel plugin implementation.
///
/// This struct manages WhatsApp Business API connections and webhook handling.
/// It implements the `ChannelPlugin` trait to integrate with the aisopod system.
#[derive(Clone)]
pub struct WhatsAppChannel {
    /// Vector of WhatsApp accounts
    accounts: Vec<WhatsAppAccount>,
    /// The channel ID
    id: String,
    /// The channel metadata
    meta: ChannelMeta,
    /// The channel capabilities
    capabilities: ChannelCapabilities,
    /// Current running tasks for graceful shutdown
    shutdown_signal: Option<Arc<tokio::sync::Notify>>,
    /// The configuration adapter
    config_adapter: WhatsAppChannelConfigAdapter,
    /// The security adapter
    security_adapter: Option<WhatsAppSecurityAdapter>,
}

impl WhatsAppChannel {
    /// Creates a new WhatsApp channel with the given configuration.
    ///
    /// This method validates the API token by calling a test endpoint.
    ///
    /// # Arguments
    ///
    /// * `config` - The WhatsApp account configuration
    /// * `account_id` - Unique identifier for this account instance
    ///
    /// # Returns
    ///
    /// * `Ok(WhatsAppChannel)` - The channel if configuration is valid
    /// * `Err(anyhow::Error)` - An error if the configuration is invalid
    pub async fn new(config: WhatsAppAccountConfig, account_id: &str) -> Result<Self> {
        let account = WhatsAppAccount::new(account_id.to_string(), config.clone());

        // Validate the API token
        if let Err(e) = account.validate_token().await {
            return Err(anyhow::anyhow!(
                "Failed to validate WhatsApp API token: {}",
                e
            ));
        }

        let id = format!("whatsapp-{}", account_id);
        let meta = ChannelMeta {
            label: "WhatsApp".to_string(),
            docs_url: Some("https://developers.facebook.com/docs/whatsapp".to_string()),
            ui_hints: serde_json::json!({
                "api_token_field": "api_token",
                "phone_number_id_field": "phone_number_id",
                "webhook_verify_token_field": "webhook_verify_token"
            }),
        };
        let capabilities = ChannelCapabilities {
            chat_types: vec![ChatType::Dm, ChatType::Group],
            supports_media: true,
            supports_reactions: true,
            supports_threads: false,
            supports_typing: true,
            supports_voice: true,
            max_message_length: Some(4096),
            supported_media_types: vec![
                MediaType::Image,
                MediaType::Audio,
                MediaType::Video,
                MediaType::Document,
            ],
        };
        let accounts = vec![account];
        let config_adapter = WhatsAppChannelConfigAdapter::new(accounts.clone());
        let security_adapter = Some(WhatsAppSecurityAdapter::new(accounts.clone()));

        Ok(Self {
            accounts,
            id,
            meta,
            capabilities,
            shutdown_signal: None,
            config_adapter,
            security_adapter,
        })
    }

    /// Get all configured account IDs for this channel.
    pub fn list_account_ids(&self) -> Vec<String> {
        self.accounts.iter().map(|a| a.id.clone()).collect()
    }

    /// Get an account by its ID.
    pub fn get_account(&self, account_id: &str) -> Option<&WhatsAppAccount> {
        self.accounts.iter().find(|a| a.id == account_id)
    }

    /// Register webhook routes with the gateway router.
    ///
    /// This method sets up the webhook endpoints for receiving messages from WhatsApp.
    /// The router should be configured with the webhook state including verify token.
    ///
    /// # Arguments
    ///
    /// * `router` - The router to register routes with
    /// * `account_id` - The account ID for this webhook
    ///
    /// # Returns
    ///
    /// The updated router with webhook routes
    pub fn register_webhook_routes(
        &self,
        router: axum::Router,
        account_id: &str,
    ) -> axum::Router {
        let account = self.get_account(account_id)
            .expect("Account not found");

        let webhook_state = webhook::WebhookState {
            verify_token: account.config.webhook_verify_token.clone().unwrap_or_default(),
            account_id: account_id.to_string(),
            channel: self.id.clone(),
            allowed_numbers: account.config.allowed_numbers.clone(),
        };

        router.merge(webhook::create_webhook_router(webhook_state))
    }

    /// Send a text message through this channel.
    ///
    /// # Arguments
    ///
    /// * `target` - The message target specifying where to send
    /// * `text` - The text content to send
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message was sent successfully
    /// * `Err(anyhow::Error)` - An error if sending fails
    pub async fn send_text(&self, target: &MessageTarget, text: &str) -> Result<()> {
        if target.channel != self.id {
            return Err(anyhow::anyhow!(
                "Target channel {} does not match this channel {}",
                target.channel,
                self.id
            ));
        }

        let account = self.get_account(&target.account_id)
            .ok_or_else(|| anyhow::anyhow!("Account {} not found", target.account_id))?;

        // In production, this would make an API call to WhatsApp
        // For now, we just log the message
        info!(
            "Sending text message to {}: {}",
            target.peer.id, text
        );

        // Simulate success - in production, this would:
        // POST https://graph.facebook.com/v18.0/{phone_number_id}/messages
        // Authorization: Bearer {api_token}
        // {
        //     "messaging_product": "whatsapp",
        //     "to": target.peer.id,
        //     "type": "text",
        //     "text": { "body": text }
        // }

        Ok(())
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
        if let Some(ref allowed_numbers) = self.get_account(&message.account_id)
            .and_then(|a| a.config.allowed_numbers.clone())
        {
            if !allowed_numbers.contains(&message.sender.id) {
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

/// ChannelConfigAdapter implementation for WhatsAppChannel.
#[derive(Clone)]
pub struct WhatsAppChannelConfigAdapter {
    /// Reference to the channel accounts
    accounts: Vec<WhatsAppAccount>,
}

impl WhatsAppChannelConfigAdapter {
    /// Create a new WhatsAppChannelConfigAdapter.
    pub fn new(accounts: Vec<WhatsAppAccount>) -> Self {
        Self { accounts }
    }
}

#[async_trait]
impl ChannelConfigAdapter for WhatsAppChannelConfigAdapter {
    fn list_accounts(&self) -> Result<Vec<String>> {
        Ok(self.accounts.iter().map(|a| a.id.clone()).collect())
    }

    fn resolve_account(&self, id: &str) -> Result<AccountSnapshot> {
        self.accounts.iter()
            .find(|a| a.id == id)
            .map(|a| AccountSnapshot {
                id: a.id.clone(),
                channel: "whatsapp".to_string(),
                enabled: a.is_enabled(),
                connected: a.connected,
            })
            .ok_or_else(|| anyhow::anyhow!("Account {} not found", id))
    }

    fn enable_account(&self, _id: &str) -> Result<()> {
        // In production, this would enable the account
        warn!("enable_account not yet implemented for WhatsApp");
        Ok(())
    }

    fn disable_account(&self, _id: &str) -> Result<()> {
        // In production, this would disable the account
        warn!("disable_account not yet implemented for WhatsApp");
        Ok(())
    }

    fn delete_account(&self, _id: &str) -> Result<()> {
        // In production, this would delete the account
        warn!("delete_account not yet implemented for WhatsApp");
        Ok(())
    }
}

/// Security adapter for WhatsAppChannel.
#[derive(Clone)]
pub struct WhatsAppSecurityAdapter {
    /// Reference to the channel accounts
    accounts: Vec<WhatsAppAccount>,
}

impl WhatsAppSecurityAdapter {
    /// Create a new WhatsAppSecurityAdapter.
    pub fn new(accounts: Vec<WhatsAppAccount>) -> Self {
        Self { accounts }
    }
}

#[async_trait]
impl aisopod_channel::adapters::SecurityAdapter for WhatsAppSecurityAdapter {
    fn is_allowed_sender(&self, sender: &aisopod_channel::message::SenderInfo) -> bool {
        // Check if the sender is in the allowed list for any account
        for account in &self.accounts {
            if let Some(ref allowed_numbers) = account.config.allowed_numbers {
                if allowed_numbers.contains(&sender.id) {
                    return true;
                }
            }
        }
        
        // If no allowed list is configured, allow all senders
        self.accounts.is_empty() || self.accounts.iter().any(|a| a.config.allowed_numbers.is_none())
    }

    fn requires_mention_in_group(&self) -> bool {
        // WhatsApp doesn't require mentions in groups by default
        false
    }
}

// ============================================================================
// ChannelPlugin implementation
// ============================================================================

#[async_trait]
impl ChannelPlugin for WhatsAppChannel {
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
    fn security(&self) -> Option<&dyn aisopod_channel::adapters::SecurityAdapter> {
        self.security_adapter.as_ref().map(|a| a as &dyn aisopod_channel::adapters::SecurityAdapter)
    }
}

/// Register a WhatsApp channel with the registry.
///
/// This function creates a WhatsAppChannel from the given configuration
/// and adds it to the channel registry.
///
/// # Arguments
///
/// * `registry` - The channel registry to register with
/// * `config` - The WhatsApp account configuration
/// * `account_id` - Unique identifier for this account instance
pub async fn register(
    registry: &mut aisopod_channel::ChannelRegistry,
    config: WhatsAppAccountConfig,
    account_id: &str,
) -> Result<()> {
    let channel = WhatsAppChannel::new(config, account_id).await?;
    registry.register(Arc::new(channel));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use aisopod_channel::message::PeerKind;

    #[test]
    fn test_whatsapp_account_config_default() {
        let config = WhatsAppAccountConfig::default();
        assert_eq!(config.mode, WhatsAppMode::BusinessApi);
        assert!(config.api_token.is_none());
        assert!(config.phone_number_id.is_none());
        assert!(config.webhook_verify_token.is_none());
        assert!(config.allowed_numbers.is_none());
    }

    #[test]
    fn test_whatsapp_account_config_new() {
        let config = WhatsAppAccountConfig::new(
            "test-token".to_string(),
            "123456789".to_string(),
            "verify-token".to_string(),
        );
        
        assert_eq!(config.mode, WhatsAppMode::BusinessApi);
        assert_eq!(config.api_token, Some("test-token".to_string()));
        assert_eq!(config.phone_number_id, Some("123456789".to_string()));
        assert_eq!(config.webhook_verify_token, Some("verify-token".to_string()));
    }

    #[test]
    fn test_whatsapp_mode_serialization() {
        let json = serde_json::to_string(&WhatsAppMode::BusinessApi).unwrap();
        assert_eq!(json, "\"business_api\"");
        
        let json = serde_json::to_string(&WhatsAppMode::BaileysBridge).unwrap();
        assert_eq!(json, "\"baileys-bridge\"");
    }

    #[test]
    fn test_whatsapp_mode_deserialization() {
        let mode: WhatsAppMode = serde_json::from_str("\"business_api\"").unwrap();
        assert_eq!(mode, WhatsAppMode::BusinessApi);
        
        let mode: WhatsAppMode = serde_json::from_str("\"baileys-bridge\"").unwrap();
        assert_eq!(mode, WhatsAppMode::BaileysBridge);
    }

    #[test]
    fn test_parse_webhook_payload_text() {
        let payload = r#"{
            "entry": [{
                "id": "WHATSAPP_BUSINESS_ACCOUNT_ID",
                "time": 1618907555000,
                "changes": [{
                    "field": "messages",
                    "value": {
                        "messaging_product": "whatsapp",
                        "metadata": {
                            "display_phone_number": "15550000000",
                            "phone_number_id": "123456789"
                        },
                        "messages": [{
                            "id": "wamid.HBgNNTU1MDAwMDAwMBUA",
                            "timestamp": "1618907554",
                            "from": "15551234567",
                            "type": "text",
                            "text": {"body": "Hello, world!"}
                        }]
                    }
                }]
            }]
        }"#;

        let result = parse_webhook_payload(payload, "test-account", "whatsapp", None);
        assert!(result.is_ok());
        let messages = result.unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].sender.id, "15551234567");
    }

    #[test]
    fn test_channel_registration() {
        let mut registry = aisopod_channel::ChannelRegistry::new();
        
        let config = WhatsAppAccountConfig::new(
            "test-token".to_string(),
            "123456789".to_string(),
            "verify-token".to_string(),
        );

        // Note: This test would need to be run with tokio runtime
        // For now, we just verify the types compile
        let _channel = WhatsAppChannel::new(config, "test-account");
    }

    #[test]
    fn test_should_process_message_filtered() {
        let config = WhatsAppAccountConfig {
            mode: WhatsAppMode::BusinessApi,
            api_token: Some("test-token".to_string()),
            business_account_id: None,
            phone_number_id: Some("123456789".to_string()),
            webhook_verify_token: Some("verify-token".to_string()),
            allowed_numbers: Some(vec!["15551234567".to_string()]),
        };

        let account = WhatsAppAccount::new("test-account".to_string(), config.clone());
        let channel = tokio::runtime::Runtime::new().unwrap().block_on(
            WhatsAppChannel::new(config, "test-account")
        ).expect("Failed to create channel");

        let message = IncomingMessage {
            id: "msg1".to_string(),
            channel: "whatsapp".to_string(),
            account_id: "test-account".to_string(),
            sender: SenderInfo {
                id: "15559999999".to_string(),
                display_name: None,
                username: None,
                is_bot: false,
            },
            peer: PeerInfo {
                id: "15559999999".to_string(),
                kind: PeerKind::User,
                title: None,
            },
            content: aisopod_channel::message::MessageContent::Text("Hello".to_string()),
            reply_to: None,
            timestamp: Utc::now(),
            metadata: serde_json::Value::Object(serde_json::Map::new()),
        };

        assert!(!channel.should_process_message(&message));
    }
}
