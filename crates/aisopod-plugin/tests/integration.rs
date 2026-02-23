//! Integration tests for the aisopod plugin system.
//!
//! This module provides comprehensive tests covering:
//! - Plugin registration and lifecycle
//! - Hook execution and dispatch
//! - Plugin manifest parsing
//! - Security features (command registration, argument sanitization)
//! - Plugin configuration extraction and validation

use aisopod_plugin::config::{ConfigError, ConfigReloadable, PluginConfig, PluginConfigSchema};
use aisopod_plugin::hook::{Hook, HookContext, HookHandler, HookRegistry};
use aisopod_plugin::manifest::PluginManifest;
use aisopod_plugin::security::{sanitize_argument, validate_command_name, MAX_ARG_SIZE, RESERVED_COMMANDS};
use aisopod_plugin::{CommandRegistry, PluginCommand, PluginRegistry};
use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;
use std::collections::HashMap;

// Mock plugin for testing
#[derive(Debug)]
struct MockPlugin {
    id: String,
    init_called: Arc<std::sync::Mutex<bool>>,
    shutdown_called: Arc<std::sync::Mutex<bool>>,
}

impl MockPlugin {
    fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            init_called: Arc::new(std::sync::Mutex::new(false)),
            shutdown_called: Arc::new(std::sync::Mutex::new(false)),
        }
    }
}

#[async_trait]
impl aisopod_plugin::Plugin for MockPlugin {
    fn id(&self) -> &str {
        &self.id
    }

    fn meta(&self) -> &aisopod_plugin::PluginMeta {
        static META: std::sync::OnceLock<aisopod_plugin::PluginMeta> = std::sync::OnceLock::new();
        META.get_or_init(|| {
            aisopod_plugin::PluginMeta::new(
                "test-plugin",
                "1.0.0",
                "A test plugin",
                "Test Author",
                vec![],
                vec![],
            )
        })
    }

    fn register(&self, _api: &mut aisopod_plugin::PluginApi) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    async fn init(&self, _ctx: &aisopod_plugin::PluginContext) -> Result<(), Box<dyn std::error::Error>> {
        *self.init_called.lock().unwrap() = true;
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        *self.shutdown_called.lock().unwrap() = true;
        Ok(())
    }
}

// Mock hook handler for testing
#[derive(Debug, Clone)]
struct MockHookHandler {
    called: Arc<std::sync::Mutex<bool>>,
}

impl MockHookHandler {
    fn new() -> Self {
        Self {
            called: Arc::new(std::sync::Mutex::new(false)),
        }
    }

    fn was_called(&self) -> bool {
        *self.called.lock().unwrap()
    }
}

#[async_trait]
impl HookHandler for MockHookHandler {
    async fn handle(&self, _ctx: &HookContext) -> Result<(), Box<dyn std::error::Error>> {
        *self.called.lock().unwrap() = true;
        Ok(())
    }
}

// Mock plugin with config reload support
#[derive(Debug)]
struct ConfigReloadablePlugin {
    id: String,
    last_config: Arc<std::sync::Mutex<Option<PluginConfig>>>,
}

impl ConfigReloadablePlugin {
    fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            last_config: Arc::new(std::sync::Mutex::new(None)),
        }
    }

    fn get_last_config(&self) -> Option<PluginConfig> {
        self.last_config.lock().unwrap().clone()
    }
}

#[async_trait]
impl aisopod_plugin::Plugin for ConfigReloadablePlugin {
    fn id(&self) -> &str {
        &self.id
    }

    fn meta(&self) -> &aisopod_plugin::PluginMeta {
        static META: std::sync::OnceLock<aisopod_plugin::PluginMeta> = std::sync::OnceLock::new();
        META.get_or_init(|| {
            aisopod_plugin::PluginMeta::new(
                "test-plugin",
                "1.0.0",
                "A config reloadable plugin",
                "Test Author",
                vec![],
                vec![],
            )
        })
    }

    fn register(&self, _api: &mut aisopod_plugin::PluginApi) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    async fn init(&self, _ctx: &aisopod_plugin::PluginContext) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

#[async_trait]
impl ConfigReloadable for ConfigReloadablePlugin {
    async fn on_config_reload(&self, new_config: &PluginConfig) -> Result<(), Box<dyn std::error::Error>> {
        *self.last_config.lock().unwrap() = Some(new_config.clone());
        Ok(())
    }
}

// Test: Plugin registration succeeds
#[test]
fn test_plugin_registration() {
    let mut registry = PluginRegistry::new();
    let plugin = Arc::new(MockPlugin::new("test-plugin"));
    
    assert!(registry.register(plugin).is_ok());
    // Check that plugin was registered via get
    assert!(registry.get("test-plugin").is_some());
}

// Test: Duplicate plugin registration is rejected
#[test]
fn test_duplicate_registration() {
    let mut registry = PluginRegistry::new();
    let p1 = Arc::new(MockPlugin::new("test-plugin"));
    let p2 = Arc::new(MockPlugin::new("test-plugin"));
    
    assert!(registry.register(p1).is_ok());
    match registry.register(p2) {
        Err(aisopod_plugin::RegistryError::DuplicatePlugin(id)) => assert_eq!(id, "test-plugin"),
        _ => panic!("Expected DuplicatePlugin error"),
    }
    // Verify only one plugin is registered
    assert!(registry.get("test-plugin").is_some());
    assert_eq!(registry.list().len(), 1);
}

// Test: Hook dispatch calls all registered handlers
#[tokio::test]
async fn test_hook_dispatch() {
    let mut registry = HookRegistry::new();
    
    let handler = Arc::new(MockHookHandler::new());
    registry.register(Hook::BeforeAgentRun, "test".into(), handler.clone());
    
    let ctx = HookContext::new(Hook::BeforeAgentRun);
    registry.dispatch(&ctx).await;
    
    assert!(handler.was_called());
}

// Test: Hook dispatch with multiple handlers
#[tokio::test]
async fn test_hook_dispatch_multiple() {
    let mut registry = HookRegistry::new();
    
    let handler1 = Arc::new(MockHookHandler::new());
    let handler2 = Arc::new(MockHookHandler::new());
    
    registry.register(Hook::AfterAgentRun, "plugin-1".into(), handler1.clone());
    registry.register(Hook::AfterAgentRun, "plugin-2".into(), handler2.clone());
    
    let ctx = HookContext::new(Hook::AfterAgentRun);
    registry.dispatch(&ctx).await;
    
    assert!(handler1.was_called());
    assert!(handler2.was_called());
}

// Test: Hook dispatch with error handling
#[tokio::test]
async fn test_hook_dispatch_error_handling() {
    let mut registry = HookRegistry::new();
    
    #[derive(Clone)]
    struct FailingHandler;
    
    #[async_trait]
    impl HookHandler for FailingHandler {
        async fn handle(&self, _ctx: &HookContext) -> Result<(), Box<dyn std::error::Error>> {
            Err("Intentional failure".into())
        }
    }
    
    let handler = Arc::new(FailingHandler);
    registry.register(Hook::OnSessionCreate, "failing".into(), handler);
    
    let ctx = HookContext::new(Hook::OnSessionCreate);
    // Should not panic even with failing handler
    registry.dispatch(&ctx).await;
}

// Test: Valid manifest parses correctly
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
    assert_eq!(manifest.plugin.name, "Test Plugin");
    assert_eq!(manifest.plugin.version, "0.1.0");
    assert_eq!(manifest.plugin.description, "A test plugin");
    assert_eq!(manifest.plugin.author, "Test Author");
    assert_eq!(manifest.plugin.entry_point, "libtest");
}

// Test: Invalid manifest produces clear error
#[test]
fn test_invalid_manifest_missing_id() {
    let toml = r#"
        [plugin]
        name = "Test Plugin"
        version = "0.1.0"
        description = "A test plugin"
        author = "Test Author"
        entry_point = "libtest"
    "#;
    
    let result = PluginManifest::from_str(toml);
    assert!(result.is_err());
}

// Test: Reserved command names are rejected
#[test]
fn test_reserved_command_rejection() {
    let reserved_commands = ["help", "status", "config", "plugin", "version"];
    
    for cmd in reserved_commands {
        let command = PluginCommand::new(
            cmd,
            "Test command",
            cmd,
            false,
            Arc::new(|_| Ok(())),
        );
        
        let registry = CommandRegistry::new();
        let result = registry.register(command);
        
        match result {
            Err(aisopod_plugin::SecurityError::ReservedCommandName(name)) => {
                assert_eq!(name, cmd);
            }
            _ => panic!("Expected ReservedCommandName error for '{}'", cmd),
        }
    }
}

// Test: Argument sanitization removes control characters
#[test]
fn test_argument_sanitization() {
    // Null bytes should be removed
    assert_eq!(sanitize_argument("hello\x00world").unwrap(), "helloworld");
    
    // Other control characters should be removed
    assert_eq!(sanitize_argument("line1\x0Bline2").unwrap(), "line1line2");
    
    // Newline and tab should be preserved
    assert_eq!(sanitize_argument("line1\nline2").unwrap(), "line1\nline2");
    assert_eq!(sanitize_argument("col1\tcol2").unwrap(), "col1\tcol2");
}

// Test: Oversized arguments are rejected
#[test]
fn test_argument_size_limit() {
    // Arguments exceeding MAX_ARG_SIZE should fail
    let large_arg = "a".repeat(MAX_ARG_SIZE + 1);
    let result = sanitize_argument(&large_arg);
    assert!(result.is_err());
    
    // Exactly MAX_ARG_SIZE should succeed
    let exact_arg = "a".repeat(MAX_ARG_SIZE);
    assert!(sanitize_argument(&exact_arg).is_ok());
}

// Test: Plugin config extraction works correctly
#[test]
fn test_plugin_config_extraction() {
    let main = json!({
        "plugins": {
            "my-plugin": {
                "api_key": "secret",
                "enabled": true
            }
        }
    });
    
    let config = PluginConfig::from_main_config("my-plugin", &main);
    
    assert_eq!(config.plugin_id(), "my-plugin");
    assert_eq!(config.get("api_key"), Some(&json!("secret")));
    assert!(config.get("enabled").unwrap().as_bool().unwrap());
}

// Test: Missing plugin config defaults to empty object
#[test]
fn test_plugin_config_missing_defaults_to_empty() {
    let main = json!({});
    let config = PluginConfig::from_main_config("nonexistent", &main);
    
    assert!(config.values().is_object());
    assert_eq!(config.plugin_id(), "nonexistent");
}

// Test: Plugin config with defaults
#[test]
fn test_plugin_config_with_defaults() {
    let schema = PluginConfigSchema::new(
        "test-plugin",
        json!({
            "type": "object",
            "properties": {
                "key1": { "type": "string" },
                "key2": { "type": "number" }
            }
        }),
        Some(json!({
            "key1": "default_value",
            "key2": 100
        })),
    );
    
    let config = json!({
        "key1": "custom_value"
    });
    
    let merged = schema.merge_defaults(&config);
    
    assert_eq!(merged["key1"], "custom_value");
    assert_eq!(merged["key2"], 100);
}

// Test: Config validation with required fields
#[test]
fn test_config_validation_required_fields() {
    let schema = PluginConfigSchema::new(
        "test-plugin",
        json!({
            "type": "object",
            "required": ["api_key", "endpoint"],
            "properties": {
                "api_key": { "type": "string" },
                "endpoint": { "type": "string" }
            }
        }),
        None,
    );
    
    // Valid config
    let valid_config = json!({
        "api_key": "secret",
        "endpoint": "https://api.example.com"
    });
    assert!(schema.validate(&valid_config).is_ok());
    
    // Missing required field
    let invalid_config = json!({
        "api_key": "secret"
    });
    match schema.validate(&invalid_config) {
        Err(aisopod_plugin::ConfigError::MissingRequiredField { plugin_id, field }) => {
            assert_eq!(plugin_id, "test-plugin");
            assert_eq!(field, "endpoint");
        }
        _ => panic!("Expected MissingRequiredField error"),
    }
}

// Test: Config validation with type checking
#[test]
fn test_config_validation_type_checking() {
    let schema = PluginConfigSchema::new(
        "test-plugin",
        json!({
            "type": "object",
            "properties": {
                "port": { "type": "number" },
                "host": { "type": "string" }
            }
        }),
        None,
    );
    
    // Valid types
    let valid_config = json!({
        "port": 8080,
        "host": "localhost"
    });
    assert!(schema.validate(&valid_config).is_ok());
    
    // Invalid types
    let invalid_config = json!({
        "port": "8080",
        "host": 123
    });
    match schema.validate(&invalid_config) {
        Err(aisopod_plugin::ConfigError::InvalidValue { .. }) => {
            // Expected error
        }
        _ => panic!("Expected InvalidValue error"),
    }
}

// Test: ConfigReloadable trait implementation
#[tokio::test]
async fn test_config_reloadable() {
    let plugin = Arc::new(ConfigReloadablePlugin::new("reloadable-plugin"));
    
    let new_config = PluginConfig::new(
        "reloadable-plugin",
        json!({
            "api_key": "new_secret",
            "enabled": false
        })
    );
    
    // Call on_config_reload
    plugin.on_config_reload(&new_config).await.unwrap();
    
    // Verify config was updated
    let last_config = plugin.get_last_config();
    assert!(last_config.is_some());
    assert_eq!(last_config.unwrap().values()["api_key"], "new_secret");
}

// Test: Plugin registry lifecycle
#[tokio::test]
async fn test_plugin_registry_lifecycle() {
    let mut registry = PluginRegistry::new();
    
    let plugin = Arc::new(MockPlugin::new("lifecycle-test"));
    
    // Register
    assert!(registry.register(plugin).is_ok());
    
    // Create context
    let ctx = aisopod_plugin::PluginContext::new(
        Arc::new(json!({})),
        std::path::PathBuf::new(),
    );
    
    // Initialize
    assert!(registry.init_all(&ctx).await.is_ok());
    
    // Shutdown
    assert!(registry.shutdown_all().await.is_ok());
}

// Test: Command registration with security
#[test]
fn test_command_registration_security() {
    let mut registry = CommandRegistry::new();
    
    // Valid command
    let command = PluginCommand::new(
        "mycommand",
        "My command description",
        "mycommand [args]",
        false,
        Arc::new(|_| Ok(())),
    );
    assert!(registry.register(command).is_ok());
    
    // Command with reserved name
    let reserved_command = PluginCommand::new(
        "help",
        "Should fail",
        "help",
        false,
        Arc::new(|_| Ok(())),
    );
    match registry.register(reserved_command) {
        Err(aisopod_plugin::SecurityError::ReservedCommandName(name)) => {
            assert_eq!(name, "help");
        }
        _ => panic!("Expected ReservedCommandName error"),
    }
}

// Test: Multiple plugins with hook registration
#[tokio::test]
async fn test_multiple_plugins_with_hooks() {
    let mut registry = HookRegistry::new();
    
    let handler1 = Arc::new(MockHookHandler::new());
    let handler2 = Arc::new(MockHookHandler::new());
    
    registry.register(Hook::BeforeAgentRun, "plugin-1".into(), handler1.clone());
    registry.register(Hook::BeforeAgentRun, "plugin-2".into(), handler2.clone());
    
    let ctx = HookContext::new(Hook::BeforeAgentRun);
    registry.dispatch(&ctx).await;
    
    assert!(handler1.was_called());
    assert!(handler2.was_called());
    assert_eq!(registry.handler_count(&Hook::BeforeAgentRun), 2);
}

// Test: Hook registry total count
#[test]
fn test_hook_registry_total_count() {
    let mut registry = HookRegistry::new();
    
    let handler = Arc::new(MockHookHandler::new());
    
    registry.register(Hook::BeforeAgentRun, "plugin-1".into(), handler.clone());
    registry.register(Hook::AfterAgentRun, "plugin-2".into(), handler.clone());
    registry.register(Hook::OnSessionCreate, "plugin-3".into(), handler.clone());
    
    assert_eq!(registry.total_hook_count(), 3);
    assert_eq!(registry.hook_type_count(), 3);
}

// Test: Reserved commands list is not empty
#[test]
fn test_reserved_commands_not_empty() {
    assert!(!RESERVED_COMMANDS.is_empty());
    assert!(RESERVED_COMMANDS.len() >= 70);
}

// Test: Command name validation
#[test]
fn test_command_name_validation() {
    // Valid names
    assert!(validate_command_name("mystatus").is_ok());
    assert!(validate_command_name("my-plugin").is_ok());
    assert!(validate_command_name("test_command_123").is_ok());
    
    // Reserved names rejected
    assert!(validate_command_name("help").is_err());
    assert!(validate_command_name("HELP").is_err());
    
    // Invalid formats
    assert!(validate_command_name("").is_err());
    assert!(validate_command_name(&"a".repeat(65)).is_err());
    assert!(validate_command_name("bad!char").is_err());
}
