//! Tool Policy Enforcement system for controlling tool access.
//!
//! This module provides a policy engine that allows or denies tool usage
//! based on configurable allow lists (whitelists) and deny lists (blacklists)
//! at both global and per-agent levels.
//!
//! # Policy Semantics
//!
//! - **Allow list**: If present, only tools in the allow list are permitted.
//! - **Deny list**: If present, tools in the deny list are blocked.
//! - **Precedence**: Deny lists always take precedence over allow lists.
//!
//! # Example
//!
//! ```ignore
//! use aisopod_tools::{ToolPolicy, ToolPolicyEngine};
//!
//! // Create a policy engine with a global deny list
//! let mut engine = ToolPolicyEngine::new();
//!
//! // Set a global policy that blocks the 'bash' tool
//! let global_policy = ToolPolicy {
//!     allow: None,
//!     deny: Some(vec!["bash".to_string(), "python".to_string()]),
//! };
//! engine.set_global_policy(global_policy);
//!
//! // Set a per-agent policy that allows only specific tools
//! let agent_policy = ToolPolicy {
//!     allow: Some(vec!["read_file".to_string(), "list_files".to_string()]),
//!     deny: None,
//! };
//! engine.set_agent_policy("agent-1".to_string(), agent_policy);
//!
//! // Check if a tool is allowed
//! match engine.is_allowed("agent-1", "read_file") {
//!     Ok(()) => println!("Tool is allowed"),
//!     Err(reason) => println!("Tool denied: {}", reason),
//! }
//! ```

use std::collections::HashMap;

/// A policy that controls which tools are allowed or denied.
///
/// The policy supports both allow lists (whitelists) and deny lists (blacklists).
/// When both are specified, deny lists take precedence over allow lists.
///
/// # Semantics
///
/// - If `allow` is `Some(vec![...])`, only tools in this list are permitted.
/// - If `deny` is `Some(vec![...])`, tools in this list are blocked.
/// - If `allow` is `None` and `deny` is `Some(...)`, all tools except denied ones are allowed.
/// - If both are `None`, all tools are permitted (unrestricted mode).
/// - If both are `Some(...)`, denied tools are blocked regardless of the allow list.
///
/// # Example
///
/// ```ignore
/// // Allow only specific tools
/// let policy = ToolPolicy {
///     allow: Some(vec!["read_file".to_string(), "list_files".to_string()]),
///     deny: None,
/// };
///
/// // Block specific tools (all others allowed)
/// let policy = ToolPolicy {
///     allow: None,
///     deny: Some(vec!["bash".to_string(), "python".to_string()]),
/// };
///
/// // Unrestricted - all tools allowed
/// let policy = ToolPolicy {
///     allow: None,
///     deny: None,
/// };
/// ```
#[derive(Debug, Clone, Default)]
pub struct ToolPolicy {
    /// Optional allow list - if present, only these tools are permitted.
    pub allow: Option<Vec<String>>,
    /// Optional deny list - if present, these tools are blocked.
    pub deny: Option<Vec<String>>,
}

impl ToolPolicy {
    /// Creates a new empty policy with no restrictions.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a policy with only an allow list (whitelist mode).
    ///
    /// # Arguments
    ///
    /// * `allowed_tools` - Vector of tool names that are permitted.
    pub fn allow_list(allowed_tools: Vec<String>) -> Self {
        Self {
            allow: Some(allowed_tools),
            deny: None,
        }
    }

    /// Creates a policy with only a deny list (blacklist mode).
    ///
    /// # Arguments
    ///
    /// * `denied_tools` - Vector of tool names that are blocked.
    pub fn deny_list(denied_tools: Vec<String>) -> Self {
        Self {
            allow: None,
            deny: Some(denied_tools),
        }
    }

    /// Checks if a tool is explicitly allowed by this policy.
    ///
    /// Returns `true` if the tool is in the allow list.
    /// Returns `false` if tool is in deny list or no allow list is set.
    fn is_explicitly_allowed(&self, tool_name: &str) -> bool {
        // If denied, never allowed
        if let Some(deny_list) = &self.deny {
            if deny_list.contains(&tool_name.to_string()) {
                return false;
            }
        }
        // Only return true if tool is explicitly in allow list
        match &self.allow {
            Some(allow_list) => allow_list.contains(&tool_name.to_string()),
            None => false,
        }
    }

    /// Checks if a tool is explicitly denied by this policy.
    ///
    /// Returns `true` if the tool is in the deny list.
    fn is_explicitly_denied(&self, tool_name: &str) -> bool {
        match &self.deny {
            Some(deny_list) => deny_list.contains(&tool_name.to_string()),
            None => false,
        }
    }
}

/// Engine that evaluates tool access policies for agents.
///
/// The engine holds a global policy that applies to all agents, and
/// individual agent policies that can override the global policy.
/// Policy evaluation follows these rules:
///
/// 1. Deny always takes precedence - if any policy denies the tool, it's blocked.
/// 2. Per-agent policies override global policies.
/// 3. If an allow list is present, only tools in it are permitted.
/// 4. If no allow list exists, all tools are allowed (unless denied).
///
/// # Example
///
/// ```ignore
/// let mut engine = ToolPolicyEngine::new();
///
/// // Set global policy - block dangerous tools
/// engine.set_global_policy(ToolPolicy::deny_list(vec![
///     "bash".to_string(),
///     "python".to_string(),
/// ]));
///
/// // Agent-specific override - allow more tools for this agent
/// engine.set_agent_policy("dev-agent".to_string(), ToolPolicy::new());
///
/// // Check permissions
/// let result = engine.is_allowed("dev-agent", "bash");
/// assert!(result.is_err()); // bash is globally denied
/// ```
#[derive(Debug, Clone)]
pub struct ToolPolicyEngine {
    global_policy: ToolPolicy,
    agent_policies: HashMap<String, ToolPolicy>,
}

impl ToolPolicyEngine {
    /// Creates a new `ToolPolicyEngine` with default (unrestricted) global policy.
    pub fn new() -> Self {
        Self {
            global_policy: ToolPolicy::new(),
            agent_policies: HashMap::new(),
        }
    }

    /// Creates a new `ToolPolicyEngine` with the specified global policy.
    ///
    /// # Arguments
    ///
    /// * `global_policy` - The initial global policy for all agents.
    pub fn with_global_policy(global_policy: ToolPolicy) -> Self {
        Self {
            global_policy,
            agent_policies: HashMap::new(),
        }
    }

    /// Sets the global policy that applies to all agents without specific overrides.
    ///
    /// # Arguments
    ///
    /// * `policy` - The policy to apply globally.
    pub fn set_global_policy(&mut self, policy: ToolPolicy) {
        self.global_policy = policy;
    }

    /// Sets a per-agent policy that overrides the global policy for that agent.
    ///
    /// If an agent has a policy set, it takes precedence over the global policy
    /// during `is_allowed` evaluation.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The unique identifier of the agent.
    /// * `policy` - The policy to apply to this agent.
    pub fn set_agent_policy(&mut self, agent_id: String, policy: ToolPolicy) {
        self.agent_policies.insert(agent_id, policy);
    }

    /// Removes the policy for a specific agent, falling back to the global policy.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The unique identifier of the agent whose policy should be removed.
    pub fn remove_agent_policy(&mut self, agent_id: &str) {
        self.agent_policies.remove(agent_id);
    }

    /// Gets the effective policy for an agent (agent-specific or global).
    #[allow(dead_code)]
    fn get_effective_policy(&self, agent_id: &str) -> &ToolPolicy {
        self.agent_policies
            .get(agent_id)
            .unwrap_or(&self.global_policy)
    }

    /// Evaluates whether a tool is allowed for a specific agent.
    ///
    /// Returns `Ok(())` if the tool is permitted, or `Err(reason)` with a
    /// descriptive denial message if the tool is blocked.
    ///
    /// # Policy Evaluation Order
    ///
    /// 1. If agent-specific deny list exists and tool is in it, deny.
    /// 2. If agent-specific allow list exists and tool is NOT in it, deny.
    /// 3. If agent-specific allow list exists, allow (agent overrides global deny).
    /// 4. If global deny list exists and tool is in it, deny.
    /// 5. If global allow list exists and tool is NOT in it, deny.
    /// 6. Tool is allowed.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The unique identifier of the agent requesting tool access.
    /// * `tool_name` - The name of the tool being requested.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - The tool is allowed.
    /// * `Err(String)` - The tool is denied with a descriptive reason.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut engine = ToolPolicyEngine::new();
    /// engine.set_global_policy(ToolPolicy::deny_list(vec!["bash".to_string()]));
    ///
    /// assert!(engine.is_allowed("agent-1", "read_file").is_ok());
    /// assert!(engine.is_allowed("agent-1", "bash").is_err());
    /// ```
    pub fn is_allowed(&self, agent_id: &str, tool_name: &str) -> Result<(), String> {
        // Rule 1: Check agent-specific deny list first (agent policy overrides global)
        if let Some(agent_policy) = self.agent_policies.get(agent_id) {
            if agent_policy.is_explicitly_denied(tool_name) {
                return Err(format!(
                    "Tool '{}' is denied by agent policy for agent '{}'",
                    tool_name, agent_id
                ));
            }
        }

        // Rule 2: If agent-specific allow list exists, check it (agent overrides global)
        if let Some(agent_policy) = self.agent_policies.get(agent_id) {
            if agent_policy.allow.is_some() {
                if !agent_policy.is_explicitly_allowed(tool_name) {
                    return Err(format!(
                        "Tool '{}' is not in the allow list for agent '{}'",
                        tool_name, agent_id
                    ));
                }
                // Agent has allow list and tool is in it - agent overrides global deny
                return Ok(());
            }
        }

        // Rule 3: Check global deny list
        if self.global_policy.is_explicitly_denied(tool_name) {
            return Err(format!(
                "Tool '{}' is denied by the global policy",
                tool_name
            ));
        }

        // Rule 4: Check global allow list
        if self.global_policy.allow.is_some()
            && !self.global_policy.is_explicitly_allowed(tool_name)
        {
            return Err(format!(
                "Tool '{}' is not in the global allow list",
                tool_name
            ));
        }

        // Rule 5: Tool is allowed
        Ok(())
    }
}

impl Default for ToolPolicyEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_policy_default_is_unrestricted() {
        let policy = ToolPolicy::new();
        assert!(policy.allow.is_none());
        assert!(policy.deny.is_none());
    }

    #[test]
    fn test_tool_policy_allow_list() {
        let policy =
            ToolPolicy::allow_list(vec!["read_file".to_string(), "list_files".to_string()]);

        assert!(policy.is_explicitly_allowed("read_file"));
        assert!(policy.is_explicitly_allowed("list_files"));
        assert!(!policy.is_explicitly_allowed("bash"));
        assert!(policy.deny.is_none());
    }

    #[test]
    fn test_tool_policy_deny_list() {
        let policy = ToolPolicy::deny_list(vec!["bash".to_string(), "python".to_string()]);

        assert!(!policy.is_explicitly_allowed("read_file"));
        assert!(!policy.is_explicitly_allowed("bash"));
        assert!(!policy.is_explicitly_allowed("python"));
        assert!(policy.allow.is_none());
    }

    #[test]
    fn test_engine_default_is_unrestricted() {
        let engine = ToolPolicyEngine::new();
        assert!(engine.is_allowed("any-agent", "any-tool").is_ok());
    }

    #[test]
    fn test_global_deny_list_blocks_tools() {
        let mut engine = ToolPolicyEngine::new();
        engine.set_global_policy(ToolPolicy::deny_list(vec![
            "bash".to_string(),
            "python".to_string(),
        ]));

        assert!(engine.is_allowed("agent-1", "read_file").is_ok());
        assert!(engine.is_allowed("agent-1", "bash").is_err());
        assert!(engine.is_allowed("agent-1", "python").is_err());
    }

    #[test]
    fn test_global_allow_list_allows_only_whitelisted_tools() {
        let mut engine = ToolPolicyEngine::new();
        engine.set_global_policy(ToolPolicy::allow_list(vec![
            "read_file".to_string(),
            "list_files".to_string(),
        ]));

        assert!(engine.is_allowed("agent-1", "read_file").is_ok());
        assert!(engine.is_allowed("agent-1", "list_files").is_ok());
        assert!(engine.is_allowed("agent-1", "bash").is_err());
    }

    #[test]
    fn test_agent_policy_overrides_global_deny() {
        let mut engine = ToolPolicyEngine::new();
        engine.set_global_policy(ToolPolicy::deny_list(vec!["bash".to_string()]));

        // Agent-1 uses global policy - bash is denied
        assert!(engine.is_allowed("agent-1", "bash").is_err());

        // Agent-2 has override that allows bash
        engine.set_agent_policy(
            "agent-2".to_string(),
            ToolPolicy::allow_list(vec!["bash".to_string()]),
        );
        assert!(engine.is_allowed("agent-2", "bash").is_ok());
    }

    #[test]
    fn test_agent_deny_overrides_global_allow() {
        let mut engine = ToolPolicyEngine::new();
        engine.set_global_policy(ToolPolicy::allow_list(vec![
            "read_file".to_string(),
            "bash".to_string(),
        ]));

        // Agent-1 uses global policy - both tools allowed
        assert!(engine.is_allowed("agent-1", "read_file").is_ok());
        assert!(engine.is_allowed("agent-1", "bash").is_ok());

        // Agent-2 has deny list that blocks bash even though global allows it
        engine.set_agent_policy(
            "agent-2".to_string(),
            ToolPolicy::deny_list(vec!["bash".to_string()]),
        );
        assert!(engine.is_allowed("agent-2", "read_file").is_ok());
        assert!(engine.is_allowed("agent-2", "bash").is_err());
    }

    #[test]
    fn test_remove_agent_policy_falls_back_to_global() {
        let mut engine = ToolPolicyEngine::new();
        engine.set_global_policy(ToolPolicy::deny_list(vec!["bash".to_string()]));

        // Set agent policy - agent-1 is now allowed bash
        engine.set_agent_policy(
            "agent-1".to_string(),
            ToolPolicy::allow_list(vec!["bash".to_string()]),
        );
        assert!(engine.is_allowed("agent-1", "bash").is_ok());

        // Remove agent policy - should fall back to global (deny)
        engine.remove_agent_policy("agent-1");
        // After removing agent policy, agent-1 falls back to global deny policy
        // so bash should now be denied
        assert!(engine.is_allowed("agent-1", "bash").is_err());
    }

    #[test]
    fn test_deny_takes_precedence_over_allow() {
        let mut engine = ToolPolicyEngine::new();
        let policy = ToolPolicy {
            allow: Some(vec!["read_file".to_string(), "bash".to_string()]),
            deny: Some(vec!["bash".to_string()]),
        };
        engine.set_global_policy(policy);

        // bash is in both lists, but deny takes precedence
        assert!(engine.is_allowed("agent-1", "read_file").is_ok());
        assert!(engine.is_allowed("agent-1", "bash").is_err());
    }

    #[test]
    fn test_deny_precedence_message() {
        let mut engine = ToolPolicyEngine::new();
        engine.set_global_policy(ToolPolicy::deny_list(vec!["bash".to_string()]));

        let result = engine.is_allowed("agent-1", "bash");
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("bash"));
        assert!(error_msg.contains("global"));
    }

    #[test]
    fn test_agent_deny_message() {
        let mut engine = ToolPolicyEngine::new();
        engine.set_global_policy(ToolPolicy::new());
        engine.set_agent_policy(
            "agent-1".to_string(),
            ToolPolicy::deny_list(vec!["bash".to_string()]),
        );

        let result = engine.is_allowed("agent-1", "bash");
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("bash"));
        assert!(error_msg.contains("agent-1"));
    }
}
