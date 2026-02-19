//! Approval Workflow system for dangerous tool operations.
//!
//! This module provides a framework for requesting and tracking approvals
//! before executing potentially dangerous operations. It supports both
//! synchronous approval requests and automatic approval for safe operations.
//!
//! # Core Types
//!
//! - [`ApprovalRequest`]: A request for approval to perform an operation.
//! - [`ApprovalResponse`]: The response to an approval request (Approved, Denied, TimedOut).
//! - [`ApprovalHandler`]: Trait for handling approval requests.
//! - [`ApprovalStateTracker`]: Tracks the state of all approval requests.
//! - [`RiskLevel`]: Enum representing the risk level of an operation.
//!
//! # Auto-Approval
//!
//! The [`is_auto_approved`] function checks if a command is safe enough to
//! execute without requiring approval. Safe patterns include:
//! - `echo`, `pwd`, `date`, `whoami`, `hostname`, `id`, `uname`
//! - `ls`, `cat`, `head`, `tail`, `grep`, `find` (read-only operations)
//! - Commands that start with these safe commands
//!
//! # Example
//!
//! ```ignore
//! use aisopod_tools::{ApprovalRequest, ApprovalResponse, ApprovalHandler, RiskLevel};
//!
//! struct ManualApprovalHandler;
//!
//! impl ApprovalHandler for ManualApprovalHandler {
//!     async fn request_approval(
//!         &self,
//!         request: ApprovalRequest,
//!     ) -> Result<ApprovalResponse, ApprovalError> {
//!         // Present request to user and await response
//!         println!("Approval requested for: {}", request.operation);
//!         // ... wait for user input ...
//!         Ok(ApprovalResponse::Approved)
//!     }
//! }
//! ```

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

// Counter for generating unique IDs
static REQUEST_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Risk level of a dangerous operation.
///
/// Used to determine the urgency and approval requirements for operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum RiskLevel {
    /// Low risk - minimal potential for harm (e.g., read-only operations).
    Low,
    /// Medium risk - potential for moderate harm (e.g., file modifications).
    Medium,
    /// High risk - significant potential for harm (e.g., system modifications).
    High,
    /// Critical risk - potentially catastrophic consequences (e.g., destructive operations).
    Critical,
}

impl RiskLevel {
    /// Returns a numeric score for the risk level (for comparison).
    pub fn score(&self) -> u8 {
        match self {
            RiskLevel::Low => 1,
            RiskLevel::Medium => 2,
            RiskLevel::High => 3,
            RiskLevel::Critical => 4,
        }
    }

    /// Returns a human-readable description of the risk level.
    pub fn description(&self) -> &'static str {
        match self {
            RiskLevel::Low => "Low risk - minimal potential for harm",
            RiskLevel::Medium => "Medium risk - potential for moderate harm",
            RiskLevel::High => "High risk - significant potential for harm",
            RiskLevel::Critical => "Critical risk - potentially catastrophic consequences",
        }
    }
}

/// A request for approval to perform a dangerous operation.
///
/// This struct captures all necessary information about an operation
/// that requires user approval before execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    /// Unique identifier for this approval request.
    pub id: String,
    /// Identifier of the agent requesting approval.
    pub agent_id: String,
    /// Description of the operation being requested.
    pub operation: String,
    /// The risk level of the operation.
    pub risk_level: RiskLevel,
    /// Optional timeout for this approval request.
    pub timeout: Option<Duration>,
    /// Optional metadata about the request.
    pub metadata: Option<serde_json::Value>,
}

impl ApprovalRequest {
    /// Creates a new approval request with default settings.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The ID of the agent making the request.
    /// * `operation` - Description of the operation.
    /// * `risk_level` - The risk level of the operation.
    pub fn new(
        agent_id: impl Into<String>,
        operation: impl Into<String>,
        risk_level: RiskLevel,
    ) -> Self {
        // Convert inputs to String to avoid move issues
        let agent_id = agent_id.into();
        let operation = operation.into();
        
        // Generate a unique ID using timestamp + atomic counter
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let counter = REQUEST_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        Self {
            id: format!("{}-{}-{}", agent_id, timestamp, counter),
            agent_id,
            operation,
            risk_level,
            timeout: None,
            metadata: None,
        }
    }

    /// Sets the timeout for this approval request.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Sets metadata for this approval request.
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Response to an approval request.
///
/// Represents the possible outcomes of an approval request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApprovalResponse {
    /// The operation has been approved.
    Approved,
    /// The operation has been denied.
    Denied {
        /// Reason for the denial.
        reason: String,
    },
    /// The approval request timed out.
    TimedOut,
}

impl ApprovalResponse {
    /// Returns true if the operation was approved.
    pub fn is_approved(&self) -> bool {
        matches!(self, ApprovalResponse::Approved)
    }

    /// Returns true if the operation was denied.
    pub fn is_denied(&self) -> bool {
        matches!(self, ApprovalResponse::Denied { .. })
    }

    /// Returns true if the request timed out.
    pub fn is_timed_out(&self) -> bool {
        matches!(self, ApprovalResponse::TimedOut)
    }

    /// Returns the denial reason if applicable.
    pub fn denial_reason(&self) -> Option<&str> {
        match self {
            ApprovalResponse::Denied { reason } => Some(reason.as_str()),
            _ => None,
        }
    }
}

/// Trait for handling approval requests.
///
/// Implementations of this trait are responsible for:
/// - Presenting approval requests to users or external systems
/// - Awaiting approval decisions
/// - Handling timeouts
#[async_trait]
pub trait ApprovalHandler: Send + Sync {
    /// Request approval for an operation.
    ///
    /// This method should block until a decision is made or the timeout expires.
    ///
    /// # Arguments
    ///
    /// * `request` - The approval request containing operation details.
    ///
    /// # Returns
    ///
    /// * `Ok(ApprovalResponse)` - The decision on the request.
    /// * `Err(ApprovalError)` - An error occurred while processing the request.
    async fn request_approval(
        &self,
        request: ApprovalRequest,
    ) -> Result<ApprovalResponse, ApprovalError>;
}

/// Error type for approval-related failures.
#[derive(Debug, thiserror::Error)]
pub enum ApprovalError {
    /// The approval request was denied by the user.
    #[error("Approval denied: {0}")]
    Denied(String),
    /// The approval handler encountered an internal error.
    #[error("Approval handler error: {0}")]
    HandlerError(String),
    /// The approval request timed out.
    #[error("Approval request timed out")]
    Timeout,
}

/// Tracks the state of approval requests.
///
/// This struct maintains a registry of all approval requests and their
/// current states (pending, approved, denied, timed_out).
#[derive(Debug, Default)]
pub struct ApprovalStateTracker {
    /// Map of request ID to its current state.
    states: std::sync::Arc<std::sync::RwLock<HashMap<String, ApprovalState>>>,
    /// Counter for generating sequential IDs.
    counter: AtomicU64,
}

/// Current state of an approval request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApprovalState {
    /// The request is pending approval.
    Pending,
    /// The request has been approved.
    Approved,
    /// The request has been denied.
    Denied { reason: String },
    /// The request has timed out.
    TimedOut,
}

impl ApprovalStateTracker {
    /// Creates a new `ApprovalStateTracker` with no tracked requests.
    pub fn new() -> Self {
        Self {
            states: std::sync::Arc::new(std::sync::RwLock::new(HashMap::new())),
            counter: AtomicU64::new(0),
        }
    }

    /// Records a new pending approval request.
    ///
    /// # Arguments
    ///
    /// * `request_id` - The ID of the approval request.
    pub fn record_pending(&self, request_id: impl Into<String>) -> Result<()> {
        let mut states = self
            .states
            .write()
            .map_err(|_| anyhow!("Failed to acquire write lock on states"))?;
        states.insert(request_id.into(), ApprovalState::Pending);
        Ok(())
    }

    /// Records that an approval request was approved.
    ///
    /// # Arguments
    ///
    /// * `request_id` - The ID of the approval request.
    pub fn record_approved(&self, request_id: impl Into<String>) -> Result<()> {
        let mut states = self
            .states
            .write()
            .map_err(|_| anyhow!("Failed to acquire write lock on states"))?;
        states.insert(request_id.into(), ApprovalState::Approved);
        Ok(())
    }

    /// Records that an approval request was denied.
    ///
    /// # Arguments
    ///
    /// * `request_id` - The ID of the approval request.
    /// * `reason` - The reason for denial.
    pub fn record_denied(&self, request_id: impl Into<String>, reason: impl Into<String>) -> Result<()> {
        let mut states = self
            .states
            .write()
            .map_err(|_| anyhow!("Failed to acquire write lock on states"))?;
        states.insert(
            request_id.into(),
            ApprovalState::Denied {
                reason: reason.into(),
            },
        );
        Ok(())
    }

    /// Records that an approval request timed out.
    ///
    /// # Arguments
    ///
    /// * `request_id` - The ID of the approval request.
    pub fn record_timed_out(&self, request_id: impl Into<String>) -> Result<()> {
        let mut states = self
            .states
            .write()
            .map_err(|_| anyhow!("Failed to acquire write lock on states"))?;
        states.insert(request_id.into(), ApprovalState::TimedOut);
        Ok(())
    }

    /// Gets the current state of an approval request.
    ///
    /// # Arguments
    ///
    /// * `request_id` - The ID of the approval request.
    pub fn get_state(&self, request_id: impl Into<String>) -> Result<Option<ApprovalState>> {
        let states = self
            .states
            .read()
            .map_err(|_| anyhow!("Failed to acquire read lock on states"))?;
        Ok(states.get(&request_id.into()).cloned())
    }

    /// Returns a list of all pending approval requests.
    pub fn list_pending(&self) -> Result<Vec<String>> {
        let states = self
            .states
            .read()
            .map_err(|_| anyhow!("Failed to acquire read lock on states"))?;
        Ok(states
            .iter()
            .filter(|(_, state)| matches!(state, ApprovalState::Pending))
            .map(|(id, _)| id.clone())
            .collect())
    }

    /// Returns a list of all approved approval requests.
    pub fn list_approved(&self) -> Result<Vec<String>> {
        let states = self
            .states
            .read()
            .map_err(|_| anyhow!("Failed to acquire read lock on states"))?;
        Ok(states
            .iter()
            .filter(|(_, state)| matches!(state, ApprovalState::Approved))
            .map(|(id, _)| id.clone())
            .collect())
    }

    /// Returns a list of all denied approval requests with their reasons.
    pub fn list_denied(&self) -> Result<Vec<(String, String)>> {
        let states = self
            .states
            .read()
            .map_err(|_| anyhow!("Failed to acquire read lock on states"))?;
        Ok(states
            .iter()
            .filter_map(|(id, state)| {
                if let ApprovalState::Denied { reason } = state {
                    Some((id.clone(), reason.clone()))
                } else {
                    None
                }
            })
            .collect())
    }

    /// Returns a list of all timed-out approval requests.
    pub fn list_timed_out(&self) -> Result<Vec<String>> {
        let states = self
            .states
            .read()
            .map_err(|_| anyhow!("Failed to acquire read lock on states"))?;
        Ok(states
            .iter()
            .filter(|(_, state)| matches!(state, ApprovalState::TimedOut))
            .map(|(id, _)| id.clone())
            .collect())
    }

    /// Returns a summary of all approval request states.
    pub fn summary(&self) -> Result<ApprovalSummary> {
        let states = self
            .states
            .read()
            .map_err(|_| anyhow!("Failed to acquire read lock on states"))?;
        let mut summary = ApprovalSummary::default();
        for state in states.values() {
            match state {
                ApprovalState::Pending => summary.pending += 1,
                ApprovalState::Approved => summary.approved += 1,
                ApprovalState::Denied { .. } => summary.denied += 1,
                ApprovalState::TimedOut => summary.timed_out += 1,
            }
        }
        Ok(summary)
    }
}

/// Summary of approval request states.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ApprovalSummary {
    /// Number of pending requests.
    pub pending: u64,
    /// Number of approved requests.
    pub approved: u64,
    /// Number of denied requests.
    pub denied: u64,
    /// Number of timed-out requests.
    pub timed_out: u64,
}

/// Checks if a command is auto-approved (safe enough to execute without approval).
///
/// This function uses pattern matching to identify commands that are considered
/// safe based on their operation type. Safe patterns include:
///
/// - **Information commands**: `echo`, `pwd`, `date`, `whoami`, `hostname`, `id`, `uname`
/// - **Read-only file operations**: `ls`, `cat`, `head`, `tail`, `grep`, `find`
/// - **Safe shell utilities**: `true`, `false`, `test`, `[`, `echo`, `printf`
///
/// # Arguments
///
/// * `command` - The shell command to check.
///
/// # Returns
///
/// * `true` - The command is considered safe and auto-approved.
/// * `false` - The command requires approval.
///
/// # Example
///
/// ```
/// use aisopod_tools::is_auto_approved;
///
/// assert!(is_auto_approved("echo hello"));
/// assert!(is_auto_approved("ls -la"));
/// assert!(is_auto_approved("pwd"));
/// assert!(!is_auto_approved("rm -rf /"));
/// assert!(!is_auto_approved("curl http://example.com"));
/// ```
pub fn is_auto_approved(command: &str) -> bool {
    // Trim the command and get the first word (the command name)
    let cmd = command.trim();
    let first_word = cmd.split_whitespace().next().unwrap_or("");

    // Safe informational commands
    let safe_info_commands = [
        "echo", "pwd", "date", "whoami", "hostname", "id", "uname", "true", "false", "test", "[",
        "printf",
    ];

    // Safe read-only file operations
    let safe_read_commands = ["ls", "cat", "head", "tail", "grep", "find", "stat", "file"];

    // Check if the first word matches any safe command
    // Also check for common safe patterns
    safe_info_commands
        .iter()
        .chain(&safe_read_commands)
        .any(|&safe| first_word == safe)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_level_score() {
        assert_eq!(RiskLevel::Low.score(), 1);
        assert_eq!(RiskLevel::Medium.score(), 2);
        assert_eq!(RiskLevel::High.score(), 3);
        assert_eq!(RiskLevel::Critical.score(), 4);
    }

    #[test]
    fn test_risk_level_description() {
        assert_eq!(RiskLevel::Low.description(), "Low risk - minimal potential for harm");
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

    #[test]
    fn test_approval_request_creation() {
        let request = ApprovalRequest::new("agent-1", "test operation", RiskLevel::Low);
        assert!(!request.id.is_empty());
        assert_eq!(request.agent_id, "agent-1");
        assert_eq!(request.operation, "test operation");
        assert_eq!(request.risk_level, RiskLevel::Low);
        assert!(request.timeout.is_none());
    }

    #[test]
    fn test_approval_request_with_timeout() {
        let request =
            ApprovalRequest::new("agent-1", "test operation", RiskLevel::Low)
                .with_timeout(Duration::from_secs(30));
        assert!(request.timeout.is_some());
        assert_eq!(request.timeout.unwrap(), Duration::from_secs(30));
    }

    #[test]
    fn test_approval_response_variants() {
        let approved = ApprovalResponse::Approved;
        let denied = ApprovalResponse::Denied {
            reason: "Too dangerous".to_string(),
        };
        let timed_out = ApprovalResponse::TimedOut;

        assert!(approved.is_approved());
        assert!(!approved.is_denied());
        assert!(!approved.is_timed_out());

        assert!(denied.is_denied());
        assert_eq!(denied.denial_reason(), Some("Too dangerous"));

        assert!(timed_out.is_timed_out());
    }

    #[test]
    fn test_is_auto_approved_safe_commands() {
        assert!(is_auto_approved("echo hello"));
        assert!(is_auto_approved("echo 'hello world'"));
        assert!(is_auto_approved("pwd"));
        assert!(is_auto_approved("date"));
        assert!(is_auto_approved("whoami"));
        assert!(is_auto_approved("hostname"));
        assert!(is_auto_approved("id"));
        assert!(is_auto_approved("uname -a"));
        assert!(is_auto_approved("true"));
        assert!(is_auto_approved("false"));
        assert!(is_auto_approved("ls"));
        assert!(is_auto_approved("ls -la"));
        assert!(is_auto_approved("ls -la /tmp"));
        assert!(is_auto_approved("cat /etc/passwd"));
        assert!(is_auto_approved("head -n 10 file.txt"));
        assert!(is_auto_approved("tail -f log.txt"));
        assert!(is_auto_approved("grep 'pattern' file.txt"));
        assert!(is_auto_approved("find /tmp -name '*.txt'"));
    }

    #[test]
    fn test_is_auto_approved_dangerous_commands() {
        assert!(!is_auto_approved("rm -rf /"));
        assert!(!is_auto_approved("sudo apt-get install"));
        assert!(!is_auto_approved("curl http://example.com"));
        assert!(!is_auto_approved("wget http://example.com"));
        assert!(!is_auto_approved("docker run"));
        assert!(!is_auto_approved("python script.py"));
        assert!(!is_auto_approved("node app.js"));
        assert!(!is_auto_approved("rm -rf /tmp"));
        assert!(!is_auto_approved("mv file /etc"));
        assert!(!is_auto_approved("chmod 777 /"));
    }

    #[test]
    fn test_approval_state_tracker() {
        let tracker = ApprovalStateTracker::new();

        // Record pending
        tracker.record_pending("req-1").unwrap();
        assert_eq!(tracker.list_pending().unwrap(), vec!["req-1".to_string()]);

        // Record approved
        tracker.record_approved("req-1").unwrap();
        assert!(tracker.list_pending().unwrap().is_empty());
        assert_eq!(tracker.list_approved().unwrap(), vec!["req-1".to_string()]);

        // Record denied
        tracker.record_pending("req-2").unwrap();
        tracker.record_denied("req-2", "Too dangerous").unwrap();
        assert_eq!(tracker.list_denied().unwrap(), vec![("req-2".to_string(), "Too dangerous".to_string())]);

        // Record timed out
        tracker.record_pending("req-3").unwrap();
        tracker.record_timed_out("req-3").unwrap();
        assert_eq!(tracker.list_timed_out().unwrap(), vec!["req-3".to_string()]);

        // Check summary
        let summary = tracker.summary().unwrap();
        assert_eq!(summary.pending, 0);
        assert_eq!(summary.approved, 1);
        assert_eq!(summary.denied, 1);
        assert_eq!(summary.timed_out, 1);
    }
}
