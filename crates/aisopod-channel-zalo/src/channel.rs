//! Zalo channel implementation.
//!
//! This module implements the `ChannelPlugin` trait for Zalo, enabling
//! the bot to receive and send messages via the Zalo OA API.
//!
//! # Features
//!
//! - Push message sending via Zalo OA API
//! - Webhook-based event receiving
//! - OAuth authentication with token refresh
//! - Support for text, image, and file messages
//! - Multi-account support with account-specific configurations
//! - Security adapter for sender validation

use crate::api::ZaloApi;
use crate::auth::ZaloAuth;
use crate::config::ZaloConfig;
use crate::webhook::{create_webhook_router, WebhookState};

use aisopod_channel::adapters::{
    AccountConfig, AccountSnapshot, ChannelConfigAdapter, SecurityAdapter,
};
use aisopod_channel::message::{
    IncomingMessage, Media, MessageContent as ChannelMessageContent, MessagePart, MessageTarget,
    PeerInfo, PeerKind, SenderInfo,
};
use aisopod_channel::plugin::ChannelPlugin;
use aisopod_channel::types::{
    ChannelCapabilities, ChannelMeta, ChatType, MediaType as ChannelMediaType,
};
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

/// A Zalo account wraps the configuration with its state.
#[derive(Clone)]
pub struct ZaloAccount {
    /// Unique identifier for this account
    pub id: String,
    /// The account configuration
    pub config: ZaloConfig,
    /// Whether this account is currently connected
    pub connected: bool,
    /// The timestamp of the last connection
    pub last_connected: Option<DateTime<Utc>>,
}

impl ZaloAccount {
    /// Create a new ZaloAccount with the given configuration.
    pub fn new(id: String, config: ZaloConfig) -> Self {
        Self {
            id,
            config,
            connected: false,
            last_connected: None,
        }
    }

    /// Check if the account is enabled.
    pub fn is_enabled(&self) -> bool {
        // Account is enabled if it has required credentials
        !self.config.app_id.is_empty()
            && !self.config.app_secret.is_empty()
            && !self.config.refresh_token.is_empty()
    }

    /// Validate the OAuth credentials by attempting to refresh the access token.
    pub async fn validate_credentials(&self) -> Result<()> {
        let auth = ZaloAuth::new(
            self.config.app_id.clone(),
            self.config.app_secret.clone(),
            self.config.refresh_token.clone(),
        );

        // Attempt to get an access token (this will refresh if needed)
        let mut api = ZaloApi::new(auth);
        match api.get_access_token().await {
            Ok(_) => Ok(()),
            Err(e) => {
                warn!("Failed to validate Zalo credentials: {}", e);
                Err(anyhow::anyhow!(
                    "Failed to validate Zalo credentials: {}",
                    e
                ))
            }
        }
    }
}

/// Zalo channel plugin implementation.
///
/// This struct manages Zalo OA connections and webhook handling.
/// It implements the `ChannelPlugin` trait to integrate with the aisopod system.
#[derive(Clone)]
pub struct ZaloChannel {
    /// Vector of Zalo accounts
    accounts: Vec<ZaloAccount>,
    /// The channel ID
    id: String,
    /// The channel metadata
    meta: ChannelMeta,
    /// The channel capabilities
    capabilities: ChannelCapabilities,
    /// Current running tasks for graceful shutdown
    shutdown_signal: Option<Arc<tokio::sync::Notify>>,
    /// The configuration adapter
    config_adapter: ZaloChannelConfigAdapter,
    /// The security adapter
    security_adapter: Option<ZaloSecurityAdapter>,
    /// The webhook state for routing
    webhook_state: Option<WebhookState>,
}

impl ZaloChannel {
    /// Creates a new Zalo channel with the given configuration.
    ///
    /// This method validates the OAuth credentials by calling the Zalo API.
    ///
    /// # Arguments
    ///
    /// * `config` - The Zalo account configuration
    /// * `account_id` - Unique identifier for this account instance
    ///
    /// # Returns
    ///
    /// * `Ok(ZaloChannel)` - The channel if configuration is valid
    /// * `Err(anyhow::Error)` - An error if the configuration is invalid
    pub async fn new(config: ZaloConfig, account_id: &str) -> Result<Self> {
        let account = ZaloAccount::new(account_id.to_string(), config.clone());

        // Validate the OAuth credentials
        if let Err(e) = account.validate_credentials().await {
            return Err(anyhow::anyhow!(
                "Failed to validate Zalo OAuth credentials: {}",
                e
            ));
        }

        let id = format!("zalo-{}", account_id);
        let meta = ChannelMeta {
            label: "Zalo".to_string(),
            docs_url: Some("https://open.zalo.me".to_string()),
            ui_hints: serde_json::json!({
                "app_id_field": "app_id",
                "app_secret_field": "app_secret",
                "refresh_token_field": "refresh_token",
                "webhook_port_field": "webhook_port"
            }),
        };
        let capabilities = ChannelCapabilities {
            chat_types: vec![ChatType::Dm],
            supports_media: true,
            supports_reactions: false,
            supports_threads: false,
            supports_typing: false,
            supports_voice: false,
            max_message_length: Some(1000),
            supported_media_types: vec![ChannelMediaType::Image, ChannelMediaType::Document],
        };
        let accounts = vec![account];
        let config_adapter = ZaloChannelConfigAdapter::new(accounts.clone());

        // Create webhook state
        let webhook_state = Some(WebhookState {
            oa_secret_key: config.oa_secret_key.clone(),
            channel_id: id.clone(),
        });
        let security_adapter = Some(ZaloSecurityAdapter::new(accounts.clone()));

        Ok(Self {
            accounts,
            id,
            meta,
            capabilities,
            shutdown_signal: None,
            config_adapter,
            security_adapter,
            webhook_state,
        })
    }

    /// Get all configured account IDs for this channel.
    pub fn list_account_ids(&self) -> Vec<String> {
        self.accounts.iter().map(|a| a.id.clone()).collect()
    }

    /// Get an account by its ID.
    pub fn get_account(&self, account_id: &str) -> Option<&ZaloAccount> {
        self.accounts.iter().find(|a| a.id == account_id)
    }

    /// Get the webhook state.
    pub fn webhook_state(&self) -> Option<&WebhookState> {
        self.webhook_state.as_ref()
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

        let account = self
            .get_account(&target.account_id)
            .ok_or_else(|| anyhow::anyhow!("Account {} not found", target.account_id))?;

        if !account.is_enabled() {
            return Err(anyhow::anyhow!(
                "Account {} is not enabled",
                target.account_id
            ));
        }

        let auth = ZaloAuth::new(
            account.config.app_id.clone(),
            account.config.app_secret.clone(),
            account.config.refresh_token.clone(),
        );
        let mut api = ZaloApi::new(auth);

        api.send_text_message(&target.peer.id, text).await?;

        info!(
            "Sent text message to {} in channel {}",
            target.peer.id, self.id
        );

        Ok(())
    }

    /// Send an image message through this channel.
    ///
    /// # Arguments
    ///
    /// * `target` - The message target specifying where to send
    /// * `image_url` - The URL of the image to send
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message was sent successfully
    /// * `Err(anyhow::Error)` - An error if sending fails
    pub async fn send_image(&self, target: &MessageTarget, image_url: &str) -> Result<()> {
        if target.channel != self.id {
            return Err(anyhow::anyhow!(
                "Target channel {} does not match this channel {}",
                target.channel,
                self.id
            ));
        }

        let account = self
            .get_account(&target.account_id)
            .ok_or_else(|| anyhow::anyhow!("Account {} not found", target.account_id))?;

        if !account.is_enabled() {
            return Err(anyhow::anyhow!(
                "Account {} is not enabled",
                target.account_id
            ));
        }

        let auth = ZaloAuth::new(
            account.config.app_id.clone(),
            account.config.app_secret.clone(),
            account.config.refresh_token.clone(),
        );
        let mut api = ZaloApi::new(auth);

        api.send_image_message(&target.peer.id, image_url).await?;

        info!(
            "Sent image message to {} in channel {}",
            target.peer.id, self.id
        );

        Ok(())
    }

    /// Register webhook routes with the gateway router.
    ///
    /// This method sets up the webhook endpoints for receiving messages from Zalo.
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

        router.merge(create_webhook_router(
            account.config.oa_secret_key.clone(),
            self.id.clone(),
        ))
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
        // Check if the sender is in the allowed list for any account
        if let Some(account) = self.get_account(&message.account_id) {
            // For now, we process all messages from any user
            // This can be enhanced with sender filtering if needed
            return true;
        }

        // If account not found, don't process
        false
    }
}

/// ChannelConfigAdapter implementation for ZaloChannel.
#[derive(Clone)]
pub struct ZaloChannelConfigAdapter {
    /// Reference to the channel accounts
    accounts: Vec<ZaloAccount>,
}

impl ZaloChannelConfigAdapter {
    /// Create a new ZaloChannelConfigAdapter.
    pub fn new(accounts: Vec<ZaloAccount>) -> Self {
        Self { accounts }
    }
}

#[async_trait]
impl ChannelConfigAdapter for ZaloChannelConfigAdapter {
    fn list_accounts(&self) -> Result<Vec<String>> {
        Ok(self.accounts.iter().map(|a| a.id.clone()).collect())
    }

    fn resolve_account(&self, id: &str) -> Result<AccountSnapshot> {
        self.accounts
            .iter()
            .find(|a| a.id == id)
            .map(|a| AccountSnapshot {
                id: a.id.clone(),
                channel: "zalo".to_string(),
                enabled: a.is_enabled(),
                connected: a.connected,
            })
            .ok_or_else(|| anyhow::anyhow!("Account {} not found", id))
    }

    fn enable_account(&self, _id: &str) -> Result<()> {
        // In production, this would enable the account
        warn!("enable_account not yet implemented for Zalo");
        Ok(())
    }

    fn disable_account(&self, _id: &str) -> Result<()> {
        // In production, this would disable the account
        warn!("disable_account not yet implemented for Zalo");
        Ok(())
    }

    fn delete_account(&self, _id: &str) -> Result<()> {
        // In production, this would delete the account
        warn!("delete_account not yet implemented for Zalo");
        Ok(())
    }
}

/// Security adapter for ZaloChannel.
#[derive(Clone)]
pub struct ZaloSecurityAdapter {
    /// Reference to the channel accounts
    accounts: Vec<ZaloAccount>,
}

impl ZaloSecurityAdapter {
    /// Create a new ZaloSecurityAdapter.
    pub fn new(accounts: Vec<ZaloAccount>) -> Self {
        Self { accounts }
    }
}

#[async_trait]
impl SecurityAdapter for ZaloSecurityAdapter {
    fn is_allowed_sender(&self, _sender: &SenderInfo) -> bool {
        // Zalo doesn't have a built-in allowlist mechanism
        // All senders are allowed by default
        true
    }

    fn requires_mention_in_group(&self) -> bool {
        // Zalo OA doesn't require mentions in DMs
        false
    }
}

// ============================================================================
// ChannelPlugin implementation
// ============================================================================

#[async_trait]
impl ChannelPlugin for ZaloChannel {
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

/// Register a Zalo channel with the registry.
///
/// This function creates a ZaloChannel from the given configuration
/// and adds it to the channel registry.
///
/// # Arguments
///
/// * `registry` - The channel registry to register with
/// * `config` - The Zalo account configuration
/// * `account_id` - Unique identifier for this account instance
pub async fn register(
    registry: &mut aisopod_channel::ChannelRegistry,
    config: ZaloConfig,
    account_id: &str,
) -> Result<()> {
    let channel = ZaloChannel::new(config, account_id).await?;
    registry.register(Arc::new(channel));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zalo_account_config_default() {
        let config = ZaloConfig::default();
        assert!(config.app_id.is_empty());
        assert!(config.app_secret.is_empty());
        assert!(config.refresh_token.is_empty());
        assert_eq!(config.webhook_port, 8080);
    }

    #[test]
    fn test_zalo_account_config_serialization() {
        let config = ZaloConfig {
            app_id: "test_app_id".to_string(),
            app_secret: "test_app_secret".to_string(),
            refresh_token: "test_refresh_token".to_string(),
            webhook_port: 8080,
            oa_secret_key: "test_secret_key".to_string(),
            webhook_path: "/zalo/webhook".to_string(),
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("test_app_id"));
        assert!(json.contains("test_app_secret"));
    }

    #[test]
    fn test_zalo_account_is_enabled() {
        let config = ZaloConfig {
            app_id: "test_app_id".to_string(),
            app_secret: "test_app_secret".to_string(),
            refresh_token: "test_refresh_token".to_string(),
            webhook_port: 8080,
            oa_secret_key: "test_secret_key".to_string(),
            webhook_path: "/zalo/webhook".to_string(),
        };

        let account = ZaloAccount::new("test-account".to_string(), config);
        assert!(account.is_enabled());
    }

    #[test]
    fn test_zalo_account_disabled() {
        let config = ZaloConfig::default();

        let account = ZaloAccount::new("test-account".to_string(), config);
        assert!(!account.is_enabled());
    }
}
