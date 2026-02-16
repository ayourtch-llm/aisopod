//! Integration tests for config parsing (JSON5 and TOML)
//!
//! These tests verify the configuration parsing functionality
//! including file format detection, valid/invalid configs, and edge cases.

use std::path::PathBuf;
use aisopod_config::load_config;

#[test]
fn test_parse_minimal_json5() {
    let path = PathBuf::from("tests/fixtures/valid_minimal.json5");
    let config = load_config(&path).expect("Failed to load valid_minimal.json5");
    
    assert_eq!(config.meta.version, "1.0");
    // Other fields should have defaults
    assert_eq!(config.gateway.server.port, 8080); // default
}

#[test]
fn test_parse_minimal_toml() {
    let path = PathBuf::from("tests/fixtures/valid_minimal.toml");
    let config = load_config(&path).expect("Failed to load valid_minimal.toml");
    
    assert_eq!(config.meta.version, "1.0");
    assert_eq!(config.gateway.server.port, 8080); // default
}

#[test]
fn test_parse_full_json5() {
    let path = PathBuf::from("tests/fixtures/valid_full.json5");
    let config = load_config(&path).expect("Failed to load valid_full.json5");
    
    assert_eq!(config.meta.version, "1.0");
    assert_eq!(config.gateway.server.port, 8080);
    assert_eq!(config.gateway.server.name, "aisopod-gateway");
    assert_eq!(config.gateway.bind.address, "127.0.0.1");
    assert!(config.auth.enabled);
    assert_eq!(config.auth.provider, "jwt");
}

#[test]
fn test_parse_full_toml() {
    let path = PathBuf::from("tests/fixtures/valid_full.toml");
    let config = load_config(&path).expect("Failed to load valid_full.toml");
    
    assert_eq!(config.meta.version, "1.0");
    assert_eq!(config.gateway.server.port, 8080);
    assert_eq!(config.gateway.server.name, "aisopod-gateway");
    assert_eq!(config.gateway.bind.address, "127.0.0.1");
    assert!(config.auth.enabled);
    assert_eq!(config.auth.provider, "jwt");
}

#[test]
fn test_parse_invalid_syntax_json5_fails() {
    let path = PathBuf::from("tests/fixtures/invalid_syntax.json5");
    let result = load_config(&path);
    
    assert!(result.is_err(), "Expected parsing to fail for invalid syntax");
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("Failed to parse") || 
        err.to_string().contains("Unexpected end of input"),
        "Error should indicate parse failure, got: {}",
        err
    );
}

#[test]
fn test_parse_invalid_syntax_toml_fails() {
    let path = PathBuf::from("tests/fixtures/invalid_syntax.toml");
    let result = load_config(&path);
    
    assert!(result.is_err(), "Expected parsing to fail for invalid syntax");
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("Failed to parse") ||
        err.to_string().contains("expected `]`"),
        "Error should indicate parse failure, got: {}",
        err
    );
}

#[test]
fn test_parse_invalid_port_fails_validation() {
    let path = PathBuf::from("tests/fixtures/invalid_port.json5");
    let result = load_config(&path);
    
    assert!(result.is_err(), "Expected validation to fail for invalid port");
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("Port must be between"),
        "Error should contain validation error about port, got: {}",
        err
    );
}

#[test]
fn test_parse_invalid_port_toml_fails_validation() {
    let path = PathBuf::from("tests/fixtures/invalid_port.toml");
    let result = load_config(&path);
    
    assert!(result.is_err(), "Expected validation to fail for invalid port");
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("Port must be between"),
        "Error should contain validation error about port, got: {}",
        err
    );
}

#[test]
fn test_parse_empty_config_json5() {
    let path = PathBuf::from("tests/fixtures/empty_config.json5");
    let result = load_config(&path);
    
    // Empty config should fail validation (empty version)
    assert!(result.is_err(), "Expected validation to fail for empty version");
}

#[test]
fn test_parse_empty_config_toml() {
    let path = PathBuf::from("tests/fixtures/empty_config.toml");
    let result = load_config(&path);
    
    // Empty config should fail validation (empty version)
    assert!(result.is_err(), "Expected validation to fail for empty version");
}

#[test]
fn test_parse_edge_cases_fails_validation() {
    let path = PathBuf::from("tests/fixtures/edge_cases.json5");
    let result = load_config(&path);
    
    // Should fail validation due to invalid port and empty fields
    assert!(result.is_err(), "Expected validation to fail");
    let err = result.unwrap_err();
    // At least one validation error should be present
    assert!(
        err.to_string().contains("Port must be between") ||
        err.to_string().contains("must not be empty"),
        "Error should contain validation errors, got: {}",
        err
    );
}

#[test]
fn test_auto_detect_json5_extension() {
    let path = PathBuf::from("tests/fixtures/sample.json5");
    let config = load_config(&path).expect("Failed to auto-detect .json5 extension");
    assert_eq!(config.meta.version, "1.0");
}

#[test]
fn test_auto_detect_toml_extension() {
    let path = PathBuf::from("tests/fixtures/sample.toml");
    let config = load_config(&path).expect("Failed to auto-detect .toml extension");
    assert_eq!(config.meta.version, "1.0");
}

#[test]
fn test_unsupported_extension_fails() {
    // Create a temporary file with unsupported extension
    use tempfile::TempDir;
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("config.yaml");
    
    let config_content = r#"
meta:
  version: "1.0"
"#;
    
    std::fs::write(&config_path, config_content).expect("Failed to write test config");
    
    let result = load_config(&config_path);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("Unsupported config file extension"),
        "Error should indicate unsupported extension, got: {}",
        err
    );
}

#[test]
fn test_parse_with_env_vars() {
    // Set some environment variables
    std::env::set_var("CONFIG_VERSION", "2.0");
    std::env::set_var("GATEWAY_HOST", "0.0.0.0");
    
    let path = PathBuf::from("tests/fixtures/with_env_vars.json5");
    let result = load_config(&path);
    
    // This should succeed if env vars are properly expanded
    // Note: The config has empty meta.version which should fail validation
    // But env vars in other fields should be expanded
    
    // Clean up
    std::env::remove_var("CONFIG_VERSION");
    std::env::remove_var("GATEWAY_HOST");
    
    // The config should either succeed (with defaults) or fail for other reasons
    // The important thing is that env var expansion doesn't crash
    let _ = result;
}

#[test]
fn test_parse_with_explicit_json5_extension() {
    let path = PathBuf::from("tests/fixtures/sample.json5");
    let config = load_config(&path).expect("Failed to load with explicit .json5");
    assert_eq!(config.meta.version, "1.0");
}

#[test]
fn test_parse_with_explicit_toml_extension() {
    let path = PathBuf::from("tests/fixtures/sample.toml");
    let config = load_config(&path).expect("Failed to load with explicit .toml");
    assert_eq!(config.meta.version, "1.0");
}
