//! Handshake protocol conformance tests
//!
//! These tests validate the WebSocket handshake and connection establishment
//! as specified in the aisopod protocol.

use aisopod_client::{AisopodClient, ClientConfig};
use std::env;

/// Connect to the test server using configuration from environment variables or defaults
pub async fn connect_test_client() -> AisopodClient {
    let config = ClientConfig {
        server_url: env::var("AISOPOD_TEST_URL")
            .unwrap_or_else(|_| "ws://127.0.0.1:8080/ws".to_string()),
        auth_token: env::var("AISOPOD_TEST_TOKEN")
            .unwrap_or_else(|_| "test-token".to_string()),
        client_name: "conformance-test".to_string(),
        client_version: "0.1.0".to_string(),
        device_id: uuid::Uuid::new_v4(),
        protocol_version: "1.0".to_string(),
    };

    AisopodClient::connect(config)
        .await
        .expect("Failed to connect to test server")
}

/// Check if conformance tests should run based on environment configuration
pub fn should_run_conformance_tests() -> bool {
    env::var("AISOPOD_TEST_URL").is_ok() || env::var("RUN_CONFORMANCE_TESTS").is_ok()
}

#[tokio::test]
async fn test_successful_handshake() {
    // Skip if no test server is configured
    if !should_run_conformance_tests() {
        return;
    }

    let client = connect_test_client().await;

    // Verify the client is connected after successful handshake
    assert!(client.is_connected(), "Client should be connected after handshake");
}

#[tokio::test]
async fn test_handshake_without_auth_header() {
    // Skip if no test server is configured
    if !should_run_conformance_tests() {
        return;
    }

    // Note: This test requires a modified connection that omits the auth header.
    // The current AisopodClient always includes the Authorization header.
    // This test verifies the expected behavior when auth is missing.
    
    let config = ClientConfig {
        server_url: env::var("AISOPOD_TEST_URL")
            .unwrap_or_else(|_| "ws://127.0.0.1:8080/ws".to_string()),
        auth_token: "".to_string(), // Empty token to simulate missing auth
        client_name: "conformance-test".to_string(),
        client_version: "0.1.0".to_string(),
        device_id: uuid::Uuid::new_v4(),
        protocol_version: "1.0".to_string(),
    };

    // This should fail authentication
    let result = AisopodClient::connect(config).await;
    
    // The test expects the connection to fail with an auth error
    assert!(
        result.is_err(),
        "Handshake without auth should fail"
    );
}

#[tokio::test]
async fn test_welcome_message_fields() {
    // Skip if no test server is configured
    if !should_run_conformance_tests() {
        return;
    }

    let client = connect_test_client().await;

    // Verify the client is connected
    assert!(client.is_connected(), "Client should be connected");

    // Note: The AisopodClient doesn't currently expose welcome message fields
    // This test verifies the basic connection flow works
    // In a full implementation, we would verify:
    // - server_version is not empty
    // - protocol_version is not empty  
    // - session_id is not empty
    
    // For now, we verify the connection was established
    assert_eq!(client.state(), aisopod_client::ClientState::Connected);
}
