//! Twitch Messaging Interface (TMI) client.
//!
//! This module provides a client for connecting to Twitch's IRC-based
//! messaging interface (TMI), handling connection management, message
//! sending and receiving, and parsing Twitch-specific message tags.

use crate::badges::parse_badges;
use anyhow::{anyhow, bail, Result};
use futures::{SinkExt, StreamExt, TryFutureExt};
use serde::Deserialize;
use std::collections::HashMap;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, WebSocketStream, MaybeTlsStream};
use tracing::{debug, error, info, warn};

/// Twitch message received from the server.
#[derive(Debug, Clone)]
pub struct TwitchMessage {
    /// The channel the message was sent to
    pub channel: String,
    /// The username of the sender
    pub username: String,
    /// The text content of the message
    pub text: String,
    /// Twitch message tags (badges, badges_info, etc.)
    pub tags: TwitchTags,
    /// Whether this is a whisper (private message)
    pub is_whisper: bool,
}

/// Twitch message tags containing user metadata.
#[derive(Debug, Clone, Default)]
pub struct TwitchTags {
    /// Display name of the user
    pub display_name: String,
    /// User's badges (moderator, subscriber, etc.)
    pub badges: Vec<String>,
    /// Whether the user is a moderator
    pub is_mod: bool,
    /// Whether the user is a subscriber
    pub is_subscriber: bool,
    /// User ID on Twitch
    pub user_id: String,
    /// Badge information (e.g., "moderator=1")
    pub badge_info: Vec<String>,
}

/// TMI (Twitch Messaging Interface) client.
///
/// This struct manages a connection to Twitch's WebSocket-based IRC interface.
/// It handles connection setup, authentication, channel joining, and message
/// sending/receiving.
pub struct TmiClient {
    /// WebSocket stream to Twitch's TMI server
    websocket: WebSocketStream<MaybeTlsStream<TcpStream>>,
    /// The bot's username
    username: String,
    /// Whether whispers are enabled
    enable_whispers: bool,
}

impl TmiClient {
    /// Connect to Twitch's TMI server with the given credentials.
    ///
    /// # Arguments
    ///
    /// * `username` - The bot's Twitch username
    /// * `oauth_token` - The OAuth token for authentication
    ///
    /// # Returns
    ///
    /// * `Ok(TmiClient)` - The connected client
    /// * `Err(anyhow::Error)` - An error if connection fails
    pub async fn connect(username: &str, oauth_token: &str) -> Result<Self> {
        info!(
            "Connecting to Twitch TMI as {}",
            username
        );

        // Connect to Twitch's WebSocket TMI server
        let (websocket, _) = connect_async("wss://irc-ws.chat.twitch.tv:443").map_err(|e| {
            error!("Failed to connect to Twitch TMI: {}", e);
            anyhow!("Failed to connect to Twitch TMI: {}", e)
        }).await?;

        let mut client = Self {
            websocket,
            username: username.to_string(),
            enable_whispers: false,
        };

        // Send CAP REQ to request additional capabilities
        client
            .send_command("CAP REQ :twitch.tv/membership twitch.tv/tags twitch.tv/commands")
            .await?;

        // Send PASS with OAuth token
        client
            .send_command(&format!("PASS {}", oauth_token))
            .await?;

        // Send NICK with username
        client
            .send_command(&format!("NICK {}", username))
            .await?;

        info!("Connected to Twitch TMI as {}", username);
        Ok(client)
    }

    /// Send a raw IRC command to the server.
    async fn send_command(&mut self, command: &str) -> Result<()> {
        debug!("Sending command: {}", command);
        let message = format!("{}\r\n", command);
        self.websocket
            .send(Message::text(message))
            .await
            .map_err(|e| {
                error!("Failed to send command: {}", e);
                anyhow!("Failed to send command: {}", e)
            })?;
        Ok(())
    }

    /// Join a channel (channel names must include the # prefix).
    ///
    /// # Arguments
    ///
    /// * `channel` - The channel to join (e.g., "#channelname")
    pub async fn join_channel(&mut self, channel: &str) -> Result<()> {
        info!("Joining channel {}", channel);
        self.send_command(&format!("JOIN {}", channel)).await
    }

    /// Leave a channel.
    ///
    /// # Arguments
    ///
    /// * `channel` - The channel to leave
    pub async fn part_channel(&mut self, channel: &str) -> Result<()> {
        info!("Leaving channel {}", channel);
        self.send_command(&format!("PART {}", channel)).await
    }

    /// Send a message to a channel.
    ///
    /// # Arguments
    ///
    /// * `channel` - The target channel (e.g., "#channelname")
    /// * `message` - The message to send
    pub async fn send_message(&mut self, channel: &str, message: &str) -> Result<()> {
        debug!("Sending message to {}: {}", channel, message);
        self.send_command(&format!("PRIVMSG {} :{}", channel, message))
            .await
    }

    /// Send a whisper to another user.
    ///
    /// Whispers are sent as PRIVMSG to the bot's own channel with a special
    /// format: `/w <username> <message>`.
    ///
    /// # Arguments
    ///
    /// * `username` - The recipient's username
    /// * `message` - The whisper message
    pub async fn send_whisper(&mut self, username: &str, message: &str) -> Result<()> {
        info!("Sending whisper to {}: {}", username, message);
        self.send_command(&format!(
            "PRIVMSG #{} :/w {} {}",
            self.username, username, message
        ))
        .await
    }

    /// Read a message from the server.
    ///
    /// This method waits for and parses incoming messages from Twitch.
    /// It handles PING/PONG keepalive automatically.
    ///
    /// # Returns
    ///
    /// * `Ok(TwitchMessage)` - A parsed Twitch message
    /// * `Err(anyhow::Error)` - An error if reading fails
    pub async fn read_message(&mut self) -> Result<TwitchMessage> {
        loop {
            // Read the next message from the websocket
            let msg_result = self
                .websocket
                .next()
                .await
                .ok_or_else(|| anyhow!("Connection closed by server"))?;

            match msg_result {
                Ok(Message::Text(text)) => {
                    debug!("Received: {}", text);
                    // Handle PING with PONG
                    if text.starts_with("PING") {
                        let ping_value = text.trim_start_matches("PING").trim();
                        debug!("Sending PONG for: {}", ping_value);
                        self.send_command(&format!("PONG {}", ping_value))
                            .await?;
                        continue; // Continue reading for actual messages
                    }

                    // Parse the IRC message
                    return self.parse_message(&text);
                }
                Ok(Message::Close(_)) => {
                    error!("WebSocket connection closed");
                    return Err(anyhow!("WebSocket connection closed"));
                }
                Ok(_) => {
                    debug!("Ignoring non-text message");
                    continue;
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    return Err(anyhow!("WebSocket error: {}", e));
                }
            }
        }
    }

    /// Parse an IRC message line into a TwitchMessage.
    fn parse_message(&self, line: &str) -> Result<TwitchMessage> {
        if line.trim().is_empty() {
            return Err(anyhow!("Empty message"));
        }

        let mut channel = String::new();
        let mut username = String::new();
        let mut text = String::new();
        let mut tags = TwitchTags::default();
        let mut is_whisper = false;

        let line = line.trim();

        // Parse tags if present (starts with @)
        if line.starts_with('@') {
            let (tags_part, rest) = line.split_once(' ').ok_or_else(|| {
                error!("Malformed message: {}", line);
                anyhow!("Malformed message: {}", line)
            })?;

            tags = self.parse_tags(tags_part);

            let rest = rest.trim_start();
            if rest.starts_with(':') {
                // Parse the prefix (username)
                let prefix = rest.trim_start_matches(':');
                if let Some((user, rest)) = prefix.split_once('!') {
                    username = user.to_string();
                } else {
                    username = prefix.to_string();
                }

                let rest = rest.trim_start();
                if let Some((cmd, content)) = rest.split_once(' ') {
                    let content = content.trim_start_matches(':');

                    // Check if this is a whisper (PRIVMSG to the bot's channel with /w)
                    if cmd == "PRIVMSG" {
                        if let Some(chan) = content.split_once(' ') {
                            channel = chan.0.to_string();
                            text = chan.1.to_string();
                        } else {
                            channel = content.to_string();
                        }
                    } else if cmd == "PRIVMSG" && content.contains("/w ") {
                        // This is a whisper
                        is_whisper = true;
                        let parts: Vec<&str> = content.split_whitespace().collect();
                        if parts.len() >= 2 {
                            // Extract username and message from whisper format
                            let whisper_rest = content.strip_prefix("PRIVMSG #").unwrap_or(content);
                            let whisper_rest = whisper_rest.strip_prefix(&self.username).unwrap_or(whisper_rest);
                            let whisper_rest = whisper_rest.trim();
                            if whisper_rest.starts_with("/w ") {
                                let whisper_content = whisper_rest.strip_prefix("/w ").unwrap_or(whisper_rest);
                                if let Some((user, msg)) = whisper_content.split_once(' ') {
                                    username = user.to_string();
                                    text = msg.to_string();
                                    channel = format!("#{}", user);
                                }
                            }
                        }
                    }
                }
            }
        } else if line.starts_with(':') {
            // Message without tags
            let rest = line.trim_start_matches(':');
            if let Some((user, rest)) = rest.split_once('!') {
                username = user.to_string();
                let rest = rest.trim_start();
                if let Some((cmd, content)) = rest.split_once(' ') {
                    let content = content.trim_start_matches(':');
                    if cmd == "PRIVMSG" {
                        if let Some(chan) = content.split_once(' ') {
                            channel = chan.0.to_string();
                            text = chan.1.to_string();
                        } else {
                            channel = content.to_string();
                        }
                    }
                }
            }
        }

        Ok(TwitchMessage {
            channel,
            username,
            text,
            tags,
            is_whisper,
        })
    }

    /// Parse IRC message tags.
    fn parse_tags(&self, tags_str: &str) -> TwitchTags {
        let mut tags = TwitchTags::default();

        // Remove the @ prefix if present
        let tags_str = tags_str.trim_start_matches('@');

        for tag in tags_str.split(';') {
            if let Some((key, value)) = tag.split_once('=') {
                match key {
                    "display-name" => tags.display_name = value.to_string(),
                    "badges" => tags.badges = parse_badges(value).iter().map(|b| b.name.clone()).collect(),
                    "badges-info" => tags.badge_info = value.split(',').map(|s| s.to_string()).collect(),
                    "mod" => tags.is_mod = value == "1",
                    "subscriber" => tags.is_subscriber = value == "1",
                    "user-id" => tags.user_id = value.to_string(),
                    _ => {} // Ignore other tags for now
                }
            }
        }

        tags
    }

    /// Enable whisper support.
    pub fn enable_whispers(&mut self) {
        self.enable_whispers = true;
    }

    /// Check if whispers are enabled.
    pub fn whispers_enabled(&self) -> bool {
        self.enable_whispers
    }
}

/// Parse a raw IRC message line into components.
///
/// This is a utility function that can be used for debugging or custom parsing.
///
/// # Arguments
///
/// * `line` - The raw IRC message line
///
/// # Returns
///
/// A tuple of (tags, prefix, command, params) or an error.
pub fn parse_irc_line(line: &str) -> Result<(
    Option<HashMap<String, String>>,
    Option<String>,
    String,
    Vec<String>,
)> {
    let line = line.trim();
    let mut tags: Option<HashMap<String, String>> = None;
    let mut prefix: Option<String> = None;
    let mut command = String::new();
    let mut params: Vec<String> = Vec::new();

    let mut remaining = line;

    // Parse tags if present
    if remaining.starts_with('@') {
        let (tags_str, rest) = remaining.split_once(' ').ok_or_else(|| anyhow!("No space after tags"))?;
        tags = Some(parse_tag_string(tags_str.trim_start_matches('@')));
        remaining = rest.trim_start();
    }

    // Parse prefix if present
    if remaining.starts_with(':') {
        let (pref, rest) = remaining.split_once(' ').ok_or_else(|| anyhow!("No space after prefix"))?;
        prefix = Some(pref.trim_start_matches(':').to_string());
        remaining = rest.trim_start();
    }

    // Parse command
    if let Some((cmd, rest)) = remaining.split_once(' ') {
        command = cmd.to_string();
        remaining = rest.trim_start();
    } else {
        command = remaining.to_string();
        return Ok((tags, prefix, command, params));
    }

    // Parse parameters
    let parts: Vec<&str> = remaining.split(' ').collect();
    let mut colon_found = false;
    for (i, part) in parts.iter().enumerate() {
        if !colon_found && part.starts_with(':') {
            // Last parameter starts with : and contains spaces
            // Get everything after the :
            let param = remaining[remaining.find(':').unwrap() + 1..].to_string();
            params.push(param);
            break;
        } else if i > 0 && parts[i - 1].starts_with(':') {
            // Already got the colon parameter
            break;
        } else {
            params.push(part.to_string());
        }
        if part.starts_with(':') {
            colon_found = true;
        }
    }

    Ok((tags, prefix, command, params))
}

/// Parse a tag string into a map.
fn parse_tag_string(tag_str: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();

    for tag in tag_str.split(';') {
        if let Some((key, value)) = tag.split_once('=') {
            map.insert(key.to_string(), value.to_string());
        }
    }

    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tag_string() {
        let tags = parse_tag_string("display-name=TestBot;mod=1;subscriber=0");
        assert_eq!(tags.get("display-name"), Some(&"TestBot".to_string()));
        assert_eq!(tags.get("mod"), Some(&"1".to_string()));
        assert_eq!(tags.get("subscriber"), Some(&"0".to_string()));
    }

    #[test]
    fn test_parse_irc_line_with_tags() {
        let line = "@badges=moderator/1;display-name=TestBot :testbot!testbot@testbot.tmi.twitch.tv PRIVMSG #channel :Hello world";
        let (tags, prefix, cmd, params) = parse_irc_line(line).unwrap();

        assert!(tags.is_some());
        assert_eq!(prefix, Some("testbot!testbot@testbot.tmi.twitch.tv".to_string()));
        assert_eq!(cmd, "PRIVMSG");
        assert_eq!(params, vec!["#channel", "Hello world"]);
    }

    #[test]
    fn test_parse_irc_line_without_tags() {
        let line = ":testbot!testbot@testbot.tmi.twitch.tv PRIVMSG #channel :Hello world";
        let (tags, prefix, cmd, params) = parse_irc_line(line).unwrap();

        assert!(tags.is_none());
        assert_eq!(prefix, Some("testbot!testbot@testbot.tmi.twitch.tv".to_string()));
        assert_eq!(cmd, "PRIVMSG");
        assert_eq!(params, vec!["#channel", "Hello world"]);
    }

    #[test]
    fn test_parse_irc_line_with_colon_param() {
        let line = "@badges=moderator/1 :testbot!testbot@testbot.tmi.twitch.tv PRIVMSG #channel :Hello world with spaces";
        let (tags, prefix, cmd, params) = parse_irc_line(line).unwrap();

        assert!(tags.is_some());
        assert_eq!(cmd, "PRIVMSG");
        assert_eq!(params, vec!["#channel", "Hello world with spaces"]);
    }
}
