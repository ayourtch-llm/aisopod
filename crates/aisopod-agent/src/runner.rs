//! Agent runner for executing agent operations.
//!
//! This module provides the `AgentRunner` struct which orchestrates
//! agent execution using configuration, provider registry, tool registry,
//! and session store.

use std::sync::Arc;

use anyhow::Result;
use tokio::sync::broadcast;

use crate::types::{AgentEvent, AgentRunParams, AgentRunResult};
use crate::resolution;

/// Extension trait for AgentRunner to support subagent spawning.
pub trait SubagentRunnerExt {
    /// Gets the maximum subagent depth from config.
    fn get_max_subagent_depth(&self) -> usize;

    /// Validates a model against the subagent allowlist for a given agent.
    fn validate_model_allowlist(&self, agent_id: &str, model: &str) -> Result<()>;

    /// Extracts the resource budget from the config (if any).
    fn get_resource_budget(&self) -> Option<crate::subagent::ResourceBudget>;
}

/// The central struct for running agents.
///
/// `AgentRunner` holds the necessary dependencies for agent execution:
/// - Configuration for agent behavior
/// - Provider registry for model access
/// - Tool registry for tool execution
/// - Session store for conversation state
///
/// # Example
///
/// ```ignore
/// use std::sync::Arc;
/// use aisopod_agent::{AgentRunner, AgentRunParams};
/// use aisopod_config::AisopodConfig;
/// use aisopod_provider::ProviderRegistry;
/// use aisopod_tools::ToolRegistry;
/// use aisopod_session::SessionStore;
///
/// async fn example() -> anyhow::Result<()> {
///     let config = Arc::new(AisopodConfig::default());
///     let providers = Arc::new(ProviderRegistry::new());
///     let tools = Arc::new(ToolRegistry::new());
///     let sessions = Arc::new(SessionStore::new());
///
///     let runner = AgentRunner::new(config, providers, tools, sessions);
///
///     // Run an agent
///     let params = AgentRunParams::new(
///         "session_123",
///         vec![],
///         Some("my-agent")
///     );
///     let stream = runner.run(params).await?;
///
///     Ok(())
/// }
/// ```
pub struct AgentRunner {
    /// The agent configuration.
    config: Arc<aisopod_config::AisopodConfig>,
    /// The provider registry for model access.
    providers: Arc<aisopod_provider::ProviderRegistry>,
    /// The tool registry for tool execution.
    tools: Arc<aisopod_tools::ToolRegistry>,
    /// The session store for conversation state.
    sessions: Arc<aisopod_session::SessionStore>,
}

impl AgentRunner {
    /// Creates a new `AgentRunner` with the given dependencies.
    ///
    /// # Arguments
    ///
    /// * `config` - The agent configuration.
    /// * `providers` - The provider registry for model access.
    /// * `tools` - The tool registry for tool execution.
    /// * `sessions` - The session store for conversation state.
    pub fn new(
        config: Arc<aisopod_config::AisopodConfig>,
        providers: Arc<aisopod_provider::ProviderRegistry>,
        tools: Arc<aisopod_tools::ToolRegistry>,
        sessions: Arc<aisopod_session::SessionStore>,
    ) -> Self {
        Self {
            config,
            providers,
            tools,
            sessions,
        }
    }

    /// Gets the agent configuration.
    pub fn config(&self) -> &Arc<aisopod_config::AisopodConfig> {
        &self.config
    }

    /// Gets the provider registry.
    pub fn providers(&self) -> &Arc<aisopod_provider::ProviderRegistry> {
        &self.providers
    }

    /// Gets the tool registry.
    pub fn tools(&self) -> &Arc<aisopod_tools::ToolRegistry> {
        &self.tools
    }

    /// Gets the session store.
    pub fn sessions(&self) -> &Arc<aisopod_session::SessionStore> {
        &self.sessions
    }

    /// Runs an agent with the given parameters and returns the result directly.
    ///
    /// This method executes the pipeline and returns the final result
    /// instead of a stream. It's useful for spawning subagents where
    /// we need the final result to calculate resource budget deductions.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters for the agent run.
    ///
    /// # Returns
    ///
    /// Returns the final result of the agent run, or an error if
    /// the run failed.
    pub async fn run_and_get_result(&self, params: AgentRunParams) -> Result<AgentRunResult> {
        let pipeline = crate::pipeline::AgentPipeline::new(
            self.config.clone(),
            self.providers.clone(),
            self.tools.clone(),
            self.sessions.clone(),
        );
        // Create a dummy event channel that we ignore
        let (event_tx, _) = tokio::sync::mpsc::channel(100);
        pipeline.execute(&params, &event_tx).await
    }

    /// Runs an agent with the given parameters.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters for the agent run.
    ///
    /// # Returns
    ///
    /// Returns a stream of agent events on success, or an error if
    /// the run failed to start.
    pub async fn run(&self, params: AgentRunParams) -> Result<crate::pipeline::AgentRunStream> {
        // Create a channel for streaming events
        let (event_tx, event_rx) = tokio::sync::mpsc::channel(100);

        // Clone the pipeline dependencies
        let config = self.config.clone();
        let providers = self.providers.clone();
        let tools = self.tools.clone();
        let sessions = self.sessions.clone();

        // Spawn the pipeline execution
        tokio::spawn(async move {
            let pipeline = crate::pipeline::AgentPipeline::new(config, providers, tools, sessions);
            if let Err(e) = pipeline.execute(&params, &event_tx).await {
                let _ = event_tx.send(crate::types::AgentEvent::Error {
                    message: e.to_string(),
                }).await;
            }
        });

        // Return the stream
        Ok(crate::pipeline::AgentRunStream::new(event_rx))
    }

    /// Subscribes to agent events for a session.
    ///
    /// # Arguments
    ///
    /// * `session_key` - The session key to subscribe to.
    ///
    /// # Returns
    ///
    /// Returns a receiver for agent events.
    pub fn subscribe(&self, _session_key: &str) -> broadcast::Receiver<AgentEvent> {
        // TODO: Implement actual subscription mechanism using broadcast channel
        // For now, return a stub receiver
        let (_tx, rx) = broadcast::channel(1);
        rx
    }

    /// Aborts the agent run for the given session.
    ///
    /// # Arguments
    ///
    /// * `session_key` - The session key to abort.
    pub async fn abort(&self, _session_key: &str) -> Result<()> {
        // TODO: Implement actual abort mechanism
        // For now, this is a stub
        Ok(())
    }
}

impl SubagentRunnerExt for AgentRunner {
    fn get_max_subagent_depth(&self) -> usize {
        // Use the global config default - need to check each agent
        // For now, return a safe default
        3
    }

    fn validate_model_allowlist(&self, agent_id: &str, model: &str) -> Result<()> {
        // Get the agent config to check its allowlist
        let agent_config = resolution::resolve_agent_config(self.config(), agent_id)
            .map_err(|e| anyhow::anyhow!("Failed to resolve agent config: {}", e))?;

        if let Some(ref allowlist) = agent_config.subagent_allowed_models {
            if !allowlist.contains(&model.to_string()) {
                return Err(anyhow::anyhow!(
                    "Model '{}' is not in the allowlist for agent '{}'",
                    model,
                    agent_id
                ));
            }
        }

        Ok(())
    }

    fn get_resource_budget(&self) -> Option<crate::subagent::ResourceBudget> {
        // For now, return None - resource budget would need to be configured
        // per-agent in the config
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_runner_new() {
        let config = Arc::new(aisopod_config::AisopodConfig::default());
        let providers = Arc::new(aisopod_provider::ProviderRegistry::new());
        let tools = Arc::new(aisopod_tools::ToolRegistry::new());
        let sessions = Arc::new(aisopod_session::SessionStore::new());

        let runner = AgentRunner::new(config, providers, tools, sessions);

        // Just verify it compiles - full tests will be added in subsequent issues
        assert_eq!(runner.config.meta.version, "1.0");
    }
}
