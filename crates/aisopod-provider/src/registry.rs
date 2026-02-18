//! Provider registry for managing model provider instances.
//!
//! This module provides the [`ProviderRegistry`] struct, which serves as a
//! central registry for AI model providers. It allows registration, lookup,
//! and listing of providers, as well as resolution of model aliases.

use crate::trait_module::ModelProvider;
use std::collections::HashMap;
use std::sync::Arc;

/// A model alias mapping a friendly name to a specific provider and model.
///
/// Aliases allow users to refer to models using convenient names (e.g.,
/// `"claude-sonnet"`) that map to the actual provider ID and model ID
/// (e.g., provider `"anthropic"`, model `"claude-3-5-sonnet"`).
#[derive(Debug, Clone, PartialEq)]
pub struct ModelAlias {
    /// The ID of the provider that hosts this model.
    pub provider_id: String,
    /// The canonical model ID within the provider.
    pub model_id: String,
}

/// A registry for managing AI model provider instances.
///
/// The `ProviderRegistry` serves as the central dispatch point for all
/// provider interactions. It stores provider instances keyed by their
/// unique ID and supports:
///
/// - Registering new providers
/// - Looking up providers by ID
/// - Listing all registered providers
/// - Registering and resolving model aliases
/// - Resolving model names (either direct IDs or aliases) to providers
///
/// # Example
///
/// ```ignore
/// use std::sync::Arc;
/// use aisopod_provider::{ModelProvider, ProviderRegistry};
///
/// async fn example(provider: impl ModelProvider) -> anyhow::Result<()> {
///     let mut registry = ProviderRegistry::new();
///     registry.register(Arc::new(provider));
///
///     // Look up a provider
///     if let Some(retrieved) = registry.get("my-provider") {
///         // Use the provider
///     }
///
///     // Register an alias for a model
///     registry.register_alias("claude-sonnet", "anthropic", "claude-3-5-sonnet");
///
///     // Resolve an alias to a provider/model pair
///     if let Some(alias) = registry.resolve_alias("claude-sonnet") {
///         println!("Alias points to: {}/{}", alias.provider_id, alias.model_id);
///     }
///
///     Ok(())
/// }
/// ```
pub struct ProviderRegistry {
    pub providers: HashMap<String, Arc<dyn ModelProvider>>,
    pub aliases: HashMap<String, ModelAlias>,
}

impl ProviderRegistry {
    /// Creates a new empty `ProviderRegistry`.
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            aliases: HashMap::new(),
        }
    }

    /// Registers a provider with the registry.
    ///
    /// The provider is keyed by its `id()` method. If a provider with
    /// the same ID is already registered, it will be replaced.
    ///
    /// # Arguments
    ///
    /// * `provider` - An `Arc` wrapping the provider instance.
    pub fn register(&mut self, provider: Arc<dyn ModelProvider>) {
        let id = provider.id().to_string();
        self.providers.insert(id, provider);
    }

    /// Unregisters a provider from the registry.
    ///
    /// Removes the provider with the given ID. If the provider is not
    /// registered, this is a no-op.
    ///
    /// # Arguments
    ///
    /// * `provider_id` - The ID of the provider to remove.
    pub fn unregister(&mut self, provider_id: &str) {
        self.providers.remove(provider_id);
    }

    /// Looks up a provider by its ID.
    ///
    /// Returns `Some` with an `Arc` to the provider if found, `None` otherwise.
    ///
    /// # Arguments
    ///
    /// * `provider_id` - The ID of the provider to look up.
    pub fn get(&self, provider_id: &str) -> Option<Arc<dyn ModelProvider>> {
        self.providers.get(provider_id).cloned()
    }

    /// Returns a list of all registered providers.
    ///
    /// The order of providers in the list is not guaranteed.
    pub fn list_providers(&self) -> Vec<Arc<dyn ModelProvider>> {
        self.providers.values().cloned().collect()
    }

    /// Registers a model alias.
    ///
    /// Creates a mapping from a friendly alias name to a specific
    /// provider and model ID.
    ///
    /// # Arguments
    ///
    /// * `alias` - The friendly name to register (e.g., `"claude-sonnet"`).
    /// * `provider_id` - The ID of the provider that hosts this model.
    /// * `model_id` - The canonical model ID within the provider.
    pub fn register_alias(&mut self, alias: &str, provider_id: &str, model_id: &str) {
        let model_alias = ModelAlias {
            provider_id: provider_id.to_string(),
            model_id: model_id.to_string(),
        };
        self.aliases.insert(alias.to_string(), model_alias);
    }

    /// Resolves a model alias to a provider/model pair.
    ///
    /// Returns `Some` with the `ModelAlias` if the alias is registered,
    /// `None` otherwise.
    ///
    /// # Arguments
    ///
    /// * `alias` - The alias to resolve.
    pub fn resolve_alias(&self, alias: &str) -> Option<&ModelAlias> {
        self.aliases.get(alias)
    }

    /// Resolves a model name (or alias) to a provider and canonical model ID.
    ///
    /// This method handles both direct provider/model references and
    /// aliases. If the name is not an alias, it is treated as a direct
    /// provider ID.
    ///
    /// # Arguments
    ///
    /// * `name` - The model name or alias to resolve.
    ///
    /// # Returns
    ///
    /// Returns `Some((provider, model_id))` if the model can be resolved,
    /// `None` otherwise.
    pub fn resolve_model(&self, name: &str) -> Option<(Arc<dyn ModelProvider>, String)> {
        // First, check if the name is an alias
        if let Some(alias) = self.aliases.get(name) {
            return self
                .providers
                .get(&alias.provider_id)
                .cloned()
                .map(|provider| (provider, alias.model_id.clone()));
        }

        // If not an alias, treat as a direct provider ID
        // We need to find which provider supports this model ID
        // by checking if any provider has this model in its list
        for (provider_id, provider) in &self.providers {
            if provider_id == name {
                // The name matches a provider ID directly
                return self
                    .providers
                    .get(provider_id)
                    .cloned()
                    .map(|provider| (provider, name.to_string()));
            }
        }

        None
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::stream;
    use std::pin::Pin;

    /// A mock provider implementation for testing.
    struct TestProvider {
        id: String,
        models: Vec<String>,
    }

    impl TestProvider {
        fn new(id: &str, models: Vec<&str>) -> Self {
            Self {
                id: id.to_string(),
                models: models.into_iter().map(|s| s.to_string()).collect(),
            }
        }
    }

    #[async_trait::async_trait]
    impl ModelProvider for TestProvider {
        fn id(&self) -> &str {
            &self.id
        }

        async fn list_models(&self) -> Result<Vec<crate::ModelInfo>, anyhow::Error> {
            Ok(self
                .models
                .iter()
                .map(|m| crate::ModelInfo {
                    id: m.clone(),
                    name: m.clone(),
                    provider: self.id.clone(),
                    context_window: 8192,
                    supports_vision: false,
                    supports_tools: false,
                })
                .collect())
        }

        async fn chat_completion(
            &self,
            _request: crate::ChatCompletionRequest,
        ) -> Result<
            Pin<Box<dyn futures_util::stream::Stream<Item = Result<crate::ChatCompletionChunk, anyhow::Error>> + Send>>,
            anyhow::Error,
        > {
            Ok(Box::pin(stream::empty()))
        }

        async fn health_check(&self) -> Result<crate::ProviderHealth, anyhow::Error> {
            Ok(crate::ProviderHealth {
                available: true,
                latency_ms: Some(100),
            })
        }
    }

    #[test]
    fn test_new_registry() {
        let registry = ProviderRegistry::new();
        assert!(registry.providers.is_empty());
        assert!(registry.aliases.is_empty());
    }

    #[test]
    fn test_register_provider() {
        let mut registry = ProviderRegistry::new();
        let provider: Arc<dyn ModelProvider> = Arc::new(TestProvider::new("test-provider", vec!["model1", "model2"]));
        registry.register(Arc::clone(&provider));

        assert_eq!(registry.providers.len(), 1);
        assert!(registry.get("test-provider").is_some());
    }

    #[test]
    fn test_unregister_provider() {
        let mut registry = ProviderRegistry::new();
        let provider: Arc<dyn ModelProvider> = Arc::new(TestProvider::new("test-provider", vec!["model1"]));
        registry.register(Arc::clone(&provider));

        assert!(registry.get("test-provider").is_some());

        registry.unregister("test-provider");
        assert!(registry.get("test-provider").is_none());
    }

    #[test]
    fn test_get_provider() {
        let mut registry = ProviderRegistry::new();
        let provider: Arc<dyn ModelProvider> = Arc::new(TestProvider::new("provider-a", vec!["model1"]));
        registry.register(Arc::clone(&provider));

        let retrieved = registry.get("provider-a").unwrap();
        assert_eq!(retrieved.id(), "provider-a");
    }

    #[test]
    fn test_get_nonexistent_provider() {
        let registry = ProviderRegistry::new();
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_list_providers() {
        let mut registry = ProviderRegistry::new();
        let p1: Arc<dyn ModelProvider> = Arc::new(TestProvider::new("provider-1", vec!["model1"]));
        let p2: Arc<dyn ModelProvider> = Arc::new(TestProvider::new("provider-2", vec!["model2"]));
        registry.register(Arc::clone(&p1));
        registry.register(Arc::clone(&p2));

        let providers = registry.list_providers();
        assert_eq!(providers.len(), 2);
        let ids: Vec<&str> = providers.iter().map(|p| p.id()).collect();
        assert!(ids.contains(&"provider-1"));
        assert!(ids.contains(&"provider-2"));
    }

    #[test]
    fn test_register_alias() {
        let mut registry = ProviderRegistry::new();
        registry.register_alias("claude-sonnet", "anthropic", "claude-3-5-sonnet");

        let alias = registry.resolve_alias("claude-sonnet").unwrap();
        assert_eq!(alias.provider_id, "anthropic");
        assert_eq!(alias.model_id, "claude-3-5-sonnet");
    }

    #[test]
    fn test_resolve_alias() {
        let mut registry = ProviderRegistry::new();
        registry.register_alias("gpt-4", "openai", "gpt-4-0613");

        let alias = registry.resolve_alias("gpt-4").unwrap();
        assert_eq!(alias.provider_id, "openai");
        assert_eq!(alias.model_id, "gpt-4-0613");
    }

    #[test]
    fn test_resolve_alias_not_found() {
        let registry = ProviderRegistry::new();
        assert!(registry.resolve_alias("nonexistent").is_none());
    }

    #[test]
    fn test_resolve_model_direct() {
        let mut registry = ProviderRegistry::new();
        let provider: Arc<dyn ModelProvider> = Arc::new(TestProvider::new("openai", vec!["gpt-4", "gpt-3.5-turbo"]));
        registry.register(Arc::clone(&provider));

        let result = registry.resolve_model("openai");
        assert!(result.is_some());
        let (resolved_provider, model_id) = result.unwrap();
        assert_eq!(resolved_provider.id(), "openai");
        assert_eq!(model_id, "openai");
    }

    #[test]
    fn test_resolve_model_via_alias() {
        let mut registry = ProviderRegistry::new();
        let provider: Arc<dyn ModelProvider> = Arc::new(TestProvider::new("anthropic", vec!["claude-3-5-sonnet"]));
        registry.register(Arc::clone(&provider));
        registry.register_alias("claude-sonnet", "anthropic", "claude-3-5-sonnet");

        let result = registry.resolve_model("claude-sonnet");
        assert!(result.is_some());
        let (resolved_provider, model_id) = result.unwrap();
        assert_eq!(resolved_provider.id(), "anthropic");
        assert_eq!(model_id, "claude-3-5-sonnet");
    }

    #[test]
    fn test_resolve_model_alias_not_found() {
        let mut registry = ProviderRegistry::new();
        let provider: Arc<dyn ModelProvider> = Arc::new(TestProvider::new("openai", vec!["gpt-4"]));
        registry.register(Arc::clone(&provider));

        assert!(registry.resolve_model("nonexistent-alias").is_none());
    }

    #[test]
    fn test_resolve_model_provider_not_found() {
        let mut registry = ProviderRegistry::new();
        registry.register_alias("unknown-model", "nonexistent", "some-model");

        assert!(registry.resolve_model("unknown-model").is_none());
    }

    #[test]
    fn test_default_registry() {
        let registry = ProviderRegistry::default();
        assert!(registry.providers.is_empty());
    }
}
