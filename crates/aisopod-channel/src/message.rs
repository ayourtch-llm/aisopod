//! Message types for the channel abstraction layer.
//!
//! This module defines the core message data types used throughout the channel
//! abstraction layer. These types represent all messages flowing between channels
//! and the agent engine.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::MediaType;

/// The kind of peer in a conversation.
///
/// This enum categorizes the different types of conversation participants
/// that messages can be sent to or received from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PeerKind {
    /// A single user.
    User,
    /// A group of users.
    Group,
    /// A channel or room.
    Channel,
    /// A thread within a channel or discussion.
    Thread,
}

/// Information about a conversation peer.
///
/// This struct represents the participant in a conversation, whether it's
/// a user, group, channel, or thread.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    /// Unique identifier for the peer.
    pub id: String,
    /// The kind of peer (user, group, channel, or thread).
    pub kind: PeerKind,
    /// Optional display title for the peer.
    pub title: Option<String>,
}

/// Information about a message sender.
///
/// This struct provides details about who sent a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SenderInfo {
    /// Sender's unique identifier.
    pub id: String,
    /// Sender's display name.
    pub display_name: Option<String>,
    /// Sender's username or handle.
    pub username: Option<String>,
    /// Whether the sender is a bot.
    pub is_bot: bool,
}

/// Media content to be sent or received.
///
/// This struct represents media content that can be embedded in messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Media {
    /// The type of media.
    pub media_type: MediaType,
    /// Optional URL where the media is hosted.
    pub url: Option<String>,
    /// Optional raw media data.
    pub data: Option<Vec<u8>>,
    /// Optional filename for the media.
    pub filename: Option<String>,
    /// Optional MIME type of the media.
    pub mime_type: Option<String>,
    /// Optional size of the media in bytes.
    pub size_bytes: Option<u64>,
}

/// A single part of a mixed message.
///
/// This enum represents one component of a potentially rich message
/// that can contain multiple content types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessagePart {
    /// Text content.
    Text(String),
    /// Media content.
    Media(Media),
}

/// The content of a message.
///
/// This enum represents all possible content types that a message can have,
/// from simple text to complex mixed content with multiple parts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageContent {
    /// Plain text content.
    Text(String),
    /// Media content.
    Media(Media),
    /// Mixed content with multiple parts.
    Mixed(Vec<MessagePart>),
}

/// Target information for an outgoing message.
///
/// This struct specifies where an outgoing message should be delivered.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageTarget {
    /// The channel identifier.
    pub channel: String,
    /// The account identifier.
    pub account_id: String,
    /// The target peer information.
    pub peer: PeerInfo,
    /// Optional thread identifier for threaded conversations.
    pub thread_id: Option<String>,
}

/// An incoming message from a channel.
///
/// This struct represents a message received from a channel, containing
/// all metadata about the message and its context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomingMessage {
    /// Unique identifier for the message.
    pub id: String,
    /// The channel identifier.
    pub channel: String,
    /// The account identifier that received the message.
    pub account_id: String,
    /// Information about the message sender.
    pub sender: SenderInfo,
    /// Information about the message peer.
    pub peer: PeerInfo,
    /// The content of the message.
    pub content: MessageContent,
    /// Optional identifier of the message this reply is in response to.
    pub reply_to: Option<String>,
    /// The timestamp when the message was sent.
    pub timestamp: DateTime<Utc>,
    /// Additional metadata as JSON.
    pub metadata: serde_json::Value,
}

impl IncomingMessage {
    /// Converts the message content to a string representation.
    ///
    /// This method extracts the text content from the message, handling
    /// different content types (text, media, mixed).
    ///
    /// # Returns
    ///
    /// A string representation of the message content.
    pub fn content_to_string(&self) -> String {
        match &self.content {
            MessageContent::Text(text) => text.clone(),
            MessageContent::Media(media) => {
                // Return a placeholder for media content
                match &media.media_type {
                    crate::types::MediaType::Image => format!("[Image: {}]", media.url.as_deref().unwrap_or("unknown")),
                    crate::types::MediaType::Audio => format!("[Audio: {}]", media.url.as_deref().unwrap_or("unknown")),
                    crate::types::MediaType::Video => format!("[Video: {}]", media.url.as_deref().unwrap_or("unknown")),
                    crate::types::MediaType::Document => format!("[Document: {}]", media.filename.as_deref().unwrap_or("unknown")),
                    crate::types::MediaType::Other(other) => format!("[{}: {}]", other, media.url.as_deref().unwrap_or("unknown")),
                }
            }
            MessageContent::Mixed(parts) => {
                parts
                    .iter()
                    .map(|part| match part {
                        MessagePart::Text(text) => text.clone(),
                        MessagePart::Media(media) => {
                            match &media.media_type {
                                crate::types::MediaType::Image => format!("[Image: {}]", media.url.as_deref().unwrap_or("unknown")),
                                crate::types::MediaType::Audio => format!("[Audio: {}]", media.url.as_deref().unwrap_or("unknown")),
                                crate::types::MediaType::Video => format!("[Video: {}]", media.url.as_deref().unwrap_or("unknown")),
                                crate::types::MediaType::Document => format!("[Document: {}]", media.filename.as_deref().unwrap_or("unknown")),
                                crate::types::MediaType::Other(other) => format!("[{}: {}]", other, media.url.as_deref().unwrap_or("unknown")),
                            }
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            }
        }
    }
}

/// An outgoing message to be sent to a channel.
///
/// This struct represents a message to be sent through a channel,
/// with all necessary routing and content information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutgoingMessage {
    /// The target for the message.
    pub target: MessageTarget,
    /// The content of the message.
    pub content: MessageContent,
    /// Optional identifier of the message this is replying to.
    pub reply_to: Option<String>,
}
