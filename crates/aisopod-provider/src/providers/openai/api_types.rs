//! OpenAI-specific request/response types.
//!
//! This module defines types used to serialize/deserialize requests and
//! responses for the OpenAI Chat Completions API.

use serde::{Deserialize, Serialize};

/// Role for a message in the OpenAI API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OpenAIRole {
    #[serde(rename = "system")]
    System,
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
    #[serde(rename = "tool")]
    Tool,
}

/// A single message in an OpenAI request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpenAIMessage {
    pub role: OpenAIRole,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<OpenAIContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<OpenAIToolCall>>,
}

/// Content for an OpenAI message, supporting both text and multi-modal parts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OpenAIContent {
    /// Simple text content.
    Text(String),
    /// Multi-modal content with multiple parts.
    Parts(Vec<OpenAIContentPart>),
}

/// A single part of multi-modal content in OpenAI API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum OpenAIContentPart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image_url")]
    ImageUrl { image_url: OpenAIImageUrl },
}

/// Image URL for OpenAI vision support.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpenAIImageUrl {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// A tool definition for the OpenAI API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpenAITool {
    pub r#type: OpenAIToolType,
    pub function: OpenAIFunctionDefinition,
}

/// The type of tool in the OpenAI API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenAIToolType {
    Function,
}

/// Function definition for a tool.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpenAIFunctionDefinition {
    pub name: String,
    pub description: Option<String>,
    pub parameters: serde_json::Value,
}

/// A tool call made by the model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpenAIToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub tool_type: OpenAIToolType,
    pub function: OpenAIFunctionCall,
}

/// Function call within a tool call.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpenAIFunctionCall {
    pub name: String,
    pub arguments: String,
}

/// The main request body for OpenAI Chat Completions API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpenAIRequest {
    pub model: String,
    pub messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<OpenAITool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<OpenAIToolChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    #[serde(default)]
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<OpenAIResponseFormat>,
}

/// Tool choice setting for OpenAI API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OpenAIToolChoice {
    #[serde(rename = "auto")]
    Auto,
    #[serde(rename = "none")]
    None,
    #[serde(rename = "required")]
    Required,
    Specific(OpenAIToolChoiceSpecific),
}

/// Specific tool choice.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpenAIToolChoiceSpecific {
    #[serde(rename = "type")]
    pub tool_type: OpenAIToolType,
    pub function: OpenAIFunctionChoice,
}

/// Function choice within tool choice.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpenAIFunctionChoice {
    pub name: String,
}

/// Response format for JSON mode.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpenAIResponseFormat {
    #[serde(rename = "type")]
    pub response_type: String,
}

/// OpenAI SSE event types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "object")]
pub enum OpenAISseEvent {
    #[serde(rename = "chat.completion")]
    ChatCompletion,
    #[serde(rename = "chat.completion.chunk")]
    ChatCompletionChunk {
        id: String,
        created: u64,
        model: String,
        choices: Vec<OpenAIChoice>,
        usage: Option<OpenAIUsage>,
    },
}

/// A choice in the OpenAI response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpenAIChoice {
    pub index: usize,
    #[serde(default)]
    pub delta: OpenAIChoiceDelta,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}

/// Delta content in a streaming choice.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct OpenAIChoiceDelta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<OpenAIRole>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<OpenAIToolCall>>,
}

/// Usage statistics in OpenAI API response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpenAIUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Error response from OpenAI API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpenAIErrorResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<OpenAIError>,
}

/// Error details from OpenAI API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpenAIError {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

/// Response from OpenAI list models endpoint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpenAIListModelResponse {
    pub object: String,
    pub data: Vec<OpenAIModel>,
}

/// Model information from OpenAI list models response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpenAIModel {
    pub id: String,
    pub object: String,
    pub created: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owned_by: Option<String>,
}
