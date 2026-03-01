//! Matrix channel plugin implementation for aisopod.
//!
//! This module implements the ChannelPlugin trait for Matrix,
//! enabling the bot to receive and send messages via the Matrix protocol.
//!
//! # Features
//!
//! - Connection to Matrix homeserver
//! - Password, access token, and SSO authentication
//! - Room and direct message support
//! - End-to-end encryption (optional)
//! - Message filtering by allowed users
//! - Group mention requirements
//!
//! # Example
//!
//! ```no_run
//! use aisopod_channel_matrix::{MatrixAccountConfig, MatrixChannel, register};
//! use aisopod_channel::ChannelRegistry;
//! use std::path::PathBuf;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), anyhow::Error> {
//!     let mut registry = ChannelRegistry::new();
//!
//!     let config = MatrixAccountConfig {
//!         homeserver_url: "https://matrix.org".to_string(),
//!         auth: aisopod_channel_matrix::MatrixAuth::Password {
//!             username: "bot".to_string(),
//!             password: "password".to_string(),
//!         },
//!         enable_e2ee: true,
//!         rooms: vec!["!room:matrix.org".to_string()],
//!         state_store_path: Some(PathBuf::from("/tmp/matrix-state")),
//!         ..Default::default()
//!     };
//!
//!     register(&mut registry, config, "matrix-main").await?;
//!
//!     Ok(())
//! }
//! ```

use crate::client::MatrixClient;
use crate::config::{MatrixAccountConfig, MatrixAuth};
use crate::encryption::setup_e2ee;
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
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

// Re-export common types - removed duplicate imports (already imported above)

/// A Matrix account wraps the client with its configuration and state.
#[derive(Clone)]
pub struct MatrixAccount {
    /// Unique identifier for this account
    pub id: String,
    /// The account configuration
    pub config: MatrixAccountConfig,
    /// The Matrix client
    pub client: Option<MatrixClient>,
    /// Whether this account is currently connected
    pub connected: bool,
    /// The timestamp of the last connection
    pub last_connected: Option<DateTime<Utc>>,
    /// Map of room IDs to room names
    pub room_names: Arc<std::sync::Mutex<HashMap<String, String>>>,
    /// Incoming message queue for async processing
    pub incoming_queue: Arc<std::sync::Mutex<Vec<IncomingMessage>>>,
}

impl MatrixAccount {
    /// Create a new MatrixAccount with the given configuration.
    pub fn new(id: String, config: MatrixAccountConfig) -> Self {
        Self {
            id,
            config,
            client: None,
            connected: false,
            last_connected: None,
            room_names: Arc::new(std::sync::Mutex::new(HashMap::new())),
            incoming_queue: Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    /// Check if the account is enabled.
    pub fn is_enabled(&self) -> bool {
        !self.config.homeserver_url.is_empty()
    }

    /// Validate the account configuration.
    pub fn validate(&self) -> Result<()> {
        if self.config.homeserver_url.is_empty() {
            return Err(anyhow::anyhow!("Homeserver URL cannot be empty"));
        }

        // Validate homeserver URL format
        url::Url::parse(&self.config.homeserver_url).map_err(|e| {
            anyhow::anyhow!(
                "Invalid homeserver URL {}: {}",
                self.config.homeserver_url,
                e
            )
        })?;

        // Validate password authentication
        if let MatrixAuth::Password { username, password } = &self.config.auth {
            if username.is_empty() {
                return Err(anyhow::anyhow!(
                    "Username cannot be empty for password authentication"
                ));
            }
            if password.is_empty() {
                return Err(anyhow::anyhow!(
                    "Password cannot be empty for password authentication"
                ));
            }
        }

        Ok(())
    }
}

/// Matrix channel plugin implementation.
///
/// This struct manages Matrix connections and implements the `ChannelPlugin` trait.
pub struct MatrixChannel {
    /// Vector of Matrix accounts
    accounts: Vec<MatrixAccount>,
    /// The channel ID
    id: String,
    /// The channel metadata
    meta: ChannelMeta,
    /// The channel capabilities
    capabilities: ChannelCapabilities,
    /// Current running tasks for graceful shutdown
    shutdown_signal: Option<Arc<tokio::sync::Notify>>,
    /// The configuration adapter
    config_adapter: MatrixChannelConfigAdapter,
    /// The security adapter
    security_adapter: Option<MatrixSecurityAdapter>,
}

impl MatrixChannel {
    /// Creates a new Matrix channel with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The Matrix account configuration
    /// * `account_id` - Unique identifier for this account instance
    ///
    /// # Returns
    ///
    /// * `Ok(MatrixChannel)` - The channel if configuration is valid
    /// * `Err(anyhow::Error)` - An error if the configuration is invalid
    pub async fn new(config: MatrixAccountConfig, account_id: &str) -> Result<Self> {
        let account = MatrixAccount::new(account_id.to_string(), config.clone());

        // Validate the configuration
        if let Err(e) = account.validate() {
            return Err(anyhow::anyhow!("Failed to validate Matrix account: {}", e));
        }

        let id = format!("matrix-{}", account_id);
        let meta = ChannelMeta {
            label: "Matrix".to_string(),
            docs_url: Some("https://matrix.org".to_string()),
            ui_hints: serde_json::json!({
                "homeserver_url_field": "homeserver_url",
                "auth_field": "auth"
            }),
        };
        let capabilities = ChannelCapabilities {
            chat_types: vec![ChatType::Dm, ChatType::Group, ChatType::Channel],
            supports_media: true,
            supports_reactions: true,
            supports_threads: true,
            supports_typing: true,
            supports_voice: false,
            max_message_length: Some(65536),
            supported_media_types: vec![
                MediaType::Image,
                MediaType::Audio,
                MediaType::Video,
                MediaType::Document,
            ],
        };
        let accounts = vec![account];

        // Initialize adapters
        let config_adapter = MatrixChannelConfigAdapter::new(accounts.clone());
        let security_adapter = Some(MatrixSecurityAdapter::new(accounts.clone()));

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
    pub fn get_account(&self, account_id: &str) -> Option<&MatrixAccount> {
        self.accounts.iter().find(|a| a.id == account_id)
    }

    /// Get an account by its ID (mutable).
    pub fn get_account_mut(&mut self, account_id: &str) -> Option<&mut MatrixAccount> {
        self.accounts.iter_mut().find(|a| a.id == account_id)
    }

    /// Connect to the Matrix homeserver for all accounts.
    pub async fn connect_all(&mut self) -> Result<()> {
        for account_id in self.list_account_ids() {
            let account = self.get_account(&account_id).unwrap();
            info!("Connecting Matrix account {}", account_id);

            // Connect to homeserver
            let client = MatrixClient::connect(&account.config).await?;

            // Join configured rooms
            if !account.config.rooms.is_empty() {
                info!("Joining {} rooms", account.config.rooms.len());
                client.join_rooms(&account.config.rooms).await?;
                info!("Successfully joined all configured rooms");
            }

            // Setup E2EE if enabled
            if account.config.enable_e2ee {
                info!("Setting up end-to-end encryption");
                let e2ee_config = crate::encryption::E2EEConfig {
                    state_store_path: account
                        .config
                        .state_store_path
                        .as_ref()
                        .map(|p| p.to_string_lossy().to_string()),
                    ..Default::default()
                };
                setup_e2ee(&client.client, &e2ee_config).await?;
            }

            // Update account state
            if let Some(acc) = self.get_account_mut(&account_id) {
                acc.client = Some(client);
                acc.connected = true;
                acc.last_connected = Some(Utc::now());
            }
        }

        Ok(())
    }

    /// Disconnect from the Matrix homeserver for all accounts.
    pub async fn disconnect_all(&mut self) -> Result<()> {
        for account_id in self.list_account_ids() {
            info!("Disconnecting Matrix account {}", account_id);
            if let Some(acc) = self.get_account_mut(&account_id) {
                acc.connected = false;
                acc.client = None;
            }
        }
        Ok(())
    }

    /// Check if a message should be processed based on security settings.
    pub fn should_process_message(&self, message: &IncomingMessage) -> bool {
        // Check if the sender is in the allowed list
        if let Some(account) = self.get_account(&message.account_id) {
            if !account.config.allowed_users.is_empty() {
                if !account.config.allowed_users.contains(&message.sender.id) {
                    warn!(
                        "Message from {} filtered (not in allowed list)",
                        message.sender.id
                    );
                    return false;
                }
            }
        }

        true
    }
}

/// ChannelConfigAdapter implementation for MatrixChannel.
#[derive(Clone)]
pub struct MatrixChannelConfigAdapter {
    /// Reference to the channel accounts
    accounts: Vec<MatrixAccount>,
}

impl MatrixChannelConfigAdapter {
    /// Create a new MatrixChannelConfigAdapter.
    pub fn new(accounts: Vec<MatrixAccount>) -> Self {
        Self { accounts }
    }
}

#[async_trait]
impl ChannelConfigAdapter for MatrixChannelConfigAdapter {
    fn list_accounts(&self) -> Result<Vec<String>> {
        Ok(self.accounts.iter().map(|a| a.id.clone()).collect())
    }

    fn resolve_account(&self, id: &str) -> Result<AccountSnapshot> {
        self.accounts
            .iter()
            .find(|a| a.id == id)
            .map(|a| AccountSnapshot {
                id: a.id.clone(),
                channel: "matrix".to_string(),
                enabled: a.is_enabled(),
                connected: a.connected,
            })
            .ok_or_else(|| anyhow::anyhow!("Account {} not found", id))
    }

    fn enable_account(&self, id: &str) -> Result<()> {
        info!("Enabling Matrix account {}", id);
        // In production, this would enable the account
        Ok(())
    }

    fn disable_account(&self, id: &str) -> Result<()> {
        info!("Disabling Matrix account {}", id);
        // In production, this would disable the account
        Ok(())
    }

    fn delete_account(&self, id: &str) -> Result<()> {
        info!("Deleting Matrix account {}", id);
        // In production, this would delete the account
        Ok(())
    }
}

/// Security adapter for MatrixChannel.
#[derive(Clone)]
pub struct MatrixSecurityAdapter {
    /// Reference to the channel accounts
    accounts: Vec<MatrixAccount>,
}

impl MatrixSecurityAdapter {
    /// Create a new MatrixSecurityAdapter.
    pub fn new(accounts: Vec<MatrixAccount>) -> Self {
        Self { accounts }
    }
}

#[async_trait]
impl SecurityAdapter for MatrixSecurityAdapter {
    fn is_allowed_sender(&self, sender: &SenderInfo) -> bool {
        // Check if the sender is in the allowed list for any account
        for account in &self.accounts {
            if account.config.allowed_users.is_empty() {
                // If no allowed list is configured, allow all senders
                return true;
            }
            if account.config.allowed_users.contains(&sender.id) {
                return true;
            }
        }
        false
    }

    fn requires_mention_in_group(&self) -> bool {
        // Check if any account requires mentions in groups
        self.accounts
            .iter()
            .any(|a| a.config.requires_mention_in_group)
    }
}

// ============================================================================
// ChannelPlugin implementation
// ============================================================================

#[async_trait]
impl ChannelPlugin for MatrixChannel {
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

/// Register a Matrix channel with the registry.
///
/// This function creates a MatrixChannel from the given configuration
/// and adds it to the channel registry.
///
/// # Arguments
///
/// * `registry` - The channel registry to register with
/// * `config` - The Matrix account configuration
/// * `account_id` - Unique identifier for this account instance
pub async fn register(
    registry: &mut aisopod_channel::ChannelRegistry,
    config: MatrixAccountConfig,
    account_id: &str,
) -> Result<()> {
    let channel = MatrixChannel::new(config, account_id).await?;
    registry.register(Arc::new(channel));
    Ok(())
}

// ============================================================================
// Message conversion utilities
// ============================================================================

/// Converts a Matrix room message to an IncomingMessage.
pub fn matrix_event_to_incoming_message(
    account_id: &str,
    room_id: &str,
    room_name: Option<String>,
    sender: &str,
    display_name: Option<String>,
    username: Option<String>,
    content: &str,
    timestamp: DateTime<Utc>,
) -> IncomingMessage {
    let sender_info = SenderInfo {
        id: sender.to_string(),
        display_name,
        username,
        is_bot: false,
    };

    let peer_info = PeerInfo {
        id: room_id.to_string(),
        kind: if room_name.is_some() {
            PeerKind::Group
        } else {
            PeerKind::User
        },
        title: room_name,
    };

    let content = MessageContent::Text(content.to_string());

    IncomingMessage {
        id: Uuid::new_v4().to_string(),
        channel: format!("matrix-{}", account_id),
        account_id: account_id.to_string(),
        sender: sender_info,
        peer: peer_info,
        content,
        reply_to: None,
        timestamp,
        metadata: serde_json::json!({
            "room_id": room_id
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_account_config_default() {
        let config = MatrixAccountConfig::default();
        assert_eq!(config.homeserver_url, "https://matrix.org");
        assert!(config.enable_e2ee);
    }

    #[tokio::test]
    async fn test_matrix_channel_id() {
        let config = MatrixAccountConfig {
            homeserver_url: "https://matrix.org".to_string(),
            auth: MatrixAuth::Password {
                username: "testuser".to_string(),
                password: "testpass".to_string(),
            },
            ..Default::default()
        };
        let channel = MatrixChannel::new(config, "test")
            .await
            .expect("Failed to create channel");
        assert_eq!(channel.id(), "matrix-test");
    }

    #[test]
    fn test_matrix_account_validation() {
        let mut config = MatrixAccountConfig::default();
        config.homeserver_url = String::new();

        let account = MatrixAccount::new("test".to_string(), config);
        assert!(account.validate().is_err());
    }

    #[test]
    fn test_matrix_account_validation_valid() {
        let config = MatrixAccountConfig {
            homeserver_url: "https://matrix.org".to_string(),
            auth: MatrixAuth::Password {
                username: "testuser".to_string(),
                password: "testpass".to_string(),
            },
            ..Default::default()
        };
        let account = MatrixAccount::new("test".to_string(), config);
        assert!(account.validate().is_ok());
    }

    #[test]
    fn test_matrix_security_adapter_allows_allowed_sender() {
        let config = MatrixAccountConfig {
            allowed_users: vec!["@allowed:matrix.org".to_string()],
            ..Default::default()
        };
        let account = MatrixAccount::new("test".to_string(), config);
        let adapter = MatrixSecurityAdapter::new(vec![account]);

        let sender = SenderInfo {
            id: "@allowed:matrix.org".to_string(),
            ..Default::default()
        };

        assert!(adapter.is_allowed_sender(&sender));
    }

    #[test]
    fn test_matrix_security_adapter_blocks_unknown_sender() {
        let config = MatrixAccountConfig {
            allowed_users: vec!["@allowed:matrix.org".to_string()],
            ..Default::default()
        };
        let account = MatrixAccount::new("test".to_string(), config);
        let adapter = MatrixSecurityAdapter::new(vec![account]);

        let sender = SenderInfo {
            id: "@unknown:matrix.org".to_string(),
            ..Default::default()
        };

        assert!(!adapter.is_allowed_sender(&sender));
    }

    #[test]
    fn test_matrix_event_to_incoming_message() {
        let msg = matrix_event_to_incoming_message(
            "main",
            "!room:matrix.org",
            Some("General".to_string()),
            "@user:matrix.org",
            Some("User".to_string()),
            Some("user".to_string()),
            "Hello, world!",
            Utc::now(),
        );

        assert_eq!(msg.channel, "matrix-main");
        assert_eq!(msg.sender.id, "@user:matrix.org");
        assert_eq!(msg.sender.display_name, Some("User".to_string()));
        assert_eq!(msg.sender.username, Some("user".to_string()));
        assert_eq!(msg.peer.id, "!room:matrix.org");
        assert_eq!(msg.peer.kind, PeerKind::Group);
        assert_eq!(msg.peer.title, Some("General".to_string()));
    }
}
