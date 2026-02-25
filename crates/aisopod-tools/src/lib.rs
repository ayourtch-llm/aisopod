//! # aisopod-tools
//!
//! Tool abstractions, registries, and implementations for function-calling and tool-use capabilities.
//!
//! ## Core Types
//!
//! This crate defines the foundational types for the tool subsystem:
//!
//! - [`Tool`]: The main async trait that all tools must implement.
//! - [`ToolContext`]: Context information passed to tool execution.
//! - [`ToolResult`]: The result returned by tool execution.
//! - [`ToolRegistry`]: Central registry for managing registered tools.
//!
//! ## Schema Normalization
//!
//! This crate also provides tool schema normalization functions:
//!
//! - [`schema::ToolDefinition`]: The internal tool definition type.
//! - [`schema::to_anthropic_format`]: Convert to Anthropic's tool format.
//! - [`schema::to_openai_format`]: Convert to OpenAI's function calling format.
//! - [`schema::to_gemini_format`]: Convert to Gemini's function declaration format.
//! - Batch conversion functions for arrays of tools.
//!
//! ## Example
//!
//! ```ignore
//! use aisopod_tools::{Tool, ToolContext, ToolResult, ToolRegistry};
//! use serde_json::json;
//!
//! struct MyTool;
//!
//! #[async_trait]
//! impl Tool for MyTool {
//!     fn name(&self) -> &str {
//!         "my_tool"
//!     }
//!
//!     fn description(&self) -> &str {
//!         "A description of what this tool does"
//!     }
//!
//!     fn parameters_schema(&self) -> serde_json::Value {
//!         json!({
//!             "type": "object",
//!             "properties": {},
//!             "required": []
//!         })
//!     }
//!
//!     async fn execute(
//!         &self,
//!         _params: serde_json::Value,
//!         _ctx: &ToolContext,
//!     ) -> Result<ToolResult> {
//!         Ok(ToolResult {
//!             content: "Tool executed successfully".to_string(),
//!             is_error: false,
//!             metadata: None,
//!         })
//!     }
//! }
//! ```

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub mod policy;
pub use policy::{ToolPolicy, ToolPolicyEngine};

pub mod registry;
pub use registry::ToolRegistry;

pub mod schema;
pub use schema::{
    to_anthropic_batch, to_anthropic_format, to_gemini_batch, to_gemini_format, to_openai_batch,
    to_openai_format, ToolDefinition,
};

pub mod approval;
pub use approval::{
    is_auto_approved, ApprovalError, ApprovalHandler, ApprovalRequest, ApprovalResponse,
    ApprovalState, ApprovalStateTracker, ApprovalSummary, NoOpApprovalHandler, RiskLevel,
};

pub mod builtins;
pub use builtins::{
    BashTool, CanvasRenderer, CanvasTool, CronTool, FileTool, InMemoryCanvasRenderer, JobScheduler,
    MessageSender, MessageTool, NoOpAgentSpawner, NoOpJobScheduler, NoOpMessageSender,
    NoOpSessionManager, ScheduledJob, SessionManager, SessionTool, SubagentTool,
};

pub mod sandbox;
pub use sandbox::config;
pub use aisopod_config::types::{SandboxConfig, SandboxRuntime, WorkspaceAccess};
pub use sandbox::{ContainerId, ExecutionResult, SandboxExecutor, WorkspaceError, WorkspaceGuard};

/// Registers all built-in tools with the given registry.
pub fn register_all_tools(registry: &mut ToolRegistry) {
    registry.register(Arc::new(BashTool::default()));
    registry.register(Arc::new(CanvasTool::with_in_memory()));
    registry.register(Arc::new(CronTool::with_noop_scheduler()));
    registry.register(Arc::new(FileTool::new()));
    registry.register(Arc::new(MessageTool::new(Arc::new(NoOpMessageSender))));
    registry.register(Arc::new(SubagentTool::new(
        Arc::new(NoOpAgentSpawner),
        3,
        None,
    )));
    registry.register(Arc::new(SessionTool::new(Arc::new(NoOpSessionManager))));
}

/// The result of a tool execution.
///
/// Contains the textual content returned to the AI model, an error flag,
/// and optional metadata for internal use.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// The textual content returned to the AI model.
    pub content: String,
    /// Indicates whether the tool call resulted in an error.
    pub is_error: bool,
    /// Optional structured metadata for internal use.
    pub metadata: Option<serde_json::Value>,
}

impl ToolResult {
    /// Creates a successful result with the given content.
    pub fn success(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            is_error: false,
            metadata: None,
        }
    }

    /// Creates an error result with the given message.
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            content: message.into(),
            is_error: true,
            metadata: None,
        }
    }

    /// Sets the metadata and returns the updated result.
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Context information passed to tool execution.
///
/// Contains information about the agent, session, and environment
/// that the tool is executing within.
#[derive(Clone)]
pub struct ToolContext {
    /// The unique identifier of the agent executing the tool.
    pub agent_id: String,
    /// The session key identifying the current conversation session.
    pub session_key: String,
    /// Optional path to the workspace directory for the tool to operate in.
    pub workspace_path: Option<PathBuf>,
    /// Optional sandbox configuration for isolated execution.
    pub sandbox_config: Option<SandboxConfig>,
    /// Optional handler for requesting user approvals before execution.
    pub approval_handler: Option<Arc<dyn ApprovalHandler>>,
    /// Optional metadata for tool execution context.
    pub metadata: Option<serde_json::Value>,
}

impl ToolContext {
    /// Creates a new ToolContext with the given agent and session identifiers.
    pub fn new(agent_id: impl Into<String>, session_key: impl Into<String>) -> Self {
        Self {
            agent_id: agent_id.into(),
            session_key: session_key.into(),
            workspace_path: None,
            sandbox_config: None,
            approval_handler: None,
            metadata: None,
        }
    }

    /// Sets the workspace path.
    pub fn with_workspace_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.workspace_path = Some(path.into());
        self
    }

    /// Sets the sandbox configuration.
    pub fn with_sandbox_config(mut self, config: SandboxConfig) -> Self {
        self.sandbox_config = Some(config);
        self
    }

    /// Sets the approval handler.
    pub fn with_approval_handler(mut self, handler: Arc<dyn ApprovalHandler>) -> Self {
        self.approval_handler = Some(handler);
        self
    }

    /// Sets the metadata for tool execution.
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Gets a value from metadata by key.
    pub fn metadata_get(&self, key: &str) -> Option<serde_json::Value> {
        self.metadata.as_ref().and_then(|m| m.get(key).cloned())
    }
}

/// The main async trait that all tools must implement.
///
/// This trait defines the interface for all tools in the system, whether
/// built-in or provided via plugins. Implementations are expected to be
/// stateless or share only immutable state.
#[async_trait]
pub trait Tool: Send + Sync {
    /// Returns the unique name of this tool.
    ///
    /// This name is used to identify the tool in tool calls and should be
    /// machine-readable (e.g., snake_case).
    fn name(&self) -> &str;

    /// Returns a human-readable description of what this tool does.
    ///
    /// This description is used by the AI model to understand when and
    /// how to use the tool.
    fn description(&self) -> &str;

    /// Returns the JSON schema describing the tool's expected parameters.
    ///
    /// The schema should follow the JSON Schema specification and describe
    /// the structure of the parameters object that will be passed to `execute`.
    fn parameters_schema(&self) -> serde_json::Value;

    /// Executes the tool with the given parameters and context.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters for the tool, validated against the schema.
    /// * `ctx` - The execution context containing agent, session, and environment info.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `ToolResult` on success, or an error if
    /// the tool execution failed.
    async fn execute(&self, params: serde_json::Value, ctx: &ToolContext) -> Result<ToolResult>;
}
