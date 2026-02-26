//! Nextcloud Talk channel plugin for aisopod.
//!
//! This crate provides a channel plugin implementation for Nextcloud Talk,
//! enabling the bot to participate in Nextcloud Talk rooms with room-based
//! messaging, file sharing integration, and Nextcloud authentication.
//!
//! # Features
//!
//! - Room-based messaging (send and receive)
//! - File sharing via Nextcloud's file system
//! - Polling for new messages
//! - Nextcloud credentials or app passwords for authentication
//! - Room listing and joining

pub mod api;
pub mod channel;
pub mod config;
pub mod files;
pub mod polling;

// Re-export common types
pub use api::{NextcloudTalkApi, TalkMessage, TalkRoom};
pub use config::NextcloudConfig;
pub use channel::NextcloudChannel;
