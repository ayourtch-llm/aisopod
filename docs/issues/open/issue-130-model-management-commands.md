# Issue 130: Implement Model Management Commands

## Summary
Implement the `aisopod models` subcommands for listing available models across all configured providers and switching the primary model for the default agent.

## Location
- Crate: `aisopod` (main binary crate)
- File: `src/commands/models.rs`

## Current Behavior
The models subcommand is a stub that panics with `todo!`. There is no CLI interface for discovering or switching models.

## Expected Behavior
Users can list all available models from all configured providers (e.g., OpenAI, Anthropic, local) and switch the primary model used by the default agent with a single command.

## Impact
Model management is essential for users who want to experiment with different models or switch providers. This avoids manual config file editing for a common operation.

## Suggested Implementation

1. Define the models subcommand and its nested subcommands:

```rust
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct ModelsArgs {
    #[command(subcommand)]
    pub command: ModelsCommands,
}

#[derive(Subcommand)]
pub enum ModelsCommands {
    /// List available models across all providers
    List {
        /// Filter by provider
        #[arg(long)]
        provider: Option<String>,
    },
    /// Switch the primary model for the default agent
    Switch {
        /// Model identifier (e.g., gpt-4, claude-3-opus)
        model: String,
    },
}
```

2. Implement the list handler:

```rust
pub async fn list_models(provider_filter: Option<String>, config_path: Option<String>) -> anyhow::Result<()> {
    let config = load_config(config_path)?;

    for provider in config.model_providers() {
        if let Some(ref filter) = provider_filter {
            if provider.name() != filter {
                continue;
            }
        }

        println!("Provider: {}", provider.name());
        println!("{}", "-".repeat(40));

        let models = provider.discover_models().await?;
        for model in models {
            let marker = if model.is_default { " (default)" } else { "" };
            println!("  {} {}{}", model.id, model.description, marker);
        }
        println!();
    }

    Ok(())
}
```

3. Implement the switch handler:

```rust
pub fn switch_model(model: &str, config_path: Option<String>) -> anyhow::Result<()> {
    let mut config = load_config(config_path)?;

    // Validate the model exists in a known provider
    let resolved = config.resolve_model(model)?;

    // Update the default agent's model
    let default_agent = config.default_agent_mut()?;
    default_agent.set_model(&resolved.full_id());

    config.save()?;
    println!("Switched default agent model to: {}", resolved.full_id());

    Ok(())
}
```

4. Update the `Commands` enum and dispatch:

```rust
// In cli.rs
Models(ModelsArgs),

// In main.rs
Commands::Models(args) => match args.command {
    ModelsCommands::List { provider } => {
        rt.block_on(commands::models::list_models(provider, cli.config))?;
    }
    ModelsCommands::Switch { model } => {
        commands::models::switch_model(&model, cli.config)?;
    }
},
```

## Dependencies
- Issue 124 (clap CLI framework)
- Issue 047 (model discovery)
- Issue 016 (configuration types)

## Acceptance Criteria
- [ ] `aisopod models list` displays all available models grouped by provider
- [ ] `aisopod models list --provider openai` filters to a specific provider
- [ ] `aisopod models switch gpt-4` updates the default agent's model
- [ ] Currently active model is indicated in the list output
- [ ] Error message when switching to an unknown model
- [ ] JSON output mode (`--json`) returns structured model data

---
*Created: 2026-02-15*
