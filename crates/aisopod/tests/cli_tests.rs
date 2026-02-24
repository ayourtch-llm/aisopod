//! CLI tests for the aisopod application
//!
//! This module contains unit tests for CLI argument parsing and
//! integration tests for CLI command execution.

use clap::Parser;
use aisopod::cli::{Cli, Commands};
use aisopod::commands::agent::AgentCommands;
use aisopod::commands::config::ConfigCommands;

// ============================================================================
// Unit Tests: Argument Parsing
// ============================================================================

#[test]
fn test_parse_gateway_defaults() {
    let cli = Cli::parse_from(["aisopod", "gateway"]);
    assert!(matches!(cli.command, Commands::Gateway(_)));
    assert!(!cli.verbose);
    assert!(!cli.json);
    assert!(cli.config.is_none());
}

#[test]
fn test_parse_global_flags() {
    let cli = Cli::parse_from(["aisopod", "--verbose", "--json", "--config", "/tmp/config.toml", "status"]);
    assert!(cli.verbose);
    assert!(cli.json);
    assert_eq!(cli.config.as_deref(), Some("/tmp/config.toml"));
}

#[test]
fn test_parse_gateway_with_flags() {
    let cli = Cli::parse_from(["aisopod", "gateway", "--bind", "0.0.0.0", "--port", "8080", "--allow-unconfigured"]);
    if let Commands::Gateway(args) = cli.command {
        assert_eq!(args.bind, "0.0.0.0");
        assert_eq!(args.port, 8080);
        assert!(args.allow_unconfigured);
    } else {
        panic!("Expected Gateway command");
    }
}

#[test]
fn test_parse_agent_list() {
    let cli = Cli::parse_from(["aisopod", "agent", "list"]);
    assert!(matches!(cli.command, Commands::Agent(_)));
}

#[test]
fn test_parse_agent_add() {
    let cli = Cli::parse_from(["aisopod", "agent", "add", "myagent"]);
    if let Commands::Agent(args) = cli.command {
        match args.command {
            AgentCommands::Add { id } => {
                assert_eq!(id, "myagent");
            }
            _ => panic!("Expected Add subcommand"),
        }
    }
}

#[test]
fn test_parse_message_with_channel() {
    let cli = Cli::parse_from(["aisopod", "message", "--channel", "telegram", "Hello world"]);
    if let Commands::Message(args) = cli.command {
        assert_eq!(args.text, "Hello world");
        assert_eq!(args.channel.as_deref(), Some("telegram"));
    } else {
        panic!("Expected Message command");
    }
}

#[test]
fn test_parse_completions() {
    let cli = Cli::parse_from(["aisopod", "completions", "bash"]);
    assert!(matches!(cli.command, Commands::Completions(_)));
}

#[test]
fn test_parse_config_set() {
    let cli = Cli::parse_from(["aisopod", "config", "set", "gateway.port", "8080"]);
    if let Commands::Config(args) = cli.command {
        match args.command {
            ConfigCommands::Set { key, value } => {
                assert_eq!(key, "gateway.port");
                assert_eq!(value, "8080");
            }
            _ => panic!("Expected Set subcommand"),
        }
    }
}

#[test]
fn test_parse_status_command() {
    let cli = Cli::parse_from(["aisopod", "status", "--detailed"]);
    assert!(matches!(cli.command, Commands::Status(_)));
}

#[test]
fn test_parse_health_command() {
    let cli = Cli::parse_from(["aisopod", "health", "--json"]);
    assert!(matches!(cli.command, Commands::Health(_)));
}

#[test]
fn test_parse_dashboard_command() {
    let cli = Cli::parse_from(["aisopod", "dashboard"]);
    assert!(matches!(cli.command, Commands::Dashboard));
}

#[test]
fn test_parse_models_list() {
    let cli = Cli::parse_from(["aisopod", "models", "list", "--provider", "openai"]);
    assert!(matches!(cli.command, Commands::Models(_)));
}

#[test]
fn test_parse_models_switch() {
    let cli = Cli::parse_from(["aisopod", "models", "switch", "gpt-4"]);
    assert!(matches!(cli.command, Commands::Models(_)));
}

#[test]
fn test_parse_channels_command() {
    let cli = Cli::parse_from(["aisopod", "channels", "list"]);
    assert!(matches!(cli.command, Commands::Channels(_)));
}

#[test]
fn test_parse_sessions_command() {
    let cli = Cli::parse_from(["aisopod", "sessions", "list"]);
    assert!(matches!(cli.command, Commands::Sessions(_)));
}

#[test]
fn test_parse_daemon_command() {
    let cli = Cli::parse_from(["aisopod", "daemon", "start"]);
    assert!(matches!(cli.command, Commands::Daemon(_)));
}

#[test]
fn test_parse_doctor_command() {
    let cli = Cli::parse_from(["aisopod", "doctor"]);
    assert!(matches!(cli.command, Commands::Doctor(_)));
}

#[test]
fn test_parse_auth_command() {
    let cli = Cli::parse_from(["aisopod", "auth", "setup"]);
    assert!(matches!(cli.command, Commands::Auth(_)));
}

#[test]
fn test_parse_reset_command() {
    let cli = Cli::parse_from(["aisopod", "reset"]);
    assert!(matches!(cli.command, Commands::Reset));
}

#[test]
fn test_parse_onboarding_command() {
    let cli = Cli::parse_from(["aisopod", "onboarding", "--config", "/tmp/config.toml"]);
    assert!(matches!(cli.command, Commands::Onboarding { .. }));
}

// ============================================================================
// Integration Tests
// ============================================================================

#[cfg(test)]
mod integration {
    use std::process::Command;

    #[test]
    fn test_help_output() {
        let output = Command::new(env!("CARGO_BIN_EXE_aisopod"))
            .arg("--help")
            .output()
            .expect("Failed to execute");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("gateway"));
        assert!(stdout.contains("agent"));
        assert!(stdout.contains("message"));
        assert!(stdout.contains("config"));
        assert!(stdout.contains("status"));
    }

    #[test]
    fn test_version_output() {
        let output = Command::new(env!("CARGO_BIN_EXE_aisopod"))
            .arg("--version")
            .output()
            .expect("Failed to execute");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("aisopod"));
    }
}
