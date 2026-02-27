//! Lark/Feishu channel implementation for aisopod.
//!
//! This crate provides integration with the Lark Open Platform API, enabling
//! aisopod to send and receive messages in Lark groups and direct messages.
//!
//! # Features
//!
//! - Send and receive messages in Lark groups and DMs
//! - Rich message cards support
//! - Event subscription webhook handling
//! - App credentials management for token refresh
//! - Feishu domain support for China region
//!
//! # Example
//!
//! ```rust,ignore
//! use aisopod_channel_lark::{LarkChannel, LarkConfig};
//!
//! let config = LarkConfig {
//!     app_id: "your_app_id".to_string(),
//!     app_secret: "your_app_secret".to_string(),
//!     verification_token: "your_verification_token".to_string(),
//!     encrypt_key: None,
//!     webhook_port: 8080,
//!     use_feishu: false,
//! };
//!
//! let channel = LarkChannel::new(config, "main")?;
//! ```
//!
//! # API References
//!
//! - [Lark Open Platform](https://open.larksuite.com)
//! - [Feishu Open Platform](https://open.feishu.cn)

pub mod api;
pub mod auth;
pub mod cards;
pub mod channel;
pub mod config;
pub mod events;

// Re-export common types
pub use api::{ApiError, LarkApi, UserProfile};
pub use auth::{AuthError, LarkAuth};
pub use cards::{CardElement, CardHeader, CardText, MessageCard};
pub use channel::{LarkAccount, LarkChannel};
pub use config::LarkConfig;
pub use events::{EventType, WebhookRequestBody, handle_event, lark_router, AppState};
