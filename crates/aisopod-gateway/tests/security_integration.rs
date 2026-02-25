//! Security integration tests for the aisopod-gateway
//!
//! These tests verify the authentication and authorization behavior of the gateway:
//! - Unauthenticated request rejection
//! - Authenticated request acceptance
//! - Scope-based authorization enforcement
//!
//! Note: These tests use axum-test for integration testing.

#![deny(unused_must_use)]

use aisopod_config::types::{AuthConfig, AuthMode, TokenCredential, PasswordCredential};
use aisopod_gateway::{server::build_app, middleware::AuthConfigData};
use axum_test::{TestServer, TestRequest};
use serde_json::json;

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

    let app = build_app(config, None).await;
    TestServer::new(app).expect("Failed to create test server")
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

    let app = build_app(config, None).await;
    TestServer::new(app).expect("Failed to create test server")
}

/// Test server builder with no authentication
async fn build_test_server_no_auth() -> TestServer {
    let config = AuthConfig {
        gateway_mode: AuthMode::None,
        ..Default::default()
    };

    let app = build_app(config, None).await;
    TestServer::new(app).expect("Failed to create test server")
}

#[tokio::test]
async fn test_unauthenticated_request_rejected() {
    let server = build_test_server_with_token("test-token", "operator", vec!["chat:write"]).await;

    let response = server
        .post("/rpc")
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "agent.list",
            "id": 1
        }))
        .await;

    assert_eq!(response.status(), 401);
    
    let body = response.json::<serde_json::Value>().await;
    assert!(body["error"].is_object());
    assert_eq!(body["error"]["code"], -32603);
}

#[tokio::test]
async fn test_authenticated_request_accepted() {
    let server = build_test_server_with_token("test-token", "operator", vec!["chat:write"]).await;

    let response = server
        .post("/rpc")
        .add_header("Authorization", "Bearer test-token")
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "system.ping",
            "id": 1
        }))
        .await;

    assert_eq!(response.status(), 200);
    
    let body = response.json::<serde_json::Value>().await;
    assert!(body["result"].is_object());
    assert_eq!(body["result"]["status"], "ok");
}

#[tokio::test]
async fn test_authenticated_password_request_accepted() {
    let server = build_test_server_with_password("admin", "password123", "operator", vec!["chat:write"]).await;

    let creds = base64::encode("admin:password123");
    let response = server
        .post("/rpc")
        .add_header("Authorization", format!("Basic {}", creds))
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "system.ping",
            "id": 1
        }))
        .await;

    assert_eq!(response.status(), 200);
    
    let body = response.json::<serde_json::Value>().await;
    assert!(body["result"].is_object());
}

#[tokio::test]
async fn test_insufficient_scope_rejected() {
    let server = build_test_server_with_token("test-token", "operator", vec!["operator.read"]).await;

    let response = server
        .post("/rpc")
        .add_header("Authorization", "Bearer test-token")
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "admin.shutdown",
            "id": 1
        }))
        .await;

    assert_eq!(response.status(), 200);
    
    let body = response.json::<serde_json::Value>().await;
    assert!(body["error"].is_object());
    assert!(body["error"]["message"]
        .as_str()
        .unwrap()
        .contains("Insufficient permissions"));
}

#[tokio::test]
async fn test_admin_scope_allows_all() {
    let server = build_test_server_with_token("admin-token", "admin", vec!["operator.admin"]).await;

    // Try to access admin.shutdown which requires admin scope
    let response = server
        .post("/rpc")
        .add_header("Authorization", "Bearer admin-token")
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "admin.shutdown",
            "id": 1
        }))
        .await;

    assert_eq!(response.status(), 200);
    
    let body = response.json::<serde_json::Value>().await;
    // Admin should be able to access the method (though it may not be implemented)
    // The important thing is that scope check passed
    assert!(body["result"].is_object() || body["error"].is_null());
}

#[tokio::test]
async fn test_no_auth_mode_allows_all() {
    let server = build_test_server_no_auth().await;

    let response = server
        .post("/rpc")
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "agent.list",
            "id": 1
        }))
        .await;

    assert_eq!(response.status(), 200);
    
    let body = response.json::<serde_json::Value>().await;
    assert!(body["result"].is_object());
}

#[tokio::test]
async fn test_missing_token_rejected() {
    let server = build_test_server_with_token("test-token", "operator", vec![]).await;

    // Request with wrong token format
    let response = server
        .post("/rpc")
        .add_header("Authorization", "Bearer wrong-token")
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "system.ping",
            "id": 1
        }))
        .await;

    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn test_missing_password_rejected() {
    let server = build_test_server_with_password("admin", "password123", "operator", vec![]).await;

    let response = server
        .post("/rpc")
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "system.ping",
            "id": 1
        }))
        .await;

    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn test_wrong_password_rejected() {
    let server = build_test_server_with_password("admin", "password123", "operator", vec![]).await;

    let creds = base64::encode("admin:wrongpassword");
    let response = server
        .post("/rpc")
        .add_header("Authorization", format!("Basic {}", creds))
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "system.ping",
            "id": 1
        }))
        .await;

    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn test_health_endpoint_always_allowed() {
    let server = build_test_server_with_token("test-token", "operator", vec![]).await;

    // Health endpoint should be accessible without authentication
    let response = server.get("/health").await;

    assert_eq!(response.status(), 200);
    
    let body = response.json::<serde_json::Value>().await;
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn test_read_method_requires_read_scope() {
    let server = build_test_server_with_token("test-token", "operator", vec!["operator.read"]).await;

    let response = server
        .post("/rpc")
        .add_header("Authorization", "Bearer test-token")
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "agent.list",
            "id": 1
        }))
        .await;

    assert_eq!(response.status(), 200);
    
    let body = response.json::<serde_json::Value>().await;
    assert!(body["result"].is_object());
}

#[tokio::test]
async fn test_write_method_requires_write_scope() {
    let server = build_test_server_with_token("test-token", "operator", vec!["operator.write"]).await;

    let response = server
        .post("/rpc")
        .add_header("Authorization", "Bearer test-token")
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "agent.start",
            "id": 1
        }))
        .await;

    assert_eq!(response.status(), 200);
    
    let body = response.json::<serde_json::Value>().await;
    // Should succeed because write scope allows write methods
    assert!(body["result"].is_object());
}

#[tokio::test]
async fn test_insufficient_scope_for_write_method() {
    let server = build_test_server_with_token("test-token", "operator", vec!["operator.read"]).await;

    let response = server
        .post("/rpc")
        .add_header("Authorization", "Bearer test-token")
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "agent.start",
            "id": 1
        }))
        .await;

    assert_eq!(response.status(), 200);
    
    let body = response.json::<serde_json::Value>().await;
    assert!(body["error"].is_object());
    assert!(body["error"]["message"]
        .as_str()
        .unwrap()
        .contains("Insufficient permissions"));
}
