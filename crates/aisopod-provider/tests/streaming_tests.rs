//! Streaming behavior tests for providers.
//!
//! These tests verify:
//! - Chunks arrive in order
//! - Stream terminates correctly with finish_reason
//! - Partial failures are handled
//! - Empty streams are handled gracefully

use std::time::Duration;

use aisopod_provider::types::*;
use aisopod_provider::trait_module::ModelProvider;
use futures_util::StreamExt;

// Re-export the mock provider
use crate::helpers::{MockProvider, create_test_request};

// ============================================================================
// Streaming Order Tests
// ============================================================================

#[tokio::test]
async fn test_stream_chunks_arrive_in_order() {
    let chunks = vec![
        Ok(ChatCompletionChunk {
            id: "chunk_1".to_string(),
            delta: MessageDelta {
                role: Some(Role::Assistant),
                content: Some("Hello".to_string()),
                tool_calls: None,
            },
            finish_reason: None,
            usage: None,
        }),
        Ok(ChatCompletionChunk {
            id: "chunk_2".to_string(),
            delta: MessageDelta {
                role: None,
                content: Some(" world".to_string()),
                tool_calls: None,
            },
            finish_reason: None,
            usage: None,
        }),
        Ok(ChatCompletionChunk {
            id: "chunk_3".to_string(),
            delta: MessageDelta {
                role: None,
                content: Some("!".to_string()),
                tool_calls: None,
            },
            finish_reason: Some(FinishReason::Stop),
            usage: Some(TokenUsage {
                prompt_tokens: 5,
                completion_tokens: 3,
                total_tokens: 8,
            }),
        }),
    ];

    let provider = MockProvider::new("test")
        .with_chunks(chunks);

    let request = create_test_request("test-model", "Hello");
    let mut stream = provider.chat_completion(request).await.unwrap();

    let mut received_chunks = Vec::new();
    while let Some(chunk) = stream.next().await {
        received_chunks.push(chunk.unwrap());
    }

    assert_eq!(received_chunks.len(), 3);
    assert_eq!(received_chunks[0].delta.content, Some("Hello".to_string()));
    assert_eq!(received_chunks[1].delta.content, Some(" world".to_string()));
    assert_eq!(received_chunks[2].delta.content, Some("!".to_string()));
}

#[tokio::test]
async fn test_stream_finish_reason_on_last_chunk() {
    let chunks = vec![
        Ok(ChatCompletionChunk {
            id: "chunk_1".to_string(),
            delta: MessageDelta {
                role: Some(Role::Assistant),
                content: Some("Response".to_string()),
                tool_calls: None,
            },
            finish_reason: None,
            usage: None,
        }),
        Ok(ChatCompletionChunk {
            id: "chunk_2".to_string(),
            delta: MessageDelta {
                role: None,
                content: None,
                tool_calls: None,
            },
            finish_reason: Some(FinishReason::Stop),
            usage: Some(TokenUsage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            }),
        }),
    ];

    let provider = MockProvider::new("test")
        .with_chunks(chunks);

    let request = create_test_request("test-model", "Hello");
    let mut stream = provider.chat_completion(request).await.unwrap();

    let mut chunks_received = Vec::new();
    while let Some(chunk) = stream.next().await {
        chunks_received.push(chunk.unwrap());
    }

    assert_eq!(chunks_received.len(), 2);
    assert!(chunks_received[0].finish_reason.is_none());
    assert_eq!(chunks_received[1].finish_reason, Some(FinishReason::Stop));
}

// ============================================================================
// Stream Termination Tests
// ============================================================================

#[tokio::test]
async fn test_stream_terminates_correctly() {
    let chunks = vec![
        Ok(ChatCompletionChunk {
            id: "chunk_1".to_string(),
            delta: MessageDelta {
                role: Some(Role::Assistant),
                content: Some("A".to_string()),
                tool_calls: None,
            },
            finish_reason: None,
            usage: None,
        }),
        Ok(ChatCompletionChunk {
            id: "chunk_2".to_string(),
            delta: MessageDelta {
                role: None,
                content: Some("B".to_string()),
                tool_calls: None,
            },
            finish_reason: Some(FinishReason::Length),
            usage: Some(TokenUsage {
                prompt_tokens: 2,
                completion_tokens: 2,
                total_tokens: 4,
            }),
        }),
    ];

    let provider = MockProvider::new("test")
        .with_chunks(chunks);

    let request = create_test_request("test-model", "Test");
    let mut stream = provider.chat_completion(request).await.unwrap();

    let mut count = 0;
    let mut last_finish_reason = None;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.unwrap();
        count += 1;
        last_finish_reason = chunk.finish_reason;
    }

    assert_eq!(count, 2);
    assert_eq!(last_finish_reason, Some(FinishReason::Length));
}

#[tokio::test]
async fn test_stream_with_tool_call_finish_reason() {
    let chunks = vec![
        Ok(ChatCompletionChunk {
            id: "chunk_1".to_string(),
            delta: MessageDelta {
                role: Some(Role::Assistant),
                content: None,
                tool_calls: Some(vec![ToolCall {
                    id: "tool_1".to_string(),
                    name: "calculator".to_string(),
                    arguments: "{\"op\":\"add\"}".to_string(),
                }]),
            },
            finish_reason: None,
            usage: None,
        }),
        Ok(ChatCompletionChunk {
            id: "chunk_2".to_string(),
            delta: MessageDelta {
                role: None,
                content: None,
                tool_calls: None,
            },
            finish_reason: Some(FinishReason::ToolCall),
            usage: Some(TokenUsage {
                prompt_tokens: 15,
                completion_tokens: 5,
                total_tokens: 20,
            }),
        }),
    ];

    let provider = MockProvider::new("test")
        .with_chunks(chunks);

    let request = create_test_request("test-model", "Use a tool");
    let mut stream = provider.chat_completion(request).await.unwrap();

    let mut tool_calls_received = 0;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.unwrap();
        if chunk.delta.tool_calls.is_some() {
            tool_calls_received += 1;
        }
        if chunk.finish_reason == Some(FinishReason::ToolCall) {
            assert!(tool_calls_received > 0);
        }
    }

    assert_eq!(tool_calls_received, 1);
}

// ============================================================================
// Empty Stream Tests
// ============================================================================

#[tokio::test]
async fn test_empty_stream_handled_gracefully() {
    let provider = MockProvider::new("test")
        .with_chunks(vec![]);

    let request = create_test_request("test-model", "Test");
    let mut stream = provider.chat_completion(request).await.unwrap();

    let mut count = 0;
    while let Some(_) = stream.next().await {
        count += 1;
    }

    assert_eq!(count, 0);
}

#[tokio::test]
async fn test_stream_with_single_chunk() {
    let chunks = vec![Ok(ChatCompletionChunk {
        id: "chunk_1".to_string(),
        delta: MessageDelta {
            role: Some(Role::Assistant),
            content: Some("Only chunk".to_string()),
            tool_calls: None,
        },
        finish_reason: Some(FinishReason::Stop),
        usage: Some(TokenUsage {
            prompt_tokens: 3,
            completion_tokens: 3,
            total_tokens: 6,
        }),
    })];

    let provider = MockProvider::new("test")
        .with_chunks(chunks);

    let request = create_test_request("test-model", "Test");
    let mut stream = provider.chat_completion(request).await.unwrap();

    let mut count = 0;
    while let Some(_) = stream.next().await {
        count += 1;
    }

    assert_eq!(count, 1);
}

// ============================================================================
// Partial Failure Tests
// ============================================================================

#[tokio::test]
async fn test_stream_with_partial_failures() {
    let chunks = vec![
        Ok(ChatCompletionChunk {
            id: "chunk_1".to_string(),
            delta: MessageDelta {
                role: Some(Role::Assistant),
                content: Some("Part1".to_string()),
                tool_calls: None,
            },
            finish_reason: None,
            usage: None,
        }),
        Err("Network error".to_string()),
    ];

    let provider = MockProvider::new("test")
        .with_chunks(chunks);

    let request = create_test_request("test-model", "Test");
    let mut stream = provider.chat_completion(request).await.unwrap();

    let mut count = 0;
    let mut error_received = false;

    while let Some(result) = stream.next().await {
        match result {
            Ok(_) => count += 1,
            Err(_) => error_received = true,
        }
    }

    assert_eq!(count, 1);
    assert!(error_received);
}

#[tokio::test]
async fn test_stream_first_chunk_failure() {
    let chunks = vec![Err("Initial error".to_string())];

    let provider = MockProvider::new("test")
        .with_chunks(chunks);

    let request = create_test_request("test-model", "Test");
    let mut stream = provider.chat_completion(request).await.unwrap();

    let mut error_received = false;
    while let Some(result) = stream.next().await {
        if result.is_err() {
            error_received = true;
        }
    }

    assert!(error_received);
}

// ============================================================================
// Delayed Streaming Tests
// ============================================================================

#[tokio::test]
async fn test_stream_with_delays() {
    let chunks = vec![
        Ok(ChatCompletionChunk {
            id: "chunk_1".to_string(),
            delta: MessageDelta {
                role: Some(Role::Assistant),
                content: Some("A".to_string()),
                tool_calls: None,
            },
            finish_reason: None,
            usage: None,
        }),
        Ok(ChatCompletionChunk {
            id: "chunk_2".to_string(),
            delta: MessageDelta {
                role: None,
                content: Some("B".to_string()),
                tool_calls: None,
            },
            finish_reason: None,
            usage: None,
        }),
        Ok(ChatCompletionChunk {
            id: "chunk_3".to_string(),
            delta: MessageDelta {
                role: None,
                content: Some("C".to_string()),
                tool_calls: None,
            },
            finish_reason: Some(FinishReason::Stop),
            usage: Some(TokenUsage {
                prompt_tokens: 3,
                completion_tokens: 3,
                total_tokens: 6,
            }),
        }),
    ];

    let provider = MockProvider::new("test")
        .with_chunks(chunks)
        .with_delay_ms(10);

    let request = create_test_request("test-model", "Test");
    let start = std::time::Instant::now();

    let mut stream = provider.chat_completion(request).await.unwrap();
    let mut count = 0;

    while let Some(_) = stream.next().await {
        count += 1;
    }

    let elapsed = start.elapsed();
    
    // Should take at least 20ms for 3 chunks with 10ms delay each
    assert!(elapsed >= Duration::from_millis(20));
    assert_eq!(count, 3);
}

// ============================================================================
// Usage Aggregation Tests
// ============================================================================

#[tokio::test]
async fn test_usage_accumulates_across_chunks() {
    let chunks = vec![
        Ok(ChatCompletionChunk {
            id: "chunk_1".to_string(),
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
        }),
        Ok(ChatCompletionChunk {
            id: "chunk_2".to_string(),
            delta: MessageDelta {
                role: None,
                content: Some("B".to_string()),
                tool_calls: None,
            },
            finish_reason: Some(FinishReason::Stop),
            usage: Some(TokenUsage {
                prompt_tokens: 5,
                completion_tokens: 2,
                total_tokens: 7,
            }),
        }),
    ];

    let provider = MockProvider::new("test")
        .with_chunks(chunks);

    let request = create_test_request("test-model", "Test");
    let mut stream = provider.chat_completion(request).await.unwrap();

    let mut final_usage = None;
    while let Some(chunk) = stream.next().await {
        final_usage = chunk.unwrap().usage;
    }

    assert!(final_usage.is_some());
    assert_eq!(final_usage.unwrap().completion_tokens, 2);
}

// ============================================================================
// Chunk Ordering Verification Tests
// ============================================================================

#[tokio::test]
async fn test_chunk_ids_are_unique() {
    let chunks = vec![
        Ok(ChatCompletionChunk {
            id: "chunk_1".to_string(),
            delta: MessageDelta {
                role: Some(Role::Assistant),
                content: Some("A".to_string()),
                tool_calls: None,
            },
            finish_reason: None,
            usage: None,
        }),
        Ok(ChatCompletionChunk {
            id: "chunk_2".to_string(),
            delta: MessageDelta {
                role: None,
                content: Some("B".to_string()),
                tool_calls: None,
            },
            finish_reason: None,
            usage: None,
        }),
        Ok(ChatCompletionChunk {
            id: "chunk_3".to_string(),
            delta: MessageDelta {
                role: None,
                content: Some("C".to_string()),
                tool_calls: None,
            },
            finish_reason: Some(FinishReason::Stop),
            usage: None,
        }),
    ];

    let provider = MockProvider::new("test")
        .with_chunks(chunks);

    let request = create_test_request("test-model", "Test");
    let mut stream = provider.chat_completion(request).await.unwrap();

    let mut ids = Vec::new();
    while let Some(chunk) = stream.next().await {
        ids.push(chunk.unwrap().id);
    }

    assert_eq!(ids.len(), 3);
    assert_eq!(ids[0], "chunk_1");
    assert_eq!(ids[1], "chunk_2");
    assert_eq!(ids[2], "chunk_3");
}

// ============================================================================
// Role Delta Tests
// ============================================================================

#[tokio::test]
async fn test_role_delta_on_first_chunk() {
    let chunks = vec![Ok(ChatCompletionChunk {
        id: "chunk_1".to_string(),
        delta: MessageDelta {
            role: Some(Role::Assistant),
            content: Some("Response".to_string()),
            tool_calls: None,
        },
        finish_reason: None,
        usage: None,
    })];

    let provider = MockProvider::new("test")
        .with_chunks(chunks);

    let request = create_test_request("test-model", "Test");
    let mut stream = provider.chat_completion(request).await.unwrap();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.unwrap();
        assert_eq!(chunk.delta.role, Some(Role::Assistant));
    }
}

#[tokio::test]
async fn test_role_delta_not_repeated() {
    let chunks = vec![
        Ok(ChatCompletionChunk {
            id: "chunk_1".to_string(),
            delta: MessageDelta {
                role: Some(Role::Assistant),
                content: Some("Start".to_string()),
                tool_calls: None,
            },
            finish_reason: None,
            usage: None,
        }),
        Ok(ChatCompletionChunk {
            id: "chunk_2".to_string(),
            delta: MessageDelta {
                role: None,
                content: Some(" more".to_string()),
                tool_calls: None,
            },
            finish_reason: None,
            usage: None,
        }),
    ];

    let provider = MockProvider::new("test")
        .with_chunks(chunks);

    let request = create_test_request("test-model", "Test");
    let mut stream = provider.chat_completion(request).await.unwrap();

    let mut role_sent = false;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.unwrap();
        if let Some(role) = chunk.delta.role {
            assert!(!role_sent);
            assert_eq!(role, Role::Assistant);
            role_sent = true;
        }
    }
}
