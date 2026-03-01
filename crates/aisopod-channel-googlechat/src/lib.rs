//! Google Chat channel plugin for aisopod.
//!
//! This crate provides a channel plugin implementation for Google Chat,
//! enabling the bot to receive and send messages via the Google Chat API.
//!
//! # Features
//!
//! - OAuth 2.0 and Service Account authentication
//! - Rich card-based message support
//! - Webhook-based event delivery
//! - Multi-account support
//! - Message filtering and access control
//! - Space management
//!
//! # Authentication
//!
//! The channel supports two authentication methods:
//!
//! ## OAuth 2.0
//!
//! ```rust,ignore
//! use aisopod_channel_googlechat::{GoogleChatConfig, GoogleChatAccountConfig, OAuth2Config};
//!
//! let config = GoogleChatAccountConfig {
//!     auth_type: AuthType::OAuth2,
//!     oauth2: Some(OAuth2Config {
//!         client_id: "your-client-id".to_string(),
//!         client_secret: "your-client-secret".to_string(),
//!         refresh_token: "your-refresh-token".to_string(),
//!         ..Default::default()
//!     }),
//!     ..Default::default()
//! };
//! ```
//!
//! ## Service Account
//!
//! ```rust,ignore
//! use aisopod_channel_googlechat::{GoogleChatConfig, GoogleChatAccountConfig, ServiceAccountConfig};
//!
//! let config = GoogleChatAccountConfig {
//!     auth_type: AuthType::ServiceAccount,
//!     service_account: Some(ServiceAccountConfig {
//!         key_file: "/path/to/key.json".to_string(),
//!         client_email: "your-service-account@test.iam.gserviceaccount.com".to_string(),
//!         private_key: "-----BEGIN PRIVATE KEY-----\n...\n-----END PRIVATE KEY-----\n".to_string(),
//!         ..Default::default()
//!     }),
//!     ..Default::default()
//! };
//! ```
//!
//! # Card Messages
//!
//! ```rust,ignore
//! use aisopod_channel_googlechat::cards::{CardBuilder, CardSection, Widget, TextParagraph, ButtonWidget, OnClick, OpenLink};
//!
//! let card = CardBuilder::new()
//!     .header(CardHeader::new("Project Update").subtitle("Weekly status"))
//!     .section(
//!         CardSection::new()
//!             .widget(Widget::TextParagraph(TextParagraph::new("All tasks completed!")))
//!             .widget(Widget::ButtonList(ButtonList::new()
//!                 .button(ButtonWidget::new("View Details")
//!                     .on_click(OnClick::OpenLink(OpenLink::new("https://example.com")))
//!                 )
//!             ))
//!     )
//!     .build();
//! ```
//!
//! # Webhook Setup
//!
//! To receive events from Google Chat, set up a webhook endpoint:
//!
//! ```rust,ignore
//! use aisopod_channel_googlechat::GoogleChatChannel;
//! use axum::Router;
//!
//! async fn setup_webhook(channel: &GoogleChatChannel, router: Router) -> Router {
//!     channel.register_webhook_routes(router, "account-id").await
//! }
//! ```

mod api;
mod auth;
mod cards;
mod channel;
mod config;
mod webhook;

// Re-export common types
pub use api::{Bot, GoogleChatClient, Message, Space, SpaceType, User};
pub use api::{
    BotType, CreateUserInvitationRequest, Invitation, ListMembersResponse, Member,
    SpaceThreadReadState,
};
pub use api::{
    CreateMessageRequest, CreateSpaceRequest, ListMessagesResponse, UpdateMessageRequest,
};
pub use api::{
    InvitationState, ListSpacesResponse, MemberRole, MemberState, MessageType, Reaction, Thread,
};
pub use auth::{
    GoogleChatAuth, GoogleChatAuthToken, OAuth2Auth, OAuth2Config, ServiceAccountAuth,
    ServiceAccountConfig,
};
pub use cards::{ButtonList, Divider, GridItem, GridWidget, IconStyle, PickerType, PickersItem};
pub use cards::{
    ButtonWidget, CardHeader, CardImage, CardSection, ImageAction, ImageStyle, ImageWidget,
    TextParagraph, Widget,
};
pub use cards::{
    DecoratedText, Icon, ImageLayout, OnClick, SelectionInputWidget, SelectionItem, SelectionType,
    TextFormat,
};
pub use channel::{
    GoogleChatAccount, GoogleChatChannel, GoogleChatChannelConfigAdapter, GoogleChatSecurityAdapter,
};
pub use config::{AuthType, GoogleChatAccountConfig, GoogleChatConfig, WebhookConfig};
pub use webhook::WebhookCardAction;
pub use webhook::{
    create_webhook_router, EventType, WebhookPayload, WebhookState, WebhookVerifyQuery,
};

use aisopod_channel::adapters::AccountConfig;
use aisopod_channel::types::ChannelMeta;
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};
