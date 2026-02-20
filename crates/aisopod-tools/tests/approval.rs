//! Approval workflow tests

use std::sync::Arc;
use std::time::Duration;

use aisopod_tools::{
    ApprovalError, ApprovalHandler, ApprovalRequest, ApprovalResponse, ApprovalStateTracker,
    ApprovalSummary, NoOpApprovalHandler, RiskLevel, is_auto_approved,
};
use serde_json::json;

// Mock approval handler for testing
struct MockApprovalHandler {
    response: std::sync::Arc<std::sync::Mutex<Option<ApprovalResponse>>>,
    auto_approve: std::sync::Arc<std::sync::Mutex<bool>>,
}

impl MockApprovalHandler {
    fn new() -> Self {
        Self {
            response: std::sync::Arc::new(std::sync::Mutex::new(None)),
            auto_approve: std::sync::Arc::new(std::sync::Mutex::new(false)),
        }
    }

    fn set_response(&self, response: ApprovalResponse) {
        *self.response.lock().unwrap() = Some(response);
    }

    fn set_auto_approve(&self, auto: bool) {
        *self.auto_approve.lock().unwrap() = auto;
    }

    fn get_response(&self) -> Option<ApprovalResponse> {
        self.response.lock().unwrap().clone()
    }
}

impl Default for MockApprovalHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl ApprovalHandler for MockApprovalHandler {
    async fn request_approval(
        &self,
        request: ApprovalRequest,
    ) -> Result<ApprovalResponse, ApprovalError> {
        // Check auto-approve first
        if *self.auto_approve.lock().unwrap() {
            return Ok(ApprovalResponse::Approved);
        }

        // Return the stored response (or default to approved for tests)
        match self.response.lock().unwrap().clone() {
            Some(response) => Ok(response),
            None => Ok(ApprovalResponse::Approved),
        }
    }
}

// Handler that always denies
struct DenyHandler;

#[async_trait::async_trait]
impl ApprovalHandler for DenyHandler {
    async fn request_approval(
        &self,
        _request: ApprovalRequest,
    ) -> Result<ApprovalResponse, ApprovalError> {
        Ok(ApprovalResponse::Denied {
            reason: "Test denial".to_string(),
        })
    }
}

// Handler that times out
struct TimeoutHandler;

#[async_trait::async_trait]
impl ApprovalHandler for TimeoutHandler {
    async fn request_approval(
        &self,
        _request: ApprovalRequest,
    ) -> Result<ApprovalResponse, ApprovalError> {
        tokio::time::sleep(Duration::from_secs(3600)).await;
        Ok(ApprovalResponse::Approved)
    }
}

#[tokio::test]
async fn test_is_auto_approved_echo() {
    assert!(is_auto_approved("echo hello"));
    assert!(is_auto_approved("echo 'hello world'"));
    assert!(is_auto_approved("echo test"));
}

#[tokio::test]
async fn test_is_auto_approved_pwd() {
    assert!(is_auto_approved("pwd"));
}

#[tokio::test]
async fn test_is_auto_approved_date() {
    assert!(is_auto_approved("date"));
}

#[tokio::test]
async fn test_is_auto_approved_whoami() {
    assert!(is_auto_approved("whoami"));
}

#[tokio::test]
async fn test_is_auto_approved_hostname() {
    assert!(is_auto_approved("hostname"));
}

#[tokio::test]
async fn test_is_auto_approved_id() {
    assert!(is_auto_approved("id"));
}

#[tokio::test]
async fn test_is_auto_approved_uname() {
    assert!(is_auto_approved("uname -a"));
}

#[tokio::test]
async fn test_is_auto_approved_ls() {
    assert!(is_auto_approved("ls"));
    assert!(is_auto_approved("ls -la"));
    assert!(is_auto_approved("ls -la /tmp"));
}

#[tokio::test]
async fn test_is_auto_approved_cat() {
    assert!(is_auto_approved("cat file.txt"));
    assert!(is_auto_approved("cat /etc/passwd"));
}

#[tokio::test]
async fn test_is_auto_approved_head() {
    assert!(is_auto_approved("head -n 10 file.txt"));
}

#[tokio::test]
async fn test_is_auto_approved_tail() {
    assert!(is_auto_approved("tail -f log.txt"));
}

#[tokio::test]
async fn test_is_auto_approved_grep() {
    assert!(is_auto_approved("grep 'pattern' file.txt"));
}

#[tokio::test]
async fn test_is_auto_approved_find() {
    assert!(is_auto_approved("find /tmp -name '*.txt'"));
}

#[tokio::test]
async fn test_is_auto_approved_true_false() {
    assert!(is_auto_approved("true"));
    assert!(is_auto_approved("false"));
}

#[tokio::test]
async fn test_is_auto_approved_test() {
    assert!(is_auto_approved("test -f file.txt"));
    assert!(is_auto_approved("[ -f file.txt ]"));
}

#[tokio::test]
async fn test_is_auto_approved_not_dangerous() {
    assert!(!is_auto_approved("rm -rf /"));
    assert!(!is_auto_approved("sudo apt-get install"));
    assert!(!is_auto_approved("curl http://example.com"));
    assert!(!is_auto_approved("wget http://example.com"));
    assert!(!is_auto_approved("docker run"));
    assert!(!is_auto_approved("python script.py"));
    assert!(!is_auto_approved("node app.js"));
    assert!(!is_auto_approved("chmod 777 /"));
    assert!(!is_auto_approved("chown -R root:root /"));
    assert!(!is_auto_approved("dd if=/dev/zero of=/dev/sda"));
}

#[tokio::test]
async fn test_is_auto_approved_complex_command() {
    assert!(!is_auto_approved("rm -rf /tmp && echo done"));
    assert!(!is_auto_approved("curl http://example.com | cat"));
    assert!(!is_auto_approved("ls && rm file"));
}

#[tokio::test]
async fn test_approval_request_creation() {
    let request = ApprovalRequest::new("agent-1", "test operation", RiskLevel::Low);
    
    assert!(!request.id.is_empty());
    assert_eq!(request.agent_id, "agent-1");
    assert_eq!(request.operation, "test operation");
    assert_eq!(request.risk_level, RiskLevel::Low);
    assert_eq!(request.timeout, Duration::from_secs(30));
}

#[tokio::test]
async fn test_approval_request_with_timeout() {
    let request = ApprovalRequest::new("agent-1", "test operation", RiskLevel::Low)
        .with_timeout(Duration::from_secs(60));
    
    assert_eq!(request.timeout, Duration::from_secs(60));
}

#[tokio::test]
async fn test_approval_request_with_metadata() {
    let request = ApprovalRequest::new("agent-1", "test operation", RiskLevel::Low)
        .with_metadata(json!({"key": "value"}));
    
    assert!(request.metadata.is_some());
    assert_eq!(request.metadata.unwrap()["key"], "value");
}

#[tokio::test]
async fn test_approval_response_approved() {
    let response = ApprovalResponse::Approved;
    
    assert!(response.is_approved());
    assert!(!response.is_denied());
    assert!(!response.is_timed_out());
    assert!(response.denial_reason().is_none());
}

#[tokio::test]
async fn test_approval_response_denied() {
    let response = ApprovalResponse::Denied {
        reason: "Too dangerous".to_string(),
    };
    
    assert!(!response.is_approved());
    assert!(response.is_denied());
    assert!(!response.is_timed_out());
    assert_eq!(response.denial_reason(), Some("Too dangerous"));
}

#[tokio::test]
async fn test_approval_response_timed_out() {
    let response = ApprovalResponse::TimedOut;
    
    assert!(!response.is_approved());
    assert!(!response.is_denied());
    assert!(response.is_timed_out());
    assert!(response.denial_reason().is_none());
}

#[tokio::test]
async fn test_auto_approve_safe_command() {
    let handler = MockApprovalHandler::new();
    handler.set_auto_approve(true);
    
    let request = ApprovalRequest::new(
        "agent-1",
        "echo hello",
        RiskLevel::Low,
    );
    
    // This simulates auto-approval path
    // In the actual system, auto-approved commands skip approval
    assert!(is_auto_approved("echo hello"));
    
    let result = handler.request_approval(request).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_approved());
}

#[tokio::test]
async fn test_manual_approval_flow() {
    let handler = MockApprovalHandler::new();
    handler.set_response(ApprovalResponse::Approved);
    
    let request = ApprovalRequest::new(
        "agent-1",
        "rm -rf /tmp/test",
        RiskLevel::Medium,
    );
    
    let result = handler.request_approval(request).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_approved());
}

#[tokio::test]
async fn test_manual_denial_flow() {
    let handler = MockApprovalHandler::new();
    handler.set_response(ApprovalResponse::Denied {
        reason: "Security policy violation".to_string(),
    });
    
    let request = ApprovalRequest::new(
        "agent-1",
        "sudo rm -rf /",
        RiskLevel::Critical,
    );
    
    let result = handler.request_approval(request).await;
    assert!(result.is_ok());
    
    let response = result.unwrap();
    assert!(response.is_denied());
    assert_eq!(response.denial_reason(), Some("Security policy violation"));
}

#[tokio::test]
async fn test_approval_timeout_flow() {
    let handler = TimeoutHandler;
    let request = ApprovalRequest::new(
        "agent-1",
        "dangerous operation",
        RiskLevel::High,
    )
    .with_timeout(Duration::from_millis(100)); // Very short timeout
    
    // Use tokio::time::timeout to simulate timeout behavior
    let result = tokio::time::timeout(Duration::from_millis(200), handler.request_approval(request))
        .await;
    
    // Should timeout
    assert!(result.is_err());
}

#[tokio::test]
async fn test_approval_state_tracker_record_pending() {
    let tracker = ApprovalStateTracker::new();
    
    let result = tracker.record_pending("request-1");
    assert!(result.is_ok());
    
    let pending = tracker.list_pending().unwrap();
    assert_eq!(pending, vec!["request-1".to_string()]);
}

#[tokio::test]
async fn test_approval_state_tracker_record_approved() {
    let tracker = ApprovalStateTracker::new();
    
    tracker.record_pending("request-1").unwrap();
    tracker.record_approved("request-1").unwrap();
    
    let pending = tracker.list_pending().unwrap();
    let approved = tracker.list_approved().unwrap();
    
    assert!(pending.is_empty());
    assert_eq!(approved, vec!["request-1".to_string()]);
}

#[tokio::test]
async fn test_approval_state_tracker_record_denied() {
    let tracker = ApprovalStateTracker::new();
    
    tracker.record_pending("request-1").unwrap();
    tracker.record_denied("request-1", "Security concern").unwrap();
    
    let pending = tracker.list_pending().unwrap();
    let denied = tracker.list_denied().unwrap();
    
    assert!(pending.is_empty());
    assert_eq!(denied, vec![("request-1".to_string(), "Security concern".to_string())]);
}

#[tokio::test]
async fn test_approval_state_tracker_record_timed_out() {
    let tracker = ApprovalStateTracker::new();
    
    tracker.record_pending("request-1").unwrap();
    tracker.record_timed_out("request-1").unwrap();
    
    let pending = tracker.list_pending().unwrap();
    let timed_out = tracker.list_timed_out().unwrap();
    
    assert!(pending.is_empty());
    assert_eq!(timed_out, vec!["request-1".to_string()]);
}

#[tokio::test]
async fn test_approval_state_tracker_get_state() {
    let tracker = ApprovalStateTracker::new();
    
    tracker.record_pending("request-1").unwrap();
    assert_eq!(tracker.get_state("request-1").unwrap(), Some(aisopod_tools::ApprovalState::Pending));
    
    tracker.record_approved("request-1").unwrap();
    assert_eq!(tracker.get_state("request-1").unwrap(), Some(aisopod_tools::ApprovalState::Approved));
}

#[tokio::test]
async fn test_approval_state_tracker_summary() {
    let tracker = ApprovalStateTracker::new();
    
    tracker.record_pending("req-1").unwrap();
    tracker.record_pending("req-2").unwrap();
    tracker.record_approved("req-1").unwrap();
    tracker.record_denied("req-3", "Reason").unwrap();
    tracker.record_timed_out("req-4").unwrap();
    
    let summary = tracker.summary().unwrap();
    
    assert_eq!(summary.pending, 1); // req-2 still pending
    assert_eq!(summary.approved, 1); // req-1
    assert_eq!(summary.denied, 1); // req-3
    assert_eq!(summary.timed_out, 1); // req-4
}

#[tokio::test]
async fn test_approval_risk_level_low() {
    let request = ApprovalRequest::new("agent-1", "echo test", RiskLevel::Low);
    assert_eq!(request.risk_level, RiskLevel::Low);
    assert_eq!(request.risk_level.score(), 1);
}

#[tokio::test]
async fn test_approval_risk_level_medium() {
    let request = ApprovalRequest::new("agent-1", "rm file", RiskLevel::Medium);
    assert_eq!(request.risk_level, RiskLevel::Medium);
    assert_eq!(request.risk_level.score(), 2);
}

#[tokio::test]
async fn test_approval_risk_level_high() {
    let request = ApprovalRequest::new("agent-1", "sudo apt-get", RiskLevel::High);
    assert_eq!(request.risk_level, RiskLevel::High);
    assert_eq!(request.risk_level.score(), 3);
}

#[tokio::test]
async fn test_approval_risk_level_critical() {
    let request = ApprovalRequest::new("agent-1", "rm -rf /", RiskLevel::Critical);
    assert_eq!(request.risk_level, RiskLevel::Critical);
    assert_eq!(request.risk_level.score(), 4);
}

#[tokio::test]
async fn test_approval_risk_level_descriptions() {
    assert_eq!(
        RiskLevel::Low.description(),
        "Low risk - minimal potential for harm"
    );
    assert_eq!(
        RiskLevel::Medium.description(),
        "Medium risk - potential for moderate harm"
    );
    assert_eq!(
        RiskLevel::High.description(),
        "High risk - significant potential for harm"
    );
    assert_eq!(
        RiskLevel::Critical.description(),
        "Critical risk - potentially catastrophic consequences"
    );
}

#[tokio::test]
async fn test_approval_handler_error_types() {
    let handler = DenyHandler;
    
    let request = ApprovalRequest::new("agent-1", "test", RiskLevel::Low);
    let result = handler.request_approval(request).await;
    
    // Should get Denied error variant
    match result {
        Ok(ApprovalResponse::Denied { .. }) => {}
        _ => panic!("Expected Denied response"),
    }
}

#[tokio::test]
async fn test_approval_request_unique_ids() {
    let request1 = ApprovalRequest::new("agent-1", "test", RiskLevel::Low);
    let request2 = ApprovalRequest::new("agent-1", "test", RiskLevel::Low);
    
    assert_ne!(request1.id, request2.id);
}

#[tokio::test]
async fn test_approval_with_metadata() {
    let handler = MockApprovalHandler::new();
    handler.set_response(ApprovalResponse::Approved);
    
    let request = ApprovalRequest::new("agent-1", "test", RiskLevel::Low)
        .with_metadata(json!({
            "file_path": "/etc/passwd",
            "action": "read"
        }));
    
    let result = handler.request_approval(request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_approval_timeout_custom_duration() {
    let request = ApprovalRequest::new("agent-1", "test", RiskLevel::Low)
        .with_timeout(Duration::from_secs(120));
    
    assert_eq!(request.timeout, Duration::from_secs(120));
}

#[tokio::test]
async fn test_approval_empty_operation() {
    let request = ApprovalRequest::new("agent-1", "", RiskLevel::Low);
    
    assert_eq!(request.operation, "");
}

#[tokio::test]
async fn test_approval_special_characters_in_operation() {
    let request = ApprovalRequest::new("agent-1", "rm -rf /tmp/test && echo done", RiskLevel::Medium);
    
    assert!(request.operation.contains("&&"));
}

#[tokio::test]
async fn test_approval_agent_id_variations() {
    let request1 = ApprovalRequest::new("agent-1", "test", RiskLevel::Low);
    let request2 = ApprovalRequest::new("agent-123", "test", RiskLevel::Low);
    let request3 = ApprovalRequest::new("my-agent-456", "test", RiskLevel::Low);
    
    assert_eq!(request1.agent_id, "agent-1");
    assert_eq!(request2.agent_id, "agent-123");
    assert_eq!(request3.agent_id, "my-agent-456");
}

#[tokio::test]
async fn test_approval_handler_noop() {
    // The NoOpApprovalHandler is used by default in ToolContext
    // It auto-approves all requests
    
    let handler = NoOpApprovalHandler::default();
    let request = ApprovalRequest::new("agent-1", "dangerous", RiskLevel::Critical);
    
    let result = handler.request_approval(request).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_approved());
}

#[tokio::test]
async fn test_approval_workflow_full_flow() {
    let tracker = ApprovalStateTracker::new();
    let handler = MockApprovalHandler::new();
    
    // Record pending
    let request = ApprovalRequest::new("agent-1", "sudo apt-get install", RiskLevel::High);
    tracker.record_pending(&request.id).unwrap();
    
    // Handler approves
    handler.set_response(ApprovalResponse::Approved);
    let result = handler.request_approval(request.clone()).await;
    assert!(result.is_ok());
    
    // Record approved
    tracker.record_approved(&request.id).unwrap();
    
    // Verify state
    let state = tracker.get_state(&request.id).unwrap();
    assert_eq!(state, Some(aisopod_tools::ApprovalState::Approved));
    
    // Verify summary
    let summary = tracker.summary().unwrap();
    assert_eq!(summary.pending, 0);
    assert_eq!(summary.approved, 1);
}

#[tokio::test]
async fn test_approval_state_tracker_multiple_requests() {
    let tracker = ApprovalStateTracker::new();
    
    // Record multiple requests
    for i in 0..5 {
        tracker.record_pending(&format!("request-{}", i)).unwrap();
    }
    
    let pending = tracker.list_pending().unwrap();
    assert_eq!(pending.len(), 5);
    
    // Approve some
    tracker.record_approved("request-0").unwrap();
    tracker.record_approved("request-1").unwrap();
    
    let summary = tracker.summary().unwrap();
    assert_eq!(summary.pending, 3);
    assert_eq!(summary.approved, 2);
}

#[tokio::test]
async fn test_approval_with_complex_metadata() {
    let handler = MockApprovalHandler::new();
    handler.set_response(ApprovalResponse::Approved);
    
    let request = ApprovalRequest::new("agent-1", "test", RiskLevel::Low)
        .with_metadata(json!({
            "nested": {
                "deep": {
                    "value": "test"
                }
            },
            "array": [1, 2, 3],
            "boolean": true,
            "number": 42.5
        }));
    
    let result = handler.request_approval(request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_approval_state_tracker_empty() {
    let tracker = ApprovalStateTracker::new();
    
    assert!(tracker.list_pending().unwrap().is_empty());
    assert!(tracker.list_approved().unwrap().is_empty());
    assert!(tracker.list_denied().unwrap().is_empty());
    assert!(tracker.list_timed_out().unwrap().is_empty());
    
    let summary = tracker.summary().unwrap();
    assert_eq!(summary.pending, 0);
    assert_eq!(summary.approved, 0);
    assert_eq!(summary.denied, 0);
    assert_eq!(summary.timed_out, 0);
}

#[tokio::test]
async fn test_approval_with_context_metadata() {
    let handler = MockApprovalHandler::new();
    handler.set_response(ApprovalResponse::Approved);
    
    let request = ApprovalRequest::new("agent-1", "test", RiskLevel::Low)
        .with_metadata(json!({
            "context": {
                "session_id": "session-123",
                "tool_name": "bash"
            }
        }));
    
    let result = handler.request_approval(request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_approval_state_tracker_concurrent_operations() {
    let tracker = std::sync::Arc::new(ApprovalStateTracker::new());
    
    let mut handles = vec![];
    for i in 0..10 {
        let tracker = tracker.clone();
        let handle = tokio::spawn(async move {
            tracker.record_pending(&format!("req-{}", i)).unwrap();
            tracker.record_approved(&format!("req-{}", i)).unwrap();
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.await.unwrap();
    }
    
    let summary = tracker.summary().unwrap();
    assert_eq!(summary.approved, 10);
}

#[tokio::test]
async fn test_approval_handler_with_account_and_peer() {
    let handler = MockApprovalHandler::new();
    handler.set_response(ApprovalResponse::Approved);
    
    let request = ApprovalRequest::new("agent-1", "test", RiskLevel::Low)
        .with_metadata(json!({
            "account": "slack-workspace",
            "peer": "user-123"
        }));
    
    let result = handler.request_approval(request).await;
    assert!(result.is_ok());
}
