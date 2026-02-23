//! Plugin configuration types for aisopod plugin system.
//!
//! This module provides the types and traits for managing plugin configuration:
//!
//! - [`PluginConfigSchema`] - Schema definition for plugin configuration
//! - [`PluginConfig`] - Resolved configuration for a single plugin
//! - [`ConfigReloadable`] - Trait for plugins that need to react to config changes
//!
//! # Configuration Schema
//!
//! Plugin configurations can be validated using JSON Schema format. The schema
//! defines required fields and their types.
//!
//! ```ignore
//! use serde_json::json;
//!
//! let schema = PluginConfigSchema::new(
//!     "my-plugin",
//!     json!({
//!         "type": "object",
//!         "required": ["api_key", "endpoint"],
//!         "properties": {
//!             "api_key": { "type": "string" },
//!             "endpoint": { "type": "string" }
//!         }
//!     })
//! );
//!
//! let config = json!({
//!     "api_key": "secret",
//!     "endpoint": "https://api.example.com"
//! });
//!
//! schema.validate(&config)?;
//! ```
//!
//! # Hot Reload
//!
//! Plugins that implement [`ConfigReloadable`] will be notified when their
//! configuration changes during a hot reload of the main configuration.
//!
//! ```ignore
//! use aisopod_plugin::{Plugin, PluginContext, PluginApi, ConfigReloadable, PluginConfig};
//! use async_trait::async_trait;
//!
//! struct MyPlugin {
//!     config: PluginConfig,
//! }
//!
//! #[async_trait]
//! impl ConfigReloadable for MyPlugin {
//!     async fn on_config_reload(&self, new_config: &PluginConfig) -> Result<(), Box<dyn std::error::Error>> {
//!         self.config = new_config.clone();
//!         println!("Config reloaded: {:?}", self.config.values);
//!         Ok(())
//!     }
//! }
//! ```

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

/// Error types for plugin configuration.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Missing required field in configuration.
    #[error("Plugin '{plugin_id}' configuration missing required field '{field}'")]
    MissingRequiredField {
        /// The plugin ID
        plugin_id: String,
        /// The missing field name
        field: String,
    },

    /// Invalid configuration value.
    #[error("Plugin '{plugin_id}' configuration has invalid value for field '{field}': {message}")]
    InvalidValue {
        /// The plugin ID
        plugin_id: String,
        /// The field name
        field: String,
        /// Error message
        message: String,
    },

    /// Schema parsing error.
    #[error("Plugin '{plugin_id}' schema error: {message}")]
    SchemaError {
        /// The plugin ID
        plugin_id: String,
        /// Error message
        message: String,
    },
}

/// Schema definition for a plugin's configuration.
///
/// This struct represents the configuration schema that a plugin expects.
/// The schema uses JSON Schema format to define the structure, required fields,
/// and types of the plugin's configuration.
///
/// # Fields
///
/// * `plugin_id` - Unique identifier for the plugin this schema applies to
/// * `schema` - JSON Schema definition for the configuration
/// * `defaults` - Optional default values for configuration fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfigSchema {
    /// Unique identifier for the plugin this schema applies to.
    pub plugin_id: String,
    /// JSON Schema definition for the configuration.
    #[serde(default)]
    pub schema: Value,
    /// Optional default values for configuration fields.
    #[serde(default)]
    pub defaults: Option<Value>,
}

impl PluginConfigSchema {
    /// Creates a new `PluginConfigSchema` instance.
    ///
    /// # Arguments
    ///
    /// * `plugin_id` - The unique identifier for the plugin
    /// * `schema` - JSON Schema definition for the configuration
    /// * `defaults` - Optional default values
    pub fn new(plugin_id: impl Into<String>, schema: Value, defaults: Option<Value>) -> Self {
        Self {
            plugin_id: plugin_id.into(),
            schema,
            defaults,
        }
    }

    /// Validates the configuration against the schema.
    ///
    /// This method checks that all required fields defined in the schema
    /// are present in the configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration to validate
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Configuration is valid
    /// * `Err(ConfigError)` - Validation failed with details
    pub fn validate(&self, config: &Value) -> Result<(), ConfigError> {
        // Check for required fields if specified in schema
        if let Some(required) = self.schema.get("required") {
            if let Some(required_fields) = required.as_array() {
                for field in required_fields {
                    if let Some(field_name) = field.as_str() {
                        if config.get(field_name).is_none() {
                            return Err(ConfigError::MissingRequiredField {
                                plugin_id: self.plugin_id.clone(),
                                field: field_name.to_string(),
                            });
                        }
                    }
                }
            }
        }

        // Additional validation for field types if schema defines them
        if let Some(properties) = self.schema.get("properties") {
            if let Some(props_obj) = properties.as_object() {
                for (field_name, field_schema) in props_obj {
                    if let Some(field_value) = config.get(field_name) {
                        self.validate_field_type(field_name, field_value, field_schema)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Validates a single field value against its schema definition.
    fn validate_field_type(
        &self,
        field_name: &str,
        value: &Value,
        field_schema: &Value,
    ) -> Result<(), ConfigError> {
        // Check type if specified
        if let Some(expected_type) = field_schema.get("type") {
            if let Some(expected_type_str) = expected_type.as_str() {
                let actual_type = match value {
                    Value::Null => "null",
                    Value::Bool(_) => "boolean",
                    Value::Number(_) => "number",
                    Value::String(_) => "string",
                    Value::Array(_) => "array",
                    Value::Object(_) => "object",
                };

                if expected_type_str != actual_type {
                    return Err(ConfigError::InvalidValue {
                        plugin_id: self.plugin_id.clone(),
                        field: field_name.to_string(),
                        message: format!("expected {}, got {}", expected_type_str, actual_type),
                    });
                }
            }
        }

        Ok(())
    }

    /// Merges the configuration with default values.
    ///
    /// This method applies default values from the schema to the configuration
    /// if those fields are missing.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration to merge with defaults
    ///
    /// # Returns
    ///
    /// A new `Value` with defaults applied
    pub fn merge_defaults(&self, config: &Value) -> Value {
        let mut config = config.clone();

        if let Some(defaults) = &self.defaults {
            if let (Value::Object(config_obj), Value::Object(defaults_obj)) =
                (&mut config, defaults)
            {
                for (key, default_value) in defaults_obj {
                    if !config_obj.contains_key(key) {
                        config_obj.insert(key.clone(), default_value.clone());
                    }
                }
            }
        }

        config
    }
}

/// Resolved configuration for a single plugin.
///
/// This struct contains the configuration values for a plugin after
/// extraction from the main aisopod configuration.
///
/// # Fields
///
/// * `plugin_id` - Unique identifier for the plugin
/// * `values` - Configuration values as a JSON value
#[derive(Debug, Clone)]
pub struct PluginConfig {
    /// Unique identifier for the plugin.
    pub plugin_id: String,
    /// Configuration values as a JSON value.
    pub values: Value,
}

impl PluginConfig {
    /// Creates a new `PluginConfig` instance.
    ///
    /// # Arguments
    ///
    /// * `plugin_id` - The unique identifier for the plugin
    /// * `values` - Configuration values as a JSON value
    pub fn new(plugin_id: impl Into<String>, values: Value) -> Self {
        Self {
            plugin_id: plugin_id.into(),
            values,
        }
    }

    /// Extracts plugin config from the main aisopod configuration.
    ///
    /// This method looks up the plugin's configuration in the main
    /// aisopod configuration under the `plugins.<plugin-id>` section.
    ///
    /// # Arguments
    ///
    /// * `plugin_id` - The unique identifier for the plugin
    /// * `main_config` - The main aisopod configuration as a JSON value
    ///
    /// # Returns
    ///
    /// A new `PluginConfig` with the extracted values. If no configuration
    /// is found for the plugin, an empty object is used as defaults.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use aisopod_plugin::config::PluginConfig;
    /// use serde_json::json;
    ///
    /// let main_config = json!({
    ///     "plugins": {
    ///         "my-plugin": {
    ///             "api_key": "secret",
    ///             "enabled": true
    ///         }
    ///     }
    /// });
    ///
    /// let config = PluginConfig::from_main_config("my-plugin", &main_config);
    /// assert_eq!(config.values["api_key"], "secret");
    /// ```
    pub fn from_main_config(plugin_id: &str, main_config: &Value) -> Self {
        let values = main_config
            .get("plugins")
            .and_then(|p| p.get(plugin_id))
            .cloned()
            .unwrap_or(Value::Object(serde_json::Map::new()));

        Self {
            plugin_id: plugin_id.to_string(),
            values,
        }
    }

    /// Gets a configuration value by key.
    ///
    /// # Arguments
    ///
    /// * `key` - The configuration key to look up
    ///
    /// # Returns
    ///
    /// `Some(&Value)` if the key exists, `None` otherwise
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.values.get(key)
    }

    /// Checks if a configuration key exists.
    ///
    /// # Arguments
    ///
    /// * `key` - The configuration key to check
    ///
    /// # Returns
    ///
    /// `true` if the key exists, `false` otherwise
    pub fn contains_key(&self, key: &str) -> bool {
        self.values.get(key).is_some()
    }

    /// Returns the configuration as an immutable reference.
    pub fn values(&self) -> &Value {
        &self.values
    }

    /// Returns the plugin ID.
    pub fn plugin_id(&self) -> &str {
        &self.plugin_id
    }
}

/// Trait for plugins that need to react to configuration changes.
///
/// Plugins that implement this trait will be notified when their
/// configuration is reloaded (e.g., during hot reload of the main config).
///
/// # Example
///
/// ```ignore
/// use aisopod_plugin::{Plugin, ConfigReloadable, PluginConfig};
/// use async_trait::async_trait;
///
/// struct MyPlugin {
///     config: PluginConfig,
/// }
///
/// #[async_trait]
/// impl ConfigReloadable for MyPlugin {
///     async fn on_config_reload(&self, new_config: &PluginConfig) -> Result<(), Box<dyn std::error::Error>> {
///         // Update internal state with new config
///         println!("Config changed for plugin: {}", new_config.plugin_id);
///         Ok(())
///     }
/// }
/// ```
#[async_trait::async_trait]
pub trait ConfigReloadable: Send + Sync {
    /// Called when the plugin's configuration has changed.
    ///
    /// # Arguments
    ///
    /// * `new_config` - The new configuration values
    ///
    /// # Errors
    ///
    /// Return an error if the configuration change cannot be applied.
    /// The plugin system will log the error but continue operation.
    async fn on_config_reload(
        &self,
        new_config: &PluginConfig,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_config_schema_new() {
        let schema = PluginConfigSchema::new(
            "test-plugin",
            serde_json::json!({
                "type": "object",
                "required": ["key"],
                "properties": {
                    "key": { "type": "string" }
                }
            }),
            None,
        );

        assert_eq!(schema.plugin_id, "test-plugin");
        assert_eq!(schema.schema["type"], "object");
        assert!(schema.defaults.is_none());
    }

    #[test]
    fn test_plugin_config_schema_validate_success() {
        let schema = PluginConfigSchema::new(
            "test-plugin",
            serde_json::json!({
                "type": "object",
                "required": ["key1", "key2"],
                "properties": {
                    "key1": { "type": "string" },
                    "key2": { "type": "number" }
                }
            }),
            None,
        );

        let config = serde_json::json!({
            "key1": "value",
            "key2": 42
        });

        assert!(schema.validate(&config).is_ok());
    }

    #[test]
    fn test_plugin_config_schema_validate_missing_field() {
        let schema = PluginConfigSchema::new(
            "test-plugin",
            serde_json::json!({
                "type": "object",
                "required": ["key1", "key2"],
                "properties": {
                    "key1": { "type": "string" }
                }
            }),
            None,
        );

        let config = serde_json::json!({
            "key1": "value"
        });

        let result = schema.validate(&config);
        assert!(result.is_err());

        match result {
            Err(ConfigError::MissingRequiredField { plugin_id, field }) => {
                assert_eq!(plugin_id, "test-plugin");
                assert_eq!(field, "key2");
            }
            _ => panic!("Expected MissingRequiredField error"),
        }
    }

    #[test]
    fn test_plugin_config_schema_validate_type_error() {
        let schema = PluginConfigSchema::new(
            "test-plugin",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "key": { "type": "string" }
                }
            }),
            None,
        );

        let config = serde_json::json!({
            "key": 42
        });

        let result = schema.validate(&config);
        assert!(result.is_err());

        match result {
            Err(ConfigError::InvalidValue { plugin_id, field, message }) => {
                assert_eq!(plugin_id, "test-plugin");
                assert_eq!(field, "key");
                assert!(message.contains("expected string, got number"));
            }
            _ => panic!("Expected InvalidValue error"),
        }
    }

    #[test]
    fn test_plugin_config_schema_merge_defaults() {
        let schema = PluginConfigSchema::new(
            "test-plugin",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "key1": { "type": "string" },
                    "key2": { "type": "number" }
                }
            }),
            Some(serde_json::json!({
                "key1": "default_value",
                "key2": 100
            })),
        );

        let config = serde_json::json!({
            "key1": "custom_value"
        });

        let merged = schema.merge_defaults(&config);

        assert_eq!(merged["key1"], "custom_value");
        assert_eq!(merged["key2"], 100);
    }

    #[test]
    fn test_plugin_config_schema_merge_defaults_missing_fields() {
        let schema = PluginConfigSchema::new(
            "test-plugin",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "key1": { "type": "string" },
                    "key2": { "type": "number" }
                }
            }),
            Some(serde_json::json!({
                "key1": "default_value",
                "key2": 100
            })),
        );

        let config = serde_json::json!({});

        let merged = schema.merge_defaults(&config);

        assert_eq!(merged["key1"], "default_value");
        assert_eq!(merged["key2"], 100);
    }

    #[test]
    fn test_plugin_config_new() {
        let config = PluginConfig::new(
            "test-plugin",
            serde_json::json!({
                "key": "value"
            }),
        );

        assert_eq!(config.plugin_id, "test-plugin");
        assert_eq!(config.values["key"], "value");
    }

    #[test]
    fn test_plugin_config_from_main_config_found() {
        let main_config = serde_json::json!({
            "plugins": {
                "my-plugin": {
                    "api_key": "secret",
                    "enabled": true
                }
            }
        });

        let config = PluginConfig::from_main_config("my-plugin", &main_config);

        assert_eq!(config.plugin_id, "my-plugin");
        assert_eq!(config.values["api_key"], "secret");
        assert!(config.values["enabled"].as_bool().unwrap());
    }

    #[test]
    fn test_plugin_config_from_main_config_not_found() {
        let main_config = serde_json::json!({
            "plugins": {
                "other-plugin": {
                    "key": "value"
                }
            }
        });

        let config = PluginConfig::from_main_config("my-plugin", &main_config);

        assert_eq!(config.plugin_id, "my-plugin");
        assert!(config.values.is_object());
        assert!(config.values.is_null() || config.values.as_object().map_or(true, |o| o.is_empty()));
    }

    #[test]
    fn test_plugin_config_from_main_config_no_plugins_section() {
        let main_config = serde_json::json!({
            "other": "value"
        });

        let config = PluginConfig::from_main_config("my-plugin", &main_config);

        assert_eq!(config.plugin_id, "my-plugin");
        assert!(config.values.is_object());
        assert!(config.values.as_object().map_or(true, |o| o.is_empty()));
    }

    #[test]
    fn test_plugin_config_get() {
        let config = PluginConfig::new(
            "test-plugin",
            serde_json::json!({
                "key1": "value1",
                "key2": 42
            }),
        );

        assert_eq!(config.get("key1"), Some(&serde_json::json!("value1")));
        assert_eq!(config.get("key2"), Some(&serde_json::json!(42)));
        assert_eq!(config.get("nonexistent"), None);
    }

    #[test]
    fn test_plugin_config_contains_key() {
        let config = PluginConfig::new(
            "test-plugin",
            serde_json::json!({
                "key": "value"
            }),
        );

        assert!(config.contains_key("key"));
        assert!(!config.contains_key("nonexistent"));
    }

    #[test]
    fn test_plugin_config_values() {
        let config = PluginConfig::new(
            "test-plugin",
            serde_json::json!({
                "key": "value"
            }),
        );

        assert_eq!(config.values()["key"], "value");
    }

    #[test]
    fn test_plugin_config_plugin_id() {
        let config = PluginConfig::new("test-plugin", serde_json::json!({}));

        assert_eq!(config.plugin_id(), "test-plugin");
    }

    #[test]
    fn test_config_error_debug() {
        let error = ConfigError::MissingRequiredField {
            plugin_id: "test-plugin".to_string(),
            field: "key".to_string(),
        };
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("MissingRequiredField"));
        assert!(debug_str.contains("test-plugin"));
        assert!(debug_str.contains("key"));
    }

    #[test]
    fn test_config_error_display() {
        let error = ConfigError::MissingRequiredField {
            plugin_id: "test-plugin".to_string(),
            field: "key".to_string(),
        };
        assert!(error.to_string().contains("test-plugin"));
        assert!(error.to_string().contains("key"));

        let error = ConfigError::InvalidValue {
            plugin_id: "test-plugin".to_string(),
            field: "key".to_string(),
            message: "expected string".to_string(),
        };
        assert!(error.to_string().contains("invalid value"));
        assert!(error.to_string().contains("key"));
    }
}
