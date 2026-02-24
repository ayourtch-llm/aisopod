//! Model management commands for the aisopod application.
//!
//! This module provides commands for listing available models across all
//! configured providers and switching the primary model for the default agent.

use anyhow::Result;
use clap::{Args, Subcommand};
use serde_json::{json, Value};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use aisopod_config::load_config;
use aisopod_config::AisopodConfig;
use aisopod_provider::discovery::ModelCatalog;
use aisopod_provider::providers;
use aisopod_provider::registry::ProviderRegistry;
use aisopod_provider::trait_module::ModelProvider;
use aisopod_provider::types::ModelInfo;
use std::collections::HashMap;
use crate::output::Output;

/// Model management command arguments
#[derive(Args)]
pub struct ModelsArgs {
    #[command(subcommand)]
    pub command: ModelsCommands,
}

/// Available model management subcommands
#[derive(Subcommand)]
pub enum ModelsCommands {
    /// List available models across all providers
    List {
        /// Filter by provider name
        #[arg(long)]
        provider: Option<String>,
        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },
    /// Switch the primary model for the default agent
    Switch {
        /// Model identifier (e.g., gpt-4, claude-3-opus)
        model: String,
        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },
}

/// Load configuration and create a provider registry
async fn load_config_and_registry(
    config_path: Option<&str>,
) -> Result<(AisopodConfig, Arc<std::sync::RwLock<ProviderRegistry>>)> {
    let config_path = config_path.unwrap_or("aisopod-config.json5");
    let config = load_config(Path::new(config_path))?;

    // Create provider registry
    let registry = ProviderRegistry::new();
    let registry_arc = Arc::new(std::sync::RwLock::new(registry));

    // Load models config and create providers
    for provider_config in &config.models.providers {
        let provider: Arc<dyn ModelProvider> = match provider_config.name.as_str() {
            "openai" => {
                let provider = providers::openai::OpenAIProvider::new(
                    provider_config.api_key.clone(),
                    Some(provider_config.endpoint.clone()),
                    None,
                    None,
                );
                Arc::new(provider)
            }
            "anthropic" => {
                let provider = providers::anthropic::AnthropicProvider::new(
                    provider_config.api_key.clone(),
                    Some(provider_config.endpoint.clone()),
                    None,
                    None,
                );
                Arc::new(provider)
            }
            "gemini" => {
                let provider = providers::gemini::GeminiProvider::new(
                    Some(provider_config.api_key.clone()),
                    None,
                    Some(provider_config.endpoint.clone()),
                    None,
                );
                Arc::new(provider)
            }
            "bedrock" => {
                let provider = providers::bedrock::BedrockProvider::new(
                    None,
                    None,
                    None,
                ).await?;
                Arc::new(provider)
            }
            "ollama" => {
                let provider = providers::ollama::OllamaProvider::new(Some(provider_config.endpoint.clone()));
                Arc::new(provider)
            }
            _ => {
                // Skip unknown providers
                continue;
            }
        };
        let mut reg = registry_arc.write().unwrap();
        reg.register(provider);
    }

    Ok((config, registry_arc))
}

/// List all available models from all configured providers
pub async fn list_models(
    provider_filter: Option<String>,
    config_path: Option<String>,
    json: bool,
) -> Result<()> {
    let (config, registry) = load_config_and_registry(config_path.as_deref()).await?;

    let catalog = ModelCatalog::new(registry, Duration::from_secs(60));
    let models = catalog.list_all().await?;
    let output = Output::new(json);

    if models.is_empty() {
        output.info("No models found.");
        return Ok(());
    }

    // Group models by provider for human-readable output
    let mut models_by_provider: HashMap<String, Vec<ModelInfo>> = HashMap::new();

    for model in &models {
        let provider_name = model.provider.clone();
        models_by_provider
            .entry(provider_name)
            .or_insert_with(Vec::new)
            .push(model.clone());
    }

    if json {
        // JSON output mode - collect all models into a single JSON array
        let models_json: Vec<serde_json::Value> = models
            .iter()
            .map(|model| {
                serde_json::to_value(model).unwrap_or_else(|_| {
                    serde_json::json!({
                        "id": model.id.clone(),
                        "name": model.name.clone(),
                        "provider": model.provider.clone(),
                        "context_window": model.context_window,
                        "supports_vision": model.supports_vision,
                        "supports_tools": model.supports_tools
                    })
                })
            })
            .collect();
        // Extract values for table
        let rows: Vec<Vec<String>> = models_json
            .iter()
            .map(|m| {
                vec![
                    m.get("name").unwrap_or(&json!("")).as_str().unwrap_or("").to_string(),
                    m.get("provider").unwrap_or(&json!("")).as_str().unwrap_or("").to_string(),
                    m.get("context_window").unwrap_or(&json!("")).to_string(),
                ]
            })
            .collect();
        output.print_table(&["Model", "Provider", "Context Window"], rows);
        return Ok(());
    }

    // Print models grouped by provider
    for (provider_name, mut provider_models) in models_by_provider {
        // Apply provider filter if specified
        if let Some(ref filter) = provider_filter {
            if provider_name != *filter {
                continue;
            }
        }

        println!("Provider: {}", provider_name);
        println!("{}", "-".repeat(40));

        // Sort models by name
        provider_models.sort_by(|a, b| a.id.cmp(&b.id));

        // Find the default model
        let default_model = config.agents.default.model.clone();

        for model in provider_models {
            let marker = if model.id == default_model {
                " (default)"
            } else {
                ""
            };
            println!("  {} {}{}", model.id, model.name, marker);
        }
        println!();
    }

    Ok(())
}

/// Switch the primary model for the default agent
pub async fn switch_model(model_id: &str, config_path: Option<String>, json: bool) -> Result<()> {
    let config_path = config_path.as_deref().unwrap_or("aisopod-config.json5");
    let config = load_config(Path::new(config_path))?;
    let output = Output::new(json);
    
    // Check if the model exists in any provider
    let found = {
        let (temp_config, registry) = load_config_and_registry(Some(config_path)).await?;
        let catalog = ModelCatalog::new(registry, Duration::from_secs(60));
        let models = catalog.list_all().await?;
        models.iter().any(|m| m.id == model_id)
    };

    if !found {
        // Try to get the list for better error message
        let (temp_config, registry) = load_config_and_registry(Some(config_path)).await?;
        let catalog = ModelCatalog::new(registry, Duration::from_secs(60));
        let models = catalog.list_all().await?;
        
        let error_message = format!(
            "Model '{}' not found in any configured provider. Available models:\n{}",
            model_id,
            models.iter().map(|m| format!("  {}", m.id)).collect::<Vec<_>>().join("\n")
        );
        
        output.error(&error_message);
        return Err(anyhow::anyhow!("Model not found"));
    }

    // Load the config for modification
    let mut config = load_config(Path::new(config_path))?;

    // Update the default agent's model
    config.agents.default.model = model_id.to_string();

    // Save the configuration
    let content = serde_json::to_string_pretty(&config)?;
    std::fs::write(config_path, content)?;

    output.success(&format!("Switched default agent model to: {}", model_id));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_models_args_default() {
        let args = ModelsArgs {
            command: ModelsCommands::List { provider: None, json: false },
        };

        match args.command {
            ModelsCommands::List { provider, .. } => {
                assert!(provider.is_none());
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_models_list_command() {
        let args = ModelsArgs {
            command: ModelsCommands::List {
                provider: Some("openai".to_string()),
                json: false,
            },
        };

        match args.command {
            ModelsCommands::List { provider, .. } => {
                assert_eq!(provider, Some("openai".to_string()));
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_models_switch_command() {
        let args = ModelsArgs {
            command: ModelsCommands::Switch {
                model: "gpt-4".to_string(),
                json: false,
            },
        };

        match args.command {
            ModelsCommands::Switch { model, .. } => {
                assert_eq!(model, "gpt-4");
            }
            _ => assert!(false),
        }
    }
}
