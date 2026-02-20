//! Model failover system for aisopod-agent.
//!
//! This module provides intelligent failover capabilities for model execution,
//! including automatic retry, model switching, and error classification.

use std::time::{Duration, Instant};

use crate::types::AgentEvent;
use crate::resolution::ModelChain;

use aisopod_provider::normalize::ProviderError;

/// Action to take when an error occurs during model execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailoverAction {
    /// Retry with the next authentication (different credential)
    RetryWithNextAuth,
    /// Wait for the specified duration then retry with the same model
    WaitAndRetry(Duration),
    /// Compact the message history and retry with the same model
    CompactAndRetry,
    /// Failover to the next model in the chain
    FailoverToNext,
    /// Abort the operation
    Abort,
}

/// Represents an attempt to execute a model.
#[derive(Debug, Clone)]
pub struct ModelAttempt {
    /// The model ID that was attempted
    pub model_id: String,
    /// The error that occurred (if any) - stored as a string since ProviderError is not Clone
    pub error: Option<String>,
    /// How long the attempt took
    pub duration: Duration,
}

/// State management for model failover.
#[derive(Debug, Clone)]
pub struct FailoverState {
    /// List of models that have been attempted
    pub attempted_models: Vec<ModelAttempt>,
    /// Index into the model chain
    pub current_model_index: usize,
    /// Maximum number of attempts per model
    pub max_attempts: usize,
    /// The model chain to iterate through
    model_chain: ModelChain,
    /// All models in the chain (primary + fallbacks)
    all_models: Vec<String>,
}

impl FailoverState {
    /// Creates a new FailoverState from a model chain.
    pub fn new(model_chain: &ModelChain) -> Self {
        Self {
            attempted_models: Vec::new(),
            current_model_index: 0,
            max_attempts: 3, // Default max attempts per model
            model_chain: model_chain.clone(),
            all_models: model_chain.all_models(),
        }
    }

    /// Gets the current model ID being attempted.
    pub fn current_model(&self) -> &str {
        if self.current_model_index < self.all_models.len() {
            &self.all_models[self.current_model_index]
        } else {
            // Fallback - should not happen in normal operation
            self.model_chain.primary()
        }
    }

    /// Advances to the next model in the chain.
    /// Returns Some(model_id) if there are more models, None if exhausted.
    pub fn advance(&mut self) -> Option<&str> {
        self.current_model_index += 1;
        if self.current_model_index < self.all_models.len() {
            Some(self.current_model())
        } else {
            None
        }
    }

    /// Returns the total number of models available.
    pub fn total_models(&self) -> usize {
        self.model_chain.all_models().len()
    }

    /// Records a model attempt with an optional error.
    pub fn record_attempt(&mut self, error: Option<ProviderError>, duration: Duration) {
        let model_id = self.current_model().to_string();
        let error_msg = error.map(|e| e.to_string());
        self.attempted_models.push(ModelAttempt {
            model_id,
            error: error_msg,
            duration,
        });
    }

    /// Returns whether we can still retry with the current model.
    pub fn can_retry_current_model(&self) -> bool {
        let model_id = self.current_model();
        let attempts_for_current = self
            .attempted_models
            .iter()
            .filter(|m| m.model_id == model_id)
            .count();
        attempts_for_current < self.max_attempts
    }

    /// Gets the last recorded attempt.
    pub fn last_attempt(&self) -> Option<&ModelAttempt> {
        self.attempted_models.last()
    }
}

/// Classifies an error and determines the appropriate failover action.
///
/// This function analyzes the type of error and determines what action
/// should be taken:
/// - Authentication errors → RetryWithNextAuth or FailoverToNext
/// - Rate limit errors → WaitAndRetry or FailoverToNext
/// - Context length errors → CompactAndRetry
/// - Timeout errors → RetryWithNextAuth or FailoverToNext
/// - Server errors → FailoverToNext
/// - Other errors → Abort
///
/// # Arguments
///
/// * `error` - The provider error to classify
///
/// # Returns
///
/// The FailoverAction to take for this error.
pub fn classify_error(error: &ProviderError) -> FailoverAction {
    match error {
        ProviderError::AuthenticationFailed { .. } => {
            // Authentication failures should try different credentials first,
            // then failover to next model if that doesn't work
            FailoverAction::RetryWithNextAuth
        }
        ProviderError::RateLimited { retry_after, .. } => {
            // Rate limits should wait and retry with the same model first
            // If there's a retry_after duration, use it
            match retry_after {
                Some(duration) => FailoverAction::WaitAndRetry(*duration),
                None => FailoverAction::WaitAndRetry(Duration::from_secs(5)), // Default wait
            }
        }
        ProviderError::ContextLengthExceeded { .. } => {
            // Context length exceeded - compact messages and retry
            FailoverAction::CompactAndRetry
        }
        ProviderError::ModelNotFound { .. } => {
            // Model not found - failover to next model
            FailoverAction::FailoverToNext
        }
        ProviderError::ServerError { .. } => {
            // Server errors - try next model
            FailoverAction::FailoverToNext
        }
        ProviderError::NetworkError { .. } => {
            // Network errors - retry with next auth, then failover
            FailoverAction::RetryWithNextAuth
        }
        ProviderError::InvalidRequest { .. } => {
            // Invalid request - usually means the request itself is problematic
            // Try compacting and retrying
            FailoverAction::CompactAndRetry
        }
        ProviderError::StreamClosed => {
            // Stream closed unexpectedly - try next model
            FailoverAction::FailoverToNext
        }
        ProviderError::Unknown { .. } => {
            // Unknown errors - abort
            FailoverAction::Abort
        }
        // Handle any future variants that may be added to ProviderError
        _ => FailoverAction::Abort,
    }
}

/// Executes a model call with intelligent failover.
///
/// This function implements the failover logic:
/// 1. Calls the model with the provided function
/// 2. If an error occurs, classifies it using classify_error
/// 3. Takes appropriate action based on the classification:
///    - RetryWithNextAuth: Switches credentials and retries
///    - WaitAndRetry: Waits for the specified duration, then retries
///    - CompactAndRetry: Returns error for caller to compact messages
///    - FailoverToNext: Switches to next model in chain
///    - Abort: Returns error
/// 4. Emits AgentEvent::ModelSwitch when switching models
/// 5. Records all attempts in the FailoverState
///
/// # Type Parameters
///
/// * `F` - The closure type that performs the model call
/// * `R` - The return type of the closure
///
/// # Arguments
///
/// * `state` - The FailoverState tracking attempts
/// * `emit_event` - A closure that emits AgentEvents
/// * `operation` - The closure that performs the model call
///
/// # Returns
///
/// Returns Ok with the result if successful, or Err with an error message
/// if all models are exhausted or failover should abort.
pub async fn execute_with_failover<F, Fut, R>(
    state: &mut FailoverState,
    emit_event: &mut dyn FnMut(AgentEvent),
    mut operation: F,
) -> Result<R, String>
where
    F: FnMut(&str) -> Fut,
    Fut: std::future::Future<Output = Result<R, ProviderError>>,
{
    let mut current_attempts = 0;
    let max_attempts = state.max_attempts;

    while state.current_model_index < state.total_models() {
        let model_id = state.current_model().to_string();
        let start_time = Instant::now();

        match operation(&model_id).await {
            Ok(result) => {
                let duration = start_time.elapsed();
                state.record_attempt(None, duration);
                return Ok(result);
            }
            Err(error) => {
                let duration = start_time.elapsed();
                state.record_attempt(Some(error.clone()), duration);

                let action = classify_error(&error);

                match action {
                    FailoverAction::RetryWithNextAuth => {
                        // For now, we just failover since we don't have
                        // multiple auth credentials to try
                        // In a full implementation, this would switch credentials
                        if state.advance().is_some() {
                            emit_event(AgentEvent::ModelSwitch {
                                from: model_id.clone(),
                                to: state.current_model().to_string(),
                                reason: "retry with next auth".to_string(),
                            });
                            current_attempts = 0;
                        } else {
                            // No more models to try
                            return Err(format!(
                                "All models exhausted. Last error: {}",
                                error
                            ));
                        }
                    }
                    FailoverAction::WaitAndRetry(duration) => {
                        // Wait before retrying
                        tokio::time::sleep(duration).await;

                        if state.can_retry_current_model() {
                            current_attempts += 1;
                            continue;
                        } else {
                            state.advance();
                            emit_event(AgentEvent::ModelSwitch {
                                from: model_id.clone(),
                                to: state.current_model().to_string(),
                                reason: "wait and retry".to_string(),
                            });
                            current_attempts = 0;
                        }
                    }
                    FailoverAction::CompactAndRetry => {
                        // Signal caller to compact messages and retry
                        // We return the error so the caller can decide how to handle it
                        return Err(format!(
                            "Context length exceeded - compact messages and retry: {}",
                            error
                        ));
                    }
                    FailoverAction::FailoverToNext => {
                        if state.advance().is_some() {
                            emit_event(AgentEvent::ModelSwitch {
                                from: model_id.clone(),
                                to: state.current_model().to_string(),
                                reason: "failover to next model".to_string(),
                            });
                            current_attempts = 0;
                        } else {
                            // No more models to try
                            return Err(format!(
                                "All models exhausted. Last error: {}",
                                error
                            ));
                        }
                    }
                    FailoverAction::Abort => {
                        return Err(format!("Aborting due to error: {}", error));
                    }
                }
            }
        }
    }

    Err(format!(
        "All models exhausted after {} attempts. Last model: {}",
        state.total_models(),
        state.current_model()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_failover_state_new() {
        let chain = ModelChain::new("gpt-4");
        let state = FailoverState::new(&chain);

        assert_eq!(state.current_model(), "gpt-4");
        assert_eq!(state.total_models(), 1);
        assert!(state.attempted_models.is_empty());
    }

    #[test]
    fn test_failover_state_with_fallbacks() {
        let chain = ModelChain::with_fallbacks(
            "gpt-4",
            vec!["gpt-3.5-turbo".to_string(), "claude-3-opus".to_string()],
        );
        let state = FailoverState::new(&chain);

        assert_eq!(state.current_model(), "gpt-4");
        assert_eq!(state.total_models(), 3);
    }

    #[test]
    fn test_failover_state_advance() {
        let chain = ModelChain::with_fallbacks(
            "gpt-4",
            vec!["gpt-3.5-turbo".to_string(), "claude-3-opus".to_string()],
        );
        let mut state = FailoverState::new(&chain);

        assert_eq!(state.current_model(), "gpt-4");
        assert_eq!(state.advance(), Some("gpt-3.5-turbo".to_string()));
        assert_eq!(state.current_model(), "gpt-3.5-turbo");
        assert_eq!(state.advance(), Some("claude-3-opus".to_string()));
        assert_eq!(state.current_model(), "claude-3-opus");
        assert_eq!(state.advance(), None);
    }

    #[test]
    fn test_failover_state_record_attempt() {
        let chain = ModelChain::new("gpt-4");
        let mut state = FailoverState::new(&chain);

        let duration = Duration::from_millis(100);
        let error = ProviderError::RateLimited {
            provider: "openai".to_string(),
            retry_after: None,
        };

        state.record_attempt(Some(error.clone()), duration);

        assert_eq!(state.attempted_models.len(), 1);
        assert_eq!(state.attempted_models[0].model_id, "gpt-4");
        assert!(state.attempted_models[0].error.is_some());
        assert_eq!(state.attempted_models[0].duration, duration);
    }

    #[test]
    fn test_classify_error_auth_failed() {
        let error = ProviderError::AuthenticationFailed {
            provider: "openai".to_string(),
            message: "Invalid API key".to_string(),
        };
        let action = classify_error(&error);
        assert_eq!(action, FailoverAction::RetryWithNextAuth);
    }

    #[test]
    fn test_classify_error_rate_limited() {
        let error = ProviderError::RateLimited {
            provider: "openai".to_string(),
            retry_after: Some(Duration::from_secs(10)),
        };
        let action = classify_error(&error);
        assert_eq!(action, FailoverAction::WaitAndRetry(Duration::from_secs(10)));
    }

    #[test]
    fn test_classify_error_context_length_exceeded() {
        let error = ProviderError::ContextLengthExceeded {
            provider: "openai".to_string(),
            max_tokens: 4096,
        };
        let action = classify_error(&error);
        assert_eq!(action, FailoverAction::CompactAndRetry);
    }

    #[test]
    fn test_classify_error_model_not_found() {
        let error = ProviderError::ModelNotFound {
            provider: "openai".to_string(),
            model: "unknown-model".to_string(),
        };
        let action = classify_error(&error);
        assert_eq!(action, FailoverAction::FailoverToNext);
    }

    #[test]
    fn test_classify_error_server_error() {
        let error = ProviderError::ServerError {
            provider: "openai".to_string(),
            status: 500,
            message: "Internal server error".to_string(),
        };
        let action = classify_error(&error);
        assert_eq!(action, FailoverAction::FailoverToNext);
    }

    #[test]
    fn test_classify_error_unknown() {
        let error = ProviderError::Unknown {
            provider: "openai".to_string(),
            message: "Unknown error".to_string(),
        };
        let action = classify_error(&error);
        assert_eq!(action, FailoverAction::Abort);
    }

    #[tokio::test]
    async fn test_execute_with_failover_success_first_attempt() {
        let chain = ModelChain::new("gpt-4");
        let mut state = FailoverState::new(&chain);
        let mut events: Vec<AgentEvent> = Vec::new();

        let result = execute_with_failover(
            &mut state,
            &mut |event| events.push(event),
            |model_id| {
                let model_id_clone = model_id.to_string();
                async move {
                    assert_eq!(model_id_clone, "gpt-4");
                    Ok("success".to_string())
                }
            },
        )
        .await;

        assert_eq!(result.unwrap(), "success");
        assert_eq!(state.attempted_models.len(), 1);
        assert!(state.attempted_models[0].error.is_none());
        assert!(events.is_empty()); // No ModelSwitch events
    }

    #[tokio::test]
    async fn test_execute_with_failover_failover_on_auth_error() {
        let chain = ModelChain::with_fallbacks(
            "gpt-4",
            vec!["gpt-3.5-turbo".to_string(), "claude-3-opus".to_string()],
        );
        let mut state = FailoverState::new(&chain);
        let mut events: Vec<AgentEvent> = Vec::new();

        // First model fails with auth error
        let result = execute_with_failover(
            &mut state,
            &mut |event| events.push(event),
            |model_id| {
                let model_id_clone = model_id.to_string();
                async move {
                    if model_id_clone == "gpt-4" {
                        Err(ProviderError::AuthenticationFailed {
                            provider: "openai".to_string(),
                            message: "Invalid API key".to_string(),
                        })
                    } else {
                        Ok("success".to_string())
                    }
                }
            },
        )
        .await;

        assert_eq!(result.unwrap(), "success");
        assert_eq!(state.attempted_models.len(), 2);
        assert_eq!(state.current_model(), "gpt-3.5-turbo");

        // Check that ModelSwitch event was emitted
        let model_switch_events: Vec<&AgentEvent> = events
            .iter()
            .filter(|e| matches!(e, AgentEvent::ModelSwitch { .. }))
            .collect();
        assert_eq!(model_switch_events.len(), 1);
    }

    #[tokio::test]
    async fn test_execute_with_failover_all_models_exhausted() {
        let chain = ModelChain::with_fallbacks(
            "gpt-4",
            vec!["gpt-3.5-turbo".to_string()],
        );
        let mut state = FailoverState::new(&chain);
        let mut events: Vec<AgentEvent> = Vec::new();

        let result: Result<String, String> = execute_with_failover(
            &mut state,
            &mut |event| events.push(event),
            |_model_id| {
                async move {
                    Err(ProviderError::AuthenticationFailed {
                        provider: "openai".to_string(),
                        message: "Invalid API key".to_string(),
                    })
                }
            },
        )
        .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("All models exhausted"));
        assert_eq!(state.attempted_models.len(), 2); // Both models attempted
    }
}
