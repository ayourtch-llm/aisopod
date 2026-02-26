//! Conformance test suite for aisopod protocol
//!
//! This module provides a comprehensive test suite that validates an aisopod
//! server's compliance with the WebSocket protocol specification.
//!
//! These tests require a running aisopod server instance and can be gated
//! behind the AISOPOD_TEST_URL environment variable.

pub mod canvas;
pub mod device_pairing;
pub mod error_handling;
pub mod handshake;
pub mod rpc_methods;
pub mod version_negotiation;

pub use aisopod_client::{AisopodClient, ClientConfig};
pub use aisopod_client::DeviceInfo;
pub use aisopod_client::DeviceCapability;

/// Connect to the test server using configuration from environment variables or defaults
///
/// This is the primary test harness function used by all conformance tests.
/// It connects to the server and performs the handshake, returning a connected client.
///
/// Environment variables:
/// - AISOPOD_TEST_URL: WebSocket URL of the server (default: ws://127.0.0.1:8080/ws)
/// - AISOPOD_TEST_TOKEN: Authentication token for the connection (default: test-token)
pub async fn connect_test_client() -> AisopodClient {
    let config = ClientConfig {
        server_url: std::env::var("AISOPOD_TEST_URL")
            .unwrap_or_else(|_| "ws://127.0.0.1:8080/ws".to_string()),
        auth_token: std::env::var("AISOPOD_TEST_TOKEN")
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

/// Create a test device info for pairing tests
pub fn test_device_info() -> DeviceInfo {
    DeviceInfo {
        device_id: uuid::Uuid::new_v4(),
        device_name: "conformance-test-device".to_string(),
        device_type: "test-client".to_string(),
        device_version: "0.1.0".to_string(),
        capabilities: vec![DeviceCapability {
            service: "test".to_string(),
            methods: vec!["test_method".to_string()],
            description: Some("Test capability for conformance tests".to_string()),
        }],
    }
}

/// Check if conformance tests should run based on environment configuration
pub fn should_run_conformance_tests() -> bool {
    std::env::var("AISOPOD_TEST_URL").is_ok() || std::env::var("RUN_CONFORMANCE_TESTS").is_ok()
}
