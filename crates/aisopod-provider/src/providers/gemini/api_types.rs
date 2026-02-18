//! Gemini-specific request/response types.
//!
//! This module defines types used to serialize/deserialize requests and
//! responses for the Google Gemini API (Generation API).

use serde::{Deserialize, Serialize};

/// Gemini model role.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GeminiRole {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "model")]
    Model,
}

/// A single part of content in a Gemini message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "role", content = "parts")]
pub enum GeminiPart {
    Text { text: String },
}

/// A content part for images in Gemini.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeminiInlineData {
    pub mime_type: String,
    pub data: String,
}

/// Function declaration for tool calling.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeminiFunctionDeclaration {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
}

/// Tool definition in Gemini API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeminiTool {
    #[serde(rename = "type")]
    pub tool_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_declarations: Option<Vec<GeminiFunctionDeclaration>>,
}

/// Function call response from Gemini.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeminiFunctionCall {
    pub name: String,
    pub args: serde_json::Value,
}

/// A candidate in the Gemini response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeminiCandidate {
    pub content: GeminiContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_usage: Option<GeminiTokenUsage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parts: Option<Vec<GeminiPart>>,
}

/// Content in a Gemini message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeminiContent {
    #[serde(rename = "role")]
    pub role: Option<GeminiRole>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parts: Option<Vec<GeminiPart>>,
}

/// A request part that can contain text or image data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GeminiRequestPart {
    Text(String),
    Image(GeminiInlineData),
}

/// The main request body for Gemini API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeminiRequest {
    pub contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<GeminiTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_config: Option<GeminiToolConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
}

/// Tool configuration for Gemini API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeminiToolConfig {
    #[serde(rename = "type")]
    pub tool_type: String,
}

/// A streaming response chunk from Gemini.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeminiStreamResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidates: Option<Vec<GeminiCandidate>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage_metadata: Option<GeminiTokenUsage>,
}

/// Token usage in Gemini API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeminiTokenUsage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_token_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidates_token_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_token_count: Option<u32>,
}

/// Error response from Gemini API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeminiErrorResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<GeminiError>,
}

/// Error details from Gemini API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeminiError {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// Response from Gemini list models endpoint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeminiListModelResponse {
    pub models: Vec<GeminiModel>,
}

/// Model information from Gemini list models response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeminiModel {
    pub name: String,
    pub base_model_id: String,
    pub version: String,
    pub display_name: String,
    pub description: String,
    pub input_token_limit: u32,
    pub output_token_limit: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported_generation_methods: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_settings: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_parameters: Option<serde_json::Value>,
}

/// Function response for tool results in Gemini.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeminiFunctionResponse {
    pub name: String,
    pub response: serde_json::Value,
}
