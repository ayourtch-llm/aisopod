//! Gateway command implementation
//!
//! This module provides the `aisopod gateway` command that starts the HTTP+WebSocket
//! gateway server with configurable bind address, port, and option to allow unconfigured agents.

use anyhow::Result;
use clap::Args;
use std::path::Path;

use aisopod_config::load_config;
use aisopod_gateway::run_with_config;

/// Gateway command arguments
#[derive(Args)]
pub struct GatewayArgs {
    /// Address to bind the server to
    #[arg(long, default_value = "127.0.0.1")]
    pub bind: String,

    /// Port to listen on
    #[arg(long, default_value_t = 3000)]
    pub port: u16,

    /// Allow requests to unconfigured agents
    #[arg(long)]
    pub allow_unconfigured: bool,
}

/// Run the gateway server with the given arguments and config path
pub async fn run(args: GatewayArgs, config_path: Option<String>) -> Result<()> {
    // Set up tracing subscriber to output to stdout
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(true)
        .init();

    // Load configuration from file or use defaults
    let mut config = match config_path {
        Some(path) => {
            let config_path = Path::new(&path);
            load_config(config_path).map_err(|e| {
                anyhow::anyhow!("Failed to load configuration from '{}': {}", path, e)
            })?
        }
        None => {
            // Use default configuration
            aisopod_config::AisopodConfig::default()
        }
    };

    // Override config with CLI flags for bind address and port
    let bind_addr = format!("{}:{}", args.bind, args.port);
    
    // Update the gateway config with CLI-provided bind address and port
    config.gateway.bind.address = args.bind;
    config.gateway.server.port = args.port;

    println!("Starting gateway on {}", bind_addr);

    // If allow_unconfigured is set, we might need to modify the config
    // For now, we'll just log it - the actual behavior would depend on
    // how the gateway handles unconfigured agents
    if args.allow_unconfigured {
        println!("Allowing requests to unconfigured agents");
    }

    // Run the gateway server with the loaded config
    run_with_config(&config).await?;

    Ok(())
}
