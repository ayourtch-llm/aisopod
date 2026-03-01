//! Signal gateway for incoming message handling.
//!
//! This module handles receiving and parsing messages from signal-cli daemon.

use crate::config::{SignalAccountConfig, SignalError};
use aisopod_channel::message::{
    IncomingMessage, Media, MessageContent, MessagePart, PeerInfo, PeerKind, SenderInfo,
};
use aisopod_channel::types::MediaType;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error, info};

/// A Signal message received from the daemon.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalMessage {
    /// Message type (e.g., "receive", "receipt", "contact")
    #[serde(rename = "type")]
    pub message_type: String,
    /// The phone number of the sender
    #[serde(default)]
    pub source: Option<String>,
    /// The phone number of the recipient (for sent messages)
    #[serde(default)]
    pub destination: Option<String>,
    /// The group ID (if in a group)
    #[serde(default)]
    pub group: Option<SignalGroup>,
    /// The timestamp of the message
    #[serde(default)]
    pub timestamp: Option<i64>,
    /// The message content
    #[serde(default)]
    pub message: Option<SignalMessageContent>,
    /// Attachments
    #[serde(default)]
    pub attachments: Option<Vec<SignalAttachment>>,
    /// Disappearing message configuration
    #[serde(default)]
    pub expires_in: Option<u64>,
    /// The message ID
    #[serde(default)]
    pub id: Option<String>,
}

/// Group information in a Signal message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalGroup {
    /// The group ID
    #[serde(rename = "id")]
    pub group_id: Option<String>,
    /// The group name
    #[serde(default)]
    pub name: Option<String>,
    /// Group members
    #[serde(default)]
    pub members: Option<Vec<String>>,
}

/// Message content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalMessageContent {
    /// The body/text of the message
    #[serde(default)]
    pub body: Option<String>,
}

/// Attachment information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalAttachment {
    /// The attachment ID
    #[serde(rename = "id")]
    pub attachment_id: Option<String>,
    /// The content type (MIME type)
    #[serde(rename = "contentType")]
    pub content_type: Option<String>,
    /// The filename
    #[serde(default)]
    pub filename: Option<String>,
    /// The size in bytes
    #[serde(default)]
    pub size: Option<u64>,
    /// The path to the attachment
    #[serde(default)]
    pub path: Option<String>,
    /// The download URL
    #[serde(default)]
    pub digest: Option<String>,
    /// The cipher text
    #[serde(default)]
    pub cipher_text: Option<String>,
}

/// Gateway for handling incoming Signal messages.
pub struct SignalGateway {
    /// Map of accounts to their message handlers
    message_handlers: HashMap<String, Vec<Box<dyn Fn(IncomingMessage) + Send + Sync>>>,
    /// Last received message IDs to prevent duplicates
    last_message_ids: HashMap<String, String>,
}

impl SignalGateway {
    /// Create a new SignalGateway instance.
    pub fn new() -> Self {
        Self {
            message_handlers: HashMap::new(),
            last_message_ids: HashMap::new(),
        }
    }

    /// Parse a raw JSON message from signal-cli.
    ///
    /// # Arguments
    ///
    /// * `json_str` - The JSON string to parse
    /// * `account_id` - The account ID that received this message
    /// * `channel_id` - The channel ID
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<IncomingMessage>)` - Parsed messages
    /// * `Err(SignalError)` - Parsing error
    pub fn parse_message(
        &self,
        json_str: &str,
        account_id: &str,
        channel_id: &str,
    ) -> Result<Vec<IncomingMessage>, SignalError> {
        debug!("Parsing Signal message: {}", json_str);

        // Parse the JSON
        let signal_msg: SignalMessage =
            serde_json::from_str(json_str).map_err(|e| SignalError::JsonParseError(e))?;

        // Only process receive messages
        if signal_msg.message_type != "receive" {
            debug!(
                "Skipping non-receive message type: {}",
                signal_msg.message_type
            );
            return Ok(Vec::new());
        }

        // Extract sender info
        let sender_id = signal_msg
            .source
            .clone()
            .unwrap_or_else(|| "unknown".to_string());

        // Extract group info if present
        let (peer, chat_type) = if let Some(group) = &signal_msg.group {
            let group_id = group
                .group_id
                .clone()
                .unwrap_or_else(|| format!("group-{}", Utc::now().timestamp()));

            (
                PeerInfo {
                    id: group_id,
                    kind: PeerKind::Group,
                    title: group.name.clone(),
                },
                aisopod_channel::types::ChatType::Group,
            )
        } else {
            (
                PeerInfo {
                    id: sender_id.clone(),
                    kind: PeerKind::User,
                    title: None,
                },
                aisopod_channel::types::ChatType::Dm,
            )
        };

        // Build message content
        let content = self.build_message_content(&signal_msg)?;

        // Parse timestamp
        let timestamp = signal_msg
            .timestamp
            .map(|ts| DateTime::<Utc>::from_timestamp(ts, 0).unwrap_or_else(Utc::now))
            .unwrap_or_else(Utc::now);

        // Create the incoming message
        let incoming = IncomingMessage {
            id: signal_msg
                .id
                .clone()
                .unwrap_or_else(|| format!("msg-{}", Utc::now().timestamp_nanos())),
            channel: channel_id.to_string(),
            account_id: account_id.to_string(),
            sender: SenderInfo {
                id: sender_id,
                display_name: None,
                username: None,
                is_bot: false,
            },
            peer,
            content,
            reply_to: None,
            timestamp,
            metadata: serde_json::json!({
                "type": signal_msg.message_type,
                "expires_in": signal_msg.expires_in,
            }),
        };

        Ok(vec![incoming])
    }

    /// Build the message content from a SignalMessage.
    fn build_message_content(
        &self,
        signal_msg: &SignalMessage,
    ) -> Result<MessageContent, SignalError> {
        // Build text content
        let mut parts: Vec<MessagePart> = Vec::new();

        if let Some(body) = &signal_msg.message.as_ref().and_then(|m| m.body.as_ref()) {
            parts.push(MessagePart::Text(body.to_string()));
        }

        // Add media attachments
        if let Some(attachments) = &signal_msg.attachments {
            for attachment in attachments {
                let media_type =
                    self.map_content_type_to_media_type(attachment.content_type.as_deref());

                parts.push(MessagePart::Media(Media {
                    media_type,
                    url: None, // Signal attachments are typically stored locally
                    data: None,
                    filename: attachment.filename.clone(),
                    mime_type: attachment.content_type.clone(),
                    size_bytes: attachment.size,
                }));
            }
        }

        // Return mixed content if we have multiple parts, otherwise single content
        if parts.is_empty() {
            Ok(MessageContent::Text("".to_string()))
        } else if parts.len() == 1 {
            match &parts[0] {
                MessagePart::Text(text) => Ok(MessageContent::Text(text.clone())),
                MessagePart::Media(media) => Ok(MessageContent::Media(media.clone())),
            }
        } else {
            Ok(MessageContent::Mixed(parts))
        }
    }

    /// Map Signal content type to MediaType.
    fn map_content_type_to_media_type(&self, content_type: Option<&str>) -> MediaType {
        match content_type {
            Some(ct) if ct.starts_with("image/") => MediaType::Image,
            Some(ct) if ct.starts_with("audio/") => MediaType::Audio,
            Some(ct) if ct.starts_with("video/") => MediaType::Video,
            Some(ct) if ct.starts_with("application/pdf") => MediaType::Document,
            Some(ct) if ct.starts_with("text/") => MediaType::Other("text".to_string()),
            Some(ct) => MediaType::Other(ct.to_string()),
            None => MediaType::Other("unknown".to_string()),
        }
    }

    /// Extract disappearing message timer from a message.
    pub fn extract_disappearing_timer(&self, signal_msg: &SignalMessage) -> Option<u64> {
        signal_msg.expires_in
    }

    /// Check if this message is a duplicate.
    pub fn is_duplicate(&mut self, account_id: &str, message_id: &str) -> bool {
        if let Some(last_id) = self.last_message_ids.get(account_id) {
            if last_id == message_id {
                return true;
            }
        }
        self.last_message_ids
            .insert(account_id.to_string(), message_id.to_string());
        false
    }

    /// Register a message handler for a specific account.
    pub fn register_message_handler<F>(&mut self, account_id: &str, handler: F)
    where
        F: Fn(IncomingMessage) + Send + Sync + 'static,
    {
        self.message_handlers
            .entry(account_id.to_string())
            .or_default()
            .push(Box::new(handler));
    }

    /// Get handlers for an account.
    pub fn get_handlers(
        &self,
        account_id: &str,
    ) -> Vec<&Box<dyn Fn(IncomingMessage) + Send + Sync>> {
        self.message_handlers
            .get(account_id)
            .map(|h| h.iter().collect::<Vec<_>>())
            .unwrap_or_default()
    }
}

/// Utility functions for Signal message handling.
pub mod message_utils {
    use super::*;

    /// Parse a JSON-RPC response.
    pub fn parse_jsonrpc_response(json_str: &str) -> Result<serde_json::Value, SignalError> {
        serde_json::from_str(json_str).map_err(|e| SignalError::JsonParseError(e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_dm_message() {
        let json = r#"{
            "type": "receive",
            "source": "+1234567890",
            "timestamp": 1618907555000,
            "message": {
                "body": "Hello, world!"
            },
            "id": "msg123"
        }"#;

        let gateway = SignalGateway::new();
        let messages = gateway
            .parse_message(json, "test-account", "signal-test")
            .unwrap();

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].sender.id, "+1234567890");
        assert_eq!(messages[0].peer.kind, PeerKind::User);
    }

    #[test]
    fn test_parse_group_message() {
        let json = r#"{
            "type": "receive",
            "source": "+1234567890",
            "group": {
                "id": "group123",
                "name": "Test Group"
            },
            "timestamp": 1618907555000,
            "message": {
                "body": "Hello group!"
            },
            "id": "msg456"
        }"#;

        let gateway = SignalGateway::new();
        let messages = gateway
            .parse_message(json, "test-account", "signal-test")
            .unwrap();

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].peer.kind, PeerKind::Group);
        assert_eq!(messages[0].peer.id, "group123");
        assert_eq!(messages[0].peer.title, Some("Test Group".to_string()));
    }

    #[test]
    fn test_parse_message_with_media() {
        let json = r#"{
            "type": "receive",
            "source": "+1234567890",
            "timestamp": 1618907555000,
            "message": {
                "body": "Here's a photo"
            },
            "attachments": [{
                "contentType": "image/jpeg",
                "filename": "photo.jpg",
                "size": 12345
            }],
            "id": "msg789"
        }"#;

        let gateway = SignalGateway::new();
        let messages = gateway
            .parse_message(json, "test-account", "signal-test")
            .unwrap();

        assert_eq!(messages.len(), 1);
        match &messages[0].content {
            MessageContent::Mixed(parts) => {
                assert_eq!(parts.len(), 2);
                assert!(matches!(parts[0], MessagePart::Text(_)));
                assert!(matches!(parts[1], MessagePart::Media(_)));
            }
            _ => panic!("Expected mixed content"),
        }
    }

    #[test]
    fn test_extract_disappearing_timer() {
        let json = r#"{
            "type": "receive",
            "source": "+1234567890",
            "timestamp": 1618907555000,
            "expires_in": 3600,
            "message": {
                "body": "This will disappear"
            },
            "id": "msg123"
        }"#;

        let gateway = SignalGateway::new();
        let messages = gateway
            .parse_message(json, "test-account", "signal-test")
            .unwrap();

        assert_eq!(messages.len(), 1);
        let metadata = &messages[0].metadata;
        assert_eq!(metadata.get("expires_in"), Some(&serde_json::json!(3600)));
    }
}
