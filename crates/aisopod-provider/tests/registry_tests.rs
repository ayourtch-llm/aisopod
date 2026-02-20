//! Registry tests for ProviderRegistry.
//!
//! These tests verify that:
//! - Provider registration, lookup, and listing work correctly
//! - Alias registration and resolution work correctly
//! - Model resolution with aliases and direct references works

use std::sync::Arc;

use aisopod_provider::registry::{ModelAlias, ProviderRegistry};
use aisopod_provider::trait_module::{ChatCompletionStream, ModelProvider};
use aisopod_provider::types::{
    ChatCompletionChunk, ChatCompletionRequest, FinishReason, Message, MessageContent,
    MessageDelta, ModelInfo, Role, TokenUsage,
};
use anyhow::Result;
use async_trait::async_trait;
use futures_util::stream::{self, StreamExt};

// ============================================================================
// Helper to make private fields accessible for testing
// ============================================================================

/// Helper function to get providers count for testing
fn providers_count(registry: &ProviderRegistry) -> usize {
    registry.providers.len()
}

/// Helper function to get aliases count for testing
fn aliases_count(registry: &ProviderRegistry) -> usize {
    registry.aliases.len()
}

/// Helper function to access providers HashMap for testing
/// This allows tests to inspect internal state when needed
fn get_providers(
    registry: &ProviderRegistry,
) -> &std::collections::HashMap<String, Arc<dyn ModelProvider>> {
    &registry.providers
}

// ============================================================================
// Provider Registration Tests
// ============================================================================

#[test]
fn test_new_registry_is_empty() {
    let registry = ProviderRegistry::new();
    assert_eq!(providers_count(&registry), 0);
    assert_eq!(aliases_count(&registry), 0);
}

#[test]
fn test_register_provider() {
    let mut registry = ProviderRegistry::new();

    let provider = TestProvider::new("test-provider", vec!["model1", "model2"]);
    registry.register(Arc::new(provider));

    assert_eq!(providers_count(&registry), 1);
    assert!(registry.get("test-provider").is_some());
}

#[test]
fn test_register_same_provider_twice_replaces() {
    let mut registry = ProviderRegistry::new();

    let provider1 = TestProvider::new("test", vec!["model1"]);
    registry.register(Arc::new(provider1));

    let provider2 = TestProvider::new("test", vec!["model2", "model3"]);
    registry.register(Arc::new(provider2));

    assert_eq!(providers_count(&registry), 1);

    let retrieved = registry.get("test").unwrap();
    assert_eq!(retrieved.id(), "test");
}

#[test]
fn test_register_different_providers() {
    let mut registry = ProviderRegistry::new();

    registry.register(Arc::new(TestProvider::new("provider1", vec!["model1"])));
    registry.register(Arc::new(TestProvider::new("provider2", vec!["model2"])));
    registry.register(Arc::new(TestProvider::new("provider3", vec!["model3"])));

    assert_eq!(providers_count(&registry), 3);
    assert!(registry.get("provider1").is_some());
    assert!(registry.get("provider2").is_some());
    assert!(registry.get("provider3").is_some());
}

// ============================================================================
// Provider Lookup Tests
// ============================================================================

#[test]
fn test_get_nonexistent_provider() {
    let registry = ProviderRegistry::new();
    assert!(registry.get("nonexistent").is_none());
}

#[test]
fn test_get_provider_returns_arc() {
    let mut registry = ProviderRegistry::new();

    let provider = TestProvider::new("test", vec![]);
    registry.register(Arc::new(provider));

    let retrieved = registry.get("test").unwrap();

    // Verify it's the same provider
    assert_eq!(retrieved.id(), "test");
}

// ============================================================================
// Provider Listing Tests
// ============================================================================

#[test]
fn test_list_providers_returns_all() {
    let mut registry = ProviderRegistry::new();

    registry.register(Arc::new(TestProvider::new("p1", vec![])));
    registry.register(Arc::new(TestProvider::new("p2", vec![])));
    registry.register(Arc::new(TestProvider::new("p3", vec![])));

    let providers = registry.list_providers();
    assert_eq!(providers.len(), 3);
}

#[test]
fn test_list_providers_returns_arcs() {
    let mut registry = ProviderRegistry::new();

    let provider1 = Arc::new(TestProvider::new("p1", vec![]));
    let provider2 = Arc::new(TestProvider::new("p2", vec![]));
    registry.register(provider1.clone());
    registry.register(provider2.clone());

    let providers = registry.list_providers();

    // Verify we get back arcs to the same providers
    let ids: Vec<&str> = providers.iter().map(|p| p.id()).collect();
    assert!(ids.contains(&"p1"));
    assert!(ids.contains(&"p2"));
}

#[test]
fn test_list_providers_can_call_methods() {
    let mut registry = ProviderRegistry::new();

    let provider = TestProvider::new("test", vec!["model1", "model2"]);
    registry.register(Arc::new(provider));

    let providers = registry.list_providers();
    let retrieved = providers.first().unwrap();

    // Verify we can actually use the provider
    assert_eq!(retrieved.id(), "test");
}

// ============================================================================
// Provider Unregistration Tests
// ============================================================================

#[test]
fn test_unregister_provider() {
    let mut registry = ProviderRegistry::new();

    let provider = TestProvider::new("test", vec![]);
    registry.register(Arc::new(provider));

    assert!(registry.get("test").is_some());

    registry.unregister("test");
    assert!(registry.get("test").is_none());
}

#[test]
fn test_unregister_nonexistent_provider() {
    let mut registry = ProviderRegistry::new();

    // Unregistering a non-existent provider should be a no-op
    registry.unregister("nonexistent");

    assert!(registry.providers.is_empty());
}

#[test]
fn test_unregister_does_not_affect_others() {
    let mut registry = ProviderRegistry::new();

    registry.register(Arc::new(TestProvider::new("keep", vec![])));
    registry.register(Arc::new(TestProvider::new("remove", vec![])));

    registry.unregister("remove");

    assert!(registry.get("keep").is_some());
    assert!(registry.get("remove").is_none());
    assert_eq!(registry.providers.len(), 1);
}

// ============================================================================
// Alias Registration Tests
// ============================================================================

#[test]
fn test_register_alias() {
    let mut registry = ProviderRegistry::new();

    registry.register_alias("claude-sonnet", "anthropic", "claude-3-5-sonnet");

    let alias = registry.resolve_alias("claude-sonnet").unwrap();
    assert_eq!(alias.provider_id, "anthropic");
    assert_eq!(alias.model_id, "claude-3-5-sonnet");
}

#[test]
fn test_register_multiple_aliases() {
    let mut registry = ProviderRegistry::new();

    registry.register_alias("gpt-4", "openai", "gpt-4-0613");
    registry.register_alias("claude-sonnet", "anthropic", "claude-3-5-sonnet");

    let gpt_alias = registry.resolve_alias("gpt-4").unwrap();
    let claude_alias = registry.resolve_alias("claude-sonnet").unwrap();

    assert_eq!(gpt_alias.provider_id, "openai");
    assert_eq!(gpt_alias.model_id, "gpt-4-0613");
    assert_eq!(claude_alias.provider_id, "anthropic");
    assert_eq!(claude_alias.model_id, "claude-3-5-sonnet");
}

#[test]
fn test_register_same_alias_twice_replaces() {
    let mut registry = ProviderRegistry::new();

    registry.register_alias("model", "provider1", "model1");
    registry.register_alias("model", "provider2", "model2");

    let alias = registry.resolve_alias("model").unwrap();
    assert_eq!(alias.provider_id, "provider2");
    assert_eq!(alias.model_id, "model2");
}

// ============================================================================
// Alias Resolution Tests
// ============================================================================

#[test]
fn test_resolve_alias_not_found() {
    let registry = ProviderRegistry::new();
    assert!(registry.resolve_alias("nonexistent").is_none());
}

#[test]
fn test_resolve_alias_returns_correct_data() {
    let mut registry = ProviderRegistry::new();

    registry.register_alias("my-model", "test-provider", "actual-model-id");

    let alias = registry.resolve_alias("my-model").unwrap();

    assert_eq!(alias.provider_id, "test-provider");
    assert_eq!(alias.model_id, "actual-model-id");
}

// ============================================================================
// Model Resolution Tests (via alias)
// ============================================================================

#[test]
fn test_resolve_model_via_alias() {
    let mut registry = ProviderRegistry::new();

    let provider = Arc::new(TestProvider::new("anthropic", vec!["claude-3-5-sonnet"]));
    registry.register(provider.clone());
    registry.register_alias("claude-sonnet", "anthropic", "claude-3-5-sonnet");

    let (resolved_provider, model_id) = registry.resolve_model("claude-sonnet").unwrap();

    assert_eq!(resolved_provider.id(), "anthropic");
    assert_eq!(model_id, "claude-3-5-sonnet");
}

#[test]
fn test_resolve_model_alias_not_found() {
    let mut registry = ProviderRegistry::new();

    let provider = Arc::new(TestProvider::new("openai", vec!["gpt-4"]));
    registry.register(provider);
    registry.register_alias("gpt-4", "openai", "gpt-4");

    assert!(registry.resolve_model("nonexistent").is_none());
}

#[test]
fn test_resolve_model_alias_provider_not_found() {
    let mut registry = ProviderRegistry::new();

    registry.register_alias("unknown", "nonexistent", "some-model");

    assert!(registry.resolve_model("unknown").is_none());
}

// ============================================================================
// Model Resolution Tests (direct provider ID)
// ============================================================================

#[test]
fn test_resolve_model_direct_provider() {
    let mut registry = ProviderRegistry::new();

    let provider = Arc::new(TestProvider::new("openai", vec!["gpt-4", "gpt-3.5-turbo"]));
    registry.register(provider.clone());

    let (resolved_provider, model_id) = registry.resolve_model("openai").unwrap();

    assert_eq!(resolved_provider.id(), "openai");
    // When resolving by direct provider ID, the model_id is the provider_id
    assert_eq!(model_id, "openai");
}

#[test]
fn test_resolve_model_provider_not_found() {
    let registry = ProviderRegistry::new();
    assert!(registry.resolve_model("nonexistent").is_none());
}

// ============================================================================
// Combined Alias and Provider Tests
// ============================================================================

#[test]
fn test_resolve_model_prefers_alias() {
    let mut registry = ProviderRegistry::new();

    // Register a provider directly
    let direct_provider = Arc::new(TestProvider::new("test", vec![]));
    registry.register(direct_provider.clone());

    // Also register an alias pointing to the same provider
    registry.register_alias("test", "test", "test-model");

    // When resolving by "test", it should find the alias first
    // Actually, looking at the code, aliases are checked first in resolve_model
    let result = registry.resolve_model("test");

    // The alias should take precedence
    assert!(result.is_some());
    let (provider, model_id) = result.unwrap();
    assert_eq!(provider.id(), "test");
    assert_eq!(model_id, "test-model");
}

#[test]
fn test_multiple_providers_same_alias_not_supported() {
    // This test documents that aliases map one-to-one
    let mut registry = ProviderRegistry::new();

    registry.register_alias("model", "provider1", "model1");

    // Registering the same alias again replaces the old one
    registry.register_alias("model", "provider2", "model2");

    let alias = registry.resolve_alias("model").unwrap();
    assert_eq!(alias.provider_id, "provider2");
}

// ============================================================================
// Default Implementation Tests
// ============================================================================

#[test]
fn test_default_registry() {
    let registry = ProviderRegistry::default();
    assert_eq!(providers_count(&registry), 0);
    assert_eq!(aliases_count(&registry), 0);
}

// ============================================================================
// Integration with ProviderRegistry Tests
// ============================================================================

#[test]
fn test_full_workflow() {
    let mut registry = ProviderRegistry::new();

    // 1. Register providers
    let anthropic = Arc::new(TestProvider::new("anthropic", vec!["claude-3-5-sonnet"]));
    let openai = Arc::new(TestProvider::new("openai", vec!["gpt-4", "gpt-3.5-turbo"]));

    registry.register(anthropic.clone());
    registry.register(openai.clone());

    // 2. Register aliases
    registry.register_alias("claude-sonnet", "anthropic", "claude-3-5-sonnet");
    registry.register_alias("gpt-4", "openai", "gpt-4");

    // 3. Test provider lookup
    assert!(registry.get("anthropic").is_some());
    assert!(registry.get("openai").is_some());

    // 4. Test alias resolution
    let alias1 = registry.resolve_alias("claude-sonnet").unwrap();
    assert_eq!(alias1.provider_id, "anthropic");

    let alias2 = registry.resolve_alias("gpt-4").unwrap();
    assert_eq!(alias2.provider_id, "openai");

    // 5. Test model resolution via alias
    let (provider, model_id) = registry.resolve_model("claude-sonnet").unwrap();
    assert_eq!(provider.id(), "anthropic");
    assert_eq!(model_id, "claude-3-5-sonnet");

    let (provider, model_id) = registry.resolve_model("gpt-4").unwrap();
    assert_eq!(provider.id(), "openai");
    assert_eq!(model_id, "gpt-4");

    // 6. Test model resolution via direct provider ID
    let (provider, model_id) = registry.resolve_model("openai").unwrap();
    assert_eq!(provider.id(), "openai");
    assert_eq!(model_id, "openai");
}

// ============================================================================
// Mock Provider Implementation
// ============================================================================

/// A simple mock provider for testing the registry.
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

#[async_trait]
impl ModelProvider for TestProvider {
    fn id(&self) -> &str {
        &self.id
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        Ok(self
            .models
            .iter()
            .map(|m| ModelInfo {
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
        _request: ChatCompletionRequest,
    ) -> Result<ChatCompletionStream> {
        Ok(Box::pin(stream::empty()))
    }

    async fn health_check(&self) -> Result<aisopod_provider::ProviderHealth> {
        Ok(aisopod_provider::ProviderHealth {
            available: true,
            latency_ms: Some(10),
        })
    }
}
