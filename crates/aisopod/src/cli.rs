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
    Gateway,
    /// Manage agents
    Agent,
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
    let _cli = Cli::parse();

    // Dispatch to subcommand stubs
    match _cli.command {
        Commands::Gateway => todo!("gateway command"),
        Commands::Agent => todo!("agent command"),
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
