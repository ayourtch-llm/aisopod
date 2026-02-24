//! Doctor command implementation for system diagnostics.
//!
//! This module provides the `aisopod doctor` command that runs comprehensive
//! system diagnostics and reports the results.

use anyhow::Result;
use clap::Args;
use std::io::{self, Write};
use std::path::Path;

use aisopod_config::{load_config, default_config_path, AisopodConfig};

/// Doctor command arguments
#[derive(Args)]
pub struct DoctorArgs {
    /// Run extended diagnostics
    #[arg(long)]
    pub verbose: bool,
}

/// Construct the gateway HTTP URL from config
fn gateway_http_url(gateway_config: &aisopod_config::types::GatewayConfig) -> String {
    let bind_addr = &gateway_config.bind.address;
    let port = gateway_config.server.port;
    format!("http://{}:{}", bind_addr, port)
}

/// Load configuration from file or use defaults
fn load_config_or_default(config_path: Option<&str>) -> Result<aisopod_config::AisopodConfig> {
    match config_path {
        Some(path) => {
            let config_path = Path::new(path);
            load_config(config_path).map_err(|e| {
                anyhow::anyhow!("Failed to load configuration from '{}': {}", path, e)
            })
        }
        None => {
            // Use default config path
            let default_path = aisopod_config::default_config_path();
            if default_path.exists() {
                load_config(&default_path).map_err(|e| {
                    anyhow::anyhow!("Failed to load configuration from '{}': {}", default_path.display(), e)
                })
            } else {
                // If no config file exists, return empty config
                Ok(aisopod_config::AisopodConfig::default())
            }
        }
    }
}

/// Run the doctor command with the given arguments and config path
pub async fn run_doctor(args: DoctorArgs, config_path: Option<String>) -> Result<()> {
    println!("aisopod Doctor\n");
    println!("Running diagnostics...\n");

    let mut passed = 0;
    let mut failed = 0;

    // Check 1: Configuration file exists and is valid
    let config_result = load_config_or_default(config_path.as_deref());
    let config_err = config_result.as_ref().err().map(|e| e.to_string());
    let config_ok = config_result.is_ok();
    print_diagnostic("Configuration file", config_ok, config_err);
    if config_ok { passed += 1; } else { failed += 1; }

    // Check 2: At least one authentication profile configured
    if let Ok(ref config) = config_result {
        let auth_profiles_ok = !config.auth.profiles.is_empty();
        print_diagnostic("Authentication profiles configured", auth_profiles_ok, None);
        if auth_profiles_ok { passed += 1; } else { failed += 1; }

        // Check 3: API keys are set for configured auth profiles
        for profile in &config.auth.profiles {
            // Check if API key is set (non-empty)
            let key_set = !profile.api_key.expose().is_empty();
            print_diagnostic(
                &format!("  {} API key", profile.name),
                key_set,
                if key_set { None } else { Some("API key not configured".to_string()) },
            );
            if key_set { passed += 1; } else { failed += 1; }
        }

        // Check 4: Gateway connectivity
        let client = reqwest::Client::new();
        let gw_url = gateway_http_url(&config.gateway);
        let gw_ok = match client.get(format!("{}/health", gw_url))
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
        {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        };
        print_diagnostic("Gateway reachable", gw_ok,
            if gw_ok { None } else { Some("Gateway is not running".to_string()) });
        if gw_ok { passed += 1; } else { failed += 1; }

        // Check 5: Network connectivity (can reach external APIs)
        if args.verbose {
            let net_ok = client.get("https://api.openai.com")
                .timeout(std::time::Duration::from_secs(5))
                .send().await
                .is_ok();
            print_diagnostic("External network access", net_ok, None);
            if net_ok { passed += 1; } else { failed += 1; }
        }
    }

    println!("\n{} passed, {} failed", passed, failed);
    
    if failed > 0 {
        writeln!(io::stderr(), "{} diagnostic check(s) failed", failed)?;
        std::process::exit(1);
    }
    
    Ok(())
}

/// Print a diagnostic check result
fn print_diagnostic(name: &str, ok: bool, detail: Option<String>) {
    let symbol = if ok { "✓" } else { "✗" };
    print!("  {} {}", symbol, name);
    if let Some(d) = detail {
        print!(" ({})", d);
    }
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doctor_args_default() {
        let args = DoctorArgs {
            verbose: false,
        };

        assert!(!args.verbose);
    }

    #[test]
    fn test_doctor_args_verbose() {
        let args = DoctorArgs {
            verbose: true,
        };

        assert!(args.verbose);
    }
}
