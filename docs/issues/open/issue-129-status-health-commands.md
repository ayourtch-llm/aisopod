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

## Acceptance Criteria
- [ ] `aisopod status` shows gateway status, agent count, channel count, and session count
- [ ] `aisopod health` runs health checks and reports pass/fail for each
- [ ] `aisopod dashboard` displays a live-updating status view
- [ ] Commands handle gracefully when gateway is not running
- [ ] JSON output mode (`--json`) returns structured data
- [ ] Exit code is non-zero when health checks fail

---
*Created: 2026-02-15*
