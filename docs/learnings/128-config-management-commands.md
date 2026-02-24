# Learning: Config Management Commands Implementation

This document captures key learnings from implementing the config management commands (Issue #128).

## Learning 1: Using BTreeMap for Ordered Configuration Output

When displaying configuration to users, we need predictable ordering. Using `BTreeMap<String, String>` instead of `HashMap<String, String>` or `serde_json::Map<String, String>` provides:

- **Sorted keys**: Alphabetically sorted output for predictable display
- **Standard library**: Well-documented API with no external dependencies
- **Insert method**: Standard `insert()` method works reliably

```rust
fn config_to_display_map(config: &AisopodConfig) -> BTreeMap<String, String> {
    let mut result: BTreeMap<String, String> = BTreeMap::new();
    // ... flatten logic ...
    result
}
```

## Learning 2: Handling Move Semantics in Configuration Updates

When modifying configuration in-place, `serde_json::to_value()` takes ownership of the value. This causes "use after move" errors.

### Problem

```rust
fn set_config(config: &mut AisopodConfig, key: &str, value: &str) -> Result<()> {
    let json_value = serde_json::to_value(config)?;  // config moved here
    // ... modify json_value ...
    *config = new_config;  // Error: config was moved
}
```

### Solution

Clone the config first to avoid moving:

```rust
fn set_config(config: &mut AisopodConfig, key: &str, value: &str) -> Result<()> {
    let mut cloned_config = config.clone();
    let mut json_value = serde_json::to_value(&cloned_config)?;
    
    // ... modify json_value ...
    
    *config = cloned_config;  // Update original
    Ok(())
}
```

## Learning 3: Secure Password Input with rpassword

For sensitive input like API keys and tokens, we need to mask the input. The `rpassword` crate provides this cross-platform:

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

### Dependencies

Added to `Cargo.toml`:

```toml
rpassword = "7.4.0"
```

## Learning 4: JSON Path Navigation for Nested Configuration

To support dot-separated paths like "gateway.server.port", we need to navigate nested JSON objects:

```rust
fn set_config(config: &mut AisopodConfig, key: &str, value: &str) -> Result<()> {
    let parts: Vec<&str> = key.split('.').collect();
    
    // Navigate to parent object
    let mut target = &mut json_value;
    for &part in parts.iter().take(parts.len() - 1) {
        target = target
            .get_mut(part)
            .ok_or_else(|| anyhow!("Invalid key path: {} (no such field)", key))?;
    }
    
    // Set final value
    let final_key = parts[parts.len() - 1];
    *target.get_mut(final_key).ok_or_else(...)? = Value::String(value.to_string());
    
    Ok(())
}
```

## Learning 5: Exporting Types for CLI Integration

The CLI needs access to configuration types but they weren't exported. We had to:

1. Add exports to `types/mod.rs`:
```rust
pub use models::Model;
pub use models::ModelProvider;
```

2. Import in the CLI module:
```rust
use aisopod_config::types::ModelProvider;
```

## Learning 6: Platform-Specific Configuration

To support multiple messaging platforms, we created platform-specific config structs:

```rust
pub struct ChannelsConfig {
    pub telegram: TelegramConfig,
    pub discord: DiscordConfig,
    pub whatsapp: WhatsappConfig,
    pub slack: SlackConfig,
    // ... other platforms
}

pub struct TelegramConfig {
    pub token: Option<Sensitive<String>>,
}

pub struct DiscordConfig {
    pub token: Option<Sensitive<String>>,
    pub client_secret: Option<Sensitive<String>>,
}
```

This provides type safety and clear documentation for each platform's configuration needs.

## Learning 7: Interactive CLI Prompts

We implemented several helper functions for interactive configuration:

- `prompt()`: Basic input with echo
- `prompt_with_default()`: Input with default value suggestion
- `prompt_password()`: Masked input for sensitive data
- `prompt_select()`: Numbered menu selection

These provide a good user experience for the configuration wizard.

## Learning 8: Testing Interactive Functions

Interactive functions are hard to test automatically. We created tests that verify:

- Function signatures and return types
- Sensitive field detection logic
- Configuration loading behavior
- Key path parsing

For interactive tests, we skip or use manual verification.
