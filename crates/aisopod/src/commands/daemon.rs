//! Daemon management commands for the aisopod application.
//!
//! This module provides the `aisopod daemon` subcommand family for managing
//! the aisopod background service:
//! - install: Install aisopod as a system service
//! - uninstall: Remove aisopod system service
//! - start: Start the daemon
//! - stop: Stop the daemon
//! - status: Show daemon status
//! - logs: Tail daemon logs

use anyhow::{anyhow, Context, Result};
use clap::{Args, Subcommand};
use std::path::Path;
use std::path::PathBuf;
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
    Install(InstallArgs),
    /// Uninstall aisopod system service
    Uninstall,
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

/// Install command arguments
#[derive(Args)]
pub struct InstallArgs {
    /// Install at system level (requires sudo)
    #[arg(long)]
    pub system: bool,
}

/// Get the current executable path
fn get_exe_path() -> Result<std::path::PathBuf> {
    std::env::current_exe().context("Failed to get current executable path")
}

/// Install aisopod as a system service
pub fn install_daemon(args: InstallArgs) -> Result<()> {
    let exe_path = get_exe_path()?;

    if cfg!(target_os = "linux") {
        install_systemd_service(&exe_path, args.system)?;
    } else if cfg!(target_os = "macos") {
        install_launchctl_service(&exe_path)?;
    } else {
        return Err(anyhow!("Daemon installation not supported on this platform"));
    }

    Ok(())
}

/// Install systemd service on Linux
fn install_systemd_service(exe_path: &Path, system_level: bool) -> Result<()> {
    let unit = generate_systemd_unit(exe_path, system_level);

    let service_path = if system_level {
        PathBuf::from("/etc/systemd/system/aisopod.service")
    } else {
        let home = std::env::var("HOME")
            .with_context(|| "Cannot determine home directory")?;
        let dir = PathBuf::from(&home).join(".config/systemd/user");
        std::fs::create_dir_all(&dir)?;
        dir.join("aisopod.service")
    };

    std::fs::write(&service_path, &unit)
        .with_context(|| format!("Failed to write systemd service file to {}", service_path.display()))?;

    // Reload systemd daemon
    let daemon_reload_args = if system_level { &["daemon-reload"][..] } else { &["--user", "daemon-reload"][..] };
    Command::new("systemctl")
        .args(daemon_reload_args)
        .status()
        .with_context(|| "Failed to reload systemd daemon")?;

    // Enable the service
    let enable_args = if system_level {
        &["enable", "aisopod"][..]
    } else {
        &["--user", "enable", "aisopod"][..]
    };
    Command::new("systemctl")
        .args(enable_args)
        .status()
        .with_context(|| "Failed to enable aisopod service")?;

    let install_msg = if system_level {
        "Systemd service"
    } else {
        "User-level systemd service"
    };
    println!("{} installed and enabled at {}", install_msg, service_path.display());
    println!("Start with: aisopod daemon start");
    Ok(())
}

/// Generate systemd unit file content
fn generate_systemd_unit(exe_path: &Path, system_level: bool) -> String {
    let user_line = if system_level {
        "User=aisopod\n"
    } else {
        ""
    };

    let wanted_by = if system_level {
        "multi-user.target"
    } else {
        "default.target"
    };

    format!(
        r#"[Unit]
Description=Aisopod AI Gateway
After=network.target

[Service]
Type=simple
{user_line}ExecStart={binary_path} gateway
Restart=on-failure
RestartSec=5
Environment=AISOPOD_CONFIG=/etc/aisopod/config.json

[Install]
WantedBy={wanted_by}
"#,
        user_line = user_line,
        binary_path = exe_path.display(),
        wanted_by = wanted_by
    )
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

/// Get the systemd service path based on level
fn get_systemd_service_path(system_level: bool) -> Result<PathBuf> {
    if system_level {
        Ok(PathBuf::from("/etc/systemd/system/aisopod.service"))
    } else {
        let home = std::env::var("HOME")
            .with_context(|| "Cannot determine home directory")?;
        Ok(PathBuf::from(&home).join(".config/systemd/user/aisopod.service"))
    }
}

/// Uninstall aisopod system service
pub fn uninstall_daemon() -> Result<()> {
    if cfg!(target_os = "linux") {
        // Try to detect whether the service was installed at user or system level
        // by checking both locations
        let system_path = PathBuf::from("/etc/systemd/system/aisopod.service");
        let user_home = std::env::var("HOME")
            .with_context(|| "Cannot determine home directory")?;
        let user_path = PathBuf::from(&user_home).join(".config/systemd/user/aisopod.service");

        let (service_path, system_level) = if system_path.exists() {
            (system_path, true)
        } else if user_path.exists() {
            (user_path, false)
        } else {
            return Err(anyhow!(
                "aisopod service is not installed. No service file found."
            ));
        };

        std::fs::remove_file(&service_path)
            .with_context(|| format!("Failed to remove service file {}", service_path.display()))?;

        // Disable the service
        let disable_args = if system_level {
            &["disable", "aisopod"][..]
        } else {
            &["--user", "disable", "aisopod"][..]
        };
        Command::new("systemctl")
            .args(disable_args)
            .status()
            .with_context(|| "Failed to disable aisopod service")?;

        // Reload systemd daemon
        let daemon_reload_args = if system_level {
            &["daemon-reload"][..]
        } else {
            &["--user", "daemon-reload"][..]
        };
        Command::new("systemctl")
            .args(daemon_reload_args)
            .status()
            .with_context(|| "Failed to reload systemd daemon")?;

        let uninstall_msg = if system_level {
            "Systemd service"
        } else {
            "User-level systemd service"
        };
        println!("{} uninstalled from {}", uninstall_msg, service_path.display());
        Ok(())
    } else if cfg!(target_os = "macos") {
        let plist = plist_path()?;
        let plist_path_obj = PathBuf::from(&plist);
        
        if !plist_path_obj.exists() {
            return Err(anyhow!(
                "aisopod service is not installed. No plist file found at {}",
                plist
            ));
        }

        std::fs::remove_file(&plist_path_obj)
            .with_context(|| format!("Failed to remove plist file {}", plist))?;

        Command::new("launchctl")
            .args(["unload", &plist])
            .status()
            .with_context(|| "Failed to unload launch agent")?;

        println!("LaunchAgent uninstalled from {}", plist);
        Ok(())
    } else {
        Err(anyhow!("Daemon uninstallation not supported on this platform"))
    }
}

/// Run the daemon command with the given arguments
pub fn run(args: DaemonArgs) -> Result<()> {
    match args.command {
        DaemonCommands::Install(install_args) => install_daemon(install_args),
        DaemonCommands::Uninstall => uninstall_daemon(),
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
            command: DaemonCommands::Install(InstallArgs { system: false }),
        };

        match args.command {
            DaemonCommands::Install(install_args) => {
                assert!(!install_args.system);
            }
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
        assert!(matches!(DaemonCommands::Install(InstallArgs { system: false }), DaemonCommands::Install(_)));
        assert!(matches!(DaemonCommands::Start, DaemonCommands::Start));
        assert!(matches!(DaemonCommands::Stop, DaemonCommands::Stop));
        assert!(matches!(DaemonCommands::Status, DaemonCommands::Status));
    }

    #[test]
    fn test_daemon_uninstall_command() {
        assert!(matches!(DaemonCommands::Uninstall, DaemonCommands::Uninstall));
    }

    #[test]
    fn test_install_args_default() {
        let args = InstallArgs { system: false };
        assert!(!args.system);
    }

    #[test]
    fn test_install_args_system() {
        let args = InstallArgs { system: true };
        assert!(args.system);
    }
}
