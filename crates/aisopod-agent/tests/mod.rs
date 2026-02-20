//! Integration tests for aisopod-agent crate.
//!
//! This module organizes all test modules for the agent engine.

// Test modules
pub mod abort;
pub mod compaction;
pub mod failover;
pub mod helpers;
pub mod pipeline;
pub mod prompt;
pub mod resolution;
pub mod subagent;
pub mod transcript;
pub mod usage;

// Re-export common test utilities
pub use helpers::{
    test_agent_run_params,
    test_agent_run_params_with_id,
    test_agent_run_result,
    collect_events,
    test_config,
    test_config_with_fallbacks,
    test_session_store,
    test_tool_registry,
    test_abort_registry,
    test_abort_handle,
};
