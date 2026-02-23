//! ABI definitions for dynamic plugin loading.
//!
//! This module defines the Application Binary Interface (ABI) version
//! and function signatures that all dynamic plugins must implement.
//!
//! # ABI Versioning
//!
//! The ABI version is a critical component for ensuring compatibility
//! between the host application and dynamically loaded plugins. When
//! the Plugin trait or PluginApi changes in a breaking way, the ABI
//! version should be incremented.
//!
//! # Example
//!
//! A dynamic plugin must export these symbols:
//!
//! ```ignore
//! use aisopod_plugin::{Plugin, PluginMeta, PluginContext, PluginApi};
//! use async_trait::async_trait;
//! use std::sync::Arc;
//!
//! const ABI_VERSION: u32 = 1;
//!
//! #[derive(Debug)]
//! struct MyPlugin {
//!     meta: PluginMeta,
//! }
//!
//! impl MyPlugin {
//!     pub fn new() -> Self {
//!         Self {
//!             meta: PluginMeta::new(
//!                 "my-plugin",
//!                 "1.0.0",
//!                 "A sample dynamic plugin",
//!                 "Author Name",
//!                 vec![],
//!                 vec![],
//!             ),
//!         }
//!     }
//! }
//!
//! #[async_trait]
//! impl Plugin for MyPlugin {
//!     fn id(&self) -> &str {
//!         "my-plugin"
//!     }
//!
//!     fn meta(&self) -> &PluginMeta {
//!         &self.meta
//!     }
//!
//!     fn register(&self, _api: &mut PluginApi) -> Result<(), Box<dyn std::error::Error>> {
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
//!
//! /// ABI version symbol - every dynamic plugin must export this.
//! #[no_mangle]
//! pub extern "C" fn aisopod_plugin_abi_version() -> u32 {
//!     ABI_VERSION
//! }
//!
//! /// Plugin creation symbol - every dynamic plugin must export this.
//! #[no_mangle]
//! pub unsafe extern "C" fn aisopod_plugin_create() -> *mut dyn Plugin {
//!     Box::into_raw(Box::new(MyPlugin::new()))
//! }
//! ```

use crate::Plugin;

/// ABI version for plugin compatibility checking.
/// Bump this when the Plugin trait or PluginApi changes in a breaking way.
pub const ABI_VERSION: u32 = 1;

/// Function signature that every dynamic plugin must export to create instances.
///
/// This is the entry point for dynamic plugins. The host calls this function
/// to obtain a new plugin instance. The plugin must return a raw pointer to
/// a Box<dyn Plugin>.
///
/// # Safety
///
/// This function is unsafe because it returns a raw pointer. The caller must
/// ensure the pointer is valid and properly owned.
pub type PluginCreateFn = unsafe extern "C" fn() -> *mut dyn Plugin;

/// Function signature that every dynamic plugin must export to query ABI version.
///
/// The host calls this function to verify compatibility before attempting to
/// create a plugin instance.
pub type PluginAbiVersionFn = unsafe extern "C" fn() -> u32;

/// Function signature for plugin cleanup/destructor.
///
/// This optional function can be exported by plugins to perform cleanup
/// when the plugin instance is destroyed. If not present, the host will
/// use Box::from_raw to destroy the plugin.
pub type PluginDestroyFn = unsafe extern "C" fn(*mut dyn Plugin);
