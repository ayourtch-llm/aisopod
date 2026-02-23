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
//! ## Manifest Format
//!
//! Plugins can include a `aisopod.plugin.toml` manifest file that describes
//! their identity, version, capabilities, and compatibility constraints.
//! See the [`manifest`] module for details.
//!
//! ## Registry
//!
//! The [`PluginRegistry`] provides centralized lifecycle management:
//!
//! - **Registration**: Plugins are registered via [`PluginRegistry::register()`]
//! - **Initialization**: All plugins initialized via [`PluginRegistry::init_all()`]
//! - **Shutdown**: All plugins shut down via [`PluginRegistry::shutdown_all()`]
//!
//! ## Dynamic Plugin Loading
//!
//! Plugins can be loaded dynamically from shared libraries (`.so`, `.dylib`, `.dll`):
//!
//! ```ignore
//! use aisopod_plugin::dynamic::{DynamicPluginLoader, LoadError};
//! use std::path::PathBuf;
//!
//! let loader = DynamicPluginLoader::new(vec![PathBuf::from("~/.aisopod/plugins")]);
//! let discovered = loader.discover()?;
//!
//! for plugin in discovered {
//!     match unsafe { loader.load_plugin(&plugin) } {
//!         Ok(p) => println!("Loaded: {}", p.id()),
//!         Err(e) => eprintln!("Failed: {}", e),
//!     }
//! }
//! ```
//!
//! ## Example
//!
//! This example shows the basic structure of a plugin:
//!
//! ```ignore
//! use aisopod_plugin::{Plugin, PluginMeta, PluginContext, PluginApi, manifest::PluginManifest};
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
//! - [`manifest`]: Plugin manifest format and parser
//! - [`context`]: Runtime context for plugins
//! - [`trait`]: Core plugin trait definitions
//! - [`api`]: Plugin API for capability registration
//! - [`command`]: Plugin command types for CLI integration
//! - [`hook`]: Lifecycle hook types
//! - [`registry`]: Plugin registry for lifecycle management
//! - [`abi`]: ABI definitions for dynamic plugins
//! - [`dynamic`]: Dynamic plugin loading from shared libraries

pub mod abi;
pub mod api;
pub mod command;
pub mod context;
pub mod dynamic;
pub mod hook;
pub mod manifest;
pub mod meta;
pub mod registry;
pub mod r#trait;

pub use abi::{ABI_VERSION, PluginAbiVersionFn, PluginCreateFn, PluginDestroyFn};
pub use api::PluginApi;
pub use command::PluginCommand;
pub use context::PluginContext;
pub use dynamic::{DiscoveredPlugin, DynamicPluginLoader, LoadError};
pub use hook::{Hook, HookContext, HookHandler, HookRegistry, PluginHookHandler};
pub use manifest::{ManifestError, PluginCapabilities, PluginCompatibility, PluginManifest, PluginManifestInfo};
pub use meta::PluginMeta;
pub use registry::{PluginRegistry, RegistryError};
pub use r#trait::Plugin;
