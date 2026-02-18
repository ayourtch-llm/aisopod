//! Ollama provider implementation for local LLM inference.

use anyhow::Result;
use async_trait::async_trait;
use futures_core::Stream;
use futures_util::stream::{self, BoxStream, StreamExt};
use std::pin::Pin;
use tracing::{debug, instrument, warn};

use crate::trait_module::{ChatCompletionStream, ModelProvider};
use crate::types::*;

/// Ollama provider implementation.
///
/// This struct implements the [`ModelProvider`] trait for the Ollama
/// REST API, supporting streaming chat completions, local model discovery,
/// and a configurable endpoint URL for connecting to locally running
/// Ollama instances.
///
/// # Example
///
/// ```ignore
/// use aisopod_provider::providers::ollama::OllamaProvider;
///
/// let provider = OllamaProvider::new(Some("http://localhost:11434".to_string()));
/// ```
pub struct OllamaProvider {
    client: reqwest::Client,
    base_url: String,
}

impl OllamaProvider {
    /// Creates a new Ollama provider instance.
    ///
    /// # Arguments
    ///
    /// * `base_url` - The base URL for the Ollama API (defaults to "http://localhost:11434").
    pub fn new(base_url: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.unwrap_or_else(|| "http://localhost:11434".to_string()),
        }
    }

    /// Converts a core [`Message`] to an Ollama message.
    fn convert_message(&self, message: &Message) -> OllamaMessage {
        let role = match message.role {
            Role::System => OllamaRole::System,
            Role::User => OllamaRole::User,
            Role::Assistant => OllamaRole::Assistant,
            Role::Tool => OllamaRole::Tool,
        };

        let content = match &message.content {
            MessageContent::Text(text) => text.clone(),
            MessageContent::Parts(parts) => {
                // For multi-modal content, concatenate text parts
                // Ollama's chat API doesn't directly support multi-modal in the same way
                parts
                    .iter()
                    .filter_map(|part| match part {
                        ContentPart::Text { text } => Some(text.clone()),
                        ContentPart::Image { .. } => None, // Images not supported in chat API
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        };

        OllamaMessage {
            role,
            content,
        }
    }

    /// Converts a core [`ToolDefinition`] to an Ollama function.
    fn convert_tool(&self, tool: &ToolDefinition) -> OllamaFunction {
        OllamaFunction {
            name: tool.name.clone(),
            description: Some(tool.description.clone()),
            parameters: Some(tool.parameters.clone()),
        }
    }

    /// Builds the Ollama request from a core request.
    fn build_ollama_request(&self, request: &ChatCompletionRequest) -> OllamaRequest {
        let messages = request
            .messages
            .iter()
            .map(|message| self.convert_message(message))
            .collect();

        let tools = request.tools.as_ref().map(|tools| {
            tools
                .iter()
                .map(|tool| OllamaTool::Function(self.convert_tool(tool)))
                .collect()
        });

        OllamaRequest {
            model: request.model.clone(),
            messages,
            tools,
            stream: request.stream,
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            stop: request.stop.clone(),
        }
    }
}

/// Ollama API request types (private module)
mod api_types {
    use serde::{Deserialize, Serialize};

    /// The role of a message in the Ollama chat API.
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub enum OllamaRole {
        #[serde(rename = "system")]
        System,
        #[serde(rename = "user")]
        User,
        #[serde(rename = "assistant")]
        Assistant,
        #[serde(rename = "tool")]
        Tool,
    }

    /// A message in the Ollama chat API.
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct OllamaMessage {
        pub role: OllamaRole,
        pub content: String,
    }

    /// An Ollama function definition for tool calling.
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct OllamaFunction {
        pub name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub description: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub parameters: Option<serde_json::Value>,
    }

    /// An Ollama tool definition.
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(tag = "type")]
    pub enum OllamaTool {
        #[serde(rename = "function")]
        Function(OllamaFunction),
    }

    /// A request to the Ollama chat API.
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct OllamaRequest {
        pub model: String,
        pub messages: Vec<OllamaMessage>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub tools: Option<Vec<OllamaTool>>,
        #[serde(default)]
        pub stream: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub temperature: Option<f32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub max_tokens: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub stop: Option<Vec<String>>,
    }

    /// A chat completion chunk from the Ollama API.
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct OllamaChatCompletionChunk {
        pub model: String,
        pub created_at: String,
        pub message: OllamaMessage,
        pub done: bool,
        #[serde(default)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub total_duration: Option<u64>,
        #[serde(default)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub load_duration: Option<u64>,
        #[serde(default)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub prompt_eval_count: Option<u32>,
        #[serde(default)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub prompt_eval_duration: Option<u64>,
        #[serde(default)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub eval_count: Option<u32>,
        #[serde(default)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub eval_duration: Option<u64>,
    }

    /// Response from the Ollama /api/tags endpoint.
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct OllamaTagsResponse {
        pub models: Vec<OllamaModelInfo>,
    }

    /// Information about an Ollama model.
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct OllamaModelInfo {
        pub name: String,
        pub modified_at: String,
        #[serde(default)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub size: Option<u64>,
        #[serde(default)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub digest: Option<String>,
        #[serde(default)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub parent_model: Option<String>,
        #[serde(default)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub format: Option<String>,
        #[serde(default)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub family: Option<String>,
        #[serde(default)]
        #[serde(skip_serializing_if = "Vec::is_empty")]
        pub families: Vec<String>,
        #[serde(default)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub parameter_size: Option<String>,
        #[serde(default)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub quantization_level: Option<String>,
    }

    /// Error response from Ollama API.
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct OllamaErrorResponse {
        pub error: String,
    }
}

use api_types::*;

#[async_trait]
impl ModelProvider for OllamaProvider {
    fn id(&self) -> &str {
        "ollama"
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        debug!("Listing Ollama models");

        let url = format!("{}/api/tags", self.base_url);

        let response = self
            .client
            .request(reqwest::Method::GET, &url)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await?;
            return Err(anyhow::anyhow!(
                "Ollama API error ({}): {}",
                status,
                body.trim()
            ));
        }

        let tags_response: OllamaTagsResponse = response.json().await?;

        let models = tags_response
            .models
            .into_iter()
            .map(|model| ModelInfo {
                id: model.name.clone(),
                name: model.name,
                provider: "ollama".to_string(),
                context_window: 8192, // Default Ollama context window
                supports_vision: false, // Ollama vision support varies by model
                supports_tools: false, // Ollama tool support varies by model
            })
            .collect();

        Ok(models)
    }

    async fn chat_completion(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionStream> {
        let ollama_request = self.build_ollama_request(&request);

        debug!(
            "Sending request to Ollama API: model={}, has_tools={}",
            ollama_request.model,
            ollama_request.tools.is_some()
        );

        let url = format!("{}/api/chat", self.base_url);

        let response = self
            .client
            .request(reqwest::Method::POST, &url)
            .json(&ollama_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await?;
            return Err(anyhow::anyhow!(
                "Ollama API error ({}): {}",
                status,
                body.trim()
            ));
        }

        // Parse the newline-delimited JSON stream
        let stream = response.bytes_stream().map(move |chunk| {
            let chunk = chunk.map_err(|e| anyhow::anyhow!("Stream error: {}", e))?;

            // Decode the chunk as UTF-8
            let text = String::from_utf8_lossy(&chunk);

            // Process line by line (newline-delimited JSON)
            let mut chunks = Vec::new();
            for line in text.lines() {
                if line.trim().is_empty() {
                    continue;
                }
                if let Some(parsed) = Self::parse_ollama_chunk(line) {
                    chunks.push(parsed);
                }
            }

            if chunks.is_empty() {
                return Ok(None);
            }

            // Return the first chunk for now
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
        let url = format!("{}/api/tags", self.base_url);

        let start = std::time::Instant::now();

        let response = self
            .client
            .request(reqwest::Method::GET, &url)
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

impl OllamaProvider {
    /// Converts an Ollama role to the core Role type.
    fn convert_role(role: OllamaRole) -> Role {
        match role {
            OllamaRole::System => Role::System,
            OllamaRole::User => Role::User,
            OllamaRole::Assistant => Role::Assistant,
            OllamaRole::Tool => Role::Tool,
        }
    }

    /// Parses an Ollama chunk response into a chat completion chunk.
    fn parse_ollama_chunk(line: &str) -> Option<ChatCompletionChunk> {
        let chunk: OllamaChatCompletionChunk = serde_json::from_str(line).ok()?;

        // Convert finish reason
        let finish_reason = if chunk.done {
            Some(FinishReason::Stop)
        } else {
            None
        };

        // Build usage if available
        let usage = chunk
            .prompt_eval_count
            .or(chunk.eval_count)
            .map(|prompt_tokens| {
                let completion_tokens = chunk.eval_count.unwrap_or(0);
                TokenUsage {
                    prompt_tokens,
                    completion_tokens,
                    total_tokens: prompt_tokens + completion_tokens,
                }
            });

        Some(ChatCompletionChunk {
            id: chunk.created_at.clone(),
            delta: MessageDelta {
                role: Some(Self::convert_role(chunk.message.role)),
                content: Some(chunk.message.content),
                tool_calls: None,
            },
            finish_reason,
            usage,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_message_text() {
        let provider = OllamaProvider::new(None);

        let message = Message {
            role: Role::User,
            content: MessageContent::Text("Hello".to_string()),
            tool_calls: None,
            tool_call_id: None,
        };

        let result = provider.convert_message(&message);
        assert_eq!(result.role, OllamaRole::User);
        assert_eq!(result.content, "Hello");
    }

    #[test]
    fn test_convert_message_system() {
        let provider = OllamaProvider::new(None);

        let message = Message {
            role: Role::System,
            content: MessageContent::Text("You are a helpful assistant".to_string()),
            tool_calls: None,
            tool_call_id: None,
        };

        let result = provider.convert_message(&message);
        assert_eq!(result.role, OllamaRole::System);
    }

    #[test]
    fn test_convert_message_parts() {
        let provider = OllamaProvider::new(None);

        let message = Message {
            role: Role::User,
            content: MessageContent::Parts(vec![
                ContentPart::Text { text: "Line 1".to_string() },
                ContentPart::Text { text: "Line 2".to_string() },
            ]),
            tool_calls: None,
            tool_call_id: None,
        };

        let result = provider.convert_message(&message);
        assert_eq!(result.content, "Line 1\nLine 2");
    }

    #[test]
    fn test_build_ollama_request() {
        let provider = OllamaProvider::new(None);

        let request = ChatCompletionRequest {
            model: "llama3".to_string(),
            messages: vec![Message {
                role: Role::User,
                content: MessageContent::Text("Hello".to_string()),
                tool_calls: None,
                tool_call_id: None,
            }],
            tools: None,
            temperature: Some(0.7),
            max_tokens: Some(1000),
            stop: None,
            stream: true,
        };

        let ollama_request = provider.build_ollama_request(&request);

        assert_eq!(ollama_request.model, "llama3");
        assert_eq!(ollama_request.messages.len(), 1);
        assert_eq!(ollama_request.temperature, Some(0.7));
        assert_eq!(ollama_request.max_tokens, Some(1000));
        assert!(ollama_request.stream);
    }

    #[test]
    fn test_parse_ollama_chunk() {
        let line = r#"{
            "model": "llama3",
            "created_at": "2024-01-01T00:00:00.000Z",
            "message": {
                "role": "assistant",
                "content": "Hello"
            },
            "done": false
        }"#;

        // Debug: print the parsed Ollama chunk
        let parsed: Result<OllamaChatCompletionChunk, _> = serde_json::from_str(line);
        println!("Parsed Ollama chunk: {:?}", parsed);
        
        let result = OllamaProvider::parse_ollama_chunk(line);
        println!("Parsed chat completion chunk: {:?}", result);
        assert!(result.is_some(), "Failed to parse Ollama chunk");

        let chunk = result.unwrap();
        assert_eq!(chunk.delta.role, Some(Role::Assistant));
        assert_eq!(chunk.delta.content, Some("Hello".to_string()));
        assert_eq!(chunk.finish_reason, None);
    }

    #[test]
    fn test_parse_ollama_chunk_done() {
        let line = r#"{
            "model": "llama3",
            "created_at": "2024-01-01T00:00:00.000Z",
            "message": {
                "role": "assistant",
                "content": ""
            },
            "done": true,
            "prompt_eval_count": 10,
            "eval_count": 5
        }"#;

        // Debug: print the parsed Ollama chunk
        let parsed: Result<OllamaChatCompletionChunk, _> = serde_json::from_str(line);
        println!("Parsed Ollama chunk: {:?}", parsed);
        
        let result = OllamaProvider::parse_ollama_chunk(line);
        println!("Parsed chat completion chunk: {:?}", result);
        assert!(result.is_some(), "Failed to parse Ollama chunk with done");

        let chunk = result.unwrap();
        assert_eq!(chunk.finish_reason, Some(FinishReason::Stop));
        assert!(chunk.usage.is_some());
        let usage = chunk.usage.unwrap();
        assert_eq!(usage.prompt_tokens, 10);
        assert_eq!(usage.completion_tokens, 5);
        assert_eq!(usage.total_tokens, 15);
    }
}
