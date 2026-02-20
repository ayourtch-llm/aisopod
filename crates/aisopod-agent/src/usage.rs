//! Usage tracking for agent execution.
//!
//! This module provides the `UsageTracker` struct which tracks token usage
//! at per-request, per-session, and per-agent levels.

use dashmap::DashMap;
use serde::{Deserialize, Serialize};

use crate::types::UsageReport;

/// A tracker for token usage across sessions and agents.
///
/// `UsageTracker` maintains separate usage reports for each session and agent,
/// allowing for aggregation and querying of usage statistics.
///
/// # Example
///
/// ```ignore
/// use aisopod_agent::usage::UsageTracker;
///
/// let tracker = UsageTracker::new();
///
/// // Record a request
/// tracker.record_request("session_1", "agent_1", 100, 50);
///
/// // Get session usage
/// let session_usage = tracker.get_session_usage("session_1");
/// assert!(session_usage.is_some());
/// assert_eq!(session_usage.unwrap().input_tokens, 100);
///
/// // Get agent usage
/// let agent_usage = tracker.get_agent_usage("agent_1");
/// assert!(agent_usage.is_some());
/// assert_eq!(agent_usage.unwrap().input_tokens, 100);
/// ```
#[derive(Debug, Default)]
pub struct UsageTracker {
    /// Per-session usage, keyed by session_key
    session_usage: DashMap<String, UsageReport>,
    /// Per-agent usage, keyed by agent_id
    agent_usage: DashMap<String, UsageReport>,
}

impl UsageTracker {
    /// Creates a new `UsageTracker` with empty usage maps.
    pub fn new() -> Self {
        Self {
            session_usage: DashMap::new(),
            agent_usage: DashMap::new(),
        }
    }

    /// Records a request with the given token counts.
    ///
    /// This method adds the token counts to both the session and agent
    /// usage reports, incrementing the request count for both.
    ///
    /// # Arguments
    ///
    /// * `session_key` - The session key identifying the conversation session.
    /// * `agent_id` - The agent ID for the request.
    /// * `input_tokens` - The number of input tokens used.
    /// * `output_tokens` - The number of output tokens used.
    pub fn record_request(
        &self,
        session_key: &str,
        agent_id: &str,
        input_tokens: u64,
        output_tokens: u64,
    ) {
        // Update session usage
        {
            let mut session_entry = self
                .session_usage
                .entry(session_key.to_string())
                .or_insert(UsageReport::default());
            session_entry.add(input_tokens, output_tokens);
        }

        // Update agent usage
        {
            let mut agent_entry = self
                .agent_usage
                .entry(agent_id.to_string())
                .or_insert(UsageReport::default());
            agent_entry.add(input_tokens, output_tokens);
        }
    }

    /// Gets the cumulative usage report for a session.
    ///
    /// # Arguments
    ///
    /// * `session_key` - The session key to look up.
    ///
    /// # Returns
    ///
    /// Returns `Some(UsageReport)` if usage exists for the session,
    /// or `None` if the session has no recorded usage.
    pub fn get_session_usage(&self, session_key: &str) -> Option<UsageReport> {
        self.session_usage
            .get(session_key)
            .map(|report| report.clone())
    }

    /// Gets the cumulative usage report for an agent.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The agent ID to look up.
    ///
    /// # Returns
    ///
    /// Returns `Some(UsageReport)` if usage exists for the agent,
    /// or `None` if the agent has no recorded usage.
    pub fn get_agent_usage(&self, agent_id: &str) -> Option<UsageReport> {
        self.agent_usage.get(agent_id).map(|report| report.clone())
    }

    /// Resets usage for a session.
    ///
    /// This clears the session usage but does not affect agent usage.
    /// This is useful when a session ends and a new session begins
    /// with the same session key.
    ///
    /// # Arguments
    ///
    /// * `session_key` - The session key to reset.
    pub fn reset_session(&self, session_key: &str) {
        self.session_usage.remove(session_key);
    }

    /// Gets all session keys with recorded usage.
    ///
    /// # Returns
    ///
    /// Returns a Vec of session keys.
    pub fn session_keys(&self) -> Vec<String> {
        self.session_usage
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }

    /// Gets all agent IDs with recorded usage.
    ///
    /// # Returns
    ///
    /// Returns a Vec of agent IDs.
    pub fn agent_keys(&self) -> Vec<String> {
        self.agent_usage
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    #[test]
    fn test_new_tracker_has_empty_maps() {
        let tracker = UsageTracker::new();
        assert!(tracker.session_usage.is_empty());
        assert!(tracker.agent_usage.is_empty());
    }

    #[test]
    fn test_record_single_request() {
        let tracker = UsageTracker::new();
        tracker.record_request("session_1", "agent_1", 100, 50);

        // Check session usage
        let session_usage = tracker.get_session_usage("session_1").unwrap();
        assert_eq!(session_usage.input_tokens, 100);
        assert_eq!(session_usage.output_tokens, 50);
        assert_eq!(session_usage.total_tokens, 150);
        assert_eq!(session_usage.request_count, 1);

        // Check agent usage
        let agent_usage = tracker.get_agent_usage("agent_1").unwrap();
        assert_eq!(agent_usage.input_tokens, 100);
        assert_eq!(agent_usage.output_tokens, 50);
        assert_eq!(agent_usage.total_tokens, 150);
        assert_eq!(agent_usage.request_count, 1);
    }

    #[test]
    fn test_accumulation_across_requests() {
        let tracker = UsageTracker::new();

        // First request
        tracker.record_request("session_1", "agent_1", 100, 50);

        // Second request
        tracker.record_request("session_1", "agent_1", 200, 100);

        // Third request
        tracker.record_request("session_1", "agent_1", 50, 25);

        // Check session usage - should be cumulative
        let session_usage = tracker.get_session_usage("session_1").unwrap();
        assert_eq!(session_usage.input_tokens, 350); // 100 + 200 + 50
        assert_eq!(session_usage.output_tokens, 175); // 50 + 100 + 25
        assert_eq!(session_usage.total_tokens, 525);
        assert_eq!(session_usage.request_count, 3);

        // Check agent usage - should be cumulative
        let agent_usage = tracker.get_agent_usage("agent_1").unwrap();
        assert_eq!(agent_usage.input_tokens, 350);
        assert_eq!(agent_usage.output_tokens, 175);
        assert_eq!(agent_usage.total_tokens, 525);
        assert_eq!(agent_usage.request_count, 3);
    }

    #[test]
    fn test_per_agent_aggregation() {
        let tracker = UsageTracker::new();

        // Session 1, Agent 1
        tracker.record_request("session_1", "agent_1", 100, 50);

        // Session 2, Agent 1 (different session, same agent)
        tracker.record_request("session_2", "agent_1", 200, 100);

        // Session 1, Agent 2 (same session, different agent)
        tracker.record_request("session_1", "agent_2", 50, 25);

        // Check agent 1 usage - should aggregate across sessions
        let agent_usage = tracker.get_agent_usage("agent_1").unwrap();
        assert_eq!(agent_usage.input_tokens, 300); // 100 + 200
        assert_eq!(agent_usage.output_tokens, 150);
        assert_eq!(agent_usage.total_tokens, 450);
        assert_eq!(agent_usage.request_count, 2);

        // Check agent 2 usage
        let agent_usage = tracker.get_agent_usage("agent_2").unwrap();
        assert_eq!(agent_usage.input_tokens, 50);
        assert_eq!(agent_usage.output_tokens, 25);
        assert_eq!(agent_usage.total_tokens, 75);
        assert_eq!(agent_usage.request_count, 1);

        // Check session 1 usage - should aggregate across agents
        let session_usage = tracker.get_session_usage("session_1").unwrap();
        assert_eq!(session_usage.input_tokens, 150); // 100 + 50
        assert_eq!(session_usage.output_tokens, 75);
        assert_eq!(session_usage.total_tokens, 225);
        assert_eq!(session_usage.request_count, 2);
    }

    #[test]
    fn test_reset_session() {
        let tracker = UsageTracker::new();

        // Record some requests
        tracker.record_request("session_1", "agent_1", 100, 50);
        tracker.record_request("session_1", "agent_1", 200, 100);

        // Reset the session
        tracker.reset_session("session_1");

        // Session usage should be cleared
        assert!(tracker.get_session_usage("session_1").is_none());

        // Agent usage should still exist (reset only affects session)
        let agent_usage = tracker.get_agent_usage("agent_1").unwrap();
        assert_eq!(agent_usage.input_tokens, 300);
        assert_eq!(agent_usage.output_tokens, 150);
        assert_eq!(agent_usage.request_count, 2);
    }

    #[test]
    fn test_nonexistent_session() {
        let tracker = UsageTracker::new();

        assert!(tracker.get_session_usage("nonexistent").is_none());
        assert!(tracker.get_agent_usage("nonexistent").is_none());
    }

    #[test]
    fn test_concurrent_access() {
        use std::thread;

        let tracker = Arc::new(UsageTracker::new());

        // Spawn multiple threads to record requests concurrently
        let mut handles = Vec::new();
        for i in 0..10 {
            let tracker_clone = Arc::clone(&tracker);
            let handle = thread::spawn(move || {
                for j in 0..10 {
                    let session_key = format!("session_{}", i);
                    let agent_id = format!("agent_{}", j % 3);
                    tracker_clone.record_request(&session_key, &agent_id, 10, 5);
                }
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify usage - each session has 10 requests
        for i in 0..10 {
            let session_usage = tracker
                .get_session_usage(&format!("session_{}", i))
                .unwrap();
            assert_eq!(session_usage.input_tokens, 100); // 10 requests * 10 tokens
            assert_eq!(session_usage.output_tokens, 50);
            assert_eq!(session_usage.total_tokens, 150);
            assert_eq!(session_usage.request_count, 10);
        }

        // Verify agent 0 usage (should have received requests from all sessions)
        // For each session: j % 3 = 0 when j = 0, 3, 6, 9 (4 requests)
        // 4 requests * 10 sessions * 10 tokens = 400 input tokens
        let agent_usage = tracker.get_agent_usage("agent_0").unwrap();
        assert_eq!(agent_usage.input_tokens, 400); // 4 requests per session * 10 sessions * 10 tokens
        assert_eq!(agent_usage.output_tokens, 200);
        assert_eq!(agent_usage.total_tokens, 600);
        assert_eq!(agent_usage.request_count, 40);
    }

    #[test]
    fn test_empty_usage_report_fields() {
        let report = UsageReport::new(0, 0);
        assert_eq!(report.input_tokens, 0);
        assert_eq!(report.output_tokens, 0);
        assert_eq!(report.total_tokens, 0);
        assert_eq!(report.request_count, 0);
    }

    #[test]
    fn test_usage_report_with_large_numbers() {
        let report = UsageReport::new(1_000_000, 500_000);
        assert_eq!(report.input_tokens, 1_000_000);
        assert_eq!(report.output_tokens, 500_000);
        assert_eq!(report.total_tokens, 1_500_000);
        assert_eq!(report.request_count, 0);
    }
}
