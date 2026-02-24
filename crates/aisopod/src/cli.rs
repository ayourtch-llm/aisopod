//! CLI arguments for the aisopod application.
//!
//! This module provides the command-line interface for interacting with aisopod.
//! It uses clap for argument parsing and provides subcommands for various operations.

use clap::{Parser, Subcommand};

use crate::commands;

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
    Message(crate::commands::message::MessageArgs),
    /// Manage configuration
    Config(crate::commands::config::ConfigArgs),
    /// Show system status
    Status(crate::commands::status::StatusArgs),
    /// Run health check
    Health(crate::commands::status::HealthArgs),
    /// Display live dashboard
    Dashboard,
    /// Manage models
    Models(crate::commands::models::ModelsArgs),
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
        Commands::Message(args) => {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
            rt.block_on(crate::commands::message::run(args, cli.config))
                .expect("Message command failed");
        }
        Commands::Config(args) => {
            crate::commands::config::run(args, cli.config).expect("Config command failed");
        }
        Commands::Status(args) => {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
            rt.block_on(crate::commands::status::run_status(args, cli.config, cli.json)).expect("Status command failed");
        }
        Commands::Health(args) => {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
            rt.block_on(crate::commands::status::run_health(cli.config, args.json)).expect("Health command failed");
        }
        Commands::Dashboard => {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
            rt.block_on(crate::commands::status::run_dashboard(cli.config)).expect("Dashboard command failed");
        }
        Commands::Models(args) => {
            match args.command {
                crate::commands::models::ModelsCommands::List { provider } => {
                    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
                    rt.block_on(crate::commands::models::list_models(provider, cli.config)).expect("Models list command failed");
                }
                crate::commands::models::ModelsCommands::Switch { model } => {
                    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
                    rt.block_on(crate::commands::models::switch_model(&model, cli.config)).expect("Models switch command failed");
                }
            }
        }
        Commands::Channels => todo!("channels command"),
        Commands::Sessions => todo!("sessions command"),
        Commands::Daemon => todo!("daemon command"),
        Commands::Doctor => todo!("doctor command"),
        Commands::Auth => todo!("auth command"),
        Commands::Reset => todo!("reset command"),
        Commands::Completions => todo!("completions command"),
    }
}
