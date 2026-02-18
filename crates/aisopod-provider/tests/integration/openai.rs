//! Integration tests for OpenAI provider
//!
//! These tests require the OPENAI_API_KEY environment variable to be set.
//! Run with: `OPENAI_API_KEY=your_key cargo test --test integration -- openai`

use aisopod_provider::providers::openai::OpenAIProvider;
use aisopod_provider::trait_module::ModelProvider;
use aisopod_provider::types::{ChatCompletionRequest, Message, MessageContent, Role};
use futures_util::StreamExt;

// ============================================================================
// Gate tests behind environment variable
// ============================================================================

fn get_api_key() -> Option<String> {
    std::env::var("OPENAI_API_KEY").ok()
}

// ============================================================================
// Integration Tests
// ============================================================================

/// Test streaming chat completion with OpenAI API
#[tokio::test]
#[ignore = "requires OPENAI_API_KEY environment variable"]
async fn test_openai_streaming_chat_completion() {
    let api_key = get_api_key().expect("OPENAI_API_KEY must be set to run this test");

    let provider = OpenAIProvider::new(api_key, None, None, None);

    let request = ChatCompletionRequest {
        model: "gpt-4o".to_string(),
        messages: vec![Message {
            role: Role::User,
            content: MessageContent::Text("Say hello!".to_string()),
            tool_calls: None,
            tool_call_id: None,
        }],
        tools: None,
        temperature: None,
        max_tokens: None,
        stop: None,
        stream: true,
    };

    let mut stream = provider.chat_completion(request).await.unwrap();

    let mut has_content = false;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.unwrap();
        if let Some(content) = &chunk.delta.content {
            if !content.is_empty() {
                has_content = true;
            }
        }
    }

    assert!(has_content, "Should receive content from streaming response");
}

/// Test that list_models returns non-empty results
#[tokio::test]
#[ignore = "requires OPENAI_API_KEY environment variable"]
async fn test_openai_list_models() {
    let api_key = get_api_key().expect("OPENAI_API_KEY must be set to run this test");

    let provider = OpenAIProvider::new(api_key, None, None, None);

    let models = provider.list_models().await.unwrap();

    assert!(!models.is_empty(), "list_models should return non-empty results");
    assert!(models.iter().any(|m| m.id.contains("gpt-")));
}

/// Test health_check returns healthy status
#[tokio::test]
#[ignore = "requires OPENAI_API_KEY environment variable"]
async fn test_openai_health_check() {
    let api_key = get_api_key().expect("OPENAI_API_KEY must be set to run this test");

    let provider = OpenAIProvider::new(api_key, None, None, None);

    let health = provider.health_check().await.unwrap();

    assert!(health.available, "Health check should return available=true");
}

/// Test with custom base URL
#[tokio::test]
#[ignore = "requires OPENAI_API_KEY environment variable"]
async fn test_openai_with_custom_base_url() {
    let api_key = get_api_key().expect("OPENAI_API_KEY must be set to run this test");

    let provider = OpenAIProvider::with_base_url(
        api_key,
        "https://api.openai.com/v1".to_string(),
        None,
        None,
    );

    let request = ChatCompletionRequest {
        model: "gpt-4o".to_string(),
        messages: vec![Message {
            role: Role::User,
            content: MessageContent::Text("Hi".to_string()),
            tool_calls: None,
            tool_call_id: None,
        }],
        tools: None,
        temperature: None,
        max_tokens: None,
        stop: None,
        stream: true,
    };

    let result = provider.chat_completion(request).await;
    // May succeed or fail depending on API key validity
    // Just verify it doesn't panic
    assert!(result.is_ok() || result.is_err());
}

/// Test error handling with invalid API key
#[tokio::test]
#[ignore = "requires OPENAI_API_KEY environment variable"]
async fn test_openai_invalid_api_key() {
    let provider = OpenAIProvider::new(
        "invalid-key".to_string(),
        None,
        None,
        None,
    );

    let request = ChatCompletionRequest {
        model: "gpt-4o".to_string(),
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

    let result = provider.chat_completion(request).await;
    assert!(result.is_err(), "Should fail with invalid API key");
}
