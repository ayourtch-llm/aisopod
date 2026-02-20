//! Schema normalization tests for tool definitions.

use aisopod_tools::schema::{
    to_anthropic_batch, to_anthropic_format, to_gemini_batch, to_gemini_format, to_openai_batch,
    to_openai_format, ToolDefinition,
};

#[test]
fn test_tool_definition_creation() {
    let tool = ToolDefinition::new(
        "calculator",
        "A calculator tool",
        serde_json::json!({"type": "object"}),
    );

    assert_eq!(tool.name, "calculator");
    assert_eq!(tool.description, "A calculator tool");
    assert_eq!(tool.parameters, serde_json::json!({"type": "object"}));
}

#[test]
fn test_to_anthropic_format_single_tool() {
    let tool = ToolDefinition::new(
        "calculator",
        "A calculator",
        serde_json::json!({
            "type": "object",
            "properties": {
                "operation": {"type": "string"},
                "value": {"type": "number"}
            }
        }),
    );

    let result = to_anthropic_format(&tool);

    assert_eq!(result["name"], "calculator");
    assert_eq!(result["description"], "A calculator");
    assert_eq!(result["input_schema"]["type"], "object");
}

#[test]
fn test_to_openai_format_single_tool() {
    let tool = ToolDefinition::new(
        "calculator",
        "A calculator",
        serde_json::json!({
            "type": "object",
            "properties": {}
        }),
    );

    let result = to_openai_format(&tool);

    assert_eq!(result["type"], "function");
    assert_eq!(result["function"]["name"], "calculator");
    assert_eq!(result["function"]["description"], "A calculator");
    assert_eq!(result["function"]["parameters"]["type"], "object");
}

#[test]
fn test_to_gemini_format_single_tool() {
    let tool = ToolDefinition::new(
        "calculator",
        "A calculator",
        serde_json::json!({
            "type": "object",
            "properties": {}
        }),
    );

    let result = to_gemini_format(&tool);

    assert_eq!(result["name"], "calculator");
    assert_eq!(result["description"], "A calculator");
    assert_eq!(result["parameters"]["type"], "object");
}

#[test]
fn test_anthropic_batch_conversion() {
    let tools = vec![
        ToolDefinition::new("tool_a", "First tool", serde_json::json!({})),
        ToolDefinition::new("tool_b", "Second tool", serde_json::json!({})),
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
        ToolDefinition::new("tool_a", "First tool", serde_json::json!({})),
        ToolDefinition::new("tool_b", "Second tool", serde_json::json!({})),
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
        ToolDefinition::new("tool_a", "First tool", serde_json::json!({})),
        ToolDefinition::new("tool_b", "Second tool", serde_json::json!({})),
    ];

    let result = to_gemini_batch(&tools);

    assert!(result.is_array());
    assert_eq!(result.as_array().unwrap().len(), 2);
    assert_eq!(result[0]["name"], "tool_a");
    assert_eq!(result[1]["parameters"], serde_json::json!({}));
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

#[test]
fn test_tool_definition_with_complex_parameters() {
    let tool = ToolDefinition::new(
        "complex_tool",
        "A tool with complex parameters",
        serde_json::json!({
            "type": "object",
            "properties": {
                "nested": {
                    "type": "object",
                    "properties": {
                        "deep": {
                            "type": "object",
                            "properties": {
                                "value": {"type": "string"}
                            }
                        }
                    }
                },
                "array": {
                    "type": "array",
                    "items": {"type": "number"}
                }
            }
        }),
    );

    assert_eq!(tool.name, "complex_tool");
    assert!(tool.parameters.is_object());
}

#[test]
fn test_tool_definition_empty_name() {
    let tool = ToolDefinition::new("", "A tool", serde_json::json!({}));

    assert_eq!(tool.name, "");
}

#[test]
fn test_tool_definition_empty_description() {
    let tool = ToolDefinition::new("tool", "", serde_json::json!({}));

    assert_eq!(tool.description, "");
}
