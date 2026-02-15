//! Integration tests for JSON5 config file parsing
//!
//! These tests verify the configuration loading functionality using
//! the sample fixture files.

use std::path::PathBuf;
use aisopod_config::load_config;

#[test]
fn test_load_sample_json5() {
    let path = PathBuf::from("tests/fixtures/sample.json5");
    let config = load_config(&path).expect("Failed to load config");
    
    assert_eq!(config.meta.version, "1.0");
    assert_eq!(config.gateway.server.port, 8080);
    assert_eq!(config.gateway.server.name, "aisopod-gateway");
    assert_eq!(config.gateway.bind.address, "127.0.0.1");
    assert!(!config.gateway.tls.enabled);
}

#[test]
fn test_load_sample_with_auto_detect() {
    let path = PathBuf::from("tests/fixtures/sample.json5");
    let config = load_config(&path).expect("Failed to load config");
    
    // Verify the config loads with auto-detection
    assert!(config.meta.version == "1.0");
}
