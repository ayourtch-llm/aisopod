//! Test helpers for provider testing.
//!
//! This module provides mock infrastructure for testing provider implementations
//! without making real HTTP calls.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures_core::Stream;
use futures_util::stream::{self, StreamExt};
use pin_project_lite::pin_project;
use std::pin::Pin;

use aisopod_provider::trait_module::{ChatCompletionStream, ModelProvider};
use aisopod_provider::types::*;

/// Alias for Result with default error type
pub type Result<T, E = anyhow::Error> = std::result::Result<T, E>;

// ============================================================================
// Mock Provider Implementation
// ============================================================================

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

// ============================================================================
// Mock SSE Event Builder
// ============================================================================

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

// ============================================================================
// Test Model Helpers
// ============================================================================

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

// ============================================================================
// Mock Stream Creation Helpers
// ============================================================================

/// Creates a stream that yields chunks with specified content.
pub fn create_stream_with_chunks(chunks: Vec<&str>) -> Vec<Result<ChatCompletionChunk, String>> {
    let len = chunks.len();
    chunks
        .into_iter()
        .enumerate()
        .map(|(i, content)| {
            Ok(ChatCompletionChunk {
                id: format!("chunk_{}", i + 1),
                delta: MessageDelta {
                    role: if i == 0 { Some(Role::Assistant) } else { None },
                    content: Some(content.to_string()),
                    tool_calls: None,
                },
                finish_reason: if i == len - 1 {
                    Some(FinishReason::Stop)
                } else {
                    None
                },
                usage: if i == len - 1 {
                    Some(TokenUsage {
                        prompt_tokens: 5,
                        completion_tokens: i as u32 + 1,
                        total_tokens: i as u32 + 6,
                    })
                } else {
                    None
                },
            })
        })
        .collect()
}

/// Creates a stream with partial failures at specified indices.
pub fn create_stream_with_failures(
    chunks: Vec<&str>,
    failure_indices: Vec<usize>,
) -> Vec<Result<ChatCompletionChunk, anyhow::Error>> {
    let len = chunks.len();
    chunks
        .into_iter()
        .enumerate()
        .map(|(i, content)| {
            if failure_indices.contains(&i) {
                Err(anyhow::anyhow!("Chunk {} failed", i + 1))
            } else {
                Ok(ChatCompletionChunk {
                    id: format!("chunk_{}", i + 1),
                    delta: MessageDelta {
                        role: if i == 0 { Some(Role::Assistant) } else { None },
                        content: Some(content.to_string()),
                        tool_calls: None,
                    },
                    finish_reason: if i == len - 1 {
                        Some(FinishReason::Stop)
                    } else {
                        None
                    },
                    usage: None,
                })
            }
        })
        .collect()
}

// ============================================================================
// Mock Provider for Testing Multiple Scenarios
// ============================================================================

/// A mock provider that can be configured for different testing scenarios.
pub struct ConfigurableMockProvider {
    id: String,
    models: Vec<ModelInfo>,
    on_chat_completion:
        Mutex<Option<Box<dyn Fn(ChatCompletionRequest) -> ChatCompletionStream + Send + Sync>>>,
    on_list_models: Mutex<Option<Box<dyn Fn() -> Vec<ModelInfo> + Send + Sync>>>,
    on_health_check: Mutex<Option<Box<dyn Fn() -> ProviderHealth + Send + Sync>>>,
}

impl ConfigurableMockProvider {
    /// Creates a new configurable mock provider.
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            models: vec![create_test_model("test-model", "Test Model")],
            on_chat_completion: Mutex::new(None),
            on_list_models: Mutex::new(None),
            on_health_check: Mutex::new(None),
        }
    }

    /// Sets the handler for chat completion.
    pub fn with_chat_completion_handler<F>(mut self, handler: F) -> Self
    where
        F: Fn(ChatCompletionRequest) -> ChatCompletionStream + Send + Sync + 'static,
    {
        *self.on_chat_completion.lock().unwrap() = Some(Box::new(handler));
        self
    }

    /// Sets the handler for list_models.
    pub fn with_list_models_handler<F>(mut self, handler: F) -> Self
    where
        F: Fn() -> Vec<ModelInfo> + Send + Sync + 'static,
    {
        *self.on_list_models.lock().unwrap() = Some(Box::new(handler));
        self
    }

    /// Sets the handler for health_check.
    pub fn with_health_check_handler<F>(mut self, handler: F) -> Self
    where
        F: Fn() -> ProviderHealth + Send + Sync + 'static,
    {
        *self.on_health_check.lock().unwrap() = Some(Box::new(handler));
        self
    }

    /// Creates a streaming response.
    fn create_stream(
        &self,
        chunks: Vec<Result<ChatCompletionChunk, String>>,
    ) -> ChatCompletionStream {
        let stream = async_stream::stream! {
            for chunk in chunks {
                yield chunk.map_err(|e| anyhow::anyhow!(e));
            }
        };
        Box::pin(stream)
    }
}

#[async_trait::async_trait]
impl ModelProvider for ConfigurableMockProvider {
    fn id(&self) -> &str {
        &self.id
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>, anyhow::Error> {
        if let Some(handler) = self.on_list_models.lock().unwrap().as_ref() {
            Ok(handler())
        } else {
            Ok(self.models.clone())
        }
    }

    async fn chat_completion(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionStream> {
        if let Some(handler) = self.on_chat_completion.lock().unwrap().as_ref() {
            Ok(handler(request))
        } else {
            // Default implementation
            Ok(self.create_stream(vec![Ok(ChatCompletionChunk {
                id: "chunk_1".to_string(),
                delta: MessageDelta {
                    role: Some(Role::Assistant),
                    content: Some("Default response".to_string()),
                    tool_calls: None,
                },
                finish_reason: Some(FinishReason::Stop),
                usage: Some(TokenUsage {
                    prompt_tokens: 1,
                    completion_tokens: 2,
                    total_tokens: 3,
                }),
            })]))
        }
    }

    async fn health_check(&self) -> Result<ProviderHealth, anyhow::Error> {
        if let Some(handler) = self.on_health_check.lock().unwrap().as_ref() {
            Ok(handler())
        } else {
            Ok(ProviderHealth {
                available: true,
                latency_ms: Some(10),
            })
        }
    }
}

// ============================================================================
// Stream Verification Helpers
// ============================================================================

/// Verifies that a stream contains the expected content in order.
pub async fn verify_stream_content(
    mut stream: ChatCompletionStream,
    expected_content: Vec<&str>,
) -> Result<()> {
    let mut received_chunks: Vec<String> = Vec::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        if let Some(content) = &chunk.delta.content {
            received_chunks.push(content.clone());
        }
    }

    assert_eq!(
        received_chunks.len(),
        expected_content.len(),
        "Expected {} chunks, got {}",
        expected_content.len(),
        received_chunks.len()
    );

    for (i, expected) in expected_content.iter().enumerate() {
        assert_eq!(
            received_chunks[i], *expected,
            "Chunk {} mismatch: expected '{}', got '{}'",
            i, expected, received_chunks[i]
        );
    }

    Ok(())
}

/// Verifies that a stream terminates with the expected finish reason.
pub async fn verify_stream_termination(
    mut stream: ChatCompletionStream,
    expected_finish_reason: FinishReason,
) -> Result<()> {
    let mut last_finish_reason = None;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        last_finish_reason = chunk.finish_reason;
    }

    assert_eq!(
        last_finish_reason,
        Some(expected_finish_reason.clone()),
        "Expected finish reason {:?}, got {:?}",
        expected_finish_reason,
        last_finish_reason
    );

    Ok(())
}

// ============================================================================
// Tool Call Helpers
// ============================================================================

/// Creates a test tool call for testing.
pub fn create_test_tool_call_with_args(id: &str, name: &str, args: serde_json::Value) -> ToolCall {
    ToolCall {
        id: id.to_string(),
        name: name.to_string(),
        arguments: serde_json::to_string(&args).unwrap_or_default(),
    }
}

/// Creates a stream with tool calls.
pub fn create_tool_call_stream(
    tool_call_id: &str,
    tool_name: &str,
    tool_args: &str,
    finish_reason: FinishReason,
) -> Vec<Result<ChatCompletionChunk, String>> {
    vec![
        Ok(ChatCompletionChunk {
            id: "tool_chunk_1".to_string(),
            delta: MessageDelta {
                role: Some(Role::Assistant),
                content: None,
                tool_calls: Some(vec![ToolCall {
                    id: tool_call_id.to_string(),
                    name: tool_name.to_string(),
                    arguments: tool_args.to_string(),
                }]),
            },
            finish_reason: None,
            usage: None,
        }),
        Ok(ChatCompletionChunk {
            id: "tool_chunk_2".to_string(),
            delta: MessageDelta {
                role: None,
                content: None,
                tool_calls: None,
            },
            finish_reason: Some(finish_reason),
            usage: Some(TokenUsage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            }),
        }),
    ]
}
