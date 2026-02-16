use std::path::PathBuf;
use std::sync::Arc;

use serde_json::Value;

/// Runtime context provided to plugins during initialization.
///
/// This struct contains runtime information and resources that plugins
/// may need during their `init()` phase, such as configuration data
/// and access to a dedicated data directory.
pub struct PluginContext {
    /// The plugin's configuration as a JSON value.
    ///
    /// This contains the deserialized configuration from the plugin's
    /// configuration section in the main configuration file.
    pub config: Arc<Value>,
    /// The path to a dedicated data directory for this plugin.
    ///
    /// Plugins may use this directory to store persistent data,
    /// caches, or other runtime files. The directory is guaranteed
    /// to exist and be writable by the plugin.
    pub data_dir: PathBuf,
}

impl PluginContext {
    /// Creates a new `PluginContext` instance.
    pub fn new(config: Arc<Value>, data_dir: PathBuf) -> Self {
        Self { config, data_dir }
    }
}

impl std::fmt::Debug for PluginContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginContext")
            .field("data_dir", &self.data_dir)
            .finish()
    }
}
