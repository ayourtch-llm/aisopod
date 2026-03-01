//! Lark/Feishu channel plugin implementation.
//!
//! This module implements the ChannelPlugin trait for Lark/Feishu,
//! enabling aisopod to send and receive messages via the Lark API.

use aisopod_channel::adapters::{
    AccountConfig, AccountSnapshot, ChannelConfigAdapter, SecurityAdapter,
};
use aisopod_channel::message::{
    IncomingMessage, Media, MessageContent, MessagePart, MessageTarget, PeerInfo, PeerKind,
    SenderInfo,
};
use aisopod_channel::plugin::ChannelPlugin;
use aisopod_channel::types::{ChannelCapabilities, ChannelMeta, ChatType, MediaType};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

use crate::api::LarkApi;
use crate::auth::LarkAuth;
use crate::cards::MessageCard;
use crate::config::LarkConfig;

/// A Lark/Feishu account with its configuration.
#[derive(Clone)]
pub struct LarkAccount {
    /// Unique identifier for this account
    pub id: String,
    /// The account configuration
    pub config: LarkConfig,
    /// API client for this account
    pub api: Arc<Mutex<LarkApi>>,
}

impl LarkAccount {
    /// Creates a new Lark account.
    pub fn new(id: String, config: LarkConfig) -> Result<Self> {
        let auth = LarkAuth::new(
            config.app_id.clone(),
            config.app_secret.clone(),
            config.use_feishu,
        );
        let api = LarkApi::new(auth);
        Ok(Self {
            id,
            config,
            api: Arc::new(Mutex::new(api)),
        })
    }
}

/// Lark channel plugin.
///
/// This struct manages Lark connections and implements the ChannelPlugin trait.
pub struct LarkChannel {
    /// Vector of Lark accounts
    accounts: Vec<LarkAccount>,
    /// The channel ID
    id: String,
    /// The channel metadata
    meta: ChannelMeta,
    /// The channel capabilities
    capabilities: ChannelCapabilities,
    /// The configuration adapter
    config_adapter: LarkChannelConfigAdapter,
    /// The security adapter
    security_adapter: Option<LarkSecurityAdapter>,
}

impl LarkChannel {
    /// Creates a new Lark channel with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The Lark account configuration
    /// * `account_id` - Unique identifier for this account instance
    ///
    /// # Returns
    ///
    /// * `Ok(LarkChannel)` - The channel
    /// * `Err(anyhow::Error)` - An error if configuration is invalid
    pub fn new(config: LarkConfig, account_id: &str) -> Result<Self> {
        let account = LarkAccount::new(account_id.to_string(), config.clone())?;

        let id = format!("lark-{}", account_id);
        let meta = ChannelMeta {
            label: "Lark/Feishu".to_string(),
            docs_url: Some(if config.use_feishu {
                "https://www.feishu.cn".to_string()
            } else {
                "https://www.larksuite.com".to_string()
            }),
            ui_hints: serde_json::json!({
                "app_id_field": "app_id",
                "app_secret_field": "app_secret"
            }),
        };

        let capabilities = ChannelCapabilities {
            chat_types: vec![ChatType::Dm, ChatType::Group],
            supports_media: true,
            supports_reactions: true,
            supports_threads: false,
            supports_typing: true,
            supports_voice: false,
            max_message_length: Some(2000),
            supported_media_types: vec![MediaType::Image, MediaType::Document],
        };

        let accounts = vec![account];

        let config_adapter = LarkChannelConfigAdapter::new(accounts.clone());
        let security_adapter = Some(LarkSecurityAdapter::new(accounts.clone()));

        Ok(Self {
            accounts,
            id,
            meta,
            capabilities,
            config_adapter,
            security_adapter,
        })
    }

    /// Gets an account by its ID.
    pub fn get_account(&self, account_id: &str) -> Option<&LarkAccount> {
        self.accounts.iter().find(|a| a.id == account_id)
    }

    /// Gets an account by its ID (mutable).
    pub fn get_account_mut(&mut self, account_id: &str) -> Option<&mut LarkAccount> {
        self.accounts.iter_mut().find(|a| a.id == account_id)
    }

    /// Gets all active account IDs.
    pub fn get_account_ids(&self) -> Vec<String> {
        self.accounts.iter().map(|a| a.id.clone()).collect()
    }

    /// Adds a new account to the channel.
    pub fn add_account(&mut self, account: LarkAccount) {
        self.accounts.push(account);
    }

    /// Checks if a message should be processed based on security settings.
    pub fn should_process_message(&self, message: &IncomingMessage) -> bool {
        if let Some(account) = self.get_account(&message.account_id) {
            if !account.config.verification_token.is_empty() {
                // In a real implementation, check against allowed senders
                // For now, allow all messages if token is configured
                return true;
            }
        }
        false
    }

    /// Sends a text message to a chat.
    ///
    /// # Arguments
    ///
    /// * `account_id` - The account ID to use for sending
    /// * `chat_id` - The chat ID to send to
    /// * `text` - The text content to send
    pub async fn send_text(&self, account_id: &str, chat_id: &str, text: &str) -> Result<()> {
        let account = self
            .get_account(account_id)
            .ok_or_else(|| anyhow::anyhow!("Account {} not found", account_id))?;

        let mut api = account.api.lock().await;
        api.send_text(chat_id, text).await?;
        Ok(())
    }

    /// Sends a rich message card to a chat.
    ///
    /// # Arguments
    ///
    /// * `account_id` - The account ID to use for sending
    /// * `chat_id` - The chat ID to send to
    /// * `card` - The message card to send
    pub async fn send_card(
        &self,
        account_id: &str,
        chat_id: &str,
        card: MessageCard,
    ) -> Result<()> {
        let account = self
            .get_account(account_id)
            .ok_or_else(|| anyhow::anyhow!("Account {} not found", account_id))?;

        let mut api = account.api.lock().await;
        let card_json = card.to_json()?;
        api.send_card(chat_id, card_json).await?;
        Ok(())
    }
}

/// Configuration adapter for Lark channel.
pub struct LarkChannelConfigAdapter {
    accounts: Vec<LarkAccount>,
}

impl LarkChannelConfigAdapter {
    fn new(accounts: Vec<LarkAccount>) -> Self {
        Self { accounts }
    }
}

impl ChannelConfigAdapter for LarkChannelConfigAdapter {
    fn list_accounts(&self) -> Result<Vec<String>> {
        Ok(self.accounts.iter().map(|a| a.id.clone()).collect())
    }

    fn resolve_account(&self, id: &str) -> Result<AccountSnapshot> {
        let account = self
            .accounts
            .iter()
            .find(|a| a.id == id)
            .ok_or_else(|| anyhow::anyhow!("Account {} not found", id))?;

        // In a real implementation, check connection status
        Ok(AccountSnapshot {
            id: account.id.clone(),
            channel: "lark".to_string(),
            enabled: true,
            connected: true, // Simplified for demo
        })
    }

    fn enable_account(&self, _id: &str) -> Result<()> {
        // In a real implementation, enable the account
        Ok(())
    }

    fn disable_account(&self, _id: &str) -> Result<()> {
        // In a real implementation, disable the account
        Ok(())
    }

    fn delete_account(&self, _id: &str) -> Result<()> {
        // In a real implementation, delete the account
        Ok(())
    }
}

/// Security adapter for Lark channel.
pub struct LarkSecurityAdapter {
    accounts: Vec<LarkAccount>,
}

impl LarkSecurityAdapter {
    fn new(accounts: Vec<LarkAccount>) -> Self {
        Self { accounts }
    }
}

impl SecurityAdapter for LarkSecurityAdapter {
    fn is_allowed_sender(&self, _sender: &SenderInfo) -> bool {
        // In a real implementation, check against allowed senders
        true
    }

    fn requires_mention_in_group(&self) -> bool {
        // Lark requires mentioning the bot in group chats
        true
    }
}

#[async_trait]
impl ChannelPlugin for LarkChannel {
    fn id(&self) -> &str {
        &self.id
    }

    fn meta(&self) -> &ChannelMeta {
        &self.meta
    }

    fn capabilities(&self) -> &ChannelCapabilities {
        &self.capabilities
    }

    fn config(&self) -> &dyn ChannelConfigAdapter {
        &self.config_adapter
    }

    fn security(&self) -> Option<&dyn SecurityAdapter> {
        self.security_adapter
            .as_ref()
            .map(|a| a as &dyn SecurityAdapter)
    }

    async fn connect(&mut self) -> Result<()> {
        info!("Connecting to Lark channels...");
        // In a real implementation, establish WebSocket connections
        // or set up polling for each account
        Ok(())
    }

    async fn send(&self, msg: aisopod_channel::OutgoingMessage) -> Result<()> {
        debug!("Sending message: {:?}", msg);

        let account_id = &msg.target.account_id;

        // Get the account
        let account = self
            .get_account(account_id)
            .ok_or_else(|| anyhow::anyhow!("Account {} not found", account_id))?;

        let mut api = account.api.lock().await;

        // Extract chat_id from target - use peer.id for Lark
        let chat_id = msg.target.peer.id.clone();

        // Process message content
        match &msg.content {
            aisopod_channel::MessageContent::Text(text) => api.send_text(&chat_id, text).await?,
            aisopod_channel::MessageContent::Mixed(parts) => {
                for part in parts {
                    match part {
                        aisopod_channel::MessagePart::Text(text) => {
                            api.send_text(&chat_id, text).await?
                        }
                        aisopod_channel::MessagePart::Media(media) => {
                            // For now, just send a text notification
                            // In a real implementation, upload and send media
                            let url: String =
                                media.url.as_ref().map(|s| s.clone()).unwrap_or_else(|| {
                                    media
                                        .data
                                        .as_ref()
                                        .map(|v| v.len().to_string())
                                        .unwrap_or_else(|| "unknown".to_string())
                                });
                            api.send_text(&chat_id, &format!("Sent media: {}", url))
                                .await?
                        }
                    }
                }
            }
            aisopod_channel::MessageContent::Media(media) => {
                // For now, just send a text notification
                // In a real implementation, upload and send media
                let url: String = media.url.as_ref().map(|s| s.clone()).unwrap_or_else(|| {
                    media
                        .data
                        .as_ref()
                        .map(|v| v.len().to_string())
                        .unwrap_or_else(|| "unknown".to_string())
                });
                api.send_text(&chat_id, &format!("Sent media: {}", url))
                    .await?
            }
        }

        Ok(())
    }

    async fn receive(&mut self) -> Result<aisopod_channel::IncomingMessage> {
        // In a real implementation, wait for incoming messages from webhooks or polling
        Err(anyhow::anyhow!(
            "Receive is not implemented for this channel"
        ))
    }

    async fn disconnect(&mut self) -> Result<()> {
        info!("Disconnecting from Lark channels...");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lark_channel_new() {
        let config = LarkConfig {
            app_id: "test_app_id".to_string(),
            app_secret: "test_app_secret".to_string(),
            verification_token: "test_token".to_string(),
            encrypt_key: None,
            webhook_port: 8080,
            use_feishu: false,
        };

        let channel = LarkChannel::new(config, "test").unwrap();
        assert_eq!(channel.id(), "lark-test");
        assert_eq!(channel.meta().label, "Lark/Feishu");
    }

    #[test]
    fn test_lark_channel_feishu() {
        let config = LarkConfig {
            app_id: "test_app_id".to_string(),
            app_secret: "test_app_secret".to_string(),
            verification_token: "test_token".to_string(),
            encrypt_key: None,
            webhook_port: 8080,
            use_feishu: true,
        };

        let channel = LarkChannel::new(config, "test").unwrap();
        assert_eq!(channel.meta().label, "Lark/Feishu");
    }
}
