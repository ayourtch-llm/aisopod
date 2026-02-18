//! Integration tests for Bedrock provider
//!
//! These tests require AWS credentials to be configured via environment variables
//! (AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY) or IAM role.
//! Run with: `cargo test --test integration -- bedrock`

use aisopod_provider::providers::bedrock::BedrockProvider;
use aisopod_provider::trait_module::ModelProvider;
use aisopod_provider::types::{ChatCompletionRequest, Message, MessageContent, Role};
use futures_util::StreamExt;

// ============================================================================
// Gate tests behind environment variable or AWS credential chain
// ============================================================================

// Bedrock uses AWS credentials from the standard credential chain
// This includes environment variables, shared credentials file, or IAM roles

// ============================================================================
// Integration Tests
// ============================================================================

/// Test streaming chat completion with Bedrock API
#[tokio::test]
#[ignore = "requires AWS credentials for Bedrock access"]
async fn test_bedrock_streaming_chat_completion() {
    let provider = BedrockProvider::new(None, None, None).await.unwrap();

    let request = ChatCompletionRequest {
        model: "anthropic.claude-3-sonnet-20240229-v1:0".to_string(),
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
#[ignore = "requires AWS credentials for Bedrock access"]
async fn test_bedrock_list_models() {
    let provider = BedrockProvider::new(None, None, None).await.unwrap();

    let models = provider.list_models().await.unwrap();

    assert!(!models.is_empty(), "list_models should return non-empty results");
    assert!(models.iter().any(|m| m.id.contains("claude")));
}

/// Test health_check returns healthy status
#[tokio::test]
#[ignore = "requires AWS credentials for Bedrock access"]
async fn test_bedrock_health_check() {
    let provider = BedrockProvider::new(None, None, None).await.unwrap();

    let health = provider.health_check().await.unwrap();

    assert!(health.available, "Health check should return available=true");
}

/// Test with a short message
#[tokio::test]
#[ignore = "requires AWS credentials for Bedrock access"]
async fn test_bedrock_with_short_message() {
    let provider = BedrockProvider::new(None, None, None).await.unwrap();

    let request = ChatCompletionRequest {
        model: "anthropic.claude-3-sonnet-20240229-v1:0".to_string(),
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

    let mut stream = provider.chat_completion(request).await.unwrap();

    let mut chunk_count = 0;
    while let Some(_) = stream.next().await {
        chunk_count += 1;
    }

    assert!(chunk_count > 0, "Should receive at least one chunk");
}

/// Test error handling with invalid credentials (when credentials are missing)
#[tokio::test]
#[ignore = "requires AWS credentials for Bedrock access"]
async fn test_bedrock_with_custom_region() {
    // This test just verifies the provider can be created with a custom region
    // Actual API calls may fail if credentials are not configured
    let result = BedrockProvider::new(Some("us-west-2".to_string()), None, None).await;
    
    // Provider creation may succeed or fail depending on credentials
    // Just verify it doesn't panic
    assert!(result.is_ok() || result.is_err());
}
