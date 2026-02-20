//! Failover-related tests for agent engine.
//!
//! This module tests the model failover system including error classification
//! and failover state management.

use aisopod_agent::failover::{classify_error, FailoverAction, FailoverState, ModelAttempt};
use aisopod_agent::resolution::ModelChain;
use aisopod_provider::normalize::ProviderError;
use std::time::Duration;

#[test]
fn test_failover_action_enum() {
    assert_eq!(
        FailoverAction::RetryWithNextAuth,
        FailoverAction::RetryWithNextAuth
    );
    assert_eq!(
        FailoverAction::WaitAndRetry(Duration::from_secs(5)),
        FailoverAction::WaitAndRetry(Duration::from_secs(5))
    );
    assert_eq!(
        FailoverAction::CompactAndRetry,
        FailoverAction::CompactAndRetry
    );
    assert_eq!(
        FailoverAction::FailoverToNext,
        FailoverAction::FailoverToNext
    );
    assert_eq!(FailoverAction::Abort, FailoverAction::Abort);
}

#[test]
fn test_model_attempt_new() {
    let attempt = ModelAttempt {
        model_id: "test-model".to_string(),
        error: None,
        duration: Duration::from_millis(100),
    };

    assert_eq!(attempt.model_id, "test-model");
    assert!(attempt.error.is_none());
    assert_eq!(attempt.duration, Duration::from_millis(100));
}

#[test]
fn test_model_attempt_with_error() {
    let attempt = ModelAttempt {
        model_id: "test-model".to_string(),
        error: Some("Connection failed".to_string()),
        duration: Duration::from_millis(100),
    };

    assert_eq!(attempt.model_id, "test-model");
    assert_eq!(attempt.error, Some("Connection failed".to_string()));
}

#[test]
fn test_failover_state_new() {
    let model_chain = ModelChain::new("gpt-4");
    let state = FailoverState::new(&model_chain);

    assert_eq!(state.current_model(), "gpt-4");
    assert_eq!(state.total_models(), 1);
    assert!(state.attempted_models.is_empty());
}

#[test]
fn test_failover_state_with_fallbacks() {
    let model_chain = ModelChain::with_fallbacks(
        "gpt-4",
        vec!["gpt-3.5-turbo".to_string(), "claude-3-opus".to_string()],
    );
    let state = FailoverState::new(&model_chain);

    assert_eq!(state.current_model(), "gpt-4");
    assert_eq!(state.total_models(), 3);
}

#[test]
fn test_failover_state_advance() {
    let model_chain = ModelChain::with_fallbacks("gpt-4", vec!["gpt-3.5-turbo".to_string()]);
    let mut state = FailoverState::new(&model_chain);

    assert_eq!(state.current_model(), "gpt-4");
    assert!(state.advance().is_some());
    assert_eq!(state.current_model(), "gpt-3.5-turbo");
    assert!(state.advance().is_none());
}

#[test]
fn test_failover_state_record_attempt() {
    let model_chain = ModelChain::new("gpt-4");
    let mut state = FailoverState::new(&model_chain);

    let error = ProviderError::Unknown {
        provider: "test".to_string(),
        message: "Some error".to_string(),
    };

    state.record_attempt(Some(error), Duration::from_millis(100));

    assert_eq!(state.attempted_models.len(), 1);
    assert_eq!(state.attempted_models[0].model_id, "gpt-4");
    assert!(state.attempted_models[0].error.is_some());
    assert_eq!(
        state.attempted_models[0].duration,
        Duration::from_millis(100)
    );
}

#[test]
fn test_failover_state_can_retry_current_model() {
    let model_chain = ModelChain::new("gpt-4");
    let mut state = FailoverState::new(&model_chain);

    assert!(state.can_retry_current_model());

    // Record max attempts
    for _ in 0..3 {
        state.record_attempt(
            Some(ProviderError::Unknown {
                provider: "test".to_string(),
                message: "Error".to_string(),
            }),
            Duration::from_millis(100),
        );
    }

    // Should not be able to retry after max attempts
    assert!(!state.can_retry_current_model());
}

#[test]
fn test_failover_state_last_attempt() {
    let model_chain = ModelChain::new("gpt-4");
    let mut state = FailoverState::new(&model_chain);

    assert!(state.last_attempt().is_none());

    state.record_attempt(None, Duration::from_millis(100));

    assert!(state.last_attempt().is_some());
    assert_eq!(state.last_attempt().unwrap().model_id, "gpt-4");
}

#[test]
fn test_classify_error_authentication_failed() {
    let error = ProviderError::AuthenticationFailed {
        provider: "test".to_string(),
        message: "Auth failed".to_string(),
    };

    let action = classify_error(&error);
    assert_eq!(action, FailoverAction::RetryWithNextAuth);
}

#[test]
fn test_classify_error_rate_limited_with_retry_after() {
    let error = ProviderError::RateLimited {
        provider: "test".to_string(),
        retry_after: Some(Duration::from_secs(10)),
    };

    let action = classify_error(&error);
    assert_eq!(
        action,
        FailoverAction::WaitAndRetry(Duration::from_secs(10))
    );
}

#[test]
fn test_classify_error_rate_limited_default_retry() {
    let error = ProviderError::RateLimited {
        provider: "test".to_string(),
        retry_after: None,
    };

    let action = classify_error(&error);
    assert_eq!(action, FailoverAction::WaitAndRetry(Duration::from_secs(5)));
}

#[test]
fn test_classify_error_context_length_exceeded() {
    let error = ProviderError::ContextLengthExceeded {
        provider: "test".to_string(),
        max_tokens: 10000,
    };

    let action = classify_error(&error);
    assert_eq!(action, FailoverAction::CompactAndRetry);
}

#[test]
fn test_classify_error_model_not_found() {
    let error = ProviderError::ModelNotFound {
        provider: "test".to_string(),
        model: "test-model".to_string(),
    };

    let action = classify_error(&error);
    assert_eq!(action, FailoverAction::FailoverToNext);
}

#[test]
fn test_classify_error_server_error() {
    let error = ProviderError::ServerError {
        provider: "test".to_string(),
        status: 500,
        message: "Internal server error".to_string(),
    };

    let action = classify_error(&error);
    assert_eq!(action, FailoverAction::FailoverToNext);
}

#[test]
fn test_classify_error_network_error() {
    let error = ProviderError::NetworkError {
        provider: "test".to_string(),
        message: "Connection timeout".to_string(),
    };

    let action = classify_error(&error);
    assert_eq!(action, FailoverAction::RetryWithNextAuth);
}

#[test]
fn test_classify_error_invalid_request() {
    let error = ProviderError::InvalidRequest {
        provider: "test".to_string(),
        message: "Invalid parameters".to_string(),
    };

    let action = classify_error(&error);
    assert_eq!(action, FailoverAction::CompactAndRetry);
}

#[test]
fn test_classify_error_stream_closed() {
    let error = ProviderError::StreamClosed;

    let action = classify_error(&error);
    assert_eq!(action, FailoverAction::FailoverToNext);
}

#[test]
fn test_classify_error_unknown() {
    let error = ProviderError::Unknown {
        provider: "test".to_string(),
        message: "Unknown error".to_string(),
    };

    let action = classify_error(&error);
    assert_eq!(action, FailoverAction::Abort);
}

#[test]
fn test_failover_state_initialization_with_single_model() {
    let model_chain = ModelChain::new("test-model");
    let state = FailoverState::new(&model_chain);

    assert_eq!(state.current_model_index, 0);
    assert_eq!(state.total_models(), 1);
    assert!(state.attempted_models.is_empty());
}

#[test]
fn test_failover_state_max_attempts_default() {
    let model_chain = ModelChain::new("gpt-4");
    let state = FailoverState::new(&model_chain);

    // Check that state has default max_attempts of 3
    // We can't directly access max_attempts, so we test via can_retry_current_model
    let mut state = FailoverState::new(&model_chain);

    // Should be able to retry 3 times
    assert!(state.can_retry_current_model());

    state.record_attempt(None, Duration::from_millis(100));
    assert!(state.can_retry_current_model());

    state.record_attempt(None, Duration::from_millis(100));
    assert!(state.can_retry_current_model());

    state.record_attempt(None, Duration::from_millis(100));
    assert!(!state.can_retry_current_model());
}
