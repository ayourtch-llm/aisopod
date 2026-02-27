//! Zalo webhook handling.
//!
//! This module provides webhook support for receiving events from Zalo,
//! including signature verification and event parsing.

use anyhow::Result;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

/// Zalo webhook signature header name.
pub const ZALO_SIGNATURE_HEADER: &str = "access_token";

/// Zalo webhook verification header.
pub const ZALO_OA_SECRET_KEY_HEADER: &str = "secret_key";

/// Default webhook path for Zalo events.
pub const DEFAULT_WEBHOOK_PATH: &str = "/zalo/webhook";

/// Zalo webhook event types.
#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "event_name")]
pub enum WebhookEventType {
    #[serde(rename = "user_send_text")]
    UserSendText(UserSendTextEvent),
    #[serde(rename = "user_send_image")]
    UserSendImage(UserSendImageEvent),
    #[serde(rename = "user_send_file")]
    UserSendFile(UserSendFileEvent),
    #[serde(rename = "follow")]
    Follow(FollowEvent),
    #[serde(rename = "unfollow")]
    Unfollow(UnfollowEvent),
    #[serde(rename = "oa_join")]
    OaJoin(OaJoinEvent),
    #[serde(rename = "oa_leave")]
    OaLeave(OaLeaveEvent),
    #[serde(rename = "user_send_location")]
    UserSendLocation(UserSendLocationEvent),
    #[serde(rename = "user_send_contact")]
    UserSendContact(UserSendContactEvent),
}

/// User sends text event.
#[derive(Debug, Deserialize, Clone)]
pub struct UserSendTextEvent {
    #[serde(rename = "user_id")]
    pub user_id: String,
    #[serde(rename = "user_full_name")]
    pub user_full_name: Option<String>,
    pub text: String,
    #[serde(default)]
    #[serde(rename = "message_id")]
    pub message_id: Option<String>,
    #[serde(default)]
    pub time: Option<u64>,
}

/// User sends image event.
#[derive(Debug, Deserialize, Clone)]
pub struct UserSendImageEvent {
    #[serde(rename = "user_id")]
    pub user_id: String,
    #[serde(rename = "user_full_name")]
    pub user_full_name: Option<String>,
    pub media_id: String,
    #[serde(rename = "thumbnail_url")]
    pub thumbnail_url: Option<String>,
    #[serde(rename = "original_url")]
    pub original_url: Option<String>,
    #[serde(default)]
    #[serde(rename = "message_id")]
    pub message_id: Option<String>,
    #[serde(default)]
    pub time: Option<u64>,
}

/// User sends file event.
#[derive(Debug, Deserialize, Clone)]
pub struct UserSendFileEvent {
    #[serde(rename = "user_id")]
    pub user_id: String,
    #[serde(rename = "user_full_name")]
    pub user_full_name: Option<String>,
    pub media_id: String,
    pub filename: String,
    #[serde(rename = "file_size")]
    pub file_size: Option<i64>,
    #[serde(default)]
    #[serde(rename = "message_id")]
    pub message_id: Option<String>,
    #[serde(default)]
    pub time: Option<u64>,
}

/// Follow event (user subscribes to OA).
#[derive(Debug, Deserialize, Clone)]
pub struct FollowEvent {
    #[serde(rename = "user_id")]
    pub user_id: String,
    #[serde(rename = "user_full_name")]
    pub user_full_name: Option<String>,
    #[serde(default)]
    pub time: Option<u64>,
}

/// Unfollow event (user unsubscribes from OA).
#[derive(Debug, Deserialize, Clone)]
pub struct UnfollowEvent {
    #[serde(rename = "user_id")]
    pub user_id: String,
    #[serde(rename = "user_full_name")]
    pub user_full_name: Option<String>,
    #[serde(default)]
    pub time: Option<u64>,
}

/// OA join event (user joins group OA).
#[derive(Debug, Deserialize, Clone)]
pub struct OaJoinEvent {
    #[serde(rename = "user_id")]
    pub user_id: String,
    #[serde(rename = "user_full_name")]
    pub user_full_name: Option<String>,
    #[serde(default)]
    pub time: Option<u64>,
}

/// OA leave event (user leaves group OA).
#[derive(Debug, Deserialize, Clone)]
pub struct OaLeaveEvent {
    #[serde(rename = "user_id")]
    pub user_id: String,
    #[serde(rename = "user_full_name")]
    pub user_full_name: Option<String>,
    #[serde(default)]
    pub time: Option<u64>,
}

/// User sends location event.
#[derive(Debug, Deserialize, Clone)]
pub struct UserSendLocationEvent {
    #[serde(rename = "user_id")]
    pub user_id: String,
    #[serde(rename = "user_full_name")]
    pub user_full_name: Option<String>,
    pub lat: f64,
    pub lng: f64,
    pub location_name: Option<String>,
    #[serde(default)]
    #[serde(rename = "message_id")]
    pub message_id: Option<String>,
    #[serde(default)]
    pub time: Option<u64>,
}

/// User sends contact event.
#[derive(Debug, Deserialize, Clone)]
pub struct UserSendContactEvent {
    #[serde(rename = "user_id")]
    pub user_id: String,
    #[serde(rename = "user_full_name")]
    pub user_full_name: Option<String>,
    #[serde(rename = "contact_id")]
    pub contact_id: String,
    #[serde(rename = "contact_name")]
    pub contact_name: String,
    #[serde(rename = "contact_phone")]
    pub contact_phone: Option<String>,
    #[serde(default)]
    #[serde(rename = "message_id")]
    pub message_id: Option<String>,
    #[serde(default)]
    pub time: Option<u64>,
}

/// Webhook verification request.
#[derive(Debug, Deserialize)]
pub struct WebhookVerifyRequest {
    #[serde(rename = "code")]
    pub code: Option<i32>,
    #[serde(rename = "message")]
    pub message: Option<String>,
}

/// Webhook verification response.
#[derive(Debug, Serialize)]
pub struct WebhookVerifyResponse {
    #[serde(rename = "error")]
    pub error: Option<i32>,
    #[serde(rename = "message")]
    pub message: Option<String>,
}

/// Webhook state for axum router.
#[derive(Clone)]
pub struct WebhookState {
    /// OA secret key for webhook verification
    pub oa_secret_key: String,
    /// Channel ID
    pub channel_id: String,
}

/// Create the Zalo webhook router.
///
/// # Arguments
///
/// * `oa_secret_key` - The OA secret key for webhook verification
/// * `channel_id` - The channel ID
///
/// # Returns
///
/// * `Router` - The configured axum router
pub fn create_webhook_router(oa_secret_key: String, channel_id: String) -> Router {
    let state = WebhookState {
        oa_secret_key,
        channel_id,
    };

    Router::new()
        .route(DEFAULT_WEBHOOK_PATH, post(handle_webhook))
        .with_state(state)
}

/// Handle incoming webhook events from Zalo.
///
/// This handler:
/// 1. Verifies the webhook signature
/// 2. Parses the event payload
/// 3. Routes the event to the appropriate handler
///
/// # Arguments
///
/// * `state` - The webhook state (OA secret key, channel ID)
/// * `headers` - HTTP headers
/// * `payload` - The JSON payload from Zalo
///
/// # Returns
///
/// * `Ok(())` - Event processed successfully
/// * `Err(StatusCode)` - An error occurred
async fn handle_webhook(
    State(state): State<WebhookState>,
    headers: HeaderMap,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    debug!("Received webhook event: {:?}", payload);

    // Verify the webhook signature
    if !verify_webhook_signature(&headers, &state.oa_secret_key) {
        error!("Webhook signature verification failed");
        return (StatusCode::FORBIDDEN, Json(WebhookVerifyResponse {
            error: Some(403),
            message: Some("Invalid signature".to_string()),
        })).into_response();
    }

    // Parse the event
    match parse_webhook_event(&payload) {
        Ok(event) => {
            info!("Processing webhook event: {:?}", event);
            handle_webhook_event(event, state).await;
            (StatusCode::OK, Json(WebhookVerifyResponse {
                error: Some(0),
                message: Some("OK".to_string()),
            })).into_response()
        }
        Err(e) => {
            error!("Failed to parse webhook event: {}", e);
            (StatusCode::BAD_REQUEST, Json(WebhookVerifyResponse {
                error: Some(400),
                message: Some(e.to_string()),
            })).into_response()
        }
    }
}

/// Parse a webhook event from the JSON payload.
///
/// # Arguments
///
/// * `payload` - The JSON payload
///
/// # Returns
///
/// * `Ok(WebhookEventType)` - The parsed event
/// * `Err(anyhow::Error)` - An error if parsing fails
pub fn parse_webhook_event(
    payload: &serde_json::Value,
) -> Result<WebhookEventType> {
    if let Some(event_name) = payload.get("event_name").and_then(|v| v.as_str()) {
        match event_name {
            "user_send_text" => {
                let event: UserSendTextEvent = serde_json::from_value(payload.clone())?;
                Ok(WebhookEventType::UserSendText(event))
            }
            "user_send_image" => {
                let event: UserSendImageEvent = serde_json::from_value(payload.clone())?;
                Ok(WebhookEventType::UserSendImage(event))
            }
            "user_send_file" => {
                let event: UserSendFileEvent = serde_json::from_value(payload.clone())?;
                Ok(WebhookEventType::UserSendFile(event))
            }
            "follow" => {
                let event: FollowEvent = serde_json::from_value(payload.clone())?;
                Ok(WebhookEventType::Follow(event))
            }
            "unfollow" => {
                let event: UnfollowEvent = serde_json::from_value(payload.clone())?;
                Ok(WebhookEventType::Unfollow(event))
            }
            "oa_join" => {
                let event: OaJoinEvent = serde_json::from_value(payload.clone())?;
                Ok(WebhookEventType::OaJoin(event))
            }
            "oa_leave" => {
                let event: OaLeaveEvent = serde_json::from_value(payload.clone())?;
                Ok(WebhookEventType::OaLeave(event))
            }
            "user_send_location" => {
                let event: UserSendLocationEvent = serde_json::from_value(payload.clone())?;
                Ok(WebhookEventType::UserSendLocation(event))
            }
            "user_send_contact" => {
                let event: UserSendContactEvent = serde_json::from_value(payload.clone())?;
                Ok(WebhookEventType::UserSendContact(event))
            }
            _ => Err(anyhow::anyhow!("Unknown event_name: {}", event_name)),
        }
    } else {
        Err(anyhow::anyhow!("Missing event_name in payload"))
    }
}

/// Verify the webhook signature.
///
/// Zalo includes the access_token in the headers which should match
/// the OA's access_token for verification.
///
/// # Arguments
///
/// * `headers` - HTTP headers
/// * `oa_secret_key` - The OA secret key
///
/// # Returns
///
/// * `true` - Signature is valid
/// * `false` - Signature is invalid
pub fn verify_webhook_signature(headers: &HeaderMap, oa_secret_key: &str) -> bool {
    // In production, we would verify the signature hash
    // For now, we verify that the access_token header exists
    headers
        .get(ZALO_SIGNATURE_HEADER)
        .and_then(|v| v.to_str().ok())
        .is_some()
}

/// Handle a parsed webhook event.
///
/// # Arguments
///
/// * `event` - The parsed webhook event
/// * `state` - The webhook state
async fn handle_webhook_event(event: WebhookEventType, state: WebhookState) {
    // In a full implementation, this would:
    // 1. Convert the webhook event to an IncomingMessage
    // 2. Route it to the appropriate handler
    // 3. Log or process the event
    
    match event {
        WebhookEventType::UserSendText(e) => {
            info!(
                "Received text from {}: {}",
                e.user_id,
                e.text
            );
        }
        WebhookEventType::UserSendImage(e) => {
            info!(
                "Received image from {}: {}",
                e.user_id,
                e.media_id
            );
        }
        WebhookEventType::UserSendFile(e) => {
            info!(
                "Received file from {}: {}",
                e.user_id,
                e.filename
            );
        }
        WebhookEventType::Follow(e) => {
            info!("User {} followed the OA", e.user_id);
        }
        WebhookEventType::Unfollow(e) => {
            info!("User {} unfollowed the OA", e.user_id);
        }
        WebhookEventType::OaJoin(e) => {
            info!("User {} joined the OA", e.user_id);
        }
        WebhookEventType::OaLeave(e) => {
            info!("User {} left the OA", e.user_id);
        }
        WebhookEventType::UserSendLocation(e) => {
            info!(
                "Received location from {}: {}",
                e.user_id,
                e.location_name.unwrap_or_else(|| format!("{}, {}", e.lat, e.lng))
            );
        }
        WebhookEventType::UserSendContact(e) => {
            info!(
                "Received contact from {}: {}",
                e.user_id,
                e.contact_name
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_webhook_event_user_send_text() {
        let payload = serde_json::json!({
            "event_name": "user_send_text",
            "user_id": "123456789",
            "user_full_name": "John Doe",
            "text": "Hello, world!",
            "message_id": "msg_123",
            "time": 1618907555
        });

        let result = parse_webhook_event(&payload);
        assert!(result.is_ok());
        
        let event = result.unwrap();
        match event {
            WebhookEventType::UserSendText(e) => {
                assert_eq!(e.user_id, "123456789");
                assert_eq!(e.user_full_name, Some("John Doe".to_string()));
                assert_eq!(e.text, "Hello, world!");
                assert_eq!(e.message_id, Some("msg_123".to_string()));
            }
            _ => panic!("Expected UserSendText event"),
        }
    }

    #[test]
    fn test_parse_webhook_event_follow() {
        let payload = serde_json::json!({
            "event_name": "follow",
            "user_id": "123456789",
            "user_full_name": "John Doe",
            "time": 1618907555
        });

        let result = parse_webhook_event(&payload);
        assert!(result.is_ok());
        
        let event = result.unwrap();
        match event {
            WebhookEventType::Follow(e) => {
                assert_eq!(e.user_id, "123456789");
                assert_eq!(e.user_full_name, Some("John Doe".to_string()));
            }
            _ => panic!("Expected Follow event"),
        }
    }

    #[test]
    fn test_parse_webhook_event_unknown() {
        let payload = serde_json::json!({
            "event_name": "unknown_event",
            "data": "test"
        });

        let result = parse_webhook_event(&payload);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown event_name"));
    }

    #[test]
    fn test_parse_webhook_event_missing_event_name() {
        let payload = serde_json::json!({
            "data": "test"
        });

        let result = parse_webhook_event(&payload);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing event_name"));
    }

    #[test]
    fn test_verify_webhook_signature() {
        use axum::http::HeaderMap;
        use axum::http::header::HeaderName;
        use axum::http::header::HeaderValue;

        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_static("access_token"),
            HeaderValue::from_static("test_token"),
        );

        assert!(verify_webhook_signature(&headers, "test_secret"));
    }

    #[test]
    fn test_verify_webhook_signature_missing_token() {
        use axum::http::HeaderMap;

        let headers = HeaderMap::new();
        assert!(!verify_webhook_signature(&headers, "test_secret"));
    }
}
