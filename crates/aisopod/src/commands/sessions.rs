//! Session management commands for the aisopod application.
//!
//! This module provides commands for managing conversation sessions:
//! - `list`: List active sessions with metadata (agent, channel, last activity)
//! - `clear`: Clear session history for specific or all sessions
//!
//! Also provides a top-level `reset` command for resetting all sessions.

use anyhow::{anyhow, Context, Result};
use clap::{Args, Subcommand};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::path::Path;

use aisopod_config::load_config;
use aisopod_config::types::AisopodConfig;
use aisopod_session::{SessionFilter, SessionStore};
use crate::output::Output;

/// Information about a session.
///
/// Contains metadata about a session including its identifier,
/// associated agent and channel, and last activity timestamp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    /// The unique identifier of the session.
    pub id: String,
    /// The identifier of the agent that owns this session.
    pub agent_id: String,
    /// The channel type (e.g., "discord", "slack", "telegram").
    pub channel: String,
    /// The timestamp when the session was last active.
    pub last_active: String,
}

impl SessionInfo {
    /// Creates a new SessionInfo from a session summary.
    pub fn from_summary(session: &aisopod_session::SessionSummary) -> Self {
        Self {
            id: session.key.peer_id.clone(),
            agent_id: session.key.agent_id.clone(),
            channel: session.key.channel.clone(),
            last_active: session.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}

/// Session management command arguments
#[derive(Args)]
pub struct SessionsArgs {
    #[command(subcommand)]
    pub command: SessionsCommands,
}

/// Available session management subcommands
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
        #[arg(long)]
        id: Option<String>,
    },
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
            // Use default configuration
            Ok(aisopod_config::AisopodConfig::default())
        }
    }
}

/// Prompt the user for input
fn prompt(prompt_text: &str) -> Result<String> {
    print!("{}", prompt_text);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_string())
}

/// Build session store path from config
fn build_session_store_path(config: &AisopodConfig) -> String {
    // Use the default path from config or create a default
    // For now, use the current directory with a default filename
    let db_path = std::env::current_dir()
        .unwrap_or_default()
        .join("aisopod-sessions.db");
    db_path.to_string_lossy().to_string()
}

/// List all active sessions with optional filters
pub async fn list_sessions(
    agent: Option<String>,
    channel: Option<String>,
    config_path: Option<String>,
) -> Result<()> {
    let config = load_config_or_default(config_path.as_deref())?;

    // Build the session filter
    let mut filter = SessionFilter::new();
    if let Some(agent_id) = agent {
        filter = SessionFilter::for_agent(agent_id);
    }
    if let Some(channel_id) = channel {
        filter = SessionFilter::for_channel(channel_id);
    }

    // Build session store path
    let store_path = build_session_store_path(&config);

    // Open or create the session store
    let store = SessionStore::new(Path::new(&store_path))?;

    // List sessions
    let summaries = store.list("admin", &filter)?;

    if summaries.is_empty() {
        let output = Output::new(false);
        output.info("No sessions found.");
        return Ok(());
    }

    // Convert to SessionInfo
    let sessions: Vec<SessionInfo> = summaries.iter().map(SessionInfo::from_summary).collect();

    // Display using table format
    let output = Output::new(false);
    let headers = ["Session ID", "Agent", "Channel", "Last Active"];
    let rows: Vec<Vec<String>> = sessions
        .iter()
        .map(|s| vec![s.id.clone(), s.agent_id.clone(), s.channel.clone(), s.last_active.clone()])
        .collect();
    output.print_table(&headers, rows);

    Ok(())
}

/// Clear session history for specific or all sessions
pub async fn clear_sessions(
    session_id: Option<String>,
    config_path: Option<String>,
) -> Result<()> {
    let config = load_config_or_default(config_path.as_deref())?;

    let store_path = build_session_store_path(&config);
    let store = SessionStore::new(Path::new(&store_path))?;
    let output = Output::new(false);

    match session_id {
        Some(id) => {
            // Clear specific session by peer_id
            // First, find sessions matching the peer_id
            let mut filter = SessionFilter::new();
            filter.peer_id = Some(id.clone());
            
            let sessions = store.list("admin", &filter)?;
            
            if sessions.is_empty() {
                output.error(&format!("Session '{}' not found.", id));
                return Ok(());
            }

            let mut count = 0;
            for session in sessions {
                store.delete("admin", &session.key)?;
                count += 1;
            }
            
            output.success(&format!("Cleared {} session(s) with ID '{}'.", count, id));
        }
        None => {
            // Clear all sessions - prompt for confirmation
            let confirm = prompt("Clear ALL sessions? (yes/no): ")?;
            if confirm == "yes" {
                let filter = SessionFilter::new();
                let sessions = store.list("admin", &filter)?;

                for session in sessions {
                    store.delete("admin", &session.key)?;
                }

                output.success("All sessions cleared.");
            } else {
                output.info("Cancelled.");
            }
        }
    }

    Ok(())
}

/// Run the session management command
pub async fn run(args: SessionsArgs, config_path: Option<String>) -> Result<()> {
    match args.command {
        SessionsCommands::List { agent, channel } => {
            list_sessions(agent, channel, config_path).await?;
        }
        SessionsCommands::Clear { id } => {
            clear_sessions(id, config_path).await?;
        }
    }
    Ok(())
}

/// Reset all sessions (top-level command)
pub async fn run_reset(config_path: Option<String>) -> Result<()> {
    let confirm = prompt("This will reset ALL sessions and conversation history. Continue? (yes/no): ")?;
    let output = Output::new(false);
    
    if confirm != "yes" {
        output.info("Cancelled.");
        return Ok(());
    }

    clear_sessions(None, config_path).await?;
    output.success("All sessions have been reset.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sessions_args_default() {
        let args = SessionsArgs {
            command: SessionsCommands::List {
                agent: None,
                channel: None,
            },
        };

        match args.command {
            SessionsCommands::List { agent, channel } => {
                assert!(agent.is_none());
                assert!(channel.is_none());
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_sessions_list_with_filters() {
        let args = SessionsArgs {
            command: SessionsCommands::List {
                agent: Some("test-agent".to_string()),
                channel: Some("telegram".to_string()),
            },
        };

        match args.command {
            SessionsCommands::List { agent, channel } => {
                assert_eq!(agent.unwrap(), "test-agent");
                assert_eq!(channel.unwrap(), "telegram");
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_sessions_clear_command() {
        let args = SessionsArgs {
            command: SessionsCommands::Clear {
                id: Some("session-123".to_string()),
            },
        };

        match args.command {
            SessionsCommands::Clear { id } => {
                assert_eq!(id.unwrap(), "session-123");
            }
            _ => assert!(false),
        }
    }
}
