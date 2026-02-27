//! Zalo channel implementation for aisopod.
//!
//! This crate provides integration with the Zalo Official Account (OA) API, enabling
//! aisopod to send and receive messages with Zalo users and handle webhook events.
//!
//! # Features
//!
//! - OAuth authentication with automatic token refresh
//! - Webhook-based message receiving from Zalo OA
//! - Support for DMs (direct messages)
//! - Text, image, and file message support
//! - Multi-account support with account-specific configurations
//! - Webhook verification for secure webhook registration
//!
//! # Overview
//!
//! The Zalo channel implementation follows the aisopod channel plugin architecture:
//!
//! - [`ZaloChannel`] - The main channel plugin implementation
//! - [`ZaloAuth`] - OAuth authentication manager with token refresh
//! - [`ZaloApi`] - API client for sending messages
//! - [`ZaloConfig`] - Configuration for Zalo accounts
//!
//! # Example
//!
//! ```no_run
//! use aisopod_channel_zalo::{ZaloConfig, register};
//! use aisopod_channel::ChannelRegistry;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), anyhow::Error> {
//!     let mut registry = ChannelRegistry::new();
//!     
//!     let config = ZaloConfig {
//!         app_id: "your_app_id".to_string(),
//!         app_secret: "your_app_secret".to_string(),
//!         refresh_token: "your_refresh_token".to_string(),
//!         webhook_port: 8080,
//!         oa_secret_key: "your_secret_key".to_string(),
//!         webhook_path: "/zalo/webhook".to_string(),
//!     };
//!     
//!     register(&mut registry, config, "zalo-main").await?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! # Configuration
//!
//! The Zalo channel is configured using a [`ZaloConfig`] struct containing:
//!
//! ```toml
//! [zalo]
//! app_id = "your_zalo_app_id"
//! app_secret = "your_zalo_app_secret"
//! refresh_token = "your_oauth_refresh_token"
//! webhook_port = 8080
//! oa_secret_key = "your_webhook_secret_key"
//! webhook_path = "/zalo/webhook"
//! ```
//!
//! # OAuth Authentication
//!
//! Zalo uses OAuth2 with refresh tokens for authentication. The channel automatically:
//!
//! 1. Obtains an access token from the refresh token
//! 2. Refreshes the access token when it expires
//! 3. Rotates the refresh token when provided by Zalo
//!
//! # Webhook Setup
//!
//! To receive messages from Zalo:
//!
//! 1. Configure a webhook URL in the Zalo Developer Console pointing to your server
//! 2. The webhook path is `/zalo/webhook` by default
//! 3. Set up the webhook in Zalo with your OA secret key for verification
//!
//! # Message Types
//!
//! - **Text**: Plain text messages up to 1000 characters
//! - **Image**: Image messages with URL-based delivery
//! - **File**: File messages using Zalo's CDN

pub mod api;
pub mod auth;
pub mod channel;
pub mod config;
pub mod webhook;

// Re-export common types
pub use api::{ZaloApi, UserProfile, MessagePayload, Recipient, MessageContent, Attachment};
pub use auth::{ZaloAuth, TokenResponse, validate_access_token, TOKEN_ENDPOINT, VERIFY_ENDPOINT};
pub use channel::{ZaloChannel, ZaloAccount, ZaloChannelConfigAdapter, ZaloSecurityAdapter, register};
pub use config::ZaloConfig;
pub use webhook::{WebhookEventType, WebhookState, WebhookVerifyResponse, DEFAULT_WEBHOOK_PATH, ZALO_SIGNATURE_HEADER};

// Re-export types from aisopod-channel for convenience
pub use aisopod_channel::message::{IncomingMessage, OutgoingMessage, MessageTarget, Media, MessageContent as ChannelMessageContent, MessagePart, PeerInfo, PeerKind, SenderInfo};
pub use aisopod_channel::types::{ChannelCapabilities, ChannelMeta, ChatType, MediaType as ChannelMediaType};
pub use aisopod_channel::plugin::ChannelPlugin;
pub use aisopod_channel::ChannelRegistry;

/// Result type for Zalo channel operations.
pub type ZaloResult<T> = std::result::Result<T, anyhow::Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zalo_constants() {
        assert_eq!(auth::TOKEN_ENDPOINT, "https://oauth.zaloapp.com/v4/oa/access_token");
        assert_eq!(auth::VERIFY_ENDPOINT, "https://oauth.zaloapp.com/v4/oa/verify");
        assert_eq!(api::BASE_URL, "https://openapi.zalo.me/v3.0/oa");
        assert_eq!(webhook::DEFAULT_WEBHOOK_PATH, "/zalo/webhook");
    }
}
