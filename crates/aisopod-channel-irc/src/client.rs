//! IRC client wrapper for managing connections to IRC servers.
//!
//! This module provides a wrapper around the `irc` crate's Client type,
//! handling connection setup and basic message sending functionality.

use crate::config::IrcServerConfig;
use anyhow::Result;
use futures::StreamExt;
use irc::client::prelude::*;
use std::time::Duration;
use tracing::{error, info};

/// A wrapper around the IRC client connection.
///
/// This struct manages a single IRC server connection, handling
/// connection setup, authentication, and message sending.
pub struct IrcConnection {
    /// The underlying IRC client
    client: Client,
    /// Server hostname for identification
    server_name: String,
}

impl IrcConnection {
    /// Create a new IRC connection to the specified server.
    ///
    /// This method establishes a connection to the IRC server with the
    /// provided configuration, including TLS support and NickServ
    /// authentication if configured.
    ///
    /// # Arguments
    ///
    /// * `config` - The IRC server configuration
    ///
    /// # Returns
    ///
    /// * `Ok(IrcConnection)` - The connection if successful
    /// * `Err(anyhow::Error)` - An error if connection fails
    pub async fn connect(config: &IrcServerConfig) -> Result<Self> {
        info!(
            "Connecting to IRC server {}:{} with TLS={}",
            config.server, config.port, config.use_tls
        );

        // Build IRC client configuration
        let irc_config = Config {
            nickname: Some(config.nickname.clone()),
            server: Some(config.server.clone()),
            port: Some(config.port),
            use_tls: Some(config.use_tls),
            channels: config.channels.clone(),
            password: config.server_password.clone(),
            // Use nick_password for NickServ authentication
            nick_password: config.nickserv_password.clone(),
            // Set ping times to keep connection alive
            ping_time: Some(30u32),
            ping_timeout: Some(60u32),
            ..Config::default()
        };

        // Create the client from configuration
        let client = Client::from_config(irc_config).await.map_err(|e| {
            error!("Failed to create IRC client: {}", e);
            anyhow::anyhow!("Failed to create IRC client: {}", e)
        })?;

        // Identify with the server
        client.identify().map_err(|e| {
            error!("Failed to identify with IRC server: {}", e);
            anyhow::anyhow!("Failed to identify: {}", e)
        })?;

        info!(
            "Connected to {}:{} as {}",
            config.server, config.port, config.nickname
        );

        Ok(Self {
            client,
            server_name: config.server.clone(),
        })
    }

    /// Send a private message to a target.
    ///
    /// This method sends a PRIVMSG to the specified target (channel or user).
    ///
    /// # Arguments
    ///
    /// * `target` - The target channel or user nickname
    /// * `message` - The message content to send
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message was sent successfully
    /// * `Err(anyhow::Error)` - An error if sending fails
    pub fn send_privmsg(&self, target: &str, message: &str) -> Result<()> {
        info!(
            "Sending PRIVMSG to {} on {}: {}",
            target, self.server_name, message
        );
        self.client.send_privmsg(target, message).map_err(|e| {
            error!("Failed to send PRIVMSG to {}: {}", target, e);
            anyhow::anyhow!("Failed to send PRIVMSG to {}: {}", target, e)
        })?;
        Ok(())
    }

    /// Join a channel.
    ///
    /// # Arguments
    ///
    /// * `channel` - The channel name to join (e.g., "#channel")
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Channel was joined successfully
    /// * `Err(anyhow::Error)` - An error if joining fails
    pub fn join_channel(&self, channel: &str) -> Result<()> {
        info!("Joining channel {} on {}", channel, self.server_name);
        self.client.send_join(channel).map_err(|e| {
            error!("Failed to join channel {}: {}", channel, e);
            anyhow::anyhow!("Failed to join channel {}: {}", channel, e)
        })?;
        Ok(())
    }

    /// Get a stream of IRC events from the server.
    ///
    /// This returns the underlying client stream that can be used to
    /// receive incoming messages and events from the IRC server.
    ///
    /// # Returns
    ///
    /// An IRC client stream that yields events.
    pub fn stream(&mut self) -> Result<irc::client::ClientStream, irc::error::Error> {
        self.client.stream()
    }

    /// Get the nickname used for this connection.
    pub fn nickname(&self) -> &str {
        self.client.current_nickname()
    }

    /// Get the server name for this connection.
    pub fn server_name(&self) -> &str {
        &self.server_name
    }

    /// Quit the IRC server with an optional reason.
    ///
    /// # Arguments
    ///
    /// * `reason` - Optional quit reason
    pub fn quit(&self, reason: Option<&str>) -> Result<()> {
        info!("Quitting IRC server {}: {:?}", self.server_name, reason);
        self.client
            .send_quit(reason.unwrap_or("Goodbye"))
            .map_err(|e| {
                error!("Failed to quit {}: {}", self.server_name, e);
                anyhow::anyhow!("Failed to quit {}: {}", self.server_name, e)
            })?;
        Ok(())
    }

    /// Get a reference to the underlying client.
    pub fn client(&self) -> &Client {
        &self.client
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_irc_connection_struct() {
        // This is a basic compilation test - full integration tests would need a mock server
        // The struct should compile and have the expected fields
        let _config = IrcServerConfig {
            server: "irc.example.com".to_string(),
            port: 6697,
            use_tls: true,
            nickname: "testbot".to_string(),
            nickserv_password: None,
            channels: vec!["#test".to_string()],
            server_password: None,
        };

        // Verify the config has the expected fields
        assert_eq!(_config.server, "irc.example.com");
        assert_eq!(_config.port, 6697);
        assert!(config_use_tls(&IrcServerConfig::default()));
    }

    fn config_use_tls(config: &IrcServerConfig) -> bool {
        config.use_tls
    }
}
