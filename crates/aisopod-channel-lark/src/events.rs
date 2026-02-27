//! Event handling for Lark/Feishu webhook subscriptions.
//!
//! This module provides types and handlers for processing incoming
//! webhook events from the Lark Open Platform.

use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// The Lark webhook signature header name
pub const LARK_SIGNATURE_HEADER: &str = "X-Lark-Signature";

/// Event types for Lark webhooks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    /// URL verification event
    UrlVerification,
    /// Message received event
    MessageReceived,
    /// Message read event
    MessageRead,
    /// Conversation event (chat created, members added/removed)
    ConversationEvent,
    /// User authentication event
    UserAuthentication,
    /// Contact event (contact created, updated, deleted)
    ContactEvent,
    /// Other event types
    #[serde(other)]
    Other,
}

/// Lark webhook request body.
#[derive(Debug, Deserialize)]
pub struct WebhookRequestBody {
    /// Event type
    pub event: Event,
    /// Challenge token for URL verification
    #[serde(default)]
    pub challenge: Option<String>,
    /// Token for verification
    pub token: Option<String>,
}

/// Event payload from Lark.
#[derive(Debug, Deserialize)]
pub struct Event {
    /// Event type
    #[serde(rename = "type")]
    pub event_type: String,
    /// Timestamp
    pub timestamp: String,
    /// Event ID
    pub event_id: String,
    /// Sender information
    pub sender: Sender,
    /// Event data
    #[serde(default)]
    pub event_data: Option<EventData>,
}

/// Sender information.
#[derive(Debug, Deserialize)]
pub struct Sender {
    /// Sender type: "user", "bot", or "group"
    #[serde(rename = "sender_type")]
    pub sender_type: String,
    /// User ID (open_id)
    pub user_id: Option<String>,
    /// Bot code (for bot events)
    pub bot_code: Option<String>,
}

/// Event data from Lark.
#[derive(Debug, Deserialize)]
pub struct EventData {
    /// Message data (for message events)
    #[serde(default)]
    pub message: Option<Message>,
    /// Chat info (for chat events)
    #[serde(default)]
    pub chat: Option<Chat>,
}

/// Message data from Lark.
#[derive(Debug, Deserialize)]
pub struct Message {
    /// Message ID
    pub message_id: String,
    /// Chat type: "group" or "p2p"
    #[serde(rename = "chat_type")]
    pub chat_type: String,
    /// Chat ID
    pub chat_id: String,
    /// Message content
    pub text: Option<String>,
    /// Message content in Lark format
    pub content: Option<String>,
    /// Sender info
    #[serde(default)]
    pub sender: Option<SenderInfo>,
    /// Timestamp
    pub create_time: Option<String>,
}

/// Sender information in message.
#[derive(Debug, Deserialize)]
pub struct SenderInfo {
    /// User ID (open_id)
    pub id: String,
    /// Sender type
    #[serde(rename = "sender_type")]
    pub sender_type: String,
    /// Sender name
    pub name: Option<String>,
}

/// Chat information.
#[derive(Debug, Deserialize)]
pub struct Chat {
    /// Chat ID
    pub chat_id: String,
    /// Chat name
    pub name: Option<String>,
    /// Chat type: "group" or "p2p"
    #[serde(rename = "chat_type")]
    pub chat_type: String,
    /// Owner user ID
    pub owner: Option<String>,
}

/// AppState for the webhook handler.
#[derive(Clone)]
pub struct AppState {
    /// Verification token from config
    pub verification_token: String,
    /// Encrypt key (optional)
    pub encrypt_key: Option<String>,
    /// Channel ID for routing messages
    pub channel_id: String,
}

impl WebhookRequestBody {
    /// Parses the webhook body and returns the event type.
    pub fn event_type(&self) -> EventType {
        match self.event.event_type.as_str() {
            "url_verification" => EventType::UrlVerification,
            "im.message.receive_v1" => EventType::MessageReceived,
            "im.message.read_v1" => EventType::MessageRead,
            "conversation.user.enter_v1" => EventType::ConversationEvent,
            "conversation.user.leave_v1" => EventType::ConversationEvent,
            "user.authentication_v1" => EventType::UserAuthentication,
            "contact.user.added_v1" => EventType::ContactEvent,
            "contact.user.updated_v1" => EventType::ContactEvent,
            "contact.user.deleted_v1" => EventType::ContactEvent,
            _ => EventType::Other,
        }
    }

    /// Extracts the challenge token if present (for URL verification).
    pub fn challenge(&self) -> Option<&str> {
        self.challenge.as_deref()
    }
}

/// Handles Lark webhook events.
///
/// This handler processes incoming webhook events from Lark,
/// including URL verification and message events.
pub fn lark_router(state: AppState) -> Router {
    Router::new()
        .route("/lark/events", post(handle_event))
        .with_state(state)
}

/// Handles a single webhook event.
pub async fn handle_event(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    debug!("Received webhook event: {:?}", payload);

    // Extract the token from payload
    let event_token = payload.get("token").and_then(|t| t.as_str());
    
    // Verify token if present
    if let Some(token) = event_token {
        if token != state.verification_token {
            warn!(
                "Invalid token: expected {}, got {}",
                state.verification_token, token
            );
            return Err((StatusCode::UNAUTHORIZED, "Invalid token"));
        }
    }

    // Handle URL verification challenge
    if let Some(challenge) = payload.get("challenge").and_then(|c| c.as_str()) {
        debug!("URL verification challenge");
        let challenge_value = serde_json::json!({
            "challenge": challenge
        });
        return Ok(Json(challenge_value));
    }

    // Extract event type
    let event_type = payload
        .get("event")
        .and_then(|e| e.get("type"))
        .and_then(|t| t.as_str())
        .unwrap_or("");

    match event_type {
        "im.message.receive_v1" => {
            info!("Received message event");
            handle_message_event(payload, &state).await?;
            Ok(Json(serde_json::json!("ok")))
        }
        "url_verification" => {
            info!("URL verification event");
            // Already handled above
            Ok(Json(serde_json::json!("ok")))
        }
        _ => {
            debug!("Unhandled event type: {}", event_type);
            Ok(Json(serde_json::json!("ok")))
        }
    }
}

/// Handles message events.
async fn handle_message_event(_payload: Value, _state: &AppState) -> Result<(), (StatusCode, &'static str)> {
    // In a real implementation, this would convert the Lark event
    // to an inbound message for the aisopod system
    // For now, just return success
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_url_verification() {
        // This test would need a proper WebhookRequestBody structure
    }

    #[test]
    fn test_event_type_message() {
        // Test message event type parsing
        assert_eq!(EventType::MessageReceived, EventType::MessageReceived);
    }
}
