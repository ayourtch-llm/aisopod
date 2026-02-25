# Issue 161: Create OpenClaw Config Migration Utility and Add Deployment Tests

## Summary
Implement a CLI command `aisopod migrate --from openclaw` that converts OpenClaw JSON5 configuration files and environment variables to the aisopod format. Additionally, add smoke tests for the Docker build and deployment integration tests.

## Location
- Crate: `aisopod-cli`
- Files:
  - `crates/aisopod-cli/src/commands/migrate.rs`
  - `tests/integration/deployment_tests.rs`

## Current Behavior
There is no migration path from OpenClaw to aisopod. Users switching from OpenClaw must manually recreate their configuration. There are no automated tests for the Docker build or deployment configurations.

## Expected Behavior
A `migrate` CLI subcommand that reads an OpenClaw configuration file (JSON5 format) and outputs an equivalent aisopod configuration. Environment variable mapping from `OPENCLAW_*` to `AISOPOD_*` is handled automatically. Deployment smoke tests verify the Docker image builds and starts correctly.

## Impact
Lowers the barrier for OpenClaw users migrating to aisopod and ensures deployment configurations remain valid as the project evolves.

## Suggested Implementation

1. Add the `migrate` subcommand to the CLI in `commands/migrate.rs`:

```rust
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Map of OpenClaw config keys to aisopod equivalents
fn config_key_mapping() -> HashMap<&'static str, &'static str> {
    let mut map = HashMap::new();
    map.insert("server.port", "gateway.port");
    map.insert("server.host", "gateway.bind_address");
    map.insert("models", "providers");
    map.insert("tools", "tools");
    // Add more mappings as schemas diverge
    map
}

/// Map OPENCLAW_* env vars to AISOPOD_* equivalents
fn map_env_vars() -> Vec<(String, String)> {
    std::env::vars()
        .filter(|(k, _)| k.starts_with("OPENCLAW_"))
        .map(|(k, v)| {
            let new_key = k.replacen("OPENCLAW_", "AISOPOD_", 1);
            (new_key, v)
        })
        .collect()
}

pub fn run_migrate(from: &str, input_path: PathBuf, output_path: PathBuf) -> Result<()> {
    match from {
        "openclaw" => {
            let content = fs::read_to_string(&input_path)?;
            // Parse JSON5 input (use json5 crate)
            let openclaw_config: serde_json::Value = json5::from_str(&content)?;

            let aisopod_config = convert_openclaw_config(openclaw_config)?;

            let output = serde_json::to_string_pretty(&aisopod_config)?;
            fs::write(&output_path, output)?;

            // Print env var mappings
            let env_mappings = map_env_vars();
            if !env_mappings.is_empty() {
                println!("Environment variable mappings:");
                for (new_key, value) in &env_mappings {
                    println!("  {} = {}", new_key, value);
                }
            }

            println!("Config migrated to {}", output_path.display());
            Ok(())
        }
        other => Err(anyhow::anyhow!("Unknown source format: {}", other)),
    }
}
```

2. Add the `json5` dependency to the CLI crate's `Cargo.toml`:

```toml
[dependencies]
json5 = "0.4"
```

3. Create deployment smoke tests in `tests/integration/deployment_tests.rs`:

```rust
#[cfg(test)]
mod deployment_tests {
    use std::process::Command;

    #[test]
    #[ignore] // Run with: cargo test -- --ignored
    fn docker_image_builds() {
        let status = Command::new("docker")
            .args(["build", "-t", "aisopod-test", "."])
            .status()
            .expect("Failed to run docker build");
        assert!(status.success(), "Docker build failed");
    }

    #[test]
    #[ignore]
    fn docker_container_starts_and_responds() {
        // Start container in background
        let child = Command::new("docker")
            .args([
                "run", "-d", "--name", "aisopod-smoke-test",
                "-p", "18799:18789",
                "aisopod-test",
            ])
            .output()
            .expect("Failed to start container");
        assert!(child.status.success());

        // Wait for startup
        std::thread::sleep(std::time::Duration::from_secs(5));

        // Check health endpoint
        let health = Command::new("curl")
            .args(["-sf", "http://localhost:18799/health"])
            .output();

        // Cleanup
        let _ = Command::new("docker")
            .args(["rm", "-f", "aisopod-smoke-test"])
            .status();

        let health = health.expect("Failed to curl health endpoint");
        assert!(health.status.success(), "Health check failed");
    }
}
```

4. Add a migration test:

```rust
#[test]
fn migrate_openclaw_config() {
    let openclaw_json5 = r#"{
        server: { port: 3000, host: "localhost" },
        models: [{ name: "gpt-4" }]
    }"#;

    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), openclaw_json5).unwrap();

    let output = tempfile::NamedTempFile::new().unwrap();
    run_migrate("openclaw", tmp.path().into(), output.path().into()).unwrap();

    let result: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(output.path()).unwrap()).unwrap();
    assert_eq!(result["gateway"]["port"], 3000);
    assert_eq!(result["gateway"]["bind_address"], "localhost");
}
```

## Resolution

The OpenClaw configuration migration utility was successfully implemented with the following changes:

- Created `crates/aisopod/src/commands/migrate.rs` with migrate CLI subcommand
- Implemented config key mapping (OpenClaw -> aisopod): `server.port` -> `gateway.port`, `server.host` -> `gateway.bind_address`, etc.
- Implemented environment variable mapping (`OPENCLAW_*` -> `AISOPOD_*`)
- Implemented `convert_openclaw_config()` function to transform JSON5 to aisopod format
- Added `json5 = "0.4"` dependency to Cargo.toml
- Registered migrate subcommand in CLI (`aisopod migrate from-open-claw`)
- Created deployment smoke tests in `tests/deployment_tests.rs` (Docker build, health check - marked ignore)
- Added integration tests in `tests/integration_tests.rs` for migration
- Added unit tests for config key mapping, env var mapping, and migration functionality
- Fixed bug: comparison operator errors in test assertions (line 315 in migrate.rs, lines 161-162 in integration_tests.rs)
- All tests pass: 76 passed, 0 failed, 6 ignored
- All changes committed

## Dependencies
- Issue 016 (config types)
- Issue 124 (CLI application)

## Acceptance Criteria
- [ ] `aisopod migrate --from openclaw --input <file> --output <file>` converts configuration
- [ ] OpenClaw JSON5 config keys are mapped to aisopod equivalents
- [ ] `OPENCLAW_*` environment variables are mapped to `AISOPOD_*`
- [ ] Schema differences between OpenClaw and aisopod are handled gracefully
- [ ] Docker build smoke test passes (`cargo test -- --ignored`)
- [ ] Docker container starts and health check responds
- [ ] Migration unit tests pass
- [ ] Unknown source formats produce a clear error message

---
*Created: 2026-02-15*
*Resolved: 2026-02-25*
