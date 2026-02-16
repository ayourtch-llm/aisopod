# Issue 125: Implement Gateway Start Command

## Summary
Implement the `aisopod gateway` command that starts the HTTP+WebSocket gateway server with configurable bind address, port, and option to allow unconfigured agents.

## Location
- Crate: `aisopod` (main binary crate)
- File: `src/commands/gateway.rs`

## Current Behavior
The gateway subcommand is a stub that panics with `todo!`. There is no way to start the gateway server from the CLI.

## Expected Behavior
Running `aisopod gateway` loads the configuration, initializes the gateway server, and starts listening for HTTP and WebSocket connections. Optional flags allow overriding the bind address, port, and enabling unconfigured agent access.

## Impact
This is the primary entry point for running the aisopod server. Without this command, users cannot start the gateway from the command line.

## Suggested Implementation

1. Define the gateway subcommand arguments in `src/commands/gateway.rs`:

```rust
use clap::Args;

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
```

2. Implement the command handler:

```rust
pub async fn run(args: GatewayArgs, config_path: Option<String>) -> anyhow::Result<()> {
    // Load configuration from file or defaults
    let config = load_config(config_path)?;

    // Override config with CLI flags
    let bind_addr = format!("{}:{}", args.bind, args.port);

    // Initialize the gateway server
    let gateway = Gateway::new(config, args.allow_unconfigured)?;

    println!("Starting gateway on {}", bind_addr);

    // Start the HTTP+WS server
    gateway.serve(&bind_addr).await?;

    Ok(())
}
```

3. Update the `Commands` enum in `src/cli.rs` to include `GatewayArgs`:

```rust
#[derive(Subcommand)]
pub enum Commands {
    /// Start the HTTP+WS gateway server
    Gateway(GatewayArgs),
    // ... other commands
}
```

4. Update the dispatch in `main.rs`:

```rust
Commands::Gateway(args) => {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(commands::gateway::run(args, cli.config))?;
}
```

## Dependencies
- Issue 124 (clap CLI framework)
- Issue 026 (Axum server setup)
- Issue 016 (configuration types)

## Acceptance Criteria
- [ ] `aisopod gateway` starts the HTTP+WS gateway server using default settings
- [ ] `aisopod gateway --bind 0.0.0.0 --port 8080` overrides bind address and port
- [ ] `aisopod gateway --allow-unconfigured` enables unconfigured agent access
- [ ] Server logs startup information to stdout
- [ ] Graceful shutdown on SIGINT/SIGTERM
- [ ] Configuration file is loaded when `--config` is provided

---
*Created: 2026-02-15*
