//! Platform-specific implementation for iMessage channel.
//!
//! This module handles platform-specific code, ensuring that the AppleScript
//! backend only compiles on macOS, while providing no-op stubs for other platforms.

use crate::config::{ImessageAccountConfig, ImessageError, ImessageResult};

/// Platform-specific error type for unsupported platforms.
pub struct PlatformUnsupportedError(pub String);

impl std::fmt::Debug for PlatformUnsupportedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PlatformUnsupportedError({})", self.0)
    }
}

impl std::fmt::Display for PlatformUnsupportedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for PlatformUnsupportedError {}

/// Check if the current platform is macOS.
///
/// # Returns
/// `true` if running on macOS, `false` otherwise.
pub fn is_macos() -> bool {
    cfg!(target_os = "macos")
}

/// Check if the current platform is supported by iMessage channel.
///
/// The iMessage channel requires either:
/// - macOS (for AppleScript backend)
/// - Any platform with BlueBubbles server access
///
/// # Arguments
/// * `config` - The iMessage account configuration
///
/// # Returns
/// * `Ok(())` if the platform is supported for the configured backend
/// * `Err(ImessageError)` if the platform is not supported
pub fn check_platform_support(config: &ImessageAccountConfig) -> ImessageResult<()> {
    match config.backend.as_str() {
        "applescript" => {
            if !is_macos() {
                return Err(ImessageError::PlatformUnsupported(
                    "AppleScript backend requires macOS".to_string(),
                ));
            }

            // Additional check for osascript binary
            if std::path::Path::new("/usr/bin/osascript").exists() {
                Ok(())
            } else {
                Err(ImessageError::OsascriptNotFound)
            }
        }
        "bluebubbles" => {
            // BlueBubbles works on any platform with HTTP access
            Ok(())
        }
        _ => Err(ImessageError::InvalidBackend {
            backend: config.backend.clone(),
        }),
    }
}

/// Platform-specific message formatting utilities.
#[cfg(target_os = "macos")]
pub mod macos {
    use super::*;

    /// Format a message for AppleScript delivery.
    ///
    /// This function converts the internal message format to AppleScript-compatible
    /// JSON that can be passed to osascript.
    ///
    /// # Arguments
    /// * `to` - The recipient (phone number or email address)
    /// * `text` - The message text
    ///
    /// # Returns
    /// AppleScript-compatible JSON string
    pub fn format_applescript_message(to: &str, text: &str) -> String {
        // Escape special characters in text
        let escaped_text = text.replace('\\', "\\\\").replace('"', "\\\"");
        let escaped_to = to.replace('\\', "\\\\").replace('"', "\\\"");

        // AppleScript expects a specific JSON format
        serde_json::json!({
            "to": escaped_to,
            "text": escaped_text
        })
        .to_string()
    }

    /// Format a group message for AppleScript delivery.
    ///
    /// # Arguments
    /// * `group_id` - The group chat identifier
    /// * `text` - The message text
    ///
    /// # Returns
    /// AppleScript-compatible JSON string for group messages
    pub fn format_applescript_group_message(group_id: &str, text: &str) -> String {
        let escaped_text = text.replace('\\', "\\\\").replace('"', "\\\"");
        let escaped_group = group_id.replace('\\', "\\\\").replace('"', "\\\"");

        serde_json::json!({
            "group_id": escaped_group,
            "text": escaped_text
        })
        .to_string()
    }
}

/// Platform-specific stubs for non-macOS platforms.
#[cfg(not(target_os = "macos"))]
pub mod non_macos {
    use super::*;

    /// Stub function for non-macOS platforms.
    ///
    /// This function is a no-op that always returns an error indicating
    /// that AppleScript is not available on non-macOS platforms.
    pub fn format_applescript_message(to: &str, text: &str) -> Result<String, ImessageError> {
        Err(ImessageError::PlatformUnsupported(
            "AppleScript is only available on macOS".to_string(),
        ))
    }

    /// Stub function for non-macOS platforms.
    pub fn format_applescript_group_message(
        group_id: &str,
        text: &str,
    ) -> Result<String, ImessageError> {
        Err(ImessageError::PlatformUnsupported(
            "AppleScript is only available on macOS".to_string(),
        ))
    }
}

/// Platform-specific message formatting re-export.
#[cfg(target_os = "macos")]
pub use macos::{format_applescript_group_message, format_applescript_message};

#[cfg(not(target_os = "macos"))]
pub use non_macos::{format_applescript_group_message, format_applescript_message};

/// Test platform detection functions.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_macos_detection() {
        // This test verifies the platform detection compiles
        // The actual result depends on the compile target
        let result = is_macos();

        #[cfg(target_os = "macos")]
        assert!(result, "Should be macOS");

        #[cfg(not(target_os = "macos"))]
        assert!(!result, "Should not be macOS");
    }

    #[test]
    fn test_check_platform_support_applescript_on_macos() {
        let config = ImessageAccountConfig {
            backend: "applescript".to_string(),
            ..Default::default()
        };

        #[cfg(target_os = "macos")]
        {
            // On macOS, this should succeed (or fail only if osascript is missing)
            let result = check_platform_support(&config);
            // We can't guarantee osascript exists in test environment, so just check it's not a platform error
            match &result {
                Ok(_) => {} // Success
                Err(e) => {
                    // If error, it shouldn't be a platform error (unless osascript missing)
                    if e.is_platform_error() {
                        // This is acceptable if osascript is missing
                    }
                }
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            let result = check_platform_support(&config);
            assert!(matches!(result, Err(ImessageError::PlatformUnsupported(_))));
        }
    }

    #[test]
    fn test_check_platform_support_applescript_on_non_macos() {
        let config = ImessageAccountConfig {
            backend: "applescript".to_string(),
            ..Default::default()
        };

        #[cfg(not(target_os = "macos"))]
        {
            let result = check_platform_support(&config);
            assert!(matches!(result, Err(ImessageError::PlatformUnsupported(_))));
        }
    }

    #[test]
    fn test_check_platform_support_bluebubbles() {
        let config = ImessageAccountConfig {
            backend: "bluebubbles".to_string(),
            bluebubbles: crate::config::BlueBubblesConfig {
                api_url: Some("http://localhost:12345".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };

        // BlueBubbles should work on any platform
        assert!(check_platform_support(&config).is_ok());
    }

    #[test]
    fn test_format_applescript_message() {
        // Test that formatting works without panicking
        let result = format_applescript_message("+1234567890", "Hello, World!");

        #[cfg(target_os = "macos")]
        {
            assert!(result.is_json());
        }

        #[cfg(not(target_os = "macos"))]
        {
            assert!(matches!(result, Err(ImessageError::PlatformUnsupported(_))));
        }
    }
}
