//! Anthropic-specific request/response types.
//!
//! This module defines types used to serialize/deserialize requests and
//! responses for the Anthropic Messages API.

use serde::{Deserialize, Serialize};

/// Anthropic message role.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnthropicRole {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
}

/// A single message in an Anthropic request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnthropicMessage {
    pub role: AnthropicRole,
    pub content: Vec<AnthropicContentBlock>,
}

/// A content block within an Anthropic message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnthropicContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image {
        source: AnthropicImageSource,
        #[serde(rename = "type")]
        _type: String,
    },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
        #[serde(rename = "type")]
        _type: String,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        content: Vec<AnthropicContentBlock>,
        is_error: Option<bool>,
        #[serde(rename = "type")]
        _type: String,
    },
}

/// Image source for Anthropic content blocks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnthropicImageSource {
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
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<AnthropicTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
    #[serde(default)]
    pub stream: bool,
}

/// Anthropic SSE event types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnthropicSseEvent {
    #[serde(rename = "message_start")]
    MessageStart {
        message: AnthropicMessageEvent,
        #[serde(rename = "type")]
        _type: String,
    },
    #[serde(rename = "content_block_start")]
    ContentBlockStart {
        content_block: AnthropicContentBlock,
        index: usize,
        #[serde(rename = "type")]
        _type: String,
    },
    #[serde(rename = "content_block_delta")]
    ContentBlockDelta {
        delta: AnthropicContentBlockDelta,
        index: usize,
        #[serde(rename = "type")]
        _type: String,
    },
    #[serde(rename = "content_block_stop")]
    ContentBlockStop {
        index: usize,
        #[serde(rename = "type")]
        _type: String,
    },
    #[serde(rename = "message_delta")]
    MessageDelta {
        delta: AnthropicMessageDelta,
        usage: AnthropicUsage,
        #[serde(rename = "type")]
        _type: String,
    },
    #[serde(rename = "message_stop")]
    MessageStop {
        #[serde(rename = "type")]
        _type: String,
    },
    #[serde(rename = "ping")]
    Ping,
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
    pub stop_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequence: Option<String>,
}

/// Usage statistics in Anthropic API response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
