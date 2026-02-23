//! Security utilities for plugin command registration.
//!
//! This module provides security hardening for plugin CLI commands,
//! including reserved command name protection, argument sanitization,
//! and validation of command names.

use thiserror::Error;

/// Built-in command names that plugins are not allowed to override.
///
/// This list contains approximately 72 reserved command names that
/// correspond to the core CLI functionality. Plugins attempting to
/// register a command with one of these names will receive a
/// `SecurityError::ReservedCommandName` error.
pub const RESERVED_COMMANDS: &[&str] = &[
    "help", "version", "config", "init", "start", "stop", "restart",
    "status", "log", "logs", "plugin", "plugins", "install", "uninstall",
    "update", "upgrade", "enable", "disable", "list", "show", "get",
    "set", "delete", "remove", "create", "new", "run", "exec", "shell",
    "repl", "chat", "send", "receive", "connect", "disconnect", "login",
    "logout", "auth", "token", "key", "secret", "env", "export", "import",
    "backup", "restore", "migrate", "reset", "clear", "clean", "purge",
    "test", "check", "validate", "lint", "format", "build", "deploy",
    "publish", "release", "tag", "branch", "commit", "push", "pull",
    "fetch", "clone", "diff", "merge", "rebase", "stash", "pop",
    "apply", "patch", "doctor", "diagnose", "debug", "trace", "profile",
    "benchmark", "info", "about", "license", "completions",
];

/// Maximum allowed size for a single command argument in bytes.
///
/// This limit prevents memory exhaustion from extremely large arguments.
pub const MAX_ARG_SIZE: usize = 4096;

/// Security error types for plugin command registration and execution.
///
/// This enum captures all security-related errors that can occur when
/// registering or executing plugin CLI commands, including reserved
/// name conflicts, invalid command names, duplicate registrations,
/// and argument validation failures.
#[derive(Debug, Error)]
pub enum SecurityError {
    /// Reserved command name conflict.
    ///
    /// Plugins are not allowed to override built-in command names.
    /// This error is returned when a plugin attempts to register a
    /// command with a reserved name.
    #[error("Reserved command name: '{0}' cannot be overridden by plugins")]
    ReservedCommandName(String),

    /// Invalid command name format.
    ///
    /// Command names must be non-empty, at most 64 characters, and
    /// contain only alphanumeric characters, hyphens, and underscores.
    #[error("Invalid command name: '{0}'")]
    InvalidCommandName(String),

    /// Duplicate command registration.
    ///
    /// Each command name must be unique within the registry. This
    /// error is returned when attempting to register a command with
    /// a name that is already in use.
    #[error("Duplicate command: '{0}' is already registered")]
    DuplicateCommand(String),

    /// Command not found.
    ///
    /// This error is returned when attempting to execute or lookup
    /// a command that does not exist in the registry.
    #[error("Command not found: '{0}'")]
    CommandNotFound(String),

    /// Argument too large.
    ///
    /// Individual command arguments are limited to `MAX_ARG_SIZE`
    /// bytes to prevent memory exhaustion.
    #[error("Argument too large: {size} bytes exceeds maximum of {max} bytes")]
    ArgumentTooLarge { size: usize, max: usize },

    /// Registry lock poisoned.
    ///
    /// The internal `RwLock` has been poisoned due to a panic in
    /// another thread. This indicates a serious concurrency issue.
    #[error("Registry lock poisoned")]
    RegistryLockPoisoned,

    /// Authorization required.
    ///
    /// The command requires authentication but the user is not
    /// authenticated. This is a soft error that may be handled
    /// by the CLI system.
    #[error("Authorization required for command: '{0}'")]
    AuthorizationRequired(String),
}

/// Validates a command name against security rules.
///
/// This function checks that the command name:
/// - Is not empty
/// - Is at most 64 characters long
/// - Contains only alphanumeric characters, hyphens, and underscores
/// - Does not match any reserved built-in command name (case-insensitive)
///
/// # Arguments
///
/// * `name` - The command name to validate
///
/// # Returns
///
/// `Ok(())` if the command name is valid, or a `SecurityError` if it
/// fails any of the validation checks.
///
/// # Examples
///
/// ```
/// use aisopod_plugin::security::validate_command_name;
///
/// // Valid command names
/// assert!(validate_command_name("mystatus").is_ok());
/// assert!(validate_command_name("my-plugin").is_ok());
/// assert!(validate_command_name("test_command_123").is_ok());
///
/// // Reserved command names are rejected
/// assert!(validate_command_name("help").is_err());
/// assert!(validate_command_name("HELP").is_err());
/// assert!(validate_command_name("status").is_err());
///
/// // Invalid formats are rejected
/// assert!(validate_command_name("").is_err());
/// assert!(validate_command_name(&"a".repeat(65)).is_err());
/// assert!(validate_command_name("bad!char").is_err());
/// ```
pub fn validate_command_name(name: &str) -> Result<(), SecurityError> {
    // Check for empty name
    if name.is_empty() {
        return Err(SecurityError::InvalidCommandName(name.to_string()));
    }

    // Check maximum length (64 characters)
    if name.len() > 64 {
        return Err(SecurityError::InvalidCommandName(name.to_string()));
    }

    // Check for valid characters (alphanumeric, hyphen, underscore)
    if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(SecurityError::InvalidCommandName(name.to_string()));
    }

    // Check against reserved command names (case-insensitive)
    let lower = name.to_lowercase();
    if RESERVED_COMMANDS.contains(&lower.as_str()) {
        return Err(SecurityError::ReservedCommandName(name.to_string()));
    }

    Ok(())
}

/// Sanitizes a command-line argument.
///
/// This function enforces two security constraints:
/// 1. The argument size must not exceed `MAX_ARG_SIZE` bytes
/// 2. Control characters are removed (except newline and tab)
///
/// # Arguments
///
/// * `arg` - The argument string to sanitize
///
/// # Returns
///
/// A sanitized `String` with control characters removed, or a
/// `SecurityError::ArgumentTooLarge` if the argument exceeds the
/// maximum size.
///
/// # Examples
///
/// ```
/// use aisopod_plugin::security::sanitize_argument;
///
/// // Normal arguments are unchanged
/// assert_eq!(sanitize_argument("hello").unwrap(), "hello");
/// assert_eq!(sanitize_argument("arg with spaces").unwrap(), "arg with spaces");
///
/// // Control characters are removed
/// assert_eq!(sanitize_argument("hello\x00world").unwrap(), "helloworld");
/// assert_eq!(sanitize_argument("line1\x0Bline2").unwrap(), "line1line2");
///
/// // Newline and tab are preserved
/// assert_eq!(sanitize_argument("line1\nline2").unwrap(), "line1\nline2");
/// assert_eq!(sanitize_argument("col1\tcol2").unwrap(), "col1\tcol2");
///
/// // Large arguments are rejected
/// let large_arg = "a".repeat(4097);
/// assert!(sanitize_argument(&large_arg).is_err());
/// ```
pub fn sanitize_argument(arg: &str) -> Result<String, SecurityError> {
    // Check size limit
    if arg.len() > MAX_ARG_SIZE {
        return Err(SecurityError::ArgumentTooLarge {
            size: arg.len(),
            max: MAX_ARG_SIZE,
        });
    }

    // Remove control characters except newline and tab
    let sanitized: String = arg
        .chars()
        .filter(|c| {
            if c.is_control() {
                // Allow newline and tab
                *c == '\n' || *c == '\t'
            } else {
                true
            }
        })
        .collect();

    Ok(sanitized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reserved_commands_not_empty() {
        assert!(!RESERVED_COMMANDS.is_empty());
        assert!(RESERVED_COMMANDS.len() >= 70);
    }

    #[test]
    fn test_reserved_commands_case_insensitive() {
        // All reserved commands should be lowercase
        for cmd in RESERVED_COMMANDS {
            assert_eq!(*cmd, cmd.to_lowercase());
        }
    }

    #[test]
    fn test_max_arg_size() {
        assert_eq!(MAX_ARG_SIZE, 4096);
    }

    #[test]
    fn test_validate_command_name_valid() {
        // Basic valid names
        assert!(validate_command_name("myplugin").is_ok());
        assert!(validate_command_name("help").is_err()); // reserved
        assert!(validate_command_name("my-plugin").is_ok());
        assert!(validate_command_name("test_command").is_ok());
        assert!(validate_command_name("a1b2c3").is_ok());
        assert!(validate_command_name("a-b_c").is_ok());

        // Edge cases
        assert!(validate_command_name("a").is_ok());
        assert!(validate_command_name(&"a".repeat(64)).is_ok());
    }

    #[test]
    fn test_validate_command_name_empty() {
        let result = validate_command_name("");
        assert!(matches!(result, Err(SecurityError::InvalidCommandName(_))));
    }

    #[test]
    fn test_validate_command_name_too_long() {
        let result = validate_command_name(&"a".repeat(65));
        assert!(matches!(result, Err(SecurityError::InvalidCommandName(_))));
    }

    #[test]
    fn test_validate_command_name_invalid_chars() {
        let result = validate_command_name("bad!char");
        assert!(matches!(result, Err(SecurityError::InvalidCommandName(_))));

        let result = validate_command_name("bad char");
        assert!(matches!(result, Err(SecurityError::InvalidCommandName(_))));

        let result = validate_command_name("bad@char");
        assert!(matches!(result, Err(SecurityError::InvalidCommandName(_))));

        let result = validate_command_name("bad#char");
        assert!(matches!(result, Err(SecurityError::InvalidCommandName(_))));
    }

    #[test]
    fn test_validate_command_name_reserved() {
        // Test a few reserved commands
        let reserved = ["help", "status", "config", "plugin", "version"];
        for cmd in reserved {
            let result = validate_command_name(cmd);
            assert!(matches!(result, Err(SecurityError::ReservedCommandName(_))));
        }
    }

    #[test]
    fn test_validate_command_name_reserved_case_insensitive() {
        // Case variations should also be rejected
        let reserved = ["HELP", "Help", "STATUS", "Config", "PLUGIN"];
        for cmd in reserved {
            let result = validate_command_name(cmd);
            assert!(
                matches!(result, Err(SecurityError::ReservedCommandName(_))),
                "Expected ReservedCommandName error for '{}', got {:?}", cmd, result
            );
        }
    }

    #[test]
    fn test_sanitize_argument_valid() {
        // Normal arguments should pass
        assert_eq!(sanitize_argument("hello").unwrap(), "hello");
        assert_eq!(sanitize_argument("hello world").unwrap(), "hello world");
        assert_eq!(sanitize_argument("-flag value").unwrap(), "-flag value");
    }

    #[test]
    fn test_sanitize_argument_control_chars() {
        // Null bytes should be removed
        assert_eq!(sanitize_argument("hello\x00world").unwrap(), "helloworld");

        // Other control characters should be removed
        assert_eq!(sanitize_argument("line1\x0Bline2").unwrap(), "line1line2");
        assert_eq!(sanitize_argument("tab\x09char").unwrap(), "tab\tchar");
    }

    #[test]
    fn test_sanitize_argument_preserve_newline_tab() {
        // Newline and tab should be preserved
        assert_eq!(sanitize_argument("line1\nline2").unwrap(), "line1\nline2");
        assert_eq!(sanitize_argument("col1\tcol2").unwrap(), "col1\tcol2");
    }

    #[test]
    fn test_sanitize_argument_too_large() {
        // Arguments exceeding MAX_ARG_SIZE should fail
        let large_arg = "a".repeat(MAX_ARG_SIZE + 1);
        let result = sanitize_argument(&large_arg);
        assert!(matches!(result, Err(SecurityError::ArgumentTooLarge { .. })));

        // Exactly MAX_ARG_SIZE should succeed
        let exact_arg = "a".repeat(MAX_ARG_SIZE);
        assert!(sanitize_argument(&exact_arg).is_ok());
    }

    #[test]
    fn test_security_error_debug() {
        // Verify that all error variants implement Debug
        let e = SecurityError::ReservedCommandName("test".to_string());
        let debug_str = format!("{:?}", e);
        assert!(debug_str.contains("ReservedCommandName"));

        let e = SecurityError::InvalidCommandName("test".to_string());
        let debug_str = format!("{:?}", e);
        assert!(debug_str.contains("InvalidCommandName"));

        let e = SecurityError::DuplicateCommand("test".to_string());
        let debug_str = format!("{:?}", e);
        assert!(debug_str.contains("DuplicateCommand"));

        let e = SecurityError::CommandNotFound("test".to_string());
        let debug_str = format!("{:?}", e);
        assert!(debug_str.contains("CommandNotFound"));

        let e = SecurityError::ArgumentTooLarge {
            size: 100,
            max: 50,
        };
        let debug_str = format!("{:?}", e);
        assert!(debug_str.contains("ArgumentTooLarge"));

        let e = SecurityError::RegistryLockPoisoned;
        let debug_str = format!("{:?}", e);
        assert!(debug_str.contains("RegistryLockPoisoned"));

        let e = SecurityError::AuthorizationRequired("test".to_string());
        let debug_str = format!("{:?}", e);
        assert!(debug_str.contains("AuthorizationRequired"));
    }

    #[test]
    fn test_security_error_display() {
        // Verify error messages are descriptive
        let e = SecurityError::ReservedCommandName("help".to_string());
        assert!(e.to_string().contains("Reserved command name"));

        let e = SecurityError::InvalidCommandName("test".to_string());
        assert!(e.to_string().contains("Invalid command name"));

        let e = SecurityError::DuplicateCommand("test".to_string());
        assert!(e.to_string().contains("Duplicate command"));

        let e = SecurityError::CommandNotFound("test".to_string());
        assert!(e.to_string().contains("Command not found"));

        let e = SecurityError::ArgumentTooLarge { size: 100, max: 50 };
        assert!(e.to_string().contains("Argument too large"));
    }
}
