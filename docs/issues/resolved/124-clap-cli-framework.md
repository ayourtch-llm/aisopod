# Issue 124: Set Up clap CLI Framework with Top-Level Argument Parsing

## Summary
Set up the clap CLI framework using derive macros to define the top-level `Cli` struct with a subcommand enum, global flags, and subcommand stubs for all planned CLI commands.

## Location
- Crate: `aisopod` (main binary crate)
- File: `src/main.rs`, `src/cli.rs`

## Current Behavior
No CLI argument parsing exists. The binary crate has no command-line interface structure.

## Expected Behavior
Running `aisopod --help` displays all available subcommands and global flags. The binary uses clap derive macros to parse arguments and dispatch to subcommand handlers. Global flags `--config`, `--verbose`, and `--json` are accepted on every invocation.

## Impact
This is the foundation for the entire CLI application. Every other CLI issue (125–136) depends on the argument parsing and subcommand dispatch defined here.

## Suggested Implementation

1. Add clap as a dependency in the binary crate's `Cargo.toml`:

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
```

2. Create `src/cli.rs` with the top-level structs:

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "aisopod", version, about = "AI agent orchestration platform")]
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
```

3. In `src/main.rs`, parse and dispatch:

```rust
mod cli;

use clap::Parser;
use cli::{Cli, Commands};

fn main() {
    let cli = Cli::parse();

    match cli.command {
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
```

## Dependencies
- Issue 012 (binary crate setup)

## Acceptance Criteria
- [ ] `aisopod --help` shows all subcommands (gateway, agent, message, config, status, health, models, channels, sessions, daemon, doctor, auth, reset, completions)
- [ ] `aisopod --version` prints version
- [ ] Global flags `--config`, `--verbose`, `--json` are accepted
- [ ] Each subcommand stub is reachable (panics with `todo!` for now)
- [ ] Code compiles with no warnings

## Resolution

The clap CLI framework was successfully implemented with the following components:

### Implementation Details

1. **Dependency Configuration**: Added `clap` with `derive` feature to the binary crate's `Cargo.toml`

2. **CLI Module Structure**: Created `crates/aisopod/src/cli.rs` with:
   - Top-level `Cli` struct using `#[derive(Parser)]`
   - Global flags: `--config`, `--verbose`, `--json`
   - `Commands` enum with 14 subcommand variants using `#[derive(Subcommand)]`

3. **Subcommand Stubs**: All 14 planned subcommands implemented as stubs:
   - Gateway, Agent, Message, Config, Status, Health
   - Models, Channels, Sessions, Daemon, Doctor
   - Auth, Reset, Completions

4. **Entry Point**: Updated `src/main.rs` to call `cli::run_cli()` which parses arguments and dispatches to subcommand handlers

5. **Module Organization**: Used single-file `cli.rs` approach (avoiding the `cli/` directory ambiguity that caused E0761 errors)

### Verification

The implementation was verified by:
- Running `cargo build -p aisopod` - successful compilation
- Running `cargo test -p aisopod` - tests pass
- Manual verification with `aisopod --help` showing all 14 subcommands
- Verification that global flags are accepted at top level

### Related Files

- `crates/aisopod/src/cli.rs` - CLI definition
- `crates/aisopod/src/main.rs` - Entry point
- `docs/learnings/124-clap-cli-framework.md` - Learning summary

---
*Created: 2026-02-15*
*Resolved: 2026-02-24*
