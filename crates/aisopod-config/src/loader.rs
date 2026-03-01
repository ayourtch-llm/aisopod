//! Configuration file loading module
//!
//! This module provides functions to load configuration files in various formats
//! (JSON5, JSON, TOML) and auto-detect the format based on file extension.

use std::collections::HashSet;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};

use crate::AisopodConfig;

/// Default configuration file name
pub const DEFAULT_CONFIG_FILE: &str = "aisopod-config.json5";

/// Get the default configuration file path.
///
/// Returns the path to the default config file in the current working directory.
pub fn default_config_path() -> PathBuf {
    std::env::current_dir()
        .expect("Failed to get current directory")
        .join(DEFAULT_CONFIG_FILE)
}

/// Check if a file has insecure permissions (world-readable or group-readable)
///
/// # Arguments
///
/// * `path` - Path to the file to check
///
/// # Returns
///
/// * `Ok(())` if permissions are secure (0600 or 0640)
/// * `Err(String)` if the file has world-readable permissions (security risk)
///
/// # Security Considerations
///
/// Configuration files often contain sensitive data like API keys, passwords,
/// and tokens. Files should not be world-readable (permissions ending in 4, 5, 6, or 7).
/// We recommend:
/// - 0600 (owner read/write only) - most secure
/// - 0640 (owner read/write, group read) - acceptable for multi-user systems
fn check_file_permissions(path: &Path) -> Result<()> {
    let metadata = fs::metadata(path)
        .with_context(|| format!("Failed to get metadata for config file: {}", path.display()))?;

    let permissions = metadata.permissions();
    let mode = permissions.mode();

    // Check if world-readable (others have read permission)
    // Octal: 0004, Binary: 00000100
    if mode & 0o004 != 0 {
        return Err(anyhow!(
            "Security violation: Config file '{}' is world-readable. \
             This is a security risk as it may contain sensitive data. \
             Please set permissions to 0600 (owner read/write only) or 0640 (owner read/write, group read). \
             Current permissions: {:o} (octal)",
            path.display(),
            mode & 0o777
        ));
    }

    // Check if group-readable (we recommend 0640 max)
    // Octal: 0040, Binary: 000100000
    if mode & 0o040 != 0 {
        tracing::warn!(
            "Config file '{}' is group-readable. For maximum security, \
             consider setting permissions to 0600 (owner read/write only). \
             Current permissions: {:o} (octal)",
            path.display(),
            mode & 0o777
        );
    }

    Ok(())
}

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
/// - The file has insecure permissions (world-readable)
/// - The file format is unsupported
/// - The file content cannot be parsed
/// - The parsed content doesn't match the expected configuration structure
pub fn load_config(path: &Path) -> Result<AisopodConfig> {
    // Security: Check file permissions before reading
    check_file_permissions(path)?;

    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
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
/// - The file has insecure permissions (world-readable)
/// - The file content cannot be parsed as valid TOML
pub fn load_config_toml(path: &Path) -> Result<AisopodConfig> {
    // Security: Check file permissions before reading
    check_file_permissions(path)?;

    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;
    let mut value: serde_json::Value = toml::from_str(&contents)
        .with_context(|| format!("Failed to parse TOML config: {}", path.display()))?;
    crate::env::expand_env_vars(&mut value).with_context(|| {
        format!(
            "Failed to expand environment variables in TOML config: {}",
            path.display()
        )
    })?;

    // Process @include directives
    let canonical = path.canonicalize()?;
    let base_dir = canonical
        .parent()
        .ok_or_else(|| anyhow!("Canonicalized config path has no parent directory"))?;
    let mut seen = HashSet::new();
    seen.insert(canonical.clone());
    crate::includes::process_includes(&mut value, base_dir, &mut seen).with_context(|| {
        format!(
            "Failed to process @include directives in TOML config: {}",
            path.display()
        )
    })?;

    let config: AisopodConfig = serde_json::from_value(value)
        .with_context(|| format!("Failed to deserialize TOML config: {}", path.display()))?;

    // Validate the configuration
    config.validate().map_err(|errs| {
        let messages: Vec<String> = errs.iter().map(|e| e.to_string()).collect();
        anyhow!(
            "TOML config validation failed:\n  {}",
            messages.join("\n  ")
        )
    })?;

    Ok(config)
}

/// Load a JSON5 configuration string directly (for testing).
///
/// This is a helper function for testing generated configs.
///
/// # Arguments
///
/// * `content` - JSON5 configuration string
///
/// # Returns
///
/// * `Result<AisopodConfig>` - The parsed configuration or an error
pub fn load_config_json5_str(content: &str) -> Result<AisopodConfig> {
    let mut value: serde_json::Value =
        json5::from_str(content).with_context(|| "Failed to parse JSON5 content")?;
    crate::env::expand_env_vars(&mut value)
        .with_context(|| "Failed to expand environment variables in JSON5 content")?;

    let config: AisopodConfig =
        serde_json::from_value(value).with_context(|| "Failed to deserialize JSON5 config")?;

    config.validate().map_err(|errs| {
        let messages: Vec<String> = errs.iter().map(|e| e.to_string()).collect();
        anyhow!(
            "JSON5 config validation failed:\n  {}",
            messages.join("\n  ")
        )
    })?;

    Ok(config)
}

/// Load a TOML configuration string directly (for testing).
///
/// This is a helper function for testing generated configs.
///
/// # Arguments
///
/// * `content` - TOML configuration string
///
/// # Returns
///
/// * `Result<AisopodConfig>` - The parsed configuration or an error
pub fn load_config_toml_str(content: &str) -> Result<AisopodConfig> {
    let value: serde_json::Value =
        toml::from_str(content).with_context(|| "Failed to parse TOML content")?;

    let config: AisopodConfig =
        serde_json::from_value(value).with_context(|| "Failed to deserialize TOML config")?;

    config.validate().map_err(|errs| {
        let messages: Vec<String> = errs.iter().map(|e| e.to_string()).collect();
        anyhow!(
            "TOML config validation failed:\n  {}",
            messages.join("\n  ")
        )
    })?;

    Ok(config)
}

pub fn load_config_json5(path: &Path) -> Result<AisopodConfig> {
    // Security: Check file permissions before reading
    check_file_permissions(path)?;

    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;
    let mut value: serde_json::Value = json5::from_str(&contents)
        .with_context(|| format!("Failed to parse JSON5 config: {}", path.display()))?;
    crate::env::expand_env_vars(&mut value).with_context(|| {
        format!(
            "Failed to expand environment variables in config: {}",
            path.display()
        )
    })?;

    // Process @include directives
    let canonical = path.canonicalize()?;
    let base_dir = canonical
        .parent()
        .ok_or_else(|| anyhow!("Canonicalized config path has no parent directory"))?;
    let mut seen = HashSet::new();
    seen.insert(canonical.clone());
    crate::includes::process_includes(&mut value, base_dir, &mut seen).with_context(|| {
        format!(
            "Failed to process @include directives in config: {}",
            path.display()
        )
    })?;

    let config: AisopodConfig = serde_json::from_value(value)
        .with_context(|| format!("Failed to deserialize config: {}", path.display()))?;

    // Validate the configuration
    config.validate().map_err(|errs| {
        let messages: Vec<String> = errs.iter().map(|e| e.to_string()).collect();
        anyhow!("Config validation failed:\n  {}", messages.join("\n  "))
    })?;

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;
    use tempfile::TempDir;

    // Security hardening tests for check_file_permissions
    #[test]
    fn test_check_file_permissions_secure() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("config.json5");

        // Write a config file with secure permissions (0600)
        fs::write(&config_path, r#"{}"#).expect("Failed to write test config");
        fs::set_permissions(&config_path, std::fs::Permissions::from_mode(0o600))
            .expect("Failed to set permissions");

        let result = check_file_permissions(&config_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_file_permissions_world_readable() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("config.json5");

        // Write a config file with world-readable permissions
        fs::write(&config_path, r#"{}"#).expect("Failed to write test config");
        fs::set_permissions(&config_path, std::fs::Permissions::from_mode(0o644))
            .expect("Failed to set permissions");

        let result = check_file_permissions(&config_path);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("world-readable"));
        assert!(err.to_string().contains("security risk"));
    }

    #[test]
    fn test_check_file_permissions_group_readable() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("config.json5");

        // Write a config file with group-readable permissions (should warn but not error)
        fs::write(&config_path, r#"{}"#).expect("Failed to write test config");
        fs::set_permissions(&config_path, std::fs::Permissions::from_mode(0o640))
            .expect("Failed to set permissions");

        let result = check_file_permissions(&config_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_file_permissions_invalid_path() {
        let invalid_path = PathBuf::from("/nonexistent/path/config.json5");
        let result = check_file_permissions(&invalid_path);
        assert!(result.is_err());
    }

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
        // Set secure permissions (0600) before loading
        fs::set_permissions(&config_path, std::fs::Permissions::from_mode(0o600))
            .expect("Failed to set permissions");

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
        // Set secure permissions (0600) before loading
        fs::set_permissions(&config_path, std::fs::Permissions::from_mode(0o600))
            .expect("Failed to set permissions");

        let config = load_config(&config_path).expect("Failed to load config");
        assert_eq!(config.meta.version, "2.0");
    }

    #[test]
    fn test_load_config_auto_detect_json() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("config.json");

        let config_content = r#"{ "meta": { "version": "3.0" } }"#;
        fs::write(&config_path, config_content).expect("Failed to write test config");
        // Set secure permissions (0600) before loading
        fs::set_permissions(&config_path, std::fs::Permissions::from_mode(0o600))
            .expect("Failed to set permissions");

        let config = load_config(&config_path).expect("Failed to load config");
        assert_eq!(config.meta.version, "3.0");
    }

    #[test]
    fn test_load_config_unsupported_extension() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("config.txt");

        // The file might not exist, so we skip permission check for this test
        // or just create an empty file with proper permissions
        fs::write(&config_path, "").expect("Failed to write test config");
        fs::set_permissions(&config_path, std::fs::Permissions::from_mode(0o600))
            .expect("Failed to set permissions");

        let result = load_config(&config_path);
        assert!(result.is_err());
        let err_msg = result.expect_err("Expected error for unsupported extension");
        assert!(err_msg
            .to_string()
            .contains("Unsupported config file extension"));
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
        // Set secure permissions (0600) before loading
        fs::set_permissions(&config_path, std::fs::Permissions::from_mode(0o600))
            .expect("Failed to set permissions");

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
        // Set secure permissions (0600) before loading
        fs::set_permissions(&config_path, std::fs::Permissions::from_mode(0o600))
            .expect("Failed to set permissions");

        let config = load_config(&config_path).expect("Failed to load config");
        assert_eq!(config.meta.version, "5.0");
    }

    #[test]
    fn test_load_config_file_not_found() {
        let config_path = PathBuf::from("/nonexistent/path/config.json5");
        let result = load_config_json5(&config_path);

        assert!(result.is_err());
        let err_msg = result.expect_err("Expected error for non-existent file");
        assert!(err_msg
            .to_string()
            .contains("/nonexistent/path/config.json5"));
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
        assert!(err_msg
            .to_string()
            .contains(&config_path.display().to_string()));
    }
}
