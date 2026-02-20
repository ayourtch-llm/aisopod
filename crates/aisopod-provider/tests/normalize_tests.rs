//! Normalize tests for request/response normalization functions.
//!
//! These tests verify that:
//! - Message ordering is enforced (alternating turns)
//! - System prompts are extracted correctly
//! - Error mapping from HTTP to ProviderError works
//! - Token usage aggregation works correctly

use aisopod_provider::normalize::{
    aggregate_usage, enforce_alternating_turns, extract_system_prompt, map_http_error,
    ProviderError,
};
use aisopod_provider::types::*;

// ============================================================================
// Enforce Alternating Turns Tests
// ============================================================================

#[test]
fn test_enforce_alternating_turns_no_change() {
    let mut messages = vec![
        Message {
            role: Role::User,
            content: MessageContent::Text("First".to_string()),
            tool_calls: None,
            tool_call_id: None,
        },
        Message {
            role: Role::Assistant,
            content: MessageContent::Text("Response".to_string()),
            tool_calls: None,
            tool_call_id: None,
        },
        Message {
            role: Role::User,
            content: MessageContent::Text("Second".to_string()),
            tool_calls: None,
            tool_call_id: None,
        },
    ];

    enforce_alternating_turns(&mut messages);

    assert_eq!(messages.len(), 3);
}

#[test]
fn test_enforce_alternating_turns_merges_consecutive_user() {
    let mut messages = vec![
        Message {
            role: Role::User,
            content: MessageContent::Text("First".to_string()),
            tool_calls: None,
            tool_call_id: None,
        },
        Message {
            role: Role::User,
            content: MessageContent::Text("Second".to_string()),
            tool_calls: None,
            tool_call_id: None,
        },
        Message {
            role: Role::Assistant,
            content: MessageContent::Text("Response".to_string()),
            tool_calls: None,
            tool_call_id: None,
        },
    ];

    enforce_alternating_turns(&mut messages);

    // Should be 2 messages now: merged user + assistant
    assert_eq!(messages.len(), 2);

    // First message should be merged user content
    match &messages[0].content {
        MessageContent::Text(text) => {
            assert_eq!(text, "First Second");
        }
        _ => panic!("Expected Text content"),
    }

    // Second message should be assistant
    assert_eq!(messages[1].role, Role::Assistant);
}

#[test]
fn test_enforce_alternating_turns_merges_consecutive_assistant() {
    let mut messages = vec![
        Message {
            role: Role::User,
            content: MessageContent::Text("Hello".to_string()),
            tool_calls: None,
            tool_call_id: None,
        },
        Message {
            role: Role::Assistant,
            content: MessageContent::Text("First response".to_string()),
            tool_calls: None,
            tool_call_id: None,
        },
        Message {
            role: Role::Assistant,
            content: MessageContent::Text("Second response".to_string()),
            tool_calls: None,
            tool_call_id: None,
        },
    ];

    enforce_alternating_turns(&mut messages);

    // Should be 2 messages now: user + merged assistant
    assert_eq!(messages.len(), 2);

    // Second message should be merged assistant content
    match &messages[1].content {
        MessageContent::Text(text) => {
            assert_eq!(text, "First response Second response");
        }
        _ => panic!("Expected Text content"),
    }
}

#[test]
fn test_enforce_alternating_turns_multiple_merges() {
    let mut messages = vec![
        Message {
            role: Role::User,
            content: MessageContent::Text("A".to_string()),
            tool_calls: None,
            tool_call_id: None,
        },
        Message {
            role: Role::User,
            content: MessageContent::Text("B".to_string()),
            tool_calls: None,
            tool_call_id: None,
        },
        Message {
            role: Role::User,
            content: MessageContent::Text("C".to_string()),
            tool_calls: None,
            tool_call_id: None,
        },
        Message {
            role: Role::Assistant,
            content: MessageContent::Text("X".to_string()),
            tool_calls: None,
            tool_call_id: None,
        },
        Message {
            role: Role::Assistant,
            content: MessageContent::Text("Y".to_string()),
            tool_calls: None,
            tool_call_id: None,
        },
        Message {
            role: Role::User,
            content: MessageContent::Text("Z".to_string()),
            tool_calls: None,
            tool_call_id: None,
        },
    ];

    enforce_alternating_turns(&mut messages);

    // Should be 3 messages: merged user (A B C), merged assistant (X Y), user (Z)
    assert_eq!(messages.len(), 3);

    match &messages[0].content {
        MessageContent::Text(text) => {
            assert_eq!(text, "A B C");
        }
        _ => panic!("Expected Text content"),
    }

    match &messages[1].content {
        MessageContent::Text(text) => {
            assert_eq!(text, "X Y");
        }
        _ => panic!("Expected Text content"),
    }

    assert_eq!(messages[2].role, Role::User);
}

// ============================================================================
// Extract System Prompt Tests
// ============================================================================

#[test]
fn test_extract_system_prompt_none() {
    let mut messages = vec![
        Message {
            role: Role::User,
            content: MessageContent::Text("Hello".to_string()),
            tool_calls: None,
            tool_call_id: None,
        },
        Message {
            role: Role::Assistant,
            content: MessageContent::Text("Response".to_string()),
            tool_calls: None,
            tool_call_id: None,
        },
    ];

    let system = extract_system_prompt(&mut messages);

    assert!(system.is_none());
    assert_eq!(messages.len(), 2);
}

#[test]
fn test_extract_system_prompt_first() {
    let mut messages = vec![
        Message {
            role: Role::System,
            content: MessageContent::Text("Be helpful and concise".to_string()),
            tool_calls: None,
            tool_call_id: None,
        },
        Message {
            role: Role::User,
            content: MessageContent::Text("Hello".to_string()),
            tool_calls: None,
            tool_call_id: None,
        },
    ];

    let system = extract_system_prompt(&mut messages);

    assert_eq!(system, Some("Be helpful and concise".to_string()));
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].role, Role::User);
}

#[test]
fn test_extract_system_prompt_middle() {
    let mut messages = vec![
        Message {
            role: Role::User,
            content: MessageContent::Text("Hello".to_string()),
            tool_calls: None,
            tool_call_id: None,
        },
        Message {
            role: Role::System,
            content: MessageContent::Text("System message".to_string()),
            tool_calls: None,
            tool_call_id: None,
        },
        Message {
            role: Role::Assistant,
            content: MessageContent::Text("Response".to_string()),
            tool_calls: None,
            tool_call_id: None,
        },
    ];

    let system = extract_system_prompt(&mut messages);

    assert_eq!(system, Some("System message".to_string()));
    // System prompt should be removed, keeping other messages
    assert_eq!(messages.len(), 2);
}

#[test]
fn test_extract_system_prompt_multiple_returns_first() {
    let mut messages = vec![
        Message {
            role: Role::System,
            content: MessageContent::Text("First system".to_string()),
            tool_calls: None,
            tool_call_id: None,
        },
        Message {
            role: Role::User,
            content: MessageContent::Text("Hello".to_string()),
            tool_calls: None,
            tool_call_id: None,
        },
        Message {
            role: Role::System,
            content: MessageContent::Text("Second system".to_string()),
            tool_calls: None,
            tool_call_id: None,
        },
    ];

    let system = extract_system_prompt(&mut messages);

    // Only the first system prompt should be extracted
    assert_eq!(system, Some("First system".to_string()));
    assert_eq!(messages.len(), 2);
}

#[test]
fn test_extract_system_prompt_empty_messages() {
    let mut messages: Vec<Message> = vec![];

    let system = extract_system_prompt(&mut messages);

    assert!(system.is_none());
}

// ============================================================================
// Aggregate Usage Tests
// ============================================================================

#[test]
fn test_aggregate_usage_empty() {
    let result = aggregate_usage(&[]);
    assert_eq!(result.prompt_tokens, 0);
    assert_eq!(result.completion_tokens, 0);
    assert_eq!(result.total_tokens, 0);
}

#[test]
fn test_aggregate_usage_single() {
    let chunk = ChatCompletionChunk {
        id: "1".to_string(),
        delta: MessageDelta {
            role: Some(Role::Assistant),
            content: Some("test".to_string()),
            tool_calls: None,
        },
        finish_reason: Some(FinishReason::Stop),
        usage: Some(TokenUsage {
            prompt_tokens: 10,
            completion_tokens: 5,
            total_tokens: 15,
        }),
    };

    let result = aggregate_usage(&[chunk.clone()]);
    assert_eq!(result, chunk.usage.unwrap());
}

#[test]
fn test_aggregate_usage_multiple() {
    let chunk1 = ChatCompletionChunk {
        id: "1".to_string(),
        delta: MessageDelta {
            role: Some(Role::Assistant),
            content: Some("test1".to_string()),
            tool_calls: None,
        },
        finish_reason: Some(FinishReason::Stop),
        usage: Some(TokenUsage {
            prompt_tokens: 10,
            completion_tokens: 5,
            total_tokens: 15,
        }),
    };

    let chunk2 = ChatCompletionChunk {
        id: "2".to_string(),
        delta: MessageDelta {
            role: Some(Role::Assistant),
            content: Some("test2".to_string()),
            tool_calls: None,
        },
        finish_reason: Some(FinishReason::Stop),
        usage: Some(TokenUsage {
            prompt_tokens: 20,
            completion_tokens: 10,
            total_tokens: 30,
        }),
    };

    let result = aggregate_usage(&[chunk1, chunk2]);

    // Should sum all values
    assert_eq!(result.prompt_tokens, 30);
    assert_eq!(result.completion_tokens, 15);
    assert_eq!(result.total_tokens, 45);
}

#[test]
fn test_aggregate_usage_multiple_chunks() {
    let chunks = vec![
        ChatCompletionChunk {
            id: "1".to_string(),
            delta: MessageDelta {
                role: Some(Role::Assistant),
                content: Some("A".to_string()),
                tool_calls: None,
            },
            finish_reason: None,
            usage: Some(TokenUsage {
                prompt_tokens: 5,
                completion_tokens: 1,
                total_tokens: 6,
            }),
        },
        ChatCompletionChunk {
            id: "2".to_string(),
            delta: MessageDelta {
                role: None,
                content: Some("B".to_string()),
                tool_calls: None,
            },
            finish_reason: None,
            usage: Some(TokenUsage {
                prompt_tokens: 5,
                completion_tokens: 2,
                total_tokens: 7,
            }),
        },
        ChatCompletionChunk {
            id: "3".to_string(),
            delta: MessageDelta {
                role: None,
                content: None,
                tool_calls: None,
            },
            finish_reason: Some(FinishReason::Stop),
            usage: Some(TokenUsage {
                prompt_tokens: 5,
                completion_tokens: 3,
                total_tokens: 8,
            }),
        },
    ];

    let result = aggregate_usage(&chunks);

    // Should sum all values (5+5+5=15, 1+2+3=6, 6+7+8=21)
    assert_eq!(result.prompt_tokens, 15);
    assert_eq!(result.completion_tokens, 6);
    assert_eq!(result.total_tokens, 21);
}

// ============================================================================
// HTTP Error Mapping Tests
// ============================================================================

#[test]
fn test_map_http_error_401() {
    let error = map_http_error("openai", 401, r#"{"error": {"message": "Invalid key"}}"#);

    match error {
        ProviderError::AuthenticationFailed { provider, message } => {
            assert_eq!(provider, "openai");
            assert!(message.contains("Invalid key"));
        }
        _ => panic!("Expected AuthenticationFailed"),
    }
}

#[test]
fn test_map_http_error_403() {
    let error = map_http_error("openai", 403, "");

    match error {
        ProviderError::AuthenticationFailed { provider, .. } => {
            assert_eq!(provider, "openai");
        }
        _ => panic!("Expected AuthenticationFailed"),
    }
}

#[test]
fn test_map_http_error_429() {
    let error = map_http_error("openai", 429, "");

    match error {
        ProviderError::RateLimited { provider, .. } => {
            assert_eq!(provider, "openai");
        }
        _ => panic!("Expected RateLimited"),
    }
}

#[test]
fn test_map_http_error_400() {
    let error = map_http_error("openai", 400, "");

    match error {
        ProviderError::InvalidRequest { provider, .. } => {
            assert_eq!(provider, "openai");
        }
        _ => panic!("Expected InvalidRequest"),
    }
}

#[test]
fn test_map_http_error_404() {
    let error = map_http_error("openai", 404, "");

    match error {
        ProviderError::ModelNotFound { provider, .. } => {
            assert_eq!(provider, "openai");
        }
        _ => panic!("Expected ModelNotFound"),
    }
}

#[test]
fn test_map_http_error_413() {
    let error = map_http_error("openai", 413, "");

    match error {
        ProviderError::ContextLengthExceeded { provider, .. } => {
            assert_eq!(provider, "openai");
        }
        _ => panic!("Expected ContextLengthExceeded"),
    }
}

#[test]
fn test_map_http_error_500() {
    let error = map_http_error("openai", 500, "");

    match error {
        ProviderError::ServerError {
            provider, status, ..
        } => {
            assert_eq!(provider, "openai");
            assert_eq!(status, 500);
        }
        _ => panic!("Expected ServerError"),
    }
}

#[test]
fn test_map_http_error_unknown() {
    let error = map_http_error("openai", 418, "");

    match error {
        ProviderError::Unknown { provider, .. } => {
            assert_eq!(provider, "openai");
        }
        _ => panic!("Expected Unknown"),
    }
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_enforce_alternating_turns_single_message() {
    let mut messages = vec![Message {
        role: Role::User,
        content: MessageContent::Text("Single".to_string()),
        tool_calls: None,
        tool_call_id: None,
    }];

    enforce_alternating_turns(&mut messages);

    assert_eq!(messages.len(), 1);
}

#[test]
fn test_enforce_alternating_turns_empty() {
    let mut messages: Vec<Message> = vec![];

    enforce_alternating_turns(&mut messages);

    assert!(messages.is_empty());
}

#[test]
fn test_extract_system_prompt_only_system() {
    let mut messages = vec![Message {
        role: Role::System,
        content: MessageContent::Text("Only system".to_string()),
        tool_calls: None,
        tool_call_id: None,
    }];

    let system = extract_system_prompt(&mut messages);

    assert_eq!(system, Some("Only system".to_string()));
    assert!(messages.is_empty());
}

#[test]
fn test_aggregate_usage_with_none() {
    let chunks = vec![
        ChatCompletionChunk {
            id: "1".to_string(),
            delta: MessageDelta {
                role: Some(Role::Assistant),
                content: Some("A".to_string()),
                tool_calls: None,
            },
            finish_reason: None,
            usage: None, // No usage in this chunk
        },
        ChatCompletionChunk {
            id: "2".to_string(),
            delta: MessageDelta {
                role: None,
                content: Some("B".to_string()),
                tool_calls: None,
            },
            finish_reason: Some(FinishReason::Stop),
            usage: Some(TokenUsage {
                prompt_tokens: 5,
                completion_tokens: 3,
                total_tokens: 8,
            }),
        },
    ];

    // aggregate_usage now takes chunks directly
    let result = aggregate_usage(&chunks);
    assert_eq!(result.prompt_tokens, 5);
}

// ============================================================================
// Multi-Modal Content Tests
// ============================================================================

#[test]
fn test_enforce_alternating_turns_with_parts() {
    let mut messages = vec![
        Message {
            role: Role::User,
            content: MessageContent::Parts(vec![ContentPart::Text {
                text: "First".to_string(),
            }]),
            tool_calls: None,
            tool_call_id: None,
        },
        Message {
            role: Role::User,
            content: MessageContent::Text("Second".to_string()),
            tool_calls: None,
            tool_call_id: None,
        },
    ];

    enforce_alternating_turns(&mut messages);

    assert_eq!(messages.len(), 1);
    // The merged content should be Parts variant
    match &messages[0].content {
        MessageContent::Parts(parts) => {
            assert_eq!(parts.len(), 2);
        }
        _ => panic!("Expected Parts content"),
    }
}

// ============================================================================
// System Prompt with Parts Content Tests
// ============================================================================

#[test]
fn test_extract_system_prompt_with_parts() {
    let mut messages = vec![
        Message {
            role: Role::System,
            content: MessageContent::Parts(vec![ContentPart::Text {
                text: "System".to_string(),
            }]),
            tool_calls: None,
            tool_call_id: None,
        },
        Message {
            role: Role::User,
            content: MessageContent::Text("Hello".to_string()),
            tool_calls: None,
            tool_call_id: None,
        },
    ];

    let system = extract_system_prompt(&mut messages);

    assert_eq!(system, Some("System".to_string()));
    assert_eq!(messages.len(), 1);
}
