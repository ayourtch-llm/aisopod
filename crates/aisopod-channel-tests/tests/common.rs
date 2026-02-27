//! Shared test helpers for channel integration tests.
//!
//! This module provides common test utilities and fixtures that can be
//! used across all channel integration tests.

use aisopod_channel::message::{IncomingMessage, OutgoingMessage, MessageContent, MessagePart, Media, PeerInfo, PeerKind, SenderInfo, MessageTarget};
use aisopod_channel::types::MediaType;
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::{DateTime, Utc};

/// Creates a simple test outgoing message.
///
/// # Arguments
///
/// * `text` - The text content of the message
/// * `channel` - The channel identifier (e.g., "signal", "imessage")
/// * `account_id` - The account identifier
/// * `peer_id` - The target peer identifier
///
/// # Returns
///
/// An OutgoingMessage with the specified content.
pub fn test_outbound_message(text: &str, channel: &str, account_id: &str, peer_id: &str) -> OutgoingMessage {
    OutgoingMessage {
        target: MessageTarget {
            channel: channel.to_string(),
            account_id: account_id.to_string(),
            peer: PeerInfo {
                id: peer_id.to_string(),
                kind: PeerKind::User,
                title: None,
            },
            thread_id: None,
        },
        content: MessageContent::Text(text.to_string()),
        reply_to: None,
    }
}

/// Creates a test incoming message.
///
/// # Arguments
///
/// * `text` - The text content of the message
/// * `channel` - The channel identifier
/// * `account_id` - The account identifier
/// * `sender_id` - The sender's identifier
/// * `sender_name` - The sender's display name
///
/// # Returns
///
/// An IncomingMessage with the specified content.
pub fn test_inbound_message(text: &str, channel: &str, account_id: &str, sender_id: &str, sender_name: Option<&str>) -> IncomingMessage {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as i64;
    
    IncomingMessage {
        id: format!("msg_{}", timestamp),
        channel: channel.to_string(),
        account_id: account_id.to_string(),
        sender: SenderInfo {
            id: sender_id.to_string(),
            display_name: sender_name.map(|s| s.to_string()),
            username: Some(sender_id.to_string()),
            is_bot: false,
        },
        peer: PeerInfo {
            id: "test_peer".to_string(),
            kind: PeerKind::User,
            title: None,
        },
        content: MessageContent::Text(text.to_string()),
        reply_to: None,
        timestamp: Utc::now(),
        metadata: serde_json::json!({}),
    }
}

/// Creates a test media message.
///
/// # Arguments
///
/// * `media_type` - The type of media
/// * `url` - The URL where the media is hosted
/// * `channel` - The channel identifier
/// * `account_id` - The account identifier
/// * `peer_id` - The target peer identifier
///
/// # Returns
///
/// An OutgoingMessage with media content.
pub fn test_media_message(media_type: MediaType, url: &str, channel: &str, account_id: &str, peer_id: &str) -> OutgoingMessage {
    OutgoingMessage {
        target: MessageTarget {
            channel: channel.to_string(),
            account_id: account_id.to_string(),
            peer: PeerInfo {
                id: peer_id.to_string(),
                kind: PeerKind::User,
                title: None,
            },
            thread_id: None,
        },
        content: MessageContent::Media(Media {
            media_type,
            url: Some(url.to_string()),
            data: None,
            filename: None,
            mime_type: None,
            size_bytes: None,
        }),
        reply_to: None,
    }
}

/// Asserts that a message contains the expected text.
///
/// # Arguments
///
/// * `msg` - The message to check
/// * `expected_text` - The expected text content
pub fn assert_message_text(msg: &IncomingMessage, expected_text: &str) {
    let actual_text = msg.content_to_string();
    assert_eq!(actual_text, expected_text, "Message text mismatch");
}

/// Creates a test group peer info.
///
/// # Arguments
///
/// * `group_id` - The group identifier
/// * `group_name` - The group name
///
/// # Returns
///
/// A PeerInfo with Group kind.
pub fn test_group_peer(group_id: &str, group_name: Option<&str>) -> PeerInfo {
    PeerInfo {
        id: group_id.to_string(),
        kind: PeerKind::Group,
        title: group_name.map(|s| s.to_string()),
    }
}

/// Creates a test signal account ID.
pub fn test_signal_account() -> String {
    "test-signal".to_string()
}

/// Creates a test iMessage account ID.
pub fn test_imessage_account() -> String {
    "test-imessage".to_string()
}

/// Creates a test Google Chat account ID.
pub fn test_googlechat_account() -> String {
    "test-googlechat".to_string()
}

/// Creates a test Microsoft Teams account ID.
pub fn test_msteams_account() -> String {
    "test-msteams".to_string()
}
