use async_trait::async_trait;
use std::error::Error;

use crate::{PluginContext, PluginMeta};

/// The core trait that all plugins must implement.
///
/// The `Plugin` trait defines the lifecycle interface that every plugin
/// in the aisopod system must implement. Plugins go through several stages:
///
/// 1. **Discovery**: The plugin's `id()` and `meta()` are queried to identify
///    and describe the plugin.
/// 2. **Registration**: The `register()` method is called to allow the plugin
///    to register its capabilities with the system.
/// 3. **Initialization**: After all plugins are registered, `init()` is called
///    with the runtime context to perform any async setup.
/// 4. **Shutdown**: When the system is shutting down, `shutdown()` is called
///    to allow graceful cleanup.
///
/// # Lifetime and Ownership
///
/// Plugins must implement `Send + Sync` to support both compiled-in plugins
/// and dynamically loaded plugins. The trait is object-safe and can be used
/// as `dyn Plugin`.
///
/// # Example
///
/// ```rust
/// use aisopod_plugin::{Plugin, PluginMeta, PluginContext, PluginApi};
/// use std::sync::Arc;
/// use serde_json::Value;
///
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
///                 vec!["text".to_string()],
///                 vec!["discord".to_string()],
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
///     fn register(&self, _api: &mut dyn PluginApi) -> Result<(), Box<dyn std::error::Error>> {
///         // Register plugin capabilities here
///         Ok(())
///     }
///
///     async fn init(&self, _ctx: &PluginContext) -> Result<(), Box<dyn std::error::Error>> {
///         // Perform async initialization
///         Ok(())
///     }
///
///     async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
///         // Perform cleanup
///         Ok(())
///     }
/// }
/// ```
#[async_trait]
pub trait Plugin: Send + Sync + std::fmt::Debug {
    /// Returns the unique identifier for this plugin.
    ///
    /// This ID should be stable across versions and unique among all plugins.
    /// It is typically used for configuration lookup and plugin management.
    fn id(&self) -> &str;

    /// Returns metadata describing this plugin.
    fn meta(&self) -> &PluginMeta;

    /// Called during plugin loading to register capabilities.
    ///
    /// This method is called after all plugins are instantiated but before
    /// any plugin is initialized. It allows plugins to register handlers,
    /// commands, or other capabilities with the system's API.
    ///
    /// # Errors
    ///
    /// Returns an error if registration fails due to missing dependencies,
    /// invalid configuration, or other issues. The plugin system will
    /// consider the plugin failed if this returns an error.
    fn register(&self, _api: &mut dyn PluginApi) -> Result<(), Box<dyn Error>>;

    /// Called after all plugins are registered to perform initialization.
    ///
    /// This async method is where plugins should perform any async setup
    /// such as connecting to databases, starting background tasks, or
    /// loading cached data.
    ///
    /// # Errors
    ///
    /// Returns an error if initialization fails. The plugin will be
    /// considered failed and may be excluded from the active plugin set.
    async fn init(&self, _ctx: &PluginContext) -> Result<(), Box<dyn Error>>;

    /// Called during graceful shutdown.
    ///
    /// This async method is called when the system is shutting down.
    /// Plugins should use this opportunity to clean up resources,
    /// save state, and perform any necessary shutdown operations.
    ///
    /// # Errors
    ///
    /// Errors during shutdown are logged but do not affect the shutdown
    /// process itself.
    async fn shutdown(&self) -> Result<(), Box<dyn Error>>;
}

/// The API interface available to plugins during registration.
///
/// This trait provides the interface through which plugins can register
/// their capabilities with the system during the `register()` phase.
///
/// # Note
///
/// This is a minimal placeholder. As the plugin system grows, more methods
/// will be added to this trait to support various plugin capabilities.
pub trait PluginApi: Send + Sync {}
