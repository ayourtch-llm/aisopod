//! Interactive onboarding wizard for first-time users.
//!
//! This module provides a guided setup experience for new users,
//! walking them through authentication, model selection, channel setup, and first message.

use anyhow::Result;
use colored::Colorize;
use std::io::{self, Write};
use std::path::Path;

use crate::commands::auth::run_auth_setup;
use crate::commands::channels::{setup_channel, ChannelType};
use aisopod_config::load_config;
use aisopod_config::AisopodConfig;

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
        println!(
            "Invalid choice. Please enter a number between 1 and {}.",
            options.len()
        );
    }
}

/// Load configuration from file or use defaults
fn load_config_or_default(config_path: Option<&str>) -> Result<AisopodConfig> {
    match config_path {
        Some(path) => {
            let config_path = Path::new(path);
            load_config(config_path)
                .map_err(|e| anyhow::anyhow!("Failed to load configuration from '{}': {}", path, e))
        }
        None => {
            // Use default config path
            let default_path = aisopod_config::default_config_path();
            if default_path.exists() {
                load_config(&default_path).map_err(|e| {
                    anyhow::anyhow!(
                        "Failed to load configuration from '{}': {}",
                        default_path.display(),
                        e
                    )
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

/// Run the onboarding wizard
pub fn run_onboarding(config_path: Option<String>) -> Result<()> {
    println!("{}", "Welcome to aisopod!".bold());
    println!("Let's get you set up.\n");

    // Step 1: Auth setup
    println!("Step 1: Configure your AI provider\n");
    run_auth_setup(config_path.clone())?;

    // Step 2: Model selection
    println!("\nStep 2: Choose your default model\n");
    let model = prompt_with_default("Default model", "gpt-4")?;
    let mut config = load_config_or_default(config_path.as_deref())?;
    config.agents.default.model = model.clone();

    // Step 3: Channel setup (optional)
    println!("\nStep 3: Set up a messaging channel (optional)\n");
    let setup_choice = prompt("Would you like to set up a channel? (yes/no): ")?;
    if setup_choice == "yes" {
        let channel_type = prompt_select(
            "Which channel would you like to set up?",
            &["telegram", "discord", "whatsapp", "slack"],
        )?;
        let channel_type = match channel_type.as_str() {
            "telegram" => ChannelType::Telegram,
            "discord" => ChannelType::Discord,
            "whatsapp" => ChannelType::Whatsapp,
            "slack" => ChannelType::Slack,
            _ => unreachable!(),
        };
        setup_channel(&channel_type, config_path.clone())?;
    }

    // Step 4: Save configuration and send first message
    println!("\nStep 4: Configuration complete!\n");
    save_config(&config, config_path.clone())?;

    let first_msg = prompt("Type your first message (or press Enter to skip): ")?;
    if !first_msg.is_empty() {
        println!("\nTo send this message, start the gateway and run:");
        println!("  aisopod gateway &");
        println!("  aisopod message \"{}\"", first_msg);
    }

    println!("\n{}", "Setup complete! 🎉".green().bold());
    println!("\nNext steps:");
    println!("  aisopod gateway     - Start the gateway server");
    println!("  aisopod message     - Send a message");
    println!("  aisopod status      - Check system status");
    println!("  aisopod --help      - See all commands");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires stdin interaction - would hang in CI"]
    fn test_prompt_with_default_empty() {
        // This test would need stdin mocking, so we just verify the function compiles
        // In a real test, we would mock stdin
        let _result = prompt_with_default("Test: ", "default");
    }

    #[test]
    #[ignore = "requires stdin interaction - would hang in CI"]
    fn test_prompt_select_valid() {
        // This test would need stdin mocking, so we just verify the function compiles
        let _result = prompt_select("Test", &["option1", "option2"]);
    }
}
