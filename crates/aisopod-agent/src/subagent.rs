//! Subagent spawning functionality for the aisopod-agent crate.
//!
//! This module provides the ability to spawn child agents within a parent
//! agent's session, with support for depth limits, model allowlists,
//! thread ID propagation, and resource budget inheritance.

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::types::{AgentRunParams, AgentRunResult, UsageReport};
use crate::{AgentRunner, resolution};
use crate::runner::SubagentRunnerExt;

/// Parameters for spawning a subagent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentSpawnParams {
    /// The agent ID of the child agent to spawn.
    pub agent_id: String,
    /// The messages to pass to the child agent.
    pub messages: Vec<aisopod_provider::Message>,
    /// The session key of the parent agent (for thread ID propagation).
    pub parent_session_key: String,
    /// The current depth of the parent agent.
    pub parent_depth: usize,
    /// Optional thread ID from the parent (for context sharing).
    pub thread_id: Option<String>,
    /// Optional resource budget from the parent.
    pub resource_budget: Option<ResourceBudget>,
}

/// Resource budget for an agent run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceBudget {
    /// Maximum number of tokens allowed.
    pub max_tokens: usize,
    /// Remaining tokens available.
    pub remaining_tokens: usize,
}

impl ResourceBudget {
    /// Creates a new ResourceBudget with the given max and remaining tokens.
    pub fn new(max_tokens: usize, remaining_tokens: usize) -> Self {
        Self {
            max_tokens,
            remaining_tokens,
        }
    }

    /// Checks if the budget has enough tokens for the given usage.
    pub fn has_budget(&self, usage: usize) -> bool {
        self.remaining_tokens >= usage
    }

    /// Deducts the given usage from the budget and returns the remaining tokens.
    pub fn deduct(&mut self, usage: usize) -> Result<usize> {
        if self.remaining_tokens < usage {
            return Err(anyhow::anyhow!(
                "Insufficient budget: needed {}, remaining {}",
                usage,
                self.remaining_tokens
            ));
        }
        self.remaining_tokens -= usage;
        Ok(self.remaining_tokens)
    }
}

/// Spawns a subagent with the given parameters.
///
/// This function:
/// 1. Checks if the depth limit is exceeded
/// 2. Validates the requested agent's model against the allowlist
/// 3. Creates child AgentRunParams with incremented depth and inherited thread ID
/// 4. Calls runner.run_and_get_result() with the child params (returns final result)
/// 5. Deducts child usage from the parent's resource budget
/// 6. Returns the child's result and the updated budget
///
/// # Arguments
///
/// * `runner` - The agent runner to execute the subagent
/// * `params` - The parameters for spawning the subagent
///
/// # Returns
///
/// Returns a tuple of (child result, optional updated budget), or an error if spawning failed.
pub async fn spawn_subagent(
    runner: &AgentRunner,
    params: SubagentSpawnParams,
) -> Result<(AgentRunResult, Option<ResourceBudget>)> {
    // Step 1: Check depth limit
    let max_depth = runner.get_max_subagent_depth();
    if params.parent_depth + 1 > max_depth {
        return Err(anyhow::anyhow!(
            "Maximum subagent depth exceeded: parent depth {}, max {}",
            params.parent_depth,
            max_depth
        ));
    }

    // Step 2: Validate model against allowlist
    let model = resolution::resolve_agent_model(runner.config(), &params.agent_id)
        .map_err(|e| anyhow::anyhow!("Failed to resolve agent model: {}", e))?
        .primary;

    runner.validate_model_allowlist(&params.agent_id, &model)?;

    // Step 3: Create child AgentRunParams with incremented depth
    // Propagate thread_id from parent to child for context sharing
    let child_params = AgentRunParams::with_depth_and_thread_id(
        params.parent_session_key,
        params.messages,
        Some(params.agent_id.clone()),
        params.parent_depth + 1,
        params.thread_id,
    );

    // Step 4: Run the subagent and get result directly
    let child_result = runner.run_and_get_result(child_params).await?;

    // Step 5: Deduct child usage from parent's budget if budget was provided
    let updated_budget = if let Some(mut budget) = params.resource_budget {
        let child_tokens = child_result.usage.total_tokens as usize;
        
        // Check if we have enough budget before deducting
        if !budget.has_budget(child_tokens) {
            return Err(anyhow::anyhow!(
                "Insufficient budget for subagent: needed {} tokens, remaining {}",
                child_tokens,
                budget.remaining_tokens
            ));
        }

        // Deduct the usage and return the updated budget
        budget.deduct(child_tokens)?;
        Some(budget)
    } else {
        None
    };

    // Step 6: Return the child's result and updated budget
    Ok((child_result, updated_budget))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_resource_budget_new() {
        let budget = ResourceBudget::new(1000, 1000);
        assert_eq!(budget.max_tokens, 1000);
        assert_eq!(budget.remaining_tokens, 1000);
    }

    #[test]
    fn test_resource_budget_has_budget() {
        let budget = ResourceBudget::new(1000, 500);
        assert!(budget.has_budget(400));
        assert!(budget.has_budget(500));
        assert!(!budget.has_budget(600));
    }

    #[test]
    fn test_resource_budget_deduct() {
        let mut budget = ResourceBudget::new(1000, 500);
        let remaining = budget.deduct(200).unwrap();
        assert_eq!(remaining, 300);
        assert_eq!(budget.remaining_tokens, 300);
    }

    #[test]
    fn test_resource_budget_deduct_insufficient() {
        let mut budget = ResourceBudget::new(1000, 100);
        let result = budget.deduct(200);
        assert!(result.is_err());
        assert_eq!(budget.remaining_tokens, 100); // Should not deduct
    }

    #[test]
    fn test_subagent_spawn_params_new() {
        let params = SubagentSpawnParams {
            agent_id: "child_agent".to_string(),
            messages: vec![],
            parent_session_key: "parent_session".to_string(),
            parent_depth: 0,
            thread_id: None,
            resource_budget: None,
        };
        assert_eq!(params.agent_id, "child_agent");
        assert_eq!(params.parent_depth, 0);
    }

    #[test]
    fn test_subagent_spawn_params_with_thread_id() {
        let params = SubagentSpawnParams {
            agent_id: "child_agent".to_string(),
            messages: vec![],
            parent_session_key: "parent_session".to_string(),
            parent_depth: 0,
            thread_id: Some("thread_123".to_string()),
            resource_budget: Some(ResourceBudget::new(1000, 1000)),
        };
        assert_eq!(params.thread_id, Some("thread_123".to_string()));
    }

    #[test]
    fn test_agent_run_params_with_depth() {
        let params = AgentRunParams::with_depth(
            "session_123",
            vec![],
            Some("agent_1"),
            2,
        );
        assert_eq!(params.depth, 2);
        assert_eq!(params.agent_id, Some("agent_1".to_string()));
    }

    #[test]
    fn test_agent_run_params_default_depth() {
        let params = AgentRunParams::new(
            "session_123",
            vec![],
            Some("agent_1"),
        );
        assert_eq!(params.depth, 0);
    }

    #[test]
    fn test_resource_budget_with_max_and_remaining() {
        let budget = ResourceBudget::new(2000, 1500);
        assert_eq!(budget.max_tokens, 2000);
        assert_eq!(budget.remaining_tokens, 1500);
    }

    #[test]
    fn test_resource_budget_deduct_updates_remaining() {
        let mut budget = ResourceBudget::new(1000, 1000);
        let remaining = budget.deduct(300).unwrap();
        assert_eq!(remaining, 700);
        assert_eq!(budget.remaining_tokens, 700);
    }

    #[test]
    fn test_agent_run_params_with_depth_and_agent() {
        let params = AgentRunParams::with_depth(
            "session_xyz",
            vec![],
            Some("test_agent_1"),
            5,
        );
        assert_eq!(params.depth, 5);
        assert_eq!(params.session_key, "session_xyz");
        assert_eq!(params.agent_id, Some("test_agent_1".to_string()));
    }

    #[test]
    fn test_agent_run_params_with_thread_id() {
        let params = AgentRunParams::with_depth_and_thread_id(
            "session_123",
            vec![],
            Some("agent_1"),
            2,
            Some("thread_xyz"),
        );
        assert_eq!(params.depth, 2);
        assert_eq!(params.thread_id, Some("thread_xyz".to_string()));
    }

    #[test]
    fn test_agent_run_params_with_thread_id_none() {
        let params = AgentRunParams::with_depth_and_thread_id_str(
            "session_123",
            vec![],
            Some("agent_1"),
            1,
            None,
        );
        assert_eq!(params.depth, 1);
        assert_eq!(params.thread_id, None);
    }
}
