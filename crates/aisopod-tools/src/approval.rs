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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
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

    /// Returns the default timeout for an approval request at this risk level.
    pub fn default_timeout(&self) -> Duration {
        match self {
            RiskLevel::Low => Duration::from_secs(30),
            RiskLevel::Medium => Duration::from_secs(60),
            RiskLevel::High => Duration::from_secs(120),
            RiskLevel::Critical => Duration::from_secs(300),
        }
    }

    /// Classifies a command into a risk level based on its content.
    ///
    /// Safe commands like `ls`, `cat`, `echo`, etc. are classified as `Low`.
    /// Dangerous commands like `rm -rf`, `dd`, etc. are classified as `Critical`.
    /// Other commands are classified as `Medium` or `High` based on their potential impact.
    pub fn classify(command: &str) -> Self {
        let cmd = command.trim();
        let first_word = cmd.split_whitespace().next().unwrap_or("").to_lowercase();

        // Safe prefix patterns (multi-word commands)
        let safe_prefixes = [
            "git status",
            "git log",
            "git diff",
            "git branch",
            "git show",
            "git remote",
        ];

        for prefix in safe_prefixes {
            if cmd.starts_with(prefix) {
                return RiskLevel::Low;
            }
        }

        // Safe command patterns
        let safe_commands = [
            "ls",
            "cat",
            "echo",
            "pwd",
            "whoami",
            "date",
            "uname",
            "wc",
            "head",
            "tail",
            "grep",
            "find",
            "which",
            "env",
            "printenv",
            "id",
            "hostname",
            "true",
            "false",
            "test",
            "[",
            "printf",
        ];

        if safe_commands.contains(&first_word.as_str()) {
            return RiskLevel::Low;
        }

        // Dangerous patterns - Critical risk
        let dangerous_critical = [
            "rm -rf",
            "mkfs",
            "dd if=",
            "> /dev/",
            "chmod 777",
            "chown root",
        ];

        for pattern in dangerous_critical {
            if cmd.contains(pattern) {
                return RiskLevel::Critical;
            }
        }

        // Dangerous patterns - High risk
        let dangerous_high = [
            "rm ",
            "rm\"",
            "mv ",
            "mv\"",
            "chmod ",
            "chown ",
            "sudo ",
            "su ",
        ];

        for pattern in dangerous_high {
            if cmd.contains(pattern) {
                return RiskLevel::High;
            }
        }

        // Check for command chaining with dangerous operations
        if cmd.contains("|") && (cmd.contains("curl") || cmd.contains("wget") || cmd.contains("curl\"") || cmd.contains("wget\"")) {
            return RiskLevel::High;
        }

        RiskLevel::Medium
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
    /// Timeout for this approval request.
    pub timeout: Duration,
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
            timeout: Duration::from_secs(30),
            metadata: None,
        }
    }

    /// Creates a new approval request with auto-classified risk level.
    ///
    /// This method automatically determines the risk level based on the
    /// operation content using the RiskLevel::classify function.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The ID of the agent making the request.
    /// * `operation` - Description of the operation.
    pub fn with_auto_classify(agent_id: impl Into<String>, operation: impl Into<String>) -> Self {
        let operation = operation.into();
        let risk_level = RiskLevel::classify(&operation);
        Self::new(agent_id, operation, risk_level)
    }

    /// Sets the timeout for this approval request.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Sets metadata for this approval request.
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Rules for auto-approving low-risk operations.
///
/// This struct controls which risk levels are automatically approved
/// without requiring user intervention. Operations at or below the
/// configured risk level are auto-approved.
#[derive(Debug, Clone)]
pub struct AutoApproveRules {
    /// The maximum risk level to auto-approve.
    pub max_auto_approve_level: RiskLevel,
}

impl Default for AutoApproveRules {
    fn default() -> Self {
        Self {
            max_auto_approve_level: RiskLevel::Low,
        }
    }
}

impl AutoApproveRules {
    /// Creates a new AutoApproveRules with the specified max level.
    pub fn new(max_auto_approve_level: RiskLevel) -> Self {
        Self {
            max_auto_approve_level,
        }
    }

    /// Returns true if the given risk level should be auto-approved.
    pub fn should_auto_approve(&self, risk_level: &RiskLevel) -> bool {
        risk_level <= &self.max_auto_approve_level
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

/// Alias for ApprovalResponse for compatibility with the approval workflow.
/// This is used in the ApprovalHandler trait and approval request processing.
pub type ApprovalDecision = ApprovalResponse;

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

/// A no-op approval handler that always approves requests.
///
/// This implementation is useful for testing scenarios where actual
/// approval is not needed.
#[derive(Clone, Default)]
pub struct NoOpApprovalHandler;

#[async_trait]
impl ApprovalHandler for NoOpApprovalHandler {
    async fn request_approval(
        &self,
        _request: ApprovalRequest,
    ) -> Result<ApprovalResponse, ApprovalError> {
        Ok(ApprovalResponse::Approved)
    }
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
    #[allow(dead_code)]
    counter: AtomicU64,
}

/// Current state of an approval request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    pub fn record_denied(
        &self,
        request_id: impl Into<String>,
        reason: impl Into<String>,
    ) -> Result<()> {
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
    // Trim the command
    let cmd = command.trim();

    // Check for dangerous shell operators that could chain commands
    let dangerous_operators = ["&&", "||", "|", ";", "`", "$("];
    for op in dangerous_operators {
        if cmd.contains(op) {
            return false;
        }
    }

    // Get the first word (the command name)
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

    #[test]
    fn test_approval_request_creation() {
        let request = ApprovalRequest::new("agent-1", "test operation", RiskLevel::Low);
        assert!(!request.id.is_empty());
        assert_eq!(request.agent_id, "agent-1");
        assert_eq!(request.operation, "test operation");
        assert_eq!(request.risk_level, RiskLevel::Low);
        assert_eq!(request.timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_approval_request_with_timeout() {
        let request = ApprovalRequest::new("agent-1", "test operation", RiskLevel::Low)
            .with_timeout(Duration::from_secs(60));
        assert_eq!(request.timeout, Duration::from_secs(60));
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
        assert_eq!(
            tracker.list_denied().unwrap(),
            vec![("req-2".to_string(), "Too dangerous".to_string())]
        );

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

    #[test]
    fn test_risk_level_classify_safe_commands() {
        // Safe info commands
        assert_eq!(RiskLevel::classify("echo hello"), RiskLevel::Low);
        assert_eq!(RiskLevel::classify("echo 'hello world'"), RiskLevel::Low);
        assert_eq!(RiskLevel::classify("pwd"), RiskLevel::Low);
        assert_eq!(RiskLevel::classify("date"), RiskLevel::Low);
        assert_eq!(RiskLevel::classify("whoami"), RiskLevel::Low);
        assert_eq!(RiskLevel::classify("hostname"), RiskLevel::Low);
        assert_eq!(RiskLevel::classify("id"), RiskLevel::Low);
        assert_eq!(RiskLevel::classify("uname -a"), RiskLevel::Low);
        assert_eq!(RiskLevel::classify("true"), RiskLevel::Low);
        assert_eq!(RiskLevel::classify("false"), RiskLevel::Low);

        // Safe read commands
        assert_eq!(RiskLevel::classify("ls"), RiskLevel::Low);
        assert_eq!(RiskLevel::classify("ls -la"), RiskLevel::Low);
        assert_eq!(RiskLevel::classify("ls -la /tmp"), RiskLevel::Low);
        assert_eq!(RiskLevel::classify("cat /etc/passwd"), RiskLevel::Low);
        assert_eq!(RiskLevel::classify("head -n 10 file.txt"), RiskLevel::Low);
        assert_eq!(RiskLevel::classify("tail -f log.txt"), RiskLevel::Low);
        assert_eq!(RiskLevel::classify("grep 'pattern' file.txt"), RiskLevel::Low);
        assert_eq!(RiskLevel::classify("find /tmp -name '*.txt'"), RiskLevel::Low);
        assert_eq!(RiskLevel::classify("wc -l file.txt"), RiskLevel::Low);
        assert_eq!(RiskLevel::classify("which python"), RiskLevel::Low);
        assert_eq!(RiskLevel::classify("env"), RiskLevel::Low);
        assert_eq!(RiskLevel::classify("printenv"), RiskLevel::Low);
    }

    #[test]
    fn test_risk_level_classify_git_commands() {
        // Git read-only commands
        assert_eq!(RiskLevel::classify("git status"), RiskLevel::Low);
        assert_eq!(RiskLevel::classify("git log"), RiskLevel::Low);
        assert_eq!(RiskLevel::classify("git diff"), RiskLevel::Low);
        assert_eq!(RiskLevel::classify("git branch"), RiskLevel::Low);
        assert_eq!(RiskLevel::classify("git show HEAD"), RiskLevel::Low);
        assert_eq!(RiskLevel::classify("git remote -v"), RiskLevel::Low);
    }

    #[test]
    fn test_risk_level_classify_dangerous_critical() {
        // Critical risk commands
        assert_eq!(RiskLevel::classify("rm -rf /"), RiskLevel::Critical);
        assert_eq!(RiskLevel::classify("rm -rf /tmp"), RiskLevel::Critical);
        assert_eq!(RiskLevel::classify("mkfs /dev/sda1"), RiskLevel::Critical);
        assert_eq!(RiskLevel::classify("dd if=/dev/zero of=/dev/sda"), RiskLevel::Critical);
        assert_eq!(RiskLevel::classify("chmod 777 /"), RiskLevel::Critical);
        assert_eq!(RiskLevel::classify("chown root:root /"), RiskLevel::Critical);
    }

    #[test]
    fn test_risk_level_classify_dangerous_high() {
        // High risk commands
        assert_eq!(RiskLevel::classify("rm file.txt"), RiskLevel::High);
        assert_eq!(RiskLevel::classify("mv file /etc"), RiskLevel::High);
        assert_eq!(RiskLevel::classify("chmod 644 file"), RiskLevel::High);
        assert_eq!(RiskLevel::classify("chown user:group file"), RiskLevel::High);
        assert_eq!(RiskLevel::classify("sudo apt-get install"), RiskLevel::High);
        assert_eq!(RiskLevel::classify("su root"), RiskLevel::High);
        assert_eq!(RiskLevel::classify("curl http://example.com | bash"), RiskLevel::High);
        assert_eq!(RiskLevel::classify("wget http://example.com | sh"), RiskLevel::High);
    }

    #[test]
    fn test_risk_level_classify_medium() {
        // Medium risk commands (other commands)
        assert_eq!(RiskLevel::classify("python script.py"), RiskLevel::Medium);
        assert_eq!(RiskLevel::classify("node app.js"), RiskLevel::Medium);
        assert_eq!(RiskLevel::classify("docker run"), RiskLevel::Medium);
        assert_eq!(RiskLevel::classify("npm install"), RiskLevel::Medium);
    }

    #[test]
    fn test_risk_level_default_timeout() {
        assert_eq!(RiskLevel::Low.default_timeout(), Duration::from_secs(30));
        assert_eq!(RiskLevel::Medium.default_timeout(), Duration::from_secs(60));
        assert_eq!(RiskLevel::High.default_timeout(), Duration::from_secs(120));
        assert_eq!(RiskLevel::Critical.default_timeout(), Duration::from_secs(300));
    }

    #[test]
    fn test_auto_approve_rules_default() {
        let rules = AutoApproveRules::default();
        assert_eq!(rules.max_auto_approve_level, RiskLevel::Low);
    }

    #[test]
    fn test_auto_approve_rules_new() {
        let rules = AutoApproveRules::new(RiskLevel::Medium);
        assert_eq!(rules.max_auto_approve_level, RiskLevel::Medium);
    }

    #[test]
    fn test_auto_approve_rules_should_auto_approve() {
        let rules = AutoApproveRules::default();

        // Low should be auto-approved
        assert!(rules.should_auto_approve(&RiskLevel::Low));

        // Medium, High, Critical should not be auto-approved
        assert!(!rules.should_auto_approve(&RiskLevel::Medium));
        assert!(!rules.should_auto_approve(&RiskLevel::High));
        assert!(!rules.should_auto_approve(&RiskLevel::Critical));
    }

    #[test]
    fn test_auto_approve_rules_with_higher_threshold() {
        let rules = AutoApproveRules::new(RiskLevel::Medium);

        // Low and Medium should be auto-approved
        assert!(rules.should_auto_approve(&RiskLevel::Low));
        assert!(rules.should_auto_approve(&RiskLevel::Medium));

        // High and Critical should not be auto-approved
        assert!(!rules.should_auto_approve(&RiskLevel::High));
        assert!(!rules.should_auto_approve(&RiskLevel::Critical));
    }

    #[test]
    fn test_approval_decision_alias() {
        // Test that ApprovalDecision is an alias for ApprovalResponse
        let decision: ApprovalDecision = ApprovalResponse::Approved;
        assert!(decision.is_approved());
    }

    #[test]
    fn test_risk_level_ordering() {
        // Test that RiskLevel implements PartialOrd
        assert!(RiskLevel::Low < RiskLevel::Medium);
        assert!(RiskLevel::Medium < RiskLevel::High);
        assert!(RiskLevel::High < RiskLevel::Critical);

        assert!(RiskLevel::Low <= RiskLevel::Low);
        assert!(RiskLevel::Medium >= RiskLevel::Low);
    }
}
