//! Audit logging for security-relevant events
//!
//! This module provides structured audit logging for all security-relevant events
//! in the gateway, including authentication attempts, authorization decisions,
//! tool executions, and configuration changes.

use tracing::{info, warn};

/// Log a successful authentication.
///
/// This should be called when a user successfully authenticates via
/// token or password authentication.
///
/// # Arguments
/// * `client_ip` - The IP address of the client making the request
/// * `auth_mode` - The authentication mode used ("token" or "password")
/// * `role` - The role assigned to the authenticated user
pub fn log_auth_success(client_ip: &str, auth_mode: &str, role: &str) {
    info!(
        target: "audit",
        event = "auth_success",
        client_ip = client_ip,
        auth_mode = auth_mode,
        role = role,
        "Authentication successful"
    );
}

/// Log a failed authentication attempt.
///
/// This should be called when authentication fails, which could indicate
/// a brute force attack or misconfigured client.
///
/// # Arguments
/// * `client_ip` - The IP address of the client making the request
/// * `auth_mode` - The authentication mode used ("token" or "password")
/// * `reason` - The reason for authentication failure
pub fn log_auth_failure(client_ip: &str, auth_mode: &str, reason: &str) {
    warn!(
        target: "audit",
        event = "auth_failure",
        client_ip = client_ip,
        auth_mode = auth_mode,
        reason = reason,
        "Authentication failed"
    );
}

/// Log an authorization decision.
///
/// This should be called after checking if a user has the required
/// scope for a specific RPC method.
///
/// # Arguments
/// * `method` - The RPC method being accessed
/// * `required_scope` - The scope required to access the method
/// * `granted` - Whether access was granted
/// * `client_ip` - The IP address of the client making the request
pub fn log_authz_decision(
    method: &str,
    required_scope: &str,
    granted: bool,
    client_ip: &str,
) {
    if granted {
        info!(
            target: "audit",
            event = "authz_granted",
            method = method,
            required_scope = required_scope,
            client_ip = client_ip,
            "Authorization granted"
        );
    } else {
        warn!(
            target: "audit",
            event = "authz_denied",
            method = method,
            required_scope = required_scope,
            client_ip = client_ip,
            "Authorization denied"
        );
    }
}

/// Log a tool execution.
///
/// This should be called before and after tool execution to track
/// which tools are being run by which agents.
///
/// # Arguments
/// * `tool_name` - The name of the tool being executed
/// * `agent_id` - The ID of the agent executing the tool
/// * `sandboxed` - Whether the tool runs in a sandbox
/// * `session_key` - The session key for correlation
pub fn log_tool_execution(
    tool_name: &str,
    agent_id: &str,
    sandboxed: bool,
    session_key: &str,
) {
    info!(
        target: "audit",
        event = "tool_execution",
        tool_name = tool_name,
        agent_id = agent_id,
        sandboxed = sandboxed,
        session_key = session_key,
        "Tool executed"
    );
}

/// Log an approval workflow event.
///
/// This should be called when an approval request is made, approved,
/// or denied.
///
/// # Arguments
/// * `request_id` - The unique ID of the approval request
/// * `agent_id` - The ID of the agent making the request
/// * `operation` - The operation being approved
/// * `decision` - The decision made ("approved" or "denied")
/// * `duration_ms` - The time taken to make the decision in milliseconds
pub fn log_approval_event(
    request_id: &str,
    agent_id: &str,
    operation: &str,
    decision: &str,
    duration_ms: u64,
) {
    info!(
        target: "audit",
        event = "approval_decision",
        request_id = request_id,
        agent_id = agent_id,
        operation = operation,
        decision = decision,
        duration_ms = duration_ms,
        "Approval workflow completed"
    );
}

/// Log a configuration change (with secrets redacted).
///
/// This should be called when configuration is updated. The old_value
/// and new_value parameters should already have secrets redacted.
///
/// # Arguments
/// * `field` - The configuration field that changed
/// * `old_value` - The previous value (secrets already redacted)
/// * `new_value` - The new value (secrets already redacted)
pub fn log_config_change(field: &str, old_value: &str, new_value: &str) {
    info!(
        target: "audit",
        event = "config_change",
        field = field,
        old_value = old_value,
        new_value = new_value,
        "Configuration changed"
    );
}

/// Redact sensitive information from a value.
///
/// This function redacts common sensitive patterns like passwords, tokens,
/// and API keys from strings before logging them.
///
/// # Arguments
/// * `value` - The value to redact
///
/// # Returns
/// A redacted version of the value, or the original if no sensitive patterns found.
pub fn redact_sensitive(value: &str) -> String {
    if value.is_empty() {
        return "<empty>".to_string();
    }

    // Check for highly sensitive patterns that warrant complete redaction
    if value.to_lowercase().contains("password") || value.to_lowercase().contains("token") {
        return "******".to_string();
    }

    // For other sensitive patterns (secret, key), truncate to 20 chars of original
    if value.to_lowercase().contains("secret") || value.to_lowercase().contains("key") {
        if value.len() > 20 {
            format!("{}...", &value[..20])
        } else {
            value.to_string()
        }
    } else if value.len() > 20 {
        // For long non-sensitive values, just truncate
        format!("{}...", &value[..20])
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_sensitive_basic() {
        let input = "password123";
        let output = redact_sensitive(input);
        assert_eq!(output, "******");
    }

    #[test]
    fn test_redact_sensitive_long_value() {
        let input = "my-secret-api-key-value-that-is-very-long";
        let output = redact_sensitive(input);
        assert!(output.starts_with("my-secre"));
        assert!(output.ends_with("..."));
        assert!(output.len() <= 23); // 20 chars + "..."
    }

    #[test]
    fn test_redact_sensitive_empty() {
        let output = redact_sensitive("");
        assert_eq!(output, "<empty>");
    }

    #[test]
    fn test_log_auth_success_compiles() {
        // Just verify this compiles
        log_auth_success("192.168.1.1", "token", "admin");
    }

    #[test]
    fn test_log_auth_failure_compiles() {
        // Just verify this compiles
        log_auth_failure("192.168.1.1", "password", "invalid credentials");
    }

    #[test]
    fn test_log_authz_decision_compiles() {
        // Just verify this compiles
        log_authz_decision("agent.list", "operator.read", true, "192.168.1.1");
    }

    #[test]
    fn test_log_tool_execution_compiles() {
        // Just verify this compiles
        log_tool_execution("bash", "agent-123", true, "session-456");
    }

    #[test]
    fn test_log_approval_event_compiles() {
        // Just verify this compiles
        log_approval_event("req-123", "agent-456", "agent.start", "approved", 42);
    }

    #[test]
    fn test_log_config_change_compiles() {
        // Just verify this compiles
        log_config_change("max_connections", "100", "200");
    }
}
