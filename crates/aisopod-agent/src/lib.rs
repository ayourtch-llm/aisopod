//! # aisopod-agent
//!
//! Core agent loop, orchestration logic, and agent lifecycle management.

pub mod runner;
pub mod types;

// Re-export key types from crate root
pub use runner::AgentRunner;
pub use types::{AgentEvent, AgentRunParams, AgentRunResult, UsageReport};
