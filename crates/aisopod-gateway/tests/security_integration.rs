//! Security integration tests for the aisopod-gateway
//!
//! These tests verify the authentication and authorization behavior of the gateway:
//! - Unauthenticated request rejection
//! - Authenticated request acceptance
//! - Scope-based authorization enforcement
//!
//! Note: These tests use reqwest for HTTP requests to the gateway.

use aisopod_config::types::{AuthConfig, AuthMode, TokenCredential, PasswordCredential};
use aisopod_gateway::server::build_app;
use axum::Router;
use serde_json::json;
use std::net::{SocketAddr, TcpListener};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::oneshot;

// ============================================================================
// Test Helper Functions
// ============================================================================

/// Global counter for unique port allocation - each test gets its own port
static PORT_COUNTER: std::sync::atomic::AtomicU16 = std::sync::atomic::AtomicU16::new(40000);

/// Find an available port for testing
fn find_available_port() -> u16 {
    PORT_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 40000
}

/// Wait for port to be released (helps with port reuse in parallel tests)
fn wait_for_port_release(port: u16) {
    let max_attempts = 50;
    let delay = Duration::from_millis(50);

    for _ in 0..max_attempts {
        if TcpListener::bind(("127.0.0.1", port)).is_ok() {
            return;
        }
        std::thread::sleep(delay);
    }
}

/// Start the gateway server for integration tests with custom auth config
async fn start_test_server_with_auth(config: AuthConfig) -> SocketAddr {
    let app = build_app(config, None).await;

    let addr: SocketAddr = format!("127.0.0.1:{}", find_available_port()).parse().unwrap();

    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    let listener = tokio::net::TcpListener::bind(&addr).await.expect("Failed to bind to port");

    // Run the server in a background task
    let server_task = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                shutdown_rx.await.ok();
            })
            .await
            .expect("Server error");
    });

    // Wait for server to be ready
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Don't wait for shutdown, just return the address
    let _shutdown_rx = shutdown_rx;

    addr
}

/// Test server builder with token authentication
async fn build_test_server_with_token(
    token: &str,
    role: &str,
    scopes: Vec<&str>,
) -> SocketAddr {
    let config = AuthConfig {
        gateway_mode: AuthMode::Token,
        tokens: vec![TokenCredential {
            token: token.to_string(),
            role: role.to_string(),
            scopes: scopes.iter().map(|s| s.to_string()).collect(),
        }],
        ..Default::default()
    };

    start_test_server_with_auth(config).await
}

/// Test server builder with password authentication
async fn build_test_server_with_password(
    username: &str,
    password: &str,
    role: &str,
    scopes: Vec<&str>,
) -> SocketAddr {
    let config = AuthConfig {
        gateway_mode: AuthMode::Password,
        passwords: vec![PasswordCredential {
            username: username.to_string(),
            password: aisopod_config::sensitive::Sensitive::new(password.to_string()),
            role: role.to_string(),
            scopes: scopes.iter().map(|s| s.to_string()).collect(),
        }],
        ..Default::default()
    };

    start_test_server_with_auth(config).await
}

/// Test server builder with no authentication
async fn build_test_server_no_auth() -> SocketAddr {
    let config = AuthConfig {
        gateway_mode: AuthMode::None,
        ..Default::default()
    };

    start_test_server_with_auth(config).await
}

// ============================================================================
// Authentication Tests
// ============================================================================

#[tokio::test]
async fn test_unauthenticated_request_rejected() {
    let addr = build_test_server_with_token("test-token", "operator", vec!["chat:write"]).await;

    let url = format!("http://{}/rpc", addr);

    let response = reqwest::Client::new()
        .post(&url)
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "agent.list",
            "id": 1
        }))
        .send()
        .await
        .expect("Request failed");

    assert_eq!(response.status(), 401);

    let body = response.json::<serde_json::Value>().await.expect("Failed to parse body");
    assert!(body["error"].is_object());
    assert_eq!(body["error"]["code"], -32603);
}

#[tokio::test]
async fn test_authenticated_request_accepted() {
    let addr = build_test_server_with_token("test-token", "operator", vec!["chat:write"]).await;

    let url = format!("http://{}/rpc", addr);

    let response = reqwest::Client::new()
        .post(&url)
        .bearer_auth("test-token")
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "system.ping",
            "id": 1
        }))
        .send()
        .await
        .expect("Request failed");

    assert_eq!(response.status(), 200);

    let body = response.json::<serde_json::Value>().await.expect("Failed to parse body");
    assert!(body["result"].is_object());
    assert_eq!(body["result"]["status"], "ok");
}

#[tokio::test]
async fn test_authenticated_password_request_accepted() {
    let addr = build_test_server_with_password("admin", "password123", "operator", vec!["chat:write"]).await;

    let url = format!("http://{}/rpc", addr);

    let creds = base64::encode("admin:password123");
    let response = reqwest::Client::new()
        .post(&url)
        .header("Authorization", format!("Basic {}", creds))
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "system.ping",
            "id": 1
        }))
        .send()
        .await
        .expect("Request failed");

    assert_eq!(response.status(), 200);

    let body = response.json::<serde_json::Value>().await.expect("Failed to parse body");
    assert!(body["result"].is_object());
}

#[tokio::test]
async fn test_insufficient_scope_rejected() {
    let addr = build_test_server_with_token("test-token", "operator", vec!["operator.read"]).await;

    let url = format!("http://{}/rpc", addr);

    let response = reqwest::Client::new()
        .post(&url)
        .bearer_auth("test-token")
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "admin.shutdown",
            "id": 1
        }))
        .send()
        .await
        .expect("Request failed");

    assert_eq!(response.status(), 200);

    let body = response.json::<serde_json::Value>().await.expect("Failed to parse body");
    assert!(body["error"].is_object());
    assert!(body["error"]["message"]
        .as_str()
        .unwrap()
        .contains("Insufficient permissions"));
}

#[tokio::test]
async fn test_admin_scope_allows_all() {
    let addr = build_test_server_with_token("admin-token", "admin", vec!["operator.admin"]).await;

    let url = format!("http://{}/rpc", addr);

    // Try to access admin.shutdown which requires admin scope
    let response = reqwest::Client::new()
        .post(&url)
        .bearer_auth("admin-token")
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "admin.shutdown",
            "id": 1
        }))
        .send()
        .await
        .expect("Request failed");

    assert_eq!(response.status(), 200);

    let body = response.json::<serde_json::Value>().await.expect("Failed to parse body");
    // Admin should be able to access the method (though it may not be implemented)
    // The important thing is that scope check passed
    assert!(body["result"].is_object() || body["error"].is_null());
}

#[tokio::test]
async fn test_no_auth_mode_allows_all() {
    let addr = build_test_server_no_auth().await;

    let url = format!("http://{}/rpc", addr);

    let response = reqwest::Client::new()
        .post(&url)
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "agent.list",
            "id": 1
        }))
        .send()
        .await
        .expect("Request failed");

    assert_eq!(response.status(), 200);

    let body = response.json::<serde_json::Value>().await.expect("Failed to parse body");
    assert!(body["result"].is_object());
}

#[tokio::test]
async fn test_missing_token_rejected() {
    let addr = build_test_server_with_token("test-token", "operator", vec![]).await;

    let url = format!("http://{}/rpc", addr);

    // Request with wrong token format
    let response = reqwest::Client::new()
        .post(&url)
        .bearer_auth("wrong-token")
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "system.ping",
            "id": 1
        }))
        .send()
        .await
        .expect("Request failed");

    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn test_missing_password_rejected() {
    let addr = build_test_server_with_password("admin", "password123", "operator", vec![]).await;

    let url = format!("http://{}/rpc", addr);

    let response = reqwest::Client::new()
        .post(&url)
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "system.ping",
            "id": 1
        }))
        .send()
        .await
        .expect("Request failed");

    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn test_wrong_password_rejected() {
    let addr = build_test_server_with_password("admin", "password123", "operator", vec![]).await;

    let url = format!("http://{}/rpc", addr);

    let creds = base64::encode("admin:wrongpassword");
    let response = reqwest::Client::new()
        .post(&url)
        .header("Authorization", format!("Basic {}", creds))
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "system.ping",
            "id": 1
        }))
        .send()
        .await
        .expect("Request failed");

    assert_eq!(response.status(), 401);
}

// ============================================================================
// Scope-based Authorization Tests
// ============================================================================

#[tokio::test]
async fn test_health_endpoint_always_allowed() {
    let addr = build_test_server_with_token("test-token", "operator", vec![]).await;

    let url = format!("http://{}/health", addr);

    // Health endpoint should be accessible without authentication
    let response = reqwest::get(&url).await.expect("Health request failed");

    assert_eq!(response.status(), 200);

    let body = response.json::<serde_json::Value>().await.expect("Failed to parse body");
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn test_read_method_requires_read_scope() {
    let addr = build_test_server_with_token("test-token", "operator", vec!["operator.read"]).await;

    let url = format!("http://{}/rpc", addr);

    let response = reqwest::Client::new()
        .post(&url)
        .bearer_auth("test-token")
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "agent.list",
            "id": 1
        }))
        .send()
        .await
        .expect("Request failed");

    assert_eq!(response.status(), 200);

    let body = response.json::<serde_json::Value>().await.expect("Failed to parse body");
    assert!(body["result"].is_object());
}

#[tokio::test]
async fn test_write_method_requires_write_scope() {
    let addr = build_test_server_with_token("test-token", "operator", vec!["operator.write"]).await;

    let url = format!("http://{}/rpc", addr);

    let response = reqwest::Client::new()
        .post(&url)
        .bearer_auth("test-token")
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "agent.start",
            "id": 1
        }))
        .send()
        .await
        .expect("Request failed");

    assert_eq!(response.status(), 200);

    let body = response.json::<serde_json::Value>().await.expect("Failed to parse body");
    // Should succeed because write scope allows write methods
    assert!(body["result"].is_object());
}

#[tokio::test]
async fn test_insufficient_scope_for_write_method() {
    let addr = build_test_server_with_token("test-token", "operator", vec!["operator.read"]).await;

    let url = format!("http://{}/rpc", addr);

    let response = reqwest::Client::new()
        .post(&url)
        .bearer_auth("test-token")
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "agent.start",
            "id": 1
        }))
        .send()
        .await
        .expect("Request failed");

    assert_eq!(response.status(), 200);

    let body = response.json::<serde_json::Value>().await.expect("Failed to parse body");
    assert!(body["error"].is_object());
    assert!(body["error"]["message"]
        .as_str()
        .unwrap()
        .contains("Insufficient permissions"));
}
