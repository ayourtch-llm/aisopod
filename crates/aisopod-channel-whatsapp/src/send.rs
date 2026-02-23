//! WhatsApp Business API message sending functionality.
//!
//! This module provides functionality for sending text messages, media messages,
//! and other message types via the WhatsApp Business API.
//!
//! # Features
//!
//! - Sending text messages with optional reply context
//! - Sending media messages (images, documents, audio, video, stickers)
//! - Message ID tracking and response handling
//! - Error handling for WhatsApp API errors

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, trace};
use std::time::Duration;

use crate::connection::WhatsAppAccount;
use aisopod_channel::message::{Media, MessageContent, MessageTarget, OutgoingMessage};

/// Maximum number of retry attempts for rate-limited requests.
const MAX_RETRY_ATTEMPTS: u32 = 3;

/// Initial delay for exponential backoff (in milliseconds).
const INITIAL_BACKOFF_MS: u64 = 1000;

/// WhatsApp API HTTP status codes for error handling.
const HTTP_STATUS_TOO_MANY_REQUESTS: u16 = 429; // Rate limit
const HTTP_STATUS_UNAUTHORIZED: u16 = 401;       // Invalid token

/// Options for sending a WhatsApp message.
#[derive(Debug, Clone, Default)]
pub struct SendOptions {
    /// Optional message ID to reply to (provides context for the reply)
    pub reply_to: Option<String>,
    /// Whether to send a read receipt
    pub read_receipt: bool,
    /// Whether to send a "typing" indicator (record status)
    pub typing_indicator: bool,
    /// Whether to include the sender's phone number in the reply context
    pub include_phone_number: bool,
}

/// A message ID returned from WhatsApp after sending.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MessageId(pub String);

impl MessageId {
    /// Create a new message ID.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the message ID as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for MessageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The result of sending a message to WhatsApp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendResult {
    /// The message ID assigned by WhatsApp
    pub message_id: MessageId,
    /// Whether the message was accepted by WhatsApp
    pub accepted: bool,
    /// Optional error message if the message was not accepted
    pub error: Option<String>,
}

/// WhatsApp message types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageType {
    /// Text message
    Text,
    /// Image message
    Image,
    /// Document message
    Document,
    /// Audio message
    Audio,
    /// Video message
    Video,
    /// Sticker message
    Sticker,
    /// Location message
    Location,
    /// Reaction message
    Reaction,
}

/// WhatsApp API error response structure.
#[derive(Debug, Clone, Deserialize)]
pub struct WhatsAppApiError {
    /// Error message
    pub message: Option<String>,
    /// Error type
    #[serde(rename = "type")]
    pub error_type: Option<String>,
    /// Error code
    pub code: Option<u16>,
    /// Error subcode
    #[serde(rename = "error_subcode")]
    pub error_subcode: Option<u16>,
    /// Error user message
    #[serde(rename = "error_user_title")]
    pub error_user_title: Option<String>,
    /// Error user message
    #[serde(rename = "error_user_msg")]
    pub error_user_msg: Option<String>,
    /// FBTrace ID for debugging
    #[serde(rename = "fbtrace_id")]
    pub fbtrace_id: Option<String>,
}

/// WhatsApp API response with potential error details.
#[derive(Debug, Clone, Deserialize)]
pub struct WhatsAppErrorResponse {
    /// Error details
    pub error: Option<WhatsAppApiError>,
}

/// Error types for WhatsApp sending operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SendError {
    /// Invalid phone number format
    InvalidPhoneNumber(String),
    /// WhatsApp API error with detailed parsing
    ApiError {
        /// HTTP status code
        status_code: u16,
        /// Raw error response
        response: String,
        /// Parsed error details if available
        parsed_error: Option<WhatsAppApiError>,
    },
    /// Rate limit exceeded
    RateLimitExceeded {
        /// Retry after seconds
        retry_after: Option<u64>,
        /// Raw response
        response: String,
    },
    /// Invalid or expired token
    InvalidToken {
        /// Raw response
        response: String,
    },
    /// Network error
    NetworkError(String),
    /// Configuration error
    ConfigError(String),
    /// Message too long
    MessageTooLong(usize),
}

impl std::fmt::Display for SendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SendError::InvalidPhoneNumber(num) => write!(f, "Invalid phone number: {}", num),
            SendError::ApiError(msg) => write!(f, "WhatsApp API error: {}", msg),
            SendError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            SendError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            SendError::MessageTooLong(len) => write!(f, "Message too long: {} characters", len),
        }
    }
}

impl std::error::Error for SendError {}

impl From<anyhow::Error> for SendError {
    fn from(err: anyhow::Error) -> Self {
        SendError::ApiError {
            status_code: 0,
            response: err.to_string(),
            parsed_error: None,
        }
    }
}

impl From<reqwest::Error> for SendError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            SendError::NetworkError(format!("Request timeout: {}", err))
        } else if err.is_connect() {
            SendError::NetworkError(format!("Connection error: {}", err))
        } else if err.is_decode() {
            SendError::NetworkError(format!("Response decode error: {}", err))
        } else {
            SendError::NetworkError(err.to_string())
        }
    }
}

/// WhatsApp API endpoints.
const API_BASE_URL: &str = "https://graph.facebook.com/v18.0";

/// WhatsApp message API endpoint template.
const MESSAGE_ENDPOINT: &str = "{}/messages";

/// WhatsApp media upload endpoint template.
const MEDIA_ENDPOINT: &str = "{}/media";

/// WhatsApp media download endpoint template.
const MEDIA_DOWNLOAD_ENDPOINT: &str = "{}/media";

// ============================================================================
// WhatsApp Message Payload Structures
// ============================================================================

/// WhatsApp message payload.
#[derive(Debug, Clone, Serialize)]
pub struct WhatsAppMessagePayload {
    /// The messaging product.
    #[serde(rename = "messaging_product")]
    pub messaging_product: String,
    /// The recipient phone number.
    pub to: String,
    /// The message type.
    #[serde(rename = "type")]
    pub message_type: String,
    /// Text content (for text messages).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<WhatsAppText>,
    /// Image content (for image messages).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<WhatsAppMedia>,
    /// Document content (for document messages).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document: Option<WhatsAppMedia>,
    /// Audio content (for audio messages).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<WhatsAppMedia>,
    /// Video content (for video messages).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video: Option<WhatsAppMedia>,
    /// Sticker content (for sticker messages).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sticker: Option<WhatsAppMedia>,
    /// Context for replies (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<WhatsAppMessageContext>,
    /// Reaction content (for reaction messages).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reaction: Option<WhatsAppReaction>,
}

/// Text message content.
#[derive(Debug, Clone, Serialize)]
pub struct WhatsAppText {
    /// The message body.
    pub body: String,
}

/// Media message content.
#[derive(Debug, Clone, Serialize)]
pub struct WhatsAppMedia {
    /// The media ID or URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The media URL (for public URLs).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
    /// Optional caption.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    /// Optional filename (for documents).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
}

/// Context for replying to a message.
#[derive(Debug, Clone, Serialize)]
pub struct WhatsAppMessageContext {
    /// The message ID to reply to.
    #[serde(rename = "message_id")]
    pub message_id: String,
}

/// Reaction message content.
#[derive(Debug, Clone, Serialize)]
pub struct WhatsAppReaction {
    /// The message ID to react to.
    #[serde(rename = "message_id")]
    pub message_id: String,
    /// The emoji reaction.
    pub emoji: String,
}

/// WhatsApp API response for sending a message.
#[derive(Debug, Clone, Deserialize)]
pub struct SendMessageResponse {
    /// The messages array.
    pub messages: Vec<SendMessageResult>,
}

/// Result of sending a message to WhatsApp.
#[derive(Debug, Clone, Deserialize)]
pub struct SendMessageResult {
    /// The message ID.
    #[serde(rename = "id")]
    pub message_id: String,
}

// ============================================================================
// Message Sending Implementation
// ============================================================================

impl WhatsAppAccount {
    /// Build the WhatsApp API endpoint URL.
    fn build_endpoint(&self, endpoint_template: &str) -> String {
        let phone_number_id = self.config.phone_number_id.as_ref()
            .expect("Phone number ID not configured");
        format!(endpoint_template, phone_number_id)
    }

    /// Build the Authorization header for WhatsApp API requests.
    fn build_auth_header(&self) -> String {
        let api_token = self.config.api_token.as_ref()
            .expect("API token not configured");
        format!("Bearer {}", api_token)
    }

    /// Send a text message to a WhatsApp user.
    ///
    /// # Arguments
    ///
    /// * `to` - The recipient's phone number in E.164 format
    /// * `text` - The message text
    /// * `options` - Optional sending options
    ///
    /// # Returns
    ///
    /// * `Ok(MessageId)` - The ID of the sent message
    /// * `Err(SendError)` - An error if sending fails
    pub async fn send_text(
        &self,
        to: &str,
        text: &str,
        options: Option<SendOptions>,
    ) -> Result<MessageId, SendError> {
        // Validate phone number format
        self.validate_phone_number(to)?;

        // Check message length (WhatsApp has a 4096 character limit)
        if text.len() > 4096 {
            return Err(SendError::MessageTooLong(text.len()));
        }

        let options = options.unwrap_or_default();

        // Build the payload
        let mut payload = WhatsAppMessagePayload {
            messaging_product: "whatsapp".to_string(),
            to: to.to_string(),
            message_type: "text".to_string(),
            text: Some(WhatsAppText { body: text.to_string() }),
            image: None,
            document: None,
            audio: None,
            video: None,
            sticker: None,
            context: None,
            reaction: None,
        };

        // Add reply context if specified
        if let Some(reply_to) = options.reply_to {
            payload.context = Some(WhatsAppMessageContext {
                message_id: reply_to,
            });
        }

        // Build the URL and headers
        let endpoint = self.build_endpoint(MESSAGE_ENDPOINT);
        let auth_header = self.build_auth_header();

        debug!("Sending text message to {} via {}", to, endpoint);

        // Make the API request
        let client = reqwest::Client::new();
        let response = client
            .post(&endpoint)
            .header("Authorization", &auth_header)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!(
                "Failed to send text message: status={}, error={}",
                response.status(),
                error_text
            );
            return Err(SendError::ApiError(error_text));
        }

        let result: SendMessageResponse = response.json().await?;
        let message_id = result.messages
            .first()
            .map(|m| MessageId::new(&m.message_id))
            .ok_or_else(|| SendError::ApiError("No message ID in response".to_string()))?;

        info!("Sent text message to {}: {}", to, message_id);

        Ok(message_id)
    }

    /// Send a media message to a WhatsApp user.
    ///
    /// # Arguments
    ///
    /// * `to` - The recipient's phone number in E.164 format
    /// * `media` - The media to send
    /// * `options` - Optional sending options
    ///
    /// # Returns
    ///
    /// * `Ok(MessageId)` - The ID of the sent message
    /// * `Err(SendError)` - An error if sending fails
    pub async fn send_media(
        &self,
        to: &str,
        media: &Media,
        options: Option<SendOptions>,
    ) -> Result<MessageId, SendError> {
        // Validate phone number format
        self.validate_phone_number(to)?;

        let options = options.unwrap_or_default();

        // Build the payload based on media type
        let (message_type, media_content) = match media.media_type {
            aisopod_channel::types::MediaType::Image => (
                "image",
                WhatsAppMedia {
                    id: None,
                    link: media.url.clone(),
                    caption: None,
                    filename: media.filename.clone(),
                },
            ),
            aisopod_channel::types::MediaType::Document => (
                "document",
                WhatsAppMedia {
                    id: None,
                    link: media.url.clone(),
                    caption: None,
                    filename: media.filename.clone(),
                },
            ),
            aisopod_channel::types::MediaType::Audio => (
                "audio",
                WhatsAppMedia {
                    id: None,
                    link: media.url.clone(),
                    caption: None,
                    filename: None,
                },
            ),
            aisopod_channel::types::MediaType::Video => (
                "video",
                WhatsAppMedia {
                    id: None,
                    link: media.url.clone(),
                    caption: None,
                    filename: None,
                },
            ),
            aisopod_channel::types::MediaType::Other(_) => {
                return Err(SendError::ConfigError(
                    "Unsupported media type for WhatsApp".to_string(),
                ));
            }
        };

        let mut payload = WhatsAppMessagePayload {
            messaging_product: "whatsapp".to_string(),
            to: to.to_string(),
            message_type: message_type.to_string(),
            text: None,
            image: None,
            document: None,
            audio: None,
            video: None,
            sticker: None,
            context: None,
            reaction: None,
        };

        // Set the appropriate media field based on type
        match media.media_type {
            aisopod_channel::types::MediaType::Image => {
                payload.image = Some(media_content);
            }
            aisopod_channel::types::MediaType::Document => {
                payload.document = Some(media_content);
            }
            aisopod_channel::types::MediaType::Audio => {
                payload.audio = Some(media_content);
            }
            aisopod_channel::types::MediaType::Video => {
                payload.video = Some(media_content);
            }
            aisopod_channel::types::MediaType::Other(_) => {}
        }

        // Add reply context if specified
        if let Some(reply_to) = options.reply_to {
            payload.context = Some(WhatsAppMessageContext {
                message_id: reply_to,
            });
        }

        // Build the URL and headers
        let endpoint = self.build_endpoint(MESSAGE_ENDPOINT);
        let auth_header = self.build_auth_header();

        debug!("Sending media message to {} via {}", to, endpoint);

        // Make the API request
        let client = reqwest::Client::new();
        let response = client
            .post(&endpoint)
            .header("Authorization", &auth_header)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!(
                "Failed to send media message: status={}, error={}",
                response.status(),
                error_text
            );
            return Err(SendError::ApiError(error_text));
        }

        let result: SendMessageResponse = response.json().await?;
        let message_id = result.messages
            .first()
            .map(|m| MessageId::new(&m.message_id))
            .ok_or_else(|| SendError::ApiError("No message ID in response".to_string()))?;

        info!("Sent media message to {}: {}", to, message_id);

        Ok(message_id)
    }

    /// Send an outgoing message through this WhatsApp account.
    ///
    /// This method dispatches to the appropriate sending method based on
    /// the message content type.
    ///
    /// # Arguments
    ///
    /// * `message` - The outgoing message to send
    ///
    /// # Returns
    ///
    /// * `Ok(MessageId)` - The ID of the sent message
    /// * `Err(SendError)` - An error if sending fails
    pub async fn send_outgoing(
        &self,
        message: &OutgoingMessage,
    ) -> Result<MessageId, SendError> {
        let target = &message.target;
        let phone_number = &target.peer.id;

        match &message.content {
            MessageContent::Text(text) => {
                let options = SendOptions {
                    reply_to: message.reply_to.clone(),
                    ..Default::default()
                };
                self.send_text(phone_number, text, Some(options)).await
            }
            MessageContent::Media(media) => {
                let options = SendOptions {
                    reply_to: message.reply_to.clone(),
                    ..Default::default()
                };
                self.send_media(phone_number, media, Some(options)).await
            }
            MessageContent::Mixed(_) => {
                // For mixed content, we'll send the first text part as text
                // and ignore media parts for now
                let text = Self::extract_text_from_mixed(&message.content)?;
                let options = SendOptions {
                    reply_to: message.reply_to.clone(),
                    ..Default::default()
                };
                self.send_text(phone_number, &text, Some(options)).await
            }
        }
    }

    /// Extract text content from mixed message parts.
    fn extract_text_from_mixed(content: &MessageContent) -> Result<String, SendError> {
        match content {
            MessageContent::Text(text) => Ok(text.clone()),
            MessageContent::Media(_) => Ok("[Media content]".to_string()),
            MessageContent::Mixed(parts) => {
                let text_parts: Vec<String> = parts
                    .iter()
                    .filter_map(|part| {
                        match part {
                            MessagePart::Text(text) => Some(text.clone()),
                            MessagePart::Media(_) => None,
                        }
                    })
                    .collect();
                Ok(text_parts.join(" "))
            }
        }
    }

    /// Validate a phone number format.
    ///
    /// WhatsApp expects phone numbers in E.164 format (e.g., +14151234567).
    fn validate_phone_number(&self, phone_number: &str) -> Result<(), SendError> {
        if phone_number.is_empty() {
            return Err(SendError::InvalidPhoneNumber(
                "Phone number cannot be empty".to_string(),
            ));
        }

        // Basic validation: must start with + and have at least 8 digits
        if !phone_number.starts_with('+') {
            return Err(SendError::InvalidPhoneNumber(
                "Phone number must start with +".to_string(),
            ));
        }

        // Remove the + and check for digits
        let digits: String = phone_number[1..].chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() < 8 || digits.len() > 15 {
            return Err(SendError::InvalidPhoneNumber(
                "Phone number must have 8-15 digits".to_string(),
            ));
        }

        Ok(())
    }

    /// Send a read receipt for a message.
    ///
    /// # Arguments
    ///
    /// * `message_id` - The ID of the message to mark as read
    ///
    /// # Returns
    ///
    /// * `Ok(())` - The read receipt was sent successfully
    /// * `Err(SendError)` - An error if sending fails
    pub async fn send_read_receipt(
        &self,
        message_id: &str,
    ) -> Result<(), SendError> {
        let phone_number_id = self.config.phone_number_id.as_ref()
            .ok_or_else(|| SendError::ConfigError("Phone number ID not configured".to_string()))?;

        let endpoint = format!("{}/messages", phone_number_id);
        let auth_header = self.build_auth_header();

        let payload = serde_json::json!({
            "messaging_product": "whatsapp",
            "status": "read",
            "message_id": message_id
        });

        debug!("Sending read receipt for message {}", message_id);

        let client = reqwest::Client::new();
        let response = client
            .post(&endpoint)
            .header("Authorization", &auth_header)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!(
                "Failed to send read receipt: status={}, error={}",
                response.status(),
                error_text
            );
            return Err(SendError::ApiError(error_text));
        }

        info!("Sent read receipt for message {}", message_id);
        Ok(())
    }

    /// Send a "recording" status to indicate the bot is active.
    ///
    /// Note: WhatsApp doesn't have a direct "typing" API, but we can
    /// send a status update to indicate activity.
    ///
    /// # Arguments
    ///
    /// * `to` - The recipient's phone number
    /// * `status` - The status to send ("composing" or "recording")
    ///
    /// # Returns
    ///
    /// * `Ok(())` - The status was sent successfully
    /// * `Err(SendError)` - An error if sending fails
    pub async fn send_status(
        &self,
        to: &str,
        status: &str,
    ) -> Result<(), SendError> {
        self.validate_phone_number(to)?;

        let phone_number_id = self.config.phone_number_id.as_ref()
            .ok_or_else(|| SendError::ConfigError("Phone number ID not configured".to_string()))?;

        let endpoint = format!("{}/messages", phone_number_id);
        let auth_header = self.build_auth_header();

        let payload = serde_json::json!({
            "messaging_product": "whatsapp",
            "status": status,
            "to": to
        });

        trace!("Sending status '{}' to {}", status, to);

        let client = reqwest::Client::new();
        let response = client
            .post(&endpoint)
            .header("Authorization", &auth_header)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!(
                "Failed to send status: status={}, error={}",
                response.status(),
                error_text
            );
            return Err(SendError::ApiError(error_text));
        }

        info!("Sent status '{}' to {}", status, to);
        Ok(())
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use aisopod_channel::types::MediaType;
    use std::sync::Arc;

    #[test]
    fn test_send_options_default() {
        let options = SendOptions::default();
        assert!(options.reply_to.is_none());
        assert!(!options.read_receipt);
        assert!(!options.typing_indicator);
        assert!(!options.include_phone_number);
    }

    #[test]
    fn test_send_options_builder() {
        let options = SendOptions {
            reply_to: Some("msg123".to_string()),
            read_receipt: true,
            typing_indicator: true,
            include_phone_number: true,
        };

        assert_eq!(options.reply_to, Some("msg123".to_string()));
        assert!(options.read_receipt);
        assert!(options.typing_indicator);
        assert!(options.include_phone_number);
    }

    #[test]
    fn test_message_id_display() {
        let id = MessageId::new("wamid.HBgNNTU1MDAwMDAwMBUA");
        assert_eq!(id.as_str(), "wamid.HBgNNTU1MDAwMDAwMBUA");
        assert_eq!(id.to_string(), "wamid.HBgNNTU1MDAwMDAwMBUA");
    }

    #[test]
    fn test_validate_phone_number_valid() {
        // Create a minimal account config for testing
        let config = crate::connection::WhatsAppAccountConfig {
            mode: crate::connection::WhatsAppMode::BusinessApi,
            api_token: Some("test-token".to_string()),
            business_account_id: None,
            phone_number_id: Some("123456789".to_string()),
            webhook_verify_token: Some("verify-token".to_string()),
            allowed_numbers: None,
        };

        let account = WhatsAppAccount::new("test".to_string(), config);
        
        // Valid numbers should pass
        assert!(account.validate_phone_number("+14151234567").is_ok());
        assert!(account.validate_phone_number("+447911123456").is_ok());
        assert!(account.validate_phone_number("+8613800138000").is_ok());
    }

    #[test]
    fn test_validate_phone_number_invalid() {
        let config = crate::connection::WhatsAppAccountConfig {
            mode: crate::connection::WhatsAppMode::BusinessApi,
            api_token: Some("test-token".to_string()),
            business_account_id: None,
            phone_number_id: Some("123456789".to_string()),
            webhook_verify_token: Some("verify-token".to_string()),
            allowed_numbers: None,
        };

        let account = WhatsAppAccount::new("test".to_string(), config);
        
        // Invalid numbers should fail
        assert!(account.validate_phone_number("").is_err());
        assert!(account.validate_phone_number("14151234567").is_err()); // Missing +
        assert!(account.validate_phone_number("+123").is_err()); // Too short
        assert!(account.validate_phone_number("+12345678901234567").is_err()); // Too long
    }

    #[test]
    fn test_whatsapp_text_payload() {
        let text = WhatsAppText {
            body: "Hello, world!".to_string(),
        };

        let json = serde_json::to_string(&text).unwrap();
        assert_eq!(json, r#"{"body":"Hello, world!"}"#);
    }

    #[test]
    fn test_whatsapp_media_payload() {
        let media = WhatsAppMedia {
            id: Some("media123".to_string()),
            link: Some("https://example.com/image.jpg".to_string()),
            caption: Some("Check out this image!".to_string()),
            filename: Some("image.jpg".to_string()),
        };

        let json = serde_json::to_string(&media).unwrap();
        let expected = serde_json::json!({
            "id": "media123",
            "link": "https://example.com/image.jpg",
            "caption": "Check out this image!",
            "filename": "image.jpg"
        });

        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_whatsapp_message_context() {
        let context = WhatsAppMessageContext {
            message_id: "msg123".to_string(),
        };

        let json = serde_json::to_string(&context).unwrap();
        assert_eq!(json, r#"{"message_id":"msg123"}"#);
    }

    #[test]
    fn test_whatsapp_message_payload_text() {
        let payload = WhatsAppMessagePayload {
            messaging_product: "whatsapp".to_string(),
            to: "14151234567".to_string(),
            message_type: "text".to_string(),
            text: Some(WhatsAppText { body: "Hello!".to_string() }),
            image: None,
            document: None,
            audio: None,
            video: None,
            sticker: None,
            context: None,
            reaction: None,
        };

        let json = serde_json::to_string(&payload).unwrap();
        let expected = serde_json::json!({
            "messaging_product": "whatsapp",
            "to": "14151234567",
            "type": "text",
            "text": {"body": "Hello!"}
        });

        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_whatsapp_message_payload_with_context() {
        let payload = WhatsAppMessagePayload {
            messaging_product: "whatsapp".to_string(),
            to: "14151234567".to_string(),
            message_type: "text".to_string(),
            text: Some(WhatsAppText { body: "Hello!".to_string() }),
            image: None,
            document: None,
            audio: None,
            video: None,
            sticker: None,
            context: Some(WhatsAppMessageContext {
                message_id: "reply_to_msg".to_string(),
            }),
            reaction: None,
        };

        let json = serde_json::to_string(&payload).unwrap();
        let expected = serde_json::json!({
            "messaging_product": "whatsapp",
            "to": "14151234567",
            "type": "text",
            "text": {"body": "Hello!"},
            "context": {"message_id": "reply_to_msg"}
        });

        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_media_content_serialization() {
        let media = Media {
            media_type: MediaType::Image,
            url: Some("https://example.com/image.jpg".to_string()),
            data: None,
            filename: Some("image.jpg".to_string()),
            mime_type: Some("image/jpeg".to_string()),
            size_bytes: None,
        };

        // Verify media can be serialized
        let json = serde_json::to_string(&media).unwrap();
        let parsed: Media = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed.media_type, MediaType::Image);
        assert_eq!(parsed.url, Some("https://example.com/image.jpg".to_string()));
        assert_eq!(parsed.filename, Some("image.jpg".to_string()));
    }

    #[test]
    fn test_message_content_extraction() {
        let content = MessageContent::Mixed(vec![
            MessagePart::Text("Hello ".to_string()),
            MessagePart::Text("world!".to_string()),
        ]);

        let text = WhatsAppAccount::extract_text_from_mixed(&content).unwrap();
        assert_eq!(text, "Hello world!");
    }

    #[test]
    fn test_message_content_extraction_single_text() {
        let content = MessageContent::Text("Single message".to_string());

        let text = WhatsAppAccount::extract_text_from_mixed(&content).unwrap();
        assert_eq!(text, "Single message");
    }

    #[test]
    fn test_message_content_extraction_media() {
        let media = Media {
            media_type: MediaType::Image,
            url: Some("https://example.com/image.jpg".to_string()),
            data: None,
            filename: None,
            mime_type: None,
            size_bytes: None,
        };

        let content = MessageContent::Media(media);

        let text = WhatsAppAccount::extract_text_from_mixed(&content).unwrap();
        assert_eq!(text, "[Media content]");
    }

    #[test]
    fn test_send_error_display() {
        let err = SendError::InvalidPhoneNumber("+123".to_string());
        assert!(err.to_string().contains("Invalid phone number"));

        let err = SendError::ApiError("Rate limited".to_string());
        assert!(err.to_string().contains("WhatsApp API error"));

        let err = SendError::MessageTooLong(5000);
        assert!(err.to_string().contains("Message too long"));
    }

    #[test]
    fn test_error_from_anyhow() {
        let anyhow_err = anyhow::anyhow!("Test error");
        let send_err = SendError::from(anyhow_err);
        
        match send_err {
            SendError::ApiError(msg) => assert!(msg.contains("Test error")),
            _ => panic!("Expected ApiError variant"),
        }
    }
}
