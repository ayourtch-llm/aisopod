//! Sandbox configuration module
//!
//! This module provides sandbox-related types for tool execution isolation.
//! It re-exports the main types from `aisopod-config` for convenience.

pub mod config;
pub mod executor;

pub use aisopod_config::types::{SandboxConfig, SandboxRuntime, WorkspaceAccess};
pub use executor::{ContainerId, ExecutionResult, SandboxExecutor};
