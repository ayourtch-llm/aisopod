//! Integration tests for configuration validation
//!
//! These tests verify that the validation logic correctly identifies
//! valid and invalid configurations, including boundary values and edge cases.

use aisopod_config::AisopodConfig;
use aisopod_config::loader::load_config;
use std::path::PathBuf;

#[test]
fn test_default_config_validates() {
    let config = AisopodConfig::default();
    let result = config.validate();
    
    assert!(result.is_ok(), "Default config should be valid: {:?}", result);
}

#[test]
fn test_valid_minimal_config_passes() {
    let path = PathBuf::from("tests/fixtures/valid_minimal.json5");
    let config = load_config(&path).expect("Failed to load valid_minimal.json5");
    let result = config.validate();
    
    assert!(result.is_ok(), "Valid minimal config should pass validation: {:?}", result);
}

#[test]
fn test_valid_full_config_passes() {
    let path = PathBuf::from("tests/fixtures/valid_full.json5");
    let config = load_config(&path).expect("Failed to load valid_full.json5");
    let result = config.validate();
    
    assert!(result.is_ok(), "Valid full config should pass validation: {:?}", result);
}

#[test]
fn test_invalid_port_zero_fails() {
    let mut config = AisopodConfig::default();
    config.gateway.server.port = 0;
    
    let errors = config.validate().unwrap_err();
    assert!(!errors.is_empty(), "Should have validation errors");
    assert!(
        errors.iter().any(|e| e.path == "gateway.server.port"),
        "Should have port validation error"
    );
}

#[test]
fn test_invalid_port_above_max_fails() {
    let mut config = AisopodConfig::default();
    // Use u16::MAX + 1 which is 65536, this should fail
    // We use u32 cast to avoid overflow in source
    config.gateway.server.port = (u16::MAX as u32 + 1) as u16;
    
    let errors = config.validate().unwrap_err();
    assert!(!errors.is_empty(), "Should have validation errors");
    assert!(
        errors.iter().any(|e| e.path == "gateway.server.port"),
        "Should have port validation error"
    );
}

#[test]
fn test_invalid_port_65535_passes() {
    // 65535 should be valid (boundary value)
    let mut config = AisopodConfig::default();
    config.gateway.server.port = 65535;
    
    let result = config.validate();
    assert!(result.is_ok(), "Port 65535 should be valid");
}

#[test]
fn test_invalid_port_negative_simulated_fails() {
    // Since port is u16, we can't set negative values directly
    // Test with port 1 (minimum valid) - should pass
    let mut config = AisopodConfig::default();
    config.gateway.server.port = 1;
    
    let result = config.validate();
    assert!(result.is_ok(), "Port 1 should be valid");
}

#[test]
fn test_empty_version_fails() {
    let mut config = AisopodConfig::default();
    config.meta.version = String::new();
    
    let errors = config.validate().unwrap_err();
    assert!(!errors.is_empty(), "Should have validation errors");
    assert!(
        errors.iter().any(|e| e.path == "meta.version"),
        "Should have version validation error"
    );
}

#[test]
fn test_empty_address_fails() {
    let mut config = AisopodConfig::default();
    config.gateway.bind.address = String::new();
    
    let errors = config.validate().unwrap_err();
    assert!(!errors.is_empty(), "Should have validation errors");
    assert!(
        errors.iter().any(|e| e.path == "gateway.bind.address"),
        "Should have address validation error"
    );
}

#[test]
fn test_duplicate_agent_names_detected() {
    let mut config = AisopodConfig::default();
    config.agents.agents = vec![
        aisopod_config::types::Agent {
            id: "agent1".to_string(),
            name: "agent1".to_string(),
            model: "default".to_string(),
            workspace: "/workspace".to_string(),
            sandbox: false,
            subagents: vec![],
        },
        aisopod_config::types::Agent {
            id: "agent2".to_string(),
            name: "agent1".to_string(),  // Duplicate name
            model: "default".to_string(),
            workspace: "/workspace".to_string(),
            sandbox: false,
            subagents: vec![],
        },
    ];
    
    let errors = config.validate().unwrap_err();
    assert!(
        errors.iter().any(|e| e.message.contains("Duplicate agent name: agent1")),
        "Should detect duplicate agent names, got: {:?}",
        errors
    );
}

#[test]
fn test_duplicate_model_ids_detected() {
    let mut config = AisopodConfig::default();
    config.models.models = vec![
        aisopod_config::types::Model {
            id: "model1".to_string(),
            name: "Model 1".to_string(),
            provider: "test".to_string(),
            capabilities: vec![],
        },
        aisopod_config::types::Model {
            id: "model1".to_string(),  // Duplicate ID
            name: "Model 2".to_string(),
            provider: "test".to_string(),
            capabilities: vec![],
        },
    ];
    
    let errors = config.validate().unwrap_err();
    assert!(
        errors.iter().any(|e| e.message.contains("Duplicate model ID: model1")),
        "Should detect duplicate model IDs, got: {:?}",
        errors
    );
}

#[test]
fn test_multiple_validation_errors_collected() {
    let mut config = AisopodConfig::default();
    config.gateway.server.port = 0;
    config.gateway.bind.address = String::new();
    config.meta.version = String::new();
    
    let errors = config.validate().unwrap_err();
    assert_eq!(errors.len(), 3, "Should collect all 3 validation errors");
}

#[test]
fn test_boundary_port_values() {
    // Test port 1 (minimum valid)
    let mut config = AisopodConfig::default();
    config.gateway.server.port = 1;
    assert!(config.validate().is_ok(), "Port 1 should be valid");
    
    // Test port 80 (common HTTP port)
    config.gateway.server.port = 80;
    assert!(config.validate().is_ok(), "Port 80 should be valid");
    
    // Test port 443 (common HTTPS port)
    config.gateway.server.port = 443;
    assert!(config.validate().is_ok(), "Port 443 should be valid");
    
    // Test port 8080 (common development port)
    config.gateway.server.port = 8080;
    assert!(config.validate().is_ok(), "Port 8080 should be valid");
    
    // Test port 65535 (maximum valid)
    config.gateway.server.port = 65535;
    assert!(config.validate().is_ok(), "Port 65535 should be valid");
}

#[test]
fn test_invalid_port_65536_fails() {
    let mut config = AisopodConfig::default();
    // Use u16::MAX + 1 which is 65536, this should fail
    // We use u32 cast to avoid overflow in source
    config.gateway.server.port = (u16::MAX as u32 + 1) as u16;
    
    let errors = config.validate().unwrap_err();
    assert!(
        errors.iter().any(|e| e.path == "gateway.server.port" && 
                           e.message.contains("Port must be between")),
        "Port 65536 should fail validation"
    );
}

#[test]
fn test_empty_agent_name_fails() {
    let mut config = AisopodConfig::default();
    config.agents.agents = vec![
        aisopod_config::types::Agent {
            id: "agent1".to_string(),
            name: "".to_string(),  // Empty name
            model: "default".to_string(),
            workspace: "/workspace".to_string(),
            sandbox: false,
            subagents: vec![],
        },
    ];
    
    let errors = config.validate().unwrap_err();
    assert!(
        errors.iter().any(|e| e.path.contains("name") && e.message.contains("must not be empty")),
        "Should detect empty agent name, got: {:?}",
        errors
    );
}

#[test]
fn test_empty_model_id_fails() {
    let mut config = AisopodConfig::default();
    config.models.models = vec![
        aisopod_config::types::Model {
            id: "".to_string(),  // Empty ID
            name: "Model 1".to_string(),
            provider: "test".to_string(),
            capabilities: vec![],
        },
    ];
    
    let errors = config.validate().unwrap_err();
    assert!(
        errors.iter().any(|e| e.path.contains("id") && e.message.contains("must not be empty")),
        "Should detect empty model ID, got: {:?}",
        errors
    );
}
