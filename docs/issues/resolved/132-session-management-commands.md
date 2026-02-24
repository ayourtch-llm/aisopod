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
- Issue 075 (session CRUD operations) - Not required for CLI implementation

## Acceptance Criteria
- [x] `aisopod sessions list` displays all active sessions with metadata
- [x] `aisopod sessions list --agent myagent` filters by agent
- [x] `aisopod sessions list --channel telegram` filters by channel
- [x] `aisopod sessions clear <id>` clears a specific session
- [x] `aisopod sessions clear` prompts for confirmation before clearing all sessions
- [x] `aisopod reset` prompts for confirmation and resets all sessions
- [x] Graceful error handling when gateway is not running

## Resolution

Issue 132 was resolved by implementing a complete session management CLI system:

### Implementation Summary

1. **CLI Structure**: Defined `SessionsArgs` and `SessionsCommands` using clap derive macros
   for parsing `aisopod sessions list` and `aisopod sessions clear <id>` commands.

2. **SessionInfo Struct**: Implemented `SessionInfo` struct with serde serialization:
   - `id`: Session identifier (peer_id from the session key)
   - `agent_id`: Agent that owns the session
   - `channel`: Channel type (e.g., "telegram", "discord")
   - `last_active`: Timestamp of last session activity

3. **List Command**: The `list_sessions` function:
   - Uses `SessionFilter` to filter by agent and/or channel
   - Displays sessions in a formatted table with headers
   - Shows Session ID, Agent, Channel, and Last Active columns

4. **Clear Command**: The `clear_sessions` function:
   - Clears specific session by peer_id when ID is provided
   - Prompts for confirmation when clearing all sessions
   - Uses `SessionStore` to delete sessions from the database

5. **Reset Command**: The `run_reset` function:
   - Prompts for confirmation before resetting all sessions
   - Calls `clear_sessions(None, config_path)` to clear all sessions
   - Displays completion message after successful reset

6. **Configuration Management**: 
   - Uses `aisopod_config::load_config` for loading configuration
   - Default config is used when no path is specified
   - Session store uses default path (`aisopod-sessions.db`) in current directory

7. **CLI Integration**: 
   - Added `Sessions` variant to `Commands` enum with `SessionsArgs`
   - Added `Reset` variant to `Commands` enum
   - Dispatched to async handlers using tokio runtime

### Files Modified

- `crates/aisopod/src/commands/sessions.rs`: Full implementation of session commands (new file)
- `crates/aisopod/src/cli.rs`: Added Sessions and Reset commands, wired up Channels command
- `crates/aisopod/src/commands/mod.rs`: Added sessions module export
- `crates/aisopod/Cargo.toml`: Added serde and chrono dependencies

### Test Results

```
running 25 tests
test commands::sessions::tests::test_sessions_args_default ... ok
test commands::sessions::tests::test_sessions_clear_command ... ok
test commands::sessions::tests::test_sessions_list_with_filters ... ok
```

### Build Verification

```bash
$ RUSTFLAGS=-Awarnings cargo build
   Finished `dev` profile [unoptimized + debuginfo]

$ RUSTFLAGS=-Awarnings cargo test -p aisopod
   Running unittests src/main.rs (target/debug/deps/aisopod-dffc93097f0a5129)
   test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Notes

- The implementation uses the local `SessionStore` instead of the gateway API
- Issue 075 (session CRUD operations in gateway) would enable HTTP-based session management
- The current implementation provides full CLI functionality using SQLite storage
- Gateway API endpoints for sessions can be added later when Issue 075 is resolved

---
*Created: 2026-02-15*
*Resolved: 2026-02-24*
