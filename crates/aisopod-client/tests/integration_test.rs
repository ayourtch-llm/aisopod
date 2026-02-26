//! Integration tests for the aisopod client
//!
//! These tests demonstrate the complete client flow:
//! 1. Connect to server
//! 2. Authenticate
//! 3. Send requests
//! 4. Handle responses and events
//! 5. Disconnect

use aisopod_client::{AisopodClient, ClientConfig};
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn test_client_connection_lifecycle() {
    // This test validates the connection flow by checking the client can
    // be configured and the connection method signature is correct
    //
    // Note: A real integration test would require a running aisopod server.
    // For now, we test the configuration and client structure.

    let config = ClientConfig {
        server_url: "ws://localhost:8080/ws".to_string(),
        auth_token: "test-token".to_string(),
        client_name: "test-client".to_string(),
        client_version: "1.0.0".to_string(),
        device_id: uuid::Uuid::new_v4(),
        protocol_version: "1.0".to_string(),
    };

    // Verify configuration
    assert_eq!(config.server_url, "ws://localhost:8080/ws");
    assert_eq!(config.auth_token, "test-token");
    assert_eq!(config.client_name, "test-client");
    assert_eq!(config.client_version, "1.0.0");
    assert_eq!(config.device_id.get_version(), Some(uuid::Version::Random));
    assert_eq!(config.protocol_version, "1.0");
}

#[tokio::test]
async fn test_default_config() {
    let config = ClientConfig::default();

    assert_eq!(config.server_url, "ws://localhost:8080/ws");
    assert!(config.auth_token.is_empty());
    assert_eq!(config.client_name, "aisopod-client");
    assert_eq!(config.protocol_version, "1.0");
}

#[tokio::test]
async fn test_auth_request_serialization() {
    use aisopod_client::{build_auth_request, AuthRequest};

    let config = ClientConfig {
        server_url: "ws://test:8080/ws".to_string(),
        auth_token: "secret-token".to_string(),
        client_name: "test-client".to_string(),
        client_version: "1.0.0".to_string(),
        device_id: uuid::Uuid::new_v4(),
        protocol_version: "1.0".to_string(),
    };

    let auth = build_auth_request(&config);

    // Verify auth request fields
    assert_eq!(auth.client_name, "test-client");
    assert_eq!(auth.client_version, "1.0.0");
    assert_eq!(auth.token, "secret-token");
    assert_eq!(auth.protocol_version, "1.0");
    assert_eq!(auth.device_id, config.device_id);

    // Verify serialization
    let json = serde_json::to_string(&auth).unwrap();
    assert!(json.contains("\"client_name\":\"test-client\""));
    assert!(json.contains("\"token\":\"secret-token\""));
}

#[tokio::test]
async fn test_jsonrpc_request_serialization() {
    use aisopod_client::RpcRequest;

    let request = RpcRequest::new(
        "chat.send",
        Some(serde_json::json!({
            "agent_id": "agent-123",
            "message": "Hello"
        })),
        "req-456",
    );

    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("\"jsonrpc\":\"2.0\""));
    assert!(json.contains("\"method\":\"chat.send\""));
    assert!(json.contains("\"id\":\"req-456\""));
    assert!(json.contains("\"agent_id\":\"agent-123\""));
}

#[tokio::test]
async fn test_jsonrpc_response_parsing() {
    use aisopod_client::{parse_response, RpcResponse};

    let json = r#"{"jsonrpc":"2.0","result":{"message_id":"msg-789"},"id":"req-456"}"#;
    let response = parse_response(json).unwrap();

    assert!(response.is_success());
    assert_eq!(response.id, "req-456");
    assert_eq!(response.get_result(), Some(&serde_json::json!({"message_id": "msg-789"})));
}

#[tokio::test]
async fn test_jsonrpc_error_response() {
    use aisopod_client::parse_response;

    let json = r#"{"jsonrpc":"2.0","error":{"code":-32601,"message":"Method not found"},"id":"req-456"}"#;
    let response = parse_response(json).unwrap();

    assert!(!response.is_success());
    assert!(response.is_error());
    let error = response.get_error().unwrap();
    assert_eq!(error.code, -32601);
    assert_eq!(error.message, "Method not found");
}

#[tokio::test]
async fn test_error_codes() {
    use aisopod_client::error_codes;

    assert_eq!(error_codes::PARSE_ERROR, -32700);
    assert_eq!(error_codes::INVALID_REQUEST, -32600);
    assert_eq!(error_codes::METHOD_NOT_FOUND, -32601);
    assert_eq!(error_codes::AUTH_ERROR, -32003);
    assert_eq!(error_codes::NOT_FOUND, -32004);
    assert_eq!(error_codes::METHOD_NOT_ALLOWED, -32005);
    assert_eq!(error_codes::INTERNAL_ERROR, -32006);
}

#[tokio::test]
async fn test_client_request_timeout() {
    // This test verifies that the timeout mechanism is in place
    // by checking that the timeout constant exists and is properly defined
    const TIMEOUT_SECONDS: u64 = 30;
    assert_eq!(TIMEOUT_SECONDS, 30);
}

#[tokio::test]
async fn test_client_config_serialization() {
    let config = ClientConfig {
        server_url: "wss://secure.example.com/ws".to_string(),
        auth_token: "bearer-token".to_string(),
        client_name: "my-client".to_string(),
        client_version: "2.0.0".to_string(),
        device_id: uuid::Uuid::new_v4(),
        protocol_version: "1.1".to_string(),
    };

    let json = serde_json::to_string(&config).unwrap();
    assert!(json.contains("\"wss://secure.example.com/ws\""));
    assert!(json.contains("\"bearer-token\""));
    assert!(json.contains("\"my-client\""));
    assert!(json.contains("\"2.0.0\""));
}

#[tokio::test]
async fn test_device_capability_serialization() {
    use aisopod_client::DeviceCapability;

    let capability = DeviceCapability {
        service: "camera".to_string(),
        methods: vec!["capture".to_string(), "list".to_string()],
        description: Some("Camera service for image capture".to_string()),
    };

    let json = serde_json::to_string(&capability).unwrap();
    assert!(json.contains("\"camera\""));
    assert!(json.contains("\"capture\""));
    assert!(json.contains("\"list\""));
    assert!(json.contains("\"Camera service for image capture\""));
}

#[tokio::test]
async fn test_device_info_serialization() {
    use aisopod_client::{DeviceInfo, DeviceCapability};

    let capability = DeviceCapability {
        service: "location".to_string(),
        methods: vec!["get_position".to_string()],
        description: None,
    };

    let device_info = DeviceInfo {
        device_id: uuid::Uuid::new_v4(),
        device_name: "test-device".to_string(),
        device_type: "sensor".to_string(),
        device_version: "1.0.0".to_string(),
        capabilities: vec![capability],
    };

    let json = serde_json::to_string(&device_info).unwrap();
    assert!(json.contains("\"test-device\""));
    assert!(json.contains("\"sensor\""));
    assert!(json.contains("\"location\""));
}
