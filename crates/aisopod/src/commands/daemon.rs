//! Daemon management commands for the aisopod application.
//!
//! This module provides the `aisopod daemon` subcommand family for managing
//! the aisopod background service:
//! - install: Install aisopod as a system service
//! - start: Start the daemon
//! - stop: Stop the daemon
//! - status: Show daemon status
//! - logs: Tail daemon logs

use anyhow::{anyhow, Context, Result};
use clap::{Args, Subcommand};
use std::path::Path;
use std::process::Command;

/// Daemon management command arguments
#[derive(Args)]
pub struct DaemonArgs {
    #[command(subcommand)]
    pub command: DaemonCommands,
}

/// Available daemon subcommands
#[derive(Subcommand)]
pub enum DaemonCommands {
    /// Install aisopod as a system service
    Install,
    /// Start the daemon
    Start,
    /// Stop the daemon
    Stop,
    /// Show daemon status
    Status,
    /// Tail daemon logs
    Logs {
        /// Number of lines to show
        #[arg(long, default_value_t = 50)]
        lines: usize,

        /// Follow log output
        #[arg(long, short)]
        follow: bool,
    },
}

/// Get the current executable path
fn get_exe_path() -> Result<std::path::PathBuf> {
    std::env::current_exe().context("Failed to get current executable path")
}

/// Install aisopod as a system service
pub fn install_daemon() -> Result<()> {
    let exe_path = get_exe_path()?;

    if cfg!(target_os = "linux") {
        install_systemd_service(&exe_path)?;
    } else if cfg!(target_os = "macos") {
        install_launchctl_service(&exe_path)?;
    } else {
        return Err(anyhow!("Daemon installation not supported on this platform"));
    }

    Ok(())
}

/// Install systemd service on Linux
fn install_systemd_service(exe_path: &Path) -> Result<()> {
    let unit = format!(
        r#"[Unit]
Description=aisopod AI Agent Orchestration Platform
After=network.target

[Service]
Type=simple
ExecStart={} gateway
Restart=on-failure
RestartSec=5
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
"#,
        exe_path.display()
    );

    let service_path = "/etc/systemd/system/aisopod.service";
    std::fs::write(service_path, unit)
        .with_context(|| format!("Failed to write systemd service file to {}", service_path))?;

    // Reload systemd daemon
    Command::new("systemctl")
        .args(["daemon-reload"])
        .status()
        .with_context(|| "Failed to reload systemd daemon")?;

    // Enable the service
    Command::new("systemctl")
        .args(["enable", "aisopod"])
        .status()
        .with_context(|| "Failed to enable aisopod service")?;

    println!("Systemd service installed and enabled.");
    println!("Start with: aisopod daemon start");
    Ok(())
}

/// Install launchctl service on macOS
fn install_launchctl_service(exe_path: &Path) -> Result<()> {
    let plist = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.aisopod.daemon</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
        <string>gateway</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/usr/local/var/log/aisopod.log</string>
    <key>StandardErrorPath</key>
    <string>/usr/local/var/log/aisopod.err</string>
</dict>
</plist>
"#,
        exe_path.display()
    );

    let home_dir = dirs::home_dir().ok_or_else(|| anyhow!("Cannot determine home directory"))?;
    let plist_path = home_dir.join("Library/LaunchAgents/com.aisopod.daemon.plist");

    std::fs::write(&plist_path, plist)
        .with_context(|| format!("Failed to write plist file to {}", plist_path.display()))?;

    println!("LaunchAgent plist installed at {}", plist_path.display());
    println!("Start with: aisopod daemon start");
    Ok(())
}

/// Start the daemon service
pub fn start_daemon() -> Result<()> {
    if cfg!(target_os = "linux") {
        Command::new("systemctl")
            .args(["start", "aisopod"])
            .status()
            .with_context(|| "Failed to start aisopod service")?;
    } else if cfg!(target_os = "macos") {
        let plist = plist_path()?;
        Command::new("launchctl")
            .args(["load", &plist])
            .status()
            .with_context(|| "Failed to load launch agent")?;
    } else {
        return Err(anyhow!("Daemon management not supported on this platform"));
    }
    println!("Daemon started.");
    Ok(())
}

/// Stop the daemon service
pub fn stop_daemon() -> Result<()> {
    if cfg!(target_os = "linux") {
        Command::new("systemctl")
            .args(["stop", "aisopod"])
            .status()
            .with_context(|| "Failed to stop aisopod service")?;
    } else if cfg!(target_os = "macos") {
        let plist = plist_path()?;
        Command::new("launchctl")
            .args(["unload", &plist])
            .status()
            .with_context(|| "Failed to unload launch agent")?;
    } else {
        return Err(anyhow!("Daemon management not supported on this platform"));
    }
    println!("Daemon stopped.");
    Ok(())
}

/// Show daemon status
pub fn daemon_status() -> Result<()> {
    if cfg!(target_os = "linux") {
        Command::new("systemctl")
            .args(["status", "aisopod"])
            .status()
            .with_context(|| "Failed to get aisopod service status")?;
    } else if cfg!(target_os = "macos") {
        Command::new("launchctl")
            .args(["list", "com.aisopod.daemon"])
            .status()
            .with_context(|| "Failed to get launch agent status")?;
    } else {
        return Err(anyhow!("Daemon management not supported on this platform"));
    }
    Ok(())
}

/// Tail daemon logs
pub fn tail_logs(lines: usize, follow: bool) -> Result<()> {
    if cfg!(target_os = "linux") {
        let lines_arg = format!("--lines={}", lines);
        let mut args = vec!["--unit=aisopod", &lines_arg];
        if follow {
            args.push("--follow");
        }
        Command::new("journalctl")
            .args(&args)
            .status()
            .with_context(|| "Failed to tail systemd logs")?;
    } else if cfg!(target_os = "macos") {
        let log_path = "/usr/local/var/log/aisopod.log";
        let lines_arg = lines.to_string();
        let mut args = vec!["-n", &lines_arg, log_path];
        if follow {
            args.push("-f");
        }
        Command::new("tail")
            .args(&args)
            .status()
            .with_context(|| "Failed to tail logs")?;
    } else {
        return Err(anyhow!("Daemon management not supported on this platform"));
    }
    Ok(())
}

/// Get the plist path for macOS
fn plist_path() -> Result<String> {
    let home = dirs::home_dir().ok_or_else(|| anyhow!("Cannot determine home directory"))?;
    Ok(home
        .join("Library/LaunchAgents/com.aisopod.daemon.plist")
        .to_string_lossy()
        .to_string())
}

/// Run the daemon command with the given arguments
pub fn run(args: DaemonArgs) -> Result<()> {
    match args.command {
        DaemonCommands::Install => install_daemon(),
        DaemonCommands::Start => start_daemon(),
        DaemonCommands::Stop => stop_daemon(),
        DaemonCommands::Status => daemon_status(),
        DaemonCommands::Logs { lines, follow } => tail_logs(lines, follow),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daemon_args_default() {
        let args = DaemonArgs {
            command: DaemonCommands::Install,
        };

        match args.command {
            DaemonCommands::Install => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn test_daemon_logs_args() {
        let args = DaemonArgs {
            command: DaemonCommands::Logs {
                lines: 100,
                follow: true,
            },
        };

        match args.command {
            DaemonCommands::Logs { lines, follow } => {
                assert_eq!(lines, 100);
                assert!(follow);
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_daemon_logs_default_args() {
        let args = DaemonArgs {
            command: DaemonCommands::Logs {
                lines: 50,
                follow: false,
            },
        };

        match args.command {
            DaemonCommands::Logs { lines, follow } => {
                assert_eq!(lines, 50);
                assert!(!follow);
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_daemon_commands_enum() {
        assert!(matches!(DaemonCommands::Install, DaemonCommands::Install));
        assert!(matches!(DaemonCommands::Start, DaemonCommands::Start));
        assert!(matches!(DaemonCommands::Stop, DaemonCommands::Stop));
        assert!(matches!(DaemonCommands::Status, DaemonCommands::Status));
    }
}
