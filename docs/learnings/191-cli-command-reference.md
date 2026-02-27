# CLI Command Reference Implementation Learnings

## Overview

This document captures key learnings from implementing the CLI Command Reference for aisopod (Issue 191).

## Project Structure

### CLI Command Organization

The aisopod CLI uses a hierarchical structure with clap for argument parsing:

```
aisopod/
├── src/
│   ├── cli.rs                    # Top-level CLI definition
│   └── commands/                 # Command modules
│       ├── gateway.rs           # Gateway management
│       ├── agent.rs             # Agent management
│       ├── message.rs           # Message sending
│       ├── config.rs            # Configuration management
│       ├── status.rs            # Status and health checks
│       ├── models.rs            # Model management
│       ├── channels.rs          # Channel management
│       ├── sessions.rs          # Session management
│       ├── daemon.rs            # Daemon management
│       ├── doctor.rs            # System diagnostics
│       ├── auth.rs              # Authentication management
│       ├── completions.rs       # Shell completions
│       ├── onboarding.rs        # Onboarding wizard
│       └── migrate.rs           # Configuration migration
```

### Command Patterns

All commands follow a consistent pattern:

1. **Args struct** - Defines command-line arguments using `#[derive(Args)]`
2. **Commands enum** - Defines subcommands using `#[derive(Subcommand)]`
3. **Run function** - Implements the command logic with `config_path` support
4. **Async support** - Commands that need async use `tokio::runtime::Runtime`

Example pattern:
```rust
#[derive(Args)]
pub struct CommandArgs {
    #[command(subcommand)]
    pub command: Commands,
}

pub fn run(args: CommandArgs, config_path: Option<String>) -> Result<()> {
    // Load config with default path fallback
    let config_path_ref = config_path.as_deref();
    let mut config = load_config_or_default(config_path_ref)?;
    
    // Handle subcommands
    match args.command {
        Commands::Subcommand { .. } => {
            // Implementation
        }
    }
}
```

### Configuration Loading

The project uses a consistent pattern for configuration loading:

```rust
fn load_config_or_default(config_path: Option<&str>) -> Result<AisopodConfig> {
    match config_path {
        Some(path) => {
            let config_path = Path::new(path);
            load_config(config_path).map_err(|e| {
                anyhow!("Failed to load configuration from '{}': {}", path, e)
            })
        }
        None => {
            // Use default config path
            let default_path = aisopod_config::default_config_path();
            if default_path.exists() {
                load_config(&default_path).map_err(|e| {
                    anyhow!("Failed to load configuration from '{}': {}", default_path.display(), e)
                })
            } else {
                // If no config file exists, return empty config
                Ok(AisopodConfig::default())
            }
        }
    }
}
```

This pattern:
- Supports overriding config path via CLI
- Falls back to default config path
- Returns default config if no config file exists
- Provides informative error messages

### Interactive Prompts

The project uses `rpassword` crate for secure password input:

```rust
fn prompt_password(prompt_text: &str) -> Result<String> {
    #[cfg(unix)]
    {
        let password = rpassword::prompt_password(prompt_text)?;
        Ok(password)
    }
    #[cfg(not(unix))]
    {
        print!("{}", prompt_text);
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Ok(input.trim().to_string())
    }
}
```

This pattern:
- Uses platform-appropriate secure input for passwords
- Falls back to regular input on non-Unix systems
- Properly flushes stdout before prompting

## Documentation Practices

### Command Documentation Template

Each command should be documented with:
1. Description
2. Usage syntax
3. All arguments with descriptions
4. All options with defaults
5. Environment variable overrides
6. Practical examples

### Shell Completion Support

The project supports multiple shells for completions:
- Bash
- Zsh
- Fish
- PowerShell
- Elvish

Completions are generated using `clap_complete` and the `Shell` enum from clap.

### Health Check Patterns

The health check command verifies:
- Gateway reachability
- Configuration validity
- Required resources (agents, auth, etc.)

This pattern ensures users can quickly diagnose issues.

## Common Patterns and Best Practices

### Error Handling

Use `anyhow::Result` for error propagation with context:

```rust
load_config(config_path).map_err(|e| {
    anyhow!("Failed to load configuration from '{}': {}", path, e)
})
```

### Output Formatting

The project uses `Output` struct for JSON vs human-readable output:

```rust
let output = Output::new(json);
if json {
    // JSON output
} else {
    // Human-readable output
}
```

### Template System

The channel creation feature uses a template system with variable substitution:

```rust
content.replace("{{name}}", name)
    .replace("{{pascal_name}}", pascal_name)
    .replace("{{display_name}}", display_name)
```

This allows flexible code generation from templates.

## Key Learnings

### 1. Consistency is Key

The CLI structure is highly consistent across commands:
- Same argument pattern (`Args`, `Commands`, `run`)
- Same config loading pattern
- Same error handling approach

This makes the codebase maintainable and predictable.

### 2. Documentation Should Match Code

The CLI reference should be updated when:
- New commands are added
- Existing command options change
- Behavior changes

### 3. Interactive Commands Need Special Testing

Commands with interactive prompts should use `#[ignore]` in tests:

```rust
#[test]
#[ignore = "requires stdin interaction - would hang in CI"]
fn test_prompt_with_default_empty() {
    // Test implementation
}
```

### 4. Platform-Specific Code

Use `#[cfg(target_os = "...")]` for platform-specific features:

```rust
#[cfg(target_os = "linux")]
pub fn install_daemon(args: InstallArgs) -> Result<()> {
    // Linux-specific implementation
}

#[cfg(target_os = "macos")]
pub fn install_daemon(args: InstallArgs) -> Result<()> {
    // macOS-specific implementation
}
```

### 5. Environment Variable Mapping

When migrating from other systems, create explicit mappings:

```rust
pub fn env_var_mapping() -> Vec<(&'static str, &'static str)> {
    vec![
        ("OPENCLAW_SERVER_PORT", "AISOPOD_GATEWAY_SERVER_PORT"),
        // ... more mappings
    ]
}
```

## Future Improvements

1. **Auto-generate documentation**: Consider using `clap::Command::write_help` or a similar approach to generate help text programmatically.

2. **Command validation**: Add more robust validation of command arguments.

3. **Progress feedback**: For long-running operations, consider adding progress indicators using `indicatif`.

4. **Configuration schema**: Document the configuration schema for better IDE autocomplete support.

5. **API versioning**: Document API version changes and migration paths.

---

*Created: 2026-02-27*
*Issue: 191*
