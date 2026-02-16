use serde::{Deserialize, Serialize};

/// Metadata describing a plugin's identity and capabilities.
///
/// This struct contains information about a plugin that can be used
/// for discovery, registration, and compatibility checking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMeta {
    /// The unique identifier for this plugin.
    pub name: String,
    /// The version of this plugin following semantic versioning.
    pub version: String,
    /// A brief description of what this plugin does.
    pub description: String,
    /// The author or organization that created this plugin.
    pub author: String,
    /// The channel types this plugin supports (e.g., "text", "voice", "dm").
    pub supported_channels: Vec<String>,
    /// The provider types this plugin supports (e.g., "discord", "slack", "matrix").
    pub supported_providers: Vec<String>,
}

impl PluginMeta {
    /// Creates a new `PluginMeta` instance with the given fields.
    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        description: impl Into<String>,
        author: impl Into<String>,
        supported_channels: Vec<String>,
        supported_providers: Vec<String>,
    ) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            description: description.into(),
            author: author.into(),
            supported_channels,
            supported_providers,
        }
    }
}
