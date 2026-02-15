# Issue 114: Implement Plugin CLI Command Registration with Security Hardening

## Summary
Allow plugins to register custom CLI subcommands via `PluginApi` while enforcing security measures including reserved command name protection, argument sanitization, authorization checks, and registry locking during command execution.

## Location
- Crate: `aisopod-plugin`
- File: `crates/aisopod-plugin/src/commands.rs`, `crates/aisopod-plugin/src/security.rs`

## Current Behavior
The `PluginApi` (Issue 108) accepts command registrations via `register_command()` and the `PluginCommand` type exists, but there is no validation, security hardening, or dispatch mechanism for plugin-contributed CLI commands.

## Expected Behavior
Plugin commands are validated against a list of ~72 reserved built-in command names that cannot be overridden. Arguments are sanitized (max 4KB size, control character removal). Commands that require authorization are gated behind an auth check. The command registry is locked during execution to prevent concurrent modification.

## Impact
Without security hardening, malicious or buggy plugins could override critical built-in commands, inject control characters into terminal output, or cause race conditions during command execution.

## Suggested Implementation
1. **Define reserved command names in `security.rs`:**
   ```rust
   /// Built-in command names that plugins are not allowed to override.
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
   pub const MAX_ARG_SIZE: usize = 4096;
   ```
2. **Implement argument sanitization:**
   ```rust
   pub fn sanitize_argument(arg: &str) -> Result<String, SecurityError> {
       if arg.len() > MAX_ARG_SIZE {
           return Err(SecurityError::ArgumentTooLarge {
               size: arg.len(),
               max: MAX_ARG_SIZE,
           });
       }
       // Remove control characters (except newline and tab)
       let sanitized: String = arg
           .chars()
           .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
           .collect();
       Ok(sanitized)
   }
   ```
3. **Implement command validation:**
   ```rust
   pub fn validate_command_name(name: &str) -> Result<(), SecurityError> {
       let lower = name.to_lowercase();
       if RESERVED_COMMANDS.contains(&lower.as_str()) {
           return Err(SecurityError::ReservedCommandName(name.to_string()));
       }
       if name.is_empty() || name.len() > 64 {
           return Err(SecurityError::InvalidCommandName(name.to_string()));
       }
       // Only allow alphanumeric characters, hyphens, and underscores
       if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
           return Err(SecurityError::InvalidCommandName(name.to_string()));
       }
       Ok(())
   }
   ```
4. **Implement `CommandRegistry` with locking in `commands.rs`:**
   ```rust
   use std::collections::HashMap;
   use std::sync::RwLock;
   use crate::PluginCommand;

   pub struct CommandRegistry {
       commands: RwLock<HashMap<String, PluginCommand>>,
   }

   impl CommandRegistry {
       pub fn new() -> Self {
           Self {
               commands: RwLock::new(HashMap::new()),
           }
       }

       pub fn register(&self, command: PluginCommand) -> Result<(), SecurityError> {
           validate_command_name(&command.name)?;
           let mut cmds = self.commands.write()
               .map_err(|_| SecurityError::RegistryLockPoisoned)?;
           if cmds.contains_key(&command.name) {
               return Err(SecurityError::DuplicateCommand(command.name.clone()));
           }
           cmds.insert(command.name.clone(), command);
           Ok(())
       }

       pub fn execute(&self, name: &str, args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
           let sanitized_args: Vec<String> = args
               .iter()
               .map(|a| sanitize_argument(a))
               .collect::<Result<Vec<_>, _>>()?;

           let cmds = self.commands.read()
               .map_err(|_| SecurityError::RegistryLockPoisoned)?;
           let cmd = cmds.get(name)
               .ok_or_else(|| SecurityError::CommandNotFound(name.to_string()))?;

           if cmd.require_auth {
               // Authorization check placeholder — integrate with auth system
               tracing::debug!(command = %name, "Authorization check required");
           }

           (cmd.handler)(&sanitized_args)
       }
   }
   ```
5. **Define `SecurityError`:**
   ```rust
   #[derive(Debug, thiserror::Error)]
   pub enum SecurityError {
       #[error("Reserved command name: '{0}' cannot be overridden by plugins")]
       ReservedCommandName(String),
       #[error("Invalid command name: '{0}'")]
       InvalidCommandName(String),
       #[error("Duplicate command: '{0}' is already registered")]
       DuplicateCommand(String),
       #[error("Command not found: '{0}'")]
       CommandNotFound(String),
       #[error("Argument too large: {size} bytes exceeds maximum of {max} bytes")]
       ArgumentTooLarge { size: usize, max: usize },
       #[error("Registry lock poisoned")]
       RegistryLockPoisoned,
   }
   ```

## Dependencies
- Issue 108 (PluginApi for capability registration)
- Issue 110 (PluginRegistry lifecycle management)

## Acceptance Criteria
- [ ] Plugins can register custom CLI subcommands via `PluginApi`
- [ ] ~72 reserved built-in command names are protected from override
- [ ] Attempting to register a reserved name returns `SecurityError::ReservedCommandName`
- [ ] Arguments are sanitized: max 4KB size enforced, control characters removed
- [ ] Commands with `require_auth = true` trigger authorization checks
- [ ] `CommandRegistry` uses `RwLock` for safe concurrent access
- [ ] Duplicate command registration is detected and rejected
- [ ] `SecurityError` provides descriptive error variants
- [ ] `cargo build -p aisopod-plugin` compiles without errors

---
*Created: 2026-02-15*
