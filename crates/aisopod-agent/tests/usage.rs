//! Usage tracking tests for agent engine.
//!
//! This module tests the UsageTracker which tracks token usage
//! at per-request, per-session, and per-agent levels.

use std::sync::Arc;
use aisopod_agent::usage::UsageTracker;

#[test]
fn test_new_tracker_has_empty_maps() {
    let tracker = UsageTracker::new();
    assert_eq!(tracker.session_keys().len(), 0);
    assert_eq!(tracker.agent_keys().len(), 0);
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
fn test_empty_usage_report_fields() {
    let report = aisopod_agent::types::UsageReport::new(0, 0);
    assert_eq!(report.input_tokens, 0);
    assert_eq!(report.output_tokens, 0);
    assert_eq!(report.total_tokens, 0);
    assert_eq!(report.request_count, 0);
}

#[test]
fn test_usage_report_with_large_numbers() {
    let report = aisopod_agent::types::UsageReport::new(1_000_000, 500_000);
    assert_eq!(report.input_tokens, 1_000_000);
    assert_eq!(report.output_tokens, 500_000);
    assert_eq!(report.total_tokens, 1_500_000);
    assert_eq!(report.request_count, 0);
}

#[test]
fn test_session_keys_returns_all_keys() {
    let tracker = UsageTracker::new();
    
    tracker.record_request("session_a", "agent_1", 10, 5);
    tracker.record_request("session_b", "agent_1", 20, 10);
    tracker.record_request("session_c", "agent_2", 30, 15);
    
    let keys = tracker.session_keys();
    assert_eq!(keys.len(), 3);
    assert!(keys.contains(&"session_a".to_string()));
    assert!(keys.contains(&"session_b".to_string()));
    assert!(keys.contains(&"session_c".to_string()));
}

#[test]
fn test_agent_keys_returns_all_keys() {
    let tracker = UsageTracker::new();
    
    tracker.record_request("session_1", "agent_a", 10, 5);
    tracker.record_request("session_1", "agent_b", 20, 10);
    tracker.record_request("session_2", "agent_c", 30, 15);
    
    let keys = tracker.agent_keys();
    assert_eq!(keys.len(), 3);
    assert!(keys.contains(&"agent_a".to_string()));
    assert!(keys.contains(&"agent_b".to_string()));
    assert!(keys.contains(&"agent_c".to_string()));
}

#[test]
fn test_concurrent_access() {
    let tracker = Arc::new(UsageTracker::new());
    
    // Spawn multiple threads to record requests concurrently
    let mut handles = Vec::new();
    for i in 0..10 {
        let tracker_clone = Arc::clone(&tracker);
        let handle = std::thread::spawn(move || {
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
        let session_usage = tracker.get_session_usage(&format!("session_{}", i)).unwrap();
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
