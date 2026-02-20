use serde::{Deserialize, Serialize};

/// Plugins configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginsConfig {
    /// Plugin registry
    #[serde(default)]
    pub registry: Vec<PluginEntry>,
    /// Plugin settings
    #[serde(default)]
    pub settings: PluginSettings,
}

/// Plugin registry entry
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginEntry {
    /// Plugin ID
    pub id: String,
    /// Plugin name
    #[serde(default)]
    pub name: String,
    /// Plugin version
    #[serde(default)]
    pub version: String,
    /// Enabled flag
    #[serde(default)]
    pub enabled: bool,
}

/// Plugin settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginSettings {
    /// Auto-load plugins
    #[serde(default)]
    pub auto_load: bool,
    /// Plugin directory
    #[serde(default)]
    pub plugin_dir: String,
    /// Load timeout in seconds
    #[serde(default = "default_timeout")]
    pub load_timeout: u64,
}

fn default_timeout() -> u64 {
    30
}
