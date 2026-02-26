//! Google Chat webhook endpoint handlers.
//!
//! This module provides webhook endpoints for receiving incoming events
//! from Google Chat, including messages and card interactions.

use anyhow::Result;
use axum::{
    extract::Query,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Datelike, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, info};

use crate::api::{Message, User};

/// Query parameters for webhook verification.
#[derive(Debug, Clone, Deserialize)]
pub struct WebhookVerifyQuery {
    /// The hub mode (e.g., "subscribe").
    pub hub_mode: Option<String>,
    /// The hub challenge for verification.
    pub hub_challenge: Option<String>,
    /// The hub verify token.
    pub hub_verify_token: Option<String>,
}

/// Webhook payload from Google Chat.
#[derive(Debug, Clone, Deserialize)]
pub struct WebhookPayload {
    /// Type of event.
    #[serde(rename = "type")]
    pub event_type: EventType,
    /// Timestamp of the event.
    pub timestamp: String,
    /// Event ID.
    pub event_id: String,
    /// Space where the event occurred.
    pub space: Space,
    /// User who triggered the event.
    pub user: Option<User>,
    /// Message that triggered the event (for message events).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<Message>,
    /// Card action data (for card button clicks).
    #[serde(rename = "cardAction", skip_serializing_if = "Option::is_none")]
    pub card_action: Option<WebhookCardAction>,
}

/// Event type.
#[derive(Debug, Clone, Deserialize, PartialEq, PartialOrd, Eq, Ord)]
pub enum EventType {
    /// A user sent a message.
    #[serde(rename = "MESSAGE")]
    Message,
    /// A user created a room.
    #[serde(rename = "ROOM_CREATED")]
    RoomCreated,
    /// A user joined a room.
    #[serde(rename = "USER_JOINED")]
    UserJoined,
    /// A user left a room.
    #[serde(rename = "USER_LEFT")]
    UserLeft,
    /// A user was added to a room.
    #[serde(rename = "USER_ADDED")]
    UserAdded,
    /// A user was removed from a room.
    #[serde(rename = "USER_REMOVED")]
    UserRemoved,
    /// A room's settings were updated.
    #[serde(rename = "ROOM_UPDATED")]
    RoomUpdated,
    /// A card button was clicked.
    #[serde(rename = "CARD_CLICKED")]
    CardClicked,
}

/// Space information.
#[derive(Debug, Clone, Deserialize)]
pub struct Space {
    /// Resource name of the space.
    pub name: String,
    /// Space type.
    #[serde(rename = "type")]
    pub space_type: String,
    /// Space display name.
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
}

/// Card action data from webhook.
#[derive(Debug, Clone, Deserialize)]
pub struct WebhookCardAction {
    /// Action name.
    #[serde(rename = "actionName")]
    pub action_name: Option<String>,
    /// Action ID.
    #[serde(rename = "actionId")]
    pub action_id: Option<String>,
    /// User who clicked the action.
    pub user: Option<User>,
    /// Parameters associated with the action.
    #[serde(default)]
    pub parameters: Vec<ActionParameter>,
    /// Resource name of the message containing the card.
    #[serde(rename = "resourceName")]
    pub resource_name: Option<String>,
}

/// Card action parameter.
#[derive(Debug, Clone, Deserialize)]
pub struct ActionParameter {
    /// Parameter name.
    pub name: String,
    /// Parameter value.
    pub value: String,
}

/// Webhook response.
#[derive(Debug, Clone, Serialize)]
pub struct WebhookResponse {
    /// Status of the webhook processing.
    pub status: String,
    /// Optional message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Webhook handler state.
#[derive(Clone)]
pub struct WebhookState {
    /// The webhook verify token.
    pub verify_token: String,
    /// The account ID for this webhook.
    pub account_id: String,
    /// The channel identifier.
    pub channel: String,
    /// Optional allowed space IDs.
    pub allowed_spaces: Option<Vec<String>>,
    /// Optional callback function for processing events.
    #[allow(dead_code)]
    pub event_callback: Option<Arc<dyn Fn(WebhookPayload) -> Result<()> + Send + Sync>>,
}

impl WebhookState {
    /// Create a new webhook state.
    pub fn new(verify_token: impl Into<String>, account_id: impl Into<String>, channel: impl Into<String>) -> Self {
        Self {
            verify_token: verify_token.into(),
            account_id: account_id.into(),
            channel: channel.into(),
            allowed_spaces: None,
            event_callback: None,
        }
    }

    /// Set allowed space IDs.
    pub fn allowed_spaces(mut self, spaces: Vec<String>) -> Self {
        self.allowed_spaces = Some(spaces);
        self
    }

    /// Set event callback function.
    pub fn event_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(WebhookPayload) -> Result<()> + Send + Sync + 'static,
    {
        self.event_callback = Some(Arc::new(callback));
        self
    }

    /// Check if a space is allowed.
    pub fn is_space_allowed(&self, space_id: &str) -> bool {
        if let Some(ref allowed_spaces) = self.allowed_spaces {
            allowed_spaces.iter().any(|s| s == space_id)
        } else {
            true // If no allowlist, allow all spaces
        }
    }
}

/// Create a webhook router with the given state.
///
/// # Arguments
///
/// * `state` - The webhook state containing configuration
///
/// # Returns
///
/// An Axum Router with webhook endpoints
pub fn create_webhook_router(state: WebhookState) -> Router {
    Router::new()
        .route("/webhook", get(webhook_verify_handler))
        .route("/webhook", post(webhook_message_handler))
        .with_state(Arc::new(state))
}

/// Handle webhook verification GET request.
///
/// This handler responds to Google Chat's webhook verification challenge.
/// Google Chat sends a GET request with query parameters to verify the webhook URL.
async fn webhook_verify_handler(
    Query(query): Query<WebhookVerifyQuery>,
    state: axum::extract::State<Arc<WebhookState>>,
) -> impl IntoResponse {
    debug!(
        "Webhook verification request: mode={:?}, challenge={:?}, verify_token={:?}",
        query.hub_mode, query.hub_challenge, query.hub_verify_token
    );

    // Check if this is a subscription request
    if query.hub_mode.as_deref() != Some("subscribe") {
        error!("Invalid webhook mode: {:?}", query.hub_mode);
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Invalid mode"})),
        );
    }

    // Verify the token
    if query.hub_verify_token != Some(state.verify_token.clone()) {
        error!(
            "Invalid verify token: expected '{}', got {:?}",
            state.verify_token,
            query.hub_verify_token
        );
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Invalid verify token"})),
        );
    }

    // Return the challenge
    info!("Webhook verification successful");
    let challenge = query.hub_challenge.clone();
    (
        StatusCode::OK,
        Json(serde_json::json!({ "hub_challenge": challenge })),
    )
}

/// Handle incoming event POST request.
///
/// This handler receives webhook payloads from Google Chat when events occur.
async fn webhook_message_handler(
    state: axum::extract::State<Arc<WebhookState>>,
    Json(payload): Json<WebhookPayload>,
) -> impl IntoResponse {
    info!(
        "Received webhook event: type={:?}, space={}",
        payload.event_type,
        payload.space.name
    );

    // Validate the space
    let space_id = payload.space.name.strip_prefix("spaces/").unwrap_or(&payload.space.name);
    if !state.is_space_allowed(space_id) {
        error!(
            "Event from space {} not in allowed list",
            payload.space.name
        );
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Space not allowed"})),
        );
    }

    // Log event details based on type
    match payload.event_type {
        EventType::Message => {
            if let Some(ref message) = payload.message {
                info!(
                    "Message from {}: {}",
                    message.sender.as_ref().map(|s| s.display_name.as_deref().unwrap_or("unknown")).unwrap_or("unknown"),
                    message.text.as_deref().unwrap_or("")
                );
            }
        }
        EventType::RoomCreated => {
            info!(
                "Room created: {}",
                payload.space.display_name.as_deref().unwrap_or(&payload.space.name)
            );
        }
        EventType::UserJoined | EventType::UserAdded => {
            if let Some(ref user) = payload.user {
                info!(
                    "User joined: {} ({})",
                    user.display_name.as_deref().unwrap_or("unknown"),
                    user.email.as_deref().unwrap_or("")
                );
            }
        }
        EventType::UserLeft | EventType::UserRemoved => {
            if let Some(ref user) = payload.user {
                info!(
                    "User left: {} ({})",
                    user.display_name.as_deref().unwrap_or("unknown"),
                    user.email.as_deref().unwrap_or("")
                );
            }
        }
        EventType::CardClicked => {
            info!(
                "Card action: {}",
                payload.card_action.as_ref().and_then(|a| a.action_name.as_deref()).unwrap_or("unknown")
            );
        }
        _ => {
            info!("Received unknown event type: {:?}", payload.event_type);
        }
    }

    // Call the event callback if registered
    if let Some(ref callback) = state.event_callback {
        if let Err(e) = callback(payload.clone()) {
            error!("Event callback failed: {}", e);
        }
    }

    (StatusCode::OK, Json(serde_json::json!({"status": "received"})))
}

/// Parse a timestamp string to DateTime.
pub fn parse_timestamp(timestamp: &str) -> Result<DateTime<Utc>> {
    // Google Chat timestamps are in RFC 3339 format
    // Example: "2021-04-15T10:30:00.000000000Z"
    let timestamp = timestamp.trim_end_matches('Z');
    let timestamp = timestamp.trim_end_matches('0').trim_end_matches('.');
    
    // Try parsing with nanoseconds first
    if let Ok(dt) = DateTime::parse_from_rfc3339(timestamp) {
        return Ok(dt.with_timezone(&Utc));
    }
    
    // Try with just seconds
    let timestamp = format!("{}Z", timestamp);
    DateTime::parse_from_rfc3339(&timestamp)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| anyhow::anyhow!("Failed to parse timestamp '{}': {}", timestamp, e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn test_webhook_verify_query_deserialization() {
        // Test that the struct can be constructed and has the right fields
        let query = WebhookVerifyQuery {
            hub_mode: Some("subscribe".to_string()),
            hub_challenge: Some("123456".to_string()),
            hub_verify_token: Some("mytoken".to_string()),
        };
        
        assert_eq!(query.hub_mode, Some("subscribe".to_string()));
        assert_eq!(query.hub_challenge, Some("123456".to_string()));
        assert_eq!(query.hub_verify_token, Some("mytoken".to_string()));
    }

    #[test]
    fn test_webhook_payload_deserialization_message() {
        let payload = r#"{
            "type": "MESSAGE",
            "timestamp": "2021-04-15T10:30:00.000000000Z",
            "event_id": "event123",
            "space": {
                "name": "spaces/SPACE123",
                "type": "SPACE",
                "displayName": "Test Space"
            },
            "user": {
                "name": "users/USER123",
                "displayName": "John Doe",
                "email": "john@example.com",
                "type": "HUMAN"
            },
            "message": {
                "name": "spaces/SPACE123/messages/MESSAGE123",
                "sender": {
                    "name": "users/USER123",
                    "displayName": "John Doe",
                    "email": "john@example.com",
                    "type": "HUMAN"
                },
                "text": "Hello, world!",
                "createTime": "2021-04-15T10:30:00.000000000Z"
            }
        }"#;

        let result: Result<WebhookPayload, _> = serde_json::from_str(payload);
        assert!(result.is_ok());
        let payload = result.unwrap();
        
        assert_eq!(payload.event_type, EventType::Message);
        assert_eq!(payload.space.name, "spaces/SPACE123");
        assert_eq!(payload.message.as_ref().unwrap().text, Some("Hello, world!".to_string()));
    }

    #[test]
    fn test_webhook_payload_deserialization_card_clicked() {
        let payload = r#"{
            "type": "CARD_CLICKED",
            "timestamp": "2021-04-15T10:30:00.000000000Z",
            "event_id": "event456",
            "space": {
                "name": "spaces/SPACE123",
                "type": "SPACE"
            },
            "user": {
                "name": "users/USER123",
                "displayName": "John Doe",
                "type": "HUMAN"
            },
            "cardAction": {
                "actionName": "update_status",
                "actionId": "update_button",
                "parameters": [
                    {
                        "name": "status",
                        "value": "complete"
                    }
                ]
            }
        }"#;

        let result: Result<WebhookPayload, _> = serde_json::from_str(payload);
        assert!(result.is_ok());
        let payload = result.unwrap();
        
        assert_eq!(payload.event_type, EventType::CardClicked);
        assert_eq!(payload.card_action.as_ref().unwrap().action_name, Some("update_status".to_string()));
    }

    #[test]
    fn test_event_type_deserialization() {
        let event: EventType = serde_json::from_str("\"MESSAGE\"").unwrap();
        assert_eq!(event, EventType::Message);

        let event: EventType = serde_json::from_str("\"ROOM_CREATED\"").unwrap();
        assert_eq!(event, EventType::RoomCreated);

        let event: EventType = serde_json::from_str("\"CARD_CLICKED\"").unwrap();
        assert_eq!(event, EventType::CardClicked);
    }

    #[test]
    fn test_webhook_state_default() {
        let state = WebhookState::new("mytoken", "account1", "googlechat");
        
        assert_eq!(state.verify_token, "mytoken");
        assert_eq!(state.account_id, "account1");
        assert_eq!(state.channel, "googlechat");
        assert!(state.allowed_spaces.is_none());
        assert!(state.event_callback.is_none());
    }

    #[test]
    fn test_webhook_state_allowed_spaces() {
        let state = WebhookState::new("mytoken", "account1", "googlechat")
            .allowed_spaces(vec!["SPACE123".to_string(), "SPACE456".to_string()]);
        
        assert!(state.is_space_allowed("SPACE123"));
        assert!(state.is_space_allowed("SPACE456"));
        assert!(!state.is_space_allowed("SPACE789"));
    }

    #[test]
    fn test_parse_timestamp() {
        let timestamp = "2021-04-15T10:30:00.000000000Z";
        let result = parse_timestamp(timestamp);
        assert!(result.is_ok());
        
        let dt = result.unwrap();
        assert_eq!(dt.year(), 2021);
        assert_eq!(dt.month(), 4);
        assert_eq!(dt.day(), 15);
    }

    #[test]
    fn test_webhook_state_with_callback() {
        let callback_called = std::sync::Arc::new(AtomicBool::new(false));
        let callback_called_clone = callback_called.clone();
        
        let state = WebhookState::new("mytoken", "account1", "googlechat")
            .event_callback(move |payload: WebhookPayload| {
                callback_called_clone.store(true, std::sync::atomic::Ordering::SeqCst);
                Ok(())
            });
        
        if let Some(ref callback) = state.event_callback {
            let _ = callback(WebhookPayload {
                event_type: EventType::Message,
                timestamp: "2021-04-15T10:30:00.000000000Z".to_string(),
                event_id: "event123".to_string(),
                space: Space {
                    name: "spaces/SPACE123".to_string(),
                    space_type: "SPACE".to_string(),
                    display_name: None,
                },
                user: None,
                message: None,
                card_action: None,
            });
        }
        
        assert!(callback_called.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[test]
    fn test_webhook_response_serialization() {
        let response = WebhookResponse {
            status: "received".to_string(),
            message: Some("Success".to_string()),
        };
        
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("received"));
        assert!(json.contains("Success"));
    }

    #[test]
    fn test_webhook_state_without_allowlist() {
        let state = WebhookState::new("mytoken", "account1", "googlechat");
        
        // Without allowlist, all spaces are allowed
        assert!(state.is_space_allowed("SPACE123"));
        assert!(state.is_space_allowed("SPACE456"));
    }

    #[test]
    fn test_card_action_deserialization() {
        let payload = r#"{
            "type": "CARD_CLICKED",
            "timestamp": "2021-04-15T10:30:00.000000000Z",
            "event_id": "event456",
            "space": {
                "name": "spaces/SPACE123",
                "type": "SPACE"
            },
            "user": {
                "name": "users/USER123",
                "displayName": "John Doe",
                "type": "HUMAN"
            },
            "cardAction": {
                "actionName": "update_status",
                "actionId": "update_button",
                "parameters": [
                    {
                        "name": "status",
                        "value": "complete"
                    }
                ],
                "resourceName": "spaces/SPACE123/messages/MESSAGE123"
            }
        }"#;

        let result: Result<WebhookPayload, _> = serde_json::from_str(payload);
        assert!(result.is_ok());
        
        let payload = result.unwrap();
        let card_action = payload.card_action.unwrap();
        
        assert_eq!(card_action.action_name, Some("update_status".to_string()));
        assert_eq!(card_action.action_id, Some("update_button".to_string()));
        assert_eq!(card_action.resource_name, Some("spaces/SPACE123/messages/MESSAGE123".to_string()));
        assert_eq!(card_action.parameters.len(), 1);
        assert_eq!(card_action.parameters[0].name, "status");
        assert_eq!(card_action.parameters[0].value, "complete");
    }
}
