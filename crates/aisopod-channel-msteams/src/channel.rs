//! Microsoft Teams channel plugin implementation.
//!
//! This module implements the ChannelPlugin trait for Microsoft Teams,
//! enabling the bot to receive and send messages via the Microsoft Bot Framework.

use crate::botframework::{Activity, BotFrameworkClient};
use crate::config::{MsTeamsAccountConfig, MsTeamsConfig, WebhookConfig};
use crate::auth::{AzureAuthConfig, MsTeamsAuth};
use crate::webhook::{create_webhook_router, WebhookState};
use aisopod_channel::adapters::{
    AccountConfig, AccountSnapshot, ChannelConfigAdapter, SecurityAdapter,
};
use aisopod_channel::message::{IncomingMessage, MessageTarget, PeerInfo, PeerKind, SenderInfo};
use aisopod_channel::types::{ChannelCapabilities, ChannelMeta, ChatType, MediaType};
use aisopod_channel::plugin::ChannelPlugin;
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info, warn};
use axum::Router;

/// A Microsoft Teams account wraps the configuration with its state.
#[derive(Clone)]
pub struct MsTeamsAccount {
    /// Unique identifier for this account
    pub id: String,
    /// The account configuration
    pub config: MsTeamsAccountConfig,
    /// Whether this account is currently connected
    pub connected: bool,
    /// The timestamp of the last connection
    pub last_connected: Option<DateTime<Utc>>,
    /// The Bot Framework client
    pub client: Option<BotFrameworkClient>,
}

impl MsTeamsAccount {
    /// Create a new MsTeamsAccount with the given configuration.
    pub fn new(id: String, config: MsTeamsAccountConfig) -> Result<Self> {
        // Create the Bot Framework client
        let auth = MsTeamsAuth::new(AzureAuthConfig::new(
            &config.tenant_id,
            &config.client_id,
            &config.client_secret,
        ));
        let app_id = config.bot_app_id_or_client_id().to_string();
        let client = BotFrameworkClient::new(auth, &app_id);

        Ok(Self {
            id,
            config,
            connected: false,
            last_connected: None,
            client: Some(client),
        })
    }

    /// Check if the account is enabled.
    pub fn is_enabled(&self) -> bool {
        !self.config.tenant_id.is_empty()
            && !self.config.client_id.is_empty()
            && !self.config.client_secret.is_empty()
    }

    /// Get the Bot Framework client.
    pub fn client(&self) -> Option<&BotFrameworkClient> {
        self.client.as_ref()
    }

    /// Get the Bot Framework client (mutable).
    pub fn client_mut(&mut self) -> Option<&mut BotFrameworkClient> {
        self.client.as_mut()
    }
}

/// Microsoft Teams channel plugin implementation.
///
/// This struct manages Microsoft Teams connections and webhook handling.
/// It implements the `ChannelPlugin` trait to integrate with the aisopod system.
#[derive(Clone)]
pub struct MsTeamsChannel {
    /// Vector of Microsoft Teams accounts
    accounts: Vec<MsTeamsAccount>,
    /// The channel ID
    id: String,
    /// The channel metadata
    meta: ChannelMeta,
    /// The channel capabilities
    capabilities: ChannelCapabilities,
    /// Current running tasks for graceful shutdown
    shutdown_signal: Option<Arc<tokio::sync::Notify>>,
    /// The configuration adapter
    config_adapter: MsTeamsChannelConfigAdapter,
    /// The security adapter
    security_adapter: Option<MsTeamsSecurityAdapter>,
    /// Webhook configuration
    webhook_config: WebhookConfig,
}

impl MsTeamsChannel {
    /// Creates a new Microsoft Teams channel with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The Microsoft Teams account configuration
    /// * `account_id` - Unique identifier for the first account instance
    ///
    /// # Returns
    ///
    /// * `Ok(MsTeamsChannel)` - The channel if configuration is valid
    /// * `Err(anyhow::Error)` - An error if the configuration is invalid
    pub async fn new(config: MsTeamsConfig, account_id: &str) -> Result<Self> {
        // Validate that at least one account is configured
        if config.accounts.is_empty() {
            return Err(anyhow::anyhow!(
                "Failed to validate Microsoft Teams channel: at least one account is required"
            ));
        }

        // Create accounts from all configurations in the config
        let mut accounts = Vec::new();
        
        // Create the first account using the provided account_id
        if let Some(first_config) = config.accounts.get(0).cloned() {
            let account = MsTeamsAccount::new(account_id.to_string(), first_config)?;
            accounts.push(account);
        }
        
        // Create additional accounts from remaining configs
        for (i, account_config) in config.accounts.iter().enumerate().skip(1) {
            // Use the index as the account ID for additional accounts
            let account_id = format!("account_{}", i);
            let account = MsTeamsAccount::new(account_id, account_config.clone())?;
            accounts.push(account);
        }

        // Validate that at least one account is enabled
        let has_enabled_account = accounts.iter().any(|a| a.is_enabled());
        if !has_enabled_account {
            return Err(anyhow::anyhow!(
                "Failed to validate Microsoft Teams account: no valid authentication configured"
            ));
        }

        let id = format!("msteams-{}", account_id);
        let meta = ChannelMeta {
            label: "Microsoft Teams".to_string(),
            docs_url: Some("https://learn.microsoft.com/en-us/microsoftteams/platform/bots/what-are-bots".to_string()),
            ui_hints: serde_json::json!({
                "tenant_id_field": "tenant_id",
                "client_id_field": "client_id",
                "client_secret_field": "client_secret",
                "bot_app_id_field": "bot_app_id"
            }),
        };
        let capabilities = ChannelCapabilities {
            chat_types: vec![
                ChatType::Dm,
                ChatType::Group,
                ChatType::Channel,
            ],
            supports_media: true,
            supports_reactions: true,
            supports_threads: true,
            supports_typing: true,
            supports_voice: false,
            max_message_length: Some(25000),
            supported_media_types: vec![
                MediaType::Image,
                MediaType::Audio,
                MediaType::Video,
                MediaType::Document,
            ],
        };
        let config_adapter = MsTeamsChannelConfigAdapter::new(accounts.clone());
        let security_adapter = Some(MsTeamsSecurityAdapter::new(accounts.clone()));

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

    /// Creates a new Microsoft Teams channel in webhook mode.
    ///
    /// # Arguments
    ///
    /// * `config` - The Microsoft Teams account configuration with webhook settings
    /// * `account_id` - Unique identifier for this account instance
    ///
    /// # Returns
    ///
    /// * `Ok(MsTeamsChannel)` - The channel if configuration is valid
    /// * `Err(anyhow::Error)` - An error if the configuration is invalid
    pub async fn new_webhook(config: MsTeamsConfig, account_id: &str) -> Result<Self> {
        let webhook_enabled = config.webhook.enabled;
        if !webhook_enabled {
            return Err(anyhow::anyhow!("Webhook must be enabled for webhook mode"));
        }

        Self::new(config, account_id).await
    }

    /// Get an account by its ID.
    pub fn get_account(&self, account_id: &str) -> Option<&MsTeamsAccount> {
        self.accounts.iter().find(|a| a.id == account_id)
    }

    /// Get an account by its ID (mutable).
    pub fn get_account_mut(&mut self, account_id: &str) -> Option<&mut MsTeamsAccount> {
        self.accounts.iter_mut().find(|a| a.id == account_id)
    }

    /// Get all active account IDs.
    pub fn get_account_ids(&self) -> Vec<String> {
        self.accounts.iter().map(|a| a.id.clone()).collect()
    }

    /// Add a new account to the channel.
    pub fn add_account(&mut self, account: MsTeamsAccount) {
        self.accounts.push(account);
        self.config_adapter.accounts = self.accounts.clone();
        self.security_adapter = Some(MsTeamsSecurityAdapter::new(self.accounts.clone()));
    }

    /// Remove an account by its ID.
    pub fn remove_account(&mut self, account_id: &str) -> bool {
        let len = self.accounts.len();
        self.accounts.retain(|a| a.id != account_id);
        self.config_adapter.accounts = self.accounts.clone();
        self.security_adapter = Some(MsTeamsSecurityAdapter::new(self.accounts.clone()));
        len != self.accounts.len()
    }

    /// Start receiving messages using long-polling mode.
    ///
    /// This spawns a background task that continuously polls for new activities
    /// and processes them.
    ///
    /// # Arguments
    ///
    /// * `account_id` - The account ID for this channel instance (optional, if None, polls all accounts)
    ///
    /// # Returns
    ///
    /// A handle to the background task that can be awaited or cancelled.
    pub async fn start_long_polling(
        &mut self,
        account_id: Option<&str>,
    ) -> Result<impl std::future::Future<Output = ()> + Send> {
        let accounts_to_poll: Vec<MsTeamsAccount> = match account_id {
            Some(id) => {
                self.get_account(id)
                    .cloned()
                    .map(|a| vec![a])
                    .unwrap_or_default()
            }
            None => self.accounts.clone(),
        };

        if accounts_to_poll.is_empty() {
            return Err(anyhow::anyhow!("No accounts found to poll"));
        }

        // Create shutdown signal
        let shutdown = Arc::new(tokio::sync::Notify::new());
        self.shutdown_signal = Some(shutdown.clone());

        // Create a clone of shutdown for the task
        let shutdown_task = shutdown.clone();

        let mut accounts_to_poll = self.accounts.clone();

        let task = async move {
            info!("Starting long-polling for Microsoft Teams channel");

            loop {
                tokio::select! {
                    biased;
                    _ = shutdown_task.notified() => {
                        info!("Shutdown signal received for Microsoft Teams channel");
                        break;
                    }
                    _ = async {
                        for account in &mut accounts_to_poll {
                            if let Some(client) = &mut account.client {
                                // In a real implementation, this would poll the Bot Framework API
                                // for new activities
                                warn!("Long-polling requires Bot Framework API polling implementation");
                            }
                        }
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    } => {}
                }
            }
        };

        Ok(task)
    }

    /// Start receiving messages using webhook mode.
    ///
    /// This spawns a background task that listens for incoming webhook requests.
    /// Note: The HTTP server setup must be handled by the caller (typically in aisopod-gateway).
    ///
    /// # Arguments
    ///
    /// * `account_id` - The account ID for this channel instance
    /// * `port` - The port to listen on for webhook requests
    ///
    /// # Returns
    ///
    /// A handle to the background task that can be awaited or cancelled.
    pub async fn start_webhook(
        &mut self,
        account_id: &str,
        port: u16,
    ) -> Result<impl std::future::Future<Output = ()> + Send> {
        // Find the account
        let account = self.get_account_mut(account_id)
            .ok_or_else(|| anyhow::anyhow!("Account not found: {}", account_id))?;

        // Get webhook configuration
        let microsoft_app_id = account.config.bot_app_id_or_client_id().to_string();
        let microsoft_app_password = account.config.bot_app_password.clone().unwrap_or_default();

        let client = account.client.clone().ok_or_else(|| {
            anyhow::anyhow!("Bot Framework client not initialized for account {}", account_id)
        })?;

        let webhook_state = WebhookState::new(client, account_id, &microsoft_app_id, &microsoft_app_password);
        let webhook_state = Arc::new(webhook_state);

        // Create shutdown signal if not already created
        let shutdown = self.shutdown_signal.get_or_insert_with(|| Arc::new(tokio::sync::Notify::new()));
        let shutdown_task = shutdown.clone();

        let task = async move {
            info!("Starting webhook listener for Microsoft Teams channel on port {}", port);

            // In a real implementation, this would set up an HTTP server
            // to receive webhook POST requests from Microsoft Teams.
            // For now, we provide a placeholder that demonstrates the structure.
            warn!("Webhook mode setup requires HTTP server integration (e.g., aisopod-gateway)");

            // Keep the task alive until shutdown
            shutdown_task.notified().await;
            info!("Webhook listener stopped");
        };

        Ok(task)
    }

    /// Stop receiving messages gracefully.
    pub async fn stop(&mut self) {
        if let Some(shutdown) = &self.shutdown_signal {
            shutdown.notify_one();
        }
    }

    /// Send a message to the specified target.
    pub async fn send_message(
        &mut self,
        target: &MessageTarget,
        text: &str,
    ) -> Result<()> {
        // Find the account
        let account = self.get_account_mut(&target.account_id)
            .ok_or_else(|| anyhow::anyhow!("Account not found: {}", target.account_id))?;

        let client = account.client.as_mut()
            .ok_or_else(|| anyhow::anyhow!("Bot Framework client not initialized"))?;

        // Extract conversation ID from peer
        let conversation_id = &target.peer.id;

        // Send the message
        client.send_message(conversation_id, text, None).await?;

        Ok(())
    }

    /// Send media content to the specified target.
    pub async fn send_media(
        &mut self,
        target: &MessageTarget,
        media: &aisopod_channel::message::Media,
    ) -> Result<()> {
        // Find the account
        let account = self.get_account_mut(&target.account_id)
            .ok_or_else(|| anyhow::anyhow!("Account not found: {}", target.account_id))?;

        let client = account.client.as_mut()
            .ok_or_else(|| anyhow::anyhow!("Bot Framework client not initialized"))?;

        // Extract conversation ID from peer
        let conversation_id = &target.peer.id;

        // Convert media to attachment
        let attachment = self.media_to_attachment(media)?;

        // In a real implementation, this would send the attachment
        warn!("Media sending requires Bot Framework attachment implementation");

        Ok(())
    }

    /// Convert media to Bot Framework attachment.
    fn media_to_attachment(
        &self,
        media: &aisopod_channel::message::Media,
    ) -> Result<aisopod_channel::message::Media> {
        // This is a placeholder - in a real implementation, this would
        // convert the media to the appropriate format for Bot Framework
        Ok(media.clone())
    }

    /// Create a webhook router for the specified account.
    pub fn create_webhook_router(&self, account_id: &str) -> Option<Router<Arc<WebhookState>>> {
        let account = self.get_account(account_id)?;
        
        let client = account.client.clone()?;
        let microsoft_app_id = account.config.bot_app_id_or_client_id().to_string();
        let microsoft_app_password = account.config.bot_app_password.clone().unwrap_or_default();

        let webhook_state = WebhookState::new(client, account_id, &microsoft_app_id, &microsoft_app_password);
        Some(create_webhook_router(webhook_state))
    }
}

impl aisopod_channel::plugin::ChannelPlugin for MsTeamsChannel {
    fn id(&self) -> &str {
        &self.id
    }

    fn meta(&self) -> &ChannelMeta {
        &self.meta
    }

    fn capabilities(&self) -> &ChannelCapabilities {
        &self.capabilities
    }

    fn config(&self) -> &dyn aisopod_channel::adapters::ChannelConfigAdapter {
        &self.config_adapter
    }

    fn security(&self) -> Option<&dyn aisopod_channel::adapters::SecurityAdapter> {
        self.security_adapter.as_ref().map(|adapter| adapter as &dyn aisopod_channel::adapters::SecurityAdapter)
    }
}

/// Configuration adapter for Microsoft Teams channel.
#[derive(Clone)]
pub struct MsTeamsChannelConfigAdapter {
    accounts: Vec<MsTeamsAccount>,
}

impl MsTeamsChannelConfigAdapter {
    /// Creates a new configuration adapter.
    pub fn new(accounts: Vec<MsTeamsAccount>) -> Self {
        Self { accounts }
    }
}

impl ChannelConfigAdapter for MsTeamsChannelConfigAdapter {
    fn list_accounts(&self) -> Result<Vec<String>> {
        Ok(self.accounts.iter().map(|a| a.id.clone()).collect())
    }

    fn resolve_account(&self, id: &str) -> Result<AccountSnapshot> {
        let account = self.accounts.iter().find(|a| a.id == id)
            .ok_or_else(|| anyhow::anyhow!("Account not found: {}", id))?;

        Ok(AccountSnapshot {
            id: account.id.clone(),
            channel: "msteams".to_string(),
            enabled: account.is_enabled(),
            connected: account.connected,
        })
    }

    fn enable_account(&self, _id: &str) -> Result<()> {
        // In a real implementation, this would enable the account
        warn!("enable_account is not implemented");
        Ok(())
    }

    fn disable_account(&self, _id: &str) -> Result<()> {
        // In a real implementation, this would disable the account
        warn!("disable_account is not implemented");
        Ok(())
    }

    fn delete_account(&self, _id: &str) -> Result<()> {
        // In a real implementation, this would delete the account
        warn!("delete_account is not implemented");
        Ok(())
    }
}

/// Security adapter for Microsoft Teams channel.
#[derive(Clone)]
pub struct MsTeamsSecurityAdapter {
    accounts: Vec<MsTeamsAccount>,
}

impl MsTeamsSecurityAdapter {
    /// Creates a new security adapter.
    pub fn new(accounts: Vec<MsTeamsAccount>) -> Self {
        Self { accounts }
    }
}

impl SecurityAdapter for MsTeamsSecurityAdapter {
    fn is_allowed_sender(&self, sender: &aisopod_channel::message::SenderInfo) -> bool {
        // Check if all accounts allow this sender (strict mode)
        // If any account has restrictions, the user must be allowed by that account
        for account in &self.accounts {
            if !account.is_user_allowed(&sender.id) {
                return false;  // At least one account doesn't allow this user
            }
        }
        true  // All accounts allow this user (or no restrictions on any account)
    }

    fn requires_mention_in_group(&self) -> bool {
        // Microsoft Teams requires bots to be mentioned in group chats
        true
    }
}

/// Extension methods for checking allowed users.
impl MsTeamsAccount {
    /// Check if a user ID is allowed.
    fn is_user_allowed(&self, user_id: &str) -> bool {
        self.config.allowed_users.is_empty() || self.config.allowed_users.contains(&user_id.to_string())
    }

    /// Check if a channel ID is allowed.
    fn is_channel_allowed(&self, channel_id: &str) -> bool {
        self.config.allowed_channels.is_empty() || self.config.allowed_channels.contains(&channel_id.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ms_teams_account_creation() {
        let config = MsTeamsAccountConfig::new("test", "tenant123", "client123", "secret123");
        let account = MsTeamsAccount::new("account1".to_string(), config);

        assert!(account.is_ok());
        let account = account.unwrap();
        assert_eq!(account.id, "account1");
        assert!(account.is_enabled());
    }

    #[test]
    fn test_ms_teams_channel_creation() {
        let config = MsTeamsConfig {
            accounts: vec![MsTeamsAccountConfig::new("test", "tenant123", "client123", "secret123")],
            ..Default::default()
        };

        let channel_result = tokio::runtime::Runtime::new().unwrap().block_on(MsTeamsChannel::new(config, "test1"));
        
        assert!(channel_result.is_ok());
    }

    #[test]
    fn test_ms_teams_channel_validation_error() {
        let config = MsTeamsConfig::default();

        let channel_result = tokio::runtime::Runtime::new().unwrap().block_on(MsTeamsChannel::new(config, "test1"));
        
        assert!(channel_result.is_err());
    }

    #[test]
    fn test_get_account() {
        let config = MsTeamsConfig {
            accounts: vec![MsTeamsAccountConfig::new("test", "tenant123", "client123", "secret123")],
            ..Default::default()
        };

        let mut channel = tokio::runtime::Runtime::new().unwrap().block_on(MsTeamsChannel::new(config, "test1")).unwrap();
        
        let account = channel.get_account("test1");
        assert!(account.is_some());
    }

    #[test]
    fn test_remove_account() {
        let config = MsTeamsConfig {
            accounts: vec![MsTeamsAccountConfig::new("test", "tenant123", "client123", "secret123")],
            ..Default::default()
        };

        let mut channel = tokio::runtime::Runtime::new().unwrap().block_on(MsTeamsChannel::new(config, "test1")).unwrap();
        
        let removed = channel.remove_account("test1");
        assert!(removed);
        assert!(channel.get_account("test1").is_none());
    }

    #[test]
    fn test_security_adapter() {
        let config = MsTeamsAccountConfig::new("test", "tenant123", "client123", "secret123");
        let accounts = vec![MsTeamsAccount::new("account1".to_string(), config).unwrap()];
        let adapter = MsTeamsSecurityAdapter::new(accounts);

        let sender = SenderInfo {
            id: "user1".to_string(),
            display_name: Some("User One".to_string()),
            username: Some("user1".to_string()),
            is_bot: false,
        };

        // Should be allowed since no restrictions
        assert!(adapter.is_allowed_sender(&sender));
    }
}

// Unit tests for auth module
#[cfg(test)]
mod auth_tests {
    use crate::auth::*;

    #[test]
    fn test_auth_config_creation() {
        let config = AzureAuthConfig::new("tenant123", "client123", "secret123");
        assert_eq!(config.tenant_id, "tenant123");
        assert_eq!(config.client_id, "client123");
        assert_eq!(config.client_secret, "secret123");
    }

    #[test]
    fn test_auth_config_with_resource() {
        let config = AzureAuthConfig::with_resource("tenant123", "client123", "secret123", "custom-resource");
        assert_eq!(config.resource, Some("custom-resource".to_string()));
    }

    #[test]
    fn test_auth_from_account_config() {
        let account_config = crate::config::MsTeamsAccountConfig::new("test", "tenant123", "client123", "secret123");
        let _auth = MsTeamsAuth::from_account_config(&account_config);

        // Basic test to verify auth was created
    }
}

// Unit tests for botframework module
#[cfg(test)]
mod botframework_tests {
    use crate::botframework::*;

    #[test]
    fn test_activity_create_message() {
        let activity = Activity::create_message("Hello, World!", None);
        assert_eq!(activity.activity_type, Some(ActivityType::Message));
        assert_eq!(activity.text, Some("Hello, World!".to_string()));
        assert!(activity.reply_to_id.is_none());
    }

    #[test]
    fn test_activity_create_message_with_reply() {
        let activity = Activity::create_message("Hello, World!", Some("reply123"));
        assert_eq!(activity.reply_to_id, Some("reply123".to_string()));
    }

    #[test]
    fn test_activity_create_typing() {
        let activity = Activity::create_typing();
        assert_eq!(activity.activity_type, Some(ActivityType::Typing));
    }

    #[test]
    fn test_activity_create_conversation_update() {
        let activity = Activity::create_conversation_update(
            "membersAdded",
            vec![ChannelAccount {
                id: Some("user1".to_string()),
                name: Some("User One".to_string()),
                role: Some("user".to_string()),
                ..Default::default()
            }],
        );
        assert_eq!(activity.activity_type, Some(ActivityType::ConversationUpdate));
        assert_eq!(activity.action, Some("membersAdded".to_string()));
    }

    #[test]
    fn test_activity_create_event() {
        let activity = Activity::create_event("testEvent", serde_json::json!({"key": "value"}));
        assert_eq!(activity.activity_type, Some(ActivityType::Event));
        assert_eq!(activity.name, Some("testEvent".to_string()));
        assert_eq!(activity.value, Some(serde_json::json!({"key": "value"})));
    }
}

// Unit tests for adaptive_cards module
#[cfg(test)]
mod adaptive_cards_tests {
    use crate::adaptive_cards::*;

    #[test]
    fn test_adaptive_card_creation() {
        let card = AdaptiveCard::new();
        assert_eq!(card.card_type, "AdaptiveCard");
        assert_eq!(card.version, "1.6");
    }

    #[test]
    fn test_adaptive_card_with_body() {
        let card = AdaptiveCard::new()
            .with_body(vec![
                AdaptiveCardElement::TextBlock(TextBlock::new("Hello").with_weight("bolder")),
                AdaptiveCardElement::TextBlock(TextBlock::new("World").with_wrap(true)),
            ]);

        let body = card.body.unwrap();
        assert_eq!(body.len(), 2);
    }

    #[test]
    fn test_text_block_creation() {
        let block = TextBlock::new("Test Text")
            .with_size("large")
            .with_weight("bolder")
            .with_color("accent");

        assert_eq!(block.text, "Test Text");
        assert_eq!(block.size, Some("large".to_string()));
        assert_eq!(block.weight, Some("bolder".to_string()));
        assert_eq!(block.color, Some("accent".to_string()));
    }

    #[test]
    fn test_image_creation() {
        let image = Image::new("https://example.com/image.png")
            .with_alt_text("Test Image")
            .with_size("medium");

        assert_eq!(image.url, "https://example.com/image.png");
        assert_eq!(image.alt_text, Some("Test Image".to_string()));
        assert_eq!(image.size, Some("medium".to_string()));
    }

    #[test]
    fn test_submit_action_creation() {
        let action = SubmitAction::new("Submit")
            .with_style("positive")
            .with_data(serde_json::json!({"action": "submit"}));

        assert_eq!(action.title, "Submit");
        assert_eq!(action.action_type, "Action.Submit");
        assert_eq!(action.style, Some("positive".to_string()));
        assert!(action.data.is_some());
    }

    #[test]
    fn test_helper_create_message_card() {
        let card = helpers::create_message_card("Project Update", "All tasks completed!");
        let body = card.body.unwrap();

        assert_eq!(body.len(), 2);
        if let AdaptiveCardElement::TextBlock(block) = &body[0] {
            assert_eq!(block.text, "Project Update");
            assert_eq!(block.weight, Some("bolder".to_string()));
        }
    }

    #[test]
    fn test_helper_create_action_card() {
        let card = helpers::create_action_card("Confirm Action", "Are you sure?");
        let body = card.body.unwrap();

        assert_eq!(body.len(), 3);
        assert!(card.actions.is_some());

        let actions = card.actions.unwrap();
        assert_eq!(actions.len(), 2);
    }
}
