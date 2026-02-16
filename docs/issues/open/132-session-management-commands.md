# Issue 132: Implement Session Management Commands

## Summary
Implement the `aisopod sessions` subcommands for listing active sessions, clearing session history, and a top-level `aisopod reset` command for resetting all sessions.

## Location
- Crate: `aisopod` (main binary crate)
- File: `src/commands/sessions.rs`

## Current Behavior
The sessions and reset subcommands are stubs that panic with `todo!`. There is no CLI interface for managing conversation sessions.

## Expected Behavior
Users can list all active sessions with metadata (agent, channel, last activity), clear history for specific or all sessions, and quickly reset all sessions with a confirmation prompt.

## Impact
Session management is important for privacy, debugging, and resource management. Users need the ability to view and control their conversation state.

## Suggested Implementation

1. Define the sessions subcommand and its nested subcommands:

```rust
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct SessionsArgs {
    #[command(subcommand)]
    pub command: SessionsCommands,
}

#[derive(Subcommand)]
pub enum SessionsCommands {
    /// List active sessions
    List {
        /// Filter by agent ID
        #[arg(long)]
        agent: Option<String>,

        /// Filter by channel
        #[arg(long)]
        channel: Option<String>,
    },
    /// Clear session history
    Clear {
        /// Specific session ID to clear (clears all if omitted)
        session_id: Option<String>,
    },
}
```

2. Implement the list handler:

```rust
pub async fn list_sessions(
    agent: Option<String>,
    channel: Option<String>,
    config_path: Option<String>,
) -> anyhow::Result<()> {
    let config = load_config(config_path)?;
    let gateway_url = config.gateway_http_url();

    let client = reqwest::Client::new();
    let mut url = format!("{}/sessions", gateway_url);

    // Add query filters
    let mut params = vec![];
    if let Some(a) = &agent { params.push(format!("agent={}", a)); }
    if let Some(c) = &channel { params.push(format!("channel={}", c)); }
    if !params.is_empty() {
        url = format!("{}?{}", url, params.join("&"));
    }

    let sessions: Vec<SessionInfo> = client.get(&url).send().await?.json().await?;

    println!("{:<36} {:<15} {:<12} {:<20}", "Session ID", "Agent", "Channel", "Last Active");
    println!("{}", "-".repeat(83));

    for session in sessions {
        println!(
            "{:<36} {:<15} {:<12} {:<20}",
            session.id, session.agent_id, session.channel, session.last_active
        );
    }

    Ok(())
}
```

3. Implement the clear handler:

```rust
pub async fn clear_sessions(
    session_id: Option<String>,
    config_path: Option<String>,
) -> anyhow::Result<()> {
    let config = load_config(config_path)?;
    let gateway_url = config.gateway_http_url();
    let client = reqwest::Client::new();

    match session_id {
        Some(id) => {
            client.delete(format!("{}/sessions/{}", gateway_url, id)).send().await?;
            println!("Session '{}' cleared.", id);
        }
        None => {
            let confirm = prompt("Clear ALL sessions? (yes/no): ")?;
            if confirm == "yes" {
                client.delete(format!("{}/sessions", gateway_url)).send().await?;
                println!("All sessions cleared.");
            } else {
                println!("Cancelled.");
            }
        }
    }

    Ok(())
}
```

4. Implement the top-level reset command:

```rust
pub async fn run_reset(config_path: Option<String>) -> anyhow::Result<()> {
    let confirm = prompt("This will reset ALL sessions and conversation history. Continue? (yes/no): ")?;
    if confirm != "yes" {
        println!("Cancelled.");
        return Ok(());
    }

    clear_sessions(None, config_path).await?;
    println!("All sessions have been reset.");
    Ok(())
}
```

## Dependencies
- Issue 124 (clap CLI framework)
- Issue 075 (session CRUD operations)

## Acceptance Criteria
- [ ] `aisopod sessions list` displays all active sessions with metadata
- [ ] `aisopod sessions list --agent myagent` filters by agent
- [ ] `aisopod sessions list --channel telegram` filters by channel
- [ ] `aisopod sessions clear <id>` clears a specific session
- [ ] `aisopod sessions clear` prompts for confirmation before clearing all sessions
- [ ] `aisopod reset` prompts for confirmation and resets all sessions
- [ ] Graceful error handling when gateway is not running

---
*Created: 2026-02-15*
