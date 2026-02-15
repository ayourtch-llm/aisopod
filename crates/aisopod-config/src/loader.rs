//! Configuration file loading module
//!
//! This module provides functions to load configuration files in various formats
//! (JSON5, JSON, TOML) and auto-detect the format based on file extension.

use std::path::Path;

use anyhow::{anyhow, Context, Result};

use crate::AisopodConfig;

/// Load a configuration file, auto-detecting format from extension.
///
/// Supports the following file extensions:
/// - `.json5` - JSON5 format with comments, trailing commas, and unquoted keys
/// - `.json` - Standard JSON format
/// - `.toml` - TOML format
///
/// # Arguments
///
/// * `path` - Path to the configuration file
///
/// # Returns
///
/// * `Result<AisopodConfig>` - The parsed configuration or an error
///
/// # Errors
///
/// Returns an error if:
/// - The file cannot be read
/// - The file format is unsupported
/// - The file content cannot be parsed
/// - The parsed content doesn't match the expected configuration structure
pub fn load_config(path: &Path) -> Result<AisopodConfig> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    match ext {
        "json" | "json5" => load_config_json5(path),
        "toml" => load_config_toml(path),
        _ => Err(anyhow!(
            "Unsupported config file extension: '{}'. Use .json5, .json, or .toml",
            ext
        )),
    }
}

/// Load and parse a TOML configuration file.
///
/// # Arguments
///
/// * `path` - Path to the TOML configuration file
///
/// # Returns
///
/// * `Result<AisopodConfig>` - The parsed configuration or an error
///
/// # Errors
///
/// Returns an error if:
/// - The file cannot be read
/// - The file content cannot be parsed as valid TOML
pub fn load_config_toml(path: &Path) -> Result<AisopodConfig> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;
    let config: AisopodConfig = toml::from_str(&contents)
        .with_context(|| format!("Failed to parse TOML config: {}", path.display()))?;
    Ok(config)
}

pub fn load_config_json5(path: &Path) -> Result<AisopodConfig> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;
    let config: AisopodConfig = json5::from_str(&contents)
        .with_context(|| format!("Failed to parse JSON5 config: {}", path.display()))?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_load_config_json5_basic() {
        // Create a temporary directory for test files
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("config.json5");

        // Write a basic JSON5 config
        let config_content = r#"
{
    // Basic configuration
    meta: {
        version: "1.0",
    },
    gateway: {
        server: {
            port: 8080,
        },
        bind: {
            address: "127.0.0.1",
        },
    },
}
"#;
        fs::write(&config_path, config_content).expect("Failed to write test config");

        // Load and verify
        let config = load_config_json5(&config_path).expect("Failed to load config");
        assert_eq!(config.meta.version, "1.0");
        assert_eq!(config.gateway.server.port, 8080);
        assert_eq!(config.gateway.bind.address, "127.0.0.1");
    }

    #[test]
    fn test_load_config_auto_detect_json5() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("config.json5");

        let config_content = r#"{ meta: { version: "2.0" } }"#;
        fs::write(&config_path, config_content).expect("Failed to write test config");

        let config = load_config(&config_path).expect("Failed to load config");
        assert_eq!(config.meta.version, "2.0");
    }

    #[test]
    fn test_load_config_auto_detect_json() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("config.json");

        let config_content = r#"{ "meta": { "version": "3.0" } }"#;
        fs::write(&config_path, config_content).expect("Failed to write test config");

        let config = load_config(&config_path).expect("Failed to load config");
        assert_eq!(config.meta.version, "3.0");
    }

    #[test]
    fn test_load_config_unsupported_extension() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("config.txt");

        let result = load_config(&config_path);
        assert!(result.is_err());
        let err_msg = result.expect_err("Expected error for unsupported extension");
        assert!(err_msg.to_string().contains("Unsupported config file extension"));
    }

    #[test]
    fn test_load_config_json5_with_comments() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("config.json5");

        let config_content = r#"
// This is a comment
{
    /* Multi-line
       comment */
    meta: {
        version: "4.0", // trailing comment
    },
}
"#;
        fs::write(&config_path, config_content).expect("Failed to write test config");

        let config = load_config(&config_path).expect("Failed to load config");
        assert_eq!(config.meta.version, "4.0");
    }

    #[test]
    fn test_load_config_json5_with_trailing_comma() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("config.json5");

        let config_content = r#"
{
    meta: {
        version: "5.0",
    },
}
"#;
        fs::write(&config_path, config_content).expect("Failed to write test config");

        let config = load_config(&config_path).expect("Failed to load config");
        assert_eq!(config.meta.version, "5.0");
    }

    #[test]
    fn test_load_config_file_not_found() {
        let config_path = PathBuf::from("/nonexistent/path/config.json5");
        let result = load_config_json5(&config_path);

        assert!(result.is_err());
        let err_msg = result.expect_err("Expected error for non-existent file");
        assert!(err_msg.to_string().contains("/nonexistent/path/config.json5"));
    }

    #[test]
    fn test_load_config_invalid_json5() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("config.json5");

        let config_content = r#"{ meta: { version: "invalid" }"#; // Missing closing brace
        fs::write(&config_path, config_content).expect("Failed to write test config");

        let result = load_config_json5(&config_path);
        assert!(result.is_err());
        let err_msg = result.expect_err("Expected error for invalid JSON5");
        assert!(err_msg.to_string().contains(&config_path.display().to_string()));
    }
}
