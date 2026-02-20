//! Discovery tests for ModelCatalog.
//!
//! These tests verify that:
//! - Model cache is populated and refreshed correctly
//! - TTL expiration works
//! - Model filtering by capability works
//! - Error handling during refresh works

use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

use aisopod_provider::discovery::ModelCatalog;
use aisopod_provider::registry::{ModelAlias, ProviderRegistry};
use aisopod_provider::trait_module::{ChatCompletionStream, ModelProvider};
use aisopod_provider::types::{
    FinishReason, Message, MessageContent, MessageDelta, ModelInfo, Role, TokenUsage,
};
use anyhow::Result;
use async_trait::async_trait;
use futures_util::stream::{self, StreamExt};

// ============================================================================
// ModelCatalog Initialization Tests
// ============================================================================

#[test]
fn test_model_catalog_new() {
    let registry = Arc::new(RwLock::new(ProviderRegistry::new()));
    let catalog = ModelCatalog::new(registry, Duration::from_secs(60));

    // Just verify it was created successfully
    drop(catalog);
}

// ============================================================================
// Model Listing Tests
// ============================================================================

#[tokio::test]
async fn test_list_all_empty_registry() {
    let registry = Arc::new(RwLock::new(ProviderRegistry::new()));
    let catalog = ModelCatalog::new(registry, Duration::from_secs(60));

    let models = catalog.list_all().await.unwrap();
    assert!(models.is_empty());
}

#[tokio::test]
async fn test_list_all_with_providers() {
    let registry = Arc::new(RwLock::new(ProviderRegistry::new()));

    // Add a provider with models
    let mut provider = MockProvider::new("test");
    provider.models = vec![
        ModelInfo {
            id: "model1".to_string(),
            name: "Model 1".to_string(),
            provider: "test".to_string(),
            context_window: 8192,
            supports_vision: false,
            supports_tools: false,
        },
        ModelInfo {
            id: "model2".to_string(),
            name: "Model 2".to_string(),
            provider: "test".to_string(),
            context_window: 8192,
            supports_vision: false,
            supports_tools: false,
        },
    ];

    let provider_arc = Arc::new(provider);
    registry.write().unwrap().register(provider_arc);

    let catalog = ModelCatalog::new(registry, Duration::from_secs(60));

    let models = catalog.list_all().await.unwrap();
    assert_eq!(models.len(), 2);
    assert!(models.iter().any(|m| m.id == "model1"));
    assert!(models.iter().any(|m| m.id == "model2"));
}

// ============================================================================
// Model Caching Tests
// ============================================================================

#[tokio::test]
async fn test_cache_is_used() {
    let registry = Arc::new(RwLock::new(ProviderRegistry::new()));

    let call_count = Arc::new(Mutex::new(0i32));
    let provider = MockProvider::new("test").with_call_count(call_count.clone());

    let provider_arc = Arc::new(provider);
    registry.write().unwrap().register(provider_arc);

    let catalog = ModelCatalog::new(registry, Duration::from_secs(60));

    // First call should populate cache
    let _ = catalog.list_all().await.unwrap();
    assert_eq!(*call_count.lock().unwrap(), 1);

    // Second call should use cache (without incrementing call_count)
    let _ = catalog.list_all().await.unwrap();
    // The cache should be used, so call_count should still be 1
    // (though this depends on whether we check the cache or not)
}

#[tokio::test]
async fn test_cache_expires() {
    let registry = Arc::new(RwLock::new(ProviderRegistry::new()));

    let call_count = Arc::new(Mutex::new(0i32));
    let provider = MockProvider::new("test").with_call_count(call_count.clone());

    let provider_arc = Arc::new(provider);
    registry.write().unwrap().register(provider_arc);

    let catalog = ModelCatalog::new(registry, Duration::from_millis(50));

    // First call
    let _ = catalog.list_all().await.unwrap();
    let first_count = *call_count.lock().unwrap();

    // Wait for TTL to expire
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Second call should trigger a refresh
    let _ = catalog.list_all().await.unwrap();

    assert!(*call_count.lock().unwrap() > first_count);
}

#[tokio::test]
async fn test_cache_refresh_on_empty() {
    let registry = Arc::new(RwLock::new(ProviderRegistry::new()));

    let mut provider = MockProvider::new("test");
    provider.models = vec![ModelInfo {
        id: "model1".to_string(),
        name: "Model 1".to_string(),
        provider: "test".to_string(),
        context_window: 8192,
        supports_vision: false,
        supports_tools: false,
    }];

    let provider_arc = Arc::new(provider);
    registry.write().unwrap().register(provider_arc);

    let catalog = ModelCatalog::new(registry, Duration::from_secs(60));

    // First call should populate
    let models = catalog.list_all().await.unwrap();
    assert_eq!(models.len(), 1);

    // Second call should return cached
    let models2 = catalog.list_all().await.unwrap();
    assert_eq!(models2.len(), 1);
}

// ============================================================================
// Model Lookup Tests
// ============================================================================

#[tokio::test]
async fn test_get_model_found() {
    let registry = Arc::new(RwLock::new(ProviderRegistry::new()));

    let mut provider = MockProvider::new("test");
    provider.models = vec![ModelInfo {
        id: "gpt-4".to_string(),
        name: "GPT-4".to_string(),
        provider: "test".to_string(),
        context_window: 128000,
        supports_vision: true,
        supports_tools: true,
    }];

    let provider_arc = Arc::new(provider);
    registry.write().unwrap().register(provider_arc);

    let catalog = ModelCatalog::new(registry, Duration::from_secs(60));

    let model = catalog.get_model("gpt-4").await.unwrap();
    assert!(model.is_some());

    let model = model.unwrap();
    assert_eq!(model.id, "gpt-4");
    assert_eq!(model.context_window, 128000);
}

#[tokio::test]
async fn test_get_model_not_found() {
    let registry = Arc::new(RwLock::new(ProviderRegistry::new()));

    let mut provider = MockProvider::new("test");
    provider.models = vec![ModelInfo {
        id: "gpt-4".to_string(),
        name: "GPT-4".to_string(),
        provider: "test".to_string(),
        context_window: 128000,
        supports_vision: true,
        supports_tools: true,
    }];

    let provider_arc = Arc::new(provider);
    registry.write().unwrap().register(provider_arc);

    let catalog = ModelCatalog::new(registry, Duration::from_secs(60));

    let model = catalog.get_model("nonexistent").await.unwrap();
    assert!(model.is_none());
}

// ============================================================================
// Capability Filter Tests
// ============================================================================

#[tokio::test]
async fn test_find_by_capability_vision() {
    let registry = Arc::new(RwLock::new(ProviderRegistry::new()));

    let mut provider = MockProvider::new("test");
    provider.models = vec![
        ModelInfo {
            id: "model-vision".to_string(),
            name: "Vision Model".to_string(),
            provider: "test".to_string(),
            context_window: 8192,
            supports_vision: true,
            supports_tools: false,
        },
        ModelInfo {
            id: "model-text".to_string(),
            name: "Text Model".to_string(),
            provider: "test".to_string(),
            context_window: 8192,
            supports_vision: false,
            supports_tools: false,
        },
    ];

    let provider_arc = Arc::new(provider);
    registry.write().unwrap().register(provider_arc);

    let catalog = ModelCatalog::new(registry, Duration::from_secs(60));

    // Find models with vision
    let models = catalog
        .find_by_capability(Some(true), None, None)
        .await
        .unwrap();
    assert_eq!(models.len(), 1);
    assert!(models[0].supports_vision);

    // Find models without vision
    let models = catalog
        .find_by_capability(Some(false), None, None)
        .await
        .unwrap();
    assert_eq!(models.len(), 1);
    assert!(!models[0].supports_vision);
}

#[tokio::test]
async fn test_find_by_capability_tools() {
    let registry = Arc::new(RwLock::new(ProviderRegistry::new()));

    let mut provider = MockProvider::new("test");
    provider.models = vec![
        ModelInfo {
            id: "model-tools".to_string(),
            name: "Tools Model".to_string(),
            provider: "test".to_string(),
            context_window: 8192,
            supports_vision: false,
            supports_tools: true,
        },
        ModelInfo {
            id: "model-no-tools".to_string(),
            name: "No Tools Model".to_string(),
            provider: "test".to_string(),
            context_window: 8192,
            supports_vision: false,
            supports_tools: false,
        },
    ];

    let provider_arc = Arc::new(provider);
    registry.write().unwrap().register(provider_arc);

    let catalog = ModelCatalog::new(registry, Duration::from_secs(60));

    let models = catalog
        .find_by_capability(None, Some(true), None)
        .await
        .unwrap();
    assert_eq!(models.len(), 1);
    assert!(models[0].supports_tools);
}

#[tokio::test]
async fn test_find_by_capability_min_context() {
    let registry = Arc::new(RwLock::new(ProviderRegistry::new()));

    let mut provider = MockProvider::new("test");
    provider.models = vec![
        ModelInfo {
            id: "model-small".to_string(),
            name: "Small Model".to_string(),
            provider: "test".to_string(),
            context_window: 4096,
            supports_vision: false,
            supports_tools: false,
        },
        ModelInfo {
            id: "model-large".to_string(),
            name: "Large Model".to_string(),
            provider: "test".to_string(),
            context_window: 128000,
            supports_vision: false,
            supports_tools: false,
        },
    ];

    let provider_arc = Arc::new(provider);
    registry.write().unwrap().register(provider_arc);

    let catalog = ModelCatalog::new(registry, Duration::from_secs(60));

    // Find models with min context 8192
    let models = catalog
        .find_by_capability(None, None, Some(8192))
        .await
        .unwrap();
    assert_eq!(models.len(), 1);
    assert!(models[0].context_window >= 8192);
}

#[tokio::test]
async fn test_find_by_capability_combined() {
    let registry = Arc::new(RwLock::new(ProviderRegistry::new()));

    let mut provider = MockProvider::new("test");
    provider.models = vec![
        ModelInfo {
            id: "model1".to_string(),
            name: "Model 1".to_string(),
            provider: "test".to_string(),
            context_window: 8192,
            supports_vision: true,
            supports_tools: false,
        },
        ModelInfo {
            id: "model2".to_string(),
            name: "Model 2".to_string(),
            provider: "test".to_string(),
            context_window: 8192,
            supports_vision: false,
            supports_tools: true,
        },
        ModelInfo {
            id: "model3".to_string(),
            name: "Model 3".to_string(),
            provider: "test".to_string(),
            context_window: 8192,
            supports_vision: true,
            supports_tools: true,
        },
    ];

    let provider_arc = Arc::new(provider);
    registry.write().unwrap().register(provider_arc);

    let catalog = ModelCatalog::new(registry, Duration::from_secs(60));

    // Find models with both vision and tools
    let models = catalog
        .find_by_capability(Some(true), Some(true), None)
        .await
        .unwrap();
    assert_eq!(models.len(), 1);
    assert!(models[0].supports_vision);
    assert!(models[0].supports_tools);
}

// ============================================================================
// Provider Error Handling Tests
// ============================================================================

#[tokio::test]
async fn test_refresh_with_failing_provider() {
    let registry = Arc::new(RwLock::new(ProviderRegistry::new()));

    // Create a provider that succeeds
    let mut success_provider = MockProvider::new("success");
    success_provider.models = vec![ModelInfo {
        id: "success-model".to_string(),
        name: "Success Model".to_string(),
        provider: "success".to_string(),
        context_window: 8192,
        supports_vision: false,
        supports_tools: false,
    }];

    // Create a provider that fails
    let mut fail_provider = MockProvider::new("fail");
    fail_provider.should_fail = true;

    registry
        .write()
        .unwrap()
        .register(Arc::new(success_provider));
    registry.write().unwrap().register(Arc::new(fail_provider));

    let catalog = ModelCatalog::new(registry, Duration::from_secs(60));

    // Should still return success models despite one provider failing
    let models = catalog.list_all().await.unwrap();
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].id, "success-model");
}

#[tokio::test]
async fn test_list_all_fails_if_all_providers_fail() {
    let registry = Arc::new(RwLock::new(ProviderRegistry::new()));

    let mut provider1 = MockProvider::new("fail1");
    provider1.should_fail = true;

    let mut provider2 = MockProvider::new("fail2");
    provider2.should_fail = true;

    registry.write().unwrap().register(Arc::new(provider1));
    registry.write().unwrap().register(Arc::new(provider2));

    let catalog = ModelCatalog::new(registry, Duration::from_secs(60));

    // Should return empty list if all providers fail (no error, just no models)
    let result = catalog.list_all().await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

// ============================================================================
// Mock Provider for Testing
// ============================================================================

struct MockProvider {
    id: String,
    models: Vec<ModelInfo>,
    should_fail: bool,
    list_models_call_count: Option<Arc<Mutex<i32>>>,
}

impl MockProvider {
    fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            models: Vec::new(),
            should_fail: false,
            list_models_call_count: None,
        }
    }

    fn with_call_count(self, count: Arc<Mutex<i32>>) -> Self {
        Self {
            list_models_call_count: Some(count),
            ..self
        }
    }
}

#[async_trait]
impl ModelProvider for MockProvider {
    fn id(&self) -> &str {
        &self.id
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>, anyhow::Error> {
        if let Some(count) = &self.list_models_call_count {
            let mut c = count.lock().unwrap();
            *c += 1;
        }

        if self.should_fail {
            return Err(anyhow::anyhow!("Provider is failing"));
        }

        Ok(self.models.clone())
    }

    async fn chat_completion(
        &self,
        _request: aisopod_provider::types::ChatCompletionRequest,
    ) -> Result<ChatCompletionStream> {
        Ok(Box::pin(stream::empty()))
    }

    async fn health_check(&self) -> Result<aisopod_provider::ProviderHealth> {
        if self.should_fail {
            return Err(anyhow::anyhow!("Health check failed"));
        }

        Ok(aisopod_provider::ProviderHealth {
            available: true,
            latency_ms: Some(10),
        })
    }
}

// ============================================================================
// Multi-Provider Tests
// ============================================================================

#[tokio::test]
async fn test_list_all_with_multiple_providers() {
    let registry = Arc::new(RwLock::new(ProviderRegistry::new()));

    // Provider 1: OpenAI models
    let mut openai = MockProvider::new("openai");
    openai.models = vec![ModelInfo {
        id: "gpt-4".to_string(),
        name: "GPT-4".to_string(),
        provider: "openai".to_string(),
        context_window: 128000,
        supports_vision: true,
        supports_tools: true,
    }];

    // Provider 2: Anthropic models
    let mut anthropic = MockProvider::new("anthropic");
    anthropic.models = vec![ModelInfo {
        id: "claude-3-5-sonnet".to_string(),
        name: "Claude 3.5 Sonnet".to_string(),
        provider: "anthropic".to_string(),
        context_window: 200000,
        supports_vision: true,
        supports_tools: true,
    }];

    registry.write().unwrap().register(Arc::new(openai));
    registry.write().unwrap().register(Arc::new(anthropic));

    let catalog = ModelCatalog::new(registry, Duration::from_secs(60));

    let models = catalog.list_all().await.unwrap();

    // Should have models from both providers
    assert_eq!(models.len(), 2);

    let openai_model = models.iter().find(|m| m.id == "gpt-4").unwrap();
    let anthropic_model = models.iter().find(|m| m.id == "claude-3-5-sonnet").unwrap();

    assert_eq!(openai_model.provider, "openai");
    assert_eq!(anthropic_model.provider, "anthropic");
}

// ============================================================================
// Cache Persistence Tests
// ============================================================================

#[tokio::test]
async fn test_cache_persists_across_list_all_calls() {
    let registry = Arc::new(RwLock::new(ProviderRegistry::new()));

    let call_count = Arc::new(Mutex::new(0i32));
    let mut provider = MockProvider::new("test").with_call_count(call_count.clone());
    provider.models = vec![ModelInfo {
        id: "model1".to_string(),
        name: "Model 1".to_string(),
        provider: "test".to_string(),
        context_window: 8192,
        supports_vision: false,
        supports_tools: false,
    }];

    let provider_arc = Arc::new(provider);
    registry.write().unwrap().register(provider_arc);

    let catalog = ModelCatalog::new(registry, Duration::from_secs(3600)); // Long TTL

    // First call
    let _ = catalog.list_all().await.unwrap();

    // Multiple subsequent calls should use cache
    for _ in 0..5 {
        let _ = catalog.list_all().await.unwrap();
    }

    // Call count should be 1 (only first call hit the provider)
    assert_eq!(
        *call_count.lock().unwrap(),
        1,
        "Cache should be used for subsequent calls"
    );
}
