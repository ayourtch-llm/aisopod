//! AppleScript backend implementation for iMessage channel.
//!
//! This module provides communication with macOS Messages.app using AppleScript
//! via the osascript binary. This is the native macOS approach for iMessage access.

use crate::config::{AppleScriptConfig, ImessageError, ImessageResult};
use crate::platform::{check_platform_support, is_macos};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::time::Duration;

/// Result of an AppleScript operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleScriptResult {
    /// Whether the operation succeeded
    pub success: bool,
    /// Message identifier if applicable
    pub message_id: Option<String>,
    /// Error message if applicable
    pub error: Option<String>,
    /// Raw AppleScript output
    pub raw_output: String,
}

/// AppleScript backend for iMessage.
///
/// This struct handles communication with macOS Messages.app using AppleScript.
/// It uses the `osascript` binary to execute AppleScript code that interacts
/// with the Messages application.
///
/// # Requirements
///
/// - macOS operating system
/// - osascript binary (typically at /usr/bin/osascript)
/// - Messages.app must be installed
///
/// # Example
///
/// ```no_run
/// use aisopod_channel_imessage::ApplescriptBackend;
///
/// async fn example() -> Result<(), anyhow::Error> {
///     let mut backend = ApplescriptBackend::new();
///     backend.connect().await?;
///     backend.send_text("+1234567890", "Hello from AppleScript!").await?;
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct ApplescriptBackend {
    /// Configuration for the AppleScript backend
    config: AppleScriptConfig,
    /// Whether the backend is connected
    connected: bool,
}

impl ApplescriptBackend {
    /// Creates a new AppleScript backend with the default configuration.
    pub fn new() -> Self {
        Self {
            config: AppleScriptConfig::default(),
            connected: false,
        }
    }

    /// Creates a new AppleScript backend with custom configuration.
    pub fn with_config(config: AppleScriptConfig) -> Self {
        Self {
            config,
            connected: false,
        }
    }

    /// Executes an AppleScript and returns the result.
    ///
    /// # Arguments
    /// * `script` - The AppleScript code to execute
    ///
    /// # Returns
    /// * `Ok(AppleScriptResult)` - The result of the script execution
    /// * `Err(ImessageError)` - An error if execution fails
    pub fn execute_script(&self, script: &str) -> ImessageResult<AppleScriptResult> {
        let osascript_path = self
            .config
            .osascript_path
            .as_deref()
            .unwrap_or("/usr/bin/osascript");

        let output = Command::new(osascript_path)
            .arg("-e")
            .arg(script)
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Check if the command succeeded
        let success = output.status.success();

        Ok(AppleScriptResult {
            success,
            message_id: None,
            error: if !stderr.is_empty() {
                Some(stderr.to_string())
            } else {
                None
            },
            raw_output: stdout.to_string(),
        })
    }

    /// Creates AppleScript code to send a message.
    ///
    /// # Arguments
    /// * `to` - The recipient's phone number or email address
    /// * `text` - The message text
    /// * `group_id` - Optional group chat identifier (for group messages)
    ///
    /// # Returns
    /// AppleScript code as a string
    pub fn create_send_script(&self, to: &str, text: &str, group_id: Option<&str>) -> String {
        // Escape special characters in strings
        let escaped_to = to.replace('\\', "\\\\").replace('"', "\\\"");
        let escaped_text = text.replace('\\', "\\\\").replace('"', "\\\"");

        // Escape line breaks
        let escaped_text = escaped_text.replace('\n', "\\n");

        if let Some(group_id) = group_id {
            // Group message script
            format!(
                r#"
                tell application "Messages"
                    set targetChat to chat id "{}"
                    send "{}" to targetChat
                end tell
                "#,
                group_id, escaped_text
            )
        } else {
            // DM script
            format!(
                r#"
                tell application "Messages"
                    set targetService to 1st service whose service type = kSMS
                    set targetAccount to account id "{}"
                    tell targetAccount
                        set targetChat to make new chat with properties {{destination: "{}", service: targetService}}
                        send "{}" to targetChat
                    end tell
                end tell
                "#,
                escaped_to, escaped_to, escaped_text
            )
        }
    }

    /// Creates AppleScript code to send a message with media.
    ///
    /// # Arguments
    /// * `to` - The recipient's phone number or email address
    /// * `media_path` - Path to the media file
    /// * `group_id` - Optional group chat identifier
    ///
    /// # Returns
    /// AppleScript code as a string
    pub fn create_send_media_script(
        &self,
        to: &str,
        media_path: &str,
        group_id: Option<&str>,
    ) -> String {
        let escaped_to = to.replace('\\', "\\\\").replace('"', "\\\"");
        let escaped_media = media_path.replace('\\', "\\\\").replace('"', "\\\"");

        if let Some(group_id) = group_id {
            format!(
                r#"
                tell application "Messages"
                    set targetChat to chat id "{}"
                    set targetAttachment to POSIX file "{}"
                    send targetAttachment to targetChat
                end tell
                "#,
                group_id, escaped_media
            )
        } else {
            format!(
                r#"
                tell application "Messages"
                    set targetService to 1st service whose service type = kSMS
                    set targetAccount to account id "{}"
                    tell targetAccount
                        set targetChat to make new chat with properties {{destination: "{}", service: targetService}}
                        set targetAttachment to POSIX file "{}"
                        send targetAttachment to targetChat
                    end tell
                end tell
                "#,
                escaped_to, escaped_to, escaped_media
            )
        }
    }

    /// Creates AppleScript code to retrieve recent messages.
    ///
    /// # Arguments
    /// * `limit` - Maximum number of messages to retrieve
    ///
    /// # Returns
    /// AppleScript code as a string
    pub fn create_get_messages_script(&self, limit: usize) -> String {
        format!(
            r#"
            tell application "Messages"
                set allMessages to messages of every chat
                set msgList to {{}}
                repeat with i from 1 to (count of allMessages) by 1
                    if (count of msgList) >= {} then exit repeat
                    set end of msgList to text of item i of allMessages
                end repeat
                return msgList as string
            end tell
            "#,
            limit
        )
    }

    /// Creates AppleScript code to get group chats.
    ///
    /// # Returns
    /// AppleScript code as a string
    pub fn create_get_group_chats_script(&self) -> String {
        r#"
        tell application "Messages"
            set groupList to {}
            repeat with aChat in chats
                if (class of aChat is chat) then
                    set end of groupList to {name: name of aChat, id: id of aChat}
                end if
            end repeat
            return groupList as JSON
        end tell
        "#
        .to_string()
    }

    /// Creates AppleScript code to get sender information.
    ///
    /// # Arguments
    /// * `chat_id` - The chat identifier
    ///
    /// # Returns
    /// AppleScript code as a string
    pub fn create_get_sender_script(&self, chat_id: &str) -> String {
        let escaped_id = chat_id.replace('\\', "\\\\").replace('"', "\\\"");

        format!(
            r#"
            tell application "Messages"
                set targetChat to chat id "{}"
                return sender of first message of targetChat
            end tell
            "#,
            escaped_id
        )
    }

    /// Checks if Messages.app is running.
    ///
    /// # Returns
    /// * `Ok(true)` - Messages.app is running
    /// * `Ok(false)` - Messages.app is not running
    /// * `Err(ImessageError)` - An error occurred
    pub fn is_messages_running(&self) -> ImessageResult<bool> {
        let script = r#"
            tell application "System Events"
                return name of every process contains "Messages"
            end tell
        "#;

        match self.execute_script(script) {
            Ok(result) => {
                let output = result.raw_output.trim().to_lowercase();
                Ok(output == "true" || output == "yes")
            }
            Err(e) => Err(e),
        }
    }

    /// Activates Messages.app if not already running.
    ///
    /// # Returns
    /// * `Ok(())` - Messages.app was activated or was already running
    /// * `Err(ImessageError)` - An error occurred
    pub fn ensure_messages_running(&self) -> ImessageResult<()> {
        if !self.is_messages_running()? {
            let script = r#"
                tell application "Messages"
                    activate
                end tell
            "#;
            self.execute_script(script)?;
        }
        Ok(())
    }
}

impl Default for ApplescriptBackend {
    fn default() -> Self {
        Self::new()
    }
}

/// AppleScript-based backend implementation.
#[async_trait]
pub trait AppleScriptBackendImpl: Send + Sync {
    /// Connect to Messages.app
    async fn connect(&mut self) -> ImessageResult<()>;

    /// Disconnect from Messages.app
    async fn disconnect(&mut self) -> ImessageResult<()>;

    /// Check if connected
    fn is_connected(&self) -> bool;

    /// Send a text message
    async fn send_text(&self, to: &str, text: &str) -> ImessageResult<String>;

    /// Send a text message to a group
    async fn send_text_to_group(&self, group_id: &str, text: &str) -> ImessageResult<String>;

    /// Send media
    async fn send_media(
        &self,
        to: &str,
        media_path: &str,
        mime_type: &str,
    ) -> ImessageResult<String>;

    /// Send media to a group
    async fn send_media_to_group(
        &self,
        group_id: &str,
        media_path: &str,
        mime_type: &str,
    ) -> ImessageResult<String>;
}

#[async_trait]
impl AppleScriptBackendImpl for ApplescriptBackend {
    async fn connect(&mut self) -> ImessageResult<()> {
        // Check platform support
        check_platform_support(&crate::config::ImessageAccountConfig::default())?;

        // Ensure Messages.app is running
        self.ensure_messages_running()?;

        self.connected = true;
        Ok(())
    }

    async fn disconnect(&mut self) -> ImessageResult<()> {
        self.connected = false;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    async fn send_text(&self, to: &str, text: &str) -> ImessageResult<String> {
        if !self.connected {
            return Err(ImessageError::AppleScript(
                "Not connected to Messages.app".to_string(),
            ));
        }

        let script = self.create_send_script(to, text, None);
        let result = self.execute_script(&script)?;

        if result.success {
            // Extract message ID if available
            if let Some(id) = self.extract_message_id(&result.raw_output) {
                Ok(id)
            } else {
                Ok("sent".to_string())
            }
        } else {
            Err(ImessageError::AppleScript(
                result.error.unwrap_or_else(|| "Unknown error".to_string()),
            ))
        }
    }

    async fn send_text_to_group(&self, group_id: &str, text: &str) -> ImessageResult<String> {
        if !self.connected {
            return Err(ImessageError::AppleScript(
                "Not connected to Messages.app".to_string(),
            ));
        }

        let script = self.create_send_script(group_id, text, Some(group_id));
        let result = self.execute_script(&script)?;

        if result.success {
            Ok("sent".to_string())
        } else {
            Err(ImessageError::AppleScript(
                result.error.unwrap_or_else(|| "Unknown error".to_string()),
            ))
        }
    }

    async fn send_media(
        &self,
        to: &str,
        media_path: &str,
        mime_type: &str,
    ) -> ImessageResult<String> {
        if !self.connected {
            return Err(ImessageError::AppleScript(
                "Not connected to Messages.app".to_string(),
            ));
        }

        // Verify media file exists
        if !std::path::Path::new(media_path).exists() {
            return Err(ImessageError::MediaError(format!(
                "Media file not found: {}",
                media_path
            )));
        }

        let script = self.create_send_media_script(to, media_path, None);
        let result = self.execute_script(&script)?;

        if result.success {
            Ok("sent".to_string())
        } else {
            Err(ImessageError::AppleScript(
                result.error.unwrap_or_else(|| "Unknown error".to_string()),
            ))
        }
    }

    async fn send_media_to_group(
        &self,
        group_id: &str,
        media_path: &str,
        mime_type: &str,
    ) -> ImessageResult<String> {
        if !self.connected {
            return Err(ImessageError::AppleScript(
                "Not connected to Messages.app".to_string(),
            ));
        }

        // Verify media file exists
        if !std::path::Path::new(media_path).exists() {
            return Err(ImessageError::MediaError(format!(
                "Media file not found: {}",
                media_path
            )));
        }

        let script = self.create_send_media_script(group_id, media_path, Some(group_id));
        let result = self.execute_script(&script)?;

        if result.success {
            Ok("sent".to_string())
        } else {
            Err(ImessageError::AppleScript(
                result.error.unwrap_or_else(|| "Unknown error".to_string()),
            ))
        }
    }
}

impl ApplescriptBackend {
    /// Public async method for connect (delegates to AppleScriptBackendImpl)
    pub async fn connect(&mut self) -> ImessageResult<()> {
        <Self as AppleScriptBackendImpl>::connect(self).await
    }

    /// Public async method for disconnect (delegates to AppleScriptBackendImpl)
    pub async fn disconnect(&mut self) -> ImessageResult<()> {
        <Self as AppleScriptBackendImpl>::disconnect(self).await
    }

    /// Public async method for send_text (delegates to AppleScriptBackendImpl)
    pub async fn send_text(&self, to: &str, text: &str) -> ImessageResult<String> {
        <Self as AppleScriptBackendImpl>::send_text(self, to, text).await
    }

    /// Public async method for send_text_to_group (delegates to AppleScriptBackendImpl)
    pub async fn send_text_to_group(&self, group_id: &str, text: &str) -> ImessageResult<String> {
        <Self as AppleScriptBackendImpl>::send_text_to_group(self, group_id, text).await
    }

    /// Public async method for send_media (delegates to AppleScriptBackendImpl)
    pub async fn send_media(
        &self,
        to: &str,
        media_path: &str,
        mime_type: &str,
    ) -> ImessageResult<String> {
        <Self as AppleScriptBackendImpl>::send_media(self, to, media_path, mime_type).await
    }

    /// Public async method for send_media_to_group (delegates to AppleScriptBackendImpl)
    pub async fn send_media_to_group(
        &self,
        group_id: &str,
        media_path: &str,
        mime_type: &str,
    ) -> ImessageResult<String> {
        <Self as AppleScriptBackendImpl>::send_media_to_group(self, group_id, media_path, mime_type)
            .await
    }

    /// Extracts a message ID from the AppleScript output.
    ///
    /// This is a best-effort extraction and may not always succeed.
    fn extract_message_id(&self, output: &str) -> Option<String> {
        // AppleScript output may contain message IDs in various formats
        // Try to extract common patterns
        let output = output.trim();

        // Look for UUID-like patterns
        let pattern = r"[0-9A-F]{8}-[0-9A-F]{4}-[0-9A-F]{4}-[0-9A-F]{4}-[0-9A-F]{12}";
        if let Ok(re) = regex::Regex::new(pattern) {
            if let Some(caps) = re.find(output) {
                return Some(caps.as_str().to_string());
            }
        } else if !output.is_empty() {
            return Some(output.to_string());
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_send_script_dm() {
        let backend = ApplescriptBackend::new();
        let script = backend.create_send_script("+1234567890", "Hello", None);

        assert!(script.contains("tell application \"Messages\""));
        assert!(script.contains("+1234567890"));
        assert!(script.contains("Hello"));
    }

    #[test]
    fn test_create_send_script_group() {
        let backend = ApplescriptBackend::new();
        let script = backend.create_send_script("chat123", "Hello", Some("chat123"));

        assert!(script.contains("tell application \"Messages\""));
        assert!(script.contains("chat id \"chat123\""));
        assert!(script.contains("Hello"));
    }

    #[test]
    fn test_create_send_media_script() {
        let backend = ApplescriptBackend::new();
        let script = backend.create_send_media_script("+1234567890", "/path/to/image.jpg", None);

        assert!(script.contains("POSIX file"));
        assert!(script.contains("/path/to/image.jpg"));
    }

    #[test]
    fn test_create_get_group_chats_script() {
        let backend = ApplescriptBackend::new();
        let script = backend.create_get_group_chats_script();

        assert!(script.contains("tell application \"Messages\""));
        assert!(script.contains("chats"));
    }

    #[test]
    fn test_default_backend() {
        let backend = ApplescriptBackend::new();

        assert!(!backend.is_connected());
    }

    #[test]
    fn test_with_config() {
        let config = AppleScriptConfig {
            timeout_seconds: 60,
            verbose: true,
            ..Default::default()
        };

        let backend = ApplescriptBackend::with_config(config);

        assert_eq!(backend.config.timeout_seconds, 60);
        assert!(backend.config.verbose);
    }

    #[test]
    fn test_is_macos_detection() {
        // This test verifies the platform detection compiles
        let result = is_macos();

        #[cfg(target_os = "macos")]
        assert!(result, "Should be macOS");

        #[cfg(not(target_os = "macos"))]
        assert!(!result, "Should not be macOS");
    }
}
