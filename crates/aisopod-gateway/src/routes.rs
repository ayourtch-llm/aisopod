use axum::{
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde_json::json;

/// Handler for not implemented endpoints
pub async fn not_implemented() -> impl IntoResponse {
    (axum::http::StatusCode::NOT_IMPLEMENTED, Json(json!({"error": "not implemented"})))
}

/// Build the API router with all REST endpoint stubs
pub fn api_routes() -> Router {
    Router::new()
        .route("/v1/chat/completions", get(not_implemented))
        .route("/v1/responses", get(not_implemented))
        .route("/hooks", get(not_implemented))
        .route("/tools/invoke", get(not_implemented))
        .route("/status", get(not_implemented))
}
