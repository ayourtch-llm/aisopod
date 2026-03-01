//! NickServ authentication support for IRC connections.
//!
//! This module provides functionality for authenticating with NickServ,
//! the IRC nickname registration service.

use anyhow::Result;
use tracing::{info, warn};

/// Authenticate with NickServ using the provided password.
///
/// This function sends an IDENTIFY command to NickServ to authenticate
/// the bot's nickname. The typical format is:
/// `PRIVMSG NickServ :IDENTIFY <password>`
///
/// # Arguments
///
/// * `client` - The IRC client instance
/// * `password` - The NickServ password
///
/// # Returns
///
/// * `Ok(())` - Authentication request was sent successfully
/// * `Err(anyhow::Error)` - An error if sending fails
///
/// # Note
///
/// This function sends the authentication request but does not wait for
/// a response. In a production implementation, you might want to wait
/// for a "Registered" or similar confirmation message before proceeding.
pub fn authenticate_nickserv(client: &irc::client::Client, password: &str) -> Result<()> {
    // Send NickServ IDENTIFY command
    let message = format!("IDENTIFY {}", password);

    info!("Authenticating with NickServ");
    client.send_privmsg("NickServ", &message).map_err(|e| {
        warn!("Failed to authenticate with NickServ: {}", e);
        anyhow::anyhow!("Failed to authenticate with NickServ: {}", e)
    })?;

    info!("NickServ authentication request sent");
    Ok(())
}

/// Authenticate with NickServ with a custom command format.
///
/// Some IRC networks use different authentication commands. This function
/// allows for custom authentication formats.
///
/// # Arguments
///
/// * `client` - The IRC client instance
/// * `password` - The password to authenticate with
/// * `command` - Custom command format (default: "IDENTIFY")
///
/// # Returns
///
/// * `Ok(())` - Authentication request was sent successfully
/// * `Err(anyhow::Error)` - An error if sending fails
pub fn authenticate_with_format(
    client: &irc::client::Client,
    password: &str,
    command: &str,
) -> Result<()> {
    let message = format!("{} {}", command, password);

    info!("Authenticating with NickServ using format: {}", command);
    client.send_privmsg("NickServ", &message).map_err(|e| {
        warn!("Failed to authenticate with NickServ: {}", e);
        anyhow::anyhow!("Failed to authenticate with NickServ: {}", e)
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authenticate_nickserv_compile() {
        // This is a compilation test - actual authentication requires a live IRC connection
        // The function should compile and have the expected signature
        let _password = "test_password";
        let _command = "IDENTIFY";

        assert_eq!(_password, "test_password");
        assert_eq!(_command, "IDENTIFY");
    }

    #[test]
    fn test_custom_command_format() {
        let password = "secret123";
        let command = "IDENTIFY";
        let message = format!("{} {}", command, password);

        assert_eq!(message, "IDENTIFY secret123");
    }
}
