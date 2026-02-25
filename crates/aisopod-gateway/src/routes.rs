#![allow(clippy::all)]
use axum::{
    extract::{ConnectInfo, Extension, MatchedPath, State, WebSocketUpgrade},
    http::Method,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::auth::DeviceTokenManager;
use aisopod_config::types::AuthConfig;

/// Build the device token management routes
pub fn device_token_routes() -> Router {
    use axum::routing::{get, post};

    Router::new()
        .route("/device-tokens", get(list_device_tokens))
        .route("/device-tokens", post(issue_device_token))
        .route("/device-tokens/revoke", post(revoke_device_token))
        .route("/device-tokens/refresh", post(refresh_device_token))
}

/// Handler for not implemented endpoints
pub async fn not_implemented(
    method: Method,
    matched_path: MatchedPath,
    ConnectInfo(client_ip): ConnectInfo<std::net::SocketAddr>,
) -> impl IntoResponse {
    tracing::info!(
        method = %method,
        path = %matched_path.as_str(),
        client_ip = %client_ip,
        "Request to unimplemented endpoint"
    );

    (
        axum::http::StatusCode::NOT_IMPLEMENTED,
        Json(json!({"error": "not implemented"})),
    )
}

/// Gateway status information
#[derive(Debug, Serialize, Deserialize)]
pub struct GatewayStatus {
    /// Number of configured agents
    pub agent_count: usize,
    /// Number of active channels
    pub active_channels: usize,
    /// Number of active sessions
    pub active_sessions: usize,
    /// Gateway uptime in seconds
    pub uptime: u64,
}

/// Status endpoint handler
pub async fn status(
    State(state): State<Arc<GatewayStatusState>>,
) -> impl IntoResponse {
    let status = GatewayStatus {
        agent_count: state.agent_count.load(std::sync::atomic::Ordering::Relaxed),
        active_channels: state.active_channels.load(std::sync::atomic::Ordering::Relaxed),
        active_sessions: state.active_sessions.load(std::sync::atomic::Ordering::Relaxed),
        uptime: state.start_time.elapsed().as_secs(),
    };
    Json(json!(status))
}

/// Status state for the gateway
#[derive(Debug)]
pub struct GatewayStatusState {
    /// Start time of the gateway
    pub start_time: Instant,
    /// Number of configured agents (atomic)
    pub agent_count: std::sync::atomic::AtomicUsize,
    /// Number of active channels (atomic)
    pub active_channels: std::sync::atomic::AtomicUsize,
    /// Number of active sessions (atomic)
    pub active_sessions: std::sync::atomic::AtomicUsize,
}

impl GatewayStatusState {
    /// Create a new GatewayStatusState with the given initial counts
    pub fn new(agent_count: usize, active_channels: usize, active_sessions: usize) -> Self {
        Self {
            start_time: Instant::now(),
            agent_count: std::sync::atomic::AtomicUsize::new(agent_count),
            active_channels: std::sync::atomic::AtomicUsize::new(active_channels),
            active_sessions: std::sync::atomic::AtomicUsize::new(active_sessions),
        }
    }

    /// Update the agent count
    pub fn set_agent_count(&self, count: usize) {
        self.agent_count.store(count, std::sync::atomic::Ordering::Relaxed);
    }

    /// Update the active channel count
    pub fn set_active_channels(&self, count: usize) {
        self.active_channels.store(count, std::sync::atomic::Ordering::Relaxed);
    }

    /// Update the active session count
    pub fn set_active_sessions(&self, count: usize) {
        self.active_sessions.store(count, std::sync::atomic::Ordering::Relaxed);
    }
}

impl Default for GatewayStatusState {
    fn default() -> Self {
        Self::new(0, 0, 0)
    }
}

/// Build the API router with all REST endpoint stubs
pub fn api_routes(status_state: Option<Arc<GatewayStatusState>>) -> Router {
    use axum::routing::{get, post};

    Router::new()
        .route(
            "/v1/chat/completions",
            get(not_implemented).post(not_implemented),
        )
        .route("/v1/responses", get(not_implemented).post(not_implemented))
        .route("/hooks", get(not_implemented).post(not_implemented))
        .route("/tools/invoke", get(not_implemented).post(not_implemented))
        .route(
            "/status",
            match status_state {
                Some(state) => get(move || async move { status(State(state)).await }),
                None => get(not_implemented),
            },
        )
}

/// Handler for listing device tokens
pub async fn list_device_tokens(
    Extension(manager): Extension<Arc<Mutex<DeviceTokenManager>>>,
) -> Json<serde_json::Value> {
    let mgr = manager.lock().unwrap();
    let tokens = mgr.list();
    Json(json!({
        "tokens": tokens
    }))
}

/// Request body for issuing a new device token
#[derive(Debug, Deserialize)]
pub struct IssueTokenRequest {
    pub device_name: String,
    pub device_id: Option<String>,
    pub scopes: Vec<String>,
}

/// Response for issuing a new device token
#[derive(Debug, Serialize)]
pub struct IssueTokenResponse {
    pub token: String,
    pub device_id: String,
    pub message: String,
}

/// Handler for issuing a new device token
pub async fn issue_device_token(
    Extension(manager): Extension<Arc<Mutex<DeviceTokenManager>>>,
    Json(payload): Json<IssueTokenRequest>,
) -> Json<serde_json::Value> {
    let mut mgr = manager.lock().unwrap();
    let device_id = payload.device_id.unwrap_or_else(|| {
        format!(
            "device-{}",
            uuid::Uuid::new_v4().simple().to_string()
        )
    });

    // Convert scope strings to Scope enums
    let scopes: Vec<crate::auth::Scope> = payload
        .scopes
        .iter()
        .filter_map(|s| match s.as_str() {
            "operator.admin" => Some(crate::auth::Scope::OperatorAdmin),
            "operator.read" => Some(crate::auth::Scope::OperatorRead),
            "operator.write" => Some(crate::auth::Scope::OperatorWrite),
            "operator.approvals" => Some(crate::auth::Scope::OperatorApprovals),
            "operator.pairing" => Some(crate::auth::Scope::OperatorPairing),
            _ => None,
        })
        .collect();

    match mgr
        .issue(payload.device_name, device_id.clone(), scopes)
    {
        Ok(token) => Json(json!(IssueTokenResponse {
            token,
            device_id,
            message: "Device token issued successfully".to_string(),
        })),
        Err(e) => Json(json!({
            "error": "failed to issue token",
            "message": e.to_string()
        })),
    }
}

/// Response for revoking a device token
#[derive(Debug, Serialize)]
pub struct RevokeTokenResponse {
    pub success: bool,
    pub message: String,
}

/// Handler for revoking a device token
pub async fn revoke_device_token(
    Extension(manager): Extension<Arc<Mutex<DeviceTokenManager>>>,
    Json(payload): Json<RevokeDeviceTokenRequest>,
) -> Json<serde_json::Value> {
    let mut mgr = manager.lock().unwrap();
    match mgr.revoke(&payload.device_id) {
        Ok(true) => Json(json!(RevokeTokenResponse {
            success: true,
            message: format!("Token for device '{}' revoked", payload.device_id),
        })),
        Ok(false) => Json(json!(RevokeTokenResponse {
            success: false,
            message: format!("No token found for device '{}'", payload.device_id),
        })),
        Err(e) => Json(json!({
            "error": "failed to revoke token",
            "message": e.to_string()
        })),
    }
}

/// Request body for revoking a device token
#[derive(Debug, Deserialize)]
pub struct RevokeDeviceTokenRequest {
    pub device_id: String,
}

/// Response for refreshing a device token
#[derive(Debug, Serialize)]
pub struct RefreshTokenResponse {
    pub token: String,
    pub device_id: String,
    pub message: String,
}

/// Handler for refreshing a device token
pub async fn refresh_device_token(
    Extension(manager): Extension<Arc<Mutex<DeviceTokenManager>>>,
    Json(payload): Json<RevokeDeviceTokenRequest>,
) -> Json<serde_json::Value> {
    let mut mgr = manager.lock().unwrap();
    match mgr.refresh(&payload.device_id) {
        Ok(Some(token)) => Json(json!(RefreshTokenResponse {
            token,
            device_id: payload.device_id,
            message: "Device token refreshed successfully".to_string(),
        })),
        Ok(None) => Json(json!({
            "error": "no token found or token revoked",
            "device_id": payload.device_id
        })),
        Err(e) => Json(json!({
            "error": "failed to refresh token",
            "message": e.to_string()
        })),
    }
}

/// Build WebSocket routes
pub fn ws_routes(handshake_timeout: Option<u64>) -> Router {
    Router::new().route(
        "/ws",
        get(
            move |ws: WebSocketUpgrade, request: axum::extract::Request| {
                crate::ws::ws_handler(ws, request, handshake_timeout)
            },
        ),
    )
}
