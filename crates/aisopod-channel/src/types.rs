//! Channel metadata and capability types.
//!
//! This module defines the core types used throughout the aisopod-channel crate
//! to represent channel metadata, capabilities, and supported features.

use serde::{Deserialize, Serialize};

/// Represents the type of chat conversation.
///
/// This enum categorizes different chat modalities that a channel can support.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChatType {
    /// Direct message between two users.
    Dm,
    /// Group chat with multiple participants.
    Group,
    /// Standard channel or room.
    Channel,
    /// Thread within a channel or discussion.
    Thread,
}

/// Represents the type of media content that can be sent or received.
///
/// This enum categorizes different media formats supported by channels.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaType {
    /// Image media type (e.g., JPEG, PNG).
    Image,
    /// Audio media type (e.g., MP3, WAV).
    Audio,
    /// Video media type (e.g., MP4, WebM).
    Video,
    /// Document media type (e.g., PDF, DOCX).
    Document,
    /// Other media type specified as a string identifier.
    Other(String),
}

/// Metadata about a channel implementation.
///
/// Contains descriptive information and UI hints for channel plugins.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelMeta {
    /// Human-readable label for the channel.
    pub label: String,
    /// Optional URL to documentation for this channel.
    pub docs_url: Option<String>,
    /// JSON value containing UI-specific hints for channel configuration.
    pub ui_hints: serde_json::Value,
}

/// Describes the capabilities supported by a channel.
///
/// This struct provides a comprehensive view of what features a channel
/// implementation supports, allowing the system to adapt its behavior accordingly.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChannelCapabilities {
    /// List of chat types this channel supports.
    pub chat_types: Vec<ChatType>,
    /// Whether the channel supports sending and receiving media.
    pub supports_media: bool,
    /// Whether the channel supports message reactions (emojis).
    pub supports_reactions: bool,
    /// Whether the channel supports threading/replies.
    pub supports_threads: bool,
    /// Whether the channel supports typing indicators.
    pub supports_typing: bool,
    /// Whether the channel supports voice calls or messages.
    pub supports_voice: bool,
    /// Optional maximum message length in characters.
    pub max_message_length: Option<usize>,
    /// List of media types this channel supports.
    pub supported_media_types: Vec<MediaType>,
}
