//! OpenAI provider implementation for aisopod.
//!
//! This crate provides the OpenAI provider implementation,
//! implementing the ModelProvider trait from aisopod-provider.

use aisopod_provider::providers::openai::OpenAIProvider;
use aisopod_provider::{ModelProvider, ChatCompletionRequest, Message};
use async_trait::async_trait;
use std::sync::Arc;
use tracing::info;

/// OpenAI provider.
///
/// This struct wraps the OpenAIProvider and implements the ModelProvider trait
/// for use with the aisopod system.
pub struct OpenAIPlugin {
    /// The underlying OpenAI provider
    provider: OpenAIProvider,
}

impl OpenAIPlugin {
    /// Creates a new OpenAI plugin with the given API key.
    ///
    /// # Arguments
    ///
    /// * `api_key` - The OpenAI API key
    /// * `base_url` - Optional base URL for the OpenAI API
    /// * `organization` - Optional OpenAI-Organization header
    pub fn new(api_key: String, base_url: Option<String>, organization: Option<String>) -> Self {
        let provider = OpenAIProvider::new(api_key, base_url, organization, None);
        
        Self { provider }
    }

    /// Returns a reference to the underlying OpenAI provider.
    pub fn provider(&self) -> &OpenAIProvider {
        &self.provider
    }

    /// Returns a mutable reference to the underlying OpenAI provider.
    pub fn provider_mut(&mut self) -> &mut OpenAIProvider {
        &mut self.provider
    }
}

impl Default for OpenAIPlugin {
    fn default() -> Self {
        Self::new(
            "".to_string(),
            Some("https://api.openai.com/v1".to_string()),
            None,
        )
    }
}

/// Creates a new OpenAI plugin with the given API key.
///
/// This is a convenience function for creating OpenAI plugins.
pub fn create_plugin(
    api_key: String,
    base_url: Option<String>,
    organization: Option<String>,
) -> OpenAIPlugin {
    OpenAIPlugin::new(api_key, base_url, organization)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_plugin_creation() {
        let plugin = OpenAIPlugin::new(
            "test-key".to_string(),
            Some("https://api.openai.com/v1".to_string()),
            None,
        );
        
        assert_eq!(plugin.provider().api_key(), "test-key");
    }

    #[test]
    fn test_openai_plugin_default() {
        let plugin = OpenAIPlugin::default();
        assert!(plugin.provider().api_key().is_empty());
    }
}
