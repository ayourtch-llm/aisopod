//! Microsoft Teams channel plugin for aisopod.
//!
//! This crate provides a channel plugin implementation for Microsoft Teams,
//! enabling the bot to receive and send messages via the Microsoft Bot Framework.
//!
//! # Features
//!
//! - Microsoft Bot Framework integration
//! - Azure AD authentication with client_credentials grant
//! - Channel and DM messaging support
//! - Adaptive Cards for rich content
//! - Webhook endpoint for incoming Bot Framework activities
//! - Bot Framework JWT token validation
//! - Multi-account support
//!
//! # Authentication
//!
//! The channel requires Azure AD authentication with the following credentials:
//!
//! - Tenant ID: The Azure AD tenant ID
//! - Client ID: The application (client) ID
//! - Client Secret: The application secret
//!
//! # Example
//!
//! ```ignore
//! use aisopod_channel_msteams::{MsTeamsConfig, MsTeamsAccountConfig, MsTeamsChannel};
//! use aisopod_channel::plugin::ChannelPlugin;
//!
//! // Create account configuration
//! let account_config = MsTeamsAccountConfig::new(
//!     "test_account",
//!     "tenant-id",
//!     "client-id",
//!     "client-secret",
//! );
//!
//! // Create channel configuration
//! let config = MsTeamsConfig {
//!     accounts: vec![account_config],
//!     ..Default::default()
//! };
//!
//! // Create the channel
//! let channel = MsTeamsChannel::new(config, "test1").await?;
//!
//! // Get channel metadata
//! println!("Channel: {}", channel.meta().label);
//! println!("Capabilities: {:?}", channel.capabilities());
//!
//! // Start long-polling
//! let polling_task = channel.start_long_polling(None).await?;
//! tokio::spawn(polling_task);
//!
//! // Or start webhook mode
//! let webhook_task = channel.start_webhook("test1", 3978).await?;
//! tokio::spawn(webhook_task);
//!
//! // Send a message
//! use aisopod_channel::message::MessageTarget;
//! use aisopod_channel::types::ChatType;
//!
//! let target = MessageTarget {
//!     channel: "msteams".to_string(),
//!     account_id: "test1".to_string(),
//!     peer: aisopod_channel::message::PeerInfo {
//!         id: "conversation-id".to_string(),
//!         kind: aisopod_channel::message::PeerKind::User,
//!         title: None,
//!     },
//!     thread_id: None,
//! };
//!
//! channel.send_message(&target, "Hello, Teams!").await?;
//!
//! # Ok::<(), anyhow::Error>(())
//! ```
//!
//! # Webhook Setup
//!
//! To receive events from Microsoft Teams, set up a webhook endpoint:
//!
//! ```ignore
//! use axum::Router;
//! use aisopod_channel_msteams::MsTeamsChannel;
//!
//! async fn setup_webhook(channel: &MsTeamsChannel, router: Router) -> Router {
//!     channel.create_webhook_router("account-id")
//!         .map(|r| router.nest("/msteams", r))
//!         .unwrap_or(router)
//! }
//! ```
//!
//! # Adaptive Cards
//!
//! Use Adaptive Cards for rich content messages:
//!
//! ```ignore
//! use aisopod_channel_msteams::adaptive_cards::{AdaptiveCard, TextBlock, CardBuilder};
//!
//! let card = AdaptiveCard::new()
//!     .with_body(vec![
//!         TextBlock::new("Project Update")
//!             .with_weight("bolder")
//!             .with_size("large"),
//!         TextBlock::new("All tasks completed successfully!"),
//!     ]);
//!
//! // The card can be sent as part of a message
//! ```

mod adaptive_cards;
mod auth;
mod botframework;
mod channel;
mod config;
mod webhook;

// Re-export common types
pub use adaptive_cards::{
    helpers, AdaptiveCard, AdaptiveCardAction, AdaptiveCardElement, BackgroundImage, CardElement,
    Column, ColumnSet, Container, Fact, FactSet, Image, ImageSet, InputDate, InputText,
    InputToggle, OpenUrlAction, SetImage, ShowCardAction, Spacer, SubmitAction, TextBlock,
};
pub use auth::{AzureAuthConfig, MsTeamsAuth, MsTeamsAuthToken};
pub use botframework::{
    Activity, ActivityType, BotFrameworkClient, ChannelAccount, ConversationMember,
    ConversationPsmInfo, ConversationResponse, SendActivityResponse,
};
pub use channel::{
    MsTeamsAccount, MsTeamsChannel, MsTeamsChannelConfigAdapter, MsTeamsSecurityAdapter,
};
pub use config::{MsTeamsAccountConfig, MsTeamsConfig, WebhookConfig};
pub use webhook::{
    create_validation_response, create_webhook_router, handle_webhook, validate_microsoft_app_id,
    WebhookState,
};
