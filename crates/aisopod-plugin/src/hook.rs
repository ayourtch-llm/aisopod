//! Lifecycle hook types for plugin system.
//!
//! This module defines the [`Hook`] enum and [`HookHandler`] trait
//! that allow plugins to register callbacks for various lifecycle events.

use std::sync::Arc;

/// Represents a lifecycle hook event that plugins can register for.
///
/// Hooks allow plugins to respond to system events such as startup,
/// shutdown, configuration changes, and other lifecycle events.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Hook {
    /// Triggered when the system is starting up, after all plugins are registered.
    ///
    /// Plugins can use this hook to perform any late-stage initialization
    /// that depends on other plugins being fully registered.
    SystemStart,

    /// Triggered when the system is shutting down gracefully.
    ///
    /// Plugins should use this hook to clean up resources, save state,
    /// and perform any necessary shutdown operations.
    SystemShutdown,

    /// Triggered when the plugin's configuration has been updated.
    ///
    /// This allows plugins to react to configuration changes without
    /// requiring a full restart.
    ConfigChanged,

    /// Triggered when a plugin is loaded or unloaded dynamically.
    ///
    /// This hook is for system-level plugins that manage other plugins.
    PluginLoaded,

    /// Triggered when a plugin is being unloaded.
    PluginUnloaded,
}

/// Trait for handling lifecycle hook events.
///
/// Plugins implement this trait to receive notifications about
/// system lifecycle events. Each hook handler is called with the
/// hook type and any relevant event data.
///
/// # Safety
///
/// Hook handlers must be thread-safe (`Send + Sync`) as they may be
/// called from different threads depending on when the hook is triggered.
pub trait HookHandler: Send + Sync {
    /// Called when a hook event is triggered.
    ///
    /// # Arguments
    ///
    /// * `hook` - The type of hook that was triggered
    /// * `data` - Additional data associated with the hook event, if any
    ///
    /// # Errors
    ///
    /// Return an error if the hook handler fails to process the event.
    /// For critical hooks like `SystemShutdown`, errors should be logged
    /// but not propagate to crash the system.
    fn on_hook(&self, hook: Hook, data: Option<&serde_json::Value>) -> Result<(), Box<dyn std::error::Error>>;
}

/// A typed hook handler that can be registered with the plugin API.
///
/// This struct wraps a [`HookHandler`] implementation and associates it
/// with a specific hook type. It is used by the [`PluginApi`] to manage
/// hook registrations.
#[derive(Clone)]
pub struct PluginHookHandler {
    /// The hook type this handler responds to.
    pub hook: Hook,
    /// The actual handler implementation.
    pub handler: Arc<dyn HookHandler>,
}

impl PluginHookHandler {
    /// Creates a new [`PluginHookHandler`] instance.
    pub fn new(hook: Hook, handler: Arc<dyn HookHandler>) -> Self {
        Self { hook, handler }
    }
}
