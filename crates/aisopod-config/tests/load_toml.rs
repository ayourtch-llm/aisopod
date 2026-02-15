//! Integration tests for TOML config file parsing
//!
//! These tests verify the configuration loading functionality using
//! the sample fixture files.

use std::path::PathBuf;
use aisopod_config::load_config;

#[test]
fn test_load_sample_toml() {
    let path = PathBuf::from("tests/fixtures/sample.toml");
    let config = load_config(&path).expect("Failed to load config");
    
    assert_eq!(config.meta.version, "1.0");
    assert_eq!(config.gateway.server.port, 8080);
    assert_eq!(config.gateway.server.name, "aisopod-gateway");
    assert_eq!(config.gateway.bind.address, "127.0.0.1");
    assert!(!config.gateway.tls.enabled);
}

#[test]
fn test_load_sample_with_auto_detect() {
    let path = PathBuf::from("tests/fixtures/sample.toml");
    let config = load_config(&path).expect("Failed to load config");
    
    // Verify the config loads with auto-detection
    assert_eq!(config.meta.version, "1.0");
    assert_eq!(config.gateway.server.port, 8080);
}

#[test]
fn test_load_config_toml_basic() {
    use tempfile::TempDir;
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("config.toml");
    
    let config_content = r#"
[meta]
version = "2.0"

[gateway.server]
port = 9090
"#;
    
    std::fs::write(&config_path, config_content).expect("Failed to write test config");
    
    let config = load_config(&config_path).expect("Failed to load config");
    assert_eq!(config.meta.version, "2.0");
    assert_eq!(config.gateway.server.port, 9090);
}

#[test]
fn test_load_config_toml_nested_tables() {
    use tempfile::TempDir;
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("config.toml");
    
    let config_content = r#"
[meta]
version = "3.0"

[gateway.bind]
address = "0.0.0.0"
port = 8888
"#;
    
    std::fs::write(&config_path, config_content).expect("Failed to write test config");
    
    let config = load_config(&config_path).expect("Failed to load config");
    assert_eq!(config.meta.version, "3.0");
    assert_eq!(config.gateway.bind.address, "0.0.0.0");
}

#[test]
fn test_load_config_toml_with_comments() {
    use tempfile::TempDir;
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("config.toml");
    
    let config_content = r#"
# This is a comment
[meta]
version = "4.0"  # inline comment

[gateway.server]
port = 7777  # another inline comment
"#;
    
    std::fs::write(&config_path, config_content).expect("Failed to write test config");
    
    let config = load_config(&config_path).expect("Failed to load config");
    assert_eq!(config.meta.version, "4.0");
    assert_eq!(config.gateway.server.port, 7777);
}

#[test]
fn test_load_config_toml_file_not_found() {
    let config_path = PathBuf::from("/nonexistent/path/config.toml");
    let result = load_config(&config_path);

    assert!(result.is_err());
    let err_msg = result.expect_err("Expected error for non-existent file");
    assert!(err_msg.to_string().contains("/nonexistent/path/config.toml"));
}

#[test]
fn test_load_config_toml_invalid_syntax() {
    use tempfile::TempDir;
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("config.toml");
    
    // Invalid TOML: missing closing bracket
    let config_content = r#"
[meta
version = "invalid"
"#;
    
    std::fs::write(&config_path, config_content).expect("Failed to write test config");
    
    let result = load_config(&config_path);
    assert!(result.is_err());
    let err_msg = result.expect_err("Expected error for invalid TOML");
    assert!(err_msg.to_string().contains(&config_path.display().to_string()));
}
