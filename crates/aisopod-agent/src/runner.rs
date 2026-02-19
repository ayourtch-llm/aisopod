//! Agent runner for executing agent operations.
//!
//! This module provides the `AgentRunner` struct which orchestrates
//! agent execution using configuration, provider registry, tool registry,
//! and session store.

use std::sync::Arc;

use anyhow::Result;

use crate::pipeline::AgentPipeline;
use crate::types::{AgentEvent, AgentRunParams, AgentRunResult};

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
///         "my-agent"
///     );
///     let result = runner.run(params).await?;
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

    /// Runs an agent with the given parameters.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters for the agent run.
    ///
    /// # Returns
    ///
    /// Returns the result of the agent run on success, or an error if
    /// the run failed.
    ///
    /// This method:
    /// - Resolves the agent configuration
    /// - Selects the appropriate model
    /// - Prepares tools
    /// - Builds the system prompt
    /// - Repairs the transcript for provider requirements
    /// - Calls the model
    /// - Handles tool calls in a loop until completion
    /// - Streams events to subscribers
    /// - Returns the final result
    pub async fn run(&self, params: AgentRunParams) -> Result<AgentRunResult> {
        // Create the pipeline
        let pipeline = AgentPipeline::new(
            self.config.clone(),
            self.providers.clone(),
            self.tools.clone(),
            self.sessions.clone(),
        );

        // Create event channel for streaming
        let (event_tx, _event_rx) = tokio::sync::mpsc::unbounded_channel::<AgentEvent>();

        // Execute the pipeline
        pipeline.execute(&params, &event_tx).await
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
    ///
    /// This method creates a broadcast channel for streaming agent events
    /// to multiple subscribers.
    pub async fn subscribe(&self, _session_key: &str) -> tokio::sync::broadcast::Receiver<AgentEvent> {
        // Create a broadcast channel for events
        let (tx, rx) = tokio::sync::broadcast::channel(100);
        
        // Send a subscription event to the new subscriber
        let _ = tx.send(AgentEvent::Complete {
            result: crate::types::AgentRunResult::new(
                "Subscription established".to_string(),
                None,
                crate::types::UsageReport::new(0, 0),
            ),
        });
        
        rx
    }

    /// Aborts the agent run for the given session.
    ///
    /// # Arguments
    ///
    /// * `session_key` - The session key to abort.
    pub async fn abort(&self, _session_key: &str) {
        // TODO: Implement abort logic
        // This would involve signaling cancellation to any in-progress runs
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
