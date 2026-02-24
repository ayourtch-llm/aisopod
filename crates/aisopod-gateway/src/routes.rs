#![allow(clippy::all)]
use axum::{
    extract::{ConnectInfo, MatchedPath, State, WebSocketUpgrade},
    http::Method,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use std::time::Instant;

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
