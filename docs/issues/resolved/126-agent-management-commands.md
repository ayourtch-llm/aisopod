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
- [x] `aisopod agent list` displays all configured agents
- [x] `aisopod agent add <id>` prompts interactively and saves a new agent to config
- [x] `aisopod agent delete <id>` removes an agent from the configuration
- [x] `aisopod agent identity <id>` displays the agent's full configuration
- [x] Proper error messages for missing agents or duplicate IDs
- [x] Changes persist to the configuration file

## Resolution

### Implementation Summary

The agent management commands were implemented in `crates/aisopod/src/commands/agent.rs` with the following features:

1. **CLI Interface**: Implemented `AgentArgs` and `AgentCommands` using clap for parsing
2. **Agent Commands**:
   - `list`: Lists all configured agents with ID and name
   - `add <id>`: Interactive prompts for name, model, workspace, sandbox, subagents, and system prompt
   - `delete <id>`: Removes agent by ID with proper error handling
   - `identity <id>`: Displays full agent configuration

3. **Config Persistence**: 
   - Added `default_config_path()` function in `aisopod-config/src/loader.rs`
   - Added `DEFAULT_CONFIG_FILE` constant in `aisopod-config/src/lib.rs`
   - `load_config_or_default()` and `save_config()` use consistent paths
   - Config files are written as pretty-printed JSON

4. **Error Handling**:
   - Duplicate ID detection: "Agent with ID '...' already exists."
   - Missing agent: "Agent with ID '...' not found."

### Files Modified

- `crates/aisopod-config/src/lib.rs`: Exported `default_config_path()` and `DEFAULT_CONFIG_FILE`
- `crates/aisopod-config/src/loader.rs`: Implemented `default_config_path()` function
- `crates/aisopod/src/commands/agent.rs`: Full implementation of agent commands
- `crates/aisopod/src/cli.rs`: Added `Agent` variant to `Commands` enum
- `crates/aisopod/src/main.rs`: Dispatched to agent command handler

### Test Results

```
cargo test -p aisopod-config
  running 55 tests ... ok
  running 23 env tests ... ok
  running 8 load_json5 tests ... ok
  running 7 load_toml tests ... ok
  running 4 doc tests ... ok

cargo test -p aisopod
  running 4 agent command tests ... ok

cargo build -p aisopod
  Build successful with no warnings
```

### Verification Checklist

| Item | Status |
|------|--------|
| All acceptance criteria met | ✅ |
| Tests pass for aisopod-config | ✅ (97 tests) |
| Tests pass for aisopod | ✅ (4 tests) |
| Build passes without errors | ✅ |
| Config file consistency verified | ✅ |
| Error handling verified | ✅ |
| Learning documentation created | ✅ |

---
*Created: 2026-02-15*
*Resolved: 2026-02-24*
