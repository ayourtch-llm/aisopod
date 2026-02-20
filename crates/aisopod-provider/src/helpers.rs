#![allow(clippy::all)]
//! Test helpers for provider testing.
//!
//! This module provides mock infrastructure for testing provider implementations
//! without making real HTTP calls.

use std::time::Duration;

use crate::trait_module::{ChatCompletionStream, ModelProvider};
use crate::types::*;

/// Alias for Result with default error type
pub type Result<T, E = anyhow::Error> = std::result::Result<T, E>;

/// A mock provider implementation for testing without real HTTP calls.
///
/// This provider simulates various behaviors through configuration:
/// - Success responses with configurable chunks
/// - Error responses
/// - Delayed responses
/// - Custom model lists
pub struct MockProvider {
    id: String,
    models: Vec<ModelInfo>,
    stream_chunks: Vec<Result<ChatCompletionChunk, String>>,
    stream_delay_ms: Option<u64>,
    should_fail: bool,
    error_message: Option<String>,
}

impl MockProvider {
    /// Creates a new mock provider with default success responses.
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            models: vec![ModelInfo {
                id: "mock-model".to_string(),
                name: "Mock Model".to_string(),
                provider: id.to_string(),
                context_window: 8192,
                supports_vision: false,
                supports_tools: false,
            }],
            stream_chunks: vec![
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
            ],
            stream_delay_ms: None,
            should_fail: false,
            error_message: None,
        }
    }

    /// Sets the mock provider to return a specific model list.
    pub fn with_models(mut self, models: Vec<ModelInfo>) -> Self {
        self.models = models;
        self
    }

    /// Sets custom stream chunks for testing.
    pub fn with_chunks(mut self, chunks: Vec<Result<ChatCompletionChunk, String>>) -> Self {
        self.stream_chunks = chunks;
        self
    }

    /// Adds a delay to chunk responses (for testing streaming timing).
    pub fn with_delay_ms(mut self, delay_ms: u64) -> Self {
        self.stream_delay_ms = Some(delay_ms);
        self
    }

    /// Configures the mock to fail with an error.
    pub fn with_error(mut self, error_message: &str) -> Self {
        self.should_fail = true;
        self.error_message = Some(error_message.to_string());
        self
    }

    /// Creates a streaming response with optional delays.
    fn create_stream(&self) -> ChatCompletionStream {
        let chunks = self.stream_chunks.clone();
        let delay_ms = self.stream_delay_ms;

        let stream = async_stream::stream! {
            for (i, chunk) in chunks.into_iter().enumerate() {
                if let Some(delay) = delay_ms {
                    tokio::time::sleep(Duration::from_millis(delay)).await;
                }
                match chunk {
                    Ok(chunk) => yield Ok(chunk),
                    Err(msg) => yield Err(anyhow::anyhow!(msg)),
                }
            }
        };

        Box::pin(stream)
    }
}

#[async_trait::async_trait]
impl ModelProvider for MockProvider {
    fn id(&self) -> &str {
        &self.id
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>, anyhow::Error> {
        Ok(self.models.clone())
    }

    async fn chat_completion(
        &self,
        _request: ChatCompletionRequest,
    ) -> Result<ChatCompletionStream> {
        if self.should_fail {
            return Err(anyhow::anyhow!(self
                .error_message
                .clone()
                .unwrap_or_else(|| "Mock error".to_string())));
        }
        Ok(self.create_stream())
    }

    async fn health_check(&self) -> Result<ProviderHealth, anyhow::Error> {
        Ok(ProviderHealth {
            available: !self.should_fail,
            latency_ms: Some(10),
        })
    }
}

/// A mock SSE event builder for testing.
pub struct MockSseEvent {
    event_type: String,
    data: String,
}

impl MockSseEvent {
    /// Creates a new SSE event.
    pub fn new(event_type: &str, data: &str) -> Self {
        Self {
            event_type: event_type.to_string(),
            data: data.to_string(),
        }
    }

    /// Converts the event to an SSE string format.
    pub fn to_sse_string(&self) -> String {
        format!("event: {}\ndata: {}\n\n", self.event_type, self.data)
    }
}

/// Helper function to create a test model info.
pub fn create_test_model(id: &str, name: &str) -> ModelInfo {
    ModelInfo {
        id: id.to_string(),
        name: name.to_string(),
        provider: "test-provider".to_string(),
        context_window: 8192,
        supports_vision: false,
        supports_tools: false,
    }
}

/// Helper function to create a test tool definition.
pub fn create_test_tool() -> ToolDefinition {
    ToolDefinition {
        name: "calculator".to_string(),
        description: "A simple calculator".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "operation": {"type": "string"},
                "a": {"type": "number"},
                "b": {"type": "number"}
            }
        }),
    }
}

/// Helper function to create a test tool call.
pub fn create_test_tool_call(id: &str, name: &str, arguments: &str) -> ToolCall {
    ToolCall {
        id: id.to_string(),
        name: name.to_string(),
        arguments: arguments.to_string(),
    }
}

/// Create a test request for provider testing
pub fn create_test_request(model: &str, content: &str) -> ChatCompletionRequest {
    ChatCompletionRequest {
        model: model.to_string(),
        messages: vec![Message {
            role: Role::User,
            content: MessageContent::Text(content.to_string()),
            tool_calls: None,
            tool_call_id: None,
        }],
        tools: None,
        temperature: None,
        max_tokens: None,
        stop: None,
        stream: true,
    }
}
