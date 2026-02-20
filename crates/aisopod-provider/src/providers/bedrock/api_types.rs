#![allow(clippy::all)]
//! Bedrock-specific request/response types.
//!
//! This module defines types used to serialize/deserialize requests and
//! responses for the AWS Bedrock Converse API.

use serde::{Deserialize, Serialize};

/// Bedrock message role.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BedrockRole {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
}

/// A single message in a Bedrock request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BedrockMessage {
    pub role: BedrockRole,
    pub content: Vec<BedrockContentBlock>,
}

/// A content block within a Bedrock message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum BedrockContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image {
        format: BedrockImageFormat,
        source: BedrockImageSource,
    },
    #[serde(rename = "tool_use")]
    ToolUse {
        tool_use_id: String,
        name: String,
        input: BedrockToolInput,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        content: Vec<BedrockToolResultContent>,
        is_error: Option<bool>,
    },
}

/// Image format for Bedrock content blocks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BedrockImageFormat {
    #[serde(rename = "png")]
    Png,
    #[serde(rename = "jpeg")]
    Jpeg,
    #[serde(rename = "gif")]
    Gif,
    #[serde(rename = "webp")]
    Webp,
}

/// Image source for Bedrock content blocks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum BedrockImageSource {
    #[serde(rename = "bytes")]
    Bytes { data: String },
}

/// Tool input for Bedrock tool use blocks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BedrockToolInput {
    #[serde(flatten)]
    pub inner: serde_json::Value,
}

/// Tool result content for Bedrock tool result blocks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum BedrockToolResultContent {
    #[serde(rename = "text")]
    Text { text: String },
}

/// Tool definition for Bedrock API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BedrockTool {
    pub tool_spec: BedrockToolSpec,
}

/// Tool specification for Bedrock API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BedrockToolSpec {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: BedrockToolInputSchema,
}

/// Tool input schema for Bedrock API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BedrockToolInputSchema {
    #[serde(rename = "json")]
    pub json: serde_json::Value,
}

/// The main request body for Bedrock Converse API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BedrockRequest {
    pub model_id: String,
    pub messages: Vec<BedrockMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<Vec<BedrockSystemMessage>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<BedrockTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_config: Option<BedrockToolConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inference_config: Option<BedrockInferenceConfig>,
}

/// System message for Bedrock API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BedrockSystemMessage {
    #[serde(rename = "text")]
    pub text_content: String,
}

/// Tool configuration for Bedrock API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BedrockToolConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<BedrockToolChoice>,
}

/// Tool choice setting for Bedrock API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BedrockToolChoice {
    Auto,
    Any,
    None,
    Tool(BedrockToolChoiceSpecific),
}

/// Specific tool choice.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BedrockToolChoiceSpecific {
    pub name: String,
}

/// Inference configuration for Bedrock API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BedrockInferenceConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
}

/// Bedrock streaming event types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "contentType")]
pub enum BedrockStreamEvent {
    #[serde(rename = "application/vnd.amazon.eventstream")]
    EventStream {
        header: serde_json::Value,
        payload: serde_json::Value,
    },
    #[serde(rename = "application/json")]
    ContentBlockStart {
        content_block_index: usize,
        content_block: BedrockContentBlock,
        #[serde(rename = "type")]
        event_type: String,
    },
    #[serde(rename = "application/json")]
    ContentBlockDelta {
        content_block_index: usize,
        delta: BedrockContentBlockDelta,
        #[serde(rename = "type")]
        event_type: String,
    },
    #[serde(rename = "application/json")]
    ContentBlockStop {
        content_block_index: usize,
        #[serde(rename = "type")]
        event_type: String,
    },
    #[serde(rename = "application/json")]
    MessageStart {
        message: BedrockMessageEvent,
        #[serde(rename = "type")]
        event_type: String,
    },
    #[serde(rename = "application/json")]
    MessageStop {
        stop_reason: String,
        #[serde(rename = "type")]
        event_type: String,
    },
    #[serde(rename = "application/json")]
    InternalException {
        message: String,
        #[serde(rename = "type")]
        event_type: String,
    },
    #[serde(rename = "application/json")]
    LimitExceededException {
        message: String,
        #[serde(rename = "type")]
        event_type: String,
    },
    #[serde(rename = "application/json")]
    AccessDeniedException {
        message: String,
        #[serde(rename = "type")]
        event_type: String,
    },
    #[serde(rename = "application/json")]
    ModelTimeoutException {
        message: String,
        #[serde(rename = "type")]
        event_type: String,
    },
    #[serde(rename = "application/json")]
    ValidationException {
        message: String,
        #[serde(rename = "type")]
        event_type: String,
    },
}

/// Message event in Bedrock streaming response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BedrockMessageEvent {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<BedrockRole>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<BedrockContentBlock>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<BedrockUsage>,
}

/// Content block delta in Bedrock streaming response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum BedrockContentBlockDelta {
    #[serde(rename = "text")]
    TextDelta { text: String },
    #[serde(rename = "tool_use")]
    ToolUseDelta { tool_use_id: String, input: String },
}

/// Usage statistics in Bedrock API response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BedrockUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_tokens: Option<u32>,
}

/// Error response from Bedrock API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BedrockErrorResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}
