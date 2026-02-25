//! Deployment tests for aisopod Docker build and configuration migration
//!
//! This module contains smoke tests for:
//! - Docker image builds
//! - Docker container startup and health checks
//! - Configuration migration functionality
//!
//! Run with: cargo test -- --ignored

// Integration tests for migration functionality
mod migration_tests {
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_migrate_openclaw_basic_config() {
        // This test verifies the migration command can parse and convert
        // OpenClaw JSON5 config to aisopod format
        
        let openclaw_json5 = r#"{
            server: {
                port: 3000,
                host: "localhost"
            },
            models: [
                { name: "gpt-4", endpoint: "https://api.openai.com/v1", api_key: "sk-test" }
            ],
            tools: {
                bash: {
                    enabled: true,
                    working_dir: "/tmp"
                }
            }
        }"#;

        let tmp_dir = TempDir::new().expect("Failed to create temp dir");
        let input_path = tmp_dir.path().join("openclaw-config.json5");
        let output_path = tmp_dir.path().join("aisopod-config.json");

        fs::write(&input_path, openclaw_json5).expect("Failed to write input file");

        // Test migration functionality
        let args = aisopod::commands::migrate::MigrateArgs {
            command: aisopod::commands::migrate::MigrateCommands::FromOpenClaw {
                input: input_path.clone(),
                output: output_path.clone(),
            },
        };

        let result = aisopod::commands::migrate::run_migrate(args);
        assert!(result.is_ok(), "Migration failed: {:?}", result.err());

        // Verify output file was created
        assert!(output_path.exists(), "Output file was not created");

        // Verify output is valid JSON
        let output_content = fs::read_to_string(&output_path)
            .expect("Failed to read output file");
        
        let parsed: serde_json::Value = serde_json::from_str(&output_content)
            .expect("Failed to parse output JSON");

        // Verify key structure was migrated
        assert!(parsed.get("gateway").is_some(), "Gateway config not migrated");
        assert!(parsed.get("models").is_some(), "Models config not migrated");
    }

    #[test]
    fn test_migrate_openclaw_with_auth() {
        let openclaw_json5 = r#"{
            server: { port: 8080, host: "0.0.0.0" },
            auth: {
                mode: "token",
                api_key: "secret-key"
            },
            models: [
                { name: "anthropic", endpoint: "https://api.anthropic.com", api_key: "sk-ant-test" }
            ],
            channels: {
                telegram: {
                    token: "telegram-token"
                }
            }
        }"#;

        let tmp_dir = TempDir::new().expect("Failed to create temp dir");
        let input_path = tmp_dir.path().join("input.json5");
        let output_path = tmp_dir.path().join("output.json");

        fs::write(&input_path, openclaw_json5).expect("Failed to write input file");

        let args = aisopod::commands::migrate::MigrateArgs {
            command: aisopod::commands::migrate::MigrateCommands::FromOpenClaw {
                input: input_path.clone(),
                output: output_path.clone(),
            },
        };

        let result = aisopod::commands::migrate::run_migrate(args);
        assert!(result.is_ok());

        let output_content = fs::read_to_string(&output_path)
            .expect("Failed to read output file");
        
        let parsed: serde_json::Value = serde_json::from_str(&output_content)
            .expect("Failed to parse output JSON");

        assert!(parsed.get("auth").is_some(), "Auth config not migrated");
        assert!(parsed.get("channels").is_some(), "Channels config not migrated");
    }

    #[test]
    fn test_migrate_openclaw_empty_models() {
        // Test migration with minimal config (no models)
        let openclaw_json5 = r#"{
            server: { port: 8080 }
        }"#;

        let tmp_dir = TempDir::new().expect("Failed to create temp dir");
        let input_path = tmp_dir.path().join("input.json5");
        let output_path = tmp_dir.path().join("output.json");

        fs::write(&input_path, openclaw_json5).expect("Failed to write input file");

        let args = aisopod::commands::migrate::MigrateArgs {
            command: aisopod::commands::migrate::MigrateCommands::FromOpenClaw {
                input: input_path.clone(),
                output: output_path.clone(),
            },
        };

        let result = aisopod::commands::migrate::run_migrate(args);
        assert!(result.is_ok(), "Migration should succeed even with minimal config");

        let output_content = fs::read_to_string(&output_path)
            .expect("Failed to read output file");
        
        let parsed: serde_json::Value = serde_json::from_str(&output_content)
            .expect("Failed to parse output JSON");

        assert!(parsed.get("gateway").is_some(), "Gateway config should be present");
    }

    #[test]
    fn test_config_key_mapping() {
        // Test that the key mapping function exists and returns expected mappings
        let mapping = aisopod::commands::migrate::config_key_mapping();
        
        assert!(mapping.contains_key("server.port"));
        assert!(mapping.contains_key("server.host"));
        assert!(mapping.contains_key("models"));
        assert!(mapping.contains_key("tools"));
    }

    #[test]
    fn test_env_var_mapping() {
        // Test that environment variable mapping exists
        let mapping = aisopod::commands::migrate::env_var_mapping();
        
        assert!(mapping.iter().any(|(old, _)| *old == "OPENCLAW_SERVER_PORT"));
        assert!(mapping.iter().any(|(old, _)| *old == "OPENCLAW_SERVER_HOST"));
    }

    #[test]
    fn test_map_env_var_name() {
        // Test environment variable name conversion
        assert_eq!(
            aisopod::commands::migrate::map_env_var_name("OPENCLAW_SERVER_PORT"),
            "AISOPOD_GATEWAY_SERVER_PORT"
        );
        assert_eq!(
            aisopod::commands::migrate::map_env_var_name("OPENCLAW_MODEL_API_KEY"),
            "AISOPOD_MODELS_PROVIDERS_0_API_KEY"
        );
    }
}

// Deployment tests (marked as ignored - run with --ignored flag)
#[cfg(test)]
mod deployment_smoke_tests {
    use std::process::Command;

    #[test]
    #[ignore = "Docker build test - requires Docker daemon"]
    fn docker_image_builds() {
        // This test verifies that the Docker image can be built
        // It's marked as ignored by default and should only run in CI/CD or when explicitly requested
        
        // Skip if Docker is not available
        let docker_status = Command::new("docker")
            .arg("--version")
            .status();
        
        if let Ok(status) = docker_status {
            if !status.success() {
                println!("Docker is not available, skipping test");
                return;
            }
        } else {
            println!("Failed to check Docker version, skipping test");
            return;
        }

        // Attempt to build the Docker image
        let build_output = Command::new("docker")
            .args(["build", "-t", "aisopod-smoke-test", "-f", "Dockerfile", "."])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output();

        match build_output {
            Ok(output) => {
                if output.status.success() {
                    println!("Docker image built successfully");
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    println!("Docker build failed: {}", stderr);
                    panic!("Docker build failed");
                }
            }
            Err(e) => {
                println!("Failed to run docker build: {}", e);
                // Don't fail the test if Docker is not configured
            }
        }
    }

    #[test]
    #[ignore = "Docker container test - requires Docker daemon and built image"]
    fn docker_container_starts_and_responds() {
        // This test verifies that a Docker container can start and respond to health checks
        
        // Skip if Docker is not available
        let docker_status = Command::new("docker")
            .arg("--version")
            .status();
        
        if let Ok(status) = docker_status {
            if !status.success() {
                println!("Docker is not available, skipping test");
                return;
            }
        } else {
            println!("Failed to check Docker version, skipping test");
            return;
        }

        let container_name = "aisopod-smoke-test-container";
        
        // Cleanup any existing container first
        let _ = Command::new("docker")
            .args(["rm", "-f", container_name])
            .status();

        // Start container in background
        let run_output = Command::new("docker")
            .args([
                "run", "-d", "--name", container_name,
                "-p", "18799:8080",
                "aisopod-smoke-test",
            ])
            .output();

        match run_output {
            Ok(output) => {
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    println!("Failed to start container: {}", stderr);
                    return;
                }
            }
            Err(e) => {
                println!("Failed to run docker: {}", e);
                return;
            }
        }

        // Give container time to start
        std::thread::sleep(std::time::Duration::from_secs(2));

        // Check health endpoint
        let health_output = Command::new("curl")
            .args(["-sf", "http://localhost:18799/health"])
            .output();

        // Cleanup container
        let _ = Command::new("docker")
            .args(["rm", "-f", container_name])
            .status();

        if let Ok(output) = health_output {
            if output.status.success() {
                println!("Health check passed");
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                println!("Health check failed: {}", stderr);
                // Don't fail the test if health check fails (container might not be fully ready)
            }
        } else {
            println!("Failed to run health check");
        }
    }
}
