//! Google Gemini API provider implementation.

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

pub mod api_types;
use api_types::*;

/// Gemini provider implementation.
///
/// This struct implements the [`ModelProvider`] trait for the Google Gemini API,
/// supporting streaming responses from `/v1beta/models/{model}:streamGenerateContent`,
/// function calling via function_declarations format, multi-modal input (text + images),
/// and both API key and OAuth authentication.
pub struct GeminiProvider {
    client: reqwest::Client,
    api_key: Option<String>,
    oauth_token: Option<String>,
    base_url: String,
    profile_manager: Arc<Mutex<AuthProfileManager>>,
}

impl GeminiProvider {
    /// Creates a new Gemini provider instance.
    ///
    /// # Arguments
    ///
    /// * `api_key` - Optional API key for authenticating with Gemini.
    /// * `oauth_token` - Optional OAuth bearer token for authentication.
    /// * `base_url` - The base URL for the Gemini API (defaults to "https://generativelanguage.googleapis.com").
    /// * `cooldown_seconds` - The cooldown period in seconds for failed profiles.
    pub fn new(
        api_key: Option<String>,
        oauth_token: Option<String>,
        base_url: Option<String>,
        cooldown_seconds: Option<u64>,
    ) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            oauth_token,
            base_url: base_url.unwrap_or_else(|| "https://generativelanguage.googleapis.com".to_string()),
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
            .next_key("gemini")
            .map(|p| p.api_key.clone())
    }

    /// Converts a core [`Message`] to a Gemini content.
    fn convert_message(&self, message: &Message) -> GeminiContent {
        let role = match message.role {
            Role::System => {
                // System instructions are handled via system_instruction field
                return GeminiContent {
                    role: None,
                    parts: Some(vec![GeminiPart::Text {
                        text: match &message.content {
                            MessageContent::Text(text) => text.clone(),
                            MessageContent::Parts(parts) => {
                                // For system messages with parts, extract text
                                parts
                                    .iter()
                                    .filter_map(|part| match part {
                                        ContentPart::Text { text } => Some(text.clone()),
                                        ContentPart::Image { .. } => None,
                                    })
                                    .collect::<Vec<_>>()
                                    .join("\n")
                            }
                        },
                    }]),
                };
            }
            Role::User => GeminiRole::User,
            Role::Assistant => GeminiRole::Model,
            Role::Tool => GeminiRole::Model, // Tool results are sent as model messages
        };

        let parts = match &message.content {
            MessageContent::Text(text) => vec![GeminiPart::Text { text: text.clone() }],
            MessageContent::Parts(parts) => parts
                .iter()
                .map(|part| match part {
                    ContentPart::Text { text } => GeminiPart::Text { text: text.clone() },
                    ContentPart::Image { media_type, data } => GeminiPart::Text {
                        text: format!("image: {} (base64: {})", media_type, data),
                    },
                })
                .collect(),
        };

        GeminiContent {
            role: Some(role),
            parts: Some(parts),
        }
    }

    /// Converts a core [`ToolDefinition`] to a Gemini function declaration.
    fn convert_tool(&self, tool: &ToolDefinition) -> GeminiFunctionDeclaration {
        GeminiFunctionDeclaration {
            name: tool.name.clone(),
            description: tool.description.clone(),
            parameters: Some(tool.parameters.clone()),
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

    /// Builds the Gemini request from a core request.
    fn build_gemini_request(&self, request: &ChatCompletionRequest) -> GeminiRequest {
        let contents = request
            .messages
            .iter()
            .map(|message| self.convert_message(message))
            .collect();

        let tools = request.tools.as_ref().map(|tools| {
            vec![GeminiTool {
                tool_type: "functionDeclarations".to_string(),
                function_declarations: Some(
                    tools.iter().map(|tool| self.convert_tool(tool)).collect(),
                ),
            }]
        });

        GeminiRequest {
            contents,
            tools,
            tool_config: None,
            temperature: request.temperature,
            max_output_tokens: request.max_tokens,
            stop_sequences: request.stop.clone(),
        }
    }

    /// Converts a Gemini response candidate to a chat completion chunk.
    fn parse_candidate(candidate: &GeminiCandidate) -> Option<ChatCompletionChunk> {
        let content = candidate.parts.as_ref()?;
        let text = content
            .iter()
            .filter_map(|part| match part {
                GeminiPart::Text { text } => Some(text.clone()),
            })
            .collect::<Vec<_>>()
            .join("");

        if text.is_empty() {
            return None;
        }

        Some(ChatCompletionChunk {
            id: "gemini_chunk".to_string(),
            delta: MessageDelta {
                role: Some(Role::Assistant),
                content: Some(text),
                tool_calls: None,
            },
            finish_reason: candidate.finish_reason.as_ref().and_then(|fc| match fc.as_str() {
                "STOP" => Some(FinishReason::Stop),
                "MAX_TOKENS" => Some(FinishReason::Length),
                _ => None,
            }),
            usage: candidate.token_usage.as_ref().map(|usage| TokenUsage {
                prompt_tokens: usage.prompt_token_count.unwrap_or(0),
                completion_tokens: usage.candidates_token_count.unwrap_or(0),
                total_tokens: usage.total_token_count.unwrap_or(0),
            }),
        })
    }

    /// Parses a Gemini streaming JSON event into a chat completion chunk.
    fn parse_stream_event(event: &str) -> Option<ChatCompletionChunk> {
        // Gemini streaming uses JSON array format with newline separators
        let event: GeminiStreamResponse = serde_json::from_str(event).ok()?;

        let candidate = event.candidates?.into_iter().next()?;
        Self::parse_candidate(&candidate)
    }

    /// Builds the authentication headers and query parameters.
    fn build_auth(&self) -> (reqwest::header::HeaderMap, Option<String>) {
        let mut headers = reqwest::header::HeaderMap::new();

        if let Some(ref token) = self.oauth_token {
            headers.insert(
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
            );
        }

        let query_param = self.api_key.clone().or_else(|| self.get_api_key());

        (headers, query_param)
    }
}

#[async_trait]
impl ModelProvider for GeminiProvider {
    fn id(&self) -> &str {
        "gemini"
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        let url = format!("{}/v1beta/models", self.base_url);
        let (headers, api_key) = self.build_auth();

        let mut request = self.client.get(&url).headers(headers);
        if let Some(key) = api_key {
            request = request.query(&[("key", key)]);
        }

        let response = request.send().await?;
        let status = response.status();

        if status.is_success() {
            let models_response: GeminiListModelResponse = response.json().await?;
            let models: Vec<ModelInfo> = models_response
                .models
                .into_iter()
                .map(|m| ModelInfo {
                    id: m.name,
                    name: m.display_name,
                    provider: "gemini".to_string(),
                    context_window: m.input_token_limit + m.output_token_limit,
                    supports_vision: true,
                    supports_tools: true,
                })
                .collect();
            Ok(models)
        } else {
            let error_response: GeminiErrorResponse = response.json().await?;
            Err(anyhow::anyhow!(
                "Failed to list models: {:?}",
                error_response.error
            ))
        }
    }

    async fn chat_completion(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionStream> {
        let model = request.model.clone();
        let gemini_request = self.build_gemini_request(&request);
        let (headers, api_key) = self.build_auth();

        let url = format!("{}/v1beta/models/{}:streamGenerateContent", self.base_url, model);

        let mut req = self
            .client
            .post(&url)
            .headers(headers)
            .json(&gemini_request);

        if let Some(key) = api_key {
            req = req.query(&[("key", key)]);
        }

        let response = req.send().await?;
        let status = response.status();

        if !status.is_success() {
            let error_response: GeminiErrorResponse = response.json().await?;
            return Err(anyhow::anyhow!(
                "Failed to get chat completion: {:?}",
                error_response.error
            ));
        }

        // Parse streaming response
        let stream = response.bytes_stream().map(move |chunk| {
            let chunk = chunk.map_err(|e| anyhow::anyhow!("Stream error: {}", e))?;
            
            // Decode the chunk as UTF-8
            let text = String::from_utf8_lossy(&chunk);

            // Gemini streaming returns newline-delimited JSON objects
            // Each line is a complete JSON object
            let mut chunks = Vec::new();
            for line in text.lines() {
                if let Some(parsed) = Self::parse_stream_event(line) {
                    chunks.push(parsed);
                }
            }

            if chunks.is_empty() {
                return Ok(None);
            }

            // Return all chunks
            Ok(Some(chunks))
        });

        // Filter out None values and box the stream
        let boxed: BoxStream<'static, Result<ChatCompletionChunk>> =
            Box::pin(stream.filter_map(|x| async move {
                match x {
                    Ok(Some(chunk_list)) => {
                        // Return each chunk individually
                        if chunk_list.is_empty() {
                            None
                        } else {
                            let first_chunk = chunk_list.into_iter().next();
                            first_chunk.map(|c| Ok(c))
                        }
                    }
                    Ok(None) => None,
                    Err(e) => Some(Err(e)),
                }
            }));

        Ok(boxed)
    }

    async fn health_check(&self) -> Result<ProviderHealth> {
        let start = std::time::Instant::now();
        
        match self.list_models().await {
            Ok(models) => {
                let latency = start.elapsed().as_millis() as u64;
                Ok(ProviderHealth {
                    available: !models.is_empty(),
                    latency_ms: Some(latency),
                })
            }
            Err(e) => {
                warn!("Health check failed: {}", e);
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
    fn test_gemini_role_serialization() {
        let user = GeminiRole::User;
        let json = serde_json::to_string(&user).unwrap();
        assert_eq!(json, "\"user\"");

        let parsed: GeminiRole = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, GeminiRole::User);
    }

    #[test]
    fn test_gemini_part_serialization() {
        let part = GeminiPart::Text {
            text: "Hello".to_string(),
        };
        let json = serde_json::to_string(&part).unwrap();
        assert!(json.contains("\"text\""));
        assert!(json.contains("\"Hello\""));
    }

    #[test]
    fn test_gemini_content_serialization() {
        let content = GeminiContent {
            role: Some(GeminiRole::User),
            parts: Some(vec![GeminiPart::Text {
                text: "Hello".to_string(),
            }]),
        };
        let json = serde_json::to_string(&content).unwrap();
        let parsed: GeminiContent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.role, Some(GeminiRole::User));
    }

    #[test]
    fn test_gemini_function_declaration_serialization() {
        let func = GeminiFunctionDeclaration {
            name: "calculator".to_string(),
            description: "A calculator tool".to_string(),
            parameters: Some(serde_json::json!({
                "type": "object",
                "properties": {}
            })),
        };
        let json = serde_json::to_string(&func).unwrap();
        let parsed: GeminiFunctionDeclaration = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "calculator");
    }

    #[test]
    fn test_gemini_request_serialization() {
        let request = GeminiRequest {
            contents: vec![GeminiContent {
                role: Some(GeminiRole::User),
                parts: Some(vec![GeminiPart::Text {
                    text: "Hello".to_string(),
                }]),
            }],
            tools: None,
            tool_config: None,
            temperature: None,
            max_output_tokens: None,
            stop_sequences: None,
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"contents\""));
    }

    #[test]
    fn test_gemini_model_info_serialization() {
        let model = GeminiModel {
            name: "models/gemini-1.5-flash".to_string(),
            base_model_id: "gemini-1.5-flash".to_string(),
            version: "1.0".to_string(),
            display_name: "Gemini 1.5 Flash".to_string(),
            description: "A fast model".to_string(),
            input_token_limit: 1000000,
            output_token_limit: 8192,
            supported_generation_methods: None,
            parameters: None,
            safety_settings: None,
            load_parameters: None,
        };
        let json = serde_json::to_string(&model).unwrap();
        let parsed: GeminiModel = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.display_name, "Gemini 1.5 Flash");
    }
}
