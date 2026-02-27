//! Retry logic for failed operations.
//!
//! This module provides utilities for implementing retry strategies with
//! exponential backoff, jitter, and circuit breaker patterns to handle
//! transient failures in channel operations.

use std::time::{Duration, Instant};

/// Maximum number of retry attempts allowed.
pub const MAX_RETRIES: u32 = 5;

/// Base delay for exponential backoff (1 second).
pub const BASE_DELAY: Duration = Duration::from_secs(1);

/// Maximum delay between retries (1 minute).
pub const MAX_DELAY: Duration = Duration::from_secs(60);

/// Circuit breaker state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed, requests flow normally
    Closed,
    /// Circuit is open, requests are rejected
    Open,
    /// Circuit is half-open, testing if service recovered
    HalfOpen,
}

/// Retry configuration.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Base delay for exponential backoff
    pub base_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Whether to add jitter to delays
    pub jitter: bool,
    /// Duration before circuit breaker resets
    pub circuit_breaker_timeout: Duration,
    /// Failure threshold to open circuit breaker
    pub circuit_breaker_threshold: u32,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: MAX_RETRIES,
            base_delay: BASE_DELAY,
            max_delay: MAX_DELAY,
            jitter: true,
            circuit_breaker_timeout: Duration::from_secs(30),
            circuit_breaker_threshold: 5,
        }
    }
}

/// Result of a retry operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RetryResult<T> {
    /// Operation succeeded
    Success(T),
    /// Operation failed after all retries
    Failed(TotalRetryError),
    /// Circuit breaker is open
    CircuitOpen,
}

/// Error containing the last failure and retry statistics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TotalRetryError {
    /// Number of retries attempted
    pub attempts: u32,
    /// Last error encountered
    pub last_error: String,
    /// Total time spent retrying
    pub total_duration: Duration,
}

impl TotalRetryError {
    pub fn new(attempts: u32, last_error: impl Into<String>, total_duration: Duration) -> Self {
        Self {
            attempts,
            last_error: last_error.into(),
            total_duration,
        }
    }
}

impl std::fmt::Display for TotalRetryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Retry failed after {} attempts. Last error: {}. Total duration: {:?}",
            self.attempts, self.last_error, self.total_duration
        )
    }
}

impl std::error::Error for TotalRetryError {}

/// Retry state manager.
#[derive(Debug)]
pub struct RetryState {
    config: RetryConfig,
    attempts: u32,
    circuit_state: CircuitState,
    circuit_failures: u32,
    circuit_opened_at: Option<Instant>,
}

impl RetryState {
    /// Create a new retry state with default configuration.
    pub fn new() -> Self {
        Self::with_config(RetryConfig::default())
    }

    /// Create a new retry state with custom configuration.
    pub fn with_config(config: RetryConfig) -> Self {
        Self {
            config,
            attempts: 0,
            circuit_state: CircuitState::Closed,
            circuit_failures: 0,
            circuit_opened_at: None,
        }
    }

    /// Check if retry is allowed based on circuit breaker state.
    pub fn can_retry(&self) -> bool {
        match self.circuit_state {
            CircuitState::Open => {
                // Check if timeout has passed
                if let Some(opened_at) = self.circuit_opened_at {
                    Instant::now().duration_since(opened_at) >= self.config.circuit_breaker_timeout
                } else {
                    false
                }
            }
            CircuitState::Closed | CircuitState::HalfOpen => true,
        }
    }

    /// Check if more retries are available.
    pub fn has_more_retries(&self) -> bool {
        self.attempts < self.config.max_retries
    }

    /// Increment retry count.
    pub fn increment_attempt(&mut self) {
        self.attempts += 1;
    }

    /// Get current attempt count.
    pub fn current_attempt(&self) -> u32 {
        self.attempts
    }

    /// Calculate delay before next retry.
    pub fn calculate_delay(&self) -> Duration {
        let base = self.config.base_delay;
        let exponential = base * 2u32.pow(self.attempts);
        let delay = exponential.min(self.config.max_delay);

        if self.config.jitter {
            // Add up to 20% jitter
            let jitter = (delay.as_millis() as f64 * 0.2) as u64;
            delay + Duration::from_millis(jitter)
        } else {
            delay
        }
    }

    /// Record a successful operation.
    pub fn record_success(&mut self) {
        self.circuit_failures = 0;
        self.circuit_state = CircuitState::Closed;
        self.attempts = 0;
    }

    /// Record a failed operation.
    pub fn record_failure(&mut self) {
        self.circuit_failures += 1;

        if self.circuit_failures >= self.config.circuit_breaker_threshold {
            self.circuit_state = CircuitState::Open;
            self.circuit_opened_at = Some(Instant::now());
        } else if self.circuit_state == CircuitState::HalfOpen {
            self.circuit_state = CircuitState::Open;
            self.circuit_opened_at = Some(Instant::now());
        }
    }

    /// Transition circuit to half-open state for testing.
    pub fn test_half_open(&mut self) {
        if self.circuit_state == CircuitState::Open {
            self.circuit_state = CircuitState::HalfOpen;
        }
    }

    /// Get current circuit state.
    pub fn circuit_state(&self) -> CircuitState {
        self.circuit_state
    }
}

/// Retry executor for async operations.
#[derive(Debug)]
pub struct RetryExecutor<F> {
    state: RetryState,
    operation: F,
}

impl<F, T> RetryExecutor<F>
where
    F: FnMut() -> Result<T, String>,
{
    /// Create a new retry executor.
    pub fn new(operation: F) -> Self {
        Self {
            state: RetryState::new(),
            operation,
        }
    }

    /// Create a new retry executor with custom configuration.
    pub fn with_config(operation: F, config: RetryConfig) -> Self {
        Self {
            state: RetryState::with_config(config),
            operation,
        }
    }

    /// Execute the operation with retry logic.
    pub fn execute(&mut self) -> RetryResult<T> {
        // Check circuit breaker
        if !self.state.can_retry() {
            return RetryResult::CircuitOpen;
        }

        // Check if more retries available
        if !self.state.has_more_retries() {
            return RetryResult::Failed(
                TotalRetryError::new(self.state.current_attempt(), "Max retries exceeded", Duration::ZERO),
            );
        }

        let start = Instant::now();

        loop {
            // Check if more retries available
            if !self.state.has_more_retries() {
                let error = TotalRetryError::new(
                    self.state.current_attempt(),
                    "Max retries exceeded",
                    Instant::now().duration_since(start),
                );
                return RetryResult::Failed(error);
            }

            match (self.operation)() {
                Ok(result) => {
                    self.state.record_success();
                    return RetryResult::Success(result);
                }
                Err(e) => {
                    self.state.increment_attempt();
                    self.state.record_failure();

                    if !self.state.has_more_retries() || !self.state.can_retry() {
                        let error = TotalRetryError::new(
                            self.state.current_attempt(),
                            e,
                            Instant::now().duration_since(start),
                        );
                        return RetryResult::Failed(error);
                    }

                    // Wait before retry
                    let delay = self.state.calculate_delay();
                    std::thread::sleep(delay);
                }
            }
        }
    }

    /// Get the current retry state.
    pub fn state(&self) -> &RetryState {
        &self.state
    }

    /// Get mutable access to the retry state.
    pub fn state_mut(&mut self) -> &mut RetryState {
        &mut self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_state_creation() {
        let state = RetryState::new();
        assert_eq!(state.config.max_retries, MAX_RETRIES);
        assert_eq!(state.config.base_delay, BASE_DELAY);
        assert_eq!(state.attempts, 0);
        assert_eq!(state.circuit_state, CircuitState::Closed);
    }

    #[test]
    fn test_retry_delay_calculation() {
        let mut state = RetryState::with_config(RetryConfig {
            jitter: false,
            ..Default::default()
        });

        // First retry should be base_delay
        assert_eq!(state.calculate_delay(), BASE_DELAY);

        // Second retry should be base_delay * 2
        state.attempts = 1;
        assert_eq!(state.calculate_delay(), BASE_DELAY * 2);

        // Third retry should be base_delay * 4
        state.attempts = 2;
        assert_eq!(state.calculate_delay(), BASE_DELAY * 4);
    }

    #[test]
    fn test_retry_delay_with_jitter() {
        let state = RetryState::new();
        let delay = state.calculate_delay();
        assert!(delay >= BASE_DELAY);
        assert!(delay <= BASE_DELAY * 2); // With jitter
    }

    #[test]
    fn test_circuit_breaker() {
        let mut state = RetryState::with_config(RetryConfig {
            circuit_breaker_threshold: 3,
            ..Default::default()
        });

        // Should start closed
        assert_eq!(state.circuit_state, CircuitState::Closed);

        // Record failures
        state.record_failure();
        state.record_failure();
        state.record_failure();

        // Should be open after threshold
        assert_eq!(state.circuit_state, CircuitState::Open);
        assert!(!state.can_retry());
    }

    #[test]
    fn test_retry_executor_success() {
        let mut executor = RetryExecutor::new(|| Ok::<_, String>("success".to_string()));
        let result = executor.execute();

        match result {
            RetryResult::Success(value) => assert_eq!(value, "success"),
            _ => panic!("Expected success"),
        }
    }

    #[test]
    fn test_retry_executor_failure() {
        let mut executor = RetryExecutor::with_config(
            || Err::<&str, String>("error".to_string()),
            RetryConfig {
                max_retries: 2,
                ..Default::default()
            },
        );

        let result: RetryResult<&str> = executor.execute();

        match result {
            RetryResult::Failed(error) => {
                assert_eq!(error.attempts, 2);
                assert!(error.last_error.contains("error"));
            }
            _ => panic!("Expected failure"),
        }
    }

    #[test]
    fn test_circuit_breaker_half_open() {
        let mut state = RetryState::new();

        // Open the circuit
        state.circuit_state = CircuitState::Open;
        state.circuit_opened_at = Some(Instant::now() - Duration::from_secs(60));

        // Should be able to retry (timeout passed)
        assert!(state.can_retry());

        // Transition to half-open
        state.test_half_open();
        assert_eq!(state.circuit_state, CircuitState::HalfOpen);
    }

    #[test]
    fn test_retry_executor_config() {
        let config = RetryConfig {
            max_retries: 3,
            base_delay: Duration::from_millis(10),
            jitter: false,
            ..Default::default()
        };

        let mut executor = RetryExecutor::with_config(|| Ok::<_, String>("ok".to_string()), config);
        let result = executor.execute();

        match result {
            RetryResult::Success(_) => {}
            _ => panic!("Expected success"),
        }
    }

    #[test]
    fn test_total_retry_error_display() {
        let error = TotalRetryError::new(3, "Connection failed", Duration::from_secs(5));

        let display = format!("{}", error);
        assert!(display.contains("3"));
        assert!(display.contains("Connection failed"));
        assert!(display.contains("5s"));
    }
}
