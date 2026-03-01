//! WhatsApp webhook message receiving and parsing.

use aisopod_channel::message::{
    IncomingMessage, Media, MessageContent, MessagePart, PeerInfo, PeerKind, SenderInfo,
};
use aisopod_channel::types::MediaType;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// WhatsApp incoming message structure from the Business API.
///
/// This represents the JSON payload structure from WhatsApp's webhook.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppWebhookPayload {
    /// The array of entries, each representing a change.
    pub entry: Vec<WhatsAppEntry>,
}

/// An entry in the webhook payload representing a change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppEntry {
    /// The ID of the entry.
    pub id: String,
    /// The timestamp of the change.
    pub time: i64,
    /// The changes that occurred.
    pub changes: Vec<WhatsAppChange>,
}

/// A change in the webhook payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppChange {
    /// The field that changed.
    pub field: String,
    /// The value of the change.
    pub value: WhatsAppValue,
}

/// The value of a change in the webhook payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppValue {
    /// The messaging type (e.g., "text", "image", "button").
    #[serde(rename = "messaging_product")]
    pub messaging_product: String,
    /// The metadata for this change.
    #[serde(default)]
    pub metadata: WhatsAppMetadata,
    /// The messages array.
    #[serde(default)]
    pub messages: Vec<WhatsAppMessage>,
    /// The statuses array.
    #[serde(default)]
    pub statuses: Vec<WhatsAppStatus>,
    /// The contacts array.
    #[serde(default)]
    pub contacts: Vec<WhatsAppContact>,
    /// The errors array.
    #[serde(default)]
    pub errors: Vec<WhatsAppError>,
}

/// Metadata for a WhatsApp change.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WhatsAppMetadata {
    /// The display phone number.
    pub display_phone_number: String,
    /// The phone number ID.
    pub phone_number_id: String,
    /// Optional chat information for group messages.
    #[serde(default)]
    pub chat: Option<String>,
}

/// A message received from WhatsApp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppMessage {
    /// The message ID.
    pub id: String,
    /// The timestamp of the message.
    pub timestamp: String,
    /// The message metadata.
    #[serde(default)]
    pub metadata: Option<WhatsAppMessageMetadata>,
    /// The sender information.
    #[serde(default)]
    pub from: Option<String>,
    /// The source information.
    #[serde(default)]
    pub source: Option<WhatsAppSource>,
    /// The message type.
    #[serde(rename = "type")]
    pub message_type: Option<String>,
    /// Raw content for parsing.
    #[serde(default, flatten)]
    pub content: HashMap<String, serde_json::Value>,
}

/// Metadata for a WhatsApp message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppMessageMetadata {
    /// The name of the sender.
    #[serde(default)]
    pub author: Option<String>,
}

/// A source for a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppSource {
    /// The ID of the source.
    #[serde(default)]
    pub id: Option<String>,
}

/// A contact in the contacts array.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppContact {
    /// The profile information.
    pub profiles: Vec<WhatsAppProfile>,
}

/// A contact profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppProfile {
    /// The display name.
    pub name: String,
}

/// A status update for a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppStatus {
    /// The status ID.
    pub id: String,
    /// The status (e.g., "sent", "delivered", "read").
    pub status: String,
    /// The timestamp.
    pub timestamp: String,
    /// The recipient ID.
    #[serde(default)]
    pub recipient_id: Option<String>,
}

/// An error from WhatsApp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppError {
    /// The error code.
    pub code: u16,
    /// The error title.
    pub title: String,
    /// The error details.
    #[serde(default)]
    pub details: Option<String>,
}

/// Parse a webhook payload and extract incoming messages.
///
/// # Arguments
///
/// * `payload` - The raw webhook payload string
/// * `account_id` - The account ID for this channel
/// * `channel` - The channel identifier
/// * `allowed_numbers` - Optional list of allowed phone numbers
///
/// # Returns
///
/// A vector of normalized IncomingMessage structs
pub fn parse_webhook_payload(
    payload: &str,
    account_id: &str,
    channel: &str,
    allowed_numbers: Option<&[String]>,
) -> Result<Vec<IncomingMessage>> {
    let webhook: WhatsAppWebhookPayload = serde_json::from_str(payload)?;
    let mut messages = Vec::new();

    for entry in webhook.entry {
        for change in entry.changes {
            for message in &change.value.messages {
                if let Some(from) = &message.from {
                    // Check if the sender is in the allowed list
                    if let Some(allowed) = allowed_numbers {
                        if !allowed.contains(&from.to_string()) {
                            continue;
                        }
                    }

                    // Normalize the message
                    if let Ok(incoming) = normalize_message(
                        message,
                        from,
                        &change.value.metadata.phone_number_id,
                        account_id,
                        channel,
                        entry.time,
                    ) {
                        messages.push(incoming);
                    }
                }
            }
        }
    }

    Ok(messages)
}

/// Normalize a WhatsApp message to the shared IncomingMessage type.
///
/// # Arguments
///
/// * `message` - The WhatsApp message to normalize
/// * `from` - The sender's phone number
/// * `phone_number_id` - The phone number ID
/// * `account_id` - The account ID for this channel
/// * `channel` - The channel identifier
/// * `timestamp` - The timestamp from the entry
///
/// # Returns
///
/// A normalized IncomingMessage struct
pub fn normalize_message(
    message: &WhatsAppMessage,
    from: &str,
    phone_number_id: &str,
    account_id: &str,
    channel: &str,
    timestamp: i64,
) -> Result<IncomingMessage> {
    // Convert timestamp from seconds to DateTime
    let timestamp_dt = DateTime::from_timestamp(timestamp, 0)
        .ok_or_else(|| anyhow!("Invalid timestamp: {}", timestamp))?
        .with_timezone(&Utc);

    // Determine peer info (DM vs Group)
    // Check if the message is in a group context by looking at the metadata's chat field
    // In WhatsApp's API, group messages include a "chat" field with the group ID
    let chat_id = message
        .content
        .get("chat")
        .and_then(|v| v.as_object())
        .and_then(|obj| obj.get("id"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let (peer_info, sender_info) = if let Some(group_id) = chat_id {
        // This is a group message - use the group ID as the peer
        // The sender is still the individual user who sent the message
        let peer_info = PeerInfo {
            id: group_id,
            kind: PeerKind::Group,
            title: None, // Group title would need to be fetched separately
        };

        let sender_info = SenderInfo {
            id: from.to_string(),
            display_name: None,
            username: None,
            is_bot: false,
        };

        (peer_info, sender_info)
    } else {
        // This is a DM - use the sender's phone number as the peer
        let peer_info = PeerInfo {
            id: from.to_string(),
            kind: PeerKind::User,
            title: None,
        };

        let sender_info = SenderInfo {
            id: from.to_string(),
            display_name: None,
            username: None,
            is_bot: false,
        };

        (peer_info, sender_info)
    };

    // Build the content based on message type
    let content = match message.message_type.as_deref() {
        Some("text") => {
            if let Some(serde_json::Value::Object(data)) = message.content.get("text") {
                if let Some(body) = data.get("body").and_then(|v| v.as_str()) {
                    MessageContent::Text(body.to_string())
                } else {
                    MessageContent::Text("[Empty text message]".to_string())
                }
            } else {
                MessageContent::Text("[Unknown text message format]".to_string())
            }
        }
        Some("image") => {
            if let Some(serde_json::Value::Object(data)) = message.content.get("image") {
                let media = Media {
                    media_type: MediaType::Image,
                    url: None,
                    data: None,
                    filename: data
                        .get("filename")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    mime_type: data
                        .get("mime_type")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    size_bytes: None,
                };
                MessageContent::Media(media)
            } else {
                MessageContent::Text("[Unknown image message format]".to_string())
            }
        }
        Some("audio") => {
            if let Some(serde_json::Value::Object(data)) = message.content.get("audio") {
                let media = Media {
                    media_type: MediaType::Audio,
                    url: None,
                    data: None,
                    filename: data
                        .get("filename")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    mime_type: data
                        .get("mime_type")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    size_bytes: None,
                };
                MessageContent::Media(media)
            } else {
                MessageContent::Text("[Unknown audio message format]".to_string())
            }
        }
        Some("video") => {
            if let Some(serde_json::Value::Object(data)) = message.content.get("video") {
                let media = Media {
                    media_type: MediaType::Video,
                    url: None,
                    data: None,
                    filename: data
                        .get("filename")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    mime_type: data
                        .get("mime_type")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    size_bytes: None,
                };
                MessageContent::Media(media)
            } else {
                MessageContent::Text("[Unknown video message format]".to_string())
            }
        }
        Some("document") => {
            if let Some(serde_json::Value::Object(data)) = message.content.get("document") {
                let media = Media {
                    media_type: MediaType::Document,
                    url: None,
                    data: None,
                    filename: data
                        .get("filename")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    mime_type: data
                        .get("mime_type")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    size_bytes: None,
                };
                MessageContent::Media(media)
            } else {
                MessageContent::Text("[Unknown document message format]".to_string())
            }
        }
        Some("sticker") => {
            let media = Media {
                media_type: MediaType::Other("sticker".to_string()),
                url: None,
                data: None,
                filename: None,
                mime_type: None,
                size_bytes: None,
            };
            MessageContent::Media(media)
        }
        Some("location") => {
            if let Some(serde_json::Value::Object(data)) = message.content.get("location") {
                let latitude = data.get("latitude").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let longitude = data
                    .get("longitude")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let name = data
                    .get("name")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let address = data
                    .get("address")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                let location_text = match (&name, &address) {
                    (Some(name), Some(address)) => format!(
                        "Location: {} - {} ({}°N, {}°E)",
                        name, address, latitude, longitude
                    ),
                    (Some(name), None) => {
                        format!("Location: {} ({}°N, {}°E)", name, latitude, longitude)
                    }
                    (None, Some(address)) => {
                        format!("Location: {} ({}°N, {}°E)", address, latitude, longitude)
                    }
                    (None, None) => format!("Location: {}°N, {}°E", latitude, longitude),
                };
                MessageContent::Text(location_text)
            } else {
                MessageContent::Text("[Unknown location format]".to_string())
            }
        }
        Some("contacts") => {
            MessageContent::Text("[Contact messages not fully supported]".to_string())
        }
        Some("button") => MessageContent::Text("[Button messages not fully supported]".to_string()),
        Some("reaction") => {
            MessageContent::Text("[Reaction messages not fully supported]".to_string())
        }
        Some("system") => MessageContent::Text("[System messages not fully supported]".to_string()),
        Some("unknown") | None => MessageContent::Text("[Unknown message type]".to_string()),
        _ => MessageContent::Text("[Unknown message type]".to_string()),
    };

    // Build the incoming message
    Ok(IncomingMessage {
        id: message.id.clone(),
        channel: channel.to_string(),
        account_id: account_id.to_string(),
        sender: sender_info,
        peer: peer_info,
        content,
        reply_to: None, // Would need additional metadata to determine this
        timestamp: timestamp_dt,
        metadata: serde_json::json!({
            "phone_number_id": phone_number_id,
            "message_type": message.message_type.as_deref().unwrap_or("unknown"),
            "source": message.source.as_ref().and_then(|s| s.id.clone()).unwrap_or_default()
        }),
    })
}

/// Extension trait for WhatsAppMessage to provide helper methods.
pub trait WhatsAppMessageExt {
    /// Returns the message type as a string.
    fn message_type(&self) -> &str;
}

impl WhatsAppMessageExt for WhatsAppMessage {
    fn message_type(&self) -> &str {
        self.message_type.as_deref().unwrap_or("unknown")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_text_message() {
        let payload = r#"{
            "entry": [{
                "id": "WHATSAPP_BUSINESS_ACCOUNT_ID",
                "time": 1618907555000,
                "changes": [{
                    "field": "messages",
                    "value": {
                        "messaging_product": "whatsapp",
                        "metadata": {
                            "display_phone_number": "15550000000",
                            "phone_number_id": "123456789"
                        },
                        "messages": [{
                            "id": "wamid.HBgNNTU1MDAwMDAwMBUA",
                            "timestamp": "1618907554",
                            "from": "15551234567",
                            "type": "text",
                            "text": {
                                "body": "Hello, world!"
                            }
                        }]
                    }
                }]
            }]
        }"#;

        let result = parse_webhook_payload(payload, "test-account", "whatsapp", None);
        assert!(result.is_ok(), "Failed to parse: {:?}", result);
        let messages = result.unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].channel, "whatsapp");
        assert_eq!(messages[0].account_id, "test-account");
        assert_eq!(messages[0].sender.id, "15551234567");
    }

    #[test]
    fn test_parse_image_message() {
        let payload = r#"{
            "entry": [{
                "id": "WHATSAPP_BUSINESS_ACCOUNT_ID",
                "time": 1618907555000,
                "changes": [{
                    "field": "messages",
                    "value": {
                        "messaging_product": "whatsapp",
                        "metadata": {
                            "display_phone_number": "15550000000",
                            "phone_number_id": "123456789"
                        },
                        "messages": [{
                            "id": "wamid.HBgNNTU1MDAwMDAwMBUA",
                            "timestamp": "1618907554",
                            "from": "15551234567",
                            "type": "image",
                            "image": {
                                "id": "12345",
                                "mime_type": "image/jpeg",
                                "caption": "Check out this image!"
                            }
                        }]
                    }
                }]
            }]
        }"#;

        let result = parse_webhook_payload(payload, "test-account", "whatsapp", None);
        assert!(result.is_ok(), "Failed to parse: {:?}", result);
        let messages = result.unwrap();
        assert_eq!(messages.len(), 1);

        // Check the media content
        if let MessageContent::Media(media) = &messages[0].content {
            assert_eq!(media.media_type, MediaType::Image);
            assert_eq!(media.mime_type, Some("image/jpeg".to_string()));
        } else {
            panic!("Expected Media content");
        }
    }

    #[test]
    fn test_parse_location_message() {
        let payload = r#"{
            "entry": [{
                "id": "WHATSAPP_BUSINESS_ACCOUNT_ID",
                "time": 1618907555000,
                "changes": [{
                    "field": "messages",
                    "value": {
                        "messaging_product": "whatsapp",
                        "metadata": {
                            "display_phone_number": "15550000000",
                            "phone_number_id": "123456789"
                        },
                        "messages": [{
                            "id": "wamid.HBgNNTU1MDAwMDAwMBUA",
                            "timestamp": "1618907554",
                            "from": "15551234567",
                            "type": "location",
                            "location": {
                                "latitude": 37.7749,
                                "longitude": -122.4194,
                                "name": "San Francisco",
                                "address": "San Francisco, CA"
                            }
                        }]
                    }
                }]
            }]
        }"#;

        let result = parse_webhook_payload(payload, "test-account", "whatsapp", None);
        assert!(result.is_ok(), "Failed to parse: {:?}", result);
        let messages = result.unwrap();
        assert_eq!(messages.len(), 1);

        // Check the location content
        if let MessageContent::Text(text) = &messages[0].content {
            assert!(text.contains("San Francisco"));
            assert!(text.contains("37.7749"));
        } else {
            panic!("Expected Text content for location");
        }
    }

    #[test]
    fn test_parse_multiple_messages() {
        let payload = r#"{
            "entry": [{
                "id": "WHATSAPP_BUSINESS_ACCOUNT_ID",
                "time": 1618907555000,
                "changes": [{
                    "field": "messages",
                    "value": {
                        "messaging_product": "whatsapp",
                        "metadata": {
                            "display_phone_number": "15550000000",
                            "phone_number_id": "123456789"
                        },
                        "messages": [
                            {
                                "id": "msg1",
                                "timestamp": "1618907554",
                                "from": "15551234567",
                                "type": "text",
                                "text": {"body": "First message"}
                            },
                            {
                                "id": "msg2",
                                "timestamp": "1618907555",
                                "from": "15551234568",
                                "type": "text",
                                "text": {"body": "Second message"}
                            }
                        ]
                    }
                }]
            }]
        }"#;

        let result = parse_webhook_payload(payload, "test-account", "whatsapp", None);
        assert!(result.is_ok(), "Failed to parse: {:?}", result);
        let messages = result.unwrap();
        assert_eq!(messages.len(), 2);
    }

    #[test]
    fn test_filter_allowed_numbers() {
        let payload = r#"{
            "entry": [{
                "id": "WHATSAPP_BUSINESS_ACCOUNT_ID",
                "time": 1618907555000,
                "changes": [{
                    "field": "messages",
                    "value": {
                        "messaging_product": "whatsapp",
                        "metadata": {
                            "display_phone_number": "15550000000",
                            "phone_number_id": "123456789"
                        },
                        "messages": [
                            {
                                "id": "msg1",
                                "timestamp": "1618907554",
                                "from": "15551234567",
                                "type": "text",
                                "text": {"body": "Allowed message"}
                            },
                            {
                                "id": "msg2",
                                "timestamp": "1618907555",
                                "from": "15559999999",
                                "type": "text",
                                "text": {"body": "Not allowed message"}
                            }
                        ]
                    }
                }]
            }]
        }"#;

        let allowed = vec!["15551234567".to_string()];
        let result = parse_webhook_payload(payload, "test-account", "whatsapp", Some(&allowed));
        assert!(result.is_ok(), "Failed to parse: {:?}", result);
        let messages = result.unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].sender.id, "15551234567");
    }

    #[test]
    fn test_message_type_extractor() {
        let text_msg = WhatsAppMessage {
            id: "msg1".to_string(),
            timestamp: "123".to_string(),
            metadata: None,
            from: Some("123".to_string()),
            source: None,
            message_type: Some("text".to_string()),
            content: HashMap::new(),
        };
        assert_eq!(text_msg.message_type(), "text");

        let image_msg = WhatsAppMessage {
            id: "msg2".to_string(),
            timestamp: "123".to_string(),
            metadata: None,
            from: Some("123".to_string()),
            source: None,
            message_type: Some("image".to_string()),
            content: HashMap::new(),
        };
        assert_eq!(image_msg.message_type(), "image");
    }

    #[test]
    fn test_parse_group_message() {
        // Group messages in WhatsApp Business API include a "chat" field with the group ID
        // in the message object (not in metadata)
        let payload = r#"{
            "entry": [{
                "id": "WHATSAPP_BUSINESS_ACCOUNT_ID",
                "time": 1618907555000,
                "changes": [{
                    "field": "messages",
                    "value": {
                        "messaging_product": "whatsapp",
                        "metadata": {
                            "display_phone_number": "15550000000",
                            "phone_number_id": "123456789"
                        },
                        "messages": [{
                            "id": "wamid.HBgNNTU1MDAwMDAwMBUA",
                            "timestamp": "1618907554",
                            "from": "15551234567",
                            "type": "text",
                            "text": {
                                "body": "Hello, group!"
                            },
                            "chat": {
                                "id": "332020202020202020"
                            }
                        }]
                    }
                }]
            }]
        }"#;

        let result = parse_webhook_payload(payload, "test-account", "whatsapp", None);
        assert!(result.is_ok(), "Failed to parse: {:?}", result);
        let messages = result.unwrap();
        assert_eq!(messages.len(), 1);

        // Verify it's recognized as a group message
        assert_eq!(messages[0].peer.kind, PeerKind::Group);
        assert_eq!(messages[0].peer.id, "332020202020202020");

        // Verify the sender is still the individual user
        assert_eq!(messages[0].sender.id, "15551234567");

        // Verify the content
        if let MessageContent::Text(text) = &messages[0].content {
            assert_eq!(*text, "Hello, group!");
        } else {
            panic!("Expected Text content");
        }
    }

    #[test]
    fn test_parse_group_image_message() {
        let payload = r#"{
            "entry": [{
                "id": "WHATSAPP_BUSINESS_ACCOUNT_ID",
                "time": 1618907555000,
                "changes": [{
                    "field": "messages",
                    "value": {
                        "messaging_product": "whatsapp",
                        "metadata": {
                            "display_phone_number": "15550000000",
                            "phone_number_id": "123456789"
                        },
                        "messages": [{
                            "id": "wamid.HBgNNTU1MDAwMDAwMBUA",
                            "timestamp": "1618907554",
                            "from": "15551234567",
                            "type": "image",
                            "image": {
                                "id": "12345",
                                "mime_type": "image/jpeg",
                                "caption": "Check out this image in a group!"
                            },
                            "chat": {
                                "id": "441010101010101010"
                            }
                        }]
                    }
                }]
            }]
        }"#;

        let result = parse_webhook_payload(payload, "test-account", "whatsapp", None);
        assert!(result.is_ok(), "Failed to parse: {:?}", result);
        let messages = result.unwrap();
        assert_eq!(messages.len(), 1);

        // Verify it's recognized as a group message
        assert_eq!(messages[0].peer.kind, PeerKind::Group);
        assert_eq!(messages[0].peer.id, "441010101010101010");

        // Check the media content
        if let MessageContent::Media(media) = &messages[0].content {
            assert_eq!(media.media_type, MediaType::Image);
            assert_eq!(media.mime_type, Some("image/jpeg".to_string()));
        } else {
            panic!("Expected Media content");
        }
    }

    #[test]
    fn test_parse_multiple_group_messages() {
        let payload = r#"{
            "entry": [{
                "id": "WHATSAPP_BUSINESS_ACCOUNT_ID",
                "time": 1618907555000,
                "changes": [{
                    "field": "messages",
                    "value": {
                        "messaging_product": "whatsapp",
                        "metadata": {
                            "display_phone_number": "15550000000",
                            "phone_number_id": "123456789"
                        },
                        "messages": [
                            {
                                "id": "msg1",
                                "timestamp": "1618907554",
                                "from": "15551234567",
                                "type": "text",
                                "text": {"body": "First message in group"},
                                "chat": {"id": "553030303030303030"}
                            },
                            {
                                "id": "msg2",
                                "timestamp": "1618907555",
                                "from": "15551234568",
                                "type": "text",
                                "text": {"body": "Second message in group"},
                                "chat": {"id": "553030303030303030"}
                            }
                        ]
                    }
                }]
            }]
        }"#;

        let result = parse_webhook_payload(payload, "test-account", "whatsapp", None);
        assert!(result.is_ok(), "Failed to parse: {:?}", result);
        let messages = result.unwrap();
        assert_eq!(messages.len(), 2);

        // Verify both are group messages
        assert_eq!(messages[0].peer.kind, PeerKind::Group);
        assert_eq!(messages[1].peer.kind, PeerKind::Group);
        assert_eq!(messages[0].peer.id, "553030303030303030");
        assert_eq!(messages[1].peer.id, "553030303030303030");
    }
}
