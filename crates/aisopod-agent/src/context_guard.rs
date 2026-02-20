//! Context window guard for monitoring token usage against configurable thresholds.
//!
//! This module provides a `ContextWindowGuard` struct that monitors message
//! token usage against configurable thresholds and determines when compaction
//! strategies should be applied.

use crate::compaction::CompactionSeverity;

/// Context window guard that monitors token usage against configurable thresholds.
///
/// This struct tracks context window usage and determines when compaction
/// is needed based on warn and hard limit thresholds.
#[derive(Debug, Clone)]
pub struct ContextWindowGuard {
    /// Warning threshold as a percentage (e.g., 0.8 for 80%)
    pub warn_threshold: f64,
    /// Hard limit for maximum tokens
    pub hard_limit: usize,
    /// Minimum tokens reserved for response
    pub min_available: usize,
}

impl ContextWindowGuard {
    /// Creates a new ContextWindowGuard with the given thresholds.
    ///
    /// # Arguments
    ///
    /// * `warn_threshold` - Warning threshold as a percentage (0.0 to 1.0)
    /// * `hard_limit` - Maximum token limit
    /// * `min_available` - Minimum tokens reserved for response
    pub fn new(warn_threshold: f64, hard_limit: usize, min_available: usize) -> Self {
        Self {
            warn_threshold,
            hard_limit,
            min_available,
        }
    }

    /// Creates a ContextWindowGuard from agent configuration.
    ///
    /// This method extracts context window settings from the configuration
    /// and creates a guard instance with appropriate thresholds.
    ///
    /// # Arguments
    ///
    /// * `config` - The agent configuration
    ///
    /// # Returns
    ///
    /// A new ContextWindowGuard instance
    pub fn from_config(config: &aisopod_config::types::Agent) -> Self {
        // Use reasonable defaults based on common model context windows
        // These values can be refined based on specific model capabilities
        let default_context_window = 128_000; // Common modern context window

        Self {
            warn_threshold: 0.8, // Warn at 80%
            hard_limit: default_context_window,
            min_available: 4096, // Reserve 4k tokens for response
        }
    }

    /// Creates a ContextWindowGuard from session compaction configuration.
    ///
    /// # Arguments
    ///
    /// * `compaction_config` - The session compaction configuration
    ///
    /// # Returns
    ///
    /// A new ContextWindowGuard instance
    pub fn from_compaction_config(
        compaction_config: &aisopod_config::types::CompactionConfig,
    ) -> Self {
        Self {
            warn_threshold: 0.8,
            hard_limit: compaction_config.min_messages.saturating_mul(100), // Estimate 100 tokens per message
            min_available: 1024,
        }
    }

    /// Determines if compaction is needed based on current token usage.
    ///
    /// # Arguments
    ///
    /// * `current_tokens` - The current token count
    ///
    /// # Returns
    ///
    /// True if compaction is needed (warn threshold exceeded)
    pub fn needs_compaction(&self, current_tokens: usize) -> bool {
        let warn_limit = (self.warn_threshold * self.hard_limit as f64) as usize;
        current_tokens >= warn_limit
    }

    /// Determines the severity of context window usage.
    ///
    /// # Arguments
    ///
    /// * `current_tokens` - The current token count
    ///
    /// # Returns
    ///
    /// The compaction severity: None, Warn, or Critical
    pub fn severity(&self, current_tokens: usize) -> CompactionSeverity {
        if current_tokens >= self.hard_limit {
            CompactionSeverity::Critical
        } else if current_tokens >= (self.warn_threshold * self.hard_limit as f64) as usize {
            CompactionSeverity::Warn
        } else {
            CompactionSeverity::None
        }
    }

    /// Returns the available tokens for the current request.
    ///
    /// This accounts for the reserved minimum tokens for the response.
    ///
    /// # Arguments
    ///
    /// * `current_tokens` - The current token count
    pub fn available_tokens(&self, current_tokens: usize) -> usize {
        self.hard_limit
            .saturating_sub(current_tokens)
            .saturating_sub(self.min_available)
    }

    /// Returns true if the token count is within safe limits.
    ///
    /// # Arguments
    ///
    /// * `current_tokens` - The current token count
    pub fn is_safe(&self, current_tokens: usize) -> bool {
        self.severity(current_tokens) == CompactionSeverity::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_window_guard_creation() {
        let guard = ContextWindowGuard::new(0.8, 10000, 1000);
        assert_eq!(guard.warn_threshold, 0.8);
        assert_eq!(guard.hard_limit, 10000);
        assert_eq!(guard.min_available, 1000);
    }

    #[test]
    fn test_from_config_default_values() {
        let config = aisopod_config::types::Agent::default();
        let guard = ContextWindowGuard::from_config(&config);

        assert_eq!(guard.warn_threshold, 0.8);
        assert_eq!(guard.hard_limit, 128_000);
        assert_eq!(guard.min_available, 4096);
    }

    #[test]
    fn test_severity_none() {
        let guard = ContextWindowGuard::new(0.8, 10000, 1000);

        // Below warning threshold (8000)
        assert_eq!(guard.severity(5000), CompactionSeverity::None);
    }

    #[test]
    fn test_severity_warn() {
        let guard = ContextWindowGuard::new(0.8, 10000, 1000);

        // At warning threshold
        assert_eq!(guard.severity(8000), CompactionSeverity::Warn);

        // Above warning threshold but below hard limit
        assert_eq!(guard.severity(9000), CompactionSeverity::Warn);
    }

    #[test]
    fn test_severity_critical() {
        let guard = ContextWindowGuard::new(0.8, 10000, 1000);

        // At hard limit
        assert_eq!(guard.severity(10000), CompactionSeverity::Critical);

        // Above hard limit
        assert_eq!(guard.severity(15000), CompactionSeverity::Critical);
    }

    #[test]
    fn test_needs_compaction() {
        let guard = ContextWindowGuard::new(0.8, 10000, 1000);

        // Below warning threshold - no compaction needed
        assert!(!guard.needs_compaction(5000));

        // At warning threshold - compaction needed
        assert!(guard.needs_compaction(8000));

        // Above warning threshold - compaction needed
        assert!(guard.needs_compaction(9000));
    }

    #[test]
    fn test_available_tokens() {
        let guard = ContextWindowGuard::new(0.8, 10000, 1000);

        // With 5000 tokens used: 10000 - 5000 - 1000 = 4000 available
        assert_eq!(guard.available_tokens(5000), 4000);

        // With 9000 tokens used: 10000 - 9000 - 1000 = 0 available
        assert_eq!(guard.available_tokens(9000), 0);

        // With over limit - saturates to 0
        assert_eq!(guard.available_tokens(12000), 0);
    }

    #[test]
    fn test_is_safe() {
        let guard = ContextWindowGuard::new(0.8, 10000, 1000);

        assert!(guard.is_safe(5000));
        assert!(!guard.is_safe(9000));
        assert!(!guard.is_safe(10000));
    }

    #[test]
    fn test_from_compaction_config() {
        let compaction_config = aisopod_config::types::CompactionConfig {
            enabled: true,
            min_messages: 100,
            interval: 3600,
        };

        let guard = ContextWindowGuard::from_compaction_config(&compaction_config);

        assert_eq!(guard.warn_threshold, 0.8);
        // 100 messages * 100 tokens = 10000 hard limit
        assert_eq!(guard.hard_limit, 10000);
        assert_eq!(guard.min_available, 1024);
    }
}
