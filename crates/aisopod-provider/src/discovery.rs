//! Model discovery and capability metadata system.
//!
//! This module provides a `ModelCatalog` that aggregates models from all
//! registered providers, caches the results with configurable TTL, and exposes
//! rich capability metadata through a unified interface.

use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use anyhow::Result;
use tracing::{error, info};

use crate::registry::ProviderRegistry;
use crate::types::ModelInfo;

/// A catalog of models aggregated from all registered providers.
///
/// The catalog caches model listings with a configurable TTL to avoid
/// redundant API calls. It provides methods to list all models, look up
/// a specific model by ID, and filter models by capability requirements.
pub struct ModelCatalog {
    registry: Arc<RwLock<ProviderRegistry>>,
    cache: RwLock<ModelCache>,
    cache_ttl: Duration,
}

struct ModelCache {
    models: Vec<ModelInfo>,
    last_refresh: Option<Instant>,
}

impl ModelCatalog {
    /// Creates a new `ModelCatalog` with an empty cache.
    ///
    /// # Arguments
    ///
    /// * `registry` - The provider registry to query for models.
    /// * `cache_ttl` - Time-to-live for cached models before refresh.
    pub fn new(registry: Arc<RwLock<ProviderRegistry>>, cache_ttl: Duration) -> Self {
        Self {
            registry,
            cache: RwLock::new(ModelCache {
                models: Vec::new(),
                last_refresh: None,
            }),
            cache_ttl,
        }
    }

    /// Refreshes the model cache by querying all providers.
    ///
    /// If a provider fails during refresh, the error is logged but other
    /// providers are still queried. Previously cached models are preserved.
    pub async fn refresh(&self) -> Result<()> {
        let providers = {
            let registry = self.registry.read().unwrap();
            registry.list_providers()
        };

        let mut new_models = Vec::new();
        let mut refresh_errors = Vec::new();

        for provider in providers {
            match provider.list_models().await {
                Ok(models) => {
                    new_models.extend(models);
                }
                Err(e) => {
                    error!(
                        "Failed to list models for provider '{}': {}",
                        provider.id(),
                        e
                    );
                    refresh_errors.push((provider.id().to_string(), e));
                }
            }
        }

        // Update cache with new models
        let mut cache = self.cache.write().unwrap();
        cache.models = new_models;
        cache.last_refresh = Some(Instant::now());

        if !refresh_errors.is_empty() {
            info!(
                "Refresh completed with {} errors ({} providers affected)",
                refresh_errors.len(),
                refresh_errors.len()
            );
        }

        Ok(())
    }

    /// Returns cached models, refreshing first if the cache is expired or empty.
    pub async fn list_all(&self) -> Result<Vec<ModelInfo>> {
        let cache = self.cache.read().unwrap();

        let should_refresh = match cache.last_refresh {
            Some(last) => last.elapsed() > self.cache_ttl,
            None => true,
        };
        drop(cache);

        if should_refresh {
            self.refresh().await?;
        }

        let cache = self.cache.read().unwrap();
        Ok(cache.models.clone())
    }

    /// Looks up a specific model by ID.
    pub async fn get_model(&self, model_id: &str) -> Result<Option<ModelInfo>> {
        let models = self.list_all().await?;
        Ok(models.into_iter().find(|m| m.id == model_id))
    }

    /// Filters models by capability requirements.
    ///
    /// # Arguments
    ///
    /// * `vision` - Optional requirement for vision support.
    /// * `tools` - Optional requirement for tool support.
    /// * `min_context` - Optional minimum context window size.
    pub async fn find_by_capability(
        &self,
        vision: Option<bool>,
        tools: Option<bool>,
        min_context: Option<u32>,
    ) -> Result<Vec<ModelInfo>> {
        let models = self.list_all().await?;

        Ok(models
            .into_iter()
            .filter(|model| {
                if let Some(v) = vision {
                    if model.supports_vision != v {
                        return false;
                    }
                }
                if let Some(t) = tools {
                    if model.supports_tools != t {
                        return false;
                    }
                }
                if let Some(min) = min_context {
                    if model.context_window < min {
                        return false;
                    }
                }
                true
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ChatCompletionChunk, ChatCompletionRequest, ChatCompletionStream, FinishReason, Message,
        MessageContent, MessageDelta, ModelProvider, ProviderHealth, Role, TokenUsage,
    };
    use async_trait::async_trait;
    use futures_core::Stream;
    use futures_util::stream::{self, BoxStream};
    use std::pin::Pin;

    /// A mock provider for testing that returns a fixed model list.
    struct MockProvider {
        id: String,
        models: Vec<ModelInfo>,
    }

    impl MockProvider {
        fn new(id: &str, models: Vec<ModelInfo>) -> Self {
            Self {
                id: id.to_string(),
                models,
            }
        }
    }

    #[async_trait]
    impl ModelProvider for MockProvider {
        fn id(&self) -> &str {
            &self.id
        }

        async fn list_models(&self) -> anyhow::Result<Vec<ModelInfo>> {
            Ok(self.models.clone())
        }

        async fn chat_completion(
            &self,
            _request: ChatCompletionRequest,
        ) -> anyhow::Result<ChatCompletionStream> {
            let stream = stream::iter(vec![Ok(ChatCompletionChunk {
                id: "test".to_string(),
                delta: MessageDelta {
                    role: Some(Role::Assistant),
                    content: Some("test".to_string()),
                    tool_calls: None,
                },
                finish_reason: Some(FinishReason::Stop),
                usage: Some(TokenUsage {
                    prompt_tokens: 10,
                    completion_tokens: 5,
                    total_tokens: 15,
                }),
            })]);
            Ok(Box::pin(stream) as ChatCompletionStream)
        }

        async fn health_check(&self) -> anyhow::Result<ProviderHealth> {
            Ok(ProviderHealth {
                available: true,
                latency_ms: Some(10),
            })
        }
    }

    fn create_test_catalog() -> (ModelCatalog, Arc<RwLock<ProviderRegistry>>) {
        let registry = Arc::new(RwLock::new(ProviderRegistry::new()));

        // Add mock providers
        let provider1 = MockProvider::new(
            "mock1",
            vec![
                ModelInfo {
                    id: "model1".to_string(),
                    name: "Model 1".to_string(),
                    provider: "mock1".to_string(),
                    context_window: 100000,
                    supports_vision: true,
                    supports_tools: true,
                },
                ModelInfo {
                    id: "model2".to_string(),
                    name: "Model 2".to_string(),
                    provider: "mock1".to_string(),
                    context_window: 50000,
                    supports_vision: false,
                    supports_tools: true,
                },
            ],
        );

        let provider2 = MockProvider::new(
            "mock2",
            vec![ModelInfo {
                id: "model3".to_string(),
                name: "Model 3".to_string(),
                provider: "mock2".to_string(),
                context_window: 200000,
                supports_vision: true,
                supports_tools: false,
            }],
        );

        {
            let mut reg = registry.write().unwrap();
            reg.register(Arc::new(provider1));
            reg.register(Arc::new(provider2));
        }

        let catalog = ModelCatalog::new(registry.clone(), Duration::from_secs(60));
        (catalog, registry)
    }

    #[tokio::test]
    async fn test_list_all_returns_models_from_all_providers() {
        let (catalog, _) = create_test_catalog();

        let models = catalog.list_all().await.unwrap();

        assert_eq!(models.len(), 3);
        assert!(models.iter().any(|m| m.id == "model1"));
        assert!(models.iter().any(|m| m.id == "model2"));
        assert!(models.iter().any(|m| m.id == "model3"));
    }

    #[tokio::test]
    async fn test_cache_is_respected_within_ttl() {
        let (catalog, _) = create_test_catalog();

        // First call populates cache
        let _ = catalog.list_all().await.unwrap();

        // Get the cached models
        let models1 = {
            let cache = catalog.cache.read().unwrap();
            cache.models.clone()
        };

        // Second call should use cache
        let models2 = catalog.list_all().await.unwrap();

        assert_eq!(models1, models2);
    }

    #[tokio::test]
    async fn test_refresh_forces_requery() {
        let (catalog, registry) = create_test_catalog();

        // Initial refresh
        catalog.refresh().await.unwrap();

        let models1 = {
            let cache = catalog.cache.read().unwrap();
            cache.models.clone()
        };

        // Add a new provider after initial refresh
        {
            let provider = MockProvider::new(
                "mock3",
                vec![ModelInfo {
                    id: "model4".to_string(),
                    name: "Model 4".to_string(),
                    provider: "mock3".to_string(),
                    context_window: 75000,
                    supports_vision: false,
                    supports_tools: false,
                }],
            );
            let mut reg = registry.write().unwrap();
            reg.register(Arc::new(provider));
        }

        // Force refresh
        catalog.refresh().await.unwrap();

        let models2 = {
            let cache = catalog.cache.read().unwrap();
            cache.models.clone()
        };

        assert!(models2.len() > models1.len());
        assert!(models2.iter().any(|m| m.id == "model4"));
    }

    #[tokio::test]
    async fn test_get_model_by_id() {
        let (catalog, _) = create_test_catalog();

        let model = catalog.get_model("model1").await.unwrap();
        assert!(model.is_some());
        assert_eq!(model.unwrap().id, "model1");

        let model = catalog.get_model("nonexistent").await.unwrap();
        assert!(model.is_none());
    }

    #[tokio::test]
    async fn test_find_by_capability() {
        let (catalog, _) = create_test_catalog();

        // Test vision filter
        let models = catalog
            .find_by_capability(Some(true), None, None)
            .await
            .unwrap();
        assert_eq!(models.len(), 2);
        assert!(models.iter().all(|m| m.supports_vision));

        // Test tools filter
        let models = catalog
            .find_by_capability(None, Some(true), None)
            .await
            .unwrap();
        assert_eq!(models.len(), 2);
        assert!(models.iter().all(|m| m.supports_tools));

        // Test min_context filter
        let models = catalog
            .find_by_capability(None, None, Some(75000))
            .await
            .unwrap();
        assert_eq!(models.len(), 2);
        assert!(models.iter().all(|m| m.context_window >= 75000));

        // Test combined filters
        let models = catalog
            .find_by_capability(Some(true), Some(true), None)
            .await
            .unwrap();
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].id, "model1");
    }

    #[tokio::test]
    async fn test_provider_error_during_refresh_is_handled() {
        let registry = Arc::new(RwLock::new(ProviderRegistry::new()));

        // Add a working provider
        let provider = MockProvider::new(
            "working",
            vec![ModelInfo {
                id: "working_model".to_string(),
                name: "Working Model".to_string(),
                provider: "working".to_string(),
                context_window: 100000,
                supports_vision: true,
                supports_tools: true,
            }],
        );

        {
            let mut reg = registry.write().unwrap();
            reg.register(Arc::new(provider));
        }

        // Add a failing provider (mocked to fail)
        let failing_provider = MockProvider::new("failing", vec![]);
        {
            let mut reg = registry.write().unwrap();
            reg.register(Arc::new(failing_provider));
        }

        let catalog = ModelCatalog::new(registry, Duration::from_secs(60));

        // This should complete despite one provider failing
        let models = catalog.list_all().await.unwrap();
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].id, "working_model");
    }
}
