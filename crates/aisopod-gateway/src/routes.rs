use axum::{
    extract::{MatchedPath, ConnectInfo},
    http::Method,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde_json::json;

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
    
    (axum::http::StatusCode::NOT_IMPLEMENTED, Json(json!({"error": "not implemented"})))
}

/// Build the API router with all REST endpoint stubs
pub fn api_routes() -> Router {
    use axum::routing::post;
    
    Router::new()
        .route("/v1/chat/completions", post(not_implemented))
        .route("/v1/responses", post(not_implemented))
        .route("/hooks", post(not_implemented))
        .route("/tools/invoke", get(not_implemented))
        .route("/status", get(not_implemented))
}

/// Build WebSocket routes
pub fn ws_routes() -> Router {
    Router::new()
        .route("/ws", get(crate::ws::ws_handler))
}
