//! Signal outbound message handling.
//!
//! This module handles sending messages through signal-cli.

use crate::channel::SignalAccount;
use crate::config::{SignalAccountConfig, SignalError};
use aisopod_channel::message::{Media, MessageContent, MessageTarget, PeerKind};
use aisopod_channel::types::MediaType;
use anyhow::Result;
use async_trait::async_trait;
use std::process::Command;
use tracing::{debug, error, info};

/// Outbound message sender for Signal.
#[derive(Clone)]
pub struct SignalOutbound {
    /// Timeout for send operations in seconds
    send_timeout: u64,
}

impl Default for SignalOutbound {
    fn default() -> Self {
        Self::new()
    }
}

impl SignalOutbound {
    /// Create a new SignalOutbound instance.
    pub fn new() -> Self {
        Self { send_timeout: 30 }
    }

    /// Create a new SignalOutbound with custom timeout.
    pub fn with_timeout(timeout: u64) -> Self {
        Self {
            send_timeout: timeout,
        }
    }

    /// Send a text message to the specified target.
    ///
    /// # Arguments
    ///
    /// * `target` - The message target specifying where to send
    /// * `text` - The plain text content to send
    /// * `account` - The Signal account configuration
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message was sent successfully
    /// * `Err(anyhow::Error)` - An error if sending fails
    pub async fn send_text(
        &self,
        target: &MessageTarget,
        text: &str,
        account: &SignalAccountConfig,
    ) -> Result<()> {
        info!("Sending text message to {} via Signal", target.peer.id);

        // Get the phone number for the target
        let phone_number = &target.peer.id;

        // Build the signal-cli send command
        let mut command = Command::new(
            account
                .signal_cli_path
                .as_ref()
                .unwrap_or(&"signal-cli".to_string()),
        );
        command
            .arg("-u")
            .arg(&account.phone_number)
            .arg("send")
            .arg(phone_number);

        // Add group ID if sending to a group
        if target.peer.kind == PeerKind::Group {
            if let Some(group_id) = self.get_group_id_from_target(target) {
                command.arg("-g").arg(&group_id);
            }
        }

        // Create child process and send text via stdin
        let mut child = command
            .spawn()
            .map_err(|e| SignalError::SendCommandFailed(e.to_string()))?;

        // Write text to stdin
        let stdin = child.stdin.as_mut().unwrap();
        std::io::Write::write_all(stdin, text.as_bytes())?;

        // Wait for the command to complete
        let status = child
            .wait()
            .map_err(|e| SignalError::ReceiveResponseFailed(e.to_string()))?;

        if status.success() {
            info!("Successfully sent message to {}", target.peer.id);
            Ok(())
        } else {
            error!("Failed to send message to {}", target.peer.id);
            Err(SignalError::SendCommandFailed("Command failed".to_string()).into())
        }
    }

    /// Send media content to the specified target.
    ///
    /// # Arguments
    ///
    /// * `target` - The message target specifying where to send
    /// * `media` - The media content to send
    /// * `account` - The Signal account configuration
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Media was sent successfully
    /// * `Err(anyhow::Error)` - An error if sending fails
    pub async fn send_media(
        &self,
        target: &MessageTarget,
        media: &Media,
        account: &SignalAccountConfig,
    ) -> Result<()> {
        info!("Sending media to {} via Signal", target.peer.id);

        // Download or get the media file path
        let media_path = self.get_media_path(media).await?;

        // Get the phone number for the target
        let phone_number = &target.peer.id;

        // Build the signal-cli send command with attachment
        let mut command = Command::new(
            account
                .signal_cli_path
                .as_ref()
                .unwrap_or(&"signal-cli".to_string()),
        );
        command
            .arg("-u")
            .arg(&account.phone_number)
            .arg("send")
            .arg(phone_number)
            .arg("--attachment")
            .arg(&media_path);

        // Add group ID if sending to a group
        if target.peer.kind == PeerKind::Group {
            if let Some(group_id) = self.get_group_id_from_target(target) {
                command.arg("-g").arg(&group_id);
            }
        }

        let output = command
            .output()
            .map_err(|e| SignalError::SendCommandFailed(e.to_string()))?;

        if output.status.success() {
            info!("Successfully sent media to {}", target.peer.id);
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("Failed to send media to {}: {}", target.peer.id, stderr);
            Err(SignalError::SendCommandFailed(stderr.to_string()).into())
        }
    }

    /// Send a message with content to the specified target.
    ///
    /// # Arguments
    ///
    /// * `target` - The message target specifying where to send
    /// * `content` - The content to send
    /// * `account` - The Signal account configuration
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message was sent successfully
    /// * `Err(anyhow::Error)` - An error if sending fails
    pub async fn send_content(
        &self,
        target: &MessageTarget,
        content: &MessageContent,
        account: &SignalAccountConfig,
    ) -> Result<()> {
        match content {
            MessageContent::Text(text) => self.send_text(target, text, account).await,
            MessageContent::Media(media) => self.send_media(target, media, account).await,
            MessageContent::Mixed(parts) => {
                // Handle mixed content by sending each part separately
                // Signal doesn't support mixed content in a single message
                for part in parts {
                    match part {
                        aisopod_channel::message::MessagePart::Text(text) => {
                            self.send_text(target, text, account).await?;
                        }
                        aisopod_channel::message::MessagePart::Media(media) => {
                            self.send_media(target, media, account).await?;
                        }
                    }
                }
                Ok(())
            }
        }
    }

    /// Get the group ID from the message target.
    fn get_group_id_from_target(&self, target: &MessageTarget) -> Option<String> {
        // Extract group ID from metadata if available
        // In practice, this would be part of the peer ID or in metadata
        None
    }

    /// Get the file path for media content.
    ///
    /// This method handles media that is either already a file path
    /// or needs to be saved from raw data.
    async fn get_media_path(&self, media: &Media) -> Result<String> {
        // If we have a URL, download the file
        if let Some(url) = &media.url {
            let response = reqwest::get(url)
                .await
                .map_err(|e| SignalError::MediaError(format!("Failed to download media: {}", e)))?;

            let bytes = response.bytes().await.map_err(|e| {
                SignalError::MediaError(format!("Failed to read media bytes: {}", e))
            })?;

            // Create a temporary file to store the media
            let temp_file = tempfile::NamedTempFile::new().map_err(|e| {
                SignalError::MediaError(format!("Failed to create temp file: {}", e))
            })?;

            std::fs::write(&temp_file, &bytes).map_err(|e| {
                SignalError::MediaError(format!("Failed to write temp file: {}", e))
            })?;

            Ok(temp_file.path().to_string_lossy().to_string())
        }
        // If we have raw data, create a temporary file
        else if let Some(data) = &media.data {
            let temp_file = tempfile::NamedTempFile::new().map_err(|e| {
                SignalError::MediaError(format!("Failed to create temp file: {}", e))
            })?;

            std::fs::write(&temp_file, data).map_err(|e| {
                SignalError::MediaError(format!("Failed to write temp file: {}", e))
            })?;

            Ok(temp_file.path().to_string_lossy().to_string())
        }
        // If no URL or data, we can't send
        else {
            Err(SignalError::MediaError("No media URL or data available".to_string()).into())
        }
    }
}

impl SignalOutbound {
    /// Send a message using the provided account config
    pub async fn send(
        &self,
        target: &MessageTarget,
        text: &str,
        account: &SignalAccountConfig,
    ) -> Result<()> {
        self.send_text(target, text, account).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_outbound_new() {
        let outbound = SignalOutbound::new();
        assert_eq!(outbound.send_timeout, 30);
    }

    #[test]
    fn test_signal_outbound_with_timeout() {
        let outbound = SignalOutbound::with_timeout(60);
        assert_eq!(outbound.send_timeout, 60);
    }
}
