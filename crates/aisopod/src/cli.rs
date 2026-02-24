//! CLI arguments for the aisopod application.
//!
//! This module provides the command-line interface for interacting with aisopod.
//! It uses clap for argument parsing and provides subcommands for various operations.

use clap::{Parser, Subcommand};

/// Top-level CLI arguments.
#[derive(Parser)]
#[command(name = "aisopod")]
#[command(version, about = "AI agent orchestration platform", long_about = None)]
pub struct Cli {
    /// Path to configuration file
    #[arg(long, global = true)]
    pub config: Option<String>,

    /// Enable verbose output
    #[arg(long, global = true)]
    pub verbose: bool,

    /// Output in JSON format
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Commands,
}

/// Available subcommands.
#[derive(Subcommand)]
pub enum Commands {
    /// Start the HTTP+WS gateway server
    Gateway(crate::commands::gateway::GatewayArgs),
    /// Manage agents
    Agent(crate::commands::agent::AgentArgs),
    /// Send a message
    Message,
    /// Manage configuration
    Config,
    /// Show system status
    Status,
    /// Run health check
    Health,
    /// Manage models
    Models,
    /// Manage channels
    Channels,
    /// Manage sessions
    Sessions,
    /// Manage background daemon
    Daemon,
    /// Run system diagnostics
    Doctor,
    /// Manage authentication
    Auth,
    /// Reset all sessions
    Reset,
    /// Generate shell completions
    Completions,
}

/// Main entry point for CLI processing.
pub fn run_cli() {
    let cli = Cli::parse();

    // Dispatch to subcommand handlers
    match cli.command {
        Commands::Gateway(args) => {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
            rt.block_on(crate::commands::gateway::run(args, cli.config)).expect("Gateway command failed");
        }
        Commands::Agent(args) => {
            crate::commands::agent::run(args, cli.config).expect("Agent command failed");
        }
        Commands::Message => todo!("message command"),
        Commands::Config => todo!("config command"),
        Commands::Status => todo!("status command"),
        Commands::Health => todo!("health command"),
        Commands::Models => todo!("models command"),
        Commands::Channels => todo!("channels command"),
        Commands::Sessions => todo!("sessions command"),
        Commands::Daemon => todo!("daemon command"),
        Commands::Doctor => todo!("doctor command"),
        Commands::Auth => todo!("auth command"),
        Commands::Reset => todo!("reset command"),
        Commands::Completions => todo!("completions command"),
    }
}
