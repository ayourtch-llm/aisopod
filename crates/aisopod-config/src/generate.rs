//! Configuration generation module
//!
//! This module provides functionality to generate default configuration files
//! with inline documentation and sensible default values.

use crate::types::AisopodConfig;
use anyhow::{anyhow, Result};
use std::fs;
use std::os::unix::fs::PermissionsExt;

/// Configuration format enum for code generation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConfigFormat {
    /// JSON5 format with comments, trailing commas, and unquoted keys
    Json5,
    /// TOML format
    Toml,
}

/// Generate a default configuration file as a formatted string.
///
/// # Arguments
///
/// * `format` - The desired configuration format (Json5 or Toml)
///
/// # Returns
///
/// * `Result<String>` - The formatted configuration string or an error
///
/// # Errors
///
/// Returns an error if serialization to the requested format fails.
///
/// # Examples
///
/// ```no_run
/// use aisopod_config::{generate_default_config, ConfigFormat};
///
/// let json5_config = generate_default_config(ConfigFormat::Json5).unwrap();
/// let toml_config = generate_default_config(ConfigFormat::Toml).unwrap();
/// ```
pub fn generate_default_config(format: ConfigFormat) -> Result<String> {
    let config = AisopodConfig::default();
    generate_config_with_format(&config, format)
}

/// Generate a configuration file with a custom config as a formatted string.
///
/// # Arguments
///
/// * `config` - The configuration to serialize
/// * `format` - The desired configuration format (Json5 or Toml)
///
/// # Returns
///
/// * `Result<String>` - The formatted configuration string or an error
pub fn generate_config_with_format(config: &AisopodConfig, format: ConfigFormat) -> Result<String> {
    match format {
        ConfigFormat::Json5 => generate_json5(config),
        ConfigFormat::Toml => generate_toml(config),
    }
}

/// Generate header comments for the configuration file
fn generate_header(format: ConfigFormat) -> &'static str {
    match format {
        ConfigFormat::Json5 => {
            "// Aisopod default configuration\n\
                                // Edit this file to customize your setup.\n\
                                // See documentation for all available options.\n\n"
        }
        ConfigFormat::Toml => {
            "# Aisopod default configuration\n\
                               # Edit this file to customize your setup.\n\
                               # See documentation for all available options.\n\n"
        }
    }
}

/// Generate a JSON5 configuration string
fn generate_json5(config: &AisopodConfig) -> Result<String> {
    let json_value = serde_json::to_value(config)
        .map_err(|e| anyhow!("Failed to serialize config to JSON: {}", e))?;

    // Use to_string_pretty for clean JSON output
    let raw = serde_json::to_string_pretty(&json_value)
        .map_err(|e| anyhow!("Failed to format JSON: {}", e))?;

    // Convert to JSON5 format (unquoted keys for better readability)
    let json5_str = convert_json_to_json5(&raw);

    let header = generate_header(ConfigFormat::Json5);
    Ok(format!("{}{}\n", header, json5_str))
}

/// Convert standard JSON string to JSON5 format
fn convert_json_to_json5(json_str: &str) -> String {
    // For simple cases, JSON5 is a superset of JSON
    // We can use the JSON string directly, but JSON5 allows unquoted keys
    // For now, we output valid JSON5 that is also valid JSON for compatibility
    // The json5 crate can parse this fine
    json_str.to_string()
}

/// Generate a TOML configuration string
fn generate_toml(config: &AisopodConfig) -> Result<String> {
    let raw = toml::to_string_pretty(config)
        .map_err(|e| anyhow!("Failed to serialize config to TOML: {}", e))?;

    let header = generate_header(ConfigFormat::Toml);
    Ok(format!("{}{}\n", header, raw))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loader::{load_config_json5, load_config_toml};
    use crate::loader::{load_config_json5_str, load_config_toml_str};
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_generate_default_json5() {
        let output = generate_default_config(ConfigFormat::Json5).unwrap();

        // Check for header comment
        assert!(output.contains("Aisopod default configuration"));
        assert!(output.contains("//"));

        // Verify it's parseable JSON5
        let config = load_config_json5_from_str(&output).unwrap();
        let defaults = AisopodConfig::default();
        assert_eq!(config.meta.version, defaults.meta.version);
    }

    #[test]
    fn test_generate_default_toml() {
        let output = generate_default_config(ConfigFormat::Toml).unwrap();

        // Check for header comment
        assert!(output.contains("Aisopod default configuration"));
        assert!(output.contains("# "));

        // Verify it's parseable TOML
        let config = load_config_toml_from_str(&output).unwrap();
        let defaults = AisopodConfig::default();
        assert_eq!(config.meta.version, defaults.meta.version);
    }

    #[test]
    fn test_generated_json5_is_parseable() {
        // Create temp file and write JSON5 config
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("config.json5");

        let output = generate_default_config(ConfigFormat::Json5).unwrap();
        fs::write(&config_path, &output).expect("Failed to write test config");
        // Set secure permissions (0600) before loading
        fs::set_permissions(&config_path, std::fs::Permissions::from_mode(0o600))
            .expect("Failed to set permissions");

        // Parse it back
        let config = load_config_json5(&config_path).expect("Failed to load JSON5 config");

        // Verify defaults match
        let defaults = AisopodConfig::default();
        assert_eq!(config.meta.version, defaults.meta.version);
        assert_eq!(config.gateway.server.port, defaults.gateway.server.port);
        assert_eq!(config.gateway.bind.address, defaults.gateway.bind.address);
    }

    #[test]
    fn test_generated_toml_is_parseable() {
        // Create temp file and write TOML config
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("config.toml");

        let output = generate_default_config(ConfigFormat::Toml).unwrap();
        fs::write(&config_path, &output).expect("Failed to write test config");
        // Set secure permissions (0600) before loading
        fs::set_permissions(&config_path, std::fs::Permissions::from_mode(0o600))
            .expect("Failed to set permissions");

        // Parse it back
        let config = load_config_toml(&config_path).expect("Failed to load TOML config");

        // Verify defaults match
        let defaults = AisopodConfig::default();
        assert_eq!(config.meta.version, defaults.meta.version);
        assert_eq!(config.gateway.server.port, defaults.gateway.server.port);
        assert_eq!(config.gateway.bind.address, defaults.gateway.bind.address);
    }

    #[test]
    fn test_config_format_enum() {
        assert_eq!(ConfigFormat::Json5, ConfigFormat::Json5);
        assert_eq!(ConfigFormat::Toml, ConfigFormat::Toml);
        assert_ne!(ConfigFormat::Json5, ConfigFormat::Toml);
    }

    #[test]
    fn test_generate_config_with_format() {
        let config = AisopodConfig::default();

        let json5_output = generate_config_with_format(&config, ConfigFormat::Json5).unwrap();
        let toml_output = generate_config_with_format(&config, ConfigFormat::Toml).unwrap();

        assert!(json5_output.contains("Aisopod default configuration"));
        assert!(toml_output.contains("Aisopod default configuration"));
    }

    #[test]
    fn test_default_values_match() {
        let output = generate_default_config(ConfigFormat::Json5).unwrap();
        let config = load_config_json5_from_str(&output).unwrap();

        let defaults = AisopodConfig::default();

        // Verify key default values
        assert_eq!(config.meta.version, defaults.meta.version);
        assert_eq!(config.gateway.server.port, defaults.gateway.server.port);
        assert_eq!(config.gateway.server.name, defaults.gateway.server.name);
        assert_eq!(config.gateway.bind.address, defaults.gateway.bind.address);
        assert_eq!(config.gateway.bind.ipv6, defaults.gateway.bind.ipv6);
        assert_eq!(config.gateway.tls.enabled, defaults.gateway.tls.enabled);
        assert_eq!(
            config.session.messages.max_messages,
            defaults.session.messages.max_messages
        );
        assert_eq!(config.tools.bash.enabled, defaults.tools.bash.enabled);
        assert_eq!(
            config.skills.settings.timeout,
            defaults.skills.settings.timeout
        );
    }

    #[test]
    fn test_generated_configs_are_valid() {
        // Test JSON5 validity
        let json5_output = generate_default_config(ConfigFormat::Json5).unwrap();
        let json5_config = load_config_json5_from_str(&json5_output).unwrap();

        // Test TOML validity
        let toml_output = generate_default_config(ConfigFormat::Toml).unwrap();
        let toml_config = load_config_toml_from_str(&toml_output).unwrap();

        // Both should have the same default values
        assert_eq!(json5_config.meta.version, toml_config.meta.version);
        assert_eq!(
            json5_config.gateway.server.port,
            toml_config.gateway.server.port
        );
    }
}

// Helper functions for parsing from string
#[cfg(test)]
fn load_config_json5_from_str(content: &str) -> Result<AisopodConfig> {
    crate::load_config_json5_str(content).map_err(|e| anyhow!("Failed to load: {}", e))
}

#[cfg(test)]
fn load_config_toml_from_str(content: &str) -> Result<AisopodConfig> {
    crate::load_config_toml_str(content).map_err(|e| anyhow!("Failed to load: {}", e))
}
