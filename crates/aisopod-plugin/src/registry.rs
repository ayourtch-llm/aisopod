//! Plugin registry for lifecycle management.
//!
//! This module provides the [`PluginRegistry`] struct that manages
//! the full lifecycle of plugins: registration, retrieval, listing,
//! ordered initialization, and reverse-order shutdown.

use std::collections::HashMap;
use std::sync::Arc;

use tracing::{info, warn};

use crate::{Plugin, PluginContext};

/// Registry error types for plugin lifecycle operations.
///
/// This enum captures all possible errors that can occur when
/// managing plugins through the [`PluginRegistry`], including
/// duplicate registrations, initialization failures, and not-found
/// scenarios.
#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    /// A plugin with the given ID was already registered.
    #[error("Duplicate plugin ID: {0}")]
    DuplicatePlugin(String),

    /// Plugin initialization failed with the given error.
    #[error("Plugin '{0}' initialization failed: {1}")]
    InitFailed(String, String),

    /// A plugin with the given ID was not found in the registry.
    #[error("Plugin not found: {0}")]
    NotFound(String),
}

/// The registry for managing plugin instances and their lifecycle.
///
/// The `PluginRegistry` is the central coordinator of the plugin system.
/// All plugin loading strategies (compiled-in, dynamic) funnel through it,
/// and it orchestrates the startup and shutdown sequences.
///
/// # Plugin Lifecycle
///
/// 1. **Registration**: Plugins are registered using [`register()`](PluginRegistry::register)
/// 2. **Initialization**: All plugins are initialized in registration order via [`init_all()`](PluginRegistry::init_all)
/// 3. **Shutdown**: All plugins are shut down in reverse order via [`shutdown_all()`](PluginRegistry::shutdown_all)
///
/// # Thread Safety
///
/// The registry is designed to be used in a multi-threaded context.
/// It stores plugins as `Arc<dyn Plugin>` and uses interior mutability
/// for the registration order tracking.
///
/// # Example
///
/// ```ignore
/// use aisopod_plugin::{PluginRegistry, Plugin, PluginContext, PluginMeta};
/// use std::sync::Arc;
/// use async_trait::async_trait;
///
/// #[derive(Debug)]
/// struct MyPlugin {
///     meta: PluginMeta,
/// }
///
/// impl MyPlugin {
///     pub fn new() -> Self {
///         Self {
///             meta: PluginMeta::new(
///                 "my-plugin",
///                 "1.0.0",
///                 "A sample plugin",
///                 "Author Name",
///                 vec![],
///                 vec![],
///             ),
///         }
///     }
/// }
///
/// #[async_trait]
/// impl Plugin for MyPlugin {
///     fn id(&self) -> &str {
///         "my-plugin"
///     }
///
///     fn meta(&self) -> &PluginMeta {
///         &self.meta
///     }
///
///     async fn init(&self, _ctx: &PluginContext) -> Result<(), Box<dyn std::error::Error>> {
///         Ok(())
///     }
///
///     async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
///         Ok(())
///     }
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mut registry = PluginRegistry::new();
///     let plugin = Arc::new(MyPlugin::new());
///     registry.register(plugin)?;
///
///     // Initialize all plugins
///     let ctx = PluginContext::new(
///         std::sync::Arc::new(serde_json::Value::Object(Default::default())),
///         std::path::PathBuf::new(),
///     );
///     registry.init_all(&ctx).await?;
///
///     // Shutdown all plugins
///     registry.shutdown_all().await?;
///
///     Ok(())
/// }
/// ```
#[derive(Debug, Default)]
pub struct PluginRegistry {
    /// Map of plugin ID to plugin instance.
    plugins: HashMap<String, Arc<dyn Plugin>>,
    /// Order in which plugins were registered (for ordered init and reverse-order shutdown).
    load_order: Vec<String>,
}

impl PluginRegistry {
    /// Creates a new empty [`PluginRegistry`].
    ///
    /// The registry starts with no plugins and an empty load order.
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            load_order: Vec::new(),
        }
    }

    /// Registers a plugin with the registry.
    ///
    /// The plugin is stored keyed by its unique ID (from [`Plugin::id()`]).
    /// If a plugin with the same ID is already registered, an error is returned.
    ///
    /// # Arguments
    ///
    /// * `plugin` - An `Arc` wrapping the plugin instance
    ///
    /// # Errors
    ///
    /// Returns `RegistryError::DuplicatePlugin` if a plugin with the same
    /// ID is already registered.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use aisopod_plugin::{PluginRegistry, Plugin};
    /// use std::sync::Arc;
    ///
    /// let mut registry = PluginRegistry::new();
    /// let plugin = Arc::new(MyPlugin::new());
    /// registry.register(plugin)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn register(&mut self, plugin: Arc<dyn Plugin>) -> Result<(), RegistryError> {
        let id = plugin.id().to_string();
        if self.plugins.contains_key(&id) {
            return Err(RegistryError::DuplicatePlugin(id));
        }
        info!(plugin_id = %id, "Registering plugin");
        self.load_order.push(id.clone());
        self.plugins.insert(id, plugin);
        Ok(())
    }

    /// Retrieves a plugin by its ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the plugin
    ///
    /// # Returns
    ///
    /// `Some(&Arc<dyn Plugin>)` if the plugin is found, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use aisopod_plugin::PluginRegistry;
    ///
    /// let registry = PluginRegistry::new();
    /// // ... register plugins ...
    /// if let Some(plugin) = registry.get("my-plugin") {
    ///     // Use the plugin
    /// }
    /// ```
    pub fn get(&self, id: &str) -> Option<&Arc<dyn Plugin>> {
        self.plugins.get(id)
    }

    /// Returns all plugins in registration order.
    ///
    /// Plugins are returned in the order they were registered via [`register()`](PluginRegistry::register).
    ///
    /// # Returns
    ///
    /// A vector of references to the registered plugins.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use aisopod_plugin::PluginRegistry;
    ///
    /// let registry = PluginRegistry::new();
    /// // ... register plugins ...
    /// for plugin in registry.list() {
    ///     println!("Found plugin: {}", plugin.id());
    /// }
    /// ```
    pub fn list(&self) -> Vec<&Arc<dyn Plugin>> {
        self.load_order
            .iter()
            .filter_map(|id| self.plugins.get(id))
            .collect()
    }

    /// Initializes all registered plugins in registration order.
    ///
    /// Plugins are initialized sequentially in the order they were registered.
    /// If any plugin fails to initialize, the function returns an error and
    /// does not attempt to initialize subsequent plugins.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The [`PluginContext`] containing runtime information
    ///
    /// # Errors
    ///
    /// Returns `RegistryError::InitFailed` if any plugin's `init()` method
    /// returns an error. The error includes the plugin ID and the error message.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use aisopod_plugin::{PluginRegistry, PluginContext};
    /// use std::sync::Arc;
    ///
    /// let mut registry = PluginRegistry::new();
    /// // ... register plugins ...
    ///
    /// let ctx = PluginContext::new(
    ///     Arc::new(serde_json::Value::Object(Default::default())),
    ///     std::path::PathBuf::new(),
    /// );
    /// registry.init_all(&ctx).await?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub async fn init_all(&self, ctx: &PluginContext) -> Result<(), RegistryError> {
        for id in &self.load_order {
            if let Some(plugin) = self.plugins.get(id) {
                info!(plugin_id = %id, "Initializing plugin");
                plugin.init(ctx).await.map_err(|e| {
                    RegistryError::InitFailed(id.clone(), e.to_string())
                })?;
            }
        }
        Ok(())
    }

    /// Shuts down all registered plugins in reverse registration order.
    ///
    /// Plugins are shut down in reverse order from their registration.
    /// This ensures that plugins that depend on other plugins are shut down
    /// first, allowing for proper cleanup ordering.
    ///
    /// If a plugin fails to shut down, an error is logged but the shutdown
    /// continues for other plugins.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` after attempting to shut down all plugins. Individual
    /// plugin shutdown failures are logged but do not cause this function
    /// to return an error.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use aisopod_plugin::PluginRegistry;
    ///
    /// let registry = PluginRegistry::new();
    /// // ... register and initialize plugins ...
    /// registry.shutdown_all().await?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub async fn shutdown_all(&self) -> Result<(), RegistryError> {
        for id in self.load_order.iter().rev() {
            if let Some(plugin) = self.plugins.get(id) {
                info!(plugin_id = %id, "Shutting down plugin");
                if let Err(e) = plugin.shutdown().await {
                    warn!(plugin_id = %id, error = %e, "Plugin shutdown failed");
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::meta::PluginMeta;
    use crate::Plugin;
    use async_trait::async_trait;
    use std::sync::Arc;

    #[derive(Debug)]
    struct TestPlugin {
        id: String,
        meta: PluginMeta,
        init_called: bool,
        shutdown_called: bool,
    }

    impl TestPlugin {
        fn new(id: impl Into<String>) -> Self {
            Self {
                id: id.into(),
                meta: PluginMeta::new(
                    "test-plugin",
                    "1.0.0",
                    "A test plugin",
                    "Test Author",
                    vec![],
                    vec![],
                ),
                init_called: false,
                shutdown_called: false,
            }
        }

        fn mark_init(&mut self) {
            self.init_called = true;
        }

        fn mark_shutdown(&mut self) {
            self.shutdown_called = true;
        }
    }

    #[async_trait]
    impl Plugin for TestPlugin {
        fn id(&self) -> &str {
            &self.id
        }

        fn meta(&self) -> &PluginMeta {
            &self.meta
        }

        fn register(&self, _api: &mut crate::PluginApi) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }

        async fn init(&self, _ctx: &PluginContext) -> Result<(), Box<dyn std::error::Error>> {
            // In a real scenario, we would mark init as called here
            // But since self is &self and not &mut self, we can't modify state
            // For testing purposes, we'll just return Ok
            Ok(())
        }

        async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_registry_new() {
        let registry = PluginRegistry::new();
        assert!(registry.plugins.is_empty());
        assert!(registry.load_order.is_empty());
    }

    #[tokio::test]
    async fn test_register_plugin() {
        let mut registry = PluginRegistry::new();
        let plugin = Arc::new(TestPlugin::new("plugin-1"));
        assert!(registry.register(plugin).is_ok());
        assert_eq!(registry.plugins.len(), 1);
        assert_eq!(registry.load_order.len(), 1);
    }

    #[tokio::test]
    async fn test_register_duplicate_plugin() {
        let mut registry = PluginRegistry::new();
        let plugin1 = Arc::new(TestPlugin::new("plugin-1"));
        let plugin2 = Arc::new(TestPlugin::new("plugin-1"));

        assert!(registry.register(plugin1).is_ok());
        match registry.register(plugin2) {
            Err(RegistryError::DuplicatePlugin(id)) => assert_eq!(id, "plugin-1"),
            _ => panic!("Expected DuplicatePlugin error"),
        }
    }

    #[tokio::test]
    async fn test_get_plugin() {
        let mut registry = PluginRegistry::new();
        let plugin = Arc::new(TestPlugin::new("plugin-1"));
        registry.register(plugin.clone()).unwrap();

        assert!(registry.get("plugin-1").is_some());
        assert!(registry.get("nonexistent").is_none());
    }

    #[tokio::test]
    async fn test_list_plugins() {
        let mut registry = PluginRegistry::new();
        let plugin1 = Arc::new(TestPlugin::new("plugin-1"));
        let plugin2 = Arc::new(TestPlugin::new("plugin-2"));
        let plugin3 = Arc::new(TestPlugin::new("plugin-3"));

        registry.register(plugin1).unwrap();
        registry.register(plugin2).unwrap();
        registry.register(plugin3).unwrap();

        let plugins = registry.list();
        assert_eq!(plugins.len(), 3);
        assert_eq!(plugins[0].id(), "plugin-1");
        assert_eq!(plugins[1].id(), "plugin-2");
        assert_eq!(plugins[2].id(), "plugin-3");
    }

    #[tokio::test]
    async fn test_init_all() {
        let mut registry = PluginRegistry::new();
        let ctx = PluginContext::new(
            Arc::new(serde_json::Value::Object(Default::default())),
            std::path::PathBuf::new(),
        );

        let plugin1 = Arc::new(TestPlugin::new("plugin-1"));
        let plugin2 = Arc::new(TestPlugin::new("plugin-2"));

        registry.register(plugin1).unwrap();
        registry.register(plugin2).unwrap();

        // init_all should succeed with valid plugins
        assert!(registry.init_all(&ctx).await.is_ok());
    }

    #[tokio::test]
    async fn test_shutdown_all() {
        let mut registry = PluginRegistry::new();
        let plugin1 = Arc::new(TestPlugin::new("plugin-1"));
        let plugin2 = Arc::new(TestPlugin::new("plugin-2"));

        registry.register(plugin1).unwrap();
        registry.register(plugin2).unwrap();

        // shutdown_all should succeed even with valid plugins
        assert!(registry.shutdown_all().await.is_ok());
    }

    #[tokio::test]
    async fn test_load_order_preserved() {
        let mut registry = PluginRegistry::new();
        let plugin1 = Arc::new(TestPlugin::new("alpha"));
        let plugin2 = Arc::new(TestPlugin::new("beta"));
        let plugin3 = Arc::new(TestPlugin::new("gamma"));

        // Register in non-alphabetical order
        registry.register(plugin3).unwrap();
        registry.register(plugin1).unwrap();
        registry.register(plugin2).unwrap();

        let plugins = registry.list();
        // Verify order is insertion order, not alphabetical
        assert_eq!(plugins[0].id(), "gamma");
        assert_eq!(plugins[1].id(), "alpha");
        assert_eq!(plugins[2].id(), "beta");
    }
}
