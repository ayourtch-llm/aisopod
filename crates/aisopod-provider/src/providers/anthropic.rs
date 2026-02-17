//! Anthropic Claude Messages API provider implementation.

use anyhow::Result;
use async_trait::async_trait;
use futures_core::Stream;
use futures_util::stream::{self, BoxStream, StreamExt};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::{debug, instrument, warn};

use crate::auth::{AuthProfile, AuthProfileManager};
use crate::trait_module::{ChatCompletionStream, ModelProvider};
use crate::types::*;

mod api_types;
use api_types::*;

/// Anthropic Claude provider implementation.
///
/// This struct implements the [`ModelProvider`] trait for the Anthropic
/// Messages API, supporting streaming SSE chat completions, tool use,
/// system prompt handling, and vision (image) support.
pub struct AnthropicProvider {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    default_model: String,
    profile_manager: Arc<Mutex<AuthProfileManager>>,
}

impl AnthropicProvider {
    /// Creates a new Anthropic provider instance.
    ///
    /// # Arguments
    ///
    /// * `api_key` - The API key for authenticating with Anthropic.
    /// * `base_url` - The base URL for the Anthropic API (defaults to "https://api.anthropic.com").
    /// * `default_model` - The default model ID to use for requests.
    /// * `cooldown_seconds` - The cooldown period in seconds for failed profiles.
    pub fn new(
        api_key: String,
        base_url: Option<String>,
        default_model: Option<String>,
        cooldown_seconds: Option<u64>,
    ) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            base_url: base_url.unwrap_or_else(|| "https://api.anthropic.com".to_string()),
            default_model: default_model.unwrap_or_else(|| "claude-3-5-sonnet-latest".to_string()),
            profile_manager: Arc::new(Mutex::new(AuthProfileManager::new(
                Duration::from_secs(cooldown_seconds.unwrap_or(60)),
            ))),
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
        manager
            .next_key("anthropic")
            .map(|p| p.api_key.clone())
    }

    /// Converts a core [`Message`] to an Anthropic message.
    fn convert_message(&self, message: &Message) -> Option<AnthropicMessage> {
        let role = match message.role {
            Role::System => return None, // System messages are handled separately
            Role::User => AnthropicRole::User,
            Role::Assistant => AnthropicRole::Assistant,
            Role::Tool => AnthropicRole::Assistant, // Tool results are sent as assistant messages
        };

        let content = match &message.content {
            MessageContent::Text(text) => {
                vec![AnthropicContentBlock::Text {
                    text: text.clone(),
                }]
            }
            MessageContent::Parts(parts) => {
                let mut content_blocks = Vec::new();
                for part in parts {
                    match part {
                        ContentPart::Text { text } => {
                            content_blocks.push(AnthropicContentBlock::Text {
                                text: text.clone(),
                            });
                        }
                        ContentPart::Image { media_type, data } => {
                            content_blocks.push(AnthropicContentBlock::Image {
                                source: AnthropicImageSource {
                                    r#type: "base64".to_string(),
                                    media_type: media_type.clone(),
                                    data: data.clone(),
                                },
                                _type: "image".to_string(),
                            });
                        }
                    }
                }
                content_blocks
            }
        };

        Some(AnthropicMessage {
            role,
            content,
        })
    }

    /// Converts a core [`ToolDefinition`] to an Anthropic tool.
    fn convert_tool(&self, tool: &ToolDefinition) -> AnthropicTool {
        AnthropicTool {
            name: tool.name.clone(),
            description: tool.description.clone(),
            input_schema: tool.parameters.clone(),
        }
    }

    /// Extracts system prompt from messages if present.
    fn extract_system_prompt(&self, messages: &[Message]) -> Option<String> {
        for message in messages {
            if message.role == Role::System {
                if let MessageContent::Text(text) = &message.content {
                    return Some(text.clone());
                }
            }
        }
        None
    }

    /// Builds the Anthropic request from a core request.
    fn build_anthropic_request(
        &self,
        request: &ChatCompletionRequest,
    ) -> AnthropicRequest {
        let system_prompt = self.extract_system_prompt(&request.messages);

        let mut anthropic_messages = Vec::new();
        for message in &request.messages {
            if message.role == Role::System {
                continue; // Handled separately
            }
            if let Some(anthropic_msg) = self.convert_message(message) {
                anthropic_messages.push(anthropic_msg);
            }
        }

        let tools = request.tools.as_ref().map(|tools| {
            tools.iter().map(|tool| self.convert_tool(tool)).collect()
        });

        AnthropicRequest {
            model: request.model.clone(),
            messages: anthropic_messages,
            system: system_prompt,
            tools,
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            stop_sequences: request.stop.clone(),
            stream: request.stream,
        }
    }

    /// Parses an Anthropic SSE event into a chat completion chunk.
    fn parse_sse_event(event: &str) -> Option<ChatCompletionChunk> {
        // Skip ping events
        if event.trim().starts_with(":") {
            return None;
        }

        // Parse the JSON line
        let line = event.strip_prefix("data: ")?;
        let event: AnthropicSseEvent = serde_json::from_str(line).ok()?;

        match event {
            AnthropicSseEvent::ContentBlockStart { content_block, .. } => {
                // Handle tool_use start
                if let AnthropicContentBlock::ToolUse { id, name, .. } = content_block {
                    Some(ChatCompletionChunk {
                        id: "start".to_string(),
                        delta: MessageDelta {
                            role: Some(Role::Assistant),
                            content: Some(format!("{{\"tool\":\"{}\",\"id\":\"{}\"}}", name, id)),
                            tool_calls: None,
                        },
                        finish_reason: None,
                        usage: None,
                    })
                } else {
                    None
                }
            }
            AnthropicSseEvent::ContentBlockDelta { delta, .. } => match delta {
                AnthropicContentBlockDelta::TextDelta { text } => Some(ChatCompletionChunk {
                    id: "delta".to_string(),
                    delta: MessageDelta {
                        role: None,
                        content: Some(text),
                        tool_calls: None,
                    },
                    finish_reason: None,
                    usage: None,
                }),
                AnthropicContentBlockDelta::InputJsonDelta { partial_json } => {
                    Some(ChatCompletionChunk {
                        id: "delta".to_string(),
                        delta: MessageDelta {
                            role: None,
                            content: Some(partial_json),
                            tool_calls: None,
                        },
                        finish_reason: None,
                        usage: None,
                    })
                }
            },
            AnthropicSseEvent::ContentBlockStop { .. } => None,
            AnthropicSseEvent::MessageDelta { delta, usage, .. } => {
                let finish_reason = delta.stop_reason.as_ref().and_then(|s| match s.as_str() {
                    "end_turn" => Some(FinishReason::Stop),
                    "max_tokens" => Some(FinishReason::Length),
                    "tool_use" => Some(FinishReason::ToolCall),
                    _ => None,
                });

                Some(ChatCompletionChunk {
                    id: "delta".to_string(),
                    delta: MessageDelta {
                        role: None,
                        content: None,
                        tool_calls: None,
                    },
                    finish_reason: finish_reason.or(Some(FinishReason::Stop)),
                    usage: Some(TokenUsage {
                        prompt_tokens: usage.input_tokens,
                        completion_tokens: usage.output_tokens,
                        total_tokens: usage.input_tokens + usage.output_tokens,
                    }),
                })
            }
            AnthropicSseEvent::MessageStop { .. } => None,
            AnthropicSseEvent::MessageStart { message, .. } => {
                let id = message.id.clone();
                let usage = message.usage.clone().map(|u| TokenUsage {
                    prompt_tokens: u.input_tokens,
                    completion_tokens: u.output_tokens,
                    total_tokens: u.input_tokens + u.output_tokens,
                });

                Some(ChatCompletionChunk {
                    id,
                    delta: MessageDelta {
                        role: Some(Role::Assistant),
                        content: None,
                        tool_calls: None,
                    },
                    finish_reason: None,
                    usage,
                })
            }
            AnthropicSseEvent::Ping => None,
        }
    }

    /// Converts Anthropic API error to anyhow error.
    fn handle_api_error(&self, status: u16, body: &str) -> anyhow::Error {
        match serde_json::from_str::<AnthropicErrorResponse>(body) {
            Ok(error_response) => {
                if let Some(error) = error_response.error {
                    anyhow::anyhow!(
                        "Anthropic API error ({}): {}",
                        status,
                        error.message.unwrap_or_else(|| "Unknown error".to_string())
                    )
                } else {
                    anyhow::anyhow!("Anthropic API error: {}", status)
                }
            }
            Err(_) => anyhow::anyhow!(
                "Anthropic API error ({}): {}",
                status,
                body.trim()
            ),
        }
    }
}

#[async_trait]
impl ModelProvider for AnthropicProvider {
    fn id(&self) -> &str {
        "anthropic"
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        debug!("Listing Anthropic models");

        // For now, return hardcoded models
        // In the future, this could fetch from the /v1/models endpoint
        let models = vec![
            ModelInfo {
                id: "claude-3-5-sonnet-latest".to_string(),
                name: "Claude 3.5 Sonnet".to_string(),
                provider: "anthropic".to_string(),
                context_window: 200000,
                supports_vision: true,
                supports_tools: true,
            },
            ModelInfo {
                id: "claude-3-5-haiku-latest".to_string(),
                name: "Claude 3.5 Haiku".to_string(),
                provider: "anthropic".to_string(),
                context_window: 200000,
                supports_vision: true,
                supports_tools: true,
            },
            ModelInfo {
                id: "claude-3-opus-latest".to_string(),
                name: "Claude 3 Opus".to_string(),
                provider: "anthropic".to_string(),
                context_window: 200000,
                supports_vision: true,
                supports_tools: true,
            },
            ModelInfo {
                id: "claude-3-sonnet-latest".to_string(),
                name: "Claude 3 Sonnet".to_string(),
                provider: "anthropic".to_string(),
                context_window: 200000,
                supports_vision: true,
                supports_tools: true,
            },
        ];

        Ok(models)
    }

    async fn chat_completion(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionStream> {
        let api_key = self.get_api_key().unwrap_or(self.api_key.clone());

        let anthropic_request = self.build_anthropic_request(&request);

        debug!(
            "Sending request to Anthropic API: model={}, has_system={}, has_tools={}",
            anthropic_request.model,
            anthropic_request.system.is_some(),
            anthropic_request.tools.is_some()
        );

        let url = format!("{}/v1/messages", self.base_url);

        let response = self
            .client
            .request(reqwest::Method::POST, &url)
            .header("x-api-key", &api_key)
            .header("anthropic-version", "2023-06-01")
            .header("anthropic-beta", "messages-2023-12-15")
            .header("Content-Type", "application/json")
            .json(&anthropic_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await?;
            return Err(self.handle_api_error(status, &body));
        }

        // For streaming, parse SSE stream line by line
        // The parse_sse_event function now takes a static reference
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

        let url = format!("{}/v1/versions", self.base_url);

        let start = std::time::Instant::now();

        let response = self
            .client
            .request(reqwest::Method::GET, &url)
            .header("x-api-key", &api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .send()
            .await;

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
        let provider = AnthropicProvider::new(
            "test-key".to_string(),
            None,
            None,
            Some(60),
        );

        let message = Message {
            role: Role::User,
            content: MessageContent::Text("Hello".to_string()),
            tool_calls: None,
            tool_call_id: None,
        };

        let result = provider.convert_message(&message);
        assert!(result.is_some());

        let result = result.unwrap();
        assert_eq!(result.role, AnthropicRole::User);
        assert_eq!(result.content.len(), 1);

        if let AnthropicContentBlock::Text { text } = &result.content[0] {
            assert_eq!(text, "Hello");
        } else {
            panic!("Expected Text content block");
        }
    }

    #[test]
    fn test_extract_system_prompt() {
        let provider = AnthropicProvider::new(
            "test-key".to_string(),
            None,
            None,
            Some(60),
        );

        let messages = vec![
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
        ];

        let system = provider.extract_system_prompt(&messages);
        assert_eq!(system, Some("You are a helpful assistant".to_string()));
    }

    #[test]
    fn test_build_anthropic_request() {
        let provider = AnthropicProvider::new(
            "test-key".to_string(),
            None,
            None,
            Some(60),
        );

        let request = ChatCompletionRequest {
            model: "claude-3-5-sonnet-latest".to_string(),
            messages: vec![
                Message {
                    role: Role::System,
                    content: MessageContent::Text("System prompt".to_string()),
                    tool_calls: None,
                    tool_call_id: None,
                },
                Message {
                    role: Role::User,
                    content: MessageContent::Text("User message".to_string()),
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

        let anthropic_request = provider.build_anthropic_request(&request);

        assert_eq!(anthropic_request.model, "claude-3-5-sonnet-latest");
        assert_eq!(anthropic_request.system, Some("System prompt".to_string()));
        assert_eq!(anthropic_request.messages.len(), 1);
        assert_eq!(anthropic_request.temperature, Some(0.7));
        assert_eq!(anthropic_request.max_tokens, Some(1000));
        assert!(anthropic_request.stream);
    }
}
