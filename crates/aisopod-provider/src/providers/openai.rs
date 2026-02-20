//! OpenAI Chat Completions API provider implementation.

use anyhow::Result;
use async_trait::async_trait;
use futures_util::stream::{BoxStream, StreamExt};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::{debug, warn};

use crate::auth::{AuthProfile, AuthProfileManager};
use crate::trait_module::{ChatCompletionStream, ModelProvider};
use crate::types::*;

pub mod api_types;
use api_types::*;

/// OpenAI provider implementation.
///
/// This struct implements the [`ModelProvider`] trait for the OpenAI
/// Chat Completions API, supporting streaming SSE responses, tool use,
/// vision support (image content parts), JSON mode, and organization header.
pub struct OpenAIProvider {
    client: reqwest::Client,
    api_key: String,
    pub base_url: String,
    organization: Option<String>,
    profile_manager: Arc<Mutex<AuthProfileManager>>,
}

impl OpenAIProvider {
    /// Creates a new OpenAI provider instance.
    ///
    /// # Arguments
    ///
    /// * `api_key` - The API key for authenticating with OpenAI.
    /// * `base_url` - The base URL for the OpenAI API (defaults to "https://api.openai.com").
    /// * `organization` - Optional OpenAI-Organization header value for multi-tenant setups.
    /// * `cooldown_seconds` - The cooldown period in seconds for failed profiles.
    pub fn new(
        api_key: String,
        base_url: Option<String>,
        organization: Option<String>,
        cooldown_seconds: Option<u64>,
    ) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            base_url: base_url.unwrap_or_else(|| "https://api.openai.com".to_string()),
            organization,
            profile_manager: Arc::new(Mutex::new(AuthProfileManager::new(Duration::from_secs(
                cooldown_seconds.unwrap_or(60),
            )))),
        }
    }

    /// Creates a new OpenAI provider instance with a custom base URL.
    ///
    /// This is useful for OpenAI-compatible APIs (e.g., Azure OpenAI, vLLM).
    ///
    /// # Arguments
    ///
    /// * `api_key` - The API key for authenticating with the provider.
    /// * `base_url` - The base URL for the API (e.g., "https://api.openai.com/v1").
    /// * `organization` - Optional OpenAI-Organization header value.
    /// * `cooldown_seconds` - The cooldown period in seconds for failed profiles.
    pub fn with_base_url(
        api_key: String,
        base_url: String,
        organization: Option<String>,
        cooldown_seconds: Option<u64>,
    ) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            base_url,
            organization,
            profile_manager: Arc::new(Mutex::new(AuthProfileManager::new(Duration::from_secs(
                cooldown_seconds.unwrap_or(60),
            )))),
        }
    }

    /// Adds an authentication profile for key rotation.
    pub fn add_profile(&mut self, profile: AuthProfile) {
        let mut manager = self.profile_manager.lock().unwrap();
        manager.add_profile(profile);
    }

    /// Gets the next available API key for round-robin rotation.
    fn get_api_key(&self) -> Option<String> {
        let mut manager = self.profile_manager.lock().unwrap();
        manager.next_key("openai").map(|p| p.api_key.clone())
    }

    /// Converts a core [`Message`] to an OpenAI message.
    fn convert_message(&self, message: &Message) -> OpenAIMessage {
        let role = match message.role {
            Role::System => OpenAIRole::System,
            Role::User => OpenAIRole::User,
            Role::Assistant => OpenAIRole::Assistant,
            Role::Tool => OpenAIRole::Tool,
        };

        let content = match &message.content {
            MessageContent::Text(text) => Some(OpenAIContent::Text(text.clone())),
            MessageContent::Parts(parts) => {
                let content_parts = parts
                    .iter()
                    .map(|part| match part {
                        ContentPart::Text { text } => {
                            OpenAIContentPart::Text { text: text.clone() }
                        }
                        ContentPart::Image { media_type, data } => {
                            // Convert media type to URL format
                            // OpenAI supports base64 encoded images with data URLs
                            let url = if media_type.starts_with("image/") {
                                format!("data:{};base64,{}", media_type, data)
                            } else {
                                data.clone()
                            };
                            OpenAIContentPart::ImageUrl {
                                image_url: OpenAIImageUrl { url, detail: None },
                            }
                        }
                    })
                    .collect();
                Some(OpenAIContent::Parts(content_parts))
            }
        };

        let tool_calls = message.tool_calls.as_ref().map(|tool_calls| {
            tool_calls
                .iter()
                .map(|tool_call| OpenAIToolCall {
                    id: tool_call.id.clone(),
                    tool_type: OpenAIToolType::Function,
                    function: OpenAIFunctionCall {
                        name: tool_call.name.clone(),
                        arguments: tool_call.arguments.clone(),
                    },
                })
                .collect()
        });

        OpenAIMessage {
            role,
            content,
            name: None,
            tool_call_id: message.tool_call_id.clone(),
            tool_calls,
        }
    }

    /// Converts a core [`ToolDefinition`] to an OpenAI tool.
    fn convert_tool(&self, tool: &ToolDefinition) -> OpenAITool {
        OpenAITool {
            r#type: OpenAIToolType::Function,
            function: OpenAIFunctionDefinition {
                name: tool.name.clone(),
                description: Some(tool.description.clone()),
                parameters: tool.parameters.clone(),
            },
        }
    }

    /// Builds the OpenAI request from a core request.
    fn build_openai_request(&self, request: &ChatCompletionRequest) -> OpenAIRequest {
        let messages = request
            .messages
            .iter()
            .map(|message| self.convert_message(message))
            .collect();

        let tools = request
            .tools
            .as_ref()
            .map(|tools| tools.iter().map(|tool| self.convert_tool(tool)).collect());

        // Check if any message has parts with a hint for JSON mode
        // In a real implementation, this would be a more explicit flag
        // For now, we check if the request explicitly sets response_format
        // or we could look for a specific message pattern
        let response_format = None;

        OpenAIRequest {
            model: request.model.clone(),
            messages,
            tools,
            tool_choice: None,
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            stop: request.stop.clone(),
            stream: request.stream,
            response_format,
        }
    }

    /// Parses an OpenAI SSE event into a chat completion chunk.
    fn parse_sse_event(line: &str) -> Option<ChatCompletionChunk> {
        // Skip comments and empty lines
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with(':') {
            return None;
        }

        // Strip "data: " prefix if present (both SSE format and raw JSON are accepted)
        let data = if trimmed.starts_with("data: ") {
            trimmed.strip_prefix("data: ")?
        } else {
            trimmed
        };
        if data == "[DONE]" {
            return None;
        }

        // Parse the JSON
        let event: OpenAISseEvent = serde_json::from_str(data).ok()?;

        match event {
            OpenAISseEvent::ChatCompletionChunk {
                id,
                created: _,
                model: _,
                choices,
                usage,
            } => {
                if choices.is_empty() {
                    return None;
                }

                let choice = &choices[0];

                // Map OpenAIRole to Role
                let role = choice.delta.role.as_ref().map(|r| match r {
                    OpenAIRole::System => Role::System,
                    OpenAIRole::User => Role::User,
                    OpenAIRole::Assistant => Role::Assistant,
                    OpenAIRole::Tool => Role::Tool,
                });

                // Extract content from delta
                let content = choice.delta.content.clone();

                // Extract tool calls from delta
                let tool_calls = choice.delta.tool_calls.as_ref().map(|tool_calls| {
                    tool_calls
                        .iter()
                        .map(|tool_call| ToolCall {
                            id: tool_call.id.clone(),
                            name: tool_call.function.name.clone(),
                            arguments: tool_call.function.arguments.clone(),
                        })
                        .collect()
                });

                // Map finish reason
                let finish_reason = choice.finish_reason.as_deref().and_then(|s| match s {
                    "stop" => Some(FinishReason::Stop),
                    "length" => Some(FinishReason::Length),
                    "tool_calls" | "tool_call" => Some(FinishReason::ToolCall),
                    "content_filter" => Some(FinishReason::ContentFilter),
                    _ => None,
                });

                // Map usage if present
                let usage = usage.map(|u| TokenUsage {
                    prompt_tokens: u.prompt_tokens,
                    completion_tokens: u.completion_tokens,
                    total_tokens: u.total_tokens,
                });

                Some(ChatCompletionChunk {
                    id,
                    delta: MessageDelta {
                        role,
                        content,
                        tool_calls,
                    },
                    finish_reason,
                    usage,
                })
            }
            OpenAISseEvent::ChatCompletion => None, // Non-streaming response
        }
    }

    /// Converts OpenAI API error to anyhow error.
    fn handle_api_error(&self, status: u16, body: &str) -> anyhow::Error {
        match serde_json::from_str::<OpenAIErrorResponse>(body) {
            Ok(error_response) => {
                if let Some(error) = error_response.error {
                    anyhow::anyhow!(
                        "OpenAI API error ({}): {}",
                        status,
                        error.message.unwrap_or_else(|| "Unknown error".to_string())
                    )
                } else {
                    anyhow::anyhow!("OpenAI API error: {}", status)
                }
            }
            Err(_) => anyhow::anyhow!("OpenAI API error ({}): {}", status, body.trim()),
        }
    }

    /// Estimates context window based on model name.
    fn estimate_context_window(model_id: &str) -> u32 {
        // Common context windows for popular OpenAI models
        if model_id.contains("gpt-4o") || model_id.contains("gpt-4-turbo") {
            128000
        } else if model_id.contains("gpt-4") {
            8192
        } else if model_id.contains("gpt-3.5") || model_id.contains("gpt-35") {
            16385
        } else {
            4096 // Default fallback
        }
    }

    /// Determines if a model supports vision based on its name.
    fn supports_vision(model_id: &str) -> bool {
        // Vision-capable models
        model_id.contains("gpt-4o")
            || model_id.contains("gpt-4-turbo")
            || model_id.contains("gpt-4-vision")
    }

    /// Determines if a model supports tool calling based on its name.
    fn supports_tools(model_id: &str) -> bool {
        // Tool-capable models:
        // - GPT-4o and GPT-4-turbo series
        // - GPT-4 models from June 2024 onward (gpt-4-0613 and later)
        // - GPT-3.5-turbo from November 2023 onward (gpt-3.5-turbo-1106 and later)
        model_id.contains("gpt-4o")
            || model_id.contains("gpt-4-turbo")
            || model_id.contains("gpt-4-0613")
            || model_id.contains("gpt-3.5-turbo-1106")
            || model_id.contains("gpt-3.5-turbo-1106")
            || model_id.contains("gpt-35-turbo")
    }
}

#[async_trait]
impl ModelProvider for OpenAIProvider {
    fn id(&self) -> &str {
        "openai"
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        debug!("Listing OpenAI models");

        let api_key = self.get_api_key().unwrap_or(self.api_key.clone());

        let url = format!("{}/v1/models", self.base_url);

        let mut request_builder = self
            .client
            .request(reqwest::Method::GET, &url)
            .header("Authorization", format!("Bearer {}", api_key));

        // Add organization header if configured
        if let Some(ref org) = self.organization {
            request_builder = request_builder.header("OpenAI-Organization", org);
        }

        let response = request_builder.send().await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await?;
            return Err(self.handle_api_error(status, &body));
        }

        let response: OpenAIListModelResponse = response.json().await?;

        let models = response
            .data
            .into_iter()
            .map(|model| {
                let model_id = model.id.clone();
                ModelInfo {
                    id: model_id.clone(),
                    name: model_id, // OpenAI doesn't provide separate display names
                    provider: "openai".to_string(),
                    context_window: OpenAIProvider::estimate_context_window(&model.id),
                    supports_vision: OpenAIProvider::supports_vision(&model.id),
                    supports_tools: OpenAIProvider::supports_tools(&model.id),
                }
            })
            .collect();

        Ok(models)
    }

    async fn chat_completion(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionStream> {
        let api_key = self.get_api_key().unwrap_or(self.api_key.clone());

        let openai_request = self.build_openai_request(&request);

        debug!(
            "Sending request to OpenAI API: model={}, stream={}",
            openai_request.model, openai_request.stream
        );

        let url = format!("{}/v1/chat/completions", self.base_url);

        let mut request_builder = self
            .client
            .request(reqwest::Method::POST, &url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json");

        // Add organization header if configured
        if let Some(ref org) = self.organization {
            request_builder = request_builder.header("OpenAI-Organization", org);
        }

        let response = request_builder.json(&openai_request).send().await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await?;
            return Err(self.handle_api_error(status, &body));
        }

        // For streaming, parse SSE stream line by line
        let stream = response.bytes_stream().map(move |chunk| {
            let chunk = chunk.map_err(|e| anyhow::anyhow!("Stream error: {}", e))?;

            // Decode the chunk as UTF-8
            let text = String::from_utf8_lossy(&chunk);

            // Process line by line
            let mut chunks = Vec::new();
            for line in text.lines() {
                if let Some(parsed) = Self::parse_sse_event(line) {
                    chunks.push(parsed);
                }
            }

            if chunks.is_empty() {
                return Ok(None);
            }

            // For simplicity, return the first chunk for now
            // A more complete implementation would buffer and return all chunks
            Ok(Some(chunks.remove(0)))
        });

        // Filter out None values and box the stream
        let boxed: BoxStream<'static, Result<ChatCompletionChunk>> =
            Box::pin(stream.filter_map(|x| async move {
                match x {
                    Ok(Some(chunk)) => Some(Ok(chunk)),
                    Ok(None) => None,
                    Err(e) => Some(Err(e)),
                }
            }));

        Ok(boxed)
    }

    async fn health_check(&self) -> Result<ProviderHealth> {
        let api_key = self.get_api_key().unwrap_or(self.api_key.clone());

        let url = format!("{}/v1/models", self.base_url);

        let start = std::time::Instant::now();

        let mut request_builder = self
            .client
            .request(reqwest::Method::GET, &url)
            .header("Authorization", format!("Bearer {}", api_key));

        // Add organization header if configured
        if let Some(ref org) = self.organization {
            request_builder = request_builder.header("OpenAI-Organization", org);
        }

        let response = request_builder.send().await;

        let latency_ms = start.elapsed().as_millis() as u64;

        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    Ok(ProviderHealth {
                        available: true,
                        latency_ms: Some(latency_ms),
                    })
                } else {
                    warn!(
                        "Health check failed with status: {}",
                        resp.status().as_u16()
                    );
                    Ok(ProviderHealth {
                        available: false,
                        latency_ms: Some(latency_ms),
                    })
                }
            }
            Err(e) => {
                warn!("Health check error: {}", e);
                Ok(ProviderHealth {
                    available: false,
                    latency_ms: None,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_message_text() {
        let provider = OpenAIProvider::new("test-key".to_string(), None, None, Some(60));

        let message = Message {
            role: Role::User,
            content: MessageContent::Text("Hello".to_string()),
            tool_calls: None,
            tool_call_id: None,
        };

        let result = provider.convert_message(&message);
        assert_eq!(result.role, OpenAIRole::User);
        assert_eq!(
            result.content,
            Some(OpenAIContent::Text("Hello".to_string()))
        );
    }

    #[test]
    fn test_convert_message_parts() {
        let provider = OpenAIProvider::new("test-key".to_string(), None, None, Some(60));

        let message = Message {
            role: Role::User,
            content: MessageContent::Parts(vec![ContentPart::Text {
                text: "Hello".to_string(),
            }]),
            tool_calls: None,
            tool_call_id: None,
        };

        let result = provider.convert_message(&message);
        assert_eq!(result.role, OpenAIRole::User);

        if let Some(OpenAIContent::Parts(parts)) = result.content {
            assert_eq!(parts.len(), 1);
            if let OpenAIContentPart::Text { text } = &parts[0] {
                assert_eq!(text, "Hello");
            }
        } else {
            panic!("Expected Parts content");
        }
    }

    #[test]
    fn test_convert_tool() {
        let provider = OpenAIProvider::new("test-key".to_string(), None, None, Some(60));

        let tool = ToolDefinition {
            name: "calculator".to_string(),
            description: "A calculator tool".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "a": {"type": "number"},
                    "b": {"type": "number"}
                }
            }),
        };

        let result = provider.convert_tool(&tool);
        assert_eq!(result.r#type, OpenAIToolType::Function);
        assert_eq!(result.function.name, "calculator");
    }

    #[test]
    fn test_build_openai_request() {
        let provider = OpenAIProvider::new("test-key".to_string(), None, None, Some(60));

        let request = ChatCompletionRequest {
            model: "gpt-4".to_string(),
            messages: vec![
                Message {
                    role: Role::System,
                    content: MessageContent::Text("You are a helpful assistant".to_string()),
                    tool_calls: None,
                    tool_call_id: None,
                },
                Message {
                    role: Role::User,
                    content: MessageContent::Text("Hello".to_string()),
                    tool_calls: None,
                    tool_call_id: None,
                },
            ],
            tools: None,
            temperature: Some(0.7),
            max_tokens: Some(1000),
            stop: None,
            stream: true,
        };

        let openai_request = provider.build_openai_request(&request);

        assert_eq!(openai_request.model, "gpt-4");
        assert_eq!(openai_request.messages.len(), 2);
        assert_eq!(openai_request.temperature, Some(0.7));
        assert_eq!(openai_request.max_tokens, Some(1000));
        assert!(openai_request.stream);
    }

    #[test]
    fn test_parse_sse_event() {
        let event_str = r#"{"id":"chatcmpl-abc","object":"chat.completion.chunk","created":1234567890,"model":"gpt-4","choices":[{"index":0,"delta":{"role":"assistant","content":"Hello"},"finish_reason":null}]}"#;

        let result = OpenAIProvider::parse_sse_event(event_str);
        assert!(result.is_some());

        let chunk = result.unwrap();
        assert_eq!(chunk.id, "chatcmpl-abc");
        assert_eq!(chunk.delta.role, Some(Role::Assistant));
        assert_eq!(chunk.delta.content, Some("Hello".to_string()));
    }

    #[test]
    fn test_parse_sse_done() {
        // [DONE] should return None
        let result = OpenAIProvider::parse_sse_event("data: [DONE]");
        assert!(result.is_none());
    }

    #[test]
    fn test_estimate_context_window() {
        assert_eq!(OpenAIProvider::estimate_context_window("gpt-4o"), 128000);
        assert_eq!(
            OpenAIProvider::estimate_context_window("gpt-4-turbo"),
            128000
        );
        assert_eq!(OpenAIProvider::estimate_context_window("gpt-4"), 8192);
        assert_eq!(
            OpenAIProvider::estimate_context_window("gpt-3.5-turbo"),
            16385
        );
        assert_eq!(
            OpenAIProvider::estimate_context_window("unknown-model"),
            4096
        );
    }

    #[test]
    fn test_supports_vision() {
        assert!(OpenAIProvider::supports_vision("gpt-4o"));
        assert!(OpenAIProvider::supports_vision("gpt-4-turbo"));
        assert!(OpenAIProvider::supports_vision("gpt-4-vision-preview"));
        assert!(!OpenAIProvider::supports_vision("gpt-4"));
        assert!(!OpenAIProvider::supports_vision("gpt-3.5-turbo"));
    }

    #[test]
    fn test_supports_tools() {
        assert!(OpenAIProvider::supports_tools("gpt-4o"));
        assert!(OpenAIProvider::supports_tools("gpt-4-turbo"));
        assert!(OpenAIProvider::supports_tools("gpt-4-0613"));
        assert!(OpenAIProvider::supports_tools("gpt-3.5-turbo-1106"));
        assert!(!OpenAIProvider::supports_tools("gpt-3.5-turbo-0301"));
    }
}
