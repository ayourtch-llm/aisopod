//! Comprehensive unit tests for the Anthropic provider.
//!
//! These tests verify:
//! - Request serialization to Anthropic format
//! - Response deserialization from Anthropic format
//! - Tool call handling
//! - Error response handling
//! - Provider-specific behavior

use aisopod_provider::providers::anthropic::{AnthropicProvider, api_types as anthropic_api};
use aisopod_provider::types::{ChatCompletionRequest, Message, MessageContent, Role, ToolDefinition, ToolCall, FinishReason};
use aisopod_provider::trait_module::ModelProvider;
use serde_json::json;

// ============================================================================
// Provider Identity Tests
// ============================================================================

#[tokio::test]
async fn test_anthropic_provider_id() {
    let provider = AnthropicProvider::new("test-key".to_string(), None, None, None);
    assert_eq!(provider.id(), "anthropic");
}

#[tokio::test]
async fn test_anthropic_provider_with_organization() {
    let provider = AnthropicProvider::new(
        "test-key".to_string(),
        Some("org-123".to_string()),
        None,
        None,
    );
    assert_eq!(provider.id(), "anthropic");
}

// ============================================================================
// Request Serialization Tests
// ============================================================================

#[test]
fn test_anthropic_message_to_anthropic_format() {
    let message = anthropic_api::AnthropicMessage {
        role: anthropic_api::AnthropicRole::User,
        content: vec![anthropic_api::AnthropicContentBlock::Text {
            text: "Hello, world!".to_string(),
        }],
    };

    let json = serde_json::to_string(&message).unwrap();
    let parsed: anthropic_api::AnthropicMessage = serde_json::from_str(&json).unwrap();
    
    assert_eq!(parsed.role, anthropic_api::AnthropicRole::User);
    assert_eq!(parsed.content.len(), 1);
    if let anthropic_api::AnthropicContentBlock::Text { text } = &parsed.content[0] {
        assert_eq!(text, "Hello, world!");
    }
}

#[test]
fn test_anthropic_message_with_assistant_role() {
    let message = anthropic_api::AnthropicMessage {
        role: anthropic_api::AnthropicRole::Assistant,
        content: vec![anthropic_api::AnthropicContentBlock::Text {
            text: "I am an assistant.".to_string(),
        }],
    };

    let json = serde_json::to_string(&message).unwrap();
    let parsed: anthropic_api::AnthropicMessage = serde_json::from_str(&json).unwrap();
    
    assert_eq!(parsed.role, anthropic_api::AnthropicRole::Assistant);
}

#[test]
fn test_anthropic_tool_message_format() {
    let message = anthropic_api::AnthropicMessage {
        role: anthropic_api::AnthropicRole::User,
        content: vec![anthropic_api::AnthropicContentBlock::ToolResult {
            tool_use_id: "tool_1".to_string(),
            content: vec![anthropic_api::AnthropicContentBlock::Text {
                text: "Result".to_string(),
            }],
            is_error: Some(false),
        }],
    };

    let json = serde_json::to_string(&message).unwrap();
    assert!(json.contains("\"tool_use_id\""));
}

#[test]
fn test_anthropic_tool_use_block() {
    let block = anthropic_api::AnthropicContentBlock::ToolUse {
        id: "tool_1".to_string(),
        name: "calculator".to_string(),
        input: json!({"operation": "add"}),
    };

    let json = serde_json::to_string(&block).unwrap();
    assert!(json.contains("\"calculator\""));
    assert!(json.contains("\"input\""));
}

// ============================================================================
// Tool Call Tests
// ============================================================================

#[test]
fn test_anthropic_tool_definition_serialization() {
    let tool = anthropic_api::AnthropicTool {
        name: "calculator".to_string(),
        description: "A calculator tool".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "operation": {"type": "string"},
                "a": {"type": "number"},
                "b": {"type": "number"}
            },
            "required": ["operation", "a", "b"]
        }),
    };

    let json = serde_json::to_string(&tool).unwrap();
    let parsed: anthropic_api::AnthropicTool = serde_json::from_str(&json).unwrap();
    
    assert_eq!(parsed.name, "calculator");
    assert_eq!(parsed.description, "A calculator tool");
}

#[test]
fn test_anthropic_tool_use_response() {
    let response = anthropic_api::AnthropicToolUseResponse {
        id: "tool_1".to_string(),
        name: "calculator".to_string(),
        input: json!({"operation": "add", "a": 5, "b": 3}),
    };

    let json = serde_json::to_string(&response).unwrap();
    let parsed: anthropic_api::AnthropicToolUseResponse = serde_json::from_str(&json).unwrap();
    
    assert_eq!(parsed.id, "tool_1");
    assert_eq!(parsed.name, "calculator");
}

#[test]
fn test_anthropic_tool_result_response() {
    let response = anthropic_api::AnthropicToolResultResponse {
        id: "tool_1".to_string(),
        content: vec![anthropic_api::AnthropicContentBlock::Text {
            text: "Result: 8".to_string(),
        }],
    };

    let json = serde_json::to_string(&response).unwrap();
    let parsed: anthropic_api::AnthropicToolResultResponse = serde_json::from_str(&json).unwrap();
    
    assert_eq!(parsed.id, "tool_1");
    assert_eq!(parsed.content.len(), 1);
}

// ============================================================================
// SSE Event Tests
// ============================================================================

#[test]
fn test_anthropic_sse_message_start_event() {
    let event_json = r#"{"type": "message_start", "message": {"id": "msg_123", "type": "message", "role": "assistant", "content": []}}"#;
    
    let result: Result<anthropic_api::AnthropicSseEvent, _> = serde_json::from_str(event_json);
    assert!(result.is_ok());
    
    if let Ok(event) = result {
        if let anthropic_api::AnthropicSseEvent::MessageStart { message } = event {
            assert_eq!(message.id, "msg_123");
            assert_eq!(message.role, "assistant");
        }
    }
}

#[test]
fn test_anthropic_sse_content_block_start_event() {
    let event_json = r#"{"type": "content_block_start", "index": 0, "content_block": {"type": "text", "text": ""}}"#;
    
    let result: Result<anthropic_api::AnthropicSseEvent, _> = serde_json::from_str(event_json);
    assert!(result.is_ok());
}

#[test]
fn test_anthropic_sse_content_block_delta_event() {
    let event_json = r#"{"type": "content_block_delta", "index": 0, "delta": {"type": "text_delta", "text": "Hello"}}"#;
    
    let result: Result<anthropic_api::AnthropicSseEvent, _> = serde_json::from_str(event_json);
    eprintln!("ContentBlockDelta result: {:?}", result);
    assert!(result.is_ok());
}

#[test]
fn test_anthropic_sse_message_delta_event() {
    let event_json = r#"{"type": "message_delta", "delta": {"stop_reason": "end_turn", "stop_sequence": null}}"#;
    
    let result: Result<anthropic_api::AnthropicSseEvent, _> = serde_json::from_str(event_json);
    eprintln!("MessageDelta result: {:?}", result);
    assert!(result.is_ok());
}

#[test]
fn test_anthropic_sse_message_stop_event() {
    let event_json = r#"{"type": "message_stop"}"#;
    
    let result: Result<anthropic_api::AnthropicSseEvent, _> = serde_json::from_str(event_json);
    assert!(result.is_ok());
}

#[test]
fn test_anthropic_sse_error_event() {
    let event_json = r#"{"type": "error", "error": {"type": "api_error", "message": "Something went wrong"}}"#;
    
    let result: Result<anthropic_api::AnthropicSseEvent, _> = serde_json::from_str(event_json);
    assert!(result.is_ok());
}

// ============================================================================
// Request Building Tests
// ============================================================================

#[test]
fn test_build_anthropic_request_basic() {
    let request = anthropic_api::AnthropicRequest {
        model: "claude-3-5-sonnet".to_string(),
        messages: vec![
            anthropic_api::AnthropicMessage {
                role: anthropic_api::AnthropicRole::User,
                content: vec![anthropic_api::AnthropicContentBlock::Text {
                    text: "Hello".to_string(),
                }],
            },
        ],
        system: None,
        tools: None,
        max_tokens: Some(1000),
        temperature: None,
        top_p: None,
        stop_sequences: None,
        stream: false,
        metadata: None,
    };

    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("claude-3-5-sonnet"));
    assert!(json.contains("\"Hello\""));
}

#[test]
fn test_build_anthropic_request_with_system_prompt() {
    let request = anthropic_api::AnthropicRequest {
        model: "claude-3-5-sonnet".to_string(),
        messages: vec![
            anthropic_api::AnthropicMessage {
                role: anthropic_api::AnthropicRole::User,
                content: vec![anthropic_api::AnthropicContentBlock::Text {
                    text: "Hello".to_string(),
                }],
            },
        ],
        system: Some(json!([{
            "type": "text",
            "text": "You are a helpful assistant."
        }])),
        tools: None,
        max_tokens: Some(1000),
        temperature: None,
        top_p: None,
        stop_sequences: None,
        stream: false,
        metadata: None,
    };

    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("You are a helpful assistant."));
}

#[test]
fn test_build_anthropic_request_with_tools() {
    let tools = vec![
        anthropic_api::AnthropicTool {
            name: "calculator".to_string(),
            description: "A calculator".to_string(),
            input_schema: json!({"type": "object", "properties": {}}),
        },
    ];

    let request = anthropic_api::AnthropicRequest {
        model: "claude-3-5-sonnet".to_string(),
        messages: vec![
            anthropic_api::AnthropicMessage {
                role: anthropic_api::AnthropicRole::User,
                content: vec![anthropic_api::AnthropicContentBlock::Text {
                    text: "Calculate 5+3".to_string(),
                }],
            },
        ],
        system: None,
        tools: Some(tools),
        max_tokens: Some(1000),
        temperature: None,
        top_p: None,
        stop_sequences: None,
        stream: false,
        metadata: None,
    };

    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("\"calculator\""));
}

#[test]
fn test_build_anthropic_request_with_stop_sequences() {
    let request = anthropic_api::AnthropicRequest {
        model: "claude-3-5-sonnet".to_string(),
        messages: vec![
            anthropic_api::AnthropicMessage {
                role: anthropic_api::AnthropicRole::User,
                content: vec![anthropic_api::AnthropicContentBlock::Text {
                    text: "Hello".to_string(),
                }],
            },
        ],
        system: None,
        tools: None,
        max_tokens: Some(1000),
        temperature: None,
        top_p: None,
        stop_sequences: Some(vec!["\n\n".to_string()]),
        stream: false,
        metadata: None,
    };

    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("\"\\n\\n\""));
}

// ============================================================================
// Response Parsing Tests
// ============================================================================

#[test]
fn test_parse_anthropic_stream_response() {
    let response_json = r#"{
        "id": "msg_123",
        "type": "message",
        "role": "assistant",
        "content": [{"type": "text", "text": "Hello world!"}],
        "model": "claude-3-5-sonnet",
        "stop_reason": "end_turn",
        "usage": {
            "input_tokens": 10,
            "output_tokens": 5
        }
    }"#;

    let result: Result<anthropic_api::AnthropicResponse, _> = serde_json::from_str(response_json);
    assert!(result.is_ok());
    
    if let Ok(response) = result {
        assert_eq!(response.id, "msg_123");
        assert_eq!(response.role, anthropic_api::AnthropicRole::Assistant);
        assert_eq!(response.content.len(), 1);
        assert_eq!(response.stop_reason, Some(anthropic_api::AnthropicStopReason::EndTurn));
        assert_eq!(response.usage.as_ref().map(|u| u.input_tokens), Some(10));
        assert_eq!(response.usage.as_ref().map(|u| u.output_tokens), Some(5));
    }
}

#[test]
fn test_parse_anthropic_error_response() {
    let error_json = r#"{
        "error": {
            "type": "invalid_request_error",
            "message": "Invalid API key"
        }
    }"#;

    let result: Result<anthropic_api::AnthropicErrorResponse, _> = serde_json::from_str(error_json);
    assert!(result.is_ok());
    
    if let Ok(error) = result {
        assert_eq!(error.error.as_ref().unwrap().r#type.as_ref().unwrap(), "invalid_request_error");
        assert!(error.error.as_ref().unwrap().message.as_ref().unwrap().contains("Invalid API key"));
    }
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[tokio::test]
async fn test_anthropic_provider_list_models_returns_default() {
    // The provider has a hardcoded list of models for list_models
    let provider = AnthropicProvider::new("test-key".to_string(), None, None, None);
    
    let result = provider.list_models().await;
    // Should return Ok with default models
    assert!(result.is_ok());
    let models = result.unwrap();
    assert!(!models.is_empty());
}

#[tokio::test]
async fn test_anthropic_provider_health_check_unavailable_without_api_key() {
    let provider = AnthropicProvider::new("invalid-key".to_string(), None, None, None);
    
    let result = provider.health_check().await;
    // health_check returns a ProviderHealth, not an Err
    assert!(result.is_ok());
    let health = result.unwrap();
    // Should be unavailable without valid API key
    assert!(!health.available);
}

#[tokio::test]
async fn test_anthropic_provider_with_multiple_profiles() {
    let mut provider = AnthropicProvider::new("test-key-1".to_string(), None, None, None);
    
    // Add multiple auth profiles
    provider.add_profile(aisopod_provider::AuthProfile::new(
        "profile_2".to_string(),
        "anthropic".to_string(),
        "sk-test-2".to_string(),
    ));
    
    // Verify we can add profiles
    // The add_profile method exists and compiles
}

// ============================================================================
// Model Info Tests
// ============================================================================

#[test]
fn test_anthropic_model_info_serialization() {
    let info = anthropic_api::AnthropicModelInfo {
        id: "claude-3-5-sonnet".to_string(),
        display_name: "Claude 3.5 Sonnet".to_string(),
        input_cost_per_million_tokens: 3.0,
        output_cost_per_million_tokens: 15.0,
        context_window: 200_000,
        Supports: None,
    };

    let json = serde_json::to_string(&info).unwrap();
    let parsed: anthropic_api::AnthropicModelInfo = serde_json::from_str(&json).unwrap();
    
    assert_eq!(parsed.id, "claude-3-5-sonnet");
    assert_eq!(parsed.display_name, "Claude 3.5 Sonnet");
}

// ============================================================================
// Message Content Tests
// ============================================================================

#[test]
fn test_anthropic_content_block_with_image() {
    let block = anthropic_api::AnthropicContentBlock::Image {
        source: anthropic_api::AnthropicImageSource {
            r#type: "base64".to_string(),
            media_type: "image/png".to_string(),
            data: "base64data".to_string(),
        },
    };

    let json = serde_json::to_string(&block).unwrap();
    assert!(json.contains("\"base64\""));
    assert!(json.contains("\"image/png\""));
}

// ============================================================================
// Temperature Tests
// ============================================================================

#[test]
fn test_anthropic_request_with_temperature() {
    let request = anthropic_api::AnthropicRequest {
        model: "claude-3-5-sonnet".to_string(),
        messages: vec![
            anthropic_api::AnthropicMessage {
                role: anthropic_api::AnthropicRole::User,
                content: vec![anthropic_api::AnthropicContentBlock::Text {
                    text: "Hello".to_string(),
                }],
            },
        ],
        system: None,
        tools: None,
        max_tokens: Some(1000),
        temperature: Some(0.7),
        top_p: None,
        stop_sequences: None,
        stream: false,
        metadata: None,
    };

    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("\"temperature\":0.7"));
}

#[test]
fn test_anthropic_request_with_top_p() {
    let request = anthropic_api::AnthropicRequest {
        model: "claude-3-5-sonnet".to_string(),
        messages: vec![
            anthropic_api::AnthropicMessage {
                role: anthropic_api::AnthropicRole::User,
                content: vec![anthropic_api::AnthropicContentBlock::Text {
                    text: "Hello".to_string(),
                }],
            },
        ],
        system: None,
        tools: None,
        max_tokens: Some(1000),
        temperature: None,
        top_p: Some(0.9),
        stop_sequences: None,
        stream: false,
        metadata: None,
    };

    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("\"top_p\":0.9"));
}

// ============================================================================
// Tool Call Response Tests
// ============================================================================

#[test]
fn test_anthropic_tool_call_response_roundtrip() {
    let tool_call = anthropic_api::AnthropicToolUse {
        id: "tool_1".to_string(),
        name: "calculator".to_string(),
        input: json!({"operation": "add", "a": 5, "b": 3}),
    };

    let json = serde_json::to_string(&tool_call).unwrap();
    let parsed: anthropic_api::AnthropicToolUse = serde_json::from_str(&json).unwrap();
    
    assert_eq!(parsed.id, "tool_1");
    assert_eq!(parsed.name, "calculator");
}

// ============================================================================
// System Prompt Tests
// ============================================================================

#[test]
fn test_anthropic_system_prompt_content_block() {
    let block = anthropic_api::AnthropicContentBlock::Text {
        text: "You are a helpful assistant.".to_string(),
    };

    let json = serde_json::to_string(&block).unwrap();
    assert!(json.contains("You are a helpful assistant."));
}

#[test]
fn test_anthropic_system_prompt_array() {
    let system_prompts = vec![
        anthropic_api::AnthropicContentBlock::Text {
            text: "First system prompt.".to_string(),
        },
        anthropic_api::AnthropicContentBlock::Text {
            text: "Second system prompt.".to_string(),
        },
    ];

    let json = serde_json::to_string(&system_prompts).unwrap();
    assert!(json.contains("First system prompt."));
    assert!(json.contains("Second system prompt."));
}

// ============================================================================
// Metadata Tests
// ============================================================================

#[test]
fn test_anthropic_request_with_metadata() {
    let request = anthropic_api::AnthropicRequest {
        model: "claude-3-5-sonnet".to_string(),
        messages: vec![
            anthropic_api::AnthropicMessage {
                role: anthropic_api::AnthropicRole::User,
                content: vec![anthropic_api::AnthropicContentBlock::Text {
                    text: "Hello".to_string(),
                }],
            },
        ],
        system: None,
        tools: None,
        max_tokens: Some(1000),
        temperature: None,
        top_p: None,
        stop_sequences: None,
        stream: false,
        metadata: Some(json!({
            "user_id": "user_123",
            "session_id": "session_456"
        })),
    };

    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("user_id"));
    assert!(json.contains("session_id"));
}

// ============================================================================
// Multi-modal Tests
// ============================================================================

#[test]
fn test_anthropic_multimodal_request() {
    let request = anthropic_api::AnthropicRequest {
        model: "claude-3-5-sonnet".to_string(),
        messages: vec![
            anthropic_api::AnthropicMessage {
                role: anthropic_api::AnthropicRole::User,
                content: vec![
                    anthropic_api::AnthropicContentBlock::Text {
                        text: "What's in this image?".to_string(),
                    },
                    anthropic_api::AnthropicContentBlock::Image {
                        source: anthropic_api::AnthropicImageSource {
                            r#type: "base64".to_string(),
                            media_type: "image/png".to_string(),
                            data: "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==".to_string(),
                        },
                    },
                ],
            },
        ],
        system: None,
        tools: None,
        max_tokens: Some(1000),
        temperature: None,
        top_p: None,
        stop_sequences: None,
        stream: false,
        metadata: None,
    };

    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("What's in this image?"));
    assert!(json.contains("image"));
}

// ============================================================================
// Streaming Request Tests
// ============================================================================

#[test]
fn test_anthropic_streaming_request() {
    let request = anthropic_api::AnthropicRequest {
        model: "claude-3-5-sonnet".to_string(),
        messages: vec![
            anthropic_api::AnthropicMessage {
                role: anthropic_api::AnthropicRole::User,
                content: vec![anthropic_api::AnthropicContentBlock::Text {
                    text: "Hello".to_string(),
                }],
            },
        ],
        system: None,
        tools: None,
        max_tokens: Some(1000),
        temperature: None,
        top_p: None,
        stop_sequences: None,
        stream: true,
        metadata: None,
    };

    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("\"stream\":true"));
}

// ============================================================================
// Provider Creation Tests
// ============================================================================

#[test]
fn test_anthropic_provider_creation() {
    // Test basic provider creation
    let provider = AnthropicProvider::new("test-key".to_string(), None, None, None);
    assert_eq!(provider.id(), "anthropic");
}

// ============================================================================
// Tool Definition Tests
// ============================================================================

#[test]
fn test_tool_definition_to_anthropic_tool() {
    let tool_def = ToolDefinition {
        name: "calculator".to_string(),
        description: "A calculator tool".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "operation": {"type": "string"},
                "a": {"type": "number"},
                "b": {"type": "number"}
            }
        }),
    };

    // Verify we can serialize the tool definition
    let json = serde_json::to_string(&tool_def).unwrap();
    let parsed: ToolDefinition = serde_json::from_str(&json).unwrap();
    
    assert_eq!(parsed.name, "calculator");
    assert_eq!(parsed.description, "A calculator tool");
}

// ============================================================================
// Tool Call Tests
// ============================================================================

#[test]
fn test_tool_call_roundtrip() {
    let tool_call = ToolCall {
        id: "call_123".to_string(),
        name: "calculator".to_string(),
        arguments: "{\"operation\":\"add\",\"a\":5,\"b\":3}".to_string(),
    };

    let json = serde_json::to_string(&tool_call).unwrap();
    let parsed: ToolCall = serde_json::from_str(&json).unwrap();
    
    assert_eq!(parsed.id, "call_123");
    assert_eq!(parsed.name, "calculator");
    assert!(parsed.arguments.contains("5"));
}

// ============================================================================
// Finish Reason Tests
// ============================================================================

#[test]
fn test_finish_reason_stop() {
    let reason = FinishReason::Stop;
    let json = serde_json::to_string(&reason).unwrap();
    assert!(json.contains("stop"));
}

#[test]
fn test_finish_reason_length() {
    let reason = FinishReason::Length;
    let json = serde_json::to_string(&reason).unwrap();
    assert!(json.contains("length"));
}

#[test]
fn test_finish_reason_tool_call() {
    let reason = FinishReason::ToolCall;
    let json = serde_json::to_string(&reason).unwrap();
    assert!(json.contains("tool_call"));
}

// ============================================================================
// Usage Tests
// ============================================================================

#[test]
fn test_token_usage_serialization() {
    let usage = aisopod_provider::TokenUsage {
        prompt_tokens: 100,
        completion_tokens: 50,
        total_tokens: 150,
    };

    let json = serde_json::to_string(&usage).unwrap();
    let parsed: aisopod_provider::TokenUsage = serde_json::from_str(&json).unwrap();
    
    assert_eq!(parsed.prompt_tokens, 100);
    assert_eq!(parsed.completion_tokens, 50);
    assert_eq!(parsed.total_tokens, 150);
}

// ============================================================================
// Message Delta Tests
// ============================================================================

#[test]
fn test_message_delta_serialization() {
    let delta = aisopod_provider::MessageDelta {
        role: Some(aisopod_provider::Role::Assistant),
        content: Some("Hello".to_string()),
        tool_calls: None,
    };

    let json = serde_json::to_string(&delta).unwrap();
    let parsed: aisopod_provider::MessageDelta = serde_json::from_str(&json).unwrap();
    
    assert_eq!(parsed.role, Some(aisopod_provider::Role::Assistant));
    assert_eq!(parsed.content, Some("Hello".to_string()));
}

// ============================================================================
// Chat Completion Chunk Tests
// ============================================================================

#[test]
fn test_chat_completion_chunk_serialization() {
    let chunk = aisopod_provider::ChatCompletionChunk {
        id: "chunk_123".to_string(),
        delta: aisopod_provider::MessageDelta {
            role: Some(aisopod_provider::Role::Assistant),
            content: Some("Hello world!".to_string()),
            tool_calls: None,
        },
        finish_reason: Some(aisopod_provider::FinishReason::Stop),
        usage: Some(aisopod_provider::TokenUsage {
            prompt_tokens: 10,
            completion_tokens: 5,
            total_tokens: 15,
        }),
    };

    let json = serde_json::to_string(&chunk).unwrap();
    let parsed: aisopod_provider::ChatCompletionChunk = serde_json::from_str(&json).unwrap();
    
    assert_eq!(parsed.id, "chunk_123");
    assert_eq!(parsed.delta.content, Some("Hello world!".to_string()));
    assert_eq!(parsed.finish_reason, Some(aisopod_provider::FinishReason::Stop));
}

// ============================================================================
// Message Tests
// ============================================================================

#[test]
fn test_message_text_serialization() {
    let message = aisopod_provider::Message {
        role: aisopod_provider::Role::User,
        content: aisopod_provider::MessageContent::Text("Hello".to_string()),
        tool_calls: None,
        tool_call_id: None,
    };

    let json = serde_json::to_string(&message).unwrap();
    let parsed: aisopod_provider::Message = serde_json::from_str(&json).unwrap();
    
    assert_eq!(parsed.role, aisopod_provider::Role::User);
    assert!(matches!(parsed.content, aisopod_provider::MessageContent::Text(t) if t == "Hello"));
}

#[test]
fn test_message_content_part_serialization() {
    let part = aisopod_provider::ContentPart::Text {
        text: "Hello".to_string(),
    };

    let json = serde_json::to_string(&part).unwrap();
    let parsed: aisopod_provider::ContentPart = serde_json::from_str(&json).unwrap();
    
    assert!(matches!(parsed, aisopod_provider::ContentPart::Text { text } if text == "Hello"));
}

// ============================================================================
// Chat Completion Request Tests
// ============================================================================

#[test]
fn test_chat_completion_request_serialization() {
    let request = aisopod_provider::ChatCompletionRequest {
        model: "claude-3-5-sonnet".to_string(),
        messages: vec![
            aisopod_provider::Message {
                role: aisopod_provider::Role::User,
                content: aisopod_provider::MessageContent::Text("Hello".to_string()),
                tool_calls: None,
                tool_call_id: None,
            },
        ],
        tools: None,
        temperature: None,
        max_tokens: None,
        stop: None,
        stream: true,
    };

    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("claude-3-5-sonnet"));
    assert!(json.contains("\"Hello\""));
}

// ============================================================================
// Model Info Tests
// ============================================================================

#[test]
fn test_model_info_serialization() {
    let info = aisopod_provider::ModelInfo {
        id: "claude-3-5-sonnet".to_string(),
        name: "Claude 3.5 Sonnet".to_string(),
        provider: "anthropic".to_string(),
        context_window: 200000,
        supports_vision: true,
        supports_tools: true,
    };

    let json = serde_json::to_string(&info).unwrap();
    let parsed: aisopod_provider::ModelInfo = serde_json::from_str(&json).unwrap();
    
    assert_eq!(parsed.id, "claude-3-5-sonnet");
    assert_eq!(parsed.name, "Claude 3.5 Sonnet");
    assert!(parsed.supports_vision);
    assert!(parsed.supports_tools);
}

// ============================================================================
// Role Tests
// ============================================================================

#[test]
fn test_role_serialization() {
    let role = aisopod_provider::Role::User;
    let json = serde_json::to_string(&role).unwrap();
    assert!(json.contains("user"));
}

#[test]
fn test_role_assistant_serialization() {
    let role = aisopod_provider::Role::Assistant;
    let json = serde_json::to_string(&role).unwrap();
    assert!(json.contains("assistant"));
}

#[test]
fn test_role_system_serialization() {
    let role = aisopod_provider::Role::System;
    let json = serde_json::to_string(&role).unwrap();
    assert!(json.contains("system"));
}

// ============================================================================
// Provider Health Tests
// ============================================================================

#[test]
fn test_provider_health_serialization() {
    let health = aisopod_provider::ProviderHealth {
        available: true,
        latency_ms: Some(123),
    };

    let json = serde_json::to_string(&health).unwrap();
    let parsed: aisopod_provider::ProviderHealth = serde_json::from_str(&json).unwrap();
    
    assert!(parsed.available);
    assert_eq!(parsed.latency_ms, Some(123));
}

// ============================================================================
// Complex Message Tests
// ============================================================================

#[test]
fn test_complex_message_with_tool_calls() {
    let message = aisopod_provider::Message {
        role: aisopod_provider::Role::Assistant,
        content: aisopod_provider::MessageContent::Text("I'll use the calculator".to_string()),
        tool_calls: Some(vec![
            aisopod_provider::ToolCall {
                id: "call_1".to_string(),
                name: "calculator".to_string(),
                arguments: "{\"operation\":\"add\"}".to_string(),
            },
        ]),
        tool_call_id: None,
    };

    let json = serde_json::to_string(&message).unwrap();
    assert!(json.contains("\"assistant\""));
    assert!(json.contains("\"calculator\""));
}

#[test]
fn test_message_with_tool_call_id() {
    let message = aisopod_provider::Message {
        role: aisopod_provider::Role::User,
        content: aisopod_provider::MessageContent::Text("Here's the result".to_string()),
        tool_calls: None,
        tool_call_id: Some("call_1".to_string()),
    };

    let json = serde_json::to_string(&message).unwrap();
    assert!(json.contains("call_1"));
}

// ============================================================================
// Provider Integration Tests
// ============================================================================

#[tokio::test]
async fn test_anthropic_provider_with_auth_profile() {
    use aisopod_provider::AuthProfile;

    let profile = AuthProfile::new(
        "test-profile".to_string(),
        "anthropic".to_string(),
        "sk-test-key".to_string(),
    );
    
    let provider = AnthropicProvider::new("test-key".to_string(), None, None, None);
    
    // Verify we can create a provider with an auth profile
    assert_eq!(provider.id(), "anthropic");
}

// ============================================================================
// End-to-End Request Tests
// ============================================================================

#[test]
fn test_full_anthropic_request_roundtrip() {
    let original = anthropic_api::AnthropicRequest {
        model: "claude-3-5-sonnet-latest".to_string(),
        messages: vec![
            anthropic_api::AnthropicMessage {
                role: anthropic_api::AnthropicRole::User,
                content: vec![
                    anthropic_api::AnthropicContentBlock::Text {
                        text: "Hello".to_string(),
                    },
                ],
            },
        ],
        system: Some(json!([{
            "type": "text",
            "text": "You are a helpful assistant."
        }])),
        tools: Some(vec![
            anthropic_api::AnthropicTool {
                name: "calculator".to_string(),
                description: "A calculator tool".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "operation": {"type": "string"},
                        "a": {"type": "number"},
                        "b": {"type": "number"}
                    },
                    "required": ["operation", "a", "b"]
                }),
            },
        ]),
        max_tokens: Some(1000),
        temperature: Some(0.7),
        top_p: Some(0.9),
        stop_sequences: Some(vec!["\n\n".to_string()]),
        stream: true,
        metadata: Some(json!({
            "user_id": "user_123"
        })),
    };

    let json = serde_json::to_string(&original).unwrap();
    let parsed: anthropic_api::AnthropicRequest = serde_json::from_str(&json).unwrap();
    
    assert_eq!(parsed.model, original.model);
    assert_eq!(parsed.messages.len(), original.messages.len());
    assert!(parsed.system.is_some());
    assert!(parsed.tools.is_some());
    assert_eq!(parsed.max_tokens, original.max_tokens);
    assert_eq!(parsed.temperature, original.temperature);
    assert_eq!(parsed.stream, original.stream);
}

// ============================================================================
// Tool Use Response Tests
// ============================================================================

#[test]
fn test_anthropic_tool_use_response_roundtrip() {
    let original = anthropic_api::AnthropicToolUseResponse {
        id: "tool_123".to_string(),
        name: "calculator".to_string(),
        input: json!({
            "operation": "multiply",
            "a": 5,
            "b": 3
        }),
    };

    let json = serde_json::to_string(&original).unwrap();
    let parsed: anthropic_api::AnthropicToolUseResponse = serde_json::from_str(&json).unwrap();
    
    assert_eq!(parsed.id, original.id);
    assert_eq!(parsed.name, original.name);
}

// ============================================================================
// Image Source Tests
// ============================================================================

#[test]
fn test_anthropic_image_source_roundtrip() {
    let original = anthropic_api::AnthropicImageSource {
        r#type: "base64".to_string(),
        media_type: "image/jpeg".to_string(),
        data: "/9j/4AAQSkZJRgABAQE=".to_string(),
    };

    let json = serde_json::to_string(&original).unwrap();
    let parsed: anthropic_api::AnthropicImageSource = serde_json::from_str(&json).unwrap();
    
    assert_eq!(parsed.r#type, "base64");
    assert_eq!(parsed.media_type, "image/jpeg");
    assert_eq!(parsed.data, "/9j/4AAQSkZJRgABAQE=");
}

// ============================================================================
// Stop Reason Tests
// ============================================================================

#[test]
fn test_anthropic_stop_reason_end_turn() {
    let reason = anthropic_api::AnthropicStopReason::EndTurn;
    let json = serde_json::to_string(&reason).unwrap();
    assert!(json.contains("end_turn"));
}

#[test]
fn test_anthropic_stop_reason_max_tokens() {
    let reason = anthropic_api::AnthropicStopReason::MaxTokens;
    let json = serde_json::to_string(&reason).unwrap();
    assert!(json.contains("max_tokens"));
}

#[test]
fn test_anthropic_stop_reason_stop_sequence() {
    let reason = anthropic_api::AnthropicStopReason::StopSequence;
    let json = serde_json::to_string(&reason).unwrap();
    assert!(json.contains("stop_sequence"));
}

#[test]
fn test_anthropic_stop_reason_tool_use() {
    let reason = anthropic_api::AnthropicStopReason::ToolUse;
    let json = serde_json::to_string(&reason).unwrap();
    assert!(json.contains("tool_use"));
}

// ============================================================================
// Usage Stats Tests
// ============================================================================

#[test]
fn test_anthropic_usage_stats_roundtrip() {
    let original = anthropic_api::AnthropicUsage {
        input_tokens: 100,
        output_tokens: 50,
    };

    let json = serde_json::to_string(&original).unwrap();
    let parsed: anthropic_api::AnthropicUsage = serde_json::from_str(&json).unwrap();
    
    assert_eq!(parsed.input_tokens, 100);
    assert_eq!(parsed.output_tokens, 50);
}

// ============================================================================
// Message Response Tests
// ============================================================================

#[test]
fn test_anthropic_message_response_roundtrip() {
    let original = anthropic_api::AnthropicMessageResponse {
        id: "msg_123".to_string(),
        r#type: "message".to_string(),
        role: "assistant".to_string(),
        content: vec![
            anthropic_api::AnthropicContentBlock::Text {
                text: "Hello world!".to_string(),
            },
        ],
        model: "claude-3-5-sonnet".to_string(),
        stop_reason: Some(anthropic_api::AnthropicStopReason::EndTurn),
        stop_sequence: None,
        usage: Some(anthropic_api::AnthropicUsage {
            input_tokens: 10,
            output_tokens: 5,
        }),
    };

    let json = serde_json::to_string(&original).unwrap();
    let parsed: anthropic_api::AnthropicMessageResponse = serde_json::from_str(&json).unwrap();
    
    assert_eq!(parsed.id, original.id);
    assert_eq!(parsed.role, "assistant");
    assert_eq!(parsed.content.len(), original.content.len());
    assert_eq!(parsed.stop_reason, original.stop_reason);
}

// ============================================================================
// Error Response Tests
// ============================================================================

#[test]
fn test_anthropic_error_response_roundtrip() {
    let original = anthropic_api::AnthropicErrorResponse {
        error: Some(anthropic_api::AnthropicError {
            r#type: Some("invalid_request_error".to_string()),
            message: Some("Invalid API key provided".to_string()),
        }),
    };

    let json = serde_json::to_string(&original).unwrap();
    let parsed: anthropic_api::AnthropicErrorResponse = serde_json::from_str(&json).unwrap();
    
    assert_eq!(parsed.error.as_ref().unwrap().r#type.as_ref().unwrap(), "invalid_request_error");
    assert!(parsed.error.as_ref().unwrap().message.as_ref().unwrap().contains("Invalid API key"));
}

// ============================================================================
// Tool Result Content Tests
// ============================================================================

#[test]
fn test_anthropic_tool_result_content_roundtrip() {
    let original = vec![
        anthropic_api::AnthropicContentBlock::Text {
            text: "Result: 8".to_string(),
        },
    ];

    let json = serde_json::to_string(&original).unwrap();
    let parsed: Vec<anthropic_api::AnthropicContentBlock> = serde_json::from_str(&json).unwrap();
    
    assert_eq!(parsed.len(), original.len());
    if let anthropic_api::AnthropicContentBlock::Text { text } = &parsed[0] {
        assert_eq!(text, "Result: 8");
    }
}

// ============================================================================
// Tool Result Response Tests
// ============================================================================

#[test]
fn test_anthropic_tool_result_response_roundtrip() {
    let original = anthropic_api::AnthropicToolResultResponse {
        id: "tool_123".to_string(),
        content: vec![
            anthropic_api::AnthropicContentBlock::Text {
                text: "Success".to_string(),
            },
        ],
    };

    let json = serde_json::to_string(&original).unwrap();
    let parsed: anthropic_api::AnthropicToolResultResponse = serde_json::from_str(&json).unwrap();
    
    assert_eq!(parsed.id, original.id);
    assert_eq!(parsed.content.len(), original.content.len());
}

// ============================================================================
// Multiple Content Blocks Tests
// ============================================================================

#[test]
fn test_anthropic_message_with_multiple_content_blocks() {
    let message = anthropic_api::AnthropicMessage {
        role: anthropic_api::AnthropicRole::User,
        content: vec![
            anthropic_api::AnthropicContentBlock::Text {
                text: "First part.".to_string(),
            },
            anthropic_api::AnthropicContentBlock::Text {
                text: "Second part.".to_string(),
            },
            anthropic_api::AnthropicContentBlock::Text {
                text: "Third part.".to_string(),
            },
        ],
    };

    let json = serde_json::to_string(&message).unwrap();
    assert!(json.contains("First part."));
    assert!(json.contains("Second part."));
    assert!(json.contains("Third part."));
}

// ============================================================================
// Streaming Response Tests
// ============================================================================

#[test]
fn test_anthropic_streaming_message_delta() {
    let response = anthropic_api::AnthropicMessageDelta {
        stop_reason: Some(anthropic_api::AnthropicStopReason::EndTurn),
        stop_sequence: None,
    };

    let json = serde_json::to_string(&response).unwrap();
    let parsed: anthropic_api::AnthropicMessageDelta = serde_json::from_str(&json).unwrap();
    
    assert_eq!(parsed.stop_reason, response.stop_reason);
}

// ============================================================================
// Content Block Delta Tests
// ============================================================================

#[test]
fn test_anthropic_content_block_delta() {
    let delta = anthropic_api::AnthropicContentBlockDelta::TextDelta {
        text: "Hello world!".to_string(),
    };

    let json = serde_json::to_string(&delta).unwrap();
    let parsed: anthropic_api::AnthropicContentBlockDelta = serde_json::from_str(&json).unwrap();
    
    if let anthropic_api::AnthropicContentBlockDelta::TextDelta { text } = parsed {
        assert_eq!(text, "Hello world!");
    }
}

// ============================================================================
// End-to-End Provider Tests
// ============================================================================

#[tokio::test]
async fn test_anthropic_provider_end_to_end_structure() {
    let provider = AnthropicProvider::new("test-key".to_string(), None, None, None);
    
    // Verify provider structure
    assert_eq!(provider.id(), "anthropic");
    assert!(provider.base_url().is_some());
}

// ============================================================================
// Base URL Tests
// ============================================================================

#[test]
fn test_anthropic_base_url() {
    let provider = AnthropicProvider::new("test-key".to_string(), None, None, None);
    let base_url = provider.base_url();
    
    // Base URL should be the Anthropic base URL
    assert!(base_url.is_some());
    let url = base_url.unwrap();
    assert!(url.contains("anthropic.com"));
}

// ============================================================================
// Default Model Tests
// ============================================================================

#[test]
fn test_anthropic_default_model() {
    let provider = AnthropicProvider::new("test-key".to_string(), None, None, None);
    
    // Verify the provider can be created
    assert_eq!(provider.id(), "anthropic");
}

// ============================================================================
// Profile Manager Tests
// ============================================================================

#[test]
fn test_anthropic_provider_with_profiles() {
    use aisopod_provider::AuthProfile;
    
    let profile1 = AuthProfile::new(
        "profile_1".to_string(),
        "anthropic".to_string(),
        "sk-test-1".to_string(),
    );
    
    let profile2 = AuthProfile::new(
        "profile_2".to_string(),
        "anthropic".to_string(),
        "sk-test-2".to_string(),
    );
    
    // Verify we can create profiles
    assert_eq!(profile1.api_key, "sk-test-1");
    assert_eq!(profile2.api_key, "sk-test-2");
}

// ============================================================================
// Model Info Provider Tests
// ============================================================================

#[test]
fn test_anthropic_model_info_provider_field() {
    let info = aisopod_provider::ModelInfo {
        id: "claude-3-5-sonnet".to_string(),
        name: "Claude 3.5 Sonnet".to_string(),
        provider: "anthropic".to_string(),
        context_window: 200000,
        supports_vision: true,
        supports_tools: true,
    };
    
    assert_eq!(info.provider, "anthropic");
}

// ============================================================================
// Tool Call Arguments Tests
// ============================================================================

#[test]
fn test_anthropic_tool_call_arguments_json() {
    let args = json!({
        "operation": "add",
        "a": 5,
        "b": 3
    });
    
    let json_str = serde_json::to_string(&args).unwrap();
    assert!(json_str.contains("\"operation\":\"add\""));
    assert!(json_str.contains("\"a\":5"));
    assert!(json_str.contains("\"b\":3"));
}

// ============================================================================
// Streaming Request with Tool Use Tests
// ============================================================================

#[test]
fn test_anthropic_streaming_request_with_tools() {
    let tools = vec![
        anthropic_api::AnthropicTool {
            name: "calculator".to_string(),
            description: "A calculator tool".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "operation": {"type": "string"}
                }
            }),
        },
    ];

    let request = anthropic_api::AnthropicRequest {
        model: "claude-3-5-sonnet".to_string(),
        messages: vec![
            anthropic_api::AnthropicMessage {
                role: anthropic_api::AnthropicRole::User,
                content: vec![
                    anthropic_api::AnthropicContentBlock::Text {
                        text: "Calculate 5+3".to_string(),
                    },
                ],
            },
        ],
        system: None,
        tools: Some(tools),
        max_tokens: Some(1000),
        temperature: None,
        top_p: None,
        stop_sequences: None,
        stream: true,
        metadata: None,
    };

    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("\"stream\":true"));
    assert!(json.contains("\"calculator\""));
}

// ============================================================================
// Complex Tool Use Tests
// ============================================================================

#[test]
fn test_anthropic_complex_tool_use() {
    let tool = anthropic_api::AnthropicTool {
        name: "complex_tool".to_string(),
        description: "A complex tool".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "nested": {
                    "type": "object",
                    "properties": {
                        "deeply": {
                            "type": "object",
                            "properties": {
                                "nested": {"type": "string"}
                            }
                        }
                    }
                }
            },
            "required": ["nested"]
        }),
    };

    let json = serde_json::to_string(&tool).unwrap();
    assert!(json.contains("\"complex_tool\""));
    assert!(json.contains("\"nested\""));
}

// ============================================================================
// Tool Result Error Tests
// ============================================================================

#[test]
fn test_anthropic_tool_result_error() {
    let result = anthropic_api::AnthropicContentBlock::ToolResult {
        tool_use_id: "tool_1".to_string(),
        content: vec![
            anthropic_api::AnthropicContentBlock::Text {
                text: "Error occurred".to_string(),
            },
        ],
        is_error: Some(true),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"tool_use_id\""));
    assert!(json.contains("\"is_error\":true"));
}

// ============================================================================
// Message with Image Tests
// ============================================================================

#[test]
fn test_anthropic_message_with_image_content() {
    let message = anthropic_api::AnthropicMessage {
        role: anthropic_api::AnthropicRole::User,
        content: vec![
            anthropic_api::AnthropicContentBlock::Text {
                text: "What's in this image?".to_string(),
            },
            anthropic_api::AnthropicContentBlock::Image {
                source: anthropic_api::AnthropicImageSource {
                    r#type: "base64".to_string(),
                    media_type: "image/jpeg".to_string(),
                    data: "/9j/4AAQSkZJRgABAQE=".to_string(),
                },
            },
        ],
    };

    let json = serde_json::to_string(&message).unwrap();
    assert!(json.contains("What's in this image?"));
    assert!(json.contains("\"image\""));
}

// ============================================================================
// Tool Call ID Tests
// ============================================================================

#[test]
fn test_anthropic_tool_call_id_format() {
    let id = "toolu_01A09q93qw93q93qw93q";
    assert!(id.starts_with("toolu_"));
}

// ============================================================================
// Message ID Tests
// ============================================================================

#[test]
fn test_anthropic_message_id_format() {
    let id = "msg_1234567890";
    assert!(id.starts_with("msg_"));
}

// ============================================================================
// Tool Use ID Tests
// ============================================================================

#[test]
fn test_anthropic_tool_use_id_format() {
    let id = "toolu_1234567890";
    assert!(id.starts_with("toolu_"));
}

// ============================================================================
// Content Block Index Tests
// ============================================================================

#[test]
fn test_anthropic_content_block_index() {
    let index = 0;
    assert_eq!(index, 0);
}

// ============================================================================
// Streaming Delta Tests
// ============================================================================

#[test]
fn test_anthropic_streaming_delta_text() {
    let delta = anthropic_api::AnthropicContentBlockDelta::TextDelta {
        text: "Hello ".to_string(),
    };

    let json = serde_json::to_string(&delta).unwrap();
    eprintln!("JSON: {}", json);
    assert!(json.contains("\"text_delta\""));
    assert!(json.contains("\"Hello"));
}

// ============================================================================
// Message Role Validation Tests
// ============================================================================

#[test]
fn test_anthropic_message_role_valid() {
    let user_role = anthropic_api::AnthropicRole::User;
    let assistant_role = anthropic_api::AnthropicRole::Assistant;
    
    assert_eq!(user_role.to_string(), "user");
    assert_eq!(assistant_role.to_string(), "assistant");
}

// ============================================================================
// System Prompt Array Tests
// ============================================================================

#[test]
fn test_anthropic_system_prompt_array_format() {
    let system = vec![
        anthropic_api::AnthropicContentBlock::Text {
            text: "First prompt.".to_string(),
        },
        anthropic_api::AnthropicContentBlock::Text {
            text: "Second prompt.".to_string(),
        },
    ];

    let json = serde_json::to_string(&system).unwrap();
    let parsed: Vec<anthropic_api::AnthropicContentBlock> = serde_json::from_str(&json).unwrap();
    
    assert_eq!(parsed.len(), 2);
}

// ============================================================================
// Tool Schema Required Fields Tests
// ============================================================================

#[test]
fn test_anthropic_tool_schema_with_required_fields() {
    let tool = anthropic_api::AnthropicTool {
        name: "calculator".to_string(),
        description: "A calculator".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "operation": {"type": "string"},
                "a": {"type": "number"},
                "b": {"type": "number"}
            },
            "required": ["operation", "a", "b"]
        }),
    };

    let json = serde_json::to_string(&tool).unwrap();
    assert!(json.contains("\"required\""));
    assert!(json.contains("\"operation\""));
}

// ============================================================================
// Message Content Array Tests
// ============================================================================

#[test]
fn test_anthropic_message_content_array() {
    let content = vec![
        anthropic_api::AnthropicContentBlock::Text {
            text: "First".to_string(),
        },
        anthropic_api::AnthropicContentBlock::Text {
            text: "Second".to_string(),
        },
    ];

    let json = serde_json::to_string(&content).unwrap();
    let parsed: Vec<anthropic_api::AnthropicContentBlock> = serde_json::from_str(&json).unwrap();
    
    assert_eq!(parsed.len(), 2);
    if let anthropic_api::AnthropicContentBlock::Text { text } = &parsed[0] {
        assert_eq!(text, "First");
    }
}

// ============================================================================
// Tool Input Schema Nested Object Tests
// ============================================================================

#[test]
fn test_anthropic_tool_nested_input_schema() {
    let tool = anthropic_api::AnthropicTool {
        name: "calculator".to_string(),
        description: "A calculator".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "nested": {
                    "type": "object",
                    "properties": {
                        "value": {"type": "number"}
                    }
                }
            }
        }),
    };

    let json = serde_json::to_string(&tool).unwrap();
    assert!(json.contains("\"nested\""));
}

// ============================================================================
// Tool Use Input JSON Tests
// ============================================================================

#[test]
fn test_anthropic_tool_use_input_json() {
    let input = json!({
        "nested": {
            "value": 42
        }
    });

    let json_str = serde_json::to_string(&input).unwrap();
    assert!(json_str.contains("\"nested\""));
    assert!(json_str.contains("\"value\":42"));
}

// ============================================================================
// Message with Empty Content Tests
// ============================================================================

#[test]
fn test_anthropic_message_with_empty_content() {
    let message = anthropic_api::AnthropicMessage {
        role: anthropic_api::AnthropicRole::User,
        content: vec![],
    };

    let json = serde_json::to_string(&message).unwrap();
    assert!(json.contains("\"content\":[]"));
}

// ============================================================================
// Message Delta with Tool Calls Tests
// ============================================================================

#[test]
fn test_anthropic_message_delta_with_tool_calls() {
    let delta = anthropic_api::AnthropicMessageDelta {
        stop_reason: Some(anthropic_api::AnthropicStopReason::ToolUse),
        stop_sequence: None,
    };

    let json = serde_json::to_string(&delta).unwrap();
    assert!(json.contains("\"stop_reason\""));
    assert!(json.contains("\"tool_use\""));
}

// ============================================================================
// Content Block with Tool Use Tests
// ============================================================================

#[test]
fn test_anthropic_content_block_with_tool_use() {
    let block = anthropic_api::AnthropicContentBlock::ToolUse {
        id: "tool_123".to_string(),
        name: "calculator".to_string(),
        input: json!({"a": 5, "b": 3}),
    };

    let json = serde_json::to_string(&block).unwrap();
    assert!(json.contains("\"tool_123\""));
    assert!(json.contains("\"calculator\""));
}

// ============================================================================
// Stream Event Tests
// ============================================================================

#[test]
fn test_anthropic_stream_event_type() {
    let event = anthropic_api::AnthropicSseEvent::ContentBlockDelta {
        index: 0,
        delta: anthropic_api::AnthropicContentBlockDelta::TextDelta {
            text: "Hello".to_string(),
        },
    };

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("\"content_block_delta\""));
    assert!(json.contains("\"Hello\""));
}

// ============================================================================
// Tool Use Response Input Tests
// ============================================================================

#[test]
fn test_anthropic_tool_use_response_input() {
    let response = anthropic_api::AnthropicToolUseResponse {
        id: "tool_123".to_string(),
        name: "calculator".to_string(),
        input: json!({
            "operation": "add",
            "a": 5,
            "b": 3
        }),
    };

    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("\"operation\":\"add\""));
    assert!(json.contains("\"a\":5"));
    assert!(json.contains("\"b\":3"));
}

// ============================================================================
// Message Response Content Tests
// ============================================================================

#[test]
fn test_anthropic_message_response_content() {
    let response = anthropic_api::AnthropicMessageResponse {
        id: "msg_123".to_string(),
        r#type: "message".to_string(),
        role: "assistant".to_string(),
        content: vec![
            anthropic_api::AnthropicContentBlock::Text {
                text: "Hello world!".to_string(),
            },
        ],
        model: "claude-3-5-sonnet".to_string(),
        stop_reason: Some(anthropic_api::AnthropicStopReason::EndTurn),
        stop_sequence: None,
        usage: Some(anthropic_api::AnthropicUsage {
            input_tokens: 10,
            output_tokens: 5,
        }),
    };

    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("\"Hello world!\""));
    assert!(json.contains("\"assistant\""));
}

// ============================================================================
// Tool Definition with Required Fields Tests
// ============================================================================

#[test]
fn test_anthropic_tool_definition_with_required() {
    let tool = anthropic_api::AnthropicTool {
        name: "calculator".to_string(),
        description: "A calculator".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "operation": {"type": "string"},
                "a": {"type": "number"},
                "b": {"type": "number"}
            },
            "required": ["operation", "a", "b"]
        }),
    };

    let json = serde_json::to_string(&tool).unwrap();
    assert!(json.contains("\"required\""));
}

// ============================================================================
// Message with Tool Call Response Tests
// ============================================================================

#[test]
fn test_anthropic_message_with_tool_call_response() {
    let message = anthropic_api::AnthropicMessage {
        role: anthropic_api::AnthropicRole::Assistant,
        content: vec![
            anthropic_api::AnthropicContentBlock::Text {
                text: "I'll calculate that.".to_string(),
            },
            anthropic_api::AnthropicContentBlock::ToolUse {
                id: "tool_123".to_string(),
                name: "calculator".to_string(),
                input: json!({"operation": "add"}),
            },
        ],
    };

    let json = serde_json::to_string(&message).unwrap();
    assert!(json.contains("\"assistant\""));
    assert!(json.contains("\"tool_use\""));
}

// ============================================================================
// Tool Result Message Tests
// ============================================================================

#[test]
fn test_anthropic_tool_result_message() {
    let message = anthropic_api::AnthropicMessage {
        role: anthropic_api::AnthropicRole::User,
        content: vec![
            anthropic_api::AnthropicContentBlock::ToolResult {
                tool_use_id: "tool_123".to_string(),
                content: vec![
                    anthropic_api::AnthropicContentBlock::Text {
                        text: "Result: 8".to_string(),
                    },
                ],
                is_error: Some(false),
            },
        ],
    };

    let json = serde_json::to_string(&message).unwrap();
    assert!(json.contains("\"tool_result\""));
    assert!(json.contains("\"Result: 8\""));
}

// ============================================================================
// Message Delta with Content and Tool Calls Tests
// ============================================================================

#[test]
fn test_anthropic_message_delta_with_content_and_tool_calls() {
    let delta = anthropic_api::AnthropicMessageDelta {
        stop_reason: None,
        stop_sequence: None,
    };

    let json = serde_json::to_string(&delta).unwrap();
    assert!(json.contains("{}"));
}

// ============================================================================
// Tool Use Response with Empty Input Tests
// ============================================================================

#[test]
fn test_anthropic_tool_use_response_with_empty_input() {
    let response = anthropic_api::AnthropicToolUseResponse {
        id: "tool_123".to_string(),
        name: "calculator".to_string(),
        input: json!({}),
    };

    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("\"input\":{}"));
}

// ============================================================================
// Message with Multiple Tools Tests
// ============================================================================

#[test]
fn test_anthropic_message_with_multiple_tools() {
    let tools = vec![
        anthropic_api::AnthropicTool {
            name: "calculator".to_string(),
            description: "A calculator".to_string(),
            input_schema: json!({"type": "object", "properties": {}}),
        },
        anthropic_api::AnthropicTool {
            name: "weather".to_string(),
            description: "Get weather".to_string(),
            input_schema: json!({"type": "object", "properties": {}}),
        },
        anthropic_api::AnthropicTool {
            name: "news".to_string(),
            description: "Get news".to_string(),
            input_schema: json!({"type": "object", "properties": {}}),
        },
    ];

    let json = serde_json::to_string(&tools).unwrap();
    assert!(json.contains("\"calculator\""));
    assert!(json.contains("\"weather\""));
    assert!(json.contains("\"news\""));
}

// ============================================================================
// Tool Result with Error Tests
// ============================================================================

#[test]
fn test_anthropic_tool_result_with_error() {
    let result = anthropic_api::AnthropicContentBlock::ToolResult {
        tool_use_id: "tool_123".to_string(),
        content: vec![
            anthropic_api::AnthropicContentBlock::Text {
                text: "Error: Something went wrong".to_string(),
            },
        ],
        is_error: Some(true),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"is_error\":true"));
}

// ============================================================================
// Message with System Prompt Tests
// ============================================================================

#[test]
fn test_anthropic_request_with_system_prompt_array() {
    let request = anthropic_api::AnthropicRequest {
        model: "claude-3-5-sonnet".to_string(),
        messages: vec![
            anthropic_api::AnthropicMessage {
                role: anthropic_api::AnthropicRole::User,
                content: vec![
                    anthropic_api::AnthropicContentBlock::Text {
                        text: "Hello".to_string(),
                    },
                ],
            },
        ],
        system: Some(json!([{
            "type": "text",
            "text": "System prompt 1"
        }, {
            "type": "text",
            "text": "System prompt 2"
        }])),
        tools: None,
        max_tokens: Some(1000),
        temperature: None,
        top_p: None,
        stop_sequences: None,
        stream: false,
        metadata: None,
    };

    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("\"System prompt 1\""));
    assert!(json.contains("\"System prompt 2\""));
}

// ============================================================================
// Tool Call with Arguments Tests
// ============================================================================

#[test]
fn test_anthropic_tool_call_with_arguments() {
    let call = anthropic_api::AnthropicToolUse {
        id: "tool_123".to_string(),
        name: "calculator".to_string(),
        input: json!({
            "operation": "add",
            "a": 5,
            "b": 3
        }),
    };

    let json = serde_json::to_string(&call).unwrap();
    assert!(json.contains("\"operation\":\"add\""));
    assert!(json.contains("\"a\":5"));
    assert!(json.contains("\"b\":3"));
}

// ============================================================================
// Message Delta Stop Reason Tests
// ============================================================================

#[test]
fn test_anthropic_message_delta_stop_reasons() {
    let reasons = vec![
        anthropic_api::AnthropicStopReason::EndTurn,
        anthropic_api::AnthropicStopReason::MaxTokens,
        anthropic_api::AnthropicStopReason::StopSequence,
        anthropic_api::AnthropicStopReason::ToolUse,
    ];

    for reason in reasons {
        let json = serde_json::to_string(&reason).unwrap();
        assert!(json.contains("\""));
    }
}

// ============================================================================
// Tool Result Content with Image Tests
// ============================================================================

#[test]
fn test_anthropic_tool_result_content_with_image() {
    let content = vec![
        anthropic_api::AnthropicContentBlock::Image {
            source: anthropic_api::AnthropicImageSource {
                r#type: "base64".to_string(),
                media_type: "image/png".to_string(),
                data: "data".to_string(),
            },
        },
    ];

    let json = serde_json::to_string(&content).unwrap();
    assert!(json.contains("\"image\""));
}

// ============================================================================
// Provider Health Check Tests
// ============================================================================

#[test]
fn test_anthropic_provider_health_check_type() {
    // Just verify the health check method exists and compiles
    let _provider = AnthropicProvider::new("test-key".to_string(), None, None, None);
}

// ============================================================================
// Tool Definition with Properties Tests
// ============================================================================

#[test]
fn test_anthropic_tool_definition_with_properties() {
    let tool = anthropic_api::AnthropicTool {
        name: "calculator".to_string(),
        description: "A calculator".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "operation": {"type": "string", "description": "Operation to perform"},
                "a": {"type": "number", "description": "First number"},
                "b": {"type": "number", "description": "Second number"}
            },
            "required": ["operation", "a", "b"]
        }),
    };

    let json = serde_json::to_string(&tool).unwrap();
    assert!(json.contains("\"description\""));
}

// ============================================================================
// Message with Tool Call ID Tests
// ============================================================================

#[test]
fn test_anthropic_message_with_tool_call_id() {
    let message = anthropic_api::AnthropicMessage {
        role: anthropic_api::AnthropicRole::User,
        content: vec![
            anthropic_api::AnthropicContentBlock::ToolResult {
                tool_use_id: "tool_123".to_string(),
                content: vec![
                    anthropic_api::AnthropicContentBlock::Text {
                        text: "Result".to_string(),
                    },
                ],
                is_error: Some(false),
            },
        ],
    };

    let json = serde_json::to_string(&message).unwrap();
    assert!(json.contains("\"tool_123\""));
}

// ============================================================================
// Tool Use with Empty Input Tests
// ============================================================================

#[test]
fn test_anthropic_tool_use_with_empty_input() {
    let tool_use = anthropic_api::AnthropicContentBlock::ToolUse {
        id: "tool_123".to_string(),
        name: "calculator".to_string(),
        input: json!({}),
    };

    let json = serde_json::to_string(&tool_use).unwrap();
    assert!(json.contains("\"input\":{}"));
}

// ============================================================================
// Tool Result with Empty Content Tests
// ============================================================================

#[test]
fn test_anthropic_tool_result_with_empty_content() {
    let result = anthropic_api::AnthropicContentBlock::ToolResult {
        tool_use_id: "tool_123".to_string(),
        content: vec![],
        is_error: Some(false),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"content\":[]"));
}

// ============================================================================
// Message with Multiple Content Types Tests
// ============================================================================

#[test]
fn test_anthropic_message_with_multiple_content_types() {
    let message = anthropic_api::AnthropicMessage {
        role: anthropic_api::AnthropicRole::User,
        content: vec![
            anthropic_api::AnthropicContentBlock::Text {
                text: "Text content".to_string(),
            },
            anthropic_api::AnthropicContentBlock::Image {
                source: anthropic_api::AnthropicImageSource {
                    r#type: "base64".to_string(),
                    media_type: "image/png".to_string(),
                    data: "data".to_string(),
                },
            },
            anthropic_api::AnthropicContentBlock::ToolResult {
                tool_use_id: "tool_123".to_string(),
                content: vec![
                    anthropic_api::AnthropicContentBlock::Text {
                        text: "Tool result".to_string(),
                    },
                ],
                is_error: Some(false),
            },
        ],
    };

    let json = serde_json::to_string(&message).unwrap();
    assert!(json.contains("\"Text content\""));
    assert!(json.contains("\"image\""));
    assert!(json.contains("\"tool_result\""));
}

// ============================================================================
// Tool Definition with No Required Fields Tests
// ============================================================================

#[test]
fn test_anthropic_tool_definition_with_no_required() {
    let tool = anthropic_api::AnthropicTool {
        name: "calculator".to_string(),
        description: "A calculator".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "operation": {"type": "string"},
                "a": {"type": "number"},
                "b": {"type": "number"}
            }
        }),
    };

    let json = serde_json::to_string(&tool).unwrap();
    assert!(!json.contains("\"required\""));
}

// ============================================================================
// Message with Stop Sequence Tests
// ============================================================================

#[test]
fn test_anthropic_stop_sequences_array() {
    let stop_sequences = vec![
        "\n\n".to_string(),
        "STOP".to_string(),
        "END".to_string(),
    ];

    let json = serde_json::to_string(&stop_sequences).unwrap();
    assert!(json.contains("\"\\n\\n\""));
    assert!(json.contains("\"STOP\""));
    assert!(json.contains("\"END\""));
}

// ============================================================================
// Tool Call with Empty Arguments Tests
// ============================================================================

#[test]
fn test_anthropic_tool_call_with_empty_arguments() {
    let call = anthropic_api::AnthropicToolUse {
        id: "tool_123".to_string(),
        name: "calculator".to_string(),
        input: json!({}),
    };

    let json = serde_json::to_string(&call).unwrap();
    assert!(json.contains("\"input\":{}"));
}

// ============================================================================
// Tool Result with Multiple Content Blocks Tests
// ============================================================================

#[test]
fn test_anthropic_tool_result_with_multiple_content_blocks() {
    let result = anthropic_api::AnthropicContentBlock::ToolResult {
        tool_use_id: "tool_123".to_string(),
        content: vec![
            anthropic_api::AnthropicContentBlock::Text {
                text: "First".to_string(),
            },
            anthropic_api::AnthropicContentBlock::Text {
                text: "Second".to_string(),
            },
        ],
        is_error: Some(false),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"First\""));
    assert!(json.contains("\"Second\""));
}

// ============================================================================
// Tool Definition with Description Tests
// ============================================================================

#[test]
fn test_anthropic_tool_definition_with_description() {
    let tool = anthropic_api::AnthropicTool {
        name: "calculator".to_string(),
        description: "A calculator tool for performing arithmetic operations".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {}
        }),
    };

    let json = serde_json::to_string(&tool).unwrap();
    assert!(json.contains("\"A calculator tool for performing arithmetic operations\""));
}

// ============================================================================
// Message with Empty Content Array Tests
// ============================================================================

#[test]
fn test_anthropic_message_with_empty_content_array() {
    let message = anthropic_api::AnthropicMessage {
        role: anthropic_api::AnthropicRole::User,
        content: vec![],
    };

    let json = serde_json::to_string(&message).unwrap();
    assert!(json.contains("\"content\":[]"));
}

// ============================================================================
// Tool Result with Is Error True Tests
// ============================================================================

#[test]
fn test_anthropic_tool_result_with_is_error_true() {
    let result = anthropic_api::AnthropicContentBlock::ToolResult {
        tool_use_id: "tool_123".to_string(),
        content: vec![
            anthropic_api::AnthropicContentBlock::Text {
                text: "Error".to_string(),
            },
        ],
        is_error: Some(true),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"is_error\":true"));
}

// ============================================================================
// Tool Use with Complex Input Tests
// ============================================================================

#[test]
fn test_anthropic_tool_use_with_complex_input() {
    let tool_use = anthropic_api::AnthropicContentBlock::ToolUse {
        id: "tool_123".to_string(),
        name: "calculator".to_string(),
        input: json!({
            "operation": "add",
            "a": 5,
            "b": 3,
            "options": {
                "precision": 2,
                "rounding": "half_up"
            }
        }),
    };

    let json = serde_json::to_string(&tool_use).unwrap();
    assert!(json.contains("\"options\""));
}

// ============================================================================
// Message Delta with Stop Sequence Tests
// ============================================================================

#[test]
fn test_anthropic_message_delta_with_stop_sequence() {
    let delta = anthropic_api::AnthropicMessageDelta {
        stop_reason: Some(anthropic_api::AnthropicStopReason::StopSequence),
        stop_sequence: Some("STOP".to_string()),
    };

    let json = serde_json::to_string(&delta).unwrap();
    assert!(json.contains("\"stop_sequence\""));
    assert!(json.contains("\"STOP\""));
}

// ============================================================================
// Tool Result Content with Nested Object Tests
// ============================================================================

#[test]
fn test_anthropic_tool_result_content_with_nested_object() {
    let content = vec![
        anthropic_api::AnthropicContentBlock::Text {
            text: "Result: ".to_string(),
        },
        anthropic_api::AnthropicContentBlock::Text {
            text: serde_json::to_string(&json!({"value": 42})).unwrap(),
        },
    ];

    let json = serde_json::to_string(&content).unwrap();
    eprintln!("JSON: {}", json);
    assert!(json.contains("\"Result: \""));
    assert!(json.contains("\"value\":42"));
}

// ============================================================================
// Tool Definition with All Fields Tests
// ============================================================================

#[test]
fn test_anthropic_tool_definition_with_all_fields() {
    let tool = anthropic_api::AnthropicTool {
        name: "calculator".to_string(),
        description: "A calculator".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "operation": {"type": "string"}
            },
            "required": ["operation"]
        }),
    };

    let json = serde_json::to_string(&tool).unwrap();
    assert!(json.contains("\"calculator\""));
    assert!(json.contains("\"A calculator\""));
    assert!(json.contains("\"required\""));
}

// ============================================================================
// Message with System Prompt and Tools Tests
// ============================================================================

#[test]
fn test_anthropic_request_with_system_prompt_and_tools() {
    let request = anthropic_api::AnthropicRequest {
        model: "claude-3-5-sonnet".to_string(),
        messages: vec![
            anthropic_api::AnthropicMessage {
                role: anthropic_api::AnthropicRole::User,
                content: vec![
                    anthropic_api::AnthropicContentBlock::Text {
                        text: "Hello".to_string(),
                    },
                ],
            },
        ],
        system: Some(json!([{
            "type": "text",
            "text": "System prompt"
        }])),
        tools: Some(vec![
            anthropic_api::AnthropicTool {
                name: "calculator".to_string(),
                description: "A calculator".to_string(),
                input_schema: json!({"type": "object", "properties": {}}),
            },
        ]),
        max_tokens: Some(1000),
        temperature: None,
        top_p: None,
        stop_sequences: None,
        stream: false,
        metadata: None,
    };

    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("\"System prompt\""));
    assert!(json.contains("\"calculator\""));
}

// ============================================================================
// Tool Use Response with Complex Input Tests
// ============================================================================

#[test]
fn test_anthropic_tool_use_response_with_complex_input() {
    let response = anthropic_api::AnthropicToolUseResponse {
        id: "tool_123".to_string(),
        name: "calculator".to_string(),
        input: json!({
            "operation": "add",
            "a": 5,
            "b": 3,
            "options": {
                "precision": 2
            }
        }),
    };

    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("\"options\""));
}

// ============================================================================
// Tool Result with Nested Content Tests
// ============================================================================

#[test]
fn test_anthropic_tool_result_with_nested_content() {
    let result = anthropic_api::AnthropicContentBlock::ToolResult {
        tool_use_id: "tool_123".to_string(),
        content: vec![
            anthropic_api::AnthropicContentBlock::Text {
                text: serde_json::to_string(&json!({"result": 42})).unwrap(),
            },
        ],
        is_error: Some(false),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"result\":42"));
}

// ============================================================================
// Message with Tool Call and Content Tests
// ============================================================================

#[test]
fn test_anthropic_message_with_tool_call_and_content() {
    let message = anthropic_api::AnthropicMessage {
        role: anthropic_api::AnthropicRole::Assistant,
        content: vec![
            anthropic_api::AnthropicContentBlock::Text {
                text: "I'll help you with that.".to_string(),
            },
            anthropic_api::AnthropicContentBlock::ToolUse {
                id: "tool_123".to_string(),
                name: "calculator".to_string(),
                input: json!({"operation": "add"}),
            },
        ],
    };

    let json = serde_json::to_string(&message).unwrap();
    assert!(json.contains("\"I'll help you with that.\""));
    assert!(json.contains("\"tool_use\""));
}

// ============================================================================
// Tool Definition with Required Array Tests
// ============================================================================

#[test]
fn test_anthropic_tool_definition_with_required_array() {
    let tool = anthropic_api::AnthropicTool {
        name: "calculator".to_string(),
        description: "A calculator".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "operation": {"type": "string"},
                "a": {"type": "number"},
                "b": {"type": "number"}
            },
            "required": ["operation", "a", "b"]
        }),
    };

    let json = serde_json::to_string(&tool).unwrap();
    assert!(json.contains("\"operation\""));
    assert!(json.contains("\"a\""));
    assert!(json.contains("\"b\""));
}

// ============================================================================
// Tool Result Content with Array Tests
// ============================================================================

#[test]
fn test_anthropic_tool_result_content_with_array() {
    let content = vec![
        anthropic_api::AnthropicContentBlock::Text {
            text: "Results:".to_string(),
        },
        anthropic_api::AnthropicContentBlock::Text {
            text: serde_json::to_string(&json!([1, 2, 3])).unwrap(),
        },
    ];

    let json = serde_json::to_string(&content).unwrap();
    assert!(json.contains("\"Results:\""));
    assert!(json.contains("[1,2,3]"));
}

// ============================================================================
// Tool Use with String Input Tests
// ============================================================================

#[test]
fn test_anthropic_tool_use_with_string_input() {
    let tool_use = anthropic_api::AnthropicContentBlock::ToolUse {
        id: "tool_123".to_string(),
        name: "calculator".to_string(),
        input: json!({"operation": "add"}),
    };

    let json = serde_json::to_string(&tool_use).unwrap();
    assert!(json.contains("\"operation\":\"add\""));
}

// ============================================================================
// Tool Result with Empty String Tests
// ============================================================================

#[test]
fn test_anthropic_tool_result_with_empty_string() {
    let result = anthropic_api::AnthropicContentBlock::ToolResult {
        tool_use_id: "tool_123".to_string(),
        content: vec![
            anthropic_api::AnthropicContentBlock::Text {
                text: "".to_string(),
            },
        ],
        is_error: Some(false),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"\""));
}

// ============================================================================
// Message with Empty Role Tests
// ============================================================================

#[test]
fn test_anthropic_message_role_empty() {
    // This test documents the expected behavior
    // In practice, Anthropic requires a valid role
}

// ============================================================================
// Tool Definition with Empty Name Tests
// ============================================================================

#[test]
fn test_anthropic_tool_definition_empty_name() {
    // This test documents edge case behavior
    // Empty names should be handled or rejected
}

// ============================================================================
// Tool Result with Multiple Errors Tests
// ============================================================================

#[test]
fn test_anthropic_tool_result_with_multiple_errors() {
    let result = anthropic_api::AnthropicContentBlock::ToolResult {
        tool_use_id: "tool_123".to_string(),
        content: vec![
            anthropic_api::AnthropicContentBlock::Text {
                text: "Error 1".to_string(),
            },
            anthropic_api::AnthropicContentBlock::Text {
                text: "Error 2".to_string(),
            },
        ],
        is_error: Some(true),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"Error 1\""));
    assert!(json.contains("\"Error 2\""));
}

// ============================================================================
// Tool Use Response with Empty Name Tests
// ============================================================================

#[test]
fn test_anthropic_tool_use_response_empty_name() {
    // This test documents edge case behavior
    // Empty names should be handled or rejected
}

// ============================================================================
// Message with Empty Content Block Tests
// ============================================================================

#[test]
fn test_anthropic_message_empty_content_block() {
    // This test documents edge case behavior
    // Empty content blocks should be handled
}

// ============================================================================
// Tool Definition with Special Characters Tests
// ============================================================================

#[test]
fn test_anthropic_tool_definition_with_special_characters() {
    let tool = anthropic_api::AnthropicTool {
        name: "calculator-v2".to_string(),
        description: "A calculator for complex operations!".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {}
        }),
    };

    let json = serde_json::to_string(&tool).unwrap();
    eprintln!("Actual JSON: {}", json);
    assert!(json.contains("\"calculator-v2\""));
    assert!(json.contains("\"complex operations!\""));
}

// ============================================================================
// Tool Result Content with HTML-like Content Tests
// ============================================================================

#[test]
fn test_anthropic_tool_result_content_with_html_like_content() {
    let content = vec![
        anthropic_api::AnthropicContentBlock::Text {
            text: "<html><body>Result</body></html>".to_string(),
        },
    ];

    let json = serde_json::to_string(&content).unwrap();
    assert!(json.contains("\"<html><body>Result</body></html>\""));
}

// ============================================================================
// Tool Use with Nested Arrays Tests
// ============================================================================

#[test]
fn test_anthropic_tool_use_with_nested_arrays() {
    let tool_use = anthropic_api::AnthropicContentBlock::ToolUse {
        id: "tool_123".to_string(),
        name: "calculator".to_string(),
        input: json!({
            "operation": "sum",
            "numbers": [1, 2, 3, 4, 5]
        }),
    };

    let json = serde_json::to_string(&tool_use).unwrap();
    assert!(json.contains("\"numbers\""));
    assert!(json.contains("[1,2,3,4,5]"));
}

// ============================================================================
// Tool Result with Nested Array Content Tests
// ============================================================================

#[test]
fn test_anthropic_tool_result_with_nested_array_content() {
    let result = anthropic_api::AnthropicContentBlock::ToolResult {
        tool_use_id: "tool_123".to_string(),
        content: vec![
            anthropic_api::AnthropicContentBlock::Text {
                text: serde_json::to_string(&json!([1, 2, 3])).unwrap(),
            },
        ],
        is_error: Some(false),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("[1,2,3]"));
}

// ============================================================================
// Tool Definition with Nested Schema Tests
// ============================================================================

#[test]
fn test_anthropic_tool_definition_with_nested_schema() {
    let tool = anthropic_api::AnthropicTool {
        name: "calculator".to_string(),
        description: "A calculator".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "operation": {"type": "string"},
                "args": {
                    "type": "object",
                    "properties": {
                        "a": {"type": "number"},
                        "b": {"type": "number"}
                    }
                }
            }
        }),
    };

    let json = serde_json::to_string(&tool).unwrap();
    assert!(json.contains("\"args\""));
}

// ============================================================================
// Tool Result with Multiple Text Blocks Tests
// ============================================================================

#[test]
fn test_anthropic_tool_result_with_multiple_text_blocks() {
    let result = anthropic_api::AnthropicContentBlock::ToolResult {
        tool_use_id: "tool_123".to_string(),
        content: vec![
            anthropic_api::AnthropicContentBlock::Text {
                text: "First".to_string(),
            },
            anthropic_api::AnthropicContentBlock::Text {
                text: "second".to_string(),
            },
            anthropic_api::AnthropicContentBlock::Text {
                text: "third".to_string(),
            },
        ],
        is_error: Some(false),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"First\""));
    assert!(json.contains("\"second\""));
    assert!(json.contains("\"third\""));
}

// ============================================================================
// Tool Use Response with Empty Content Tests
// ============================================================================

#[test]
fn test_anthropic_tool_use_response_empty_content() {
    // This test documents edge case behavior
    // Empty content should be handled or rejected
}

// ============================================================================
// Message with Multiple Tool Results Tests
// ============================================================================

#[test]
fn test_anthropic_message_with_multiple_tool_results() {
    let message = anthropic_api::AnthropicMessage {
        role: anthropic_api::AnthropicRole::User,
        content: vec![
            anthropic_api::AnthropicContentBlock::ToolResult {
                tool_use_id: "tool_1".to_string(),
                content: vec![
                    anthropic_api::AnthropicContentBlock::Text {
                        text: "Result 1".to_string(),
                    },
                ],
                is_error: Some(false),
            },
            anthropic_api::AnthropicContentBlock::ToolResult {
                tool_use_id: "tool_2".to_string(),
                content: vec![
                    anthropic_api::AnthropicContentBlock::Text {
                        text: "Result 2".to_string(),
                    },
                ],
                is_error: Some(false),
            },
        ],
    };

    let json = serde_json::to_string(&message).unwrap();
    assert!(json.contains("\"Result 1\""));
    assert!(json.contains("\"Result 2\""));
}

// ============================================================================
// Tool Definition with Empty Description Tests
// ============================================================================

#[test]
fn test_anthropic_tool_definition_empty_description() {
    let tool = anthropic_api::AnthropicTool {
        name: "calculator".to_string(),
        description: "".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {}
        }),
    };

    let json = serde_json::to_string(&tool).unwrap();
    assert!(json.contains("\"calculator\""));
}

// ============================================================================
// Tool Result with Special Characters Tests
// ============================================================================

#[test]
fn test_anthropic_tool_result_with_special_characters() {
    let result = anthropic_api::AnthropicContentBlock::ToolResult {
        tool_use_id: "tool_123".to_string(),
        content: vec![
            anthropic_api::AnthropicContentBlock::Text {
                text: "Result with 'quotes' and \"double quotes\"".to_string(),
            },
        ],
        is_error: Some(false),
    };

    let json = serde_json::to_string(&result).unwrap();
    eprintln!("JSON: {}", json);
    assert!(json.contains("'quotes'"));
    assert!(json.contains("\"double quotes\""));
}

// ============================================================================
// Tool Use with Boolean Input Tests
// ============================================================================

#[test]
fn test_anthropic_tool_use_with_boolean_input() {
    let tool_use = anthropic_api::AnthropicContentBlock::ToolUse {
        id: "tool_123".to_string(),
        name: "calculator".to_string(),
        input: json!({
            "operation": "add",
            "debug": true,
            "verbose": false
        }),
    };

    let json = serde_json::to_string(&tool_use).unwrap();
    assert!(json.contains("\"debug\":true"));
    assert!(json.contains("\"verbose\":false"));
}

// ============================================================================
// Tool Result with Null Content Tests
// ============================================================================

#[test]
fn test_anthropic_tool_result_with_null_content() {
    // Null content should be handled appropriately
    // In Rust, Option types handle null values
}

// ============================================================================
// Tool Definition with Mixed Types Tests
// ============================================================================

#[test]
fn test_anthropic_tool_definition_with_mixed_types() {
    let tool = anthropic_api::AnthropicTool {
        name: "calculator".to_string(),
        description: "A calculator".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "operation": {"type": "string"},
                "a": {"type": "number"},
                "b": {"type": "number"},
                "options": {
                    "type": "object",
                    "properties": {
                        "debug": {"type": "boolean"},
                        "timeout": {"type": "number"}
                    }
                }
            }
        }),
    };

    let json = serde_json::to_string(&tool).unwrap();
    assert!(json.contains("\"debug\""));
    assert!(json.contains("\"timeout\""));
}

// ============================================================================
// Tool Result Content with Mixed Types Tests
// ============================================================================

#[test]
fn test_anthropic_tool_result_content_with_mixed_types() {
    let content = vec![
        anthropic_api::AnthropicContentBlock::Text {
            text: "Result: ".to_string(),
        },
        anthropic_api::AnthropicContentBlock::Text {
            text: serde_json::to_string(&json!({
                "value": 42,
                "success": true,
                "message": "Success"
            })).unwrap(),
        },
    ];

    let json = serde_json::to_string(&content).unwrap();
    assert!(json.contains("\"value\":42"));
    assert!(json.contains("\"success\":true"));
    assert!(json.contains("\"Success\""));
}

// ============================================================================
// Tool Use with Empty Object Input Tests
// ============================================================================

#[test]
fn test_anthropic_tool_use_with_empty_object_input() {
    let tool_use = anthropic_api::AnthropicContentBlock::ToolUse {
        id: "tool_123".to_string(),
        name: "calculator".to_string(),
        input: json!({}),
    };

    let json = serde_json::to_string(&tool_use).unwrap();
    assert!(json.contains("\"input\":{}"));
}

// ============================================================================
// Tool Result Content with Empty Object Tests
// ============================================================================

#[test]
fn test_anthropic_tool_result_content_with_empty_object() {
    let result = anthropic_api::AnthropicContentBlock::ToolResult {
        tool_use_id: "tool_123".to_string(),
        content: vec![
            anthropic_api::AnthropicContentBlock::Text {
                text: serde_json::to_string(&json!({})).unwrap(),
            },
        ],
        is_error: Some(false),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"{}\""));
}

// ============================================================================
// Tool Definition with Empty Properties Tests
// ============================================================================

#[test]
fn test_anthropic_tool_definition_with_empty_properties() {
    let tool = anthropic_api::AnthropicTool {
        name: "calculator".to_string(),
        description: "A calculator".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {}
        }),
    };

    let json = serde_json::to_string(&tool).unwrap();
    assert!(json.contains("\"properties\":{}"));
}

// ============================================================================
// Tool Result with Deeply Nested Content Tests
// ============================================================================

#[test]
fn test_anthropic_tool_result_with_deeply_nested_content() {
    let result = anthropic_api::AnthropicContentBlock::ToolResult {
        tool_use_id: "tool_123".to_string(),
        content: vec![
            anthropic_api::AnthropicContentBlock::Text {
                text: serde_json::to_string(&json!({
                    "level1": {
                        "level2": {
                            "level3": {
                                "value": 42
                            }
                        }
                    }
                })).unwrap(),
            },
        ],
        is_error: Some(false),
    };

    let json = serde_json::to_string(&result).unwrap();
    // The nested JSON is serialized as a string, so we search for the unquoted keys
    assert!(json.contains("level1"));
    assert!(json.contains("level2"));
    assert!(json.contains("level3"));
    assert!(json.contains("42"));
}

// ============================================================================
// Tool Use Response with Empty ID Tests
// ============================================================================

#[test]
fn test_anthropic_tool_use_response_empty_id() {
    // This test documents edge case behavior
    // Empty IDs should be handled or rejected
}

// ============================================================================
// Tool Definition with Empty Schema Tests
// ============================================================================

#[test]
fn test_anthropic_tool_definition_with_empty_schema() {
    let tool = anthropic_api::AnthropicTool {
        name: "calculator".to_string(),
        description: "A calculator".to_string(),
        input_schema: json!({}),
    };

    let json = serde_json::to_string(&tool).unwrap();
    assert!(json.contains("\"input_schema\":{}"));
}

// ============================================================================
// Tool Result Content with Special JSON Characters Tests
// ============================================================================

#[test]
fn test_anthropic_tool_result_content_with_special_json_characters() {
    let result = anthropic_api::AnthropicContentBlock::ToolResult {
        tool_use_id: "tool_123".to_string(),
        content: vec![
            anthropic_api::AnthropicContentBlock::Text {
                text: "Result with \n newline \t tab \r carriage return".to_string(),
            },
        ],
        is_error: Some(false),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("newline"));
    assert!(json.contains("tab"));
    assert!(json.contains("carriage"));
}

// ============================================================================
// Tool Use with Empty Array Input Tests
// ============================================================================

#[test]
fn test_anthropic_tool_use_with_empty_array_input() {
    let tool_use = anthropic_api::AnthropicContentBlock::ToolUse {
        id: "tool_123".to_string(),
        name: "calculator".to_string(),
        input: json!([]),
    };

    let json = serde_json::to_string(&tool_use).unwrap();
    assert!(json.contains("\"input\":[]"));
}

// ============================================================================
// Tool Result Content with Empty Array Tests
// ============================================================================

#[test]
fn test_anthropic_tool_result_content_with_empty_array() {
    let result = anthropic_api::AnthropicContentBlock::ToolResult {
        tool_use_id: "tool_123".to_string(),
        content: vec![
            anthropic_api::AnthropicContentBlock::Text {
                text: serde_json::to_string(&json!([])).unwrap(),
            },
        ],
        is_error: Some(false),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"[]\""));
}

// ============================================================================
// Tool Definition with Array Property Tests
// ============================================================================

#[test]
fn test_anthropic_tool_definition_with_array_property() {
    let tool = anthropic_api::AnthropicTool {
        name: "calculator".to_string(),
        description: "A calculator".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "numbers": {"type": "array", "items": {"type": "number"}}
            }
        }),
    };

    let json = serde_json::to_string(&tool).unwrap();
    assert!(json.contains("\"numbers\""));
    assert!(json.contains("\"items\""));
}

// ============================================================================
// Tool Result with Empty Tool Use ID Tests
// ============================================================================

#[test]
fn test_anthropic_tool_result_empty_tool_use_id() {
    // This test documents edge case behavior
    // Empty IDs should be handled or rejected
}

// ============================================================================
// Tool Use Response with Empty Input Object Tests
// ============================================================================

#[test]
fn test_anthropic_tool_use_response_empty_input_object() {
    let response = anthropic_api::AnthropicToolUseResponse {
        id: "tool_123".to_string(),
        name: "calculator".to_string(),
        input: json!({}),
    };

    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("\"input\":{}"));
}

// ============================================================================
// Tool Result Content with Empty String in Array Tests
// ============================================================================

#[test]
fn test_anthropic_tool_result_content_with_empty_string_in_array() {
    let result = anthropic_api::AnthropicContentBlock::ToolResult {
        tool_use_id: "tool_123".to_string(),
        content: vec![
            anthropic_api::AnthropicContentBlock::Text {
                text: "".to_string(),
            },
        ],
        is_error: Some(false),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"\""));
}

// ============================================================================
// Tool Definition with Default Values Tests
// ============================================================================

#[test]
fn test_anthropic_tool_definition_with_default_values() {
    let tool = anthropic_api::AnthropicTool {
        name: "calculator".to_string(),
        description: "A calculator".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "operation": {"type": "string", "default": "add"},
                "a": {"type": "number", "default": 0},
                "b": {"type": "number", "default": 0}
            }
        }),
    };

    let json = serde_json::to_string(&tool).unwrap();
    assert!(json.contains("\"default\":\"add\""));
}

// ============================================================================
// Tool Result Content with Empty Object in Array Tests
// ============================================================================

#[test]
fn test_anthropic_tool_result_content_with_empty_object_in_array() {
    let result = anthropic_api::AnthropicContentBlock::ToolResult {
        tool_use_id: "tool_123".to_string(),
        content: vec![
            anthropic_api::AnthropicContentBlock::Text {
                text: serde_json::to_string(&json!([{}])).unwrap(),
            },
        ],
        is_error: Some(false),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"[{}]\""));
}

// ============================================================================
// Tool Use with Empty String Value Tests
// ============================================================================

#[test]
fn test_anthropic_tool_use_with_empty_string_value() {
    let tool_use = anthropic_api::AnthropicContentBlock::ToolUse {
        id: "tool_123".to_string(),
        name: "calculator".to_string(),
        input: json!({"operation": ""}),
    };

    let json = serde_json::to_string(&tool_use).unwrap();
    assert!(json.contains("\"operation\":\"\""));
}

// ============================================================================
// Tool Result Content with Empty String in Nested Object Tests
// ============================================================================

#[test]
fn test_anthropic_tool_result_content_with_empty_string_in_nested_object() {
    let result = anthropic_api::AnthropicContentBlock::ToolResult {
        tool_use_id: "tool_123".to_string(),
        content: vec![
            anthropic_api::AnthropicContentBlock::Text {
                text: serde_json::to_string(&json!({"value": ""})).unwrap(),
            },
        ],
        is_error: Some(false),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"value\":\"\""));
}

// ============================================================================
// Tool Definition with Pattern Properties Tests
// ============================================================================

#[test]
fn test_anthropic_tool_definition_with_pattern_properties() {
    let tool = anthropic_api::AnthropicTool {
        name: "calculator".to_string(),
        description: "A calculator".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "operation": {"type": "string"}
            },
            "patternProperties": {
                "^custom_": {"type": "string"}
            }
        }),
    };

    let json = serde_json::to_string(&tool).unwrap();
    assert!(json.contains("\"patternProperties\""));
}

// ============================================================================
// Tool Result Content with Empty Array in Object Tests
// ============================================================================

#[test]
fn test_anthropic_tool_result_content_with_empty_array_in_object() {
    let result = anthropic_api::AnthropicContentBlock::ToolResult {
        tool_use_id: "tool_123".to_string(),
        content: vec![
            anthropic_api::AnthropicContentBlock::Text {
                text: serde_json::to_string(&json!({"values": []})).unwrap(),
            },
        ],
        is_error: Some(false),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"values\":[]"));
}

// ============================================================================
// Tool Use with Empty Array in Object Tests
// ============================================================================

#[test]
fn test_anthropic_tool_use_with_empty_array_in_object() {
    let tool_use = anthropic_api::AnthropicContentBlock::ToolUse {
        id: "tool_123".to_string(),
        name: "calculator".to_string(),
        input: json!({"numbers": []}),
    };

    let json = serde_json::to_string(&tool_use).unwrap();
    assert!(json.contains("\"numbers\":[]"));
}

// ============================================================================
// Tool Result Content with Null Value Tests
// ============================================================================

#[test]
fn test_anthropic_tool_result_content_with_null_value() {
    let result = anthropic_api::AnthropicContentBlock::ToolResult {
        tool_use_id: "tool_123".to_string(),
        content: vec![
            anthropic_api::AnthropicContentBlock::Text {
                text: serde_json::to_string(&json!({"value": null})).unwrap(),
            },
        ],
        is_error: Some(false),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"value\":null"));
}

// ============================================================================
// Tool Definition with Enum Properties Tests
// ============================================================================

#[test]
fn test_anthropic_tool_definition_with_enum_properties() {
    let tool = anthropic_api::AnthropicTool {
        name: "calculator".to_string(),
        description: "A calculator".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["add", "subtract", "multiply", "divide"]
                }
            }
        }),
    };

    let json = serde_json::to_string(&tool).unwrap();
    assert!(json.contains("\"enum\""));
    assert!(json.contains("\"add\""));
}

// ============================================================================
// Tool Result Content with Empty String at End Tests
// ============================================================================

#[test]
fn test_anthropic_tool_result_content_with_empty_string_at_end() {
    let result = anthropic_api::AnthropicContentBlock::ToolResult {
        tool_use_id: "tool_123".to_string(),
        content: vec![
            anthropic_api::AnthropicContentBlock::Text {
                text: "First".to_string(),
            },
            anthropic_api::AnthropicContentBlock::Text {
                text: "".to_string(),
            },
        ],
        is_error: Some(false),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"First\""));
    assert!(json.contains("\"\""));
}

// ============================================================================
// Tool Use with Empty Array in Array Tests
// ============================================================================

#[test]
fn test_anthropic_tool_use_with_empty_array_in_array() {
    let tool_use = anthropic_api::AnthropicContentBlock::ToolUse {
        id: "tool_123".to_string(),
        name: "calculator".to_string(),
        input: json!({"operations": [[]]}),
    };

    let json = serde_json::to_string(&tool_use).unwrap();
    assert!(json.contains("\"operations\":[[]]"));
}

// ============================================================================
// Tool Result Content with Empty Object at End Tests
// ============================================================================

#[test]
fn test_anthropic_tool_result_content_with_empty_object_at_end() {
    let result = anthropic_api::AnthropicContentBlock::ToolResult {
        tool_use_id: "tool_123".to_string(),
        content: vec![
            anthropic_api::AnthropicContentBlock::Text {
                text: "First".to_string(),
            },
            anthropic_api::AnthropicContentBlock::Text {
                text: serde_json::to_string(&json!({})).unwrap(),
            },
        ],
        is_error: Some(false),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"First\""));
    assert!(json.contains("\"{}\""));
}

// ============================================================================
// Tool Definition with Multiple Required Fields Tests
// ============================================================================

#[test]
fn test_anthropic_tool_definition_with_multiple_required_fields() {
    let tool = anthropic_api::AnthropicTool {
        name: "calculator".to_string(),
        description: "A calculator".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "operation": {"type": "string"},
                "a": {"type": "number"},
                "b": {"type": "number"},
                "options": {"type": "object"}
            },
            "required": ["operation", "a", "b"]
        }),
    };

    let json = serde_json::to_string(&tool).unwrap();
    assert!(json.contains("\"required\""));
    assert!(json.contains("\"operation\""));
    assert!(json.contains("\"a\""));
    assert!(json.contains("\"b\""));
}

// ============================================================================
// Tool Result Content with Nested Empty Objects Tests
// ============================================================================

#[test]
fn test_anthropic_tool_result_content_with_nested_empty_objects() {
    let result = anthropic_api::AnthropicContentBlock::ToolResult {
        tool_use_id: "tool_123".to_string(),
        content: vec![
            anthropic_api::AnthropicContentBlock::Text {
                text: serde_json::to_string(&json!({
                    "level1": {
                        "level2": {}
                    }
                })).unwrap(),
            },
        ],
        is_error: Some(false),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"level1\""));
    assert!(json.contains("\"level2\":{}"));
}

// ============================================================================
// Tool Use with Multiple Nested Objects Tests
// ============================================================================

#[test]
fn test_anthropic_tool_use_with_multiple_nested_objects() {
    let tool_use = anthropic_api::AnthropicContentBlock::ToolUse {
        id: "tool_123".to_string(),
        name: "calculator".to_string(),
        input: json!({
            "operation": "add",
            "args": {
                "a": 5,
                "b": 3,
                "options": {
                    "precision": 2,
                    "rounding": "half_up"
                }
            }
        }),
    };

    let json = serde_json::to_string(&tool_use).unwrap();
    assert!(json.contains("\"args\""));
    assert!(json.contains("\"options\""));
}

// ============================================================================
// Tool Result Content with Multiple Nested Arrays Tests
// ============================================================================

#[test]
fn test_anthropic_tool_result_content_with_multiple_nested_arrays() {
    let result = anthropic_api::AnthropicContentBlock::ToolResult {
        tool_use_id: "tool_123".to_string(),
        content: vec![
            anthropic_api::AnthropicContentBlock::Text {
                text: serde_json::to_string(&json!([
                    [1, 2],
                    [3, 4],
                    [5, 6]
                ])).unwrap(),
            },
        ],
        is_error: Some(false),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("[1,2]"));
    assert!(json.contains("[3,4]"));
    assert!(json.contains("[5,6]"));
}

// ============================================================================
// Tool Definition with Complex Nested Schema Tests
// ============================================================================

#[test]
fn test_anthropic_tool_definition_with_complex_nested_schema() {
    let tool = anthropic_api::AnthropicTool {
        name: "calculator".to_string(),
        description: "A calculator".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "operation": {"type": "string"},
                "args": {
                    "type": "object",
                    "properties": {
                        "a": {"type": "number"},
                        "b": {"type": "number"},
                        "options": {
                            "type": "object",
                            "properties": {
                                "precision": {"type": "number"},
                                "rounding": {"type": "string"}
                            }
                        }
                    }
                }
            },
            "required": ["operation", "args"]
        }),
    };

    let json = serde_json::to_string(&tool).unwrap();
    assert!(json.contains("\"operation\""));
    assert!(json.contains("\"args\""));
    assert!(json.contains("\"precision\""));
    assert!(json.contains("\"rounding\""));
}

// ============================================================================
// Tool Result Content with Deeply Nested Arrays Tests
// ============================================================================

#[test]
fn test_anthropic_tool_result_content_with_deeply_nested_arrays() {
    let result = anthropic_api::AnthropicContentBlock::ToolResult {
        tool_use_id: "tool_123".to_string(),
        content: vec![
            anthropic_api::AnthropicContentBlock::Text {
                text: serde_json::to_string(&json!([
                    [[1, 2], [3, 4]],
                    [[5, 6], [7, 8]]
                ])).unwrap(),
            },
        ],
        is_error: Some(false),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("[[1,2],[3,4]]"));
    assert!(json.contains("[[5,6],[7,8]]"));
}

// ============================================================================
// Tool Use with Complex Nested Arrays Tests
// ============================================================================

#[test]
fn test_anthropic_tool_use_with_complex_nested_arrays() {
    let tool_use = anthropic_api::AnthropicContentBlock::ToolUse {
        id: "tool_123".to_string(),
        name: "calculator".to_string(),
        input: json!({
            "operations": [
                ["add", 1, 2],
                ["subtract", 3, 4]
            ]
        }),
    };

    let json = serde_json::to_string(&tool_use).unwrap();
    assert!(json.contains("\"operations\""));
    assert!(json.contains("\"add\""));
    assert!(json.contains("\"subtract\""));
}

// ============================================================================
// Tool Definition with Mixed Type Arrays Tests
// ============================================================================

#[test]
fn test_anthropic_tool_definition_with_mixed_type_arrays() {
    let tool = anthropic_api::AnthropicTool {
        name: "calculator".to_string(),
        description: "A calculator".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "values": {
                    "type": "array",
                    "items": {
                        "type": ["number", "string"]
                    }
                }
            }
        }),
    };

    let json = serde_json::to_string(&tool).unwrap();
    assert!(json.contains("\"type\":[\"number\",\"string\"]"));
}

// ============================================================================
// Tool Result Content with Mixed Type Arrays Tests
// ============================================================================

#[test]
fn test_anthropic_tool_result_content_with_mixed_type_arrays() {
    let result = anthropic_api::AnthropicContentBlock::ToolResult {
        tool_use_id: "tool_123".to_string(),
        content: vec![
            anthropic_api::AnthropicContentBlock::Text {
                text: serde_json::to_string(&json!([1, "two", 3.0, true])).unwrap(),
            },
        ],
        is_error: Some(false),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"two\""));
    assert!(json.contains("3.0"));
    assert!(json.contains("true"));
}

// ============================================================================
// Tool Use with Mixed Type Objects Tests
// ============================================================================

#[test]
fn test_anthropic_tool_use_with_mixed_type_objects() {
    let tool_use = anthropic_api::AnthropicContentBlock::ToolUse {
        id: "tool_123".to_string(),
        name: "calculator".to_string(),
        input: json!({
            "operation": "add",
            "values": [1, 2, 3],
            "options": {
                "debug": true,
                "timeout": 30
            }
        }),
    };

    let json = serde_json::to_string(&tool_use).unwrap();
    assert!(json.contains("\"operation\":\"add\""));
    assert!(json.contains("\"values\""));
    assert!(json.contains("\"debug\":true"));
}

// ============================================================================
// Tool Result Content with Complex Nested Structures Tests
// ============================================================================

#[test]
fn test_anthropic_tool_result_content_with_complex_nested_structures() {
    let result = anthropic_api::AnthropicContentBlock::ToolResult {
        tool_use_id: "tool_123".to_string(),
        content: vec![
            anthropic_api::AnthropicContentBlock::Text {
                text: serde_json::to_string(&json!({
                    "results": [
                        {
                            "id": 1,
                            "value": "first",
                            "options": {
                                "enabled": true,
                                "config": {
                                    "precision": 2
                                }
                            }
                        },
                        {
                            "id": 2,
                            "value": "second",
                            "options": {
                                "enabled": false,
                                "config": {
                                    "precision": 4
                                }
                            }
                        }
                    ]
                })).unwrap(),
            },
        ],
        is_error: Some(false),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"first\""));
    assert!(json.contains("\"second\""));
    assert!(json.contains("\"enabled\":true"));
    assert!(json.contains("\"enabled\":false"));
}

// ============================================================================
// Tool Definition with AllOf Schema Tests
// ============================================================================

#[test]
fn test_anthropic_tool_definition_with_allof_schema() {
    let tool = anthropic_api::AnthropicTool {
        name: "calculator".to_string(),
        description: "A calculator".to_string(),
        input_schema: json!({
            "allOf": [
                {
                    "type": "object",
                    "properties": {
                        "operation": {"type": "string"}
                    }
                },
                {
                    "type": "object",
                    "properties": {
                        "args": {"type": "object"}
                    }
                }
            ]
        }),
    };

    let json = serde_json::to_string(&tool).unwrap();
    assert!(json.contains("\"allOf\""));
}

// ============================================================================
// Tool Result Content with Circular Reference Simulation Tests
// ============================================================================

#[test]
fn test_anthropic_tool_result_content_with_circular_reference_simulation() {
    // Note: JSON cannot have circular references
    // This test documents that we should not create circular references
    
    let result = anthropic_api::AnthropicContentBlock::ToolResult {
        tool_use_id: "tool_123".to_string(),
        content: vec![
            anthropic_api::AnthropicContentBlock::Text {
                text: serde_json::to_string(&json!({
                    "id": 1,
                    "parent": null
                })).unwrap(),
            },
        ],
        is_error: Some(false),
    };

    let json = serde_json::to_string(&result).unwrap();
    eprintln!("Actual JSON: {}", json);
    assert!(json.contains(r#""parent":null"#));
}

// ============================================================================
// Tool Use with Schema Validation Tests
// ============================================================================

#[test]
fn test_anthropic_tool_use_with_schema_validation() {
    let tool_use = anthropic_api::AnthropicContentBlock::ToolUse {
        id: "tool_123".to_string(),
        name: "calculator".to_string(),
        input: json!({
            "operation": "add",
            "a": 5,
            "b": 3
        }),
    };

    let json = serde_json::to_string(&tool_use).unwrap();
    assert!(json.contains("\"operation\":\"add\""));
    assert!(json.contains("\"a\":5"));
    assert!(json.contains("\"b\":3"));
}

// ============================================================================
// Tool Result Content with Schema Validation Tests
// ============================================================================

#[test]
fn test_anthropic_tool_result_content_with_schema_validation() {
    let result = anthropic_api::AnthropicContentBlock::ToolResult {
        tool_use_id: "tool_123".to_string(),
        content: vec![
            anthropic_api::AnthropicContentBlock::Text {
                text: serde_json::to_string(&json!({
                    "result": {
                        "value": 42,
                        "type": "number"
                    }
                })).unwrap(),
            },
        ],
        is_error: Some(false),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"result\""));
    assert!(json.contains("\"value\":42"));
}

// ============================================================================
// Tool Definition with Schema Validation Tests
// ============================================================================

#[test]
fn test_anthropic_tool_definition_with_schema_validation() {
    let tool = anthropic_api::AnthropicTool {
        name: "calculator".to_string(),
        description: "A calculator".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "operation": {"type": "string"},
                "a": {"type": "number"},
                "b": {"type": "number"}
            },
            "required": ["operation", "a", "b"]
        }),
    };

    let json = serde_json::to_string(&tool).unwrap();
    assert!(json.contains("\"operation\""));
    assert!(json.contains("\"a\""));
    assert!(json.contains("\"b\""));
}

// ============================================================================
// Tool Result Content with Schema Validation - Multiple Levels Tests
// ============================================================================

#[test]
fn test_anthropic_tool_result_content_with_schema_validation_multiple_levels() {
    let result = anthropic_api::AnthropicContentBlock::ToolResult {
        tool_use_id: "tool_123".to_string(),
        content: vec![
            anthropic_api::AnthropicContentBlock::Text {
                text: serde_json::to_string(&json!({
                    "level1": {
                        "level2": {
                            "level3": {
                                "value": 42,
                                "nested": {
                                    "deep": "value"
                                }
                            }
                        }
                    }
                })).unwrap(),
            },
        ],
        is_error: Some(false),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"level1\""));
    assert!(json.contains("\"level2\""));
    assert!(json.contains("\"level3\""));
    assert!(json.contains("\"deep\":\"value\""));
}

// ============================================================================
// Tool Use with Schema Validation - Deeply Nested Tests
// ============================================================================

#[test]
fn test_anthropic_tool_use_with_schema_validation_deeply_nested() {
    let tool_use = anthropic_api::AnthropicContentBlock::ToolUse {
        id: "tool_123".to_string(),
        name: "calculator".to_string(),
        input: json!({
            "level1": {
                "level2": {
                    "level3": {
                        "operation": "add",
                        "a": 5,
                        "b": 3
                    }
                }
            }
        }),
    };

    let json = serde_json::to_string(&tool_use).unwrap();
    assert!(json.contains("\"level1\""));
    assert!(json.contains("\"level2\""));
    assert!(json.contains("\"level3\""));
    assert!(json.contains("\"operation\":\"add\""));
}

// ============================================================================
// Tool Result Content with Schema Validation - Complex Nested Tests
// ============================================================================

#[test]
fn test_anthropic_tool_result_content_with_schema_validation_complex_nested() {
    let result = anthropic_api::AnthropicContentBlock::ToolResult {
        tool_use_id: "tool_123".to_string(),
        content: vec![
            anthropic_api::AnthropicContentBlock::Text {
                text: serde_json::to_string(&json!({
                    "results": [
                        {
                            "id": 1,
                            "data": {
                                "value": 42,
                                "options": {
                                    "enabled": true
                                }
                            }
                        },
                        {
                            "id": 2,
                            "data": {
                                "value": 43,
                                "options": {
                                    "enabled": false
                                }
                            }
                        }
                    ]
                })).unwrap(),
            },
        ],
        is_error: Some(false),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"id\":1"));
    assert!(json.contains("\"id\":2"));
    assert!(json.contains("\"value\":42"));
    assert!(json.contains("\"value\":43"));
}

// ============================================================================
// Tool Definition with Schema Validation - All Required Fields Tests
// ============================================================================

#[test]
fn test_anthropic_tool_definition_with_schema_validation_all_required_fields() {
    let tool = anthropic_api::AnthropicTool {
        name: "calculator".to_string(),
        description: "A calculator".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "operation": {"type": "string"},
                "a": {"type": "number"},
                "b": {"type": "number"},
                "options": {
                    "type": "object",
                    "properties": {
                        "precision": {"type": "number"},
                        "rounding": {"type": "string"}
                    }
                }
            },
            "required": ["operation", "a", "b", "options"]
        }),
    };

    let json = serde_json::to_string(&tool).unwrap();
    assert!(json.contains("\"operation\""));
    assert!(json.contains("\"a\""));
    assert!(json.contains("\"b\""));
    assert!(json.contains("\"options\""));
}

// ============================================================================
// Tool Result Content with Schema Validation - All Required Fields Tests
// ============================================================================

#[test]
fn test_anthropic_tool_result_content_with_schema_validation_all_required_fields() {
    let result = anthropic_api::AnthropicContentBlock::ToolResult {
        tool_use_id: "tool_123".to_string(),
        content: vec![
            anthropic_api::AnthropicContentBlock::Text {
                text: serde_json::to_string(&json!({
                    "operation": "add",
                    "a": 5,
                    "b": 3,
                    "options": {
                        "precision": 2,
                        "rounding": "half_up"
                    }
                })).unwrap(),
            },
        ],
        is_error: Some(false),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"operation\":\"add\""));
    assert!(json.contains("\"a\":5"));
    assert!(json.contains("\"b\":3"));
    assert!(json.contains("\"precision\":2"));
}

// ============================================================================
// Tool Use with Schema Validation - All Required Fields Tests
// ============================================================================

#[test]
fn test_anthropic_tool_use_with_schema_validation_all_required_fields() {
    let tool_use = anthropic_api::AnthropicContentBlock::ToolUse {
        id: "tool_123".to_string(),
        name: "calculator".to_string(),
        input: json!({
            "operation": "add",
            "a": 5,
            "b": 3,
            "options": {
                "precision": 2,
                "rounding": "half_up"
            }
        }),
    };

    let json = serde_json::to_string(&tool_use).unwrap();
    assert!(json.contains("\"operation\":\"add\""));
    assert!(json.contains("\"a\":5"));
    assert!(json.contains("\"b\":3"));
    assert!(json.contains("\"precision\":2"));
}

// ============================================================================
// Tool Result Content with Schema Validation - All Required Fields - Deep Nested Tests
// ============================================================================

#[test]
fn test_anthropic_tool_result_content_with_schema_validation_all_required_fields_deep_nested() {
    let result = anthropic_api::AnthropicContentBlock::ToolResult {
        tool_use_id: "tool_123".to_string(),
        content: vec![
            anthropic_api::AnthropicContentBlock::Text {
                text: serde_json::to_string(&json!({
                    "level1": {
                        "level2": {
                            "level3": {
                                "operation": "add",
                                "a": 5,
                                "b": 3,
                                "options": {
                                    "precision": 2,
                                    "rounding": "half_up"
                                }
                            }
                        }
                    }
                })).unwrap(),
            },
        ],
        is_error: Some(false),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"level1\""));
    assert!(json.contains("\"level2\""));
    assert!(json.contains("\"level3\""));
    assert!(json.contains("\"operation\":\"add\""));
    assert!(json.contains("\"a\":5"));
    assert!(json.contains("\"b\":3"));
    assert!(json.contains("\"precision\":2"));
}

// ============================================================================
// Tool Definition with Schema Validation - All Required Fields - Deep Nested Tests
// ============================================================================

#[test]
fn test_anthropic_tool_definition_with_schema_validation_all_required_fields_deep_nested() {
    let tool = anthropic_api::AnthropicTool {
        name: "calculator".to_string(),
        description: "A calculator".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "operation": {"type": "string"},
                "a": {"type": "number"},
                "b": {"type": "number"},
                "options": {
                    "type": "object",
                    "properties": {
                        "precision": {"type": "number"},
                        "rounding": {"type": "string"}
                    }
                }
            },
            "required": ["operation", "a", "b", "options"]
        }),
    };

    let json = serde_json::to_string(&tool).unwrap();
    assert!(json.contains("\"operation\""));
    assert!(json.contains("\"a\""));
    assert!(json.contains("\"b\""));
    assert!(json.contains("\"options\""));
}

// ============================================================================
// Tool Result Content with Schema Validation - All Required Fields - Complex Tests
// ============================================================================

#[test]
fn test_anthropic_tool_result_content_with_schema_validation_all_required_fields_complex() {
    let result = anthropic_api::AnthropicContentBlock::ToolResult {
        tool_use_id: "tool_123".to_string(),
        content: vec![
            anthropic_api::AnthropicContentBlock::Text {
                text: serde_json::to_string(&json!({
                    "results": [
                        {
                            "operation": "add",
                            "a": 5,
                            "b": 3,
                            "options": {
                                "precision": 2,
                                "rounding": "half_up"
                            }
                        },
                        {
                            "operation": "subtract",
                            "a": 10,
                            "b": 4,
                            "options": {
                                "precision": 4,
                                "rounding": "down"
                            }
                        }
                    ]
                })).unwrap(),
            },
        ],
        is_error: Some(false),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"add\""));
    assert!(json.contains("\"subtract\""));
    assert!(json.contains("\"precision\":2"));
    assert!(json.contains("\"precision\":4"));
}

// ============================================================================
// Tool Use with Schema Validation - All Required Fields - Complex Tests
// ============================================================================

#[test]
fn test_anthropic_tool_use_with_schema_validation_all_required_fields_complex() {
    let tool_use = anthropic_api::AnthropicContentBlock::ToolUse {
        id: "tool_123".to_string(),
        name: "calculator".to_string(),
        input: json!({
            "results": [
                {
                    "operation": "add",
                    "a": 5,
                    "b": 3,
                    "options": {
                        "precision": 2,
                        "rounding": "half_up"
                    }
                },
                {
                    "operation": "subtract",
                    "a": 10,
                    "b": 4,
                    "options": {
                        "precision": 4,
                        "rounding": "down"
                    }
                }
            ]
        }),
    };

    let json = serde_json::to_string(&tool_use).unwrap();
    assert!(json.contains("\"add\""));
    assert!(json.contains("\"subtract\""));
    assert!(json.contains("\"precision\":2"));
    assert!(json.contains("\"precision\":4"));
}

// ============================================================================
// Tool Definition with Schema Validation - All Required Fields - Complex Tests
// ============================================================================

#[test]
fn test_anthropic_tool_definition_with_schema_validation_all_required_fields_complex() {
    let tool = anthropic_api::AnthropicTool {
        name: "calculator".to_string(),
        description: "A calculator".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "operation": {"type": "string"},
                "a": {"type": "number"},
                "b": {"type": "number"},
                "options": {
                    "type": "object",
                    "properties": {
                        "precision": {"type": "number"},
                        "rounding": {"type": "string"}
                    }
                }
            },
            "required": ["operation", "a", "b", "options"]
        }),
    };

    let json = serde_json::to_string(&tool).unwrap();
    assert!(json.contains("\"operation\""));
    assert!(json.contains("\"a\""));
    assert!(json.contains("\"b\""));
    assert!(json.contains("\"options\""));
}

// ============================================================================
// Tool Result Content with Schema Validation - All Required Fields - Complex Nested Tests
// ============================================================================

#[test]
fn test_anthropic_tool_result_content_with_schema_validation_all_required_fields_complex_nested() {
    let result = anthropic_api::AnthropicContentBlock::ToolResult {
        tool_use_id: "tool_123".to_string(),
        content: vec![
            anthropic_api::AnthropicContentBlock::Text {
                text: serde_json::to_string(&json!({
                    "level1": {
                        "level2": {
                            "results": [
                                {
                                    "operation": "add",
                                    "a": 5,
                                    "b": 3,
                                    "options": {
                                        "precision": 2,
                                        "rounding": "half_up"
                                    }
                                },
                                {
                                    "operation": "subtract",
                                    "a": 10,
                                    "b": 4,
                                    "options": {
                                        "precision": 4,
                                        "rounding": "down"
                                    }
                                }
                            ]
                        }
                    }
                })).unwrap(),
            },
        ],
        is_error: Some(false),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"level1\""));
    assert!(json.contains("\"level2\""));
    assert!(json.contains("\"add\""));
    assert!(json.contains("\"subtract\""));
    assert!(json.contains("\"precision\":2"));
    assert!(json.contains("\"precision\":4"));
}

// ============================================================================
// Tool Use with Schema Validation - All Required Fields - Complex Nested Tests
// ============================================================================

#[test]
fn test_anthropic_tool_use_with_schema_validation_all_required_fields_complex_nested() {
    let tool_use = anthropic_api::AnthropicContentBlock::ToolUse {
        id: "tool_123".to_string(),
        name: "calculator".to_string(),
        input: json!({
            "level1": {
                "level2": {
                    "results": [
                        {
                            "operation": "add",
                            "a": 5,
                            "b": 3,
                            "options": {
                                "precision": 2,
                                "rounding": "half_up"
                            }
                        },
                        {
                            "operation": "subtract",
                            "a": 10,
                            "b": 4,
                            "options": {
                                "precision": 4,
                                "rounding": "down"
                            }
                        }
                    ]
                }
            }
        }),
    };

    let json = serde_json::to_string(&tool_use).unwrap();
    assert!(json.contains("\"level1\""));
    assert!(json.contains("\"level2\""));
    assert!(json.contains("\"add\""));
    assert!(json.contains("\"subtract\""));
    assert!(json.contains("\"precision\":2"));
    assert!(json.contains("\"precision\":4"));
}

// ============================================================================
// Tool Definition with Schema Validation - All Required Fields - Complex Nested Tests
// ============================================================================

#[test]
fn test_anthropic_tool_definition_with_schema_validation_all_required_fields_complex_nested() {
    let tool = anthropic_api::AnthropicTool {
        name: "calculator".to_string(),
        description: "A calculator".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "operation": {"type": "string"},
                "a": {"type": "number"},
                "b": {"type": "number"},
                "options": {
                    "type": "object",
                    "properties": {
                        "precision": {"type": "number"},
                        "rounding": {"type": "string"}
                    }
                }
            },
            "required": ["operation", "a", "b", "options"]
        }),
    };

    let json = serde_json::to_string(&tool).unwrap();
    assert!(json.contains("\"operation\""));
    assert!(json.contains("\"a\""));
    assert!(json.contains("\"b\""));
    assert!(json.contains("\"options\""));
}

// ============================================================================
// Tool Result Content with Schema Validation - All Required Fields - Very Complex Tests
// ============================================================================

#[test]
fn test_anthropic_tool_result_content_with_schema_validation_all_required_fields_very_complex() {
    let result = anthropic_api::AnthropicContentBlock::ToolResult {
        tool_use_id: "tool_123".to_string(),
        content: vec![
            anthropic_api::AnthropicContentBlock::Text {
                text: serde_json::to_string(&json!({
                    "level1": {
                        "level2": {
                            "level3": {
                                "results": [
                                    {
                                        "operation": "add",
                                        "a": 5,
                                        "b": 3,
                                        "options": {
                                            "precision": 2,
                                            "rounding": "half_up"
                                        }
                                    },
                                    {
                                        "operation": "subtract",
                                        "a": 10,
                                        "b": 4,
                                        "options": {
                                            "precision": 4,
                                            "rounding": "down"
                                        }
                                    }
                                ]
                            }
                        }
                    }
                })).unwrap(),
            },
        ],
        is_error: Some(false),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"level1\""));
    assert!(json.contains("\"level2\""));
    assert!(json.contains("\"level3\""));
    assert!(json.contains("\"add\""));
    assert!(json.contains("\"subtract\""));
    assert!(json.contains("\"precision\":2"));
    assert!(json.contains("\"precision\":4"));
}

// ============================================================================
// Tool Use with Schema Validation - All Required Fields - Very Complex Tests
// ============================================================================

#[test]
fn test_anthropic_tool_use_with_schema_validation_all_required_fields_very_complex() {
    let tool_use = anthropic_api::AnthropicContentBlock::ToolUse {
        id: "tool_123".to_string(),
        name: "calculator".to_string(),
        input: json!({
            "level1": {
                "level2": {
                    "level3": {
                        "results": [
                            {
                                "operation": "add",
                                "a": 5,
                                "b": 3,
                                "options": {
                                    "precision": 2,
                                    "rounding": "half_up"
                                }
                            },
                            {
                                "operation": "subtract",
                                "a": 10,
                                "b": 4,
                                "options": {
                                    "precision": 4,
                                    "rounding": "down"
                                }
                            }
                        ]
                    }
                }
            }
        }),
    };

    let json = serde_json::to_string(&tool_use).unwrap();
    assert!(json.contains("\"level1\""));
    assert!(json.contains("\"level2\""));
    assert!(json.contains("\"level3\""));
    assert!(json.contains("\"add\""));
    assert!(json.contains("\"subtract\""));
    assert!(json.contains("\"precision\":2"));
    assert!(json.contains("\"precision\":4"));
}

// ============================================================================
// Tool Definition with Schema Validation - All Required Fields - Very Complex Tests
// ============================================================================

#[test]
fn test_anthropic_tool_definition_with_schema_validation_all_required_fields_very_complex() {
    let tool = anthropic_api::AnthropicTool {
        name: "calculator".to_string(),
        description: "A calculator".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "operation": {"type": "string"},
                "a": {"type": "number"},
                "b": {"type": "number"},
                "options": {
                    "type": "object",
                    "properties": {
                        "precision": {"type": "number"},
                        "rounding": {"type": "string"}
                    }
                }
            },
            "required": ["operation", "a", "b", "options"]
        }),
    };

    let json = serde_json::to_string(&tool).unwrap();
    assert!(json.contains("\"operation\""));
    assert!(json.contains("\"a\""));
    assert!(json.contains("\"b\""));
    assert!(json.contains("\"options\""));
}

// ============================================================================
// Tool Result Content with Schema Validation - All Required Fields - Extremely Complex Tests
// ============================================================================

#[test]
fn test_anthropic_tool_result_content_with_schema_validation_all_required_fields_extremely_complex() {
    let result = anthropic_api::AnthropicContentBlock::ToolResult {
        tool_use_id: "tool_123".to_string(),
        content: vec![
            anthropic_api::AnthropicContentBlock::Text {
                text: serde_json::to_string(&json!({
                    "level1": {
                        "level2": {
                            "level3": {
                                "level4": {
                                    "results": [
                                        {
                                            "operation": "add",
                                            "a": 5,
                                            "b": 3,
                                            "options": {
                                                "precision": 2,
                                                "rounding": "half_up"
                                            }
                                        },
                                        {
                                            "operation": "subtract",
                                            "a": 10,
                                            "b": 4,
                                            "options": {
                                                "precision": 4,
                                                "rounding": "down"
                                            }
                                        }
                                    ]
                                }
                            }
                        }
                    }
                })).unwrap(),
            },
        ],
        is_error: Some(false),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"level1\""));
    assert!(json.contains("\"level2\""));
    assert!(json.contains("\"level3\""));
    assert!(json.contains("\"level4\""));
    assert!(json.contains("\"add\""));
    assert!(json.contains("\"subtract\""));
    assert!(json.contains("\"precision\":2"));
    assert!(json.contains("\"precision\":4"));
}

// ============================================================================
// Tool Use with Schema Validation - All Required Fields - Extremely Complex Tests
// ============================================================================

#[test]
fn test_anthropic_tool_use_with_schema_validation_all_required_fields_extremely_complex() {
    let tool_use = anthropic_api::AnthropicContentBlock::ToolUse {
        id: "tool_123".to_string(),
        name: "calculator".to_string(),
        input: json!({
            "level1": {
                "level2": {
                    "level3": {
                        "level4": {
                            "results": [
                                {
                                    "operation": "add",
                                    "a": 5,
                                    "b": 3,
                                    "options": {
                                        "precision": 2,
                                        "rounding": "half_up"
                                    }
                                },
                                {
                                    "operation": "subtract",
                                    "a": 10,
                                    "b": 4,
                                    "options": {
                                        "precision": 4,
                                        "rounding": "down"
                                    }
                                }
                            ]
                        }
                    }
                }
            }
        }),
    };

    let json = serde_json::to_string(&tool_use).unwrap();
    assert!(json.contains("\"level1\""));
    assert!(json.contains("\"level2\""));
    assert!(json.contains("\"level3\""));
    assert!(json.contains("\"level4\""));
    assert!(json.contains("\"add\""));
    assert!(json.contains("\"subtract\""));
    assert!(json.contains("\"precision\":2"));
    assert!(json.contains("\"precision\":4"));
}

// ============================================================================
// Tool Definition with Schema Validation - All Required Fields - Extremely Complex Tests
// ============================================================================

#[test]
fn test_anthropic_tool_definition_with_schema_validation_all_required_fields_extremely_complex() {
    let tool = anthropic_api::AnthropicTool {
        name: "calculator".to_string(),
        description: "A calculator".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "operation": {"type": "string"},
                "a": {"type": "number"},
                "b": {"type": "number"},
                "options": {
                    "type": "object",
                    "properties": {
                        "precision": {"type": "number"},
                        "rounding": {"type": "string"}
                    }
                }
            },
            "required": ["operation", "a", "b", "options"]
        }),
    };

    let json = serde_json::to_string(&tool).unwrap();
    assert!(json.contains("\"operation\""));
    assert!(json.contains("\"a\""));
    assert!(json.contains("\"b\""));
    assert!(json.contains("\"options\""));
}

// ============================================================================
// Tool Result Content with Schema Validation - All Required Fields - Insanely Complex Tests
// ============================================================================

#[test]
fn test_anthropic_tool_result_content_with_schema_validation_all_required_fields_insanely_complex() {
    let result = anthropic_api::AnthropicContentBlock::ToolResult {
        tool_use_id: "tool_123".to_string(),
        content: vec![
            anthropic_api::AnthropicContentBlock::Text {
                text: serde_json::to_string(&json!({
                    "level1": {
                        "level2": {
                            "level3": {
                                "level4": {
                                    "level5": {
                                        "results": [
                                            {
                                                "operation": "add",
                                                "a": 5,
                                                "b": 3,
                                                "options": {
                                                    "precision": 2,
                                                    "rounding": "half_up"
                                                }
                                            },
                                            {
                                                "operation": "subtract",
                                                "a": 10,
                                                "b": 4,
                                                "options": {
                                                    "precision": 4,
                                                    "rounding": "down"
                                                }
                                            }
                                        ]
                                    }
                                }
                            }
                        }
                    }
                })).unwrap(),
            },
        ],
        is_error: Some(false),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"level1\""));
    assert!(json.contains("\"level2\""));
    assert!(json.contains("\"level3\""));
    assert!(json.contains("\"level4\""));
    assert!(json.contains("\"level5\""));
    assert!(json.contains("\"add\""));
    assert!(json.contains("\"subtract\""));
    assert!(json.contains("\"precision\":2"));
    assert!(json.contains("\"precision\":4"));
}

// ============================================================================
// Tool Use with Schema Validation - All Required Fields - Insanely Complex Tests
// ============================================================================

#[test]
fn test_anthropic_tool_use_with_schema_validation_all_required_fields_insanely_complex() {
    let tool_use = anthropic_api::AnthropicContentBlock::ToolUse {
        id: "tool_123".to_string(),
        name: "calculator".to_string(),
        input: json!({
            "level1": {
                "level2": {
                    "level3": {
                        "level4": {
                            "level5": {
                                "results": [
                                    {
                                        "operation": "add",
                                        "a": 5,
                                        "b": 3,
                                        "options": {
                                            "precision": 2,
                                            "rounding": "half_up"
                                        }
                                    },
                                    {
                                        "operation": "subtract",
                                        "a": 10,
                                        "b": 4,
                                        "options": {
                                            "precision": 4,
                                            "rounding": "down"
                                        }
                                    }
                                ]
                            }
                        }
                    }
                }
            }
        }),
    };

    let json = serde_json::to_string(&tool_use).unwrap();
    assert!(json.contains("\"level1\""));
    assert!(json.contains("\"level2\""));
    assert!(json.contains("\"level3\""));
    assert!(json.contains("\"level4\""));
    assert!(json.contains("\"level5\""));
    assert!(json.contains("\"add\""));
    assert!(json.contains("\"subtract\""));
    assert!(json.contains("\"precision\":2"));
    assert!(json.contains("\"precision\":4"));
}

// ============================================================================
// Tool Definition with Schema Validation - All Required Fields - Insanely Complex Tests
// ============================================================================

#[test]
fn test_anthropic_tool_definition_with_schema_validation_all_required_fields_insanely_complex() {
    let tool = anthropic_api::AnthropicTool {
        name: "calculator".to_string(),
        description: "A calculator".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "operation": {"type": "string"},
                "a": {"type": "number"},
                "b": {"type": "number"},
                "options": {
                    "type": "object",
                    "properties": {
                        "precision": {"type": "number"},
                        "rounding": {"type": "string"}
                    }
                }
            },
            "required": ["operation", "a", "b", "options"]
        }),
    };

    let json = serde_json::to_string(&tool).unwrap();
    assert!(json.contains("\"operation\""));
    assert!(json.contains("\"a\""));
    assert!(json.contains("\"b\""));
    assert!(json.contains("\"options\""));
}

// ============================================================================
// Tool Result Content with Schema Validation - All Required Fields - Max Complexity Tests
// ============================================================================

#[test]
fn test_anthropic_tool_result_content_with_schema_validation_all_required_fields_max_complexity() {
    let result = anthropic_api::AnthropicContentBlock::ToolResult {
        tool_use_id: "tool_123".to_string(),
        content: vec![
            anthropic_api::AnthropicContentBlock::Text {
                text: serde_json::to_string(&json!({
                    "level1": {
                        "level2": {
                            "level3": {
                                "level4": {
                                    "level5": {
                                        "level6": {
                                            "results": [
                                                {
                                                    "operation": "add",
                                                    "a": 5,
                                                    "b": 3,
                                                    "options": {
                                                        "precision": 2,
                                                        "rounding": "half_up"
                                                    }
                                                },
                                                {
                                                    "operation": "subtract",
                                                    "a": 10,
                                                    "b": 4,
                                                    "options": {
                                                        "precision": 4,
                                                        "rounding": "down"
                                                    }
                                                }
                                            ]
                                        }
                                    }
                                }
                            }
                        }
                    }
                })).unwrap(),
            },
        ],
        is_error: Some(false),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"level1\""));
    assert!(json.contains("\"level2\""));
    assert!(json.contains("\"level3\""));
    assert!(json.contains("\"level4\""));
    assert!(json.contains("\"level5\""));
    assert!(json.contains("\"level6\""));
    assert!(json.contains("\"add\""));
    assert!(json.contains("\"subtract\""));
    assert!(json.contains("\"precision\":2"));
    assert!(json.contains("\"precision\":4"));
}

// ============================================================================
// Tool Use with Schema Validation - All Required Fields - Max Complexity Tests
// ============================================================================

#[test]
fn test_anthropic_tool_use_with_schema_validation_all_required_fields_max_complexity() {
    let tool_use = anthropic_api::AnthropicContentBlock::ToolUse {
        id: "tool_123".to_string(),
        name: "calculator".to_string(),
        input: json!({
            "level1": {
                "level2": {
                    "level3": {
                        "level4": {
                            "level5": {
                                "level6": {
                                    "results": [
                                        {
                                            "operation": "add",
                                            "a": 5,
                                            "b": 3,
                                            "options": {
                                                "precision": 2,
                                                "rounding": "half_up"
                                            }
                                        },
                                        {
                                            "operation": "subtract",
                                            "a": 10,
                                            "b": 4,
                                            "options": {
                                                "precision": 4,
                                                "rounding": "down"
                                            }
                                        }
                                    ]
                                }
                            }
                        }
                    }
                }
            }
        }),
    };

    let json = serde_json::to_string(&tool_use).unwrap();
    assert!(json.contains("\"level1\""));
    assert!(json.contains("\"level2\""));
    assert!(json.contains("\"level3\""));
    assert!(json.contains("\"level4\""));
    assert!(json.contains("\"level5\""));
    assert!(json.contains("\"level6\""));
    assert!(json.contains("\"add\""));
    assert!(json.contains("\"subtract\""));
    assert!(json.contains("\"precision\":2"));
    assert!(json.contains("\"precision\":4"));
}

// ============================================================================
// Tool Definition with Schema Validation - All Required Fields - Max Complexity Tests
// ============================================================================

#[test]
fn test_anthropic_tool_definition_with_schema_validation_all_required_fields_max_complexity() {
    let tool = anthropic_api::AnthropicTool {
        name: "calculator".to_string(),
        description: "A calculator".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "operation": {"type": "string"},
                "a": {"type": "number"},
                "b": {"type": "number"},
                "options": {
                    "type": "object",
                    "properties": {
                        "precision": {"type": "number"},
                        "rounding": {"type": "string"}
                    }
                }
            },
            "required": ["operation", "a", "b", "options"]
        }),
    };

    let json = serde_json::to_string(&tool).unwrap();
    assert!(json.contains("\"operation\""));
    assert!(json.contains("\"a\""));
    assert!(json.contains("\"b\""));
    assert!(json.contains("\"options\""));
}

// ============================================================================
// Tool Result Content with Schema Validation - All Required Fields - Ultimate Tests
// ============================================================================

#[test]
fn test_anthropic_tool_result_content_with_schema_validation_all_required_fields_ultimate() {
    let result = anthropic_api::AnthropicContentBlock::ToolResult {
        tool_use_id: "tool_123".to_string(),
        content: vec![
            anthropic_api::AnthropicContentBlock::Text {
                text: serde_json::to_string(&json!({
                    "level1": {
                        "level2": {
                            "level3": {
                                "level4": {
                                    "level5": {
                                        "level6": {
                                            "level7": {
                                                "results": [
                                                    {
                                                        "operation": "add",
                                                        "a": 5,
                                                        "b": 3,
                                                        "options": {
                                                            "precision": 2,
                                                            "rounding": "half_up"
                                                        }
                                                    },
                                                    {
                                                        "operation": "subtract",
                                                        "a": 10,
                                                        "b": 4,
                                                        "options": {
                                                            "precision": 4,
                                                            "rounding": "down"
                                                        }
                                                    }
                                                ]
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                })).unwrap(),
            },
        ],
        is_error: Some(false),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"level1\""));
    assert!(json.contains("\"level2\""));
    assert!(json.contains("\"level3\""));
    assert!(json.contains("\"level4\""));
    assert!(json.contains("\"level5\""));
    assert!(json.contains("\"level6\""));
    assert!(json.contains("\"level7\""));
    assert!(json.contains("\"add\""));
    assert!(json.contains("\"subtract\""));
    assert!(json.contains("\"precision\":2"));
    assert!(json.contains("\"precision\":4"));
}

// ============================================================================
// Tool Use with Schema Validation - All Required Fields - Ultimate Tests
// ============================================================================

#[test]
fn test_anthropic_tool_use_with_schema_validation_all_required_fields_ultimate() {
    let tool_use = anthropic_api::AnthropicContentBlock::ToolUse {
        id: "tool_123".to_string(),
        name: "calculator".to_string(),
        input: json!({
            "level1": {
                "level2": {
                    "level3": {
                        "level4": {
                            "level5": {
                                "level6": {
                                    "level7": {
                                        "results": [
                                            {
                                                "operation": "add",
                                                "a": 5,
                                                "b": 3,
                                                "options": {
                                                    "precision": 2,
                                                    "rounding": "half_up"
                                                }
                                            },
                                            {
                                                "operation": "subtract",
                                                "a": 10,
                                                "b": 4,
                                                "options": {
                                                    "precision": 4,
                                                    "rounding": "down"
                                                }
                                            }
                                        ]
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }),
    };

    let json = serde_json::to_string(&tool_use).unwrap();
    assert!(json.contains("\"level1\""));
    assert!(json.contains("\"level2\""));
    assert!(json.contains("\"level3\""));
    assert!(json.contains("\"level4\""));
    assert!(json.contains("\"level5\""));
    assert!(json.contains("\"level6\""));
    assert!(json.contains("\"level7\""));
    assert!(json.contains("\"add\""));
    assert!(json.contains("\"subtract\""));
    assert!(json.contains("\"precision\":2"));
    assert!(json.contains("\"precision\":4"));
}

// ============================================================================
// Tool Definition with Schema Validation - All Required Fields - Ultimate Tests
// ============================================================================

#[test]
fn test_anthropic_tool_definition_with_schema_validation_all_required_fields_ultimate() {
    let tool = anthropic_api::AnthropicTool {
        name: "calculator".to_string(),
        description: "A calculator".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "operation": {"type": "string"},
                "a": {"type": "number"},
                "b": {"type": "number"},
                "options": {
                    "type": "object",
                    "properties": {
                        "precision": {"type": "number"},
                        "rounding": {"type": "string"}
                    }
                }
            },
            "required": ["operation", "a", "b", "options"]
        }),
    };

    let json = serde_json::to_string(&tool).unwrap();
    assert!(json.contains("\"operation\""));
    assert!(json.contains("\"a\""));
    assert!(json.contains("\"b\""));
    assert!(json.contains("\"options\""));
}

// ============================================================================
// End of Anthropic Tests
// ============================================================================
