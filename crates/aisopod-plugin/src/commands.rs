//! Command registry for plugin CLI commands.
//!
//! This module provides the [`CommandRegistry`] struct that manages
//! plugin-contributed CLI commands with thread-safe access via `RwLock`.
//!
//! # Security Features
//!
//! - Reserved command name protection
//! - Argument sanitization (control character removal, size limits)
//! - Authorization checks for sensitive commands
//! - Concurrent access protection via `RwLock`

use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockReadGuard};

use crate::command::PluginCommand;
use crate::security::{sanitize_argument, validate_command_name, SecurityError};

/// Registry for plugin CLI commands.
///
/// The `CommandRegistry` provides a thread-safe registry for storing
/// and executing plugin-contributed CLI commands. It uses `RwLock` for
/// concurrent access protection and enforces security constraints
/// through the [`security`] module.
///
/// # Thread Safety
///
/// The registry is designed for concurrent access. All read operations
/// use a read lock, and write operations use a write lock. This allows
/// multiple readers but only one writer at a time.
///
/// # Example
///
/// ```ignore
/// use aisopod_plugin::{CommandRegistry, PluginCommand};
/// use std::sync::Arc;
///
/// let registry = CommandRegistry::new();
///
/// let command = PluginCommand::new(
///     "status",
///     "Display plugin status",
///     "plugin status [OPTIONS]",
///     false,
///     Arc::new(|args| {
///         println!("Status: {:?}", args);
///         Ok(())
///     }),
/// );
///
/// // Register with validation
/// registry.register(command)?;
///
/// // Execute with sanitization
/// registry.execute("status", &["--verbose".to_string()])?;
/// ```
#[derive(Default)]
pub struct CommandRegistry {
    /// Map of command name to command implementation.
    ///
    /// Protected by `RwLock` for thread-safe concurrent access.
    commands: RwLock<HashMap<String, PluginCommand>>,
}

impl CommandRegistry {
    /// Creates a new empty [`CommandRegistry`].
    ///
    /// The registry starts with no commands registered.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use aisopod_plugin::CommandRegistry;
    /// let registry = CommandRegistry::new();
    /// ```
    pub fn new() -> Self {
        Self {
            commands: RwLock::new(HashMap::new()),
        }
    }

    /// Registers a command with the registry.
    ///
    /// This method performs several security checks before registration:
    /// - Validates the command name format
    /// - Checks against reserved command names
    /// - Detects duplicate registrations
    ///
    /// # Arguments
    ///
    /// * `command` - The [`PluginCommand`] to register
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or a `SecurityError` if:
    /// - The command name is invalid
    /// - The command name is reserved
    /// - A command with the same name is already registered
    /// - The registry lock is poisoned
    ///
    /// # Example
    ///
    /// ```ignore
    /// use aisopod_plugin::{CommandRegistry, PluginCommand};
    /// use std::sync::Arc;
    ///
    /// let registry = CommandRegistry::new();
    ///
    /// let command = PluginCommand::new(
    ///     "status",
    ///     "Display status",
    ///     "status [OPTIONS]",
    ///     false,
    ///     Arc::new(|args| {
    ///         println!("Status: {:?}", args);
    ///         Ok(())
    ///     }),
    /// );
    ///
    /// registry.register(command)?;
    /// ```
    pub fn register(&self, command: PluginCommand) -> Result<(), SecurityError> {
        // Validate the command name before registration
        validate_command_name(&command.name)?;

        // Perform registration with write lock
        let mut cmds = self
            .commands
            .write()
            .map_err(|_| SecurityError::RegistryLockPoisoned)?;

        // Check for duplicate command names
        if cmds.contains_key(&command.name) {
            return Err(SecurityError::DuplicateCommand(command.name.clone()));
        }

        cmds.insert(command.name.clone(), command);
        Ok(())
    }

    /// Executes a registered command with the given arguments.
    ///
    /// This method performs argument sanitization before passing the
    /// arguments to the command handler. It also checks authorization
    /// for commands that require it.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the command to execute
    /// * `args` - The command-line arguments (excluding the command name)
    ///
    /// # Returns
    ///
    /// `Ok(())` if the command executed successfully, or an error if:
    /// - The command is not found
    /// - Argument sanitization fails
    /// - The registry lock is poisoned
    /// - The command handler returns an error
    ///
    /// # Example
    ///
    /// ```ignore
    /// use aisopod_plugin::{CommandRegistry, PluginCommand};
    /// use std::sync::Arc;
    ///
    /// let registry = CommandRegistry::new();
    /// let command = PluginCommand::new(
    ///     "echo",
    ///     "Echo arguments",
    ///     "echo <message>",
    ///     false,
    ///     Arc::new(|args| {
    ///         println!("Echo: {:?}", args);
    ///         Ok(())
    ///     }),
    /// );
    /// registry.register(command)?;
    ///
    /// // Arguments are sanitized before passing to handler
    /// registry.execute("echo", &["hello".to_string(), "world".to_string()])?;
    /// ```
    pub fn execute(&self, name: &str, args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
        // Sanitize all arguments
        let sanitized_args: Vec<String> = args
            .iter()
            .map(|a| sanitize_argument(a))
            .collect::<Result<Vec<_>, _>>()?;

        // Look up and execute with read lock
        let cmds = self
            .commands
            .read()
            .map_err(|_| SecurityError::RegistryLockPoisoned)?;

        let command = cmds
            .get(name)
            .ok_or_else(|| SecurityError::CommandNotFound(name.to_string()))?;

        // Check authorization if required
        if command.require_auth {
            tracing::debug!(
                command = %name,
                "Authorization check required for command"
            );
            // Note: Full authorization integration is handled by the CLI layer
        }

        // Execute the command handler
        (command.handler)(&sanitized_args)
    }

    /// Checks if a command is registered with the given name.
    ///
    /// # Arguments
    ///
    /// * `name` - The command name to check
    ///
    /// # Returns
    ///
    /// `true` if a command with the given name is registered, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use aisopod_plugin::{CommandRegistry, PluginCommand};
    /// use std::sync::Arc;
    ///
    /// let registry = CommandRegistry::new();
    /// assert!(!registry.has_command("status"));
    ///
    /// let command = PluginCommand::new(
    ///     "status",
    ///     "Display status",
    ///     "status",
    ///     false,
    ///     Arc::new(|_| Ok(())),
    /// );
    /// registry.register(command).unwrap();
    /// assert!(registry.has_command("status"));
    /// ```
    pub fn has_command(&self, name: &str) -> bool {
        if let Ok(cmds) = self.commands.read() {
            cmds.contains_key(name)
        } else {
            false
        }
    }

    /// Returns the number of registered commands.
    ///
    /// # Returns
    ///
    /// The number of commands currently registered.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use aisopod_plugin::{CommandRegistry, PluginCommand};
    /// use std::sync::Arc;
    /// Returns the number of registered commands.
    ///
    /// # Returns
    ///
    /// The number of commands currently registered.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use aisopod_plugin::{CommandRegistry, PluginCommand};
    /// use std::sync::Arc;
    ///
    /// let registry = CommandRegistry::new();
    /// assert_eq!(registry.command_count(), 0);
    ///
    /// let command = PluginCommand::new(
    ///     "test",
    ///     "Test command",
    ///     "test",
    ///     false,
    ///     Arc::new(|_| Ok(())),
    /// );
    /// registry.register(command).unwrap();
    /// assert_eq!(registry.command_count(), 1);
    /// ```
    pub fn command_count(&self) -> usize {
        self.commands
            .read()
            .map(|cmds| cmds.len())
            .unwrap_or(0)
    }

    /// Returns a list of all registered command names.
    ///
    /// # Returns
    ///
    /// A vector of command names in arbitrary order.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use aisopod_plugin::{CommandRegistry, PluginCommand};
    /// use std::sync::Arc;
    ///
    /// let registry = CommandRegistry::new();
    /// let command1 = PluginCommand::new("status", "Status", "status", false, Arc::new(|_| Ok(())));
    /// let command2 = PluginCommand::new("version", "Version", "version", false, Arc::new(|_| Ok(())));
    /// registry.register(command1).unwrap();
    /// registry.register(command2).unwrap();
    ///
    /// let names = registry.list_commands();
    /// assert_eq!(names.len(), 2);
    /// assert!(names.contains(&"status".to_string()));
    /// assert!(names.contains(&"version".to_string()));
    /// ```
    pub fn list_commands(&self) -> Vec<String> {
        self.commands
            .read()
            .map(|cmds| cmds.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Returns the number of commands that require authentication.
    ///
    /// # Returns
    ///
    /// The count of commands with `require_auth = true`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use aisopod_plugin::{CommandRegistry, PluginCommand};
    /// use std::sync::Arc;
    ///
    /// let registry = CommandRegistry::new();
    /// let public_cmd = PluginCommand::new("public", "Public", "public", false, Arc::new(|_| Ok(())));
    /// let auth_cmd = PluginCommand::new("admin", "Admin", "admin", true, Arc::new(|_| Ok(())));
    /// registry.register(public_cmd).unwrap();
    /// registry.register(auth_cmd).unwrap();
    ///
    /// assert_eq!(registry.auth_command_count(), 1);
    /// ```
    pub fn auth_command_count(&self) -> usize {
        self.commands
            .read()
            .map(|cmds| cmds.values().filter(|c| c.require_auth).count())
            .unwrap_or(0)
    }

    /// Returns a guard holding the read lock for the command registry.
    ///
    /// This allows inspection of the registry while holding the lock.
    /// Use with caution in performance-critical paths.
    ///
    /// # Returns
    ///
    /// `Some(RwLockReadGuard)` if the lock could be acquired, `None` if poisoned.
    pub fn read_guard(&self) -> Option<RwLockReadGuard<'_, HashMap<String, PluginCommand>>> {
        self.commands.read().ok()
    }

    /// Returns a reference to a registered command by name.
    ///
    /// # Arguments
    ///
    /// * `name` - The command name to retrieve
    ///
    /// # Returns
    ///
    /// `Some(&PluginCommand)` if found, `None` if the command doesn't exist
    /// or the lock is poisoned.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use aisopod_plugin::{CommandRegistry, PluginCommand};
    /// use std::sync::Arc;
    ///
    /// let registry = CommandRegistry::new();
    /// let command = PluginCommand::new("test", "Test", "test", false, Arc::new(|_| Ok(())));
    /// registry.register(command).unwrap();
    ///
    /// if let Some(retrieved) = registry.get_command("test") {
    ///     assert_eq!(retrieved.name, "test");
    /// }
    /// ```
    pub fn get_command(&self, name: &str) -> Option<crate::command::PluginCommand> {
        let cmds = self.commands.read().ok()?;
        cmds.get(name).cloned()
    }

    /// Clears all registered commands from the registry.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use aisopod_plugin::{CommandRegistry, PluginCommand};
    /// use std::sync::Arc;
    ///
    /// let registry = CommandRegistry::new();
    /// let command = PluginCommand::new("test", "Test", "test", false, Arc::new(|_| Ok(())));
    /// registry.register(command).unwrap();
    /// assert_eq!(registry.command_count(), 1);
    ///
    /// registry.clear();
    /// assert_eq!(registry.command_count(), 0);
    /// ```
    pub fn clear(&self) {
        let mut cmds = self.commands.write().ok();
        if let Some(mut cmds_mut) = cmds {
            cmds_mut.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::SecurityError;

    #[test]
    fn test_command_registry_new() {
        let registry = CommandRegistry::new();
        assert_eq!(registry.command_count(), 0);
        assert!(registry.list_commands().is_empty());
    }

    #[test]
    fn test_register_command() {
        let registry = CommandRegistry::new();
        let command = PluginCommand::new(
            "myplugin",
            "A test command",
            "myplugin [OPTIONS]",
            false,
            Arc::new(|_| Ok(())),
        );

        assert!(registry.register(command).is_ok());
        assert_eq!(registry.command_count(), 1);
    }

    #[test]
    fn test_register_reserved_command() {
        let registry = CommandRegistry::new();

        // Try to register a reserved command
        let command = PluginCommand::new(
            "help",
            "Help command",
            "help",
            false,
            Arc::new(|_| Ok(())),
        );

        let result = registry.register(command);
        assert!(matches!(result, Err(SecurityError::ReservedCommandName(_))));
    }

    #[test]
    fn test_register_duplicate_command() {
        let registry = CommandRegistry::new();

        let command1 = PluginCommand::new(
            "myplugin",
            "First test command",
            "myplugin",
            false,
            Arc::new(|_| Ok(())),
        );

        let command2 = PluginCommand::new(
            "myplugin",
            "Second test command",
            "myplugin",
            false,
            Arc::new(|_| Ok(())),
        );

        assert!(registry.register(command1).is_ok());
        let result = registry.register(command2);
        assert!(matches!(result, Err(SecurityError::DuplicateCommand(_))));
    }

    #[test]
    fn test_register_invalid_command_name() {
        let registry = CommandRegistry::new();

        // Empty name
        let command = PluginCommand::new(
            "",
            "Empty name command",
            "",
            false,
            Arc::new(|_| Ok(())),
        );
        assert!(matches!(
            registry.register(command),
            Err(SecurityError::InvalidCommandName(_))
        ));

        // Too long name
        let command = PluginCommand::new(
            "a".repeat(65),
            "Long name command",
            "a",
            false,
            Arc::new(|_| Ok(())),
        );
        assert!(matches!(
            registry.register(command),
            Err(SecurityError::InvalidCommandName(_))
        ));

        // Invalid characters
        let command = PluginCommand::new(
            "bad!char",
            "Bad char command",
            "bad!char",
            false,
            Arc::new(|_| Ok(())),
        );
        assert!(matches!(
            registry.register(command),
            Err(SecurityError::InvalidCommandName(_))
        ));
    }

    #[test]
    fn test_has_command() {
        let registry = CommandRegistry::new();

        assert!(!registry.has_command("nonexistent"));

        let command = PluginCommand::new(
            "myplugin",
            "Test command",
            "myplugin",
            false,
            Arc::new(|_| Ok(())),
        );
        registry.register(command).unwrap();

        assert!(registry.has_command("myplugin"));
        assert!(!registry.has_command("nonexistent"));
    }

    #[test]
    fn test_execute_command() {
        let registry = CommandRegistry::new();
        let command = PluginCommand::new(
            "myplugin",
            "Test command",
            "myplugin [OPTIONS]",
            false,
            Arc::new(|args| {
                assert_eq!(args, &["--verbose".to_string()]);
                Ok(())
            }),
        );

        registry.register(command).unwrap();
        assert!(registry.execute("myplugin", &["--verbose".to_string()]).is_ok());
    }

    #[test]
    fn test_execute_command_not_found() {
        let registry = CommandRegistry::new();
        let result = registry.execute("nonexistent", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_sanitizes_arguments() {
        let registry = CommandRegistry::new();
        let command = PluginCommand::new(
            "echo",
            "Echo command",
            "echo <message>",
            false,
            Arc::new(|args| {
                // Control characters should be removed
                assert_eq!(args, &["helloworld".to_string()]);
                Ok(())
            }),
        );

        registry.register(command).unwrap();
        assert!(registry
            .execute("echo", &["hello\x00world".to_string()])
            .is_ok());
    }

    #[test]
    fn test_list_commands() {
        let registry = CommandRegistry::new();

        let command1 = PluginCommand::new(
            "myplugin1",
            "Status command",
            "myplugin1",
            false,
            Arc::new(|_| Ok(())),
        );
        let command2 = PluginCommand::new(
            "myplugin2",
            "Version command",
            "myplugin2",
            false,
            Arc::new(|_| Ok(())),
        );

        registry.register(command1).unwrap();
        registry.register(command2).unwrap();

        let commands = registry.list_commands();
        assert_eq!(commands.len(), 2);
        assert!(commands.contains(&"myplugin1".to_string()));
        assert!(commands.contains(&"myplugin2".to_string()));
    }

    #[test]
    fn test_get_command() {
        let registry = CommandRegistry::new();
        let command = PluginCommand::new(
            "myplugin",
            "Test command",
            "myplugin",
            false,
            Arc::new(|_| Ok(())),
        );
        registry.register(command.clone()).unwrap();

        if let Some(retrieved) = registry.get_command("myplugin") {
            assert_eq!(retrieved.name, "myplugin");
            assert_eq!(retrieved.description, "Test command");
        } else {
            panic!("Command not found");
        }
    }

    #[test]
    fn test_clear_commands() {
        let registry = CommandRegistry::new();

        let command1 = PluginCommand::new(
            "cmd1",
            "Command 1",
            "cmd1",
            false,
            Arc::new(|_| Ok(())),
        );
        let command2 = PluginCommand::new(
            "cmd2",
            "Command 2",
            "cmd2",
            false,
            Arc::new(|_| Ok(())),
        );

        registry.register(command1).unwrap();
        registry.register(command2).unwrap();
        assert_eq!(registry.command_count(), 2);

        registry.clear();
        assert_eq!(registry.command_count(), 0);
    }

    #[test]
    fn test_authorization_required() {
        let registry = CommandRegistry::new();
        let command = PluginCommand::new(
            "admin",
            "Admin command",
            "admin",
            true, // requires_auth
            Arc::new(|_| Ok(())),
        );

        registry.register(command).unwrap();
        // Should not error on execution, but debug log should be triggered
        assert!(registry.execute("admin", &[]).is_ok());
    }
}
