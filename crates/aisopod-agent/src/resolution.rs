//! Agent resolution system for determining which agent to use for a session.
//!
//! This module provides functionality to:
//! - Resolve the agent ID for a given session
//! - Resolve agent configuration by agent ID
//! - Resolve model configuration for an agent
//! - List all configured agent IDs

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::binding::{AgentBinding, BindingMatch, PeerMatch};

/// Resolution configuration for agent selection.
///
/// Contains the binding rules and default settings used for agent resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionConfig {
    /// List of agent bindings for routing
    #[serde(default)]
    pub bindings: Vec<AgentBinding>,
    /// Default agent ID to use when no bindings match
    #[serde(default)]
    pub default_agent: Option<String>,
    /// Whether to use priority-based binding selection
    #[serde(default = "default_true")]
    pub use_priority: bool,
}

fn default_true() -> bool {
    true
}

impl Default for ResolutionConfig {
    fn default() -> Self {
        Self {
            bindings: Vec::new(),
            default_agent: None,
            use_priority: true,
        }
    }
}

/// Resolves the agent ID for a given session.
///
/// Uses the provided bindings and session metadata to determine which
/// agent should handle the session. If no bindings match, returns the
/// default agent or an error if no default is configured.
///
/// # Arguments
///
/// * `config` - The aisopod configuration containing agents and bindings
/// * `session_key` - The session key to resolve
///
/// # Returns
///
/// Returns the resolved agent ID on success, or an error if resolution fails.
pub fn resolve_session_agent_id(
    config: &aisopod_config::AisopodConfig,
    session_key: &str,
) -> Result<String> {
    // TODO: In a real implementation, we would look up session metadata
    // from the session store. For now, we'll use the session_key as a
    // simple identifier and apply bindings.
    let _ = session_key;

    let bindings = &config.bindings;

    if bindings.is_empty() {
        // No bindings configured, return the first agent or an error
        if let Some(first_agent) = config.agents.agents.first() {
            return Ok(first_agent.id.clone());
        }
        return Err(anyhow!("No agents configured"));
    }

    // For now, return the agent with the highest priority binding
    // In a real implementation, we would evaluate session metadata here
    let highest_priority_binding = bindings
        .iter()
        .max_by_key(|b| b.priority)
        .ok_or_else(|| anyhow!("No bindings configured"))?;

    Ok(highest_priority_binding.agent_id.clone())
}

/// Resolves the agent configuration by agent ID.
///
/// Searches through the configured agents to find the one with the
/// matching ID and returns its configuration.
///
/// # Arguments
///
/// * `config` - The aisopod configuration to search
/// * `agent_id` - The agent ID to resolve
///
/// # Returns
///
/// Returns the agent configuration on success, or an error if the agent is not found.
pub fn resolve_agent_config(
    config: &aisopod_config::AisopodConfig,
    agent_id: &str,
) -> Result<aisopod_config::types::Agent> {
    config
        .agents
        .agents
        .iter()
        .find(|a| a.id == agent_id)
        .cloned()
        .ok_or_else(|| anyhow!("Agent not found: {}", agent_id))
}

/// Resolves the model chain for a given agent.
///
/// A model chain represents the sequence of models to try for an agent,
/// including fallback models. This function resolves the primary model
/// and any fallback models for the specified agent.
///
/// # Arguments
///
/// * `config` - The aisopod configuration to search
/// * `agent_id` - The agent ID to resolve
///
/// # Returns
///
/// Returns the model chain configuration on success, or an error if resolution fails.
pub fn resolve_agent_model(
    config: &aisopod_config::AisopodConfig,
    agent_id: &str,
) -> Result<ModelChain> {
    let agent = resolve_agent_config(config, agent_id)?;
    
    // Get the primary model for this agent
    let primary_model = if !agent.model.is_empty() {
        agent.model
    } else if !config.agents.default.model.is_empty() {
        config.agents.default.model.clone()
    } else {
        return Err(anyhow!(
            "No model configured for agent '{}' or in defaults",
            agent_id
        ));
    };

    // Get fallback models from the models config
    let fallbacks: Vec<String> = config
        .models
        .fallbacks
        .iter()
        .filter_map(|f| {
            if f.primary == primary_model {
                Some(f.fallbacks.clone())
            } else {
                None
            }
        })
        .flatten()
        .collect();

    Ok(ModelChain {
        primary: primary_model,
        fallbacks,
    })
}

/// A chain of models for an agent.
///
/// Contains the primary model and fallback models to try in case of failure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelChain {
    /// The primary model to use
    pub primary: String,
    /// Fallback models to try if the primary fails
    #[serde(default)]
    pub fallbacks: Vec<String>,
}

impl ModelChain {
    /// Creates a new ModelChain with the given primary model.
    pub fn new(primary: impl Into<String>) -> Self {
        Self {
            primary: primary.into(),
            fallbacks: Vec::new(),
        }
    }

    /// Creates a new ModelChain with primary and fallback models.
    pub fn with_fallbacks(
        primary: impl Into<String>,
        fallbacks: Vec<String>,
    ) -> Self {
        Self {
            primary: primary.into(),
            fallbacks,
        }
    }

    /// Returns the primary model ID.
    pub fn primary(&self) -> &str {
        &self.primary
    }

    /// Returns the list of fallback model IDs.
    pub fn fallbacks(&self) -> &[String] {
        &self.fallbacks
    }

    /// Returns all models in the chain (primary first, then fallbacks).
    pub fn all_models(&self) -> Vec<String> {
        let mut all = vec![self.primary.clone()];
        all.extend(self.fallbacks.iter().cloned());
        all
    }
}

/// Lists all configured agent IDs.
///
/// Returns a vector of all agent IDs in the configuration.
///
/// # Arguments
///
/// * `config` - The aisopod configuration to search
///
/// # Returns
///
/// Returns a vector of agent IDs.
pub fn list_agent_ids(config: &aisopod_config::AisopodConfig) -> Vec<String> {
    config.agents.agents.iter().map(|a| a.id.clone()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolution_config_default() {
        let config = ResolutionConfig::default();
        assert!(config.bindings.is_empty());
        assert!(config.default_agent.is_none());
        assert!(config.use_priority);
    }

    #[test]
    fn test_resolution_config_with_priority() {
        let config = ResolutionConfig {
            bindings: Vec::new(),
            default_agent: Some("default_agent".to_string()),
            use_priority: false,
        };

        assert_eq!(config.default_agent, Some("default_agent".to_string()));
        assert!(!config.use_priority);
    }

    #[test]
    fn test_model_chain_new() {
        let chain = ModelChain::new("gpt-4");
        assert_eq!(chain.primary, "gpt-4");
        assert!(chain.fallbacks.is_empty());
    }

    #[test]
    fn test_model_chain_with_fallbacks() {
        let chain = ModelChain::with_fallbacks(
            "gpt-4",
            vec!["gpt-3.5-turbo".to_string(), "claude-3-opus".to_string()],
        );

        assert_eq!(chain.primary, "gpt-4");
        assert_eq!(chain.fallbacks.len(), 2);
        assert_eq!(chain.fallbacks()[0], "gpt-3.5-turbo");
        assert_eq!(chain.fallbacks()[1], "claude-3-opus");
    }

    #[test]
    fn test_model_chain_all_models() {
        let chain = ModelChain::with_fallbacks(
            "gpt-4",
            vec!["gpt-3.5-turbo".to_string()],
        );

        let all = chain.all_models();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0], "gpt-4");
        assert_eq!(all[1], "gpt-3.5-turbo");
    }

    // Note: The following tests require a full AisopodConfig instance
    // which is not easily constructed in unit tests without more setup.
    // These tests would be better suited for integration tests.

    #[test]
    fn test_list_agent_ids_empty() {
        let config = aisopod_config::AisopodConfig::default();
        let agent_ids = list_agent_ids(&config);
        assert!(agent_ids.is_empty());
    }

    #[test]
    fn test_resolve_agent_config_not_found() {
        let config = aisopod_config::AisopodConfig::default();
        let result = resolve_agent_config(&config, "nonexistent_agent");
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_agent_model_not_found() {
        let config = aisopod_config::AisopodConfig::default();
        let result = resolve_agent_model(&config, "nonexistent_agent");
        assert!(result.is_err());
    }
}
