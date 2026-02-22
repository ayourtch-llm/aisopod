//! Agent runner for executing agent operations.
//!
//! This module provides the `AgentRunner` struct which orchestrates
//! agent execution using configuration, provider registry, tool registry,
//! and session store.

use std::sync::Arc;

use anyhow::Result;
use tokio::sync::broadcast;

use crate::abort::{AbortHandle, AbortRegistry};
use crate::memory::{inject_memory_context, MemoryConfig};
use crate::resolution;
use crate::types::{AgentEvent, AgentRunParams, AgentRunResult};
use aisopod_memory::{MemoryManager, MemoryQueryPipeline};

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
/// - Abort registry for tracking and cancelling active sessions
/// - Optional memory components for memory integration
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
    /// Optional usage tracker for recording token usage.
    usage_tracker: Option<Arc<crate::usage::UsageTracker>>,
    /// Registry for tracking active sessions and their abort handles
    abort_registry: Arc<AbortRegistry>,
    /// Optional memory query pipeline for pre-run memory injection
    memory_pipeline: Option<Arc<MemoryQueryPipeline>>,
    /// Optional memory manager for post-run memory extraction
    memory_manager: Option<Arc<MemoryManager>>,
    /// Memory configuration
    memory_config: MemoryConfig,
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
            usage_tracker: None,
            abort_registry: Arc::new(AbortRegistry::new()),
            memory_pipeline: None,
            memory_manager: None,
            memory_config: MemoryConfig::default(),
        }
    }

    /// Creates a new `AgentRunner` with the given dependencies and usage tracker.
    ///
    /// # Arguments
    ///
    /// * `config` - The agent configuration.
    /// * `providers` - The provider registry for model access.
    /// * `tools` - The tool registry for tool execution.
    /// * `sessions` - The session store for conversation state.
    /// * `usage_tracker` - The usage tracker for recording token usage.
    pub fn new_with_usage_tracker(
        config: Arc<aisopod_config::AisopodConfig>,
        providers: Arc<aisopod_provider::ProviderRegistry>,
        tools: Arc<aisopod_tools::ToolRegistry>,
        sessions: Arc<aisopod_session::SessionStore>,
        usage_tracker: Arc<crate::usage::UsageTracker>,
    ) -> Self {
        Self {
            config,
            providers,
            tools,
            sessions,
            usage_tracker: Some(usage_tracker),
            abort_registry: Arc::new(AbortRegistry::new()),
            memory_pipeline: None,
            memory_manager: None,
            memory_config: MemoryConfig::default(),
        }
    }

    /// Creates a new `AgentRunner` with memory integration.
    ///
    /// # Arguments
    ///
    /// * `config` - The agent configuration.
    /// * `providers` - The provider registry for model access.
    /// * `tools` - The tool registry for tool execution.
    /// * `sessions` - The session store for conversation state.
    /// * `memory_pipeline` - The memory query pipeline for querying memories.
    /// * `memory_manager` - The memory manager for storing memories.
    pub fn new_with_memory(
        config: Arc<aisopod_config::AisopodConfig>,
        providers: Arc<aisopod_provider::ProviderRegistry>,
        tools: Arc<aisopod_tools::ToolRegistry>,
        sessions: Arc<aisopod_session::SessionStore>,
        memory_pipeline: Arc<MemoryQueryPipeline>,
        memory_manager: Arc<MemoryManager>,
    ) -> Self {
        // Register the memory tool with the tools registry
        let memory_tool = Arc::new(crate::memory::MemoryTool::new(
            memory_pipeline.clone(),
            memory_manager.clone(),
        ));
        let mut tools_ref = Arc::into_inner(tools).unwrap();
        tools_ref.register(memory_tool);
        let tools = Arc::new(tools_ref);

        Self {
            config,
            providers,
            tools,
            sessions,
            usage_tracker: None,
            abort_registry: Arc::new(AbortRegistry::new()),
            memory_pipeline: Some(memory_pipeline),
            memory_manager: Some(memory_manager),
            memory_config: MemoryConfig::default(),
        }
    }

    /// Creates a new `AgentRunner` with memory integration and usage tracker.
    ///
    /// # Arguments
    ///
    /// * `config` - The agent configuration.
    /// * `providers` - The provider registry for model access.
    /// * `tools` - The tool registry for tool execution.
    /// * `sessions` - The session store for conversation state.
    /// * `memory_pipeline` - The memory query pipeline for querying memories.
    /// * `memory_manager` - The memory manager for storing memories.
    /// * `usage_tracker` - The usage tracker for recording token usage.
    pub fn new_with_memory_and_usage_tracker(
        config: Arc<aisopod_config::AisopodConfig>,
        providers: Arc<aisopod_provider::ProviderRegistry>,
        tools: Arc<aisopod_tools::ToolRegistry>,
        sessions: Arc<aisopod_session::SessionStore>,
        memory_pipeline: Arc<MemoryQueryPipeline>,
        memory_manager: Arc<MemoryManager>,
        usage_tracker: Arc<crate::usage::UsageTracker>,
    ) -> Self {
        // Register the memory tool with the tools registry
        let memory_tool = Arc::new(crate::memory::MemoryTool::new(
            memory_pipeline.clone(),
            memory_manager.clone(),
        ));
        let mut tools_ref = Arc::into_inner(tools).unwrap();
        tools_ref.register(memory_tool);
        let tools = Arc::new(tools_ref);

        Self {
            config,
            providers,
            tools,
            sessions,
            usage_tracker: Some(usage_tracker),
            abort_registry: Arc::new(AbortRegistry::new()),
            memory_pipeline: Some(memory_pipeline),
            memory_manager: Some(memory_manager),
            memory_config: MemoryConfig::default(),
        }
    }

    /// Returns true if usage tracking is enabled.
    pub fn has_usage_tracker(&self) -> bool {
        self.usage_tracker.is_some()
    }

    /// Gets the usage tracker if enabled.
    pub fn usage_tracker(&self) -> Option<&Arc<crate::usage::UsageTracker>> {
        self.usage_tracker.as_ref()
    }

    /// Gets the abort registry.
    pub fn abort_registry(&self) -> &Arc<AbortRegistry> {
        &self.abort_registry
    }

    /// Returns true if memory integration is enabled.
    pub fn has_memory(&self) -> bool {
        self.memory_pipeline.is_some() && self.memory_manager.is_some()
    }

    /// Gets the memory pipeline if enabled.
    pub fn memory_pipeline(&self) -> Option<&Arc<MemoryQueryPipeline>> {
        self.memory_pipeline.as_ref()
    }

    /// Gets the memory manager if enabled.
    pub fn memory_manager(&self) -> Option<&Arc<MemoryManager>> {
        self.memory_manager.as_ref()
    }

    /// Gets the memory configuration.
    pub fn memory_config(&self) -> &MemoryConfig {
        &self.memory_config
    }

    /// Sets the memory configuration.
    pub fn with_memory_config(mut self, config: MemoryConfig) -> Self {
        self.memory_config = config;
        self
    }

    /// Registers an active session with its abort handle.
    ///
    /// # Arguments
    ///
    /// * `session_key` - The session key to register.
    /// * `handle` - The abort handle for this session.
    pub fn register_active_session(
        &self,
        session_key: &str,
        handle: AbortHandle,
    ) -> Option<AbortHandle> {
        self.abort_registry.insert(session_key, handle)
    }

    /// Aborts the agent run for the given session.
    ///
    /// This will cancel the execution of the agent for the specified session.
    ///
    /// # Arguments
    ///
    /// * `session_key` - The session key to abort.
    pub async fn abort(&self, session_key: &str) -> Result<()> {
        if let Some(handle) = self.abort_registry.get(session_key) {
            handle.abort();
            Ok(())
        } else {
            // Session not found, still return Ok (idempotent)
            Ok(())
        }
    }
}

impl AgentRunner {
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
        let pipeline = if self.has_memory() {
            // Use memory-enabled pipeline
            let memory_pipeline = self.memory_pipeline.clone().unwrap();
            let memory_manager = self.memory_manager.clone().unwrap();
            if let Some(ref tracker) = self.usage_tracker {
                crate::pipeline::AgentPipeline::new_with_memory_and_usage_tracker(
                    self.config.clone(),
                    self.providers.clone(),
                    self.tools.clone(),
                    self.sessions.clone(),
                    memory_pipeline,
                    memory_manager,
                    tracker.clone(),
                )
            } else {
                crate::pipeline::AgentPipeline::new_with_memory(
                    self.config.clone(),
                    self.providers.clone(),
                    self.tools.clone(),
                    self.sessions.clone(),
                    memory_pipeline,
                    memory_manager,
                )
            }
        } else if let Some(ref tracker) = self.usage_tracker {
            // Use pipeline with usage tracker only
            crate::pipeline::AgentPipeline::new_with_usage_tracker(
                self.config.clone(),
                self.providers.clone(),
                self.tools.clone(),
                self.sessions.clone(),
                tracker.clone(),
            )
        } else {
            // Use basic pipeline
            crate::pipeline::AgentPipeline::new(
                self.config.clone(),
                self.providers.clone(),
                self.tools.clone(),
                self.sessions.clone(),
            )
        };
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

        // Clone memory components (if any)
        let memory_pipeline = self.memory_pipeline.clone();
        let memory_manager = self.memory_manager.clone();
        let usage_tracker = self.usage_tracker.clone();

        // Spawn the pipeline execution
        tokio::spawn(async move {
            let pipeline = if let (Some(memory_pipeline), Some(memory_manager)) =
                (memory_pipeline, memory_manager)
            {
                // Use memory-enabled pipeline
                if let Some(tracker) = usage_tracker {
                    crate::pipeline::AgentPipeline::new_with_memory_and_usage_tracker(
                        config,
                        providers,
                        tools,
                        sessions,
                        memory_pipeline,
                        memory_manager,
                        tracker,
                    )
                } else {
                    crate::pipeline::AgentPipeline::new_with_memory(
                        config,
                        providers,
                        tools,
                        sessions,
                        memory_pipeline,
                        memory_manager,
                    )
                }
            } else if let Some(tracker) = usage_tracker {
                crate::pipeline::AgentPipeline::new_with_usage_tracker(
                    config, providers, tools, sessions, tracker,
                )
            } else {
                crate::pipeline::AgentPipeline::new(config, providers, tools, sessions)
            };
            if let Err(e) = pipeline.execute(&params, &event_tx).await {
                let _ = event_tx
                    .send(crate::types::AgentEvent::Error {
                        message: e.to_string(),
                    })
                    .await;
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
    use aisopod_session::SessionStore;
    use aisopod_memory::{MemoryManager, MemoryManagerConfig, MemoryQueryPipeline, MockEmbeddingProvider};
    use aisopod_memory::sqlite::SqliteMemoryStore;
    use tempfile::tempdir;

    #[test]
    fn test_agent_runner_new() {
        let config = Arc::new(aisopod_config::AisopodConfig::default());
        let providers = Arc::new(aisopod_provider::ProviderRegistry::new());
        let tools = Arc::new(aisopod_tools::ToolRegistry::new());
        let sessions = Arc::new(
            SessionStore::new_in_memory().expect("Failed to create in-memory session store"),
        );

        let runner = AgentRunner::new(config, providers, tools, sessions);

        // Just verify it compiles - full tests will be added in subsequent issues
        assert_eq!(runner.config.meta.version, "1.0");
    }

    #[test]
    fn test_agent_runner_memory_tool_registered() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let store = SqliteMemoryStore::new(db_path.to_str().unwrap(), 4).unwrap();
        let store = Arc::new(store) as Arc<dyn aisopod_memory::MemoryStore>;
        let embedder = Arc::new(MockEmbeddingProvider::new(4));
        
        let pipeline = Arc::new(MemoryQueryPipeline::new(store.clone(), embedder.clone()));
        let manager = Arc::new(MemoryManager::new(store, embedder, MemoryManagerConfig::default()));
        
        let config = Arc::new(aisopod_config::AisopodConfig::default());
        let providers = Arc::new(aisopod_provider::ProviderRegistry::new());
        let mut tools = aisopod_tools::ToolRegistry::new();
        aisopod_tools::register_all_tools(&mut tools);
        let tools = Arc::new(tools);
        let sessions = Arc::new(
            SessionStore::new_in_memory().expect("Failed to create in-memory session store"),
        );

        let runner = AgentRunner::new_with_memory(
            config,
            providers,
            tools,
            sessions,
            pipeline,
            manager,
        );

        // Verify memory tool is registered
        let tools = runner.tools();
        let registered_tools = tools.list();
        assert!(registered_tools.contains(&"memory".to_string()), 
                "Memory tool should be registered. Registered tools: {:?}", registered_tools);
    }
}
