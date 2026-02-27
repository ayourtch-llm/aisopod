//! Mock Microsoft Teams Bot Framework for integration testing.
//!
//! This module provides a mock HTTP server that simulates the Microsoft Bot Framework,
//! allowing integration tests to verify Teams channel behavior without requiring
//! real Azure AD credentials or making actual API calls.

use axum::{
    extract::Json,
    http::StatusCode,
    response::IntoResponse,
    routing::{post, get},
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// Mock Microsoft Teams Bot Framework server state.
pub struct MockTeamsServer {
    /// The base URL of the mock server
    pub base_url: String,
    /// Join handle for the server task
    _handle: tokio::task::JoinHandle<()>,
}

impl MockTeamsServer {
    /// Starts a mock Microsoft Teams Bot Framework server.
    ///
    /// The server provides endpoints for:
    /// - Token acquisition: POST /{tenant}/oauth2/v2.0/token
    /// - Bot activities: POST /v3/conversations/{conversationId}/activities
    /// - Get conversation members: GET /v3/conversations/{conversationId}/members
    ///
    /// # Returns
    ///
    /// A tuple of (base_url, server_handle).
    pub async fn start() -> Self {
        let app = Router::new()
            .route("/{tenant}/oauth2/v2.0/token", post(mock_token))
            .route("/v3/conversations/{conversationId}/activities", post(send_activity))
            .route("/v3/conversations/{conversationId}/members", get(get_members))
            .route("/v3/conversations", post(create_conversation));

        // Bind to a random available port
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind to random port");
        let addr = listener.local_addr().expect("Failed to get local addr");
        
        let base_url = format!("http://{}", addr);

        let handle = tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("Mock server failed");
        });

        Self {
            base_url,
            _handle: handle,
        }
    }

    /// Returns the base URL of the mock server.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

/// Request for OAuth token.
#[derive(Debug, Deserialize)]
struct TokenRequest {
    grant_type: String,
    client_id: String,
    client_secret: String,
    scope: String,
}

/// Response for OAuth token.
#[derive(Debug, Serialize)]
struct TokenResponse {
    token_type: String,
    expires_in: u64,
    access_token: String,
}

/// Bot Framework activity.
#[derive(Debug, Deserialize)]
struct Activity {
    #[serde(rename = "type")]
    activity_type: String,
    id: Option<String>,
    channel_id: Option<String>,
    conversation: Option<Conversation>,
    from: Option<ChannelAccount>,
    recipient: Option<ChannelAccount>,
    text: Option<String>,
    text_format: Option<String>,
    locale: Option<String>,
    attachments: Option<Vec<serde_json::Value>>,
    entities: Option<Vec<serde_json::Value>>,
}

/// Conversation information.
#[derive(Debug, Deserialize)]
struct Conversation {
    id: String,
    is_group: bool,
    #[serde(rename = "conversationType")]
    conversation_type: Option<String>,
    name: Option<String>,
}

/// Channel account (user or bot).
#[derive(Debug, Serialize, Deserialize)]
struct ChannelAccount {
    id: String,
    name: Option<String>,
    #[serde(rename = "aadObjectId")]
    aad_object_id: Option<String>,
}

/// Response for sending an activity.
#[derive(Debug, Serialize)]
struct SendActivityResponse {
    id: String,
    etag: Option<String>,
}

/// Response for creating a conversation.
#[derive(Debug, Serialize)]
struct CreateConversationResponse {
    id: String,
    activity_id: String,
    etag: Option<String>,
    service_url: Option<String>,
}

/// Response for getting conversation members.
#[derive(Debug, Serialize)]
struct GetMembersResponse {
    members: Vec<ChannelAccount>,
}

/// Handler for OAuth token endpoint.
async fn mock_token(
    Json(payload): Json<TokenRequest>,
) -> Result<Json<TokenResponse>, (StatusCode, String)> {
    // Validate required fields
    if payload.grant_type != "client_credentials" {
        return Err((
            StatusCode::BAD_REQUEST,
            "grant_type must be 'client_credentials'".to_string(),
        ));
    }

    if payload.client_id.is_empty() || payload.client_secret.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "client_id and client_secret are required".to_string(),
        ));
    }

    Ok(Json(TokenResponse {
        token_type: "Bearer".to_string(),
        expires_in: 3600,
        access_token: format!("mock-token-{}", uuid::Uuid::new_v4()),
    }))
}

/// Handler for sending an activity.
async fn send_activity(
    axum::extract::Path(conversation_id): axum::extract::Path<String>,
    Json(payload): Json<Activity>,
) -> Result<Json<SendActivityResponse>, (StatusCode, String)> {
    // Validate activity type
    match payload.activity_type.as_str() {
        "message" | "typing" | "conversationUpdate" => {
            // Valid activity type
        }
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("Invalid activity type: {}", payload.activity_type),
            ));
        }
    }

    Ok(Json(SendActivityResponse {
        id: format!("activity-{}", uuid::Uuid::new_v4()),
        etag: Some(format!("\"{}\"", uuid::Uuid::new_v4())),
    }))
}

/// Handler for getting conversation members.
async fn get_members(
    axum::extract::Path(conversation_id): axum::extract::Path<String>,
) -> Json<GetMembersResponse> {
    Json(GetMembersResponse {
        members: vec![
            ChannelAccount {
                id: "bot-id".to_string(),
                name: Some("Mock Bot".to_string()),
                aad_object_id: Some("bot-object-id".to_string()),
            },
            ChannelAccount {
                id: "user-123".to_string(),
                name: Some("Test User".to_string()),
                aad_object_id: Some("user-object-id".to_string()),
            },
        ],
    })
}

/// Handler for creating a conversation.
async fn create_conversation(
    Json(payload): Json<serde_json::Value>,
) -> Json<CreateConversationResponse> {
    let conversation_id = format!("conversation-{}", uuid::Uuid::new_v4());
    
    Json(CreateConversationResponse {
        id: conversation_id,
        activity_id: format!("activity-{}", uuid::Uuid::new_v4()),
        etag: Some(format!("\"{}\"", uuid::Uuid::new_v4())),
        service_url: Some("http://localhost:3978".to_string()),
    })
}
