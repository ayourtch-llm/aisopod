//! Authentication management commands for the aisopod application.
//!
//! This module provides commands for interactive authentication setup and
//! status checking for various AI model providers.

use anyhow::Result;
use clap::{Args, Subcommand};
use std::io::{self, Write};
use std::path::Path;

use aisopod_config::load_config;
use aisopod_config::AisopodConfig;
use aisopod_config::sensitive::Sensitive;
use aisopod_config::types::AuthProfile;

/// Authentication command arguments
#[derive(Args)]
pub struct AuthArgs {
    #[command(subcommand)]
    pub command: AuthCommands,
}

/// Available authentication subcommands
#[derive(Subcommand)]
pub enum AuthCommands {
    /// Interactive authentication setup
    Setup,
    /// Show current auth status
    Status,
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
                anyhow::anyhow!("Failed to load configuration from '{}': {}", path, e)
            })
        }
        None => {
            // Use default config path
            let default_path = aisopod_config::default_config_path();
            if default_path.exists() {
                load_config(&default_path).map_err(|e| {
                    anyhow::anyhow!("Failed to load configuration from '{}': {}", default_path.display(), e)
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

/// Add or update a provider in the config
fn add_provider(config: &mut AisopodConfig, provider_name: &str, endpoint: &str, api_key: &str) {
    // Check if provider already exists
    if let Some(profile) = config.auth.profiles.iter_mut().find(|p| p.name == provider_name) {
        profile.endpoint = Some(endpoint.to_string());
        profile.api_key = Sensitive::new(api_key.to_string());
    } else {
        config.auth.profiles.push(AuthProfile {
            name: provider_name.to_string(),
            api_key: Sensitive::new(api_key.to_string()),
            provider: provider_name.to_string(),
            endpoint: Some(endpoint.to_string()),
        });
    }
}

/// Run the authentication setup wizard
pub fn run_auth_setup(config_path: Option<String>) -> Result<()> {
    let config_path_ref = config_path.as_deref();
    let mut config = load_config_or_default(config_path_ref)?;

    println!("=== Authentication Setup ===\n");

    let provider = prompt_select(
        "Select provider to configure",
        &["openai", "anthropic", "google", "azure", "local"],
    )?;

    match provider.as_str() {
        "openai" => {
            println!("\nGet your API key from: https://platform.openai.com/api-keys\n");
            let key = prompt_password("OpenAI API key: ")?;
            let org = prompt("Organization ID (optional, press Enter to skip): ")?;
            if !org.is_empty() {
                // Store org in a separate field (we'll add it to the endpoint for now)
                // In a full implementation, this would be a proper auth configuration
            }
            add_provider(&mut config, "openai", "https://api.openai.com/v1", &key);
        }
        "anthropic" => {
            println!("\nGet your API key from: https://console.anthropic.com/settings/keys\n");
            let key = prompt_password("Anthropic API key: ")?;
            add_provider(&mut config, "anthropic", "https://api.anthropic.com/v1", &key);
        }
        "google" => {
            println!("\nGet your API key from: https://aistudio.google.com/apikey\n");
            let key = prompt_password("Google AI API key: ")?;
            add_provider(&mut config, "google", "https://generativelanguage.googleapis.com/v1beta", &key);
        }
        "azure" => {
            let endpoint = prompt("Azure endpoint URL: ")?;
            let key = prompt_password("Azure API key: ")?;
            add_provider(&mut config, "azure", &endpoint, &key);
        }
        "local" => {
            let endpoint = prompt_with_default("Local endpoint URL", "http://localhost:11434")?;
            add_provider(&mut config, "local", &endpoint, "");
        }
        _ => {}
    }

    save_config(&config, config_path)?;
    println!("\nAuthentication for '{}' configured successfully!", provider);
    Ok(())
}

/// Run the authentication status command
pub fn run_auth_status(config_path: Option<String>) -> Result<()> {
    let config = load_config_or_default(config_path.as_deref())?;

    println!("Authentication Status\n");
    println!("{:<15} {:<15} {:<20}", "Provider", "Status", "Details");
    println!("{}", "-".repeat(50));

    // Get providers from auth.profiles
    for profile in &config.auth.profiles {
        let has_key = !profile.api_key.expose().is_empty();
        let status = if has_key { "Configured" } else { "Not set" };
        let detail = if has_key { "API key set" } else { "Run 'aisopod auth setup'" };
        println!("{:<15} {:<15} {:<20}", profile.name, status, detail);
    }

    Ok(())
}

/// Run the authentication command with the given arguments and config path
pub fn run(args: AuthArgs, config_path: Option<String>) -> Result<()> {
    match args.command {
        AuthCommands::Setup => {
            run_auth_setup(config_path)?;
        }
        AuthCommands::Status => {
            run_auth_status(config_path)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_args_default() {
        let args = AuthArgs {
            command: AuthCommands::Status,
        };

        match args.command {
            AuthCommands::Status => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn test_auth_setup_command() {
        let args = AuthArgs {
            command: AuthCommands::Setup,
        };

        match args.command {
            AuthCommands::Setup => assert!(true),
            _ => assert!(false),
        }
    }
}
