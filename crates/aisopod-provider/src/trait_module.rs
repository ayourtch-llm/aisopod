//! The ModelProvider trait and associated types.
//!
//! This module defines the [`ModelProvider`] trait, which is the primary
//! abstraction for communicating with AI model providers. All concrete
//! providers (Anthropic, OpenAI, Gemini, Bedrock, Ollama) implement this trait.

use anyhow::Result;
use async_trait::async_trait;
use futures_core::Stream;
use futures_util::{stream, stream::StreamExt};
use pin_project_lite::pin_project;
use std::pin::Pin;

use crate::types::*;

/// A pinned stream of chat completion chunks.
pub type ChatCompletionStream =
    Pin<Box<dyn Stream<Item = Result<ChatCompletionChunk>> + Send>>;

/// The primary trait for AI model providers.
///
/// Every concrete provider implementation must implement this trait,
/// providing a unified interface for:
/// - Listing available models
/// - Performing chat completions (with optional streaming)
/// - Checking provider health
///
/// # Example
///
/// ```ignore
/// use aisopod_provider::{ModelProvider, ChatCompletionRequest, Message, Role, MessageContent};
///
/// struct MyProvider {
///     api_key: String,
/// }
///
/// #[async_trait]
/// impl ModelProvider for MyProvider {
///     fn id(&self) -> &str {
///         "my-provider"
///     }
///
///     async fn list_models(&self) -> Result<Vec<ModelInfo>> {
///         // Fetch and return available models
///         # Ok(vec![])
///     }
///
///     async fn chat_completion(
///         &self,
///         request: ChatCompletionRequest,
///     ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatCompletionChunk>> + Send>>> {
///         // Perform chat completion and return a streaming response
///         # Ok(Box::pin(futures_util::stream::empty()))
///     }
///
///     async fn health_check(&self) -> Result<ProviderHealth> {
///         // Check if the provider is available
///         # Ok(ProviderHealth { available: true, latency_ms: None })
///     }
/// }
/// ```
#[async_trait]
pub trait ModelProvider: Send + Sync {
    /// Returns a unique identifier for this provider instance.
    ///
    /// This ID should be stable across requests and uniquely identify
    /// this specific provider configuration.
    fn id(&self) -> &str;

    /// Lists all models available from this provider.
    ///
    /// Returns a vector of [`ModelInfo`] containing information about
    /// each supported model, including context window size and capabilities.
    async fn list_models(&self) -> Result<Vec<ModelInfo>>;

    /// Performs a chat completion request.
    ///
    /// Sends a [`ChatCompletionRequest`] to the provider and returns
    /// a streaming response of [`ChatCompletionChunk`]s.
    ///
    /// The caller is responsible for consuming the stream and handling
    /// any errors that may occur during streaming.
    ///
    /// # Arguments
    ///
    /// * `request` - The chat completion request containing messages,
    ///   tools, and other parameters.
    ///
    /// # Returns
    ///
    /// A pinned boxed stream of chat completion chunks:
    /// `Pin<Box<dyn Stream<Item = Result<ChatCompletionChunk>> + Send>>`
    async fn chat_completion(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionStream>;

    /// Checks the health status of this provider.
    ///
    /// Returns a [`ProviderHealth`] indicating whether the provider
    /// is available and optionally the latency of a recent request.
    async fn health_check(&self) -> Result<ProviderHealth>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::stream;

    /// A mock provider implementation for testing.
    struct MockProvider {
        id: String,
    }

    impl MockProvider {
        fn new() -> Self {
            Self {
                id: "mock-provider".to_string(),
            }
        }
    }

    #[async_trait]
    impl ModelProvider for MockProvider {
        fn id(&self) -> &str {
            &self.id
        }

        async fn list_models(&self) -> Result<Vec<ModelInfo>> {
            Ok(vec![ModelInfo {
                id: "mock-model".to_string(),
                name: "Mock Model".to_string(),
                provider: "MockProvider".to_string(),
                context_window: 8192,
                supports_vision: false,
                supports_tools: false,
            }])
        }

        async fn chat_completion(
            &self,
            _request: ChatCompletionRequest,
        ) -> Result<ChatCompletionStream> {
            let chunks = vec![
                Ok(ChatCompletionChunk {
                    id: "chunk_1".to_string(),
                    delta: MessageDelta {
                        role: Some(Role::Assistant),
                        content: Some("Hello".to_string()),
                        tool_calls: None,
                    },
                    finish_reason: None,
                    usage: None,
                }),
                Ok(ChatCompletionChunk {
                    id: "chunk_2".to_string(),
                    delta: MessageDelta {
                        role: None,
                        content: Some(" world!".to_string()),
                        tool_calls: None,
                    },
                    finish_reason: Some(FinishReason::Stop),
                    usage: Some(TokenUsage {
                        prompt_tokens: 5,
                        completion_tokens: 3,
                        total_tokens: 8,
                    }),
                }),
            ];
            Ok(Box::pin(stream::iter(chunks)))
        }

        async fn health_check(&self) -> Result<ProviderHealth> {
            Ok(ProviderHealth {
                available: true,
                latency_ms: Some(100),
            })
        }
    }

    #[tokio::test]
    async fn test_provider_id() {
        let provider = MockProvider::new();
        assert_eq!(provider.id(), "mock-provider");
    }

    #[tokio::test]
    async fn test_list_models() {
        let provider = MockProvider::new();
        let models = provider.list_models().await.unwrap();
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].id, "mock-model");
    }

    #[tokio::test]
    async fn test_chat_completion() {
        let provider = MockProvider::new();
        let request = ChatCompletionRequest {
            model: "mock-model".to_string(),
            messages: vec![Message {
                role: Role::User,
                content: MessageContent::Text("Hello".to_string()),
                tool_calls: None,
                tool_call_id: None,
            }],
            tools: None,
            temperature: None,
            max_tokens: None,
            stop: None,
            stream: true,
        };
        let mut stream = provider.chat_completion(request).await.unwrap();
        let mut count = 0;
        while let Some(chunk) = stream.next().await {
            assert!(chunk.is_ok());
            count += 1;
        }
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_health_check() {
        let provider = MockProvider::new();
        let health = provider.health_check().await.unwrap();
        assert!(health.available);
        assert_eq!(health.latency_ms, Some(100));
    }
}
