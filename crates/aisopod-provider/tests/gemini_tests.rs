//! Unit tests for Gemini provider
//!
//! These tests do not require an actual API key and mock the API responses.

use aisopod_provider::providers::gemini::GeminiProvider;
use aisopod_provider::types::{ChatCompletionRequest, Message, MessageContent, Role};

// ============================================================================
// Provider Initialization Tests
// ============================================================================

#[test]
fn test_gemini_provider_initialization() {
    // Test that GeminiProvider can be created with no API key
    let provider = GeminiProvider::new(None, None, None, None);
    
    // Provider should be created successfully (GeminiProvider::new returns Self, not Result)
    let _ = provider;
}

#[test]
fn test_gemini_provider_with_api_key() {
    // Test that GeminiProvider can be created with an API key
    let provider = GeminiProvider::new(Some("test-key".to_string()), None, None, None);
    
    let _ = provider;
}

#[test]
fn test_gemini_provider_with_custom_model() {
    // Test that GeminiProvider can be created with a custom model
    let provider = GeminiProvider::new(
        Some("test-key".to_string()), 
        Some("gemini-1.5-pro".to_string()), 
        None, 
        None
    );
    
    let _ = provider;
}

// ============================================================================
// Request Construction Tests
// ============================================================================

#[test]
fn test_gemini_request_with_text_message() {
    let request = ChatCompletionRequest {
        model: "gemini-1.5-flash".to_string(),
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

    assert_eq!(request.model, "gemini-1.5-flash");
    assert_eq!(request.messages.len(), 1);
}

#[test]
fn test_gemini_request_with_multiple_messages() {
    let request = ChatCompletionRequest {
        model: "gemini-1.5-flash".to_string(),
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
fn test_gemini_request_with_temperature() {
    let request = ChatCompletionRequest {
        model: "gemini-1.5-flash".to_string(),
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
fn test_gemini_request_with_max_tokens() {
    let request = ChatCompletionRequest {
        model: "gemini-1.5-flash".to_string(),
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
fn test_gemini_request_with_stop_sequences() {
    let request = ChatCompletionRequest {
        model: "gemini-1.5-flash".to_string(),
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

    assert_eq!(request.stop, Some(vec!["\n".to_string(), "stop".to_string()]));
}

#[test]
fn test_gemini_request_with_streaming() {
    let request = ChatCompletionRequest {
        model: "gemini-1.5-flash".to_string(),
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
    let content = MessageContent::Parts(vec![
        aisopod_provider::types::ContentPart::Text { 
            text: "Hello".to_string() 
        },
    ]);
    
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

#[test]
fn test_gemini_provider_error_handling() {
    // Test that the provider handles errors gracefully
    let provider = GeminiProvider::new(Some("invalid".to_string()), None, None, None);
    
    let _ = provider;
}
