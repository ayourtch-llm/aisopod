//! Plugin API for capability registration.
//!
//! This module defines the [`PluginApi`] struct that plugins use during
//! registration to declare their capabilities. The API provides methods
//! to register channels, tools, CLI commands, model providers, and lifecycle hooks.

use std::sync::Arc;

use aisopod_channel::plugin::ChannelPlugin;
use aisopod_provider::ModelProvider;
use aisopod_tools::Tool;

use crate::command::PluginCommand;
use crate::hook::{Hook, HookHandler, PluginHookHandler};
use crate::security::SecurityError;

/// The API available to plugins during registration.
///
/// This struct provides the interface through which plugins can register
/// their capabilities with the system during the `register()` phase.
/// Plugins receive a mutable reference to `PluginApi` in their `register()`
/// method and use it to declare what they can do.
///
/// # Example
///
/// ```ignore
/// use aisopod_plugin::{Plugin, PluginApi};
/// use std::sync::Arc;
///
/// impl Plugin for MyPlugin {
///     fn register(&self, api: &mut PluginApi) -> Result<(), Box<dyn std::error::Error>> {
///         // Register a tool
///         api.register_tool(Arc::new(MyTool::new()));
///
///         // Register a channel
///         api.register_channel(Arc::new(MyChannel::new()));
///
///         // Register a CLI command
///         api.register_command(PluginCommand::new(
///             "status",
///             "Display status",
///             "plugin myplugin status",
///             true,
///             Arc::new(|args| {
///                 println!("Status: {:?}", args);
///                 Ok(())
///             }),
///         ));
///
///         Ok(())
///     }
/// }
/// ```
#[derive(Default)]
pub struct PluginApi {
    /// Registered channel implementations.
    pub(crate) channels: Vec<Arc<dyn ChannelPlugin>>,
    /// Registered tool implementations.
    pub(crate) tools: Vec<Arc<dyn Tool>>,
    /// Registered CLI commands.
    pub(crate) commands: Vec<PluginCommand>,
    /// Registered model providers.
    pub(crate) providers: Vec<Arc<dyn ModelProvider>>,
    /// Registered lifecycle hooks.
    pub(crate) hooks: Vec<PluginHookHandler>,
}

impl PluginApi {
    /// Creates a new empty [`PluginApi`] instance.
    ///
    /// This is typically called by the plugin host when initializing
    /// the registration phase for a plugin.
    pub fn new() -> Self {
        Self {
            channels: Vec::new(),
            tools: Vec::new(),
            commands: Vec::new(),
            providers: Vec::new(),
            hooks: Vec::new(),
        }
    }

    /// Returns the number of registered channels.
    pub fn channel_count(&self) -> usize {
        self.channels.len()
    }

    /// Returns the number of registered tools.
    pub fn tool_count(&self) -> usize {
        self.tools.len()
    }

    /// Returns the number of registered commands.
    pub fn command_count(&self) -> usize {
        self.commands.len()
    }

    /// Returns the number of registered providers.
    pub fn provider_count(&self) -> usize {
        self.providers.len()
    }

    /// Returns the number of registered hooks.
    pub fn hook_count(&self) -> usize {
        self.hooks.len()
    }

    /// Returns a reference to the registered channels.
    pub fn channels(&self) -> &[Arc<dyn ChannelPlugin>] {
        &self.channels
    }

    /// Returns a reference to the registered tools.
    pub fn tools(&self) -> &[Arc<dyn Tool>] {
        &self.tools
    }

    /// Returns a reference to the registered commands.
    pub fn commands(&self) -> &[PluginCommand] {
        &self.commands
    }

    /// Returns a reference to the registered providers.
    pub fn providers(&self) -> &[Arc<dyn ModelProvider>] {
        &self.providers
    }

    /// Returns a reference to the registered hooks.
    pub fn hooks(&self) -> &[PluginHookHandler] {
        &self.hooks
    }

    /// Register a channel implementation.
    ///
    /// This method allows plugins to contribute channel implementations
    /// that can be used for communication with external services.
    ///
    /// # Arguments
    ///
    /// * `channel` - An `Arc` wrapping the channel implementation
    pub fn register_channel(&mut self, channel: Arc<dyn ChannelPlugin>) {
        self.channels.push(channel);
    }

    /// Register a tool implementation.
    ///
    /// This method allows plugins to contribute tool implementations
    /// that can be used by AI models for function calling.
    ///
    /// # Arguments
    ///
    /// * `tool` - An `Arc` wrapping the tool implementation
    pub fn register_tool(&mut self, tool: Arc<dyn Tool>) {
        self.tools.push(tool);
    }

    /// Register a CLI subcommand.
    ///
    /// This method allows plugins to contribute CLI commands that
    /// can be invoked by users through the command-line interface.
    ///
    /// The command name is validated against security rules:
    /// - Must not be empty
    /// - Must be at most 64 characters
    /// - Must contain only alphanumeric characters, hyphens, and underscores
    /// - Must not match any reserved built-in command name
    ///
    /// # Arguments
    ///
    /// * `command` - The [`PluginCommand`] to register
    ///
    /// # Errors
    ///
    /// Returns `SecurityError::ReservedCommandName` if the command name
    /// matches a reserved built-in command.
    /// Returns `SecurityError::InvalidCommandName` if the command name
    /// fails validation (empty, too long, or contains invalid characters).
    pub fn register_command(&mut self, command: PluginCommand) -> Result<(), SecurityError> {
        // Validate the command name before registration
        crate::security::validate_command_name(&command.name)?;
        self.commands.push(command);
        Ok(())
    }

    /// Register a model provider.
    ///
    /// This method allows plugins to contribute AI model provider
    /// implementations that can be used for chat completions.
    ///
    /// # Arguments
    ///
    /// * `provider` - An `Arc` wrapping the provider implementation
    pub fn register_provider(&mut self, provider: Arc<dyn ModelProvider>) {
        self.providers.push(provider);
    }

    /// Register a lifecycle hook handler.
    ///
    /// This method allows plugins to register callbacks for lifecycle
    /// events such as system startup, shutdown, and configuration changes.
    ///
    /// # Arguments
    ///
    /// * `hook` - The type of hook to register for
    /// * `plugin_id` - The ID of the plugin registering the handler
    /// * `handler` - An `Arc` wrapping the hook handler implementation
    pub fn register_hook(&mut self, hook: Hook, plugin_id: String, handler: Arc<dyn HookHandler>) {
        self.hooks
            .push(PluginHookHandler::new(hook, plugin_id, handler));
    }
}

impl std::fmt::Debug for PluginApi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginApi")
            .field("channel_count", &self.channels.len())
            .field("tool_count", &self.tools.len())
            .field("command_count", &self.commands.len())
            .field("provider_count", &self.providers.len())
            .field("hook_count", &self.hooks.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_api_new() {
        let api = PluginApi::new();
        assert_eq!(api.channel_count(), 0);
        assert_eq!(api.tool_count(), 0);
        assert_eq!(api.command_count(), 0);
        assert_eq!(api.provider_count(), 0);
        assert_eq!(api.hook_count(), 0);
    }

    #[test]
    fn test_plugin_api_debug() {
        let api = PluginApi::new();
        let debug_str = format!("{:?}", api);
        assert!(debug_str.contains("PluginApi"));
    }

    #[test]
    fn test_register_command() {
        let mut api = PluginApi::new();
        let command = PluginCommand::new(
            "myplugin",
            "A test command",
            "myplugin [OPTIONS]",
            false,
            Arc::new(|_| Ok(())),
        );
        assert!(api.register_command(command).is_ok());
        assert_eq!(api.command_count(), 1);
    }

    #[test]
    fn test_register_command_reserved_name() {
        let mut api = PluginApi::new();
        let command =
            PluginCommand::new("help", "Help command", "help", false, Arc::new(|_| Ok(())));
        let result = api.register_command(command);
        assert!(result.is_err());
        // Should return ReservedCommandName error
    }

    #[test]
    fn test_register_command_invalid_name() {
        let mut api = PluginApi::new();

        // Empty name
        let command = PluginCommand::new("", "Empty command", "", false, Arc::new(|_| Ok(())));
        let result = api.register_command(command);
        assert!(result.is_err());

        // Too long name
        let command = PluginCommand::new(
            "a".repeat(65),
            "Long command",
            "a",
            false,
            Arc::new(|_| Ok(())),
        );
        let result = api.register_command(command);
        assert!(result.is_err());

        // Invalid characters
        let command = PluginCommand::new(
            "bad!char",
            "Bad char command",
            "bad!char",
            false,
            Arc::new(|_| Ok(())),
        );
        let result = api.register_command(command);
        assert!(result.is_err());
    }

    #[test]
    fn test_getters() {
        let mut api = PluginApi::new();

        // Check that getters return empty slices when nothing is registered
        assert!(api.channels().is_empty());
        assert!(api.tools().is_empty());
        assert!(api.providers().is_empty());
        assert!(api.hooks().is_empty());
        assert!(api.commands().is_empty());
    }
}
