//! Anthropic-specific request/response types.
//!
//! This module defines types used to serialize/deserialize requests and
//! responses for the Anthropic Messages API.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Anthropic message role.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnthropicRole {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
}

impl fmt::Display for AnthropicRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnthropicRole::User => write!(f, "user"),
            AnthropicRole::Assistant => write!(f, "assistant"),
        }
    }
}

/// Reason why an Anthropic message stopped generating.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnthropicStopReason {
    #[serde(rename = "end_turn")]
    EndTurn,
    #[serde(rename = "max_tokens")]
    MaxTokens,
    #[serde(rename = "stop_sequence")]
    StopSequence,
    #[serde(rename = "tool_use")]
    ToolUse,
}

/// A single message in an Anthropic request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnthropicMessage {
    pub role: AnthropicRole,
    pub content: Vec<AnthropicContentBlock>,
}

/// A content block within an Anthropic message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AnthropicContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image {
        source: AnthropicImageSource,
    },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        content: Vec<AnthropicContentBlock>,
        is_error: Option<bool>,
    },
}

/// Image source for Anthropic content blocks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnthropicImageSource {
    #[serde(rename = "type")]
    pub r#type: String,
    pub media_type: String,
    pub data: String,
}

/// Tool definition for Anthropic API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnthropicTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// The main request body for Anthropic Messages API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnthropicRequest {
    pub model: String,
    pub messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<AnthropicTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
    #[serde(default)]
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Anthropic SSE event types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AnthropicSseEvent {
    #[serde(rename = "message_start")]
    MessageStart {
        message: AnthropicMessageEvent,
    },
    #[serde(rename = "content_block_start")]
    ContentBlockStart {
        content_block: AnthropicContentBlock,
        index: usize,
    },
    #[serde(rename = "content_block_delta")]
    ContentBlockDelta {
        delta: AnthropicContentBlockDelta,
        index: usize,
    },
    #[serde(rename = "content_block_stop")]
    ContentBlockStop {
        index: usize,
    },
    #[serde(rename = "message_delta")]
    MessageDelta {
        delta: AnthropicMessageDelta,
        #[serde(default)]
        usage: AnthropicUsage,
    },
    #[serde(rename = "message_stop")]
    MessageStop,
    #[serde(rename = "ping")]
    Ping,
    #[serde(rename = "error")]
    Error {
        error: AnthropicSseError,
    },
}

/// Error object in Anthropic SSE stream.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnthropicSseError {
    pub r#type: String,
    pub message: String,
}

/// Message event in Anthropic SSE stream.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnthropicMessageEvent {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<AnthropicContentBlock>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequence: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<AnthropicUsage>,
}

/// Content block delta in Anthropic SSE stream.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AnthropicContentBlockDelta {
    #[serde(rename = "text_delta")]
    TextDelta { text: String },
    #[serde(rename = "input_json_delta")]
    InputJsonDelta { partial_json: String },
}

/// Message delta in Anthropic SSE stream.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnthropicMessageDelta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<AnthropicStopReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequence: Option<String>,
}

/// Usage statistics in Anthropic API response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct AnthropicUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

/// Error response from Anthropic API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnthropicErrorResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<AnthropicError>,
}

/// Error details from Anthropic API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnthropicError {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// A tool use content block in Anthropic API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnthropicToolUse {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
}

/// A tool result content block in Anthropic API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnthropicToolResult {
    pub id: String,
    pub content: Vec<AnthropicContentBlock>,
}

/// Response containing a tool use.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnthropicToolUseResponse {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
}

/// Response containing a tool result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnthropicToolResultResponse {
    pub id: String,
    pub content: Vec<AnthropicContentBlock>,
}

/// Response containing a message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnthropicMessageResponse {
    pub id: String,
    pub r#type: String,
    pub role: String,
    pub content: Vec<AnthropicContentBlock>,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<AnthropicStopReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequence: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<AnthropicUsage>,
}

/// Model information from Anthropic API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnthropicModelInfo {
    pub id: String,
    pub display_name: String,
    #[serde(rename = "input_cost_per_million_tokens")]
    pub input_cost_per_million_tokens: f64,
    #[serde(rename = "output_cost_per_million_tokens")]
    pub output_cost_per_million_tokens: f64,
    #[serde(rename = "context_window")]
    pub context_window: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub Supports: Option<serde_json::Value>,
}

/// The main response from Anthropic API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnthropicResponse {
    pub id: String,
    pub r#type: String,
    pub role: AnthropicRole,
    pub content: Vec<AnthropicContentBlock>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<AnthropicStopReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequence: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<AnthropicUsage>,
}
