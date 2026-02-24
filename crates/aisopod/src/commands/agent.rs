//! Agent management command implementation module
//!
//! This module provides the `aisopod agent` subcommand family for managing AI agents:
//! - list: List all configured agents
//! - add: Add a new agent with interactive prompts
//! - delete: Delete an agent by ID
//! - identity: Show agent identity and configuration

use anyhow::{anyhow, Result};
use clap::{Args, Subcommand};
use std::io::{self, Write};
use std::path::Path;

use aisopod_config::load_config;
use aisopod_config::types::Agent;

/// Agent management command arguments
#[derive(Args)]
pub struct AgentArgs {
    #[command(subcommand)]
    pub command: AgentCommands,
}

/// Available agent subcommands
#[derive(Subcommand)]
pub enum AgentCommands {
    /// List all configured agents
    List,
    /// Add a new agent
    Add {
        /// Unique agent identifier
        id: String,
    },
    /// Delete an agent
    Delete {
        /// Agent identifier to remove
        id: String,
    },
    /// Show agent identity and configuration
    Identity {
        /// Agent identifier to inspect
        id: String,
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

/// Load configuration from file or use defaults
fn load_config_or_default(config_path: Option<&str>) -> Result<aisopod_config::AisopodConfig> {
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
                Ok(aisopod_config::AisopodConfig::default())
            }
        }
    }
}

/// Run the agent management command with the given arguments and config path
pub fn run(args: AgentArgs, config_path: Option<String>) -> Result<()> {
    let config_path_ref = config_path.as_deref();
    let mut config = load_config_or_default(config_path_ref)?;

    match args.command {
        AgentCommands::List => {
            let agents = &config.agents.agents;
            if agents.is_empty() {
                println!("No agents configured.");
            } else {
                println!("Configured agents:");
                for agent in agents {
                    println!("  {}: {}", agent.id, agent.name);
                }
            }
        }
        AgentCommands::Add { id } => {
            // Check if agent already exists
            if config.agents.agents.iter().any(|a| a.id == id) {
                return Err(anyhow!("Agent with ID '{}' already exists.", id));
            }

            // Interactive prompts for agent configuration
            let name = prompt("Agent name: ")?;
            let model = prompt("Model (e.g. gpt-4): ")?;
            let system_prompt = prompt("System prompt: ")?;

            // Use defaults for other fields
            let workspace = prompt("Workspace path (default: /workspace): ")?;
            let workspace = if workspace.is_empty() {
                "/workspace".to_string()
            } else {
                workspace
            };

            let sandbox_input = prompt("Enable sandbox (y/n, default: n): ")?;
            let sandbox = sandbox_input.to_lowercase() == "y";

            let mut subagents: Vec<String> = Vec::new();
            loop {
                let subagent = prompt("Subagent ID (empty to finish): ")?;
                if subagent.is_empty() {
                    break;
                }
                subagents.push(subagent);
            }

            let max_subagent_depth_input =
                prompt(format!("Max subagent depth (default: 3): ").as_str())?;
            let max_subagent_depth = if max_subagent_depth_input.is_empty() {
                3usize
            } else {
                max_subagent_depth_input.parse().unwrap_or(3)
            };

            let agent = Agent {
                id,
                name,
                model,
                workspace,
                sandbox,
                subagents,
                system_prompt,
                max_subagent_depth,
                subagent_allowed_models: None,
                skills: Vec::new(),
            };

            config.agents.agents.push(agent.clone());
            save_config(&config, config_path)?;
            println!("Agent '{}' added successfully.", agent.id);
        }
        AgentCommands::Delete { id } => {
            let initial_len = config.agents.agents.len();
            config.agents.agents.retain(|a| a.id != id);

            if config.agents.agents.len() == initial_len {
                return Err(anyhow!("Agent with ID '{}' not found.", id));
            }

            save_config(&config, config_path)?;
            println!("Agent '{}' deleted.", id);
        }
        AgentCommands::Identity { id } => {
            let agent = config
                .agents
                .agents
                .iter()
                .find(|a| a.id == id)
                .ok_or_else(|| anyhow!("Agent with ID '{}' not found.", id))?;

            println!("ID:                 {}", agent.id);
            println!("Name:               {}", agent.name);
            println!("Model:              {}", agent.model);
            println!("Workspace:          {}", agent.workspace);
            println!("Sandbox:            {}", agent.sandbox);
            println!("System prompt:      {}", agent.system_prompt);
            println!(
                "Max subagent depth: {}",
                agent.max_subagent_depth
            );
            if !agent.subagents.is_empty() {
                println!(
                    "Subagents:          {}",
                    agent.subagents.join(", ")
                );
            }
        }
    }

    Ok(())
}

/// Save configuration to file
fn save_config(
    config: &aisopod_config::AisopodConfig,
    config_path: Option<String>,
) -> Result<()> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_list_empty_agents() {
        let args = AgentArgs {
            command: AgentCommands::List,
        };

        let result = run(args, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_add_agent() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("config.json5");

        // Create initial config
        let initial_config = aisopod_config::AisopodConfig::default();
        let content = serde_json::to_string_pretty(&initial_config).unwrap();
        fs::write(&config_path, content).expect("Failed to write test config");

        let args = AgentArgs {
            command: AgentCommands::Add {
                id: "test_agent".to_string(),
            },
        };

        // Note: This test will fail interactively, so we skip it
        // In a real scenario, the interactive prompts would need to be mocked
        let _ = args;
    }

    #[test]
    fn test_delete_nonexistent_agent() {
        let args = AgentArgs {
            command: AgentCommands::Delete {
                id: "nonexistent".to_string(),
            },
        };

        let result = run(args, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_identity_nonexistent_agent() {
        let args = AgentArgs {
            command: AgentCommands::Identity {
                id: "nonexistent".to_string(),
            },
        };

        let result = run(args, None);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Agent with ID 'nonexistent' not found."));
    }
}
