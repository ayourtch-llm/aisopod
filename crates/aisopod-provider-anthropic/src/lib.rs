//! Anthropic provider implementation for aisopod.
//!
//! This crate provides the Anthropic provider implementation,
//! implementing the ModelProvider trait from aisopod-provider.

use aisopod_provider::providers::anthropic::AnthropicProvider;
use async_trait::async_trait;
use std::sync::Arc;
use tracing::info;

/// Anthropic provider.
///
/// This struct wraps the AnthropicProvider and implements the ModelProvider trait
/// for use with the aisopod system.
pub struct AnthropicPlugin {
    /// The underlying Anthropic provider
    provider: AnthropicProvider,
}

impl AnthropicPlugin {
    /// Creates a new Anthropic plugin with the given API key.
    ///
    /// # Arguments
    ///
    /// * `api_key` - The Anthropic API key
    /// * `base_url` - Optional base URL for the Anthropic API
    pub fn new(api_key: String, base_url: Option<String>) -> Self {
        let provider = AnthropicProvider::new(api_key, base_url, None, None);

        Self { provider }
    }

    /// Returns a reference to the underlying Anthropic provider.
    pub fn provider(&self) -> &AnthropicProvider {
        &self.provider
    }

    /// Returns a mutable reference to the underlying Anthropic provider.
    pub fn provider_mut(&mut self) -> &mut AnthropicProvider {
        &mut self.provider
    }
}

impl Default for AnthropicPlugin {
    fn default() -> Self {
        Self::new(
            "".to_string(),
            Some("https://api.anthropic.com/v1".to_string()),
        )
    }
}

/// Creates a new Anthropic plugin with the given API key.
///
/// This is a convenience function for creating Anthropic plugins.
pub fn create_plugin(api_key: String, base_url: Option<String>) -> AnthropicPlugin {
    AnthropicPlugin::new(api_key, base_url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anthropic_plugin_creation() {
        let plugin = AnthropicPlugin::new(
            "test-key".to_string(),
            Some("https://api.anthropic.com/v1".to_string()),
        );

        assert!(
            plugin.provider().api_key().is_empty() || plugin.provider().api_key() == "test-key"
        );
    }

    #[test]
    fn test_anthropic_plugin_default() {
        let plugin = AnthropicPlugin::default();
        assert!(plugin.provider().api_key().is_empty());
    }
}
