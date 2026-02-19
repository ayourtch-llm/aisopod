//! Core types for the AI model provider abstraction layer.
//!
//! This module defines all the message types and data structures used by the
//! [`crate::ModelProvider`] trait for interacting with AI model providers.

use serde::{Deserialize, Serialize};

/// The role of a message participant in a chat conversation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Role {
    /// A system message that provides high-level instructions to the model.
    #[serde(rename = "system")]
    System,
    /// A user message representing input from the end user.
    #[serde(rename = "user")]
    User,
    /// An assistant message representing the model's response.
    #[serde(rename = "assistant")]
    Assistant,
    /// A tool message representing the result of a tool execution.
    #[serde(rename = "tool")]
    Tool,
}

/// The content of a message, supporting both simple text and multi-modal parts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum MessageContent {
    /// Simple text content.
    Text(String),
    /// Multi-modal content with multiple parts.
    Parts(Vec<ContentPart>),
}

/// A single part of multi-modal content.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ContentPart {
    /// Text content within a multi-modal message.
    Text { text: String },
    /// Image content within a multi-modal message.
    Image { media_type: String, data: String },
}

/// A message in a chat conversation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    /// The role of the message sender.
    pub role: Role,
    /// The content of the message.
    pub content: MessageContent,
    /// Optional list of tool calls triggered by this message.
    #[serde(default)]
    pub tool_calls: Option<Vec<ToolCall>>,
    /// Optional tool call ID for tool messages.
    #[serde(default)]
    pub tool_call_id: Option<String>,
}

/// A definition of a tool that can be used by the model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// The name of the tool.
    pub name: String,
    /// A description of what the tool does.
    pub description: String,
    /// The JSON schema for the tool's parameters.
    pub parameters: serde_json::Value,
}

/// A call to a tool made by the model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCall {
    /// The unique ID of this tool call.
    pub id: String,
    /// The name of the tool to call.
    pub name: String,
    /// The arguments to pass to the tool as a JSON string.
    pub arguments: String,
}

/// A request for chat completion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    /// The model ID to use for completion.
    pub model: String,
    /// The list of messages in the conversation.
    pub messages: Vec<Message>,
    /// Optional list of tools the model may use.
    #[serde(default)]
    pub tools: Option<Vec<ToolDefinition>>,
    /// The sampling temperature (0.0 to 2.0).
    #[serde(default)]
    pub temperature: Option<f32>,
    /// The maximum number of tokens to generate.
    #[serde(default)]
    pub max_tokens: Option<u32>,
    /// Optional stop sequences.
    #[serde(default)]
    pub stop: Option<Vec<String>>,
    /// Whether to stream the response.
    #[serde(default)]
    pub stream: bool,
}

/// The reason why a chat completion finished.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum FinishReason {
    /// The model reached a natural stop point.
    #[serde(rename = "stop")]
    Stop,
    /// The model reached the maximum token limit.
    #[serde(rename = "length")]
    Length,
    /// The model triggered a tool call.
    #[serde(rename = "tool_call")]
    ToolCall,
    /// The content was filtered due to safety policies.
    #[serde(rename = "content_filter")]
    ContentFilter,
    /// An error occurred during completion.
    #[serde(rename = "error")]
    Error,
}

/// Usage statistics for token consumption.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenUsage {
    /// The number of tokens in the prompt.
    pub prompt_tokens: u32,
    /// The number of tokens in the completion.
    pub completion_tokens: u32,
    /// The total number of tokens used.
    pub total_tokens: u32,
}

/// A delta (incremental change) in a streaming chat completion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageDelta {
    /// The role of the message delta.
    #[serde(default)]
    pub role: Option<Role>,
    /// The content delta.
    #[serde(default)]
    pub content: Option<String>,
    /// Optional tool calls in this delta.
    #[serde(default)]
    pub tool_calls: Option<Vec<ToolCall>>,
}

/// A chunk of a streaming chat completion response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatCompletionChunk {
    /// The unique ID of this chunk.
    pub id: String,
    /// The delta content of this chunk.
    pub delta: MessageDelta,
    /// The reason why this chunk finished (if applicable).
    #[serde(default)]
    pub finish_reason: Option<FinishReason>,
    /// Usage statistics for this chunk (only present in the final chunk).
    #[serde(default)]
    pub usage: Option<TokenUsage>,
}

/// Information about a supported model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelInfo {
    /// The unique identifier for the model.
    pub id: String,
    /// The human-readable name of the model.
    pub name: String,
    /// The provider that owns this model.
    pub provider: String,
    /// The maximum context window size in tokens.
    pub context_window: u32,
    /// Whether the model supports vision (multi-modal) inputs.
    pub supports_vision: bool,
    /// Whether the model supports tool calling.
    pub supports_tools: bool,
}

/// The health status of a provider.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProviderHealth {
    /// Whether the provider is available.
    pub available: bool,
    /// The latency in milliseconds, if measured.
    pub latency_ms: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_serialization() {
        let system = Role::System;
        let json = serde_json::to_string(&system).unwrap();
        assert_eq!(json, "\"system\"");

        let parsed: Role = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, Role::System);
    }

    #[test]
    fn test_message_content_serialization() {
        let text = MessageContent::Text("Hello".to_string());
        let json = serde_json::to_string(&text).unwrap();
        assert_eq!(json, "{\"Text\":\"Hello\"}");

        let parsed: MessageContent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, MessageContent::Text("Hello".to_string()));
    }

    #[test]
    fn test_content_part_serialization() {
        let part = ContentPart::Text {
            text: "test".to_string(),
        };
        let json = serde_json::to_string(&part).unwrap();
        assert!(json.contains("\"Text\""));
        assert!(json.contains("\"test\""));
    }

    #[test]
    fn test_message_serialization() {
        let message = Message {
            role: Role::User,
            content: MessageContent::Text("Hello".to_string()),
            tool_calls: None,
            tool_call_id: None,
        };
        let json = serde_json::to_string(&message).unwrap();
        let parsed: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, message);
    }

    #[test]
    fn test_tool_call_serialization() {
        let tool_call = ToolCall {
            id: "call_123".to_string(),
            name: "calculator".to_string(),
            arguments: "{\"operation\":\"add\"}".to_string(),
        };
        let json = serde_json::to_string(&tool_call).unwrap();
        let parsed: ToolCall = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, tool_call);
    }

    #[test]
    fn test_chat_completion_request_serialization() {
        let request = ChatCompletionRequest {
            model: "gpt-4".to_string(),
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
            stream: false,
        };
        let json = serde_json::to_string(&request).unwrap();
        let parsed: ChatCompletionRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, request);
    }

    #[test]
    fn test_finish_reason_serialization() {
        let reason = FinishReason::Stop;
        let json = serde_json::to_string(&reason).unwrap();
        assert_eq!(json, "\"stop\"");

        let parsed: FinishReason = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, FinishReason::Stop);
    }

    #[test]
    fn test_token_usage_serialization() {
        let usage = TokenUsage {
            prompt_tokens: 10,
            completion_tokens: 5,
            total_tokens: 15,
        };
        let json = serde_json::to_string(&usage).unwrap();
        let parsed: TokenUsage = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, usage);
    }

    #[test]
    fn test_message_delta_serialization() {
        let delta = MessageDelta {
            role: Some(Role::Assistant),
            content: Some("Hello!".to_string()),
            tool_calls: None,
        };
        let json = serde_json::to_string(&delta).unwrap();
        let parsed: MessageDelta = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, delta);
    }

    #[test]
    fn test_chat_completion_chunk_serialization() {
        let chunk = ChatCompletionChunk {
            id: "chunk_123".to_string(),
            delta: MessageDelta {
                role: Some(Role::Assistant),
                content: Some("Hello".to_string()),
                tool_calls: None,
            },
            finish_reason: None,
            usage: None,
        };
        let json = serde_json::to_string(&chunk).unwrap();
        let parsed: ChatCompletionChunk = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, chunk);
    }

    #[test]
    fn test_model_info_serialization() {
        let info = ModelInfo {
            id: "gpt-4-turbo".to_string(),
            name: "GPT-4 Turbo".to_string(),
            provider: "OpenAI".to_string(),
            context_window: 128000,
            supports_vision: true,
            supports_tools: true,
        };
        let json = serde_json::to_string(&info).unwrap();
        let parsed: ModelInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, info);
    }

    #[test]
    fn test_provider_health_serialization() {
        let health = ProviderHealth {
            available: true,
            latency_ms: Some(150),
        };
        let json = serde_json::to_string(&health).unwrap();
        let parsed: ProviderHealth = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, health);
    }
}
