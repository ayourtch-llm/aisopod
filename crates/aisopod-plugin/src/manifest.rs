//! Plugin manifest format and parser.
//!
//! This module defines the `aisopod.plugin.toml` manifest schema for describing
//! plugin metadata and capabilities, along with a parser for reading and validating
//! these manifests.
//!
//! # Manifest Format
//!
//! The manifest file uses TOML format and contains three main sections:
//!
//! ```toml
//! [plugin]
//! id = "my-plugin"
//! name = "My Plugin"
//! version = "0.1.0"
//! description = "A sample plugin"
//! author = "Author Name"
//! entry_point = "libmy_plugin"
//!
//! [capabilities]
//! channels = ["custom-channel"]
//! tools = ["custom-tool"]
//! providers = []
//! commands = ["my-command"]
//! hooks = ["BeforeAgentRun", "AfterAgentRun"]
//!
//! [compatibility]
//! min_host_version = "0.1.0"
//! max_host_version = "1.0.0"
//! ```
//!
//! # Usage
//!
//! ```ignore
//! use aisopod_plugin::manifest::PluginManifest;
//!
//! let manifest = PluginManifest::from_file("aisopod.plugin.toml")?;
//! println!("Plugin: {} v{}", manifest.plugin.name, manifest.plugin.version);
//! ```
//!
//! # Error Handling
//!
//! The parser returns a `ManifestError` for various failure modes:
//!
//! - `ManifestError::Io` - File reading errors
//! - `ManifestError::Parse` - TOML parsing errors
//! - `ManifestError::Validation` - Validation errors for missing or invalid fields

use std::fs;
use std::path::Path;

use semver::Version;
use serde::Deserialize;
use thiserror::Error;

/// Top-level plugin manifest structure.
///
/// This struct represents the complete `aisopod.plugin.toml` manifest file,
/// containing plugin metadata, capabilities, and compatibility information.
///
/// # Fields
///
/// * `plugin` - Core plugin information (id, name, version, entry point)
/// * `capabilities` - Optional capabilities this plugin provides
/// * `compatibility` - Optional host version compatibility constraints
#[derive(Debug, Clone, Deserialize)]
pub struct PluginManifest {
    /// Core plugin information.
    pub plugin: PluginManifestInfo,
    /// Plugin capabilities (channels, tools, etc.).
    #[serde(default)]
    pub capabilities: Option<PluginCapabilities>,
    /// Host version compatibility constraints.
    #[serde(default)]
    pub compatibility: Option<PluginCompatibility>,
}

/// Core plugin information.
///
/// This struct contains the mandatory fields that identify and describe a plugin.
///
/// # Fields
///
/// * `id` - Unique identifier for the plugin (e.g., "my-plugin")
/// * `name` - Human-readable display name (e.g., "My Plugin")
/// * `version` - Semantic version string (e.g., "0.1.0")
/// * `description` - Brief description of plugin functionality
/// * `author` - Plugin author or organization
/// * `entry_point` - Library name without path or extension (e.g., "libmy_plugin")
#[derive(Debug, Clone, Deserialize)]
pub struct PluginManifestInfo {
    /// Unique identifier for the plugin.
    pub id: String,
    /// Human-readable display name.
    pub name: String,
    /// Semantic version string.
    pub version: String,
    /// Brief description of plugin functionality.
    pub description: String,
    /// Plugin author or organization.
    pub author: String,
    /// Library name without path or extension.
    pub entry_point: String,
}

/// Plugin capabilities.
///
/// This struct describes what the plugin can do - which channels, tools,
/// providers, commands, and hooks it supports.
///
/// # Fields
///
/// * `channels` - List of channel types this plugin supports
/// * `tools` - List of tool names this plugin provides
/// * `providers` - List of provider types this plugin supports
/// * `commands` - List of CLI commands this plugin provides
/// * `hooks` - List of lifecycle hooks this plugin handles
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
pub struct PluginCapabilities {
    /// Channel types this plugin supports (e.g., "text", "voice", "dm").
    #[serde(default)]
    pub channels: Option<Vec<String>>,
    /// Tool names this plugin provides.
    #[serde(default)]
    pub tools: Option<Vec<String>>,
    /// Provider types this plugin supports.
    #[serde(default)]
    pub providers: Option<Vec<String>>,
    /// CLI commands this plugin provides.
    #[serde(default)]
    pub commands: Option<Vec<String>>,
    /// Lifecycle hooks this plugin handles.
    #[serde(default)]
    pub hooks: Option<Vec<String>>,
}

/// Host version compatibility constraints.
///
/// This struct specifies the range of host versions the plugin is compatible with.
///
/// # Fields
///
/// * `min_host_version` - Minimum compatible host version (inclusive)
/// * `max_host_version` - Maximum compatible host version (inclusive)
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
pub struct PluginCompatibility {
    /// Minimum compatible host version (inclusive).
    #[serde(default)]
    pub min_host_version: Option<String>,
    /// Maximum compatible host version (inclusive).
    #[serde(default)]
    pub max_host_version: Option<String>,
}

/// Errors that can occur during manifest parsing and validation.
#[derive(Debug, Error)]
pub enum ManifestError {
    /// File I/O error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// TOML parsing error.
    #[error("Parse error: {0}")]
    Parse(String),

    /// Validation error for missing or invalid fields.
    #[error("Validation error: {0}")]
    Validation(String),
}

impl PluginManifest {
    /// Creates a new `PluginManifest` from a file path.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the manifest file
    ///
    /// # Returns
    ///
    /// * `Ok(PluginManifest)` - Successfully parsed and validated manifest
    /// * `Err(ManifestError)` - Error reading or parsing the file
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use aisopod_plugin::manifest::PluginManifest;
    /// use std::path::Path;
    ///
    /// let manifest = PluginManifest::from_file(Path::new("aisopod.plugin.toml"))?;
    /// ```
    pub fn from_file(path: &Path) -> Result<Self, ManifestError> {
        let content = fs::read_to_string(path).map_err(ManifestError::Io)?;
        Self::from_str(&content)
    }

    /// Creates a new `PluginManifest` from a string.
    ///
    /// # Arguments
    ///
    /// * `content` - TOML string content
    ///
    /// # Returns
    ///
    /// * `Ok(PluginManifest)` - Successfully parsed and validated manifest
    /// * `Err(ManifestError)` - Error parsing TOML or validation
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use aisopod_plugin::manifest::PluginManifest;
    ///
    /// let manifest = PluginManifest::from_str(r#"
    ///     [plugin]
    ///     id = "my-plugin"
    ///     name = "My Plugin"
    ///     version = "0.1.0"
    ///     description = "A sample plugin"
    ///     author = "Author Name"
    ///     entry_point = "libmy_plugin"
    /// "#)?;
    /// ```
    pub fn from_str(content: &str) -> Result<Self, ManifestError> {
        let manifest: PluginManifest = toml::from_str(content)
            .map_err(|e| ManifestError::Parse(e.to_string()))?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Validates the manifest fields.
    ///
    /// This method checks that all required fields are present and valid:
    /// - `id` must not be empty
    /// - `name` must not be empty
    /// - `version` must be valid semantic versioning
    /// - `entry_point` must not be empty
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Manifest is valid
    /// * `Err(ManifestError)` - Validation failed with details
    fn validate(&self) -> Result<(), ManifestError> {
        // Validate plugin id is not empty
        if self.plugin.id.trim().is_empty() {
            return Err(ManifestError::Validation(
                "plugin.id must not be empty".to_string(),
            ));
        }

        // Validate plugin name is not empty
        if self.plugin.name.trim().is_empty() {
            return Err(ManifestError::Validation(
                "plugin.name must not be empty".to_string(),
            ));
        }

        // Validate version is valid semver
        Version::parse(&self.plugin.version).map_err(|_| {
            ManifestError::Validation(format!(
                "plugin.version '{}' is not valid semver",
                self.plugin.version
            ))
        })?;

        // Validate entry_point is not empty
        if self.plugin.entry_point.trim().is_empty() {
            return Err(ManifestError::Validation(
                "plugin.entry_point must not be empty".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str_valid_minimal() {
        let content = r#"
            [plugin]
            id = "my-plugin"
            name = "My Plugin"
            version = "0.1.0"
            description = "A sample plugin"
            author = "Author Name"
            entry_point = "libmy_plugin"
        "#;

        let result = PluginManifest::from_str(content);
        assert!(result.is_ok(), "Valid manifest should parse successfully");
        
        let manifest = result.unwrap();
        assert_eq!(manifest.plugin.id, "my-plugin");
        assert_eq!(manifest.plugin.name, "My Plugin");
        assert_eq!(manifest.plugin.version, "0.1.0");
        assert_eq!(manifest.plugin.description, "A sample plugin");
        assert_eq!(manifest.plugin.author, "Author Name");
        assert_eq!(manifest.plugin.entry_point, "libmy_plugin");
        assert!(manifest.capabilities.is_none());
        assert!(manifest.compatibility.is_none());
    }

    #[test]
    fn test_from_str_valid_with_capabilities() {
        let content = r#"
            [plugin]
            id = "my-plugin"
            name = "My Plugin"
            version = "0.1.0"
            description = "A sample plugin"
            author = "Author Name"
            entry_point = "libmy_plugin"

            [capabilities]
            channels = ["text", "voice"]
            tools = ["custom-tool"]
            commands = ["my-command"]
        "#;

        let result = PluginManifest::from_str(content);
        assert!(result.is_ok(), "Valid manifest with capabilities should parse");

        let manifest = result.unwrap();
        let caps = manifest.capabilities.unwrap();
        assert_eq!(caps.channels, Some(vec!["text".to_string(), "voice".to_string()]));
        assert_eq!(caps.tools, Some(vec!["custom-tool".to_string()]));
        assert_eq!(caps.commands, Some(vec!["my-command".to_string()]));
    }

    #[test]
    fn test_from_str_valid_with_compatibility() {
        let content = r#"
            [plugin]
            id = "my-plugin"
            name = "My Plugin"
            version = "0.1.0"
            description = "A sample plugin"
            author = "Author Name"
            entry_point = "libmy_plugin"

            [compatibility]
            min_host_version = "0.1.0"
            max_host_version = "1.0.0"
        "#;

        let result = PluginManifest::from_str(content);
        assert!(result.is_ok(), "Valid manifest with compatibility should parse");

        let manifest = result.unwrap();
        let compat = manifest.compatibility.unwrap();
        assert_eq!(compat.min_host_version, Some("0.1.0".to_string()));
        assert_eq!(compat.max_host_version, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_validation_empty_id() {
        let content = r#"
            [plugin]
            id = ""
            name = "My Plugin"
            version = "0.1.0"
            description = "A sample plugin"
            author = "Author Name"
            entry_point = "libmy_plugin"
        "#;

        let result = PluginManifest::from_str(content);
        assert!(result.is_err());
        match result {
            Err(ManifestError::Validation(msg)) => {
                assert!(msg.contains("id"));
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_validation_empty_name() {
        let content = r#"
            [plugin]
            id = "my-plugin"
            name = ""
            version = "0.1.0"
            description = "A sample plugin"
            author = "Author Name"
            entry_point = "libmy_plugin"
        "#;

        let result = PluginManifest::from_str(content);
        assert!(result.is_err());
        match result {
            Err(ManifestError::Validation(msg)) => {
                assert!(msg.contains("name"));
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_validation_invalid_version() {
        let content = r#"
            [plugin]
            id = "my-plugin"
            name = "My Plugin"
            version = "not-a-version"
            description = "A sample plugin"
            author = "Author Name"
            entry_point = "libmy_plugin"
        "#;

        let result = PluginManifest::from_str(content);
        assert!(result.is_err());
        match result {
            Err(ManifestError::Validation(msg)) => {
                assert!(msg.contains("version"));
                assert!(msg.contains("semver"));
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_validation_empty_entry_point() {
        let content = r#"
            [plugin]
            id = "my-plugin"
            name = "My Plugin"
            version = "0.1.0"
            description = "A sample plugin"
            author = "Author Name"
            entry_point = ""
        "#;

        let result = PluginManifest::from_str(content);
        assert!(result.is_err());
        match result {
            Err(ManifestError::Validation(msg)) => {
                assert!(msg.contains("entry_point"));
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_validation_whitespace_id() {
        let content = r#"
            [plugin]
            id = "   "
            name = "My Plugin"
            version = "0.1.0"
            description = "A sample plugin"
            author = "Author Name"
            entry_point = "libmy_plugin"
        "#;

        let result = PluginManifest::from_str(content);
        assert!(result.is_err());
        match result {
            Err(ManifestError::Validation(msg)) => {
                assert!(msg.contains("id"));
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_from_str_parse_error() {
        let content = r#"
            [plugin]
            id = "my-plugin"
            name = "My Plugin"
            # Missing closing quote
            version = "0.1.0
            description = "A sample plugin"
        "#;

        let result = PluginManifest::from_str(content);
        assert!(result.is_err());
        match result {
            Err(ManifestError::Parse(_msg)) => {
                // Just check that we got a Parse error
            }
            _ => panic!("Expected Parse error"),
        }
    }

    #[test]
    fn test_from_str_multiple_errors() {
        let content = r#"
            [plugin]
            id = ""
            name = ""
            version = "invalid-version"
            description = "A sample plugin"
            author = "Author Name"
            entry_point = ""
        "#;

        let result = PluginManifest::from_str(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_from_str_semver_edge_cases() {
        // Valid semver with prerelease
        let content = r#"
            [plugin]
            id = "my-plugin"
            name = "My Plugin"
            version = "1.0.0-alpha.1"
            description = "A sample plugin"
            author = "Author Name"
            entry_point = "libmy_plugin"
        "#;

        let result = PluginManifest::from_str(content);
        assert!(result.is_ok(), "Prerelease version should be valid");

        // Valid semver with build metadata
        let content = r#"
            [plugin]
            id = "my-plugin"
            name = "My Plugin"
            version = "1.0.0+build.123"
            description = "A sample plugin"
            author = "Author Name"
            entry_point = "libmy_plugin"
        "#;

        let result = PluginManifest::from_str(content);
        assert!(result.is_ok(), "Build metadata version should be valid");
    }

    #[test]
    fn test_capabilities_defaults() {
        let content = r#"
            [plugin]
            id = "my-plugin"
            name = "My Plugin"
            version = "0.1.0"
            description = "A sample plugin"
            author = "Author Name"
            entry_point = "libmy_plugin"

            [capabilities]
        "#;

        let result = PluginManifest::from_str(content);
        assert!(result.is_ok());

        let manifest = result.unwrap();
        let caps = manifest.capabilities.unwrap();
        assert_eq!(caps.channels, None);
        assert_eq!(caps.tools, None);
        assert_eq!(caps.providers, None);
        assert_eq!(caps.commands, None);
        assert_eq!(caps.hooks, None);
    }

    #[test]
    fn test_debug_output() {
        let content = r#"
            [plugin]
            id = "my-plugin"
            name = "My Plugin"
            version = "0.1.0"
            description = "A sample plugin"
            author = "Author Name"
            entry_point = "libmy_plugin"
        "#;

        let manifest = PluginManifest::from_str(content).unwrap();
        let debug_str = format!("{:?}", manifest);
        assert!(debug_str.contains("PluginManifest"));
        assert!(debug_str.contains("my-plugin"));
    }

    #[test]
    fn test_clone() {
        let content = r#"
            [plugin]
            id = "my-plugin"
            name = "My Plugin"
            version = "0.1.0"
            description = "A sample plugin"
            author = "Author Name"
            entry_point = "libmy_plugin"

            [capabilities]
            channels = ["text"]
        "#;

        let manifest = PluginManifest::from_str(content).unwrap();
        let cloned = manifest.clone();
        
        assert_eq!(manifest.plugin.id, cloned.plugin.id);
        assert_eq!(manifest.capabilities, cloned.capabilities);
    }
}
