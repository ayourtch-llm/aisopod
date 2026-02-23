//! Plugin command types for CLI integration.
//!
//! This module defines the [`PluginCommand`] struct that allows plugins
//! to contribute CLI subcommands to the host application.

use std::sync::Arc;

/// A CLI subcommand contributed by a plugin.
///
/// This struct represents a command that can be added to the CLI by a plugin.
/// When the plugin is loaded, its commands are registered with the CLI system
/// and can be executed by users.
///
/// # Example
///
/// ```ignore
/// use aisopod_plugin::PluginCommand;
///
/// let command = PluginCommand::new(
///     "status",
///     "Display plugin status information",
///     "plugin status [OPTIONS]",
///     true, // requires authentication
///     Box::new(|args: &[String]| {
///         println!("Plugin status command called with args: {:?}", args);
///         Ok(())
///     }),
/// );
/// ```
#[derive(Clone)]
pub struct PluginCommand {
    /// The subcommand name (e.g., "status", "list", "enable").
    ///
    /// This should be a lowercase, hyphen-separated identifier that is
    /// unique within the plugin's namespace.
    pub name: String,
    /// A brief description of what this command does.
    ///
    /// This description is used in help text to explain the command's purpose.
    pub description: String,
    /// The usage string showing how to invoke this command.
    ///
    /// This should include the command name and any expected arguments
    /// or options (e.g., "plugin status [OPTIONS]").
    pub usage: String,
    /// Whether this command requires authentication.
    ///
    /// If `true`, the command will only be available to authenticated users.
    /// If `false`, the command can be executed by anyone.
    pub require_auth: bool,
    /// The handler function that executes this command.
    ///
    /// This closure receives the command-line arguments (excluding the command name)
    /// and returns a result indicating success or failure.
    pub handler: Arc<dyn Fn(&[String]) -> Result<(), Box<dyn std::error::Error>> + Send + Sync>,
}

impl PluginCommand {
    /// Creates a new [`PluginCommand`] instance.
    ///
    /// # Arguments
    ///
    /// * `name` - The subcommand name
    /// * `description` - A brief description of what this command does
    /// * `usage` - The usage string showing how to invoke this command
    /// * `require_auth` - Whether this command requires authentication
    /// * `handler` - The handler function that executes this command
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        usage: impl Into<String>,
        require_auth: bool,
        handler: Arc<dyn Fn(&[String]) -> Result<(), Box<dyn std::error::Error>> + Send + Sync>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            usage: usage.into(),
            require_auth,
            handler,
        }
    }

    /// Creates a new [`PluginCommand`] with a mutable handler.
    ///
    /// This is a convenience method for cases where the handler needs
    /// to mutate internal state. The handler is wrapped in `Arc<Mutex<...>>`.
    pub fn with_mutable_handler<F>(
        name: impl Into<String>,
        description: impl Into<String>,
        usage: impl Into<String>,
        require_auth: bool,
        handler: F,
    ) -> Self
    where
        F: Fn(&[String]) -> Result<(), Box<dyn std::error::Error>> + Send + Sync + 'static,
    {
        Self::new(name, description, usage, require_auth, Arc::new(handler))
    }
}
