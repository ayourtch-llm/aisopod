# Issue 129: Implement Status and Health Commands

## Summary
Implement the `aisopod status`, `aisopod health`, and `aisopod dashboard` commands for monitoring system state, verifying gateway readiness, and providing a live-updating dashboard view.

## Location
- Crate: `aisopod` (main binary crate)
- File: `src/commands/status.rs`

## Current Behavior
The status and health subcommands are stubs that panic with `todo!`. There is no way to check the running system's state from the CLI.

## Expected Behavior
Users can quickly check whether the gateway is running, view the status of channels and agents, run a health check that verifies connectivity and readiness, and optionally view a live-updating dashboard.

## Impact
Observability is critical for operators. These commands provide a quick way to diagnose whether the system is functioning correctly and identify any issues.

## Suggested Implementation

1. Define the status command (no subcommands needed):

```rust
use clap::Args;

#[derive(Args)]
pub struct StatusArgs {
    /// Show extended details
    #[arg(long)]
    pub detailed: bool,
}

#[derive(Args)]
pub struct HealthArgs {}
```

2. Implement the status handler:

```rust
pub async fn run_status(args: StatusArgs, config_path: Option<String>) -> anyhow::Result<()> {
    let config = load_config(config_path)?;
    let gateway_url = config.gateway_http_url();

    // Check gateway connectivity
    let client = reqwest::Client::new();
    let gateway_status = match client.get(format!("{}/health", gateway_url)).send().await {
        Ok(resp) if resp.status().is_success() => "running",
        Ok(_) => "unhealthy",
        Err(_) => "not running",
    };

    println!("Gateway:  {}", gateway_status);

    if gateway_status == "running" {
        // Fetch detailed status from gateway
        let status: SystemStatus = client
            .get(format!("{}/status", gateway_url))
            .send().await?
            .json().await?;

        println!("Agents:   {} configured", status.agent_count);
        println!("Channels: {} active", status.active_channels);
        println!("Sessions: {} active", status.active_sessions);
        println!("Uptime:   {}", format_duration(status.uptime));
    }

    Ok(())
}
```

3. Implement the health check handler:

```rust
pub async fn run_health(config_path: Option<String>) -> anyhow::Result<()> {
    let config = load_config(config_path)?;
    let gateway_url = config.gateway_http_url();

    println!("Running health checks...\n");

    // Check 1: Gateway reachable
    let client = reqwest::Client::new();
    let gw_ok = client.get(format!("{}/health", gateway_url)).send().await.is_ok();
    print_check("Gateway reachable", gw_ok);

    // Check 2: Configuration valid
    let config_ok = config.validate().is_ok();
    print_check("Configuration valid", config_ok);

    // Check 3: At least one agent configured
    let agents_ok = !config.agents().is_empty();
    print_check("Agents configured", agents_ok);

    let all_ok = gw_ok && config_ok && agents_ok;
    println!("\nOverall: {}", if all_ok { "HEALTHY" } else { "UNHEALTHY" });

    Ok(())
}

fn print_check(name: &str, ok: bool) {
    let symbol = if ok { "✓" } else { "✗" };
    println!("  {} {}", symbol, name);
}
```

4. Implement a basic live dashboard (using crossterm for terminal control):

```rust
pub async fn run_dashboard(config_path: Option<String>) -> anyhow::Result<()> {
    let config = load_config(config_path)?;

    loop {
        // Clear screen and move cursor to top
        print!("\x1B[2J\x1B[H");

        // Fetch and display status
        run_status(StatusArgs { detailed: true }, config_path.clone()).await?;

        // Refresh every 2 seconds
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
}
```

## Dependencies
- Issue 124 (clap CLI framework)
- Issue 026 (gateway health endpoint)

## Resolution

The status and health commands have been fully implemented with the following components:

**1. Status Command (`aisopod status`):**
- Shows gateway status (running, unhealthy, or not running)
- Fetches and displays detailed system information when gateway is running:
  - Agent count
  - Active channel count
  - Active session count
  - Gateway uptime (formatted as human-readable string)
- Supports `--json` flag for structured JSON output
- Uses the `/health` and `/status` endpoints from the gateway API

**2. Health Check Command (`aisopod health`):**
- Runs three health checks:
  - Gateway reachable via HTTP `/health` endpoint
  - Configuration file is valid (passes schema and semantic validation)
  - At least one agent is configured
- Outputs check results with checkmark (✓) or cross (✗) symbols
- Supports `--json` flag for structured JSON output
- Sets non-zero exit code when any health check fails
- Overall status shows HEALTHY or UNHEALTHY based on all checks

**3. Dashboard Command (`aisopod dashboard`):**
- Displays a live-updating status view
- Clears terminal screen and refreshes status every 2 seconds
- Provides real-time monitoring of system state
- Uses ANSI escape codes for screen clearing and cursor positioning

**4. Duration Formatting Utility:**
- `format_duration()` function converts seconds to human-readable format
- Handles various time scales:
  - Seconds: `30s`
  - Minutes: `1m 30s`
  - Hours: `2h 15m`
  - Days: `1d 2h`

**5. Error Handling:**
- Gracefully handles gateway not running (connection refused)
- Provides meaningful error messages for different failure modes
- Proper JSON parsing with error propagation
- Configurable config file path with sensible defaults

**Files Modified:**
- `crates/aisopod/src/commands/status.rs` - Full implementation of all commands
- `crates/aisopod/src/cli.rs` - Command registration and dispatch
- `crates/aisopod/src/commands/mod.rs` - Module exports

**Tests:**
- Unit tests for `format_duration()` covering all time ranges
- Integration with gateway API endpoints
- JSON output format validation

---
*Created: 2026-02-15*
*Resolved: 2026-02-24*
