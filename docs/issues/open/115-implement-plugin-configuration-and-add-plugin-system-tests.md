# Issue 115: Implement Plugin Configuration and Add Plugin System Tests

## Summary
Allow plugins to define their own configuration schemas and store plugin config within the main aisopod configuration under a `[plugins]` section. Validate plugin configuration at load time, support hot reload for config changes, and add comprehensive unit tests covering registration, hook execution, manifest parsing, and security.

## Location
- Crate: `aisopod-plugin`
- File: `crates/aisopod-plugin/src/config.rs`, `crates/aisopod-plugin/tests/integration.rs`

## Current Behavior
Plugins receive a `PluginContext` (Issue 107) with a generic `serde_json::Value` config but there is no structured mechanism for plugins to declare their config schema, validate it at load time, or react to config changes.

## Expected Behavior
Plugins can define a configuration schema as a `serde_json::Value` JSON Schema. Plugin configuration lives in the main aisopod config under `plugins.<plugin-id>`. Configuration is validated when plugins are loaded. When the main config is reloaded, affected plugins are notified of config changes via a callback. Comprehensive tests verify the entire plugin system.

## Impact
This issue completes the plugin system by enabling plugin-specific configuration and ensuring quality through testing. Without configuration support, plugins cannot be customized by users. Without tests, the plugin system cannot be trusted for production use.

## Suggested Implementation
1. **Define plugin config types in `config.rs`:**
   ```rust
   use serde::{Deserialize, Serialize};
   use serde_json::Value;

   /// Schema definition for a plugin's configuration.
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct PluginConfigSchema {
       pub plugin_id: String,
       pub schema: Value,
       pub defaults: Option<Value>,
   }

   /// Resolved configuration for a single plugin.
   #[derive(Debug, Clone)]
   pub struct PluginConfig {
       pub plugin_id: String,
       pub values: Value,
   }
   ```
2. **Implement config extraction from main config:**
   ```rust
   impl PluginConfig {
       /// Extract plugin config from the main aisopod configuration.
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
   }
   ```
3. **Implement config validation:**
   ```rust
   impl PluginConfigSchema {
       pub fn validate(&self, config: &Value) -> Result<(), ConfigError> {
           // Basic validation: check required fields exist
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
           Ok(())
       }
   }
   ```
4. **Implement hot reload notification:**
   ```rust
   use async_trait::async_trait;

   #[async_trait]
   pub trait ConfigReloadable: Send + Sync {
       /// Called when the plugin's configuration has changed.
       async fn on_config_reload(
           &self,
           new_config: &PluginConfig,
       ) -> Result<(), Box<dyn std::error::Error>>;
   }
   ```
5. **Add integration tests in `tests/integration.rs`:**
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_plugin_registration() {
           let mut registry = PluginRegistry::new();
           let plugin = Arc::new(MockPlugin::new("test-plugin"));
           assert!(registry.register(plugin).is_ok());
       }

       #[test]
       fn test_duplicate_registration() {
           let mut registry = PluginRegistry::new();
           let p1 = Arc::new(MockPlugin::new("test-plugin"));
           let p2 = Arc::new(MockPlugin::new("test-plugin"));
           assert!(registry.register(p1).is_ok());
           assert!(registry.register(p2).is_err());
       }

       #[tokio::test]
       async fn test_hook_dispatch() {
           let mut hook_registry = HookRegistry::new();
           let handler = Arc::new(MockHookHandler::new());
           hook_registry.register(Hook::BeforeAgentRun, "test".into(), handler.clone());
           let ctx = HookContext {
               hook: Hook::BeforeAgentRun,
               data: HashMap::new(),
           };
           hook_registry.dispatch(&ctx).await;
           assert!(handler.was_called());
       }

       #[test]
       fn test_manifest_parsing() {
           let toml = r#"
               [plugin]
               id = "test"
               name = "Test Plugin"
               version = "0.1.0"
               description = "A test plugin"
               author = "Test Author"
               entry_point = "libtest"
           "#;
           let manifest = PluginManifest::from_str(toml).unwrap();
           assert_eq!(manifest.plugin.id, "test");
       }

       #[test]
       fn test_invalid_manifest_missing_id() {
           let toml = r#"
               [plugin]
               name = "Test Plugin"
               version = "0.1.0"
           "#;
           assert!(PluginManifest::from_str(toml).is_err());
       }

       #[test]
       fn test_reserved_command_rejection() {
           let registry = CommandRegistry::new();
           let cmd = PluginCommand {
               name: "help".to_string(),
               description: "Override help".to_string(),
               usage: "help".to_string(),
               require_auth: false,
               handler: Box::new(|_| Ok(())),
           };
           assert!(registry.register(cmd).is_err());
       }

       #[test]
       fn test_argument_sanitization() {
           let sanitized = sanitize_argument("hello\x00world").unwrap();
           assert_eq!(sanitized, "helloworld");
       }

       #[test]
       fn test_argument_size_limit() {
           let large_arg = "a".repeat(5000);
           assert!(sanitize_argument(&large_arg).is_err());
       }

       #[test]
       fn test_plugin_config_extraction() {
           let main = serde_json::json!({
               "plugins": {
                   "my-plugin": {
                       "api_key": "secret",
                       "enabled": true
                   }
               }
           });
           let config = PluginConfig::from_main_config("my-plugin", &main);
           assert_eq!(config.values["api_key"], "secret");
       }

       #[test]
       fn test_plugin_config_missing_defaults_to_empty() {
           let main = serde_json::json!({});
           let config = PluginConfig::from_main_config("nonexistent", &main);
           assert!(config.values.is_object());
       }
   }
   ```

## Dependencies
- Issue 107 (Plugin trait and PluginMeta types)
- Issue 108 (PluginApi for capability registration)
- Issue 109 (plugin manifest format and parser)
- Issue 110 (PluginRegistry lifecycle management)
- Issue 111 (compiled-in plugin loading)
- Issue 112 (dynamic shared library plugin loading)
- Issue 113 (hook system for lifecycle events)
- Issue 114 (plugin CLI command registration with security hardening)
- Issue 016 (core configuration types)

## Acceptance Criteria
- [ ] Plugins can define a configuration schema as `serde_json::Value`
- [ ] Plugin config is extracted from main aisopod config under `plugins.<plugin-id>`
- [ ] Missing plugin config defaults to an empty object
- [ ] Config validation checks required fields at plugin load time
- [ ] `ConfigReloadable` trait notifies plugins of config changes
- [ ] Unit test: plugin registration succeeds and duplicate is rejected
- [ ] Unit test: hook dispatch calls all registered handlers
- [ ] Unit test: valid manifest parses correctly
- [ ] Unit test: invalid manifest produces clear error
- [ ] Unit test: reserved command names are rejected
- [ ] Unit test: argument sanitization removes control characters
- [ ] Unit test: oversized arguments are rejected
- [ ] Unit test: plugin config extraction works correctly
- [ ] All tests pass with `cargo test -p aisopod-plugin`

---
*Created: 2026-02-15*
