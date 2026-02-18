//! Integration tests for Ollama provider
//!
//! These tests require the OLLAMA_BASE_URL environment variable to be set
//! (defaults to http://localhost:11434).
//! Run with: `OLLAMA_BASE_URL=http://localhost:11434 cargo test --test integration -- ollama`

use aisopod_provider::providers::ollama::OllamaProvider;
use aisopod_provider::trait_module::ModelProvider;
use aisopod_provider::types::{ChatCompletionRequest, Message, MessageContent, Role};
use futures_util::StreamExt;

// ============================================================================
// Gate tests behind environment variable
// ============================================================================

fn get_base_url() -> String {
    std::env::var("OLLAMA_BASE_URL")
        .unwrap_or_else(|_| "http://localhost:11434".to_string())
}

// ============================================================================
// Integration Tests
// ============================================================================

/// Test streaming chat completion with Ollama API
#[tokio::test]
#[ignore = "requires OLLAMA_BASE_URL environment variable (Ollama instance)"]
async fn test_ollama_streaming_chat_completion() {
    let base_url = get_base_url();
    let provider = OllamaProvider::new(Some(base_url));

    // Use a small model for faster testing
    let request = ChatCompletionRequest {
        model: "llama2".to_string(),
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
#[ignore = "requires OLLAMA_BASE_URL environment variable (Ollama instance)"]
async fn test_ollama_list_models() {
    let base_url = get_base_url();
    let provider = OllamaProvider::new(Some(base_url));

    let models = provider.list_models().await.unwrap();

    assert!(!models.is_empty(), "list_models should return non-empty results");
}

/// Test health_check returns healthy status
#[tokio::test]
#[ignore = "requires OLLAMA_BASE_URL environment variable (Ollama instance)"]
async fn test_ollama_health_check() {
    let base_url = get_base_url();
    let provider = OllamaProvider::new(Some(base_url));

    let health = provider.health_check().await.unwrap();

    assert!(health.available, "Health check should return available=true");
}

/// Test with a short message
#[tokio::test]
#[ignore = "requires OLLAMA_BASE_URL environment variable (Ollama instance)"]
async fn test_ollama_with_short_message() {
    let base_url = get_base_url();
    let provider = OllamaProvider::new(Some(base_url));

    let request = ChatCompletionRequest {
        model: "llama2".to_string(),
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

/// Test error handling when Ollama is not available
#[tokio::test]
#[ignore = "requires OLLAMA_BASE_URL environment variable (Ollama instance)"]
async fn test_ollama_unavailable() {
    // Use a non-existent URL to test error handling
    let provider = OllamaProvider::new(Some("http://localhost:9999".to_string()));

    let request = ChatCompletionRequest {
        model: "llama2".to_string(),
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
    // Should fail since Ollama is not available
    assert!(result.is_err(), "Should fail when Ollama is unavailable");
}
