# Issue 128: Implement Config Management Commands

## Summary
Implement the `aisopod config` subcommands for displaying, updating, and interactively configuring the application settings, including a setup wizard and channel configuration helper.

## Location
- Crate: `aisopod` (main binary crate)
- File: `src/commands/config.rs`

## Current Behavior
The config subcommand is a stub that panics with `todo!`. There is no CLI interface for managing configuration.

## Expected Behavior
Users can view the current configuration (with sensitive fields redacted), set individual configuration values, run an interactive setup wizard for first-time configuration, and interactively configure channels.

## Impact
Configuration management is essential for initial setup and ongoing maintenance. The wizard provides a guided experience for new users, reducing the barrier to entry.

## Suggested Implementation

1. Define the config subcommand and its nested subcommands:

```rust
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommands,
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Display current configuration
    Show,
    /// Set a configuration value
    Set {
        /// Configuration key (dot-separated path)
        key: String,
        /// New value
        value: String,
    },
    /// Run interactive setup wizard
    Wizard,
    /// Interactive channel configuration
    Channels,
}
```

2. Implement the `show` handler with sensitive field redaction:

```rust
fn show_config(config: &AppConfig) -> anyhow::Result<()> {
    let display = config.to_display_map();
    for (key, value) in &display {
        let redacted = if config.is_sensitive_field(key) {
            "***REDACTED***".to_string()
        } else {
            value.clone()
        };
        println!("{}: {}", key, redacted);
    }
    Ok(())
}
```

3. Implement the `set` handler:

```rust
fn set_config(config: &mut AppConfig, key: &str, value: &str) -> anyhow::Result<()> {
    config.set_value(key, value)?;
    config.save()?;
    println!("Set {} = {}", key, value);
    Ok(())
}
```

4. Implement the interactive wizard:

```rust
fn run_wizard(config: &mut AppConfig) -> anyhow::Result<()> {
    println!("=== aisopod Configuration Wizard ===\n");

    // Step 1: Gateway settings
    let bind = prompt_with_default("Gateway bind address", "127.0.0.1")?;
    let port = prompt_with_default("Gateway port", "3000")?;
    config.set_value("gateway.bind", &bind)?;
    config.set_value("gateway.port", &port)?;

    // Step 2: Default model provider
    let provider = prompt_select("Select model provider", &["openai", "anthropic", "local"])?;
    config.set_value("models.default_provider", &provider)?;

    // Step 3: API key
    let api_key = prompt_password("API key: ")?;
    config.set_value(&format!("auth.{}.api_key", provider), &api_key)?;

    config.save()?;
    println!("\nConfiguration saved successfully!");
    Ok(())
}
```

5. Implement the channels configuration helper:

```rust
fn configure_channels(config: &mut AppConfig) -> anyhow::Result<()> {
    let channel = prompt_select("Select channel to configure", &["telegram", "discord", "whatsapp", "slack"])?;

    match channel.as_str() {
        "telegram" => {
            let token = prompt_password("Telegram bot token: ")?;
            config.set_value("channels.telegram.token", &token)?;
        }
        "discord" => {
            let token = prompt_password("Discord bot token: ")?;
            config.set_value("channels.discord.token", &token)?;
        }
        // ... other channels
        _ => {}
    }

    config.save()?;
    println!("Channel '{}' configured.", channel);
    Ok(())
}
```

## Dependencies
- Issue 124 (clap CLI framework)
- Issue 016 (configuration types)
- Issue 022 (sensitive field handling)

## Acceptance Criteria
- [ ] `aisopod config show` displays all configuration with sensitive fields redacted
- [ ] `aisopod config set <key> <value>` updates and persists a configuration value
- [ ] `aisopod config wizard` walks through interactive setup and saves results
- [ ] `aisopod config channels` guides channel-specific configuration
- [ ] Invalid keys produce helpful error messages
- [ ] Configuration file is created if it does not exist

---
*Created: 2026-02-15*
