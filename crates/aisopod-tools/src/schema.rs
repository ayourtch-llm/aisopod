//! Tool schema normalization for provider-specific formats.
//!
//! This module provides conversion functions that transform internal tool
//! definitions into the provider-specific formats required by Anthropic,
//! OpenAI, and Google Gemini for function calling / tool use.
//!
//! # Overview
//!
//! Each AI provider has a slightly different format for tool/function
//! definitions. This module centralizes the conversion logic so that tool
//! definitions remain provider-agnostic.
//!
//! # Example
//!
//! ```ignore
//! use aisopod_tools::schema::{ToolDefinition, to_anthropic_format, to_openai_format, to_gemini_format};
//! use serde_json::json;
//!
//! let tool = ToolDefinition {
//!     name: "calculator".to_string(),
//!     description: "A calculator tool".to_string(),
//!     parameters: json!({
//!         "type": "object",
//!         "properties": {}
//!     }),
//! };
//!
//! // Convert to Anthropic format
//! let anthropic = to_anthropic_format(&tool);
//!
//! // Convert to OpenAI format
//! let openai = to_openai_format(&tool);
//!
//! // Convert to Gemini format
//! let gemini = to_gemini_format(&tool);
//! ```

use serde_json::{json, Value};

/// A definition of a tool with its metadata and parameter schema.
///
/// This is the internal representation used by aisopod-tools. Conversion
/// functions in this module transform this into provider-specific formats.
#[derive(Debug, Clone, PartialEq)]
pub struct ToolDefinition {
    /// The unique name of this tool.
    pub name: String,
    /// A human-readable description of what the tool does.
    pub description: String,
    /// The JSON Schema defining the tool's expected parameters.
    pub parameters: Value,
}

impl ToolDefinition {
    /// Creates a new ToolDefinition with the given name, description, and parameters.
    pub fn new(name: impl Into<String>, description: impl Into<String>, parameters: Value) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parameters,
        }
    }
}

/// Converts a ToolDefinition to Anthropic's tool format.
///
/// Anthropic uses the format:
/// ```json
/// {
///   "name": "<tool_name>",
///   "description": "<description>",
///   "input_schema": { ...parameters_schema... }
/// }
/// ```
///
/// # Arguments
///
/// * `tool` - The ToolDefinition to convert
///
/// # Returns
///
/// A serde_json::Value representing the tool in Anthropic format.
///
/// # Example
///
/// ```ignore
/// let tool = ToolDefinition::new("test", "A test tool", json!({}));
/// let anthropic = to_anthropic_format(&tool);
/// assert_eq!(anthropic["name"], "test");
/// assert_eq!(anthropic["description"], "A test tool");
/// assert_eq!(anthropic["input_schema"], json!({}));
/// ```
pub fn to_anthropic_format(tool: &ToolDefinition) -> Value {
    json!({
        "name": tool.name,
        "description": tool.description,
        "input_schema": tool.parameters
    })
}

/// Converts a ToolDefinition to OpenAI's function calling format.
///
/// OpenAI uses the format:
/// ```json
/// {
///   "type": "function",
///   "function": {
///     "name": "<tool_name>",
///     "description": "<description>",
///     "parameters": { ...parameters_schema... }
///   }
/// }
/// ```
///
/// # Arguments
///
/// * `tool` - The ToolDefinition to convert
///
/// # Returns
///
/// A serde_json::Value representing the tool in OpenAI format.
///
/// # Example
///
/// ```ignore
/// let tool = ToolDefinition::new("test", "A test tool", json!({}));
/// let openai = to_openai_format(&tool);
/// assert_eq!(openai["type"], "function");
/// assert_eq!(openai["function"]["name"], "test");
/// ```
pub fn to_openai_format(tool: &ToolDefinition) -> Value {
    json!({
        "type": "function",
        "function": {
            "name": tool.name,
            "description": tool.description,
            "parameters": tool.parameters
        }
    })
}

/// Converts a ToolDefinition to Google Gemini's function declaration format.
///
/// Gemini uses the format:
/// ```json
/// {
///   "name": "<tool_name>",
///   "description": "<description>",
///   "parameters": { ...parameters_schema... }
/// }
/// ```
///
/// # Arguments
///
/// * `tool` - The ToolDefinition to convert
///
/// # Returns
///
/// A serde_json::Value representing the tool in Gemini format.
///
/// # Example
///
/// ```ignore
/// let tool = ToolDefinition::new("test", "A test tool", json!({}));
/// let gemini = to_gemini_format(&tool);
/// assert_eq!(gemini["name"], "test");
/// ```
pub fn to_gemini_format(tool: &ToolDefinition) -> Value {
    json!({
        "name": tool.name,
        "description": tool.description,
        "parameters": tool.parameters
    })
}

/// Converts a vector of ToolDefinitions to Anthropic's tool format.
///
/// This is a batch conversion function that produces an array of tool
/// definitions in Anthropic format.
///
/// # Arguments
///
/// * `tools` - A slice of ToolDefinitions to convert
///
/// # Returns
///
/// A serde_json::Value representing an array of tools in Anthropic format.
///
/// # Example
///
/// ```ignore
/// let tools = vec![
///     ToolDefinition::new("tool_a", "First tool", json!({})),
///     ToolDefinition::new("tool_b", "Second tool", json!({})),
/// ];
/// let anthropic = to_anthropic_batch(&tools);
/// assert_eq!(anthropic.as_array().unwrap().len(), 2);
/// ```
pub fn to_anthropic_batch(tools: &[ToolDefinition]) -> Value {
    let converted: Vec<Value> = tools.iter().map(to_anthropic_format).collect();
    json!(converted)
}

/// Converts a vector of ToolDefinitions to OpenAI's function calling format.
///
/// This is a batch conversion function that produces an array of tool
/// definitions in OpenAI format.
///
/// # Arguments
///
/// * `tools` - A slice of ToolDefinitions to convert
///
/// # Returns
///
/// A serde_json::Value representing an array of tools in OpenAI format.
///
/// # Example
///
/// ```ignore
/// let tools = vec![
///     ToolDefinition::new("tool_a", "First tool", json!({})),
///     ToolDefinition::new("tool_b", "Second tool", json!({})),
/// ];
/// let openai = to_openai_batch(&tools);
/// assert_eq!(openai.as_array().unwrap().len(), 2);
/// ```
pub fn to_openai_batch(tools: &[ToolDefinition]) -> Value {
    let converted: Vec<Value> = tools.iter().map(to_openai_format).collect();
    json!(converted)
}

/// Converts a vector of ToolDefinitions to Google Gemini's function declaration format.
///
/// This is a batch conversion function that produces an array of tool
/// definitions in Gemini format.
///
/// # Arguments
///
/// * `tools` - A slice of ToolDefinitions to convert
///
/// # Returns
///
/// A serde_json::Value representing an array of tools in Gemini format.
///
/// # Example
///
/// ```ignore
/// let tools = vec![
///     ToolDefinition::new("tool_a", "First tool", json!({})),
///     ToolDefinition::new("tool_b", "Second tool", json!({})),
/// ];
/// let gemini = to_gemini_batch(&tools);
/// assert_eq!(gemini.as_array().unwrap().len(), 2);
/// ```
pub fn to_gemini_batch(tools: &[ToolDefinition]) -> Value {
    let converted: Vec<Value> = tools.iter().map(to_gemini_format).collect();
    json!(converted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_definition_new() {
        let tool = ToolDefinition::new("test_tool", "A test tool", json!({"type": "object"}));

        assert_eq!(tool.name, "test_tool");
        assert_eq!(tool.description, "A test tool");
        assert_eq!(tool.parameters, json!({"type": "object"}));
    }

    #[test]
    fn test_to_anthropic_format_single_tool() {
        let tool = ToolDefinition::new(
            "calculator",
            "A simple calculator",
            json!({
                "type": "object",
                "properties": {
                    "operation": {"type": "string"},
                    "value": {"type": "number"}
                }
            }),
        );

        let result = to_anthropic_format(&tool);

        assert_eq!(result["name"], "calculator");
        assert_eq!(result["description"], "A simple calculator");
        assert_eq!(result["input_schema"]["type"], "object");
    }

    #[test]
    fn test_to_openai_format_single_tool() {
        let tool = ToolDefinition::new(
            "calculator",
            "A simple calculator",
            json!({
                "type": "object",
                "properties": {}
            }),
        );

        let result = to_openai_format(&tool);

        assert_eq!(result["type"], "function");
        assert_eq!(result["function"]["name"], "calculator");
        assert_eq!(result["function"]["description"], "A simple calculator");
        assert_eq!(result["function"]["parameters"]["type"], "object");
    }

    #[test]
    fn test_to_gemini_format_single_tool() {
        let tool = ToolDefinition::new(
            "calculator",
            "A simple calculator",
            json!({
                "type": "object",
                "properties": {}
            }),
        );

        let result = to_gemini_format(&tool);

        assert_eq!(result["name"], "calculator");
        assert_eq!(result["description"], "A simple calculator");
        assert_eq!(result["parameters"]["type"], "object");
    }

    #[test]
    fn test_anthropic_batch_conversion() {
        let tools = vec![
            ToolDefinition::new("tool_a", "First tool", json!({})),
            ToolDefinition::new("tool_b", "Second tool", json!({})),
        ];

        let result = to_anthropic_batch(&tools);

        assert!(result.is_array());
        assert_eq!(result.as_array().unwrap().len(), 2);
        assert_eq!(result[0]["name"], "tool_a");
        assert_eq!(result[1]["name"], "tool_b");
    }

    #[test]
    fn test_openai_batch_conversion() {
        let tools = vec![
            ToolDefinition::new("tool_a", "First tool", json!({})),
            ToolDefinition::new("tool_b", "Second tool", json!({})),
        ];

        let result = to_openai_batch(&tools);

        assert!(result.is_array());
        assert_eq!(result.as_array().unwrap().len(), 2);
        assert_eq!(result[0]["type"], "function");
        assert_eq!(result[1]["function"]["name"], "tool_b");
    }

    #[test]
    fn test_gemini_batch_conversion() {
        let tools = vec![
            ToolDefinition::new("tool_a", "First tool", json!({})),
            ToolDefinition::new("tool_b", "Second tool", json!({})),
        ];

        let result = to_gemini_batch(&tools);

        assert!(result.is_array());
        assert_eq!(result.as_array().unwrap().len(), 2);
        assert_eq!(result[0]["name"], "tool_a");
        assert_eq!(result[1]["parameters"], json!({}));
    }

    #[test]
    fn test_empty_batch_conversion() {
        let tools: Vec<ToolDefinition> = vec![];

        assert!(to_anthropic_batch(&tools).is_array());
        assert!(to_openai_batch(&tools).is_array());
        assert!(to_gemini_batch(&tools).is_array());

        assert_eq!(to_anthropic_batch(&tools).as_array().unwrap().len(), 0);
        assert_eq!(to_openai_batch(&tools).as_array().unwrap().len(), 0);
        assert_eq!(to_gemini_batch(&tools).as_array().unwrap().len(), 0);
    }
}
