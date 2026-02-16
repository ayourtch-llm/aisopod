# Issue 126: Implement Agent Management Commands

## Summary
Implement the `aisopod agent` subcommands for listing, adding, deleting, and inspecting agents through the CLI.

## Location
- Crate: `aisopod` (main binary crate)
- File: `src/commands/agent.rs`

## Current Behavior
The agent subcommand is a stub that panics with `todo!`. There is no CLI interface for managing agents.

## Expected Behavior
Users can manage agents entirely from the command line: list all configured agents, add new agents with interactive prompts, delete agents by ID, and view an agent's identity and configuration details.

## Impact
Agent management is a core user workflow. This enables users to configure and manage their AI agents without manually editing configuration files.

## Suggested Implementation

1. Define the agent subcommand and its nested subcommands:

```rust
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct AgentArgs {
    #[command(subcommand)]
    pub command: AgentCommands,
}

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
```

2. Implement each handler:

```rust
pub fn run(args: AgentArgs, config_path: Option<String>) -> anyhow::Result<()> {
    let config = load_config(config_path)?;

    match args.command {
        AgentCommands::List => {
            let agents = config.agents();
            for agent in agents {
                println!("{}: {}", agent.id, agent.description);
            }
        }
        AgentCommands::Add { id } => {
            // Interactive prompts for agent configuration
            let name = prompt("Agent name: ")?;
            let model = prompt("Model (e.g. gpt-4): ")?;
            let system_prompt = prompt("System prompt: ")?;

            let agent = AgentConfig::new(&id, &name, &model, &system_prompt);
            config.add_agent(agent)?;
            config.save()?;
            println!("Agent '{}' added successfully.", id);
        }
        AgentCommands::Delete { id } => {
            config.remove_agent(&id)?;
            config.save()?;
            println!("Agent '{}' deleted.", id);
        }
        AgentCommands::Identity { id } => {
            let agent = config.get_agent(&id)?;
            println!("ID:     {}", agent.id);
            println!("Name:   {}", agent.name);
            println!("Model:  {}", agent.model);
            println!("Prompt: {}", agent.system_prompt);
        }
    }

    Ok(())
}
```

3. Update `Commands` enum in `src/cli.rs`:

```rust
Agent(AgentArgs),
```

4. Update dispatch in `main.rs`:

```rust
Commands::Agent(args) => commands::agent::run(args, cli.config)?,
```

## Dependencies
- Issue 124 (clap CLI framework)
- Issue 016 (configuration types)
- Issue 063 (agent resolution)

## Acceptance Criteria
- [ ] `aisopod agent list` displays all configured agents
- [ ] `aisopod agent add <id>` prompts interactively and saves a new agent to config
- [ ] `aisopod agent delete <id>` removes an agent from the configuration
- [ ] `aisopod agent identity <id>` displays the agent's full configuration
- [ ] Proper error messages for missing agents or duplicate IDs
- [ ] Changes persist to the configuration file

---
*Created: 2026-02-15*
