//! Webhook endpoint for incoming Bot Framework activities.
//!
//! This module provides webhook functionality for receiving Bot Framework activities
//! from Microsoft Teams. It includes HTTP endpoint setup and activity parsing.

use crate::auth::MsTeamsAuth;
use crate::botframework::{Activity, ChannelAccount};
use crate::config::WebhookConfig;
use anyhow::Result;
use axum::response::Response;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use chrono::{DateTime, Utc};
use jsonwebtoken::{decode, DecodingKey, EncodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Webhook state for the Axum router.
#[derive(Clone)]
pub struct WebhookState {
    /// The Bot Framework client
    pub client: crate::botframework::BotFrameworkClient,
    /// The account configuration
    pub account_id: String,
    /// The Microsoft App ID for webhook validation
    pub microsoft_app_id: String,
    /// The Microsoft App Password for webhook validation
    pub microsoft_app_password: String,
}

impl WebhookState {
    /// Creates a new webhook state.
    pub fn new(
        client: crate::botframework::BotFrameworkClient,
        account_id: &str,
        microsoft_app_id: &str,
        microsoft_app_password: &str,
    ) -> Self {
        Self {
            client,
            account_id: account_id.to_string(),
            microsoft_app_id: microsoft_app_id.to_string(),
            microsoft_app_password: microsoft_app_password.to_string(),
        }
    }
}

/// Webhook handler for incoming activities.
pub async fn handle_webhook(
    State(state): State<Arc<WebhookState>>,
    Json(activity): Json<Activity>,
) -> impl IntoResponse {
    debug!("Received webhook activity: {:?}", activity.activity_type);

    // Validate the activity
    if let Err(e) = validate_activity(&activity, &state) {
        error!("Invalid activity: {}", e);
        return (
            StatusCode::BAD_REQUEST,
            [("content-type", "text/plain")],
            format!("Invalid activity: {}", e),
        );
    }

    // Process the activity
    match process_activity(activity, state).await {
        Ok(_) => (
            StatusCode::OK,
            [("content-type", "application/json")],
            serde_json::to_string(&serde_json::json!({"status": "success"}))
                .unwrap_or_else(|_| "success".to_string()),
        ),
        Err(e) => {
            error!("Failed to process activity: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                [("content-type", "text/plain")],
                format!("Failed to process activity: {}", e),
            )
        }
    }
}

/// Validates an incoming activity.
fn validate_activity(activity: &Activity, state: &WebhookState) -> Result<()> {
    // Validate required fields
    if activity.activity_type.is_none() {
        return Err(anyhow::anyhow!("Missing activity type"));
    }

    if activity.conversation.is_none() {
        return Err(anyhow::anyhow!("Missing conversation reference"));
    }

    Ok(())
}

/// Processes an incoming activity.
async fn process_activity(activity: Activity, state: Arc<WebhookState>) -> Result<()> {
    match activity.activity_type.as_ref() {
        Some(activity_type) => match activity_type {
            crate::botframework::ActivityType::Message => {
                info!(
                    "Received message from {:?}: {}",
                    activity.from.as_ref().and_then(|a| a.id.as_ref()),
                    activity.text.as_deref().unwrap_or("")
                );
                // Process the message
                // This would typically involve sending the message to the agent engine
                Ok(())
            }
            crate::botframework::ActivityType::Typing => {
                info!("Received typing indicator from {:?}", activity.from);
                Ok(())
            }
            crate::botframework::ActivityType::ConversationUpdate => {
                info!("Received conversation update: {:?}", activity.action);
                Ok(())
            }
            crate::botframework::ActivityType::Event => {
                info!("Received event: {:?}", activity.name);
                Ok(())
            }
            crate::botframework::ActivityType::Invoke => {
                info!("Received invoke: {:?}", activity.name);
                Ok(())
            }
            _ => {
                warn!("Received unhandled activity type: {:?}", activity_type);
                Ok(())
            }
        },
        None => {
            error!("Activity type is None");
            Err(anyhow::anyhow!("Invalid activity: missing type"))
        }
    }
}

/// Creates an HTTP router for the webhook endpoint.
pub fn create_webhook_router(state: WebhookState) -> Router<Arc<WebhookState>> {
    Router::new()
        .route("/", post(handle_webhook))
        .route("/health", get(health_check))
        .with_state(Arc::new(state))
}

/// Health check endpoint for the webhook.
async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, [("content-type", "text/plain")], "OK")
}

/// Microsoft App ID validator.
pub fn validate_microsoft_app_id(token: &str, app_id: &str) -> Result<()> {
    let decoding_key = DecodingKey::from_secret(app_id.as_bytes());
    let validation = Validation::new(jsonwebtoken::Algorithm::HS256);

    decode::<serde_json::Value>(token, &decoding_key, &validation)
        .map_err(|e| anyhow::anyhow!("Invalid Microsoft App ID: {}", e))?;

    Ok(())
}

/// Creates a webhook validation response.
pub fn create_validation_response() -> serde_json::Value {
    serde_json::json!({
        "validationResponse": "valid"
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_activity() {
        let activity = Activity::create_message("Hello", None);
        let state = WebhookState::new(
            crate::botframework::BotFrameworkClient::new(
                crate::auth::MsTeamsAuth::new(crate::auth::AzureAuthConfig::new(
                    "tenant", "client", "secret",
                )),
                "app_id",
            ),
            "account1",
            "app_id",
            "app_password",
        );

        let result = validate_activity(&activity, &state);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_activity_missing_type() {
        let mut activity = Activity::default();
        activity.activity_type = None;

        let state = WebhookState::new(
            crate::botframework::BotFrameworkClient::new(
                crate::auth::MsTeamsAuth::new(crate::auth::AzureAuthConfig::new(
                    "tenant", "client", "secret",
                )),
                "app_id",
            ),
            "account1",
            "app_id",
            "app_password",
        );

        let result = validate_activity(&activity, &state);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_activity_missing_conversation() {
        let mut activity = Activity::create_message("Hello", None);
        activity.conversation = None;

        let state = WebhookState::new(
            crate::botframework::BotFrameworkClient::new(
                crate::auth::MsTeamsAuth::new(crate::auth::AzureAuthConfig::new(
                    "tenant", "client", "secret",
                )),
                "app_id",
            ),
            "account1",
            "app_id",
            "app_password",
        );

        let result = validate_activity(&activity, &state);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_webhook_router() {
        let azure_config =
            crate::auth::AzureAuthConfig::new("tenant_id", "client_id", "client_secret");
        let auth = MsTeamsAuth::new(azure_config);
        let client = crate::botframework::BotFrameworkClient::new(auth, "app_id");
        let state = WebhookState::new(client, "test_account", "app_id", "app_password");
        let _router = create_webhook_router(state);
        // Router creation should succeed (not fail)
        // Note: Router doesn't have a simple way to check if it's valid in axum 0.7
    }

    #[test]
    fn test_create_validation_response() {
        let response = create_validation_response();
        assert_eq!(response["validationResponse"], "valid");
    }
}
