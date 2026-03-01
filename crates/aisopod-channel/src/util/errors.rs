//! Common error types for channel operations.
//!
//! This module defines a unified `ChannelError` enum that covers common
//! error scenarios across all channel implementations. This allows for
//! consistent error handling and conversion from platform-specific errors.

use std::error::Error;
use std::fmt;

/// Common error types for channel operations.
///
/// This enum provides a unified error type that covers common error
/// scenarios across all channel implementations. It includes variants
/// for authentication, rate limiting, message validation, media handling,
/// connection issues, and platform-specific errors.
///
/// # Platform Errors
///
/// Channel implementations can convert their platform-specific errors
/// to `ChannelError` using the `From` trait implementations.
///
/// # Example
///
/// ```
/// use aisopod_channel::util::errors::ChannelError;
///
/// // Example of common error scenarios
/// let auth_error = ChannelError::AuthenticationFailed;
/// let rate_limited = ChannelError::RateLimited {
///     retry_after: std::time::Duration::from_secs(30),
/// };
/// ```
#[derive(Debug)]
pub enum ChannelError {
    /// Authentication failed - invalid or expired token
    AuthenticationFailed,

    /// Rate limit exceeded - retry after specified duration
    RateLimited {
        /// Duration to wait before retrying
        retry_after: std::time::Duration,
    },

    /// Message too long for the platform
    MessageTooLong {
        /// Maximum allowed message length
        max_length: usize,
    },

    /// Media format not supported by the platform
    MediaUnsupported {
        /// The unsupported media type
        media_type: String,
    },

    /// Connection lost - WebSocket or HTTP connection dropped
    ConnectionLost,

    /// Insufficient permissions for the requested operation
    PermissionDenied,

    /// Resource not found
    NotFound {
        /// The name or ID of the missing resource
        resource: String,
    },

    /// Platform-specific error (pass-through)
    PlatformError {
        /// Platform-specific error code
        code: String,
        /// Platform-specific error message
        message: String,
    },

    /// Generic channel error
    Generic(String),

    /// I/O error
    Io(std::io::Error),

    /// Other error type
    Other(anyhow::Error),
}

impl ChannelError {
    /// Create a new `ChannelError::PlatformError`.
    pub fn platform_error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::PlatformError {
            code: code.into(),
            message: message.into(),
        }
    }

    /// Create a new `ChannelError::NotFound`.
    pub fn not_found(resource: impl Into<String>) -> Self {
        Self::NotFound {
            resource: resource.into(),
        }
    }

    /// Create a new `ChannelError::MessageTooLong`.
    pub fn message_too_long(max_length: usize) -> Self {
        Self::MessageTooLong { max_length }
    }

    /// Create a new `ChannelError::MediaUnsupported`.
    pub fn media_unsupported(media_type: impl Into<String>) -> Self {
        Self::MediaUnsupported {
            media_type: media_type.into(),
        }
    }

    /// Create a new `ChannelError::RateLimited`.
    pub fn rate_limited(retry_after: std::time::Duration) -> Self {
        Self::RateLimited { retry_after }
    }

    /// Check if the error indicates the connection should be retried.
    pub fn should_retry(&self) -> bool {
        matches!(
            self,
            Self::RateLimited { .. } | Self::ConnectionLost | Self::Generic(_)
        )
    }

    /// Get the retry-after duration if this is a rate-limited error.
    pub fn retry_after(&self) -> Option<std::time::Duration> {
        match self {
            Self::RateLimited { retry_after } => Some(*retry_after),
            _ => None,
        }
    }
}

impl fmt::Display for ChannelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AuthenticationFailed => write!(f, "Authentication failed"),
            Self::RateLimited { retry_after } => {
                write!(f, "Rate limit exceeded, retry after {:?}", retry_after)
            }
            Self::MessageTooLong { max_length } => {
                write!(f, "Message too long (max {} characters)", max_length)
            }
            Self::MediaUnsupported { media_type } => {
                write!(f, "Media format {} not supported", media_type)
            }
            Self::ConnectionLost => write!(f, "Connection lost"),
            Self::PermissionDenied => write!(f, "Permission denied"),
            Self::NotFound { resource } => write!(f, "Resource not found: {}", resource),
            Self::PlatformError { code, message } => {
                write!(f, "Platform error [{}]: {}", code, message)
            }
            Self::Generic(msg) => write!(f, "Channel error: {}", msg),
            Self::Io(err) => write!(f, "I/O error: {}", err),
            Self::Other(err) => write!(f, "Other error: {}", err),
        }
    }
}

impl Error for ChannelError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            Self::Other(err) => {
                // anyhow::Error implements Error, but it's a struct that
                // wraps the underlying error. We can't directly cast it
                // to a trait object, but we can use the anyhow crate's
                // source() method if available
                None
            }
            _ => None,
        }
    }
}

// ============================================================================
// Platform-specific error conversions
// ============================================================================

// Note: Actual `From` implementations for platform-specific errors would be
// in the individual channel crates (aisopod-channel-telegram, etc.).
// These are provided as examples of how to implement them.

/// Example: Convert a Telegram-specific error to ChannelError.
///
/// This would be implemented in the aisopod-channel-telegram crate.
/// ```ignore
/// use teloxide::ApiError;
/// use aisopod_channel::util::errors::ChannelError;
///
/// impl From<ApiError> for ChannelError {
///     fn from(err: ApiError) -> Self {
///         match err {
///             ApiError::Unauthorized => ChannelError::AuthenticationFailed,
///             ApiError::Forbidden => ChannelError::PermissionDenied,
///             ApiError::NotFound => ChannelError::not_found("resource"),
///             ApiError::TooManyRequests { retry_after } => {
///                 ChannelError::rate_limited(retry_after)
///             }
///             _ => ChannelError::Generic(err.to_string()),
///         }
///     }
/// }
/// ```

/// Example: Convert a Discord-specific error to ChannelError.
///
/// This would be implemented in the aisopod-channel-discord crate.
/// ```ignore
/// use serenity::Error as SerenityError;
/// use aisopod_channel::util::errors::ChannelError;
///
/// impl From<SerenityError> for ChannelError {
///     fn from(err: SerenityError) -> Self {
///         match err {
///             SerenityError::Http(http_err) => {
///                 if http_err.status() == 401 {
///                     return ChannelError::AuthenticationFailed;
///                 }
///                 if http_err.status() == 403 {
///                     return ChannelError::PermissionDenied;
///                 }
///                 if http_err.status() == 404 {
///                     return ChannelError::not_found("resource");
///                 }
///                 if http_err.status() == 429 {
///                     // Handle rate limit
///                 }
///                 ChannelError::Generic(http_err.to_string())
///             }
///             _ => ChannelError::Generic(err.to_string()),
///         }
///     }
/// }
/// ```

/// Example: Convert a WhatsApp-specific error to ChannelError.
///
/// This would be implemented in the aisopod-channel-whatsapp crate.
/// ```ignore
/// use aisopod_channel::util::errors::ChannelError;
///
/// impl From<anyhow::Error> for ChannelError {
///     fn from(err: anyhow::Error) -> Self {
///         // WhatsApp API specific error parsing
///         ChannelError::Generic(err.to_string())
///     }
/// }
/// ```

/// Example: Convert a Slack-specific error to ChannelError.
///
/// This would be implemented in the aisopod-channel-slack crate.
/// ```ignore
/// use slack_rtm_api::Error as SlackError;
/// use aisopod_channel::util::errors::ChannelError;
///
/// impl From<SlackError> for ChannelError {
///     fn from(err: SlackError) -> Self {
///         ChannelError::Generic(err.to_string())
///     }
/// }
/// ```

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_error_display() {
        let error = ChannelError::AuthenticationFailed;
        assert_eq!(format!("{}", error), "Authentication failed");

        let error = ChannelError::ConnectionLost;
        assert_eq!(format!("{}", error), "Connection lost");

        let error = ChannelError::PermissionDenied;
        assert_eq!(format!("{}", error), "Permission denied");
    }

    #[test]
    fn test_rate_limited_error() {
        let retry_duration = std::time::Duration::from_secs(30);
        let error = ChannelError::RateLimited {
            retry_after: retry_duration,
        };

        assert!(format!("{}", error).contains("retry after 30s"));
        assert_eq!(error.retry_after(), Some(retry_duration));
    }

    #[test]
    fn test_message_too_long() {
        let error = ChannelError::MessageTooLong { max_length: 4096 };
        assert!(format!("{}", error).contains("4096"));
    }

    #[test]
    fn test_media_unsupported() {
        let error = ChannelError::MediaUnsupported {
            media_type: "webp".to_string(),
        };
        assert!(format!("{}", error).contains("webp"));
    }

    #[test]
    fn test_not_found() {
        let error = ChannelError::NotFound {
            resource: "channel-123".to_string(),
        };
        assert!(format!("{}", error).contains("channel-123"));
    }

    #[test]
    fn test_platform_error() {
        let error = ChannelError::PlatformError {
            code: "ERR_429".to_string(),
            message: "Too many requests".to_string(),
        };

        assert!(format!("{}", error).contains("ERR_429"));
        assert!(format!("{}", error).contains("Too many requests"));
    }

    #[test]
    fn test_generic_error() {
        let error = ChannelError::Generic("Something went wrong".to_string());
        assert!(format!("{}", error).contains("Something went wrong"));
    }

    #[test]
    fn test_should_retry() {
        // These should trigger a retry
        let error = ChannelError::RateLimited {
            retry_after: std::time::Duration::from_secs(1),
        };
        assert!(error.should_retry());

        let error = ChannelError::ConnectionLost;
        assert!(error.should_retry());

        let error = ChannelError::Generic("Temporary failure".to_string());
        assert!(error.should_retry());

        // These should NOT trigger a retry
        let error = ChannelError::AuthenticationFailed;
        assert!(!error.should_retry());

        let error = ChannelError::PermissionDenied;
        assert!(!error.should_retry());

        let error = ChannelError::NotFound {
            resource: "resource".to_string(),
        };
        assert!(!error.should_retry());
    }

    #[test]
    fn test_retry_after_duration() {
        let retry_duration = std::time::Duration::from_secs(45);
        let error = ChannelError::RateLimited {
            retry_after: retry_duration,
        };

        assert_eq!(error.retry_after(), Some(retry_duration));

        let error = ChannelError::AuthenticationFailed;
        assert_eq!(error.retry_after(), None);
    }

    #[test]
    fn test_utility_methods() {
        // Test platform_error
        let error = ChannelError::platform_error("E001", "Invalid request");
        assert!(matches!(error, ChannelError::PlatformError { .. }));

        // Test not_found
        let error = ChannelError::not_found("user-123");
        assert!(matches!(error, ChannelError::NotFound { .. }));

        // Test message_too_long
        let error = ChannelError::message_too_long(1024);
        assert!(matches!(error, ChannelError::MessageTooLong { .. }));

        // Test media_unsupported
        let error = ChannelError::media_unsupported("custom-format");
        assert!(matches!(error, ChannelError::MediaUnsupported { .. }));

        // Test rate_limited
        let error = ChannelError::rate_limited(std::time::Duration::from_secs(10));
        assert!(matches!(error, ChannelError::RateLimited { .. }));
    }

    #[test]
    fn test_comprehensive_error_handling() {
        // Test a comprehensive error handling scenario
        let errors = vec![
            ChannelError::AuthenticationFailed,
            ChannelError::RateLimited {
                retry_after: std::time::Duration::from_secs(1),
            },
            ChannelError::MessageTooLong { max_length: 4096 },
            ChannelError::MediaUnsupported {
                media_type: "webp".to_string(),
            },
            ChannelError::ConnectionLost,
            ChannelError::PermissionDenied,
            ChannelError::NotFound {
                resource: "channel".to_string(),
            },
            ChannelError::PlatformError {
                code: "ERR_500".to_string(),
                message: "Server error".to_string(),
            },
        ];

        for error in errors {
            // Each error should be displayable
            let _ = format!("{}", error);

            // Each error should implement std::error::Error
            let _: &dyn Error = &error;
        }
    }
}
