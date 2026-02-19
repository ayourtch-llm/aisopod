//! # aisopod-agent
//!
//! Core agent loop, orchestration logic, and agent lifecycle management.

pub mod binding;
pub mod prompt;
pub mod resolution;
pub mod runner;
pub mod types;

// Re-export key types from crate root
pub use binding::{AgentBinding, BindingMatch, PeerMatch};
pub use prompt::{PromptSection, SystemPromptBuilder};
pub use resolution::{
    list_agent_ids, resolve_agent_config, resolve_agent_model, resolve_session_agent_id,
    ModelChain, ResolutionConfig,
};
pub use runner::AgentRunner;
pub use types::{AgentEvent, AgentRunParams, AgentRunResult, SessionMetadata, UsageReport};
