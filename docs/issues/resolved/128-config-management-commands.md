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

## Resolution

The issue was resolved with the following changes:

### Files Created
- `crates/aisopod/src/commands/config.rs` - Complete implementation of config management commands

### Files Modified

#### `crates/aisopod/src/cli.rs`
- Updated `Commands::Config` to use `ConfigArgs` instead of being a simple variant
- Added dispatch for the config command in `run_cli()`

#### `crates/aisopod/src/commands/mod.rs`
- Added `pub mod config;` to export the config module

#### `crates/aisopod-config/src/types/channels.rs`
- Added platform-specific configuration structs for telegram, discord, whatsapp, slack, github, gitlab, bitbucket, mattermost, and matrix
- Updated `ChannelsConfig` to include these platform-specific configurations

#### `crates/aisopod-config/src/types/models.rs`
- Added `default_provider` field to `ModelsConfig`

#### `crates/aisopod-config/src/types/mod.rs`
- Exported `Model`, `ModelProvider` types from the models module

#### `crates/aisopod/Cargo.toml`
- Added `rpassword` dependency for secure password input

### Implementation Details

1. **ConfigArgs and ConfigCommands**: Defined using clap's Args and Subcommand macros for clean CLI parsing

2. **show_config**: Converts config to a flat map and displays all values with sensitive fields redacted using "***REDACTED***"

3. **set_config**: Uses JSON serialization to navigate and modify configuration values by dot-separated key paths

4. **run_wizard**: Interactive wizard that guides users through setting gateway settings, selecting a model provider, and entering API keys

5. **configure_channels**: Helper for configuring individual channel tokens interactively

### Testing
- All 12 tests pass successfully
- Tests cover config args parsing, sensitive field detection, and configuration loading

---

*Created: 2026-02-15*
*Resolved: 2026-02-24*
