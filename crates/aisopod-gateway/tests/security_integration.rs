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
use tokio::net::TcpStream;

// ============================================================================
// Test Helper Functions
// ============================================================================

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

/// Test server guard that holds the shutdown channel to keep the server alive
struct TestServer {
    addr: SocketAddr,
    _shutdown_tx: oneshot::Sender<()>,
}

impl TestServer {
    fn addr(&self) -> SocketAddr {
        self.addr
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        // Send shutdown signal when guard is dropped
        // The sender is dropped which will cause the channel to close
        // This will make the server's graceful shutdown detect the closed channel
    }
}

/// Start the gateway server for integration tests with custom auth config
/// Returns a TestServer guard that holds the shutdown channel
async fn start_test_server_with_auth(config: AuthConfig) -> TestServer {
    let app = build_app(config).await;

    // Bind to port 0 to get an available port from the OS
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind to port");
    let addr = listener.local_addr().expect("Failed to get local address");

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    // Clone the shutdown channel for the server task
    let shutdown_rx_server = shutdown_rx;

    // Run the server in a background task
    let _server_task = tokio::spawn(async move {
        let result = axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                shutdown_rx_server.await.ok();
            })
            .await;
        
        // Log any server errors
        if let Err(e) = result {
            eprintln!("Server task error: {:?}", e);
        }
    });

    // Wait for server to be ready by attempting to connect with retries
    let max_retries = 50;
    let retry_delay = Duration::from_millis(100);
    
    for attempt in 0..max_retries {
        match TcpStream::connect(&addr).await {
            Ok(_) => {
                // Successfully connected - server is ready
                // Close our side of the connection
                break;
            }
            Err(_) => {
                if attempt == max_retries - 1 {
                    panic!("Server failed to become ready after {} attempts", max_retries);
                }
                tokio::time::sleep(retry_delay).await;
            }
        }
    }

    // Return the server guard which holds the shutdown channel
    TestServer { addr, _shutdown_tx: shutdown_tx }
}

/// Test server builder with token authentication
async fn build_test_server_with_token(
    token: &str,
    role: &str,
    scopes: Vec<&str>,
) -> TestServer {
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
) -> TestServer {
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
async fn build_test_server_no_auth() -> TestServer {
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
    let server = build_test_server_with_token("test-token", "operator", vec!["operator.read"]).await;
    let addr = server.addr();

    let url = format!("http://{}/rpc", addr);

    // Request without token (unauthenticated)
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
    let server = build_test_server_with_token("test-token", "operator", vec!["operator.read"]).await;
    let addr = server.addr();

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
    let server = build_test_server_with_password("admin", "password123", "operator", vec!["operator.read"]).await;
    let addr = server.addr();

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
    let server = build_test_server_with_token("test-token", "operator", vec!["operator.read"]).await;
    let addr = server.addr();

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
    let server = build_test_server_with_token("admin-token", "admin", vec!["operator.admin"]).await;
    let addr = server.addr();

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
    let server = build_test_server_no_auth().await;
    let addr = server.addr();

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
    let server = build_test_server_with_token("test-token", "operator", vec![]).await;
    let addr = server.addr();

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
    let server = build_test_server_with_password("admin", "password123", "operator", vec![]).await;
    let addr = server.addr();

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
    let server = build_test_server_with_password("admin", "password123", "operator", vec![]).await;
    let addr = server.addr();

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
    let server = build_test_server_with_token("test-token", "operator", vec![]).await;
    let addr = server.addr();

    let url = format!("http://{}/health", addr);

    // Health endpoint should be accessible without authentication
    let response = reqwest::get(&url).await.expect("Health request failed");

    assert_eq!(response.status(), 200);

    let body = response.json::<serde_json::Value>().await.expect("Failed to parse body");
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn test_read_method_requires_read_scope() {
    let server = build_test_server_with_token("test-token", "operator", vec!["operator.read"]).await;
    let addr = server.addr();

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
    let server = build_test_server_with_token("test-token", "operator", vec!["operator.write"]).await;
    let addr = server.addr();

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
    let server = build_test_server_with_token("test-token", "operator", vec!["operator.read"]).await;
    let addr = server.addr();

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
