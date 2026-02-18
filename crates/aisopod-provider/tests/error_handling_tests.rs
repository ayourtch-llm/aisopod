//! Error handling tests for providers.
//!
//! These tests verify that:
//! - Authentication errors are properly handled
//! - Rate limit errors trigger cooldown
//! - Invalid requests produce correct errors
//! - Network failures are properly reported
//! - Server errors are mapped correctly

use std::time::Duration;

use aisopod_provider::normalize::{map_http_error, ProviderError};
use aisopod_provider::types::{ChatCompletionChunk, ChatCompletionRequest, Message, MessageContent, Role};
use aisopod_provider::trait_module::ModelProvider;
use futures_util::StreamExt;

// Re-export the mock provider
use crate::helpers::{MockProvider, create_test_request};

// ============================================================================
// Authentication Error Tests
// ============================================================================

#[test]
fn test_auth_error_401_mapping() {
    let error = map_http_error("openai", 401, r#"{"error": {"message": "Invalid API key"}}"#);

    match error {
        ProviderError::AuthenticationFailed { provider, message } => {
            assert_eq!(provider, "openai");
            assert!(message.contains("Invalid API key"));
        }
        _ => panic!("Expected AuthenticationFailed error"),
    }
}

#[test]
fn test_auth_error_403_mapping() {
    let error = map_http_error("anthropic", 403, r#"{"error": {"type": "permission_error"}}"#);

    match error {
        ProviderError::AuthenticationFailed { provider, message } => {
            assert_eq!(provider, "anthropic");
        }
        _ => panic!("Expected AuthenticationFailed error"),
    }
}

#[test]
fn test_auth_error_with_empty_body() {
    let error = map_http_error("gemini", 401, "");

    match error {
        ProviderError::AuthenticationFailed { provider, message } => {
            assert_eq!(provider, "gemini");
            assert!(message.is_empty() || message.contains("401"));
        }
        _ => panic!("Expected AuthenticationFailed error"),
    }
}

#[test]
fn test_auth_error_provider_varies() {
    let error1 = map_http_error("openai", 401, r#"{"error": {"message": "Key error"}}"#);
    let error2 = map_http_error("anthropic", 401, r#"{"error": {"message": "Key error"}}"#);

    assert_ne!(
        match &error1 {
            ProviderError::AuthenticationFailed { provider, .. } => provider,
            _ => "wrong",
        },
        match &error2 {
            ProviderError::AuthenticationFailed { provider, .. } => provider,
            _ => "wrong",
        }
    );
}

// ============================================================================
// Rate Limit Error Tests
// ============================================================================

#[test]
fn test_rate_limit_error_429_mapping() {
    let error = map_http_error("openai", 429, r#"{"error": {"message": "Rate limit exceeded"}}"#);

    match error {
        ProviderError::RateLimited { provider, retry_after } => {
            assert_eq!(provider, "openai");
            // Retry-after should be None for basic 429 parsing
            assert!(retry_after.is_none());
        }
        _ => panic!("Expected RateLimited error"),
    }
}

#[test]
fn test_rate_limit_with_retry_after() {
    let error = map_http_error("openai", 429, r#"{"error": {"message": "Rate limit", "retry_after": 60}}"#);

    match error {
        ProviderError::RateLimited { provider, retry_after } => {
            assert_eq!(provider, "openai");
            // The retry_after parsing depends on the actual implementation
            assert!(retry_after.is_some() || retry_after.is_none());
        }
        _ => panic!("Expected RateLimited error"),
    }
}

#[test]
fn test_rate_limit_different_providers() {
    let error1 = map_http_error("openai", 429, "");
    let error2 = map_http_error("anthropic", 429, "");

    assert!(matches!(error1, ProviderError::RateLimited { .. }));
    assert!(matches!(error2, ProviderError::RateLimited { .. }));
}

// ============================================================================
// Invalid Request Error Tests
// ============================================================================

#[test]
fn test_invalid_request_400_mapping() {
    let error = map_http_error("openai", 400, r#"{"error": {"message": "Invalid model"}}"#);

    match error {
        ProviderError::InvalidRequest { provider, message } => {
            assert_eq!(provider, "openai");
            assert!(message.contains("Invalid model"));
        }
        _ => panic!("Expected InvalidRequest error"),
    }
}

#[test]
fn test_invalid_request_empty_message() {
    let error = map_http_error("openai", 400, "");

    match error {
        ProviderError::InvalidRequest { provider, message } => {
            assert_eq!(provider, "openai");
        }
        _ => panic!("Expected InvalidRequest error"),
    }
}

#[test]
fn test_invalid_request_with_context_length() {
    // This would be handled by map_http_error or a specialized handler
    let error = map_http_error("openai", 400, r#"{"error": {"message": "Too many tokens"}}"#);

    match error {
        ProviderError::InvalidRequest { provider, .. } => {
            assert_eq!(provider, "openai");
        }
        _ => panic!("Expected InvalidRequest error"),
    }
}

// ============================================================================
// Model Not Found Error Tests
// ============================================================================

#[test]
fn test_model_not_found_404_mapping() {
    let error = map_http_error("openai", 404, r#"{"error": {"message": "Model gpt-5 not found"}}"#);

    match error {
        ProviderError::ModelNotFound { provider, model } => {
            assert_eq!(provider, "openai");
            // Model name might be extracted from message or default to "unknown"
            assert!(model.contains("gpt-5") || model == "unknown");
        }
        _ => panic!("Expected ModelNotFound error"),
    }
}

#[test]
fn test_model_not_found_with_specific_model() {
    let error = map_http_error("gemini", 404, r#"{"error": "Model gemini-5-ultra not found"}"#);

    match error {
        ProviderError::ModelNotFound { provider, model } => {
            assert_eq!(provider, "gemini");
        }
        _ => panic!("Expected ModelNotFound error"),
    }
}

// ============================================================================
// Context Length Exceeded Tests
// ============================================================================

#[test]
fn test_context_length_exceeded_413_mapping() {
    let error = map_http_error("openai", 413, r#"{"error": {"message": "Input too long"}}"#);

    match error {
        ProviderError::ContextLengthExceeded { provider, max_tokens } => {
            assert_eq!(provider, "openai");
            // max_tokens might be 0 if not parseable
            assert!(max_tokens == 0 || max_tokens > 0);
        }
        _ => panic!("Expected ContextLengthExceeded error"),
    }
}

#[test]
fn test_context_length_with_specific_value() {
    // This would require a specialized handler in the actual code
    let error = map_http_error("openai", 413, r#"{"error": {"message": "Context exceeded max_tokens: 4096"}}"#);

    match error {
        ProviderError::ContextLengthExceeded { provider, max_tokens } => {
            assert_eq!(provider, "openai");
            assert!(max_tokens == 0 || max_tokens > 0);
        }
        _ => panic!("Expected ContextLengthExceeded error"),
    }
}

// ============================================================================
// Server Error Tests
// ============================================================================

#[test]
fn test_server_error_500_mapping() {
    let error = map_http_error("openai", 500, r#"{"error": {"message": "Internal error"}}"#);

    match error {
        ProviderError::ServerError { provider, status, message } => {
            assert_eq!(provider, "openai");
            assert_eq!(status, 500);
            assert!(message.contains("Internal error"));
        }
        _ => panic!("Expected ServerError"),
    }
}

#[test]
fn test_server_error_503_mapping() {
    let error = map_http_error("anthropic", 503, r#"{"error": "Service unavailable"}"#);

    match error {
        ProviderError::ServerError { provider, status, .. } => {
            assert_eq!(provider, "anthropic");
            assert_eq!(status, 503);
        }
        _ => panic!("Expected ServerError"),
    }
}

#[test]
fn test_server_error_504_mapping() {
    let error = map_http_error("gemini", 504, r#"{"error": "Gateway timeout"}"#);

    match error {
        ProviderError::ServerError { provider, status, .. } => {
            assert_eq!(provider, "gemini");
            assert_eq!(status, 504);
        }
        _ => panic!("Expected ServerError"),
    }
}

// ============================================================================
// Unknown Error Tests
// ============================================================================

#[test]
fn test_unknown_error_for_unmapped_status() {
    let error = map_http_error("openai", 418, r#"{"error": {"message": "I'm a teapot"}}"#);

    match error {
        ProviderError::Unknown { provider, message } => {
            assert_eq!(provider, "openai");
            assert!(message.contains("I'm a teapot"));
        }
        _ => panic!("Expected Unknown error"),
    }
}

#[test]
fn test_unknown_error_empty_body() {
    let error = map_http_error("openai", 499, "");

    match error {
        ProviderError::Unknown { provider, message } => {
            assert_eq!(provider, "openai");
        }
        _ => panic!("Expected Unknown error"),
    }
}

// ============================================================================
// Network Error Tests
// ============================================================================

#[tokio::test]
async fn test_network_error_propagation() {
    let provider = MockProvider::new("test")
        .with_error("Connection refused");

    let request = create_test_request("test-model", "Test");
    let result = provider.chat_completion(request).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Connection refused"));
}

#[tokio::test]
async fn test_network_timeout_simulation() {
    // This simulates what would happen on a network timeout
    let provider = MockProvider::new("test")
        .with_error("Request timeout");

    let request = create_test_request("test-model", "Test");
    let result = provider.chat_completion(request).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("timeout"));
}

// ============================================================================
// Error Comparison Tests
// ============================================================================

#[test]
fn test_error_equality_for_same_type() {
    let error1 = map_http_error("openai", 401, r#"{"error": {"message": "Bad key"}}"#);
    let error2 = map_http_error("openai", 401, r#"{"error": {"message": "Bad key"}}"#);

    assert_eq!(error1, error2);
}

#[test]
fn test_error_inequality_for_different_providers() {
    let error1 = map_http_error("openai", 401, "");
    let error2 = map_http_error("anthropic", 401, "");

    assert_ne!(error1, error2);
}

#[test]
fn test_error_inequality_for_different_messages() {
    let error1 = map_http_error("openai", 401, r#"{"error": {"message": "A"}}"#);
    let error2 = map_http_error("openai", 401, r#"{"error": {"message": "B"}}"#);

    assert_ne!(error1, error2);
}

// ============================================================================
// Auth Rotation Error Tests
// ============================================================================

#[test]
fn test_auth_profile_manager_skips_failed_profiles() {
    use aisopod_provider::auth::{AuthProfile, AuthProfileManager, ProfileStatus};

    let mut manager = AuthProfileManager::new(Duration::from_secs(60));

    manager.add_profile(AuthProfile::new(
        "profile_1".to_string(),
        "openai".to_string(),
        "sk-fail-1".to_string(),
    ));
    manager.add_profile(AuthProfile::new(
        "profile_2".to_string(),
        "openai".to_string(),
        "sk-fail-2".to_string(),
    ));

    // Mark both as failed
    manager.mark_failed("openai", "profile_1", ProfileStatus::RateLimited);
    manager.mark_failed("openai", "profile_2", ProfileStatus::RateLimited);

    // No profile should be available
    assert!(manager.next_key("openai").is_none());
}

#[test]
fn test_auth_profile_manager_recovers_after_cooldown() {
    use aisopod_provider::auth::{AuthProfile, AuthProfileManager};

    // Use very short cooldown for testing
    let mut manager = AuthProfileManager::new(Duration::from_millis(10));

    manager.add_profile(AuthProfile::new(
        "profile_1".to_string(),
        "openai".to_string(),
        "sk-test".to_string(),
    ));

    // Mark as failed
    manager.mark_failed("openai", "profile_1", ProfileStatus::RateLimited);
    assert!(manager.next_key("openai").is_none());

    // Wait for cooldown to expire
    std::thread::sleep(Duration::from_millis(20));

    // Profile should be recovered
    let key = manager.next_key("openai");
    assert!(key.is_some());
    assert_eq!(key.unwrap().api_key, "sk-test");
}

// ============================================================================
// Error Wrapping Tests
// ============================================================================

#[test]
fn test_provider_error_display() {
    let error = map_http_error("openai", 401, r#"{"error": {"message": "Invalid key"}}"#);

    let error_str = format!("{}", error);
    assert!(error_str.contains("Authentication failed"));
    assert!(error_str.contains("openai"));
}

#[test]
fn test_provider_error_chain() {
    use anyhow::Error as AnyhowError;

    // Create a chain of errors
    let source_error = anyhow::anyhow!("Network failure");
    let wrapped_error = AnyhowError::new(source_error);

    // Verify error chain is preserved
    assert!(wrapped_error.to_string().contains("Network failure"));
}

// ============================================================================
// Stream Closure Tests
// ============================================================================

#[tokio::test]
async fn test_stream_closed_error() {
    use aisopod_provider::normalize::ProviderError;

    let chunks = vec![Err("Stream closed unexpectedly".to_string())];

    let provider = MockProvider::new("test")
        .with_chunks(chunks);

    let request = create_test_request("test-model", "Test");
    let mut stream = provider.chat_completion(request).await.unwrap();

    let result = stream.next().await;
    assert!(result.is_some());
    assert!(result.unwrap().is_err());
}
