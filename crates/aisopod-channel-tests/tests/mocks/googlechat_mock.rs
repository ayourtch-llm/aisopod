//! Mock Google Chat API server for integration testing.
//!
//! This module provides a mock HTTP server that simulates the Google Chat API,
//! allowing integration tests to verify Google Chat channel behavior without
//! requiring real API credentials or making actual API calls.

use axum::{
    extract::Json,
    http::StatusCode,
    response::IntoResponse,
    routing::{post, get},
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::sync::oneshot;

/// Mock Google Chat API server state.
pub struct MockGoogleChatServer {
    /// The base URL of the mock server
    pub base_url: String,
    /// Join handle for the server task
    _handle: tokio::task::JoinHandle<()>,
}

impl MockGoogleChatServer {
    /// Starts a mock Google Chat API server.
    ///
    /// The server provides endpoints for:
    /// - Creating messages: POST /v1/spaces/{space}/messages
    /// - Listing spaces: GET /v1/spaces
    /// - OAuth token endpoint: POST /oauth2/v4/token
    ///
    /// # Returns
    ///
    /// A tuple of (base_url, server_handle).
    pub async fn start() -> Self {
        let app = Router::new()
            .route("/v1/spaces", get(list_spaces))
            .route("/v1/spaces/:space/messages", post(create_message))
            .route("/oauth2/v4/token", post(mock_token))
            .route("/v1/people/@me", get(get_self_user))
            .route("/v1/users/*path", get(get_user));

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

/// Response for listing spaces.
#[derive(Debug, Serialize)]
struct ListSpacesResponse {
    spaces: Vec<Space>,
}

/// Google Chat space representation.
#[derive(Debug, Serialize)]
struct Space {
    name: String,
    display_name: String,
    #[serde(rename = "type")]
    space_type: String,
}

/// Request for creating a message.
#[derive(Debug, Deserialize)]
struct CreateMessageRequest {
    text: Option<String>,
    #[serde(rename = "cardWithHeader")]
    card_with_header: Option<serde_json::Value>,
}

/// Response for creating a message.
#[derive(Debug, Serialize)]
struct CreateMessageResponse {
    name: String,
}

/// Response for getting the self user.
#[derive(Debug, Serialize)]
struct SelfUser {
    name: String,
    display_name: String,
    emails: Vec<serde_json::Value>,
}

/// Handler for listing spaces.
async fn list_spaces() -> Json<ListSpacesResponse> {
    Json(ListSpacesResponse {
        spaces: vec![
            Space {
                name: "spaces/TESTSPACE1".to_string(),
                display_name: "Test Space".to_string(),
                space_type: "SPACE".to_string(),
            },
            Space {
                name: "spaces/GROUP1".to_string(),
                display_name: "Test Group".to_string(),
                space_type: "GROUP".to_string(),
            },
        ],
    })
}

/// Handler for creating a message.
async fn create_message(
    axum::extract::Path(space): axum::extract::Path<String>,
    Json(payload): Json<CreateMessageRequest>,
) -> Result<Json<CreateMessageResponse>, (StatusCode, String)> {
    
    // Validate the request
    if payload.text.is_none() && payload.card_with_header.is_none() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Either 'text' or 'cardWithHeader' must be provided".to_string(),
        ));
    }

    Ok(Json(CreateMessageResponse {
        name: format!("{}/messages/{}", space, uuid::Uuid::new_v4()),
    }))
}

/// Handler for getting the self user.
async fn get_self_user() -> Json<SelfUser> {
    Json(SelfUser {
        name: "users/@me".to_string(),
        display_name: "Mock Bot".to_string(),
        emails: vec![serde_json::json!({"value": "bot@example.com"})],
    })
}

/// Handler for getting a user.
async fn get_user(
    axum::extract::Path(path): axum::extract::Path<String>,
) -> Result<Json<SelfUser>, (StatusCode, String)> {
    // Extract user ID from path (e.g., "users/12345" or "users/@me")
    if path == "users/@me" || path.contains("@me") {
        return Ok(Json(SelfUser {
            name: "users/@me".to_string(),
            display_name: "Mock Bot".to_string(),
            emails: vec![serde_json::json!({"value": "bot@example.com"})],
        }));
    }

    // Extract user ID from path like "users/12345"
    let user_id = path
        .strip_prefix("users/")
        .unwrap_or(&path)
        .split('/')
        .next()
        .unwrap_or("unknown");

    Ok(Json(SelfUser {
        name: format!("users/{}", user_id),
        display_name: format!("User {}", user_id),
        emails: vec![serde_json::json!({"value": format!("user{}@example.com", user_id)})],
    }))
}

/// Handler for OAuth token endpoint.
async fn mock_token() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "access_token": "mock-access-token-12345",
        "expires_in": 3600,
        "token_type": "Bearer"
    }))
}
