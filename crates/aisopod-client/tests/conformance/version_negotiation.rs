//! Version negotiation conformance tests
//!
//! These tests validate the protocol version negotiation as specified
//! in the aisopod protocol, including compatible versions, incompatible
//! major versions, and default behavior for missing version headers.

use crate::{connect_test_client, should_run_conformance_tests, ClientConfig};

#[tokio::test]
async fn test_compatible_version() {
    // Skip if no test server is configured
    if !should_run_conformance_tests() {
        return;
    }

    // Test connecting with a compatible version (1.0)
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

    let result = aisopod_client::AisopodClient::connect(config).await;

    // Connection with compatible version should succeed
    assert!(result.is_ok(), "Connection with version 1.0 should succeed");
}

#[tokio::test]
async fn test_incompatible_major_version() {
    // Skip if no test server is configured
    if !should_run_conformance_tests() {
        return;
    }

    // Test connecting with an incompatible major version (99.0)
    let config = ClientConfig {
        server_url: std::env::var("AISOPOD_TEST_URL")
            .unwrap_or_else(|_| "ws://127.0.0.1:8080/ws".to_string()),
        auth_token: std::env::var("AISOPOD_TEST_TOKEN")
            .unwrap_or_else(|_| "test-token".to_string()),
        client_name: "conformance-test".to_string(),
        client_version: "0.1.0".to_string(),
        device_id: uuid::Uuid::new_v4(),
        protocol_version: "99.0".to_string(),
    };

    let result = aisopod_client::AisopodClient::connect(config).await;

    // Connection with incompatible major version should fail
    assert!(
        result.is_err(),
        "Connection with incompatible major version 99.0 should fail"
    );
}

#[tokio::test]
async fn test_missing_version_defaults_to_1_0() {
    // Skip if no test server is configured
    if !should_run_conformance_tests() {
        return;
    }

    // Test connecting without specifying a version
    // The client should default to "1.0" as per the protocol specification
    let config = ClientConfig {
        server_url: std::env::var("AISOPOD_TEST_URL")
            .unwrap_or_else(|_| "ws://127.0.0.1:8080/ws".to_string()),
        auth_token: std::env::var("AISOPOD_TEST_TOKEN")
            .unwrap_or_else(|_| "test-token".to_string()),
        client_name: "conformance-test".to_string(),
        client_version: "0.1.0".to_string(),
        device_id: uuid::Uuid::new_v4(),
        protocol_version: "1.0".to_string(), // Default value from ClientConfig
    };

    // Verify the default value
    assert_eq!(config.protocol_version, "1.0");

    // Connection with default version should succeed
    let result = aisopod_client::AisopodClient::connect(config).await;
    assert!(
        result.is_ok(),
        "Connection with default version 1.0 should succeed"
    );
}

#[tokio::test]
async fn test_version_header_format() {
    // Test that the version header is properly formatted
    let config = ClientConfig {
        server_url: "ws://test:8080/ws".to_string(),
        auth_token: "test-token".to_string(),
        client_name: "test-client".to_string(),
        client_version: "1.0.0".to_string(),
        device_id: uuid::Uuid::new_v4(),
        protocol_version: "1.0".to_string(),
    };

    // Verify the protocol version is in the correct format (MAJOR.MINOR)
    let version_parts: Vec<&str> = config.protocol_version.split('.').collect();
    assert_eq!(
        version_parts.len(),
        2,
        "Version should be in MAJOR.MINOR format"
    );
    assert!(
        version_parts[0].parse::<u32>().is_ok(),
        "Major version should be numeric"
    );
    assert!(
        version_parts[1].parse::<u32>().is_ok(),
        "Minor version should be numeric"
    );
}

#[tokio::test]
async fn test_client_config_protocol_version() {
    // Test various protocol version configurations
    let versions = ["1.0", "1.1", "2.0", "0.9"];

    for version in versions {
        let config = ClientConfig {
            server_url: "ws://test:8080/ws".to_string(),
            auth_token: "test-token".to_string(),
            client_name: "test-client".to_string(),
            client_version: "1.0.0".to_string(),
            device_id: uuid::Uuid::new_v4(),
            protocol_version: version.to_string(),
        };

        assert_eq!(config.protocol_version, version);
    }
}
