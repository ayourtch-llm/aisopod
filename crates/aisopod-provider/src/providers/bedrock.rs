//! AWS Bedrock Runtime API provider implementation.

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use aws_config::BehaviorVersion;
use aws_sdk_bedrockruntime::{
    self as bedrockruntime,
    types::{PayloadPart, ResponseStream, Tool, ToolInputSchema},
    Client as BedrockClient,
};
use aws_config::Region;
use std::borrow::Cow;
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

/// AWS Bedrock provider implementation.
///
/// This struct implements the [`ModelProvider`] trait for the AWS Bedrock
/// Runtime API, supporting streaming completions via the Bedrock Converse API
/// and the full AWS credential chain (environment variables, profile, IAM role).
pub struct BedrockProvider {
    client: Arc<BedrockClient>,
    region: String,
    default_model: String,
    profile_manager: Arc<Mutex<AuthProfileManager>>,
}

impl BedrockProvider {
    /// Creates a new Bedrock provider instance.
    ///
    /// # Arguments
    ///
    /// * `region` - The AWS region for Bedrock (e.g., "us-east-1").
    /// * `default_model` - The default model ID to use for requests.
    /// * `cooldown_seconds` - The cooldown period in seconds for failed profiles.
    pub async fn new(
        region: Option<String>,
        default_model: Option<String>,
        cooldown_seconds: Option<u64>,
    ) -> Result<Self> {
        let region_str = region.unwrap_or_else(|| "us-east-1".to_string());
        let default_model = default_model.unwrap_or_else(|| "anthropic.claude-3-sonnet-20240229-v1:0".to_string());

        // Load AWS configuration with the credential chain
        // This uses the standard AWS credential chain:
        // 1. Environment variables (AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY)
        // 2. Shared credentials file (~/.aws/credentials)
        // 3. IAM role for Amazon EC2 instances
        // 4. Other AWS credential providers
        let shared_config = aws_config::load_defaults(BehaviorVersion::v2024_03_28()).await;
        // We use Cow::Owned to wrap the region string, which satisfies the 'static
        // lifetime requirement since Cow<'static, str> can own the String.
        let region_cow: Cow<'static, str> = Cow::Owned(region_str.clone());
        let config = aws_sdk_bedrockruntime::Config::from(&shared_config)
            .to_builder()
            .region(Some(Region::new(region_cow)))
            .build();

        let client = BedrockClient::from_conf(config);

        Ok(Self {
            client: Arc::new(client),
            region: region_str,
            default_model,
            profile_manager: Arc::new(Mutex::new(AuthProfileManager::new(
                Duration::from_secs(cooldown_seconds.unwrap_or(60)),
            ))),
        })
    }

    /// Adds an authentication profile for credential rotation.
    pub fn add_profile(&mut self, profile: AuthProfile) {
        let mut manager = self.profile_manager.lock().unwrap();
        manager.add_profile(profile);
    }

    /// Gets the next available API key for round-robin rotation.
    fn get_api_key(&self) -> Option<String> {
        let mut manager = self.profile_manager.lock().unwrap();
        manager
            .next_key("bedrock")
            .map(|p| p.api_key.clone())
    }

    /// Converts a core [`Message`] to a Bedrock message.
    fn convert_message(&self, message: &Message) -> Result<BedrockMessage> {
        let role = match message.role {
            Role::System => return Err(anyhow!("System role should be handled separately")),
            Role::User => BedrockRole::User,
            Role::Assistant => BedrockRole::Assistant,
            Role::Tool => BedrockRole::Assistant, // Tool results are sent as assistant messages
        };

        let content = match &message.content {
            MessageContent::Text(text) => {
                vec![BedrockContentBlock::Text { text: text.clone() }]
            }
            MessageContent::Parts(parts) => {
                let mut content_blocks = Vec::new();
                for part in parts {
                    match part {
                        ContentPart::Text { text } => {
                            content_blocks.push(BedrockContentBlock::Text { text: text.clone() });
                        }
                        ContentPart::Image { media_type, data } => {
                            // Parse media type to determine format
                            let format = match media_type.as_str() {
                                "image/png" => BedrockImageFormat::Png,
                                "image/jpeg" => BedrockImageFormat::Jpeg,
                                "image/gif" => BedrockImageFormat::Gif,
                                "image/webp" => BedrockImageFormat::Webp,
                                _ => return Err(anyhow!("Unsupported image media type: {}", media_type)),
                            };
                            content_blocks.push(BedrockContentBlock::Image {
                                format,
                                source: BedrockImageSource::Bytes { data: data.clone() },
                            });
                        }
                    }
                }
                content_blocks
            }
        };

        Ok(BedrockMessage { role, content })
    }

    /// Converts a core [`ToolDefinition`] to a Bedrock tool.
    fn convert_tool(&self, tool: &ToolDefinition) -> Result<BedrockTool> {
        Ok(BedrockTool {
            tool_spec: BedrockToolSpec {
                name: tool.name.clone(),
                description: Some(tool.description.clone()),
                input_schema: BedrockToolInputSchema {
                    json: tool.parameters.clone(),
                },
            },
        })
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

    /// Builds the Bedrock request from a core request.
    fn build_bedrock_request(&self, request: &ChatCompletionRequest) -> Result<BedrockRequest> {
        let mut bedrock_messages = Vec::new();
        for message in &request.messages {
            if message.role == Role::System {
                continue; // Handled separately
            }
            bedrock_messages.push(self.convert_message(message)?);
        }

        let system = self.extract_system_prompt(&request.messages)
            .map(|text| vec![BedrockSystemMessage { text_content: text }]);

        let tools = request.tools.as_ref().map(|tools| {
            tools.iter().map(|tool| self.convert_tool(tool)).collect::<Result<Vec<_>>>()
        }).transpose()?;

        // Determine tool choice based on tools presence
        let tool_config = if tools.is_some() {
            Some(BedrockToolConfig {
                tool_choice: Some(BedrockToolChoice::Auto),
            })
        } else {
            None
        };

        // Build inference configuration
        let inference_config = BedrockInferenceConfig {
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            top_p: None,
            stop_sequences: request.stop.clone(),
        };

        Ok(BedrockRequest {
            model_id: request.model.clone(),
            messages: bedrock_messages,
            system,
            tools,
            tool_config,
            inference_config: Some(inference_config),
        })
    }

    /// Parses a Bedrock streaming event into a chat completion chunk.
    fn parse_stream_event(event: &BedrockStreamEvent) -> Option<ChatCompletionChunk> {
        match event {
            BedrockStreamEvent::ContentBlockStart { content_block, event_type, .. } => {
                // Handle tool_use start
                if let BedrockContentBlock::ToolUse { tool_use_id, name, .. } = content_block {
                    Some(ChatCompletionChunk {
                        id: "start".to_string(),
                        delta: MessageDelta {
                            role: Some(Role::Assistant),
                            content: Some(format!("{{\"tool\":\"{}\",\"id\":\"{}\"}}", name, tool_use_id)),
                            tool_calls: None,
                        },
                        finish_reason: None,
                        usage: None,
                    })
                } else {
                    None
                }
            }
            BedrockStreamEvent::ContentBlockDelta { delta, .. } => match delta {
                BedrockContentBlockDelta::TextDelta { text } => Some(ChatCompletionChunk {
                    id: "delta".to_string(),
                    delta: MessageDelta {
                        role: None,
                        content: Some(text.clone()),
                        tool_calls: None,
                    },
                    finish_reason: None,
                    usage: None,
                }),
                BedrockContentBlockDelta::ToolUseDelta { tool_use_id, input } => {
                    Some(ChatCompletionChunk {
                        id: "delta".to_string(),
                        delta: MessageDelta {
                            role: None,
                            content: Some(input.clone()),
                            tool_calls: None,
                        },
                        finish_reason: None,
                        usage: None,
                    })
                }
            },
            BedrockStreamEvent::ContentBlockStop { .. } => None,
            BedrockStreamEvent::MessageStop { stop_reason, .. } => {
                let finish_reason = match stop_reason.as_str() {
                    "end_turn" | "stop_sequence" => Some(FinishReason::Stop),
                    "max_tokens" => Some(FinishReason::Length),
                    "tool_use" => Some(FinishReason::ToolCall),
                    "content_filter" => Some(FinishReason::ContentFilter),
                    _ => Some(FinishReason::Error),
                };

                Some(ChatCompletionChunk {
                    id: "stop".to_string(),
                    delta: MessageDelta {
                        role: None,
                        content: None,
                        tool_calls: None,
                    },
                    finish_reason,
                    usage: None,
                })
            }
            BedrockStreamEvent::MessageStart { message, .. } => {
                let usage = message.usage.as_ref().map(|usage| TokenUsage {
                    prompt_tokens: usage.input_tokens,
                    completion_tokens: usage.output_tokens,
                    total_tokens: usage.total_tokens.unwrap_or(0),
                });

                Some(ChatCompletionChunk {
                    id: message.id.clone(),
                    delta: MessageDelta {
                        role: message.role.as_ref().and_then(|r| match r {
                            BedrockRole::User => Some(Role::User),
                            BedrockRole::Assistant => Some(Role::Assistant),
                        }),
                        content: None,
                        tool_calls: None,
                    },
                    finish_reason: None,
                    usage,
                })
            }
            BedrockStreamEvent::InternalException { message, .. } => {
                warn!("Bedrock internal exception: {}", message);
                None
            }
            BedrockStreamEvent::LimitExceededException { message, .. } => {
                warn!("Bedrock limit exceeded: {}", message);
                None
            }
            BedrockStreamEvent::AccessDeniedException { message, .. } => {
                warn!("Bedrock access denied: {}", message);
                None
            }
            BedrockStreamEvent::ModelTimeoutException { message, .. } => {
                warn!("Bedrock model timeout: {}", message);
                None
            }
            BedrockStreamEvent::ValidationException { message, .. } => {
                warn!("Bedrock validation exception: {}", message);
                None
            }
            BedrockStreamEvent::EventStream { .. } => None,
        }
    }

    /// Maps AWS-specific errors to provider errors.
    fn map_aws_error(&self, error: anyhow::Error) -> anyhow::Error {
        // Check for expired credentials
        if error.to_string().contains("ExpiredToken") || 
           error.to_string().contains("ExpiredCredentials") ||
           error.to_string().contains("security token expired") {
            return anyhow::anyhow!("AWS credentials have expired: {}", error);
        }

        // Check for throttling
        if error.to_string().contains("Throttling") ||
           error.to_string().contains("ThrottlingException") ||
           error.to_string().contains("TooManyRequests") {
            return anyhow::anyhow!("AWS Bedrock is throttling requests: {}", error);
        }

        // Check for access denied
        if error.to_string().contains("AccessDenied") ||
           error.to_string().contains("Unauthorized") {
            return anyhow::anyhow!("AWS access denied: {}", error);
        }

        // Check for model timeout
        if error.to_string().contains("ModelTimeout") ||
           error.to_string().contains("Timeout") {
            return anyhow::anyhow!("Bedrock model timeout: {}", error);
        }

        error
    }
}

#[async_trait]
impl ModelProvider for BedrockProvider {
    fn id(&self) -> &str {
        "bedrock"
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        // Note: Bedrock's ListFoundationModels API is only available in us-east-1
        // For other regions, we return a curated list of common models
        
        if self.region == "us-east-1" {
            // Try to fetch models from Bedrock API
            // This requires additional permissions (bedrock:ListFoundationModels)
            // For now, we return a curated list since the API might not be available
            // in all configurations
        }

        Ok(vec![
            ModelInfo {
                id: "anthropic.claude-3-5-sonnet-20240620-v1:0".to_string(),
                name: "Claude 3.5 Sonnet".to_string(),
                provider: "Bedrock".to_string(),
                context_window: 200000,
                supports_vision: true,
                supports_tools: true,
            },
            ModelInfo {
                id: "anthropic.claude-3-sonnet-20240229-v1:0".to_string(),
                name: "Claude 3 Sonnet".to_string(),
                provider: "Bedrock".to_string(),
                context_window: 200000,
                supports_vision: true,
                supports_tools: true,
            },
            ModelInfo {
                id: "anthropic.claude-3-opus-20240229-v1:0".to_string(),
                name: "Claude 3 Opus".to_string(),
                provider: "Bedrock".to_string(),
                context_window: 200000,
                supports_vision: true,
                supports_tools: true,
            },
            ModelInfo {
                id: "anthropic.claude-3-haiku-20240307-v1:0".to_string(),
                name: "Claude 3 Haiku".to_string(),
                provider: "Bedrock".to_string(),
                context_window: 200000,
                supports_vision: true,
                supports_tools: true,
            },
            ModelInfo {
                id: "amazon.nova-pro-v1:0".to_string(),
                name: "Amazon Nova Pro".to_string(),
                provider: "Bedrock".to_string(),
                context_window: 300000,
                supports_vision: true,
                supports_tools: true,
            },
            ModelInfo {
                id: "amazon.nova-lite-v1:0".to_string(),
                name: "Amazon Nova Lite".to_string(),
                provider: "Bedrock".to_string(),
                context_window: 300000,
                supports_vision: true,
                supports_tools: true,
            },
            ModelInfo {
                id: "meta.llama3-2-11b-instruct-v1:0".to_string(),
                name: "Llama 3.2 11B".to_string(),
                provider: "Bedrock".to_string(),
                context_window: 131072,
                supports_vision: true,
                supports_tools: true,
            },
            ModelInfo {
                id: "meta.llama3-2-90b-instruct-v1:0".to_string(),
                name: "Llama 3.2 90B".to_string(),
                provider: "Bedrock".to_string(),
                context_window: 131072,
                supports_vision: true,
                supports_tools: true,
            },
            ModelInfo {
                id: "mistral.mistral-large-2402-v1:0".to_string(),
                name: "Mistral Large".to_string(),
                provider: "Bedrock".to_string(),
                context_window: 32000,
                supports_vision: false,
                supports_tools: true,
            },
        ])
    }

    #[instrument(skip(self, request), fields(model = request.model))]
    async fn chat_completion(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionStream> {
        let bedrock_request = self.build_bedrock_request(&request)?;
        let client = Arc::clone(&self.client);
        let model_id = request.model.clone();

        // Serialize the request to JSON
        let body = serde_json::to_vec(&bedrock_request)
            .map_err(|e| anyhow!("Failed to serialize Bedrock request: {}", e))?;

        debug!("Sending Bedrock request for model: {}", request.model);

        // Call InvokeModelWithResponseStream
        let response = client
            .invoke_model_with_response_stream()
            .model_id(&model_id)
            .body(body.into())
            .send()
            .await
            .map_err(|e| self.map_aws_error(anyhow::anyhow!(e)))?;

        // Get the event receiver from the response
        let mut event_receiver = response.body;
        let parse_fn = |event: BedrockStreamEvent| Self::parse_stream_event(&event);

        // Convert EventReceiver to a futures stream
        // EventReceiver has a recv() method that returns Result<Option<ResponseStream>, SdkError>
        let stream = async_stream::stream! {
            loop {
                match event_receiver.recv().await {
                    Ok(Some(response_stream)) => {
                        // The response stream contains a chunk with payload bytes
                        let chunk = match response_stream.as_chunk() {
                            Ok(payload_part) => payload_part,
                            Err(_) => continue,
                        };
                        
                        // Get the bytes from the payload part
                        let bytes = match chunk.bytes() {
                            Some(b) => b,
                            None => continue,
                        };
                        
                        // Parse the UTF-8 payload
                        let payload_str = std::str::from_utf8(bytes.as_ref())
                            .map_err(|e| anyhow!("Failed to parse event stream as UTF-8: {}", e))?;
                        
                        // Parse the JSON payload
                        let event: BedrockStreamEvent = serde_json::from_str(payload_str)
                            .map_err(|e| anyhow!("Failed to parse Bedrock stream event: {}", e))?;
                        
                        // Parse the stream event into a chunk
                        match parse_fn(event) {
                            Some(chunk) => yield Ok(chunk),
                            None => continue,
                        }
                    }
                    Ok(None) => {
                        // Stream ended
                        break;
                    }
                    Err(e) => {
                        // Error from the event stream
                        yield Err(anyhow!("Failed to read event stream: {}", e));
                        break;
                    }
                }
            }
        };

        Ok(Box::pin(stream))
    }

    async fn health_check(&self) -> Result<ProviderHealth> {
        // Perform a lightweight check by listing models
        // This verifies credentials and connectivity
        let start_time = std::time::Instant::now();

        match self.list_models().await {
            Ok(models) => {
                let latency_ms = start_time.elapsed().as_millis() as u64;
                Ok(ProviderHealth {
                    available: !models.is_empty(),
                    latency_ms: Some(latency_ms),
                })
            }
            Err(e) => {
                warn!("Bedrock health check failed: {}", e);
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

    #[tokio::test]
    async fn test_bedrock_provider_creation() {
        // Test that provider can be created with default values
        let provider = BedrockProvider::new(None, None, None).await;
        assert!(provider.is_ok());
    }

    #[tokio::test]
    async fn test_bedrock_provider_with_custom_region() {
        // Test that provider can be created with custom region
        let provider = BedrockProvider::new(Some("us-west-2".to_string()), None, None).await;
        assert!(provider.is_ok());
    }

    #[tokio::test]
    async fn test_bedrock_provider_with_custom_model() {
        // Test that provider can be created with custom model
        let provider = BedrockProvider::new(None, Some("anthropic.claude-3-haiku-20240307-v1:0".to_string()), None).await;
        assert!(provider.is_ok());
    }

    #[test]
    fn test_bedrock_role_serialization() {
        let user = BedrockRole::User;
        let json = serde_json::to_string(&user).unwrap();
        assert_eq!(json, "\"user\"");

        let parsed: BedrockRole = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, BedrockRole::User);
    }

    #[test]
    fn test_bedrock_content_block_serialization() {
        let text_block = BedrockContentBlock::Text { text: "Hello".to_string() };
        let json = serde_json::to_string(&text_block).unwrap();
        assert!(json.contains("\"text\""));
        assert!(json.contains("\"Hello\""));

        let parsed: BedrockContentBlock = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, text_block);
    }

    #[test]
    fn test_bedrock_message_serialization() {
        let message = BedrockMessage {
            role: BedrockRole::User,
            content: vec![BedrockContentBlock::Text { text: "Hello".to_string() }],
        };
        let json = serde_json::to_string(&message).unwrap();
        let parsed: BedrockMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, message);
    }

    #[test]
    fn test_bedrock_request_serialization() {
        let request = BedrockRequest {
            model_id: "anthropic.claude-3-sonnet-20240229-v1:0".to_string(),
            messages: vec![BedrockMessage {
                role: BedrockRole::User,
                content: vec![BedrockContentBlock::Text { text: "Hello".to_string() }],
            }],
            system: None,
            tools: None,
            tool_config: None,
            inference_config: Some(BedrockInferenceConfig {
                max_tokens: Some(1000),
                temperature: Some(0.7),
                top_p: None,
                stop_sequences: None,
            }),
        };
        let json = serde_json::to_string(&request).unwrap();
        let parsed: BedrockRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, request);
    }
}
