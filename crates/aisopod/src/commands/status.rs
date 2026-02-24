//! Status and health commands for the aisopod application.
//!
//! This module provides commands for monitoring system state, verifying gateway readiness,
//! and providing a live-updating dashboard view.

use anyhow::Result;
use clap::Args;
use serde_json::{json, Value};
use std::path::Path;
use std::time::{Duration, Instant};

use aisopod_config::load_config;
use aisopod_gateway::GatewayStatus;

/// Format a duration in seconds to a human-readable string
pub fn format_duration(seconds: u64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        let mins = seconds / 60;
        let secs = seconds % 60;
        format!("{}m {}s", mins, secs)
    } else if seconds < 86400 {
        let hours = seconds / 3600;
        let mins = (seconds % 3600) / 60;
        format!("{}h {}m", hours, mins)
    } else {
        let days = seconds / 86400;
        let hours = (seconds % 86400) / 3600;
        format!("{}d {}h", days, hours)
    }
}

/// Construct the gateway HTTP URL from config
fn gateway_http_url(gateway_config: &aisopod_config::types::GatewayConfig) -> String {
    let bind_addr = &gateway_config.bind.address;
    let port = gateway_config.server.port;
    format!("http://{}:{}", bind_addr, port)
}

/// Status command arguments
#[derive(Args, Clone)]
pub struct StatusArgs {
    /// Show extended details
    #[arg(long)]
    pub detailed: bool,
}

/// Health command arguments
#[derive(Args)]
pub struct HealthArgs {
    /// Output in JSON format
    #[arg(long)]
    pub json: bool,
}

/// Run the status command
pub async fn run_status(args: StatusArgs, config_path: Option<String>, json: bool) -> Result<()> {
    // Load config from default path if not specified
    let config_path = config_path
        .as_deref()
        .unwrap_or("aisopod-config.json5");
    let config = load_config(Path::new(config_path))?;
    let gateway_url = gateway_http_url(&config.gateway);

    // Check gateway connectivity
    let client = reqwest::Client::new();
    let gateway_status = match client
        .get(format!("{}/health", gateway_url))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => "running",
        Ok(_) => "unhealthy",
        Err(_) => "not running",
    };

    if json {
        // JSON output mode
        let status_json = json!({
            "gateway_status": gateway_status,
        });
        println!("{}", serde_json::to_string_pretty(&status_json)?);
        
        if gateway_status == "running" {
            // Fetch detailed status from gateway
            let status: GatewayStatus = client
                .get(format!("{}/status", gateway_url))
                .send()
                .await?
                .json()
                .await?;

            let detailed_json = json!({
                "gateway_status": {
                    "agent_count": status.agent_count,
                    "active_channels": status.active_channels,
                    "active_sessions": status.active_sessions,
                    "uptime": status.uptime,
                    "uptime_formatted": format_duration(status.uptime)
                }
            });
            println!("{}", serde_json::to_string_pretty(&detailed_json)?);
        }
    } else {
        // Human-readable output
        println!("Gateway:  {}", gateway_status);

        if gateway_status == "running" {
            // Fetch detailed status from gateway
            let status: GatewayStatus = client
                .get(format!("{}/status", gateway_url))
                .send()
                .await?
                .json()
                .await?;

            println!("Agents:   {} configured", status.agent_count);
            println!("Channels: {} active", status.active_channels);
            println!("Sessions: {} active", status.active_sessions);
            println!("Uptime:   {}", format_duration(status.uptime));
        }
    }

    Ok(())
}

/// Run the health check command
pub async fn run_health(config_path: Option<String>, json: bool) -> Result<()> {
    // Load config from default path if not specified
    let config_path = config_path
        .as_deref()
        .unwrap_or("aisopod-config.json5");
    let config = load_config(Path::new(config_path))?;
    let gateway_url = gateway_http_url(&config.gateway);

    if json {
        // JSON output mode
        let client = reqwest::Client::new();
        let gw_ok = client
            .get(format!("{}/health", gateway_url))
            .send()
            .await
            .is_ok();
        
        let config_ok = config.validate().is_ok();
        let agents_ok = !config.agents.agents.is_empty();
        let all_ok = gw_ok && config_ok && agents_ok;

        let result = json!({
            "checks": {
                "gateway_reachable": gw_ok,
                "configuration_valid": config_ok,
                "agents_configured": agents_ok,
            },
            "overall": if all_ok { "HEALTHY" } else { "UNHEALTHY" }
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
        
        // Set non-zero exit code if health checks fail
        if !all_ok {
            std::process::exit(1);
        }
    } else {
        // Human-readable output
        println!("Running health checks...\n");

        // Check 1: Gateway reachable
        let client = reqwest::Client::new();
        let gw_ok = client
            .get(format!("{}/health", gateway_url))
            .send()
            .await
            .is_ok();
        print_check("Gateway reachable", gw_ok);

        // Check 2: Configuration valid
        let config_ok = config.validate().is_ok();
        print_check("Configuration valid", config_ok);

        // Check 3: At least one agent configured
        let agents_ok = !config.agents.agents.is_empty();
        print_check("Agents configured", agents_ok);

        let all_ok = gw_ok && config_ok && agents_ok;
        println!("\nOverall: {}", if all_ok { "HEALTHY" } else { "UNHEALTHY" });

        // Set non-zero exit code if health checks fail
        if !all_ok {
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Print a health check result
fn print_check(name: &str, ok: bool) {
    let symbol = if ok { "✓" } else { "✗" };
    println!("  {} {}", symbol, name);
}

/// Run the dashboard command - displays a live-updating status view
pub async fn run_dashboard(config_path: Option<String>) -> Result<()> {
    let config_path_clone = config_path.clone();
    let config_path = config_path
        .as_deref()
        .unwrap_or("aisopod-config.json5");
    let args = StatusArgs { detailed: true };

    loop {
        // Clear screen and move cursor to top (ANSI escape codes)
        print!("\x1B[2J\x1B[H");

        // Fetch and display status
        run_status(args.clone(), config_path_clone.clone(), false).await?;

        // Refresh every 2 seconds
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration_seconds() {
        assert_eq!(format_duration(0), "0s");
        assert_eq!(format_duration(1), "1s");
        assert_eq!(format_duration(30), "30s");
        assert_eq!(format_duration(59), "59s");
    }

    #[test]
    fn test_format_duration_minutes() {
        assert_eq!(format_duration(60), "1m 0s");
        assert_eq!(format_duration(90), "1m 30s");
        assert_eq!(format_duration(120), "2m 0s");
        assert_eq!(format_duration(3599), "59m 59s");
    }

    #[test]
    fn test_format_duration_hours() {
        assert_eq!(format_duration(3600), "1h 0m");
        assert_eq!(format_duration(7200), "2h 0m");
        assert_eq!(format_duration(3661), "1h 1m");
        assert_eq!(format_duration(86399), "23h 59m");
    }

    #[test]
    fn test_format_duration_days() {
        assert_eq!(format_duration(86400), "1d 0h");
        assert_eq!(format_duration(172800), "2d 0h");
        assert_eq!(format_duration(90000), "1d 1h");
    }
}
