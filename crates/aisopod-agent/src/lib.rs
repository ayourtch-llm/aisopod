//! # aisopod-agent
//!
//! Core agent loop, orchestration logic, and agent lifecycle management.

pub mod binding;
pub mod compaction;
pub mod context_guard;
pub mod failover;
pub mod pipeline;
pub mod prompt;
pub mod resolution;
pub mod runner;
pub mod subagent;
pub mod transcript;
pub mod types;

// Re-export key types from crate root
pub use binding::{AgentBinding, BindingMatch, PeerMatch};
pub use compaction::{compact_messages, estimate_token_count, select_strategy, CompactionSeverity, CompactionStrategy};
pub use context_guard::ContextWindowGuard;
pub use failover::{classify_error, execute_with_failover, FailoverAction, FailoverState, ModelAttempt};
pub use pipeline::{AgentPipeline, AgentRunStream};
pub use prompt::{PromptSection, SystemPromptBuilder};
pub use resolution::{
    list_agent_ids, resolve_agent_config, resolve_agent_model, resolve_session_agent_id,
    ModelChain, ResolutionConfig,
};
pub use runner::{AgentRunner, SubagentRunnerExt};
pub use subagent::{spawn_subagent, ResourceBudget, SubagentSpawnParams};
pub use transcript::{repair_transcript, ProviderKind};
pub use types::{AgentEvent, AgentRunParams, AgentRunResult, SessionMetadata, UsageReport};
