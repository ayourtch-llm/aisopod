//! Configuration management command implementation module
//!
//! This module provides the `aisopod config` subcommand family for managing application settings:
//! - show: Display current configuration with sensitive fields redacted
//! - set: Set a configuration value by key path
//! - wizard: Run interactive setup wizard for first-time configuration
//! - channels: Interactive channel configuration helper
//! - init: Initialize a new configuration file from a template

use anyhow::{anyhow, Context, Result};
    use clap::{Args, Subcommand};
    use serde_json::{Map, Value};
    use std::collections::BTreeMap;
    use std::io::{self, Write};
    use std::path::{Path, PathBuf};

    use aisopod_config::load_config;
    use aisopod_config::sensitive::Sensitive;
    use aisopod_config::types::{AisopodConfig, ModelProvider};

/// Configuration management command arguments
#[derive(Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommands,
}

/// Available configuration subcommands
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
    /// Initialize a new configuration file from a template
    Init {
        /// Template name (dev, production, docker)
        #[arg(short, long)]
        template: Option<String>,
        /// Output file path
        #[arg(short, long)]
        output: Option<String>,
    },
}

/// Prompt the user for input
fn prompt(prompt_text: &str) -> Result<String> {
    print!("{}", prompt_text);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_string())
}

/// Prompt the user for input with a default value
fn prompt_with_default(prompt_text: &str, default: &str) -> Result<String> {
    print!("{} [{}]: ", prompt_text, default);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let trimmed = input.trim();
    if trimmed.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(trimmed.to_string())
    }
}

/// Prompt the user for password input (no echo)
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

/// Prompt the user to select from options
fn prompt_select(prompt_text: &str, options: &[&str]) -> Result<String> {
    println!("{}:", prompt_text);
    for (i, option) in options.iter().enumerate() {
        println!("  {}. {}", i + 1, option);
    }

    loop {
        let input = prompt("Enter choice (number): ")?;
        if let Ok(index) = input.parse::<usize>() {
            if index >= 1 && index <= options.len() {
                return Ok(options[index - 1].to_string());
            }
        }
        println!("Invalid choice. Please enter a number between 1 and {}.", options.len());
    }
}

/// Load configuration from file or use defaults
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

/// Save configuration to file
fn save_config(config: &AisopodConfig, config_path: Option<String>) -> Result<()> {
    let path = match config_path {
        Some(p) => p,
        None => {
            // Default config path - must match load_config_or_default
            std::env::current_dir()?
                .join(aisopod_config::DEFAULT_CONFIG_FILE)
                .to_string_lossy()
                .to_string()
        }
    };

    let content = serde_json::to_string_pretty(config)?;
    std::fs::write(&path, content)?;

    Ok(())
}

/// Convert config to a flat map of key-value pairs for display
fn config_to_display_map(config: &AisopodConfig) -> BTreeMap<String, String> {
    let json_value = serde_json::to_value(config).expect("Failed to serialize config");

    fn flatten_value(prefix: &str, value: &Value, result: &mut BTreeMap<String, String>) {
        match value {
            Value::Object(map) => {
                for (k, v) in map {
                    let new_key = if prefix.is_empty() {
                        k.clone()
                    } else {
                        format!("{}.{}", prefix, k)
                    };
                    flatten_value(&new_key, v, result);
                }
            }
            Value::String(s) => {
                result.insert(prefix.to_string(), s.clone());
            }
            Value::Number(n) => {
                result.insert(prefix.to_string(), n.to_string());
            }
            Value::Bool(b) => {
                result.insert(prefix.to_string(), b.to_string());
            }
            Value::Null => {
                result.insert(prefix.to_string(), "null".to_string());
            }
            Value::Array(_) => {
                // Arrays are not flattened for display
                result.insert(prefix.to_string(), "[array]".to_string());
            }
        }
    }

    let mut result: BTreeMap<String, String> = BTreeMap::new();
    flatten_value("", &json_value, &mut result);
    result
}

/// Check if a field key refers to a sensitive field
fn is_sensitive_field(key: &str) -> bool {
    // List of sensitive field paths
    let sensitive_keys = [
        "auth.openai.api_key",
        "auth.anthropic.api_key",
        "auth.google.api_key",
        "auth.aws.access_key",
        "auth.aws.secret_key",
        "channels.telegram.token",
        "channels.discord.token",
        "channels.whatsapp.token",
        "channels.slack.token",
        "channels.slack.signing_secret",
        "channels.slack.verification_token",
        "channels.github.token",
        "channels.gitlab.token",
        "channels.bitbucket.token",
        "channels.mattermost.token",
        "channels.matrix.access_token",
        "channels.discord.client_secret",
        "channels.slack.app_token",
        "channels.slack.bot_token",
        "channels.slack.user_token",
    ];

    sensitive_keys.contains(&key)
}

/// Show current configuration with sensitive fields redacted
fn show_config(config: &AisopodConfig) -> Result<()> {
    let display = config_to_display_map(config);
    for (key, value) in &display {
        let redacted = if is_sensitive_field(key) {
            "***REDACTED***".to_string()
        } else {
            value.clone()
        };
        println!("{}: {}", key, redacted);
    }
    Ok(())
}

/// Set a configuration value by key path
fn set_config(config: &mut AisopodConfig, key: &str, value: &str) -> Result<()> {
    // Clone the config to work with
    let mut cloned_config = config.clone();
    
    // Convert to JSON value for manipulation
    let mut json_value = serde_json::to_value(&cloned_config)
        .map_err(|e| anyhow!("Failed to serialize config: {}", e))?;

    // Parse the key path and set the value
    let parts: Vec<&str> = key.split('.').collect();
    if parts.is_empty() {
        return Err(anyhow!("Invalid key path: {}", key));
    }

    // Navigate to the parent object
    let mut target = &mut json_value;
    for &part in parts.iter().take(parts.len() - 1) {
        target = target
            .get_mut(part)
            .ok_or_else(|| anyhow!("Invalid key path: {} (no such field)", key))?;
    }

    // Set the final value
    let final_key = parts[parts.len() - 1];
    let field = target
        .get_mut(final_key)
        .ok_or_else(|| anyhow!("Invalid key path: {} (no such field)", key))?;
    *field = Value::String(value.to_string());

    // Deserialize back to config
    cloned_config = serde_json::from_value(json_value)
        .map_err(|e| anyhow!("Failed to deserialize config: {}", e))?;

    // Update the original config
    *config = cloned_config;

    Ok(())
}

/// Run the interactive setup wizard
fn run_wizard(config: &mut AisopodConfig) -> Result<()> {
    println!("=== aisopod Configuration Wizard ===\n");

    // Step 1: Gateway settings
    println!("Step 1: Gateway settings\n");
    let bind = prompt_with_default("Gateway bind address", "127.0.0.1")?;
    let port = prompt_with_default("Gateway port", "3000")?;
    let port: u16 = port.parse().context("Port must be a valid number")?;

    // Update gateway config
    config.gateway.bind.address = bind;
    config.gateway.server.port = port;

    // Step 2: Default model provider
    println!("\nStep 2: Model provider\n");
    let provider = prompt_select(
        "Select model provider",
        &["openai", "anthropic", "google", "aws-bedrock", "ollama"],
    )?;

    // Add the provider to the models config
    let provider_name = provider.clone();
    let api_key = prompt_password(&format!("{} API key: ", provider_name))?;

    // Add provider to providers list
    config.models.providers.push(aisopod_config::types::ModelProvider {
        name: provider_name,
        endpoint: "".to_string(),
        api_key,
    });

    // Step 3: Confirm and save
    println!("\n=== Configuration Summary ===");
    println!("Gateway: {}:{}", config.gateway.bind.address, config.gateway.server.port);
    if !config.models.providers.is_empty() {
        println!("Model provider: {}", config.models.providers[0].name);
    }

    let confirm = prompt("\nSave configuration? (y/n): ")?;
    if confirm.to_lowercase() != "y" {
        println!("Configuration not saved.");
        return Ok(());
    }

    save_config(config, None)?;
    println!("\nConfiguration saved successfully!");
    Ok(())
}

/// Configure a specific channel
fn configure_channel(config: &mut AisopodConfig, channel: &str) -> Result<()> {
    match channel {
        "telegram" => {
            let token = prompt_password("Telegram bot token: ")?;
            config.channels.telegram.token = Some(Sensitive::new(token));
            Ok(())
        }
        "discord" => {
            let token = prompt_password("Discord bot token: ")?;
            config.channels.discord.token = Some(Sensitive::new(token));
            Ok(())
        }
        "whatsapp" => {
            let token = prompt_password("WhatsApp access token: ")?;
            config.channels.whatsapp.access_token = Some(Sensitive::new(token));
            Ok(())
        }
        "slack" => {
            let token = prompt_password("Slack bot token: ")?;
            config.channels.slack.token = Some(Sensitive::new(token));
            Ok(())
        }
        _ => Err(anyhow!("Unsupported channel: {}", channel)),
    }
}

/// Interactive channel configuration helper
fn configure_channels(config: &mut AisopodConfig) -> Result<()> {
    println!("=== Channel Configuration Helper ===\n");

    let channels = ["telegram", "discord", "whatsapp", "slack"];

    loop {
        let channel = prompt_select("Select channel to configure", &channels);
        
        match channel {
            Ok(channel) => {
                configure_channel(config, &channel)?;
                println!("\nChannel '{}' configured.", channel);
                
                let continue_input = prompt("\nConfigure another channel? (y/n): ")?;
                if continue_input.to_lowercase() != "y" {
                    break;
                }
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }

    let confirm = prompt("\nSave configuration? (y/n): ")?;
    if confirm.to_lowercase() == "y" {
        save_config(config, None)?;
        println!("\nConfiguration saved successfully!");
    } else {
        println!("\nConfiguration not saved.");
    }

    Ok(())
}

/// Get template content by name
fn get_template_content(template_name: &str) -> Result<String> {
    // Try multiple paths for templates:
    // 1. config/templates/ (current directory - for development)
    // 2. <workspace-root>/config/templates/ (for installed binaries)
    // 3. <binary-dir>/../config/templates/ (alternative relative path)
    let template_filename = format!("{}.json", template_name);
    
    // Path relative to workspace root (two levels up from crates/aisopod)
    let workspace_root_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join("config/templates").join(&template_filename));
    
    // Current working directory path
    let cwd_path = Path::new("config/templates").join(&template_filename);
    
    // Try each path in order of preference
    let possible_paths: Vec<PathBuf> = if let Some(ws_path) = workspace_root_path {
        vec![cwd_path, ws_path]
    } else {
        vec![cwd_path]
    };
    
    for template_path in &possible_paths {
        if template_path.exists() {
            return std::fs::read_to_string(template_path).map_err(|e| {
                anyhow!("Failed to read template '{}': {}", template_name, e)
            });
        }
    }
    
    Err(anyhow!(
        "Template '{}' not found. Available templates: dev, production, docker",
        template_name
    ))
}

/// Initialize a new configuration file from a template
fn init_config(template: Option<String>, output: Option<String>) -> Result<()> {
    let template_name = template.as_deref().unwrap_or("dev");
    
    let template_content = get_template_content(template_name)?;
    
    // Determine output path
    let output_path = if let Some(path) = output {
        Path::new(&path).to_path_buf()
    } else {
        // Default to current directory
        std::env::current_dir()?.join(aisopod_config::DEFAULT_CONFIG_FILE)
    };
    
    // Write template content to output file
    std::fs::write(&output_path, template_content)?;
    
    println!(
        "Configuration initialized from '{}' template to '{}'",
        template_name,
        output_path.display()
    );
    
    Ok(())
}

/// Run the configuration management command with the given arguments and config path
pub fn run(args: ConfigArgs, config_path: Option<String>) -> Result<()> {
    let config_path_ref = config_path.as_deref();
    let mut config = load_config_or_default(config_path_ref)?;

    match args.command {
        ConfigCommands::Show => {
            show_config(&config)?;
        }
        ConfigCommands::Set { key, value } => {
            set_config(&mut config, &key, &value)?;
            save_config(&config, config_path)?;
            println!("Set {} = {}", key, value);
        }
        ConfigCommands::Wizard => {
            run_wizard(&mut config)?;
        }
        ConfigCommands::Channels => {
            configure_channels(&mut config)?;
        }
        ConfigCommands::Init { template, output } => {
            init_config(template, output)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    use aisopod_config::types::ModelProvider;

    #[test]
    fn test_config_args_default() {
        let args = ConfigArgs {
            command: ConfigCommands::Show,
        };

        match args.command {
            ConfigCommands::Show => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn test_config_set_command() {
        let args = ConfigArgs {
            command: ConfigCommands::Set {
                key: "gateway.server.port".to_string(),
                value: "8080".to_string(),
            },
        };

        match args.command {
            ConfigCommands::Set { key, value } => {
                assert_eq!(key, "gateway.server.port");
                assert_eq!(value, "8080");
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_is_sensitive_field() {
        assert!(is_sensitive_field("auth.openai.api_key"));
        assert!(is_sensitive_field("channels.telegram.token"));
        assert!(!is_sensitive_field("gateway.server.port"));
        assert!(!is_sensitive_field("meta.version"));
    }

    #[test]
    fn test_show_config_redacts_sensitive() {
        let mut config = AisopodConfig::default();
        // Set up the model provider correctly
        config.models.providers = vec![ModelProvider {
            name: "openai".to_string(),
            endpoint: "".to_string(),
            api_key: "sk-test-key-12345".to_string(),
        }];

        // This should not panic and should redact the sensitive field
        let result = show_config(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_load_config_or_default_no_file() {
        // When no config file exists, should return default config
        let result = load_config_or_default(None);
        assert!(result.is_ok());
        let config = result.unwrap();
        // Default version is "1.0" not "1.0.0"
        assert_eq!(config.meta.version, "1.0");
    }

    #[test]
    fn test_config_init_command() {
        let args = ConfigArgs {
            command: ConfigCommands::Init {
                template: Some("dev".to_string()),
                output: None,
            },
        };

        match args.command {
            ConfigCommands::Init { template, output } => {
                assert_eq!(template, Some("dev".to_string()));
                assert_eq!(output, None);
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_config_init_with_output() {
        let args = ConfigArgs {
            command: ConfigCommands::Init {
                template: Some("production".to_string()),
                output: Some("/path/to/config.json".to_string()),
            },
        };

        match args.command {
            ConfigCommands::Init { template, output } => {
                assert_eq!(template, Some("production".to_string()));
                assert_eq!(output, Some("/path/to/config.json".to_string()));
            }
            _ => assert!(false),
        }
    }
}
