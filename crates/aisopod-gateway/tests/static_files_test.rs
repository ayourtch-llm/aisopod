//! Integration tests for static file serving

use axum::{
    body::Body,
    extract::State,
    http::{Method, Request, StatusCode},
    response::IntoResponse,
    Router,
};
use std::net::{SocketAddr, TcpListener};
use tokio::net::TcpStream;
use tower::Service;

// Import the static files module
use aisopod_gateway::static_files::{StaticFileState, static_handler};
use aisopod_config::types::WebUiConfig;

#[tokio::test]
async fn test_serve_index_html() {
    let config = WebUiConfig::default();
    let state = StaticFileState::new(config);
    
    let uri = "/".parse().unwrap();
    let response = static_handler(axum::extract::State(state.clone()), uri).await;
    
    let response = response.into_response();
    
    // Check that we got a 200 status
    assert_eq!(response.status(), StatusCode::OK);
    
    // Check Content-Type header
    let content_type = response.headers().get("content-type").unwrap();
    assert!(content_type.to_str().unwrap().contains("text/html"));
    
    // Check Cache-Control header
    let cache_control = response.headers().get("cache-control").unwrap();
    assert_eq!(cache_control.to_str().unwrap(), "no-cache");
}

#[tokio::test]
async fn test_serve_css_file() {
    let config = WebUiConfig::default();
    let state = StaticFileState::new(config);
    
    let uri = "/styles/main.css".parse().unwrap();
    let response = static_handler(axum::extract::State(state.clone()), uri).await;
    
    let response = response.into_response();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    // Check Content-Type header for CSS
    let content_type = response.headers().get("content-type").unwrap();
    assert!(content_type.to_str().unwrap().contains("text/css"));
}

#[tokio::test]
async fn test_serve_js_file() {
    let config = WebUiConfig::default();
    let state = StaticFileState::new(config);
    
    let uri = "/scripts/main.js".parse().unwrap();
    let response = static_handler(axum::extract::State(state.clone()), uri).await;
    
    let response = response.into_response();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    // Check Content-Type header for JavaScript
    let content_type = response.headers().get("content-type").unwrap();
    assert!(content_type.to_str().unwrap().contains("application/javascript"));
}

#[tokio::test]
async fn test_serve_hashed_asset() {
    let config = WebUiConfig::default();
    let state = StaticFileState::new(config);
    
    // Hashed asset should have long-lived cache
    let uri = "/assets/main.abc123def456.js".parse().unwrap();
    let response = static_handler(axum::extract::State(state.clone()), uri).await;
    
    let response = response.into_response();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    // Check Cache-Control header for hashed asset
    let cache_control = response.headers().get("cache-control").unwrap();
    let cache_val = cache_control.to_str().unwrap();
    assert!(cache_val.contains("immutable"), "Expected immutable cache: {}", cache_val);
    assert!(cache_val.contains("max-age=31536000"), "Expected max-age=31536000: {}", cache_val);
}

#[tokio::test]
async fn test_spa_fallback_for_unknown_path() {
    let config = WebUiConfig::default();
    let state = StaticFileState::new(config);
    
    // Unknown path that's not an API route should return index.html
    let uri = "/unknown/path".parse().unwrap();
    let response = static_handler(axum::extract::State(state.clone()), uri).await;
    
    let response = response.into_response();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    // Check that we got index.html
    let content_type = response.headers().get("content-type").unwrap();
    assert!(content_type.to_str().unwrap().contains("text/html"));
}

#[tokio::test]
async fn test_api_route_returns_not_found() {
    let config = WebUiConfig::default();
    let state = StaticFileState::new(config);
    
    // API routes should return 404 (they are handled by a different router)
    let uri = "/api/test".parse().unwrap();
    let response = static_handler(axum::extract::State(state.clone()), uri).await;
    
    let response = response.into_response();
    
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_nonexistent_file_returns_index_html() {
    let config = WebUiConfig::default();
    let state = StaticFileState::new(config);
    
    // Non-existent file should return index.html (SPA fallback)
    let uri = "/nonexistent.file".parse().unwrap();
    let response = static_handler(axum::extract::State(state.clone()), uri).await;
    
    let response = response.into_response();
    
    // SPA fallback should return index.html
    assert_eq!(response.status(), StatusCode::OK);
    
    let content_type = response.headers().get("content-type").unwrap();
    assert!(content_type.to_str().unwrap().contains("text/html"));
}

#[tokio::test]
async fn test_png_file_content_type() {
    let config = WebUiConfig::default();
    let state = StaticFileState::new(config);
    
    let uri = "/assets/logo.png".parse().unwrap();
    let response = static_handler(axum::extract::State(state.clone()), uri).await;
    
    let response = response.into_response();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let content_type = response.headers().get("content-type").unwrap();
    assert!(content_type.to_str().unwrap().contains("image/png"));
}

#[tokio::test]
async fn test_root_path() {
    let config = WebUiConfig::default();
    let state = StaticFileState::new(config);
    
    let uri = "/".parse().unwrap();
    let response = static_handler(axum::extract::State(state.clone()), uri).await;
    
    let response = response.into_response();
    
    // Should get 200 OK
    assert_eq!(response.status(), StatusCode::OK);
}
