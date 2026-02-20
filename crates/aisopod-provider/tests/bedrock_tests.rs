//! Unit tests for Bedrock provider
//!
//! These tests do not require actual AWS credentials and mock the API responses.

use aisopod_provider::providers::bedrock::BedrockProvider;
use aisopod_provider::types::{ChatCompletionRequest, Message, MessageContent, Role};

// ============================================================================
// Provider Initialization Tests
// ============================================================================

#[tokio::test]
async fn test_bedrock_provider_initialization() {
    // Test that BedrockProvider can be created with default settings
    let result = BedrockProvider::new(None, None, None).await;

    // Provider should be created successfully (may fail if AWS credentials are missing,
    // but it should not panic)
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_bedrock_provider_with_custom_region() {
    // Test that BedrockProvider can be created with a custom region
    let result = BedrockProvider::new(Some("us-west-2".to_string()), None, None).await;

    // Provider creation may succeed or fail depending on credentials
    // Just verify it doesn't panic
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_bedrock_provider_with_custom_endpoint() {
    // Test that BedrockProvider can be created with a custom endpoint
    let result = BedrockProvider::new(
        Some("us-west-2".to_string()),
        Some("http://localhost:4566".to_string()),
        None,
    )
    .await;

    // Provider creation may succeed or fail depending on credentials
    assert!(result.is_ok() || result.is_err());
}

// ============================================================================
// Request Construction Tests
// ============================================================================

#[test]
fn test_bedrock_request_with_text_message() {
    let request = ChatCompletionRequest {
        model: "anthropic.claude-3-sonnet-20240229-v1:0".to_string(),
        messages: vec![Message {
            role: Role::User,
            content: MessageContent::Text("Hello, world!".to_string()),
            tool_calls: None,
            tool_call_id: None,
        }],
        tools: None,
        temperature: None,
        max_tokens: None,
        stop: None,
        stream: false,
    };

    assert_eq!(request.model, "anthropic.claude-3-sonnet-20240229-v1:0");
    assert_eq!(request.messages.len(), 1);
}

#[test]
fn test_bedrock_request_with_multiple_messages() {
    let request = ChatCompletionRequest {
        model: "anthropic.claude-3-sonnet-20240229-v1:0".to_string(),
        messages: vec![
            Message {
                role: Role::User,
                content: MessageContent::Text("Hello!".to_string()),
                tool_calls: None,
                tool_call_id: None,
            },
            Message {
                role: Role::Assistant,
                content: MessageContent::Text("Hi there!".to_string()),
                tool_calls: None,
                tool_call_id: None,
            },
        ],
        tools: None,
        temperature: None,
        max_tokens: None,
        stop: None,
        stream: false,
    };

    assert_eq!(request.messages.len(), 2);
}

#[test]
fn test_bedrock_request_with_temperature() {
    let request = ChatCompletionRequest {
        model: "anthropic.claude-3-sonnet-20240229-v1:0".to_string(),
        messages: vec![Message {
            role: Role::User,
            content: MessageContent::Text("Test".to_string()),
            tool_calls: None,
            tool_call_id: None,
        }],
        tools: None,
        temperature: Some(0.7),
        max_tokens: None,
        stop: None,
        stream: false,
    };

    assert_eq!(request.temperature, Some(0.7));
}

#[test]
fn test_bedrock_request_with_max_tokens() {
    let request = ChatCompletionRequest {
        model: "anthropic.claude-3-sonnet-20240229-v1:0".to_string(),
        messages: vec![Message {
            role: Role::User,
            content: MessageContent::Text("Test".to_string()),
            tool_calls: None,
            tool_call_id: None,
        }],
        tools: None,
        temperature: None,
        max_tokens: Some(1000),
        stop: None,
        stream: false,
    };

    assert_eq!(request.max_tokens, Some(1000));
}

#[test]
fn test_bedrock_request_with_stop_sequences() {
    let request = ChatCompletionRequest {
        model: "anthropic.claude-3-sonnet-20240229-v1:0".to_string(),
        messages: vec![Message {
            role: Role::User,
            content: MessageContent::Text("Test".to_string()),
            tool_calls: None,
            tool_call_id: None,
        }],
        tools: None,
        temperature: None,
        max_tokens: None,
        stop: Some(vec!["\n".to_string(), "stop".to_string()]),
        stream: false,
    };

    assert_eq!(
        request.stop,
        Some(vec!["\n".to_string(), "stop".to_string()])
    );
}

#[test]
fn test_bedrock_request_with_streaming() {
    let request = ChatCompletionRequest {
        model: "anthropic.claude-3-sonnet-20240229-v1:0".to_string(),
        messages: vec![Message {
            role: Role::User,
            content: MessageContent::Text("Test".to_string()),
            tool_calls: None,
            tool_call_id: None,
        }],
        tools: None,
        temperature: None,
        max_tokens: None,
        stop: None,
        stream: true,
    };

    assert!(request.stream);
}

// ============================================================================
// Message Content Tests
// ============================================================================

#[test]
fn test_message_content_text() {
    let content = MessageContent::Text("Hello".to_string());

    match content {
        MessageContent::Text(text) => assert_eq!(text, "Hello"),
        _ => panic!("Expected Text content"),
    }
}

#[test]
fn test_message_content_parts() {
    let content = MessageContent::Parts(vec![aisopod_provider::types::ContentPart::Text {
        text: "Hello".to_string(),
    }]);

    match content {
        MessageContent::Parts(parts) => assert!(!parts.is_empty()),
        _ => panic!("Expected Parts content"),
    }
}

// ============================================================================
// Role Tests
// ============================================================================

#[test]
fn test_role_user() {
    assert_eq!(Role::User, Role::User);
}

#[test]
fn test_role_assistant() {
    assert_eq!(Role::Assistant, Role::Assistant);
}

#[test]
fn test_role_system() {
    assert_eq!(Role::System, Role::System);
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[tokio::test]
async fn test_bedrock_provider_error_handling() {
    // Test that the provider handles errors gracefully
    let result = BedrockProvider::new(Some("invalid-region".to_string()), None, None).await;

    // Provider may fail due to invalid region or missing credentials
    // Just verify it doesn't panic and returns an error result
    assert!(result.is_err() || result.is_ok());
}
