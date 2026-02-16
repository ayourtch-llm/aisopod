# Issue 134: Implement Diagnostic and Auth Commands

## Summary
Implement the `aisopod doctor` command for running system diagnostics and the `aisopod auth` subcommands for interactive authentication setup and status checking.

## Location
- Crate: `aisopod` (main binary crate)
- File: `src/commands/doctor.rs`, `src/commands/auth.rs`

## Current Behavior
The doctor and auth subcommands are stubs that panic with `todo!`. There is no CLI interface for diagnostics or authentication management.

## Expected Behavior
The `doctor` command runs a comprehensive set of diagnostic checks (dependencies, connectivity, configuration validity) and reports results. The `auth` subcommands guide users through setting up API keys and authentication profiles and display current auth status.

## Impact
Diagnostics help users self-service troubleshooting without filing support requests. Auth management ensures users can securely configure API keys for model providers.

## Suggested Implementation

1. Define the doctor command (no subcommands):

```rust
use clap::Args;

#[derive(Args)]
pub struct DoctorArgs {
    /// Run extended diagnostics
    #[arg(long)]
    pub verbose: bool,
}
```

2. Implement the doctor handler:

```rust
pub async fn run_doctor(args: DoctorArgs, config_path: Option<String>) -> anyhow::Result<()> {
    println!("aisopod Doctor\n");
    println!("Running diagnostics...\n");

    let mut passed = 0;
    let mut failed = 0;

    // Check 1: Configuration file exists and is valid
    let config_result = load_config(config_path.clone());
    let config_ok = config_result.is_ok();
    print_diagnostic("Configuration file", config_ok, config_result.err().map(|e| e.to_string()));
    if config_ok { passed += 1; } else { failed += 1; }

    // Check 2: At least one model provider configured
    if let Ok(ref config) = load_config(config_path.clone()) {
        let providers_ok = !config.model_providers().is_empty();
        print_diagnostic("Model providers configured", providers_ok, None);
        if providers_ok { passed += 1; } else { failed += 1; }

        // Check 3: API keys are set for configured providers
        for provider in config.model_providers() {
            let key_set = config.has_auth_for_provider(provider.name());
            print_diagnostic(
                &format!("  {} API key", provider.name()),
                key_set,
                if key_set { None } else { Some("API key not configured".to_string()) },
            );
            if key_set { passed += 1; } else { failed += 1; }
        }

        // Check 4: Gateway connectivity
        let client = reqwest::Client::new();
        let gw_url = config.gateway_http_url();
        let gw_ok = client.get(format!("{}/health", gw_url))
            .timeout(std::time::Duration::from_secs(5))
            .send().await
            .is_ok();
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
    Ok(if failed > 0 {
        anyhow::bail!("{} diagnostic check(s) failed", failed);
    })
}

fn print_diagnostic(name: &str, ok: bool, detail: Option<String>) {
    let symbol = if ok { "✓" } else { "✗" };
    print!("  {} {}", symbol, name);
    if let Some(d) = detail {
        print!(" ({})", d);
    }
    println!();
}
```

3. Define the auth subcommands:

```rust
#[derive(Args)]
pub struct AuthArgs {
    #[command(subcommand)]
    pub command: AuthCommands,
}

#[derive(Subcommand)]
pub enum AuthCommands {
    /// Interactive authentication setup
    Setup,
    /// Show current auth status
    Status,
}
```

4. Implement the auth setup wizard:

```rust
pub fn run_auth_setup(config_path: Option<String>) -> anyhow::Result<()> {
    let mut config = load_config(config_path)?;

    println!("=== Authentication Setup ===\n");

    let provider = prompt_select(
        "Select provider to configure",
        &["openai", "anthropic", "google", "azure", "local"],
    )?;

    match provider.as_str() {
        "openai" => {
            println!("\nGet your API key from: https://platform.openai.com/api-keys\n");
            let key = prompt_password("OpenAI API key: ")?;
            config.set_value("auth.openai.api_key", &key)?;
            let org = prompt("Organization ID (optional, press Enter to skip): ")?;
            if !org.is_empty() {
                config.set_value("auth.openai.org_id", &org)?;
            }
        }
        "anthropic" => {
            println!("\nGet your API key from: https://console.anthropic.com/settings/keys\n");
            let key = prompt_password("Anthropic API key: ")?;
            config.set_value("auth.anthropic.api_key", &key)?;
        }
        "google" => {
            println!("\nGet your API key from: https://aistudio.google.com/apikey\n");
            let key = prompt_password("Google AI API key: ")?;
            config.set_value("auth.google.api_key", &key)?;
        }
        "azure" => {
            let endpoint = prompt("Azure endpoint URL: ")?;
            let key = prompt_password("Azure API key: ")?;
            config.set_value("auth.azure.endpoint", &endpoint)?;
            config.set_value("auth.azure.api_key", &key)?;
        }
        "local" => {
            let endpoint = prompt_with_default("Local endpoint URL", "http://localhost:11434")?;
            config.set_value("auth.local.endpoint", &endpoint)?;
        }
        _ => {}
    }

    config.save()?;
    println!("\nAuthentication for '{}' configured successfully!", provider);
    Ok(())
}
```

5. Implement auth status:

```rust
pub fn run_auth_status(config_path: Option<String>) -> anyhow::Result<()> {
    let config = load_config(config_path)?;

    println!("Authentication Status\n");
    println!("{:<15} {:<15} {:<20}", "Provider", "Status", "Details");
    println!("{}", "-".repeat(50));

    let providers = ["openai", "anthropic", "google", "azure", "local"];
    for provider in providers {
        let has_key = config.has_auth_for_provider(provider);
        let status = if has_key { "Configured" } else { "Not set" };
        let detail = if has_key { "API key set" } else { "Run 'aisopod auth setup'" };
        println!("{:<15} {:<15} {:<20}", provider, status, detail);
    }

    Ok(())
}
```

## Dependencies
- Issue 124 (clap CLI framework)
- Issue 040 (auth profiles)
- Issue 016 (configuration types)

## Acceptance Criteria
- [ ] `aisopod doctor` runs all diagnostic checks and reports pass/fail
- [ ] `aisopod doctor --verbose` runs extended diagnostics including network checks
- [ ] Exit code is non-zero when any diagnostic fails
- [ ] `aisopod auth setup` walks through provider selection and key configuration
- [ ] `aisopod auth status` displays the auth state of all providers
- [ ] API keys are stored securely and never displayed in plain text
- [ ] Helpful URLs and instructions are shown during auth setup

---
*Created: 2026-02-15*
