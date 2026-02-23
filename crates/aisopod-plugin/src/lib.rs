//! # aisopod-plugin
//!
//! Plugin system, plugin loading, and plugin lifecycle management.
//!
//! ## Overview
//!
//! This crate provides the core types and traits for the aisopod plugin system.
//! All plugins must implement the [`Plugin`] trait and provide metadata via
//! [`PluginMeta`].
//!
//! ## Plugin Lifecycle
//!
//! Plugins go through a well-defined lifecycle:
//!
//! 1. **Construction**: Plugin instances are created
//! 2. **Metadata Query**: `Plugin::id()` and `Plugin::meta()` are called
//! 3. **Registration**: `Plugin::register()` is called to register capabilities
//! 4. **Initialization**: `Plugin::init()` is called with runtime context
//! 5. **Shutdown**: `Plugin::shutdown()` is called during system shutdown
//!
//! ## Example
//!
//! This example shows the basic structure of a plugin:
//!
//! ```ignore
//! use aisopod_plugin::{Plugin, PluginMeta, PluginContext, PluginApi};
//! use async_trait::async_trait;
//! use std::sync::Arc;
//!
//! #[derive(Debug)]
//! struct ExamplePlugin {
//!     meta: PluginMeta,
//! }
//!
//! impl ExamplePlugin {
//!     pub fn new() -> Self {
//!         Self {
//!             meta: PluginMeta::new(
//!                 "example-plugin",
//!                 "1.0.0",
//!                 "An example plugin",
//!                 "Example Author",
//!                 vec!["text".to_string()],
//!                 vec!["discord".to_string()],
//!             ),
//!         }
//!     }
//! }
//!
//! #[async_trait]
//! impl Plugin for ExamplePlugin {
//!     fn id(&self) -> &str {
//!         "example-plugin"
//!     }
//!
//!     fn meta(&self) -> &PluginMeta {
//!         &self.meta
//!     }
//!
//!     fn register(&self, api: &mut PluginApi) -> Result<(), Box<dyn std::error::Error>> {
//!         // Register capabilities with the API
//!         Ok(())
//!     }
//!
//!     async fn init(&self, _ctx: &PluginContext) -> Result<(), Box<dyn std::error::Error>> {
//!         Ok(())
//!     }
//!
//!     async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
//!         Ok(())
//!     }
//! }
//! ```
//!
//! ## Modules
//!
//! - [`meta`]: Plugin metadata types
//! - [`context`]: Runtime context for plugins
//! - [`trait`]: Core plugin trait definitions
//! - [`api`]: Plugin API for capability registration
//! - [`command`]: Plugin command types for CLI integration
//! - [`hook`]: Lifecycle hook types

pub mod api;
pub mod command;
pub mod context;
pub mod hook;
pub mod meta;
pub mod r#trait;

pub use api::PluginApi;
pub use command::PluginCommand;
pub use context::PluginContext;
pub use hook::{Hook, HookHandler, PluginHookHandler};
pub use meta::PluginMeta;
pub use r#trait::Plugin;
