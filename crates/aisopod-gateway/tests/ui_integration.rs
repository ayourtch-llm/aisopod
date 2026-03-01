//! Integration tests for the Web UI
//!
//! These tests verify that the embedded Web UI works correctly,
//! including static file serving, SPA fallback routing, and WebSocket connectivity.

#![allow(clippy::all)]

use axum::http::StatusCode;
use axum_test::TestServer;
use std::str::FromStr;

// Import the gateway app builder
use aisopod_config::types::AuthConfig;
use aisopod_gateway::build_app;

#[tokio::test]
async fn test_serves_index_html() {
    let app = build_app(AuthConfig::default()).await;
    let server = TestServer::new(app).unwrap();

    let response = server.get("/").await;
    assert_eq!(response.status_code(), StatusCode::OK);

    let content_type = response
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(content_type.contains("text/html"));

    let body = response.text();
    assert!(body.contains("Aisopod"));
}

#[tokio::test]
async fn test_serves_js_assets() {
    let app = build_app(AuthConfig::default()).await;
    let server = TestServer::new(app).unwrap();

    // Assuming Vite outputs a JS bundle — adjust path as needed
    let response = server.get("/assets/index-BoczOECE.js").await;

    // Should be 200 if the file exists
    assert_eq!(
        response.status_code(),
        StatusCode::OK,
        "JS asset should be served"
    );
}

#[tokio::test]
async fn test_serves_css_with_correct_mime() {
    let app = build_app(AuthConfig::default()).await;
    let server = TestServer::new(app).unwrap();

    let response = server.get("/assets/index-BQE4UcBt.css").await;

    // Should be 200 if the file exists
    assert_eq!(
        response.status_code(),
        StatusCode::OK,
        "CSS asset should be served"
    );

    let content_type = response
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(content_type.contains("text/css"));
}

#[tokio::test]
async fn test_spa_fallback_serves_index() {
    let app = build_app(AuthConfig::default()).await;
    let server = TestServer::new(app).unwrap();

    // Unknown route should serve index.html for client-side routing
    let response = server.get("/chat").await;
    assert_eq!(response.status_code(), StatusCode::OK);

    let content_type = response
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(content_type.contains("text/html"));

    let body = response.text();
    assert!(body.contains("Aisopod"));
}

#[tokio::test]
async fn test_spa_fallback_deep_routes() {
    let app = build_app(AuthConfig::default()).await;
    let server = TestServer::new(app).unwrap();

    for route in ["/agents", "/channels", "/config", "/sessions", "/models"] {
        let response = server.get(route).await;
        assert_eq!(response.status_code(), StatusCode::OK);
        let body = response.text();
        assert!(
            body.contains("Aisopod"),
            "Route {} should return index.html containing Aisopod",
            route
        );
    }
}

#[tokio::test]
async fn test_websocket_connection() {
    use futures::{SinkExt, StreamExt};
    use std::net::TcpStream;
    use tokio_tungstenite::WebSocketStream;

    let app = build_app(AuthConfig::default()).await;

    // Create a server manually since axum-test doesn't expose ws directly
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Give the server a moment to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Try to connect to WebSocket
    let (stream, _) = tokio_tungstenite::connect_async(format!("ws://{}/ws", addr))
        .await
        .expect("WebSocket connection should succeed");

    let (mut ws_tx, mut ws_rx) = stream.split();

    // Send a JSON-RPC ping/health check
    ws_tx
        .send(tokio_tungstenite::tungstenite::Message::Text(
            serde_json::json!({
                "jsonrpc": "2.0",
                "method": "system.ping",
                "id": 1
            })
            .to_string(),
        ))
        .await
        .expect("Failed to send ping");

    // Receive response
    let response = ws_rx.next().await.expect("Should receive a response");
    if let Ok(tokio_tungstenite::tungstenite::Message::Text(text)) = response {
        assert!(
            text.contains("jsonrpc"),
            "Response should be JSON-RPC format: {}",
            text
        );
    }
}

#[tokio::test]
async fn test_index_html_contains_theme_support() {
    let app = build_app(AuthConfig::default()).await;
    let server = TestServer::new(app).unwrap();

    let response = server.get("/").await;
    let body = response.text();

    // Verify the UI includes theme-related markup or scripts
    // The exact check depends on the Lit UI's theme implementation
    assert!(
        body.contains("theme") || body.contains("dark-mode") || body.contains("color-scheme"),
        "index.html should contain theme support markup"
    );
}

#[tokio::test]
async fn test_health_endpoint() {
    let app = build_app(AuthConfig::default()).await;
    let server = TestServer::new(app).unwrap();

    let response = server.get("/health").await;
    assert_eq!(response.status_code(), StatusCode::OK);

    let body = response.json::<serde_json::Value>();
    assert_eq!(
        body.get("status"),
        Some(&serde_json::Value::String("ok".to_string()))
    );
}

#[tokio::test]
async fn test_static_file_cache_headers() {
    let app = build_app(AuthConfig::default()).await;
    let server = TestServer::new(app).unwrap();

    // Test hashed asset gets immutable cache header
    let response = server.get("/assets/index-BQE4UcBt.css").await;
    assert_eq!(response.status_code(), StatusCode::OK);

    let cache_control = response
        .headers()
        .get("cache-control")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(
        cache_control.contains("immutable"),
        "Hashed asset should have immutable cache: {}",
        cache_control
    );
    assert!(
        cache_control.contains("max-age=31536000"),
        "Hashed asset should have max-age=31536000: {}",
        cache_control
    );
}

#[tokio::test]
async fn test_index_html_cache_headers() {
    let app = build_app(AuthConfig::default()).await;
    let server = TestServer::new(app).unwrap();

    // Test index.html gets no-cache header
    let response = server.get("/").await;
    assert_eq!(response.status_code(), StatusCode::OK);

    let cache_control = response
        .headers()
        .get("cache-control")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(
        cache_control.contains("no-cache"),
        "index.html should have no-cache: {}",
        cache_control
    );
}

#[tokio::test]
async fn test_api_routes_return_not_found() {
    let app = build_app(AuthConfig::default()).await;
    let server = TestServer::new(app).unwrap();

    // API routes should not be handled by static file router
    let response = server.get("/api/test").await;
    // Static router returns NOT_FOUND for API routes
    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_png_asset_served() {
    let app = build_app(AuthConfig::default()).await;
    let server = TestServer::new(app).unwrap();

    let response = server.get("/icon-192.png").await;
    assert_eq!(response.status_code(), StatusCode::OK);

    let content_type = response
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(content_type.contains("image/png"));
}

#[tokio::test]
async fn test_deep_spa_fallback_with_query_params() {
    let app = build_app(AuthConfig::default()).await;
    let server = TestServer::new(app).unwrap();

    // Unknown route with query parameters should still serve index.html
    let response = server.get("/agents/123?filter=test").await;
    assert_eq!(response.status_code(), StatusCode::OK);

    let body = response.text();
    assert!(body.contains("Aisopod"));
}
