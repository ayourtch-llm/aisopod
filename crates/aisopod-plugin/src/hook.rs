//! Lifecycle hook types for plugin system.
//!
//! This module defines the [`Hook`] enum, [`HookHandler`] trait, and
//! [`HookRegistry`] that allow plugins to subscribe to lifecycle events
//! throughout the application.

use std::collections::HashMap;
use std::sync::Arc;

use tracing::debug;

use crate::api::PluginApi;

/// Represents a lifecycle hook event that plugins can register for.
///
/// Hooks allow plugins to respond to system events such as startup,
/// shutdown, configuration changes, and other lifecycle events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Hook {
    /// Triggered before the agent begins processing a request.
    BeforeAgentRun,

    /// Triggered after the agent completes processing a request.
    AfterAgentRun,

    /// Triggered before a message is sent to the user.
    BeforeMessageSend,

    /// Triggered after a message is received from the user.
    AfterMessageReceive,

    /// Triggered before a tool is executed.
    BeforeToolExecute,

    /// Triggered after a tool is executed.
    AfterToolExecute,

    /// Triggered when a new session is created.
    OnSessionCreate,

    /// Triggered when a session ends.
    OnSessionEnd,

    /// Triggered when the gateway starts.
    OnGatewayStart,

    /// Triggered when the gateway shuts down.
    OnGatewayShutdown,

    /// Triggered when a client connects.
    OnClientConnect,

    /// Triggered when a client disconnects.
    OnClientDisconnect,
}

/// Context data passed to hook handlers.
///
/// This struct contains information about the hook event being triggered,
/// along with arbitrary key-value data that can be used by handlers.
#[derive(Debug, Clone)]
pub struct HookContext {
    /// The hook that was triggered.
    pub hook: Hook,
    /// Additional data associated with the hook event.
    pub data: HashMap<String, serde_json::Value>,
}

impl HookContext {
    /// Creates a new `HookContext` with the given hook and empty data.
    pub fn new(hook: Hook) -> Self {
        Self {
            hook,
            data: HashMap::new(),
        }
    }

    /// Creates a new `HookContext` with the given hook and data.
    pub fn with_data(hook: Hook, data: HashMap<String, serde_json::Value>) -> Self {
        Self { hook, data }
    }

    /// Adds a key-value pair to the context data.
    pub fn with_data_entry(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.data.insert(key.into(), value);
        self
    }
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
#[async_trait::async_trait]
pub trait HookHandler: Send + Sync {
    /// Handle a lifecycle event. Returning an error logs it but does not
    /// prevent other handlers from executing.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The hook context containing the hook type and event data
    ///
    /// # Errors
    ///
    /// Return an error if the hook handler fails to process the event.
    /// Errors are logged but do not prevent other handlers from executing.
    async fn handle(&self, ctx: &HookContext) -> Result<(), Box<dyn std::error::Error>>;
}

/// A typed hook handler that can be registered with the plugin API.
///
/// This struct wraps a [`HookHandler`] implementation and associates it
/// with a specific hook type and plugin ID. It is used by the [`crate::PluginApi`] to manage
/// hook registrations.
#[derive(Clone)]
pub struct PluginHookHandler {
    /// The hook type this handler responds to.
    pub hook: Hook,
    /// The plugin ID registering this handler.
    pub plugin_id: String,
    /// The actual handler implementation.
    pub handler: Arc<dyn HookHandler>,
}

impl PluginHookHandler {
    /// Creates a new [`PluginHookHandler`] instance.
    pub fn new(hook: Hook, plugin_id: String, handler: Arc<dyn HookHandler>) -> Self {
        Self { hook, plugin_id, handler }
    }
}

/// Registry for storing and dispatching hook handlers.
///
/// The `HookRegistry` maintains a collection of hook handlers organized
/// by hook type. It provides methods to register handlers and dispatch
/// events to all registered handlers for a given hook.
///
/// # Example
///
/// ```ignore
/// use aisopod_plugin::{HookRegistry, Hook, HookContext, HookHandler};
/// use std::sync::Arc;
/// use async_trait::async_trait;
///
/// struct MyHandler;
///
/// #[async_trait]
/// impl HookHandler for MyHandler {
///     async fn handle(&self, ctx: &HookContext) -> Result<(), Box<dyn std::error::Error>> {
///         println!("Hook triggered: {:?}", ctx.hook);
///         Ok(())
///     }
/// }
///
/// #[tokio::main]
/// async fn main() {
///     let mut registry = HookRegistry::new();
///     registry.register(Hook::SystemStart, "my-plugin".to_string(), Arc::new(MyHandler));
///
///     let ctx = HookContext::new(Hook::SystemStart);
///     registry.dispatch(&ctx).await;
/// }
/// ```
#[derive(Default)]
pub struct HookRegistry {
    /// Map of hook type to list of (plugin_id, handler) tuples.
    handlers: HashMap<Hook, Vec<(String, Arc<dyn HookHandler>)>>,
}

impl HookRegistry {
    /// Creates a new empty [`HookRegistry`].
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Registers a hook handler for a specific lifecycle event.
    ///
    /// # Arguments
    ///
    /// * `hook` - The hook type to register for
    /// * `plugin_id` - The ID of the plugin registering the handler
    /// * `handler` - An `Arc` wrapping the hook handler implementation
    pub fn register(&mut self, hook: Hook, plugin_id: String, handler: Arc<dyn HookHandler>) {
        self.handlers
            .entry(hook)
            .or_default()
            .push((plugin_id, handler));
    }

    /// Dispatches a hook event to all registered handlers.
    ///
    /// All handlers for the given hook are called asynchronously.
    /// If a handler fails, the error is logged but other handlers
    /// continue to execute.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The hook context containing the hook type and event data
    pub async fn dispatch(&self, ctx: &HookContext) {
        if let Some(handlers) = self.handlers.get(&ctx.hook) {
            for (plugin_id, handler) in handlers {
                match handler.handle(ctx).await {
                    Ok(()) => {
                        debug!(
                            plugin_id = %plugin_id,
                            hook = ?ctx.hook,
                            "Hook handler completed successfully"
                        );
                    }
                    Err(e) => {
                        debug!(
                            plugin_id = %plugin_id,
                            hook = ?ctx.hook,
                            error = %e,
                            "Hook handler failed"
                        );
                    }
                }
            }
        }
    }

    /// Returns the number of handlers registered for a given hook.
    ///
    /// # Arguments
    ///
    /// * `hook` - The hook type to check
    pub fn handler_count(&self, hook: &Hook) -> usize {
        self.handlers.get(hook).map_or(0, |h| h.len())
    }

    /// Transfers hook registrations from a `PluginApi` to this registry.
    ///
    /// This method is called after plugin registration completes to move
    /// all hook registrations from the API to the registry for dispatching.
    ///
    /// # Arguments
    ///
    /// * `api` - The `PluginApi` containing registered hooks
    pub fn transfer_from_api(&mut self, api: &PluginApi) {
        for plugin_hook in api.hooks() {
            self.register(
                plugin_hook.hook,
                plugin_hook.plugin_id.clone(),
                plugin_hook.handler.clone(),
            );
        }
    }

    /// Returns the total number of registered hooks across all types.
    pub fn total_hook_count(&self) -> usize {
        self.handlers.values().map(|v| v.len()).sum()
    }

    /// Returns the number of hook types that have registered handlers.
    pub fn hook_type_count(&self) -> usize {
        self.handlers.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_debug() {
        assert_eq!(format!("{:?}", Hook::BeforeAgentRun), "BeforeAgentRun");
        assert_eq!(format!("{:?}", Hook::AfterAgentRun), "AfterAgentRun");
    }

    #[test]
    fn test_hook_clone_eq() {
        let hook = Hook::BeforeAgentRun;
        assert_eq!(hook, Hook::BeforeAgentRun);
        assert_ne!(hook, Hook::AfterAgentRun);
        assert_eq!(hook, hook.clone());
    }

    #[test]
    fn test_hook_context_new() {
        let ctx = HookContext::new(Hook::BeforeAgentRun);
        assert_eq!(ctx.hook, Hook::BeforeAgentRun);
        assert!(ctx.data.is_empty());
    }

    #[test]
    fn test_hook_context_with_data() {
        let mut data = HashMap::new();
        data.insert("key".to_string(), serde_json::json!("value"));

        let ctx = HookContext::with_data(Hook::BeforeAgentRun, data);
        assert_eq!(ctx.hook, Hook::BeforeAgentRun);
        assert_eq!(ctx.data.len(), 1);
        assert_eq!(ctx.data.get("key"), Some(&serde_json::json!("value")));
    }

    #[test]
    fn test_hook_context_with_data_entry() {
        let ctx = HookContext::new(Hook::BeforeAgentRun)
            .with_data_entry("key", serde_json::json!("value"))
            .with_data_entry("num", serde_json::json!(42));

        assert_eq!(ctx.data.len(), 2);
        assert_eq!(ctx.data.get("key"), Some(&serde_json::json!("value")));
        assert_eq!(ctx.data.get("num"), Some(&serde_json::json!(42)));
    }

    #[tokio::test]
    async fn test_hook_registry_new() {
        let registry = HookRegistry::new();
        assert_eq!(registry.handlers.len(), 0);
        assert_eq!(registry.total_hook_count(), 0);
        assert_eq!(registry.hook_type_count(), 0);
    }

    #[tokio::test]
    async fn test_hook_registry_register() {
        let mut registry = HookRegistry::new();

        #[derive(Clone)]
        struct TestHandler;

        #[async_trait::async_trait]
        impl HookHandler for TestHandler {
            async fn handle(&self, _ctx: &HookContext) -> Result<(), Box<dyn std::error::Error>> {
                Ok(())
            }
        }

        let handler = Arc::new(TestHandler);
        registry.register(Hook::BeforeAgentRun, "test-plugin".to_string(), handler);

        assert_eq!(registry.handler_count(&Hook::BeforeAgentRun), 1);
        assert_eq!(registry.total_hook_count(), 1);
        assert_eq!(registry.hook_type_count(), 1);
    }

    #[tokio::test]
    async fn test_hook_registry_dispatch_no_handlers() {
        let registry = HookRegistry::new();
        let ctx = HookContext::new(Hook::BeforeAgentRun);

        // Should not panic when no handlers are registered
        registry.dispatch(&ctx).await;
    }

    #[tokio::test]
    async fn test_hook_registry_dispatch_with_handlers() {
        let mut registry = HookRegistry::new();

        #[derive(Clone)]
        struct TestHandler {
            called: std::sync::Arc<std::sync::Mutex<bool>>,
        }

        #[async_trait::async_trait]
        impl HookHandler for TestHandler {
            async fn handle(&self, ctx: &HookContext) -> Result<(), Box<dyn std::error::Error>> {
                *self.called.lock().unwrap() = true;
                Ok(())
            }
        }

        let called1 = std::sync::Arc::new(std::sync::Mutex::new(false));
        let called2 = std::sync::Arc::new(std::sync::Mutex::new(false));

        let handler1 = Arc::new(TestHandler { called: called1.clone() });
        let handler2 = Arc::new(TestHandler { called: called2.clone() });

        registry.register(Hook::BeforeAgentRun, "plugin-1".to_string(), handler1);
        registry.register(Hook::BeforeAgentRun, "plugin-2".to_string(), handler2);

        let ctx = HookContext::new(Hook::BeforeAgentRun);
        registry.dispatch(&ctx).await;

        assert!(*called1.lock().unwrap());
        assert!(*called2.lock().unwrap());
    }

    #[tokio::test]
    async fn test_hook_registry_dispatch_error_handling() {
        let mut registry = HookRegistry::new();

        #[derive(Clone)]
        struct FailingHandler;

        #[async_trait::async_trait]
        impl HookHandler for FailingHandler {
            async fn handle(&self, _ctx: &HookContext) -> Result<(), Box<dyn std::error::Error>> {
                Err("Intentional failure".into())
            }
        }

        let handler = Arc::new(FailingHandler);
        registry.register(Hook::BeforeAgentRun, "failing-plugin".to_string(), handler);

        let ctx = HookContext::new(Hook::BeforeAgentRun);
        // Should not panic even with failing handler
        registry.dispatch(&ctx).await;
    }

    #[tokio::test]
    async fn test_hook_registry_handler_count() {
        let mut registry = HookRegistry::new();

        #[derive(Clone)]
        struct TestHandler;

        #[async_trait::async_trait]
        impl HookHandler for TestHandler {
            async fn handle(&self, _ctx: &HookContext) -> Result<(), Box<dyn std::error::Error>> {
                Ok(())
            }
        }

        let handler = Arc::new(TestHandler);

        // Register multiple handlers for the same hook
        registry.register(Hook::BeforeAgentRun, "plugin-1".to_string(), handler.clone());
        registry.register(Hook::BeforeAgentRun, "plugin-2".to_string(), handler.clone());
        registry.register(Hook::BeforeAgentRun, "plugin-3".to_string(), handler.clone());

        assert_eq!(registry.handler_count(&Hook::BeforeAgentRun), 3);

        // Different hook should have 0 handlers
        assert_eq!(registry.handler_count(&Hook::AfterAgentRun), 0);
    }

    #[tokio::test]
    async fn test_hook_registry_total_hook_count() {
        let mut registry = HookRegistry::new();

        #[derive(Clone)]
        struct TestHandler;

        #[async_trait::async_trait]
        impl HookHandler for TestHandler {
            async fn handle(&self, _ctx: &HookContext) -> Result<(), Box<dyn std::error::Error>> {
                Ok(())
            }
        }

        let handler = Arc::new(TestHandler);

        registry.register(Hook::BeforeAgentRun, "plugin-1".to_string(), handler.clone());
        registry.register(Hook::AfterAgentRun, "plugin-2".to_string(), handler.clone());
        registry.register(Hook::OnSessionCreate, "plugin-3".to_string(), handler.clone());

        assert_eq!(registry.total_hook_count(), 3);
        assert_eq!(registry.hook_type_count(), 3);
    }
}
