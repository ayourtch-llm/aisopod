//! Platform-specific error mapping utilities.
//!
//! This module provides utilities for converting platform-specific errors
//! into standardized error types, enabling consistent error handling across
//! different messaging channel implementations.

use std::fmt;

/// Standardized error type for channel operations.
///
/// This enum provides a unified error type that can represent errors
/// from various messaging platforms while preserving platform-specific
/// error information for debugging.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelError {
    /// Authentication or authorization failed
    Authentication {
        /// Platform-specific error message
        platform: String,
        /// Detailed error message
        message: String,
    },
    /// Rate limit exceeded
    RateLimit {
        /// Platform name
        platform: String,
        /// Time until rate limit resets
        reset_after: std::time::Duration,
        /// HTTP status code if available
        status_code: Option<u16>,
    },
    /// Network or connection error
    Network {
        /// Platform name
        platform: String,
        /// Error message
        message: String,
        /// Optional HTTP status code
        status_code: Option<u16>,
    },
    /// Invalid request or parameter error
    InvalidRequest {
        /// Platform name
        platform: String,
        /// Error message
        message: String,
        /// Optional error code
        error_code: Option<String>,
    },
    /// Resource not found error
    NotFound {
        /// Platform name
        platform: String,
        /// Resource identifier
        resource_id: String,
        /// Resource type
        resource_type: String,
    },
    /// Server or platform error
    Server {
        /// Platform name
        platform: String,
        /// Error message
        message: String,
        /// Optional server response
        server_response: Option<String>,
    },
    /// Gateway or upstream service error
    Gateway {
        /// Upstream service name
        service: String,
        /// Error message
        message: String,
    },
    /// Generic error with platform context
    Generic {
        /// Platform name
        platform: String,
        /// Error message
        message: String,
    },
}

impl ChannelError {
    /// Create a new generic error with platform context.
    pub fn generic(platform: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Generic {
            platform: platform.into(),
            message: message.into(),
        }
    }

    /// Create a new authentication error.
    pub fn authentication(platform: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Authentication {
            platform: platform.into(),
            message: message.into(),
        }
    }

    /// Create a new rate limit error.
    pub fn rate_limit(
        platform: impl Into<String>,
        reset_after: std::time::Duration,
        status_code: Option<u16>,
    ) -> Self {
        Self::RateLimit {
            platform: platform.into(),
            reset_after,
            status_code,
        }
    }

    /// Create a new network error.
    pub fn network(
        platform: impl Into<String>,
        message: impl Into<String>,
        status_code: Option<u16>,
    ) -> Self {
        Self::Network {
            platform: platform.into(),
            message: message.into(),
            status_code,
        }
    }

    /// Create a new invalid request error.
    pub fn invalid_request(
        platform: impl Into<String>,
        message: impl Into<String>,
        error_code: Option<impl Into<String>>,
    ) -> Self {
        Self::InvalidRequest {
            platform: platform.into(),
            message: message.into(),
            error_code: error_code.map(|e| e.into()),
        }
    }

    /// Create a new not found error.
    pub fn not_found(
        platform: impl Into<String>,
        resource_id: impl Into<String>,
        resource_type: impl Into<String>,
    ) -> Self {
        Self::NotFound {
            platform: platform.into(),
            resource_id: resource_id.into(),
            resource_type: resource_type.into(),
        }
    }

    /// Create a new server error.
    pub fn server(
        platform: impl Into<String>,
        message: impl Into<String>,
        server_response: Option<impl Into<String>>,
    ) -> Self {
        Self::Server {
            platform: platform.into(),
            message: message.into(),
            server_response: server_response.map(|e| e.into()),
        }
    }

    /// Create a new gateway error.
    pub fn gateway(service: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Gateway {
            service: service.into(),
            message: message.into(),
        }
    }

    /// Get the platform name for this error.
    pub fn platform(&self) -> Option<&str> {
        match self {
            ChannelError::Authentication { platform, .. } => Some(platform),
            ChannelError::RateLimit { platform, .. } => Some(platform),
            ChannelError::Network { platform, .. } => Some(platform),
            ChannelError::InvalidRequest { platform, .. } => Some(platform),
            ChannelError::NotFound { platform, .. } => Some(platform),
            ChannelError::Server { platform, .. } => Some(platform),
            ChannelError::Gateway { .. } => None,
            ChannelError::Generic { platform, .. } => Some(platform),
        }
    }

    /// Get the error message.
    pub fn message(&self) -> &str {
        match self {
            ChannelError::Authentication { message, .. }
            | ChannelError::Network { message, .. }
            | ChannelError::InvalidRequest { message, .. }
            | ChannelError::Server { message, .. }
            | ChannelError::Gateway { message, .. }
            | ChannelError::Generic { message, .. } => message,
            ChannelError::RateLimit { .. } => "Rate limit exceeded",
            ChannelError::NotFound { .. } => "Resource not found",
        }
    }

    /// Check if this error indicates a rate limit was exceeded.
    pub fn is_rate_limit(&self) -> bool {
        matches!(self, ChannelError::RateLimit { .. })
    }

    /// Check if this error indicates an authentication failure.
    pub fn is_authentication(&self) -> bool {
        matches!(self, ChannelError::Authentication { .. })
    }

    /// Check if this error indicates a network issue.
    pub fn is_network(&self) -> bool {
        matches!(self, ChannelError::Network { .. })
    }

    /// Check if this error indicates a server error.
    pub fn is_server_error(&self) -> bool {
        matches!(self, ChannelError::Server { .. })
    }

    /// Check if this error indicates a gateway issue.
    pub fn is_gateway_error(&self) -> bool {
        matches!(self, ChannelError::Gateway { .. })
    }
}

impl fmt::Display for ChannelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChannelError::Authentication { platform, message } => {
                write!(f, "[{platform}] Authentication error: {message}")
            }
            ChannelError::RateLimit {
                platform,
                reset_after,
                status_code,
            } => {
                let status = status_code
                    .map(|s| format!(" (status {s})"))
                    .unwrap_or_default();
                write!(
                    f,
                    "[{platform}] Rate limit exceeded{status}, retry after {reset_after:?}"
                )
            }
            ChannelError::Network {
                platform,
                message,
                status_code,
            } => {
                let status = status_code
                    .map(|s| format!(" (status {s})"))
                    .unwrap_or_default();
                write!(f, "[{platform}] Network error{status}: {message}")
            }
            ChannelError::InvalidRequest {
                platform,
                message,
                error_code,
            } => {
                let code = error_code
                    .as_ref()
                    .map(|c| format!(" (code {c})"))
                    .unwrap_or_default();
                write!(f, "[{platform}] Invalid request{code}: {message}")
            }
            ChannelError::NotFound {
                platform,
                resource_id,
                resource_type,
            } => {
                write!(f, "[{platform}] {resource_type} '{resource_id}' not found")
            }
            ChannelError::Server {
                platform,
                message,
                server_response,
            } => {
                let response = server_response
                    .as_ref()
                    .map(|r| format!("\nServer response: {r}"))
                    .unwrap_or_default();
                write!(f, "[{platform}] Server error: {message}{response}")
            }
            ChannelError::Gateway { service, message } => {
                write!(f, "[{service}] Gateway error: {message}")
            }
            ChannelError::Generic { platform, message } => {
                write!(f, "[{platform}] Error: {message}")
            }
        }
    }
}

impl std::error::Error for ChannelError {}

/// Platform-specific error mapper trait.
///
/// Implement this trait to provide custom error mapping for a platform.
pub trait PlatformErrorMapper {
    /// Map a platform-specific error to a ChannelError.
    ///
    /// # Arguments
    ///
    /// * `error` - The platform-specific error to map
    ///
    /// # Returns
    ///
    /// Returns a ChannelError that represents the platform error.
    fn map_error(error: impl fmt::Display) -> ChannelError;

    /// Determine the platform from an error.
    fn platform_name() -> &'static str;
}

/// Result type for channel operations.
pub type ChannelResult<T> = Result<T, ChannelError>;

/// Convert an HTTP status code to a ChannelError.
///
/// # Arguments
///
/// * `status` - HTTP status code
/// * `message` - Error message
/// * `platform` - Platform name
///
/// # Returns
///
/// Returns a ChannelError appropriate for the status code.
pub fn error_from_http_status(
    status: u16,
    message: impl Into<String>,
    platform: impl Into<String>,
) -> ChannelError {
    match status {
        400 => ChannelError::invalid_request(platform, message, Some("invalid_request")),
        401 => ChannelError::authentication(platform, "Invalid credentials"),
        403 => ChannelError::authentication(platform, "Access denied"),
        404 => ChannelError::not_found(platform, "resource", "resource"),
        405 => ChannelError::invalid_request(platform, message, Some("method_not_allowed")),
        409 => ChannelError::invalid_request(platform, message, Some("conflict")),
        429 => ChannelError::rate_limit(platform, std::time::Duration::from_secs(60), Some(status)),
        500 => ChannelError::server(platform, message, None::<String>),
        502 => ChannelError::gateway(platform, "Bad gateway"),
        503 => ChannelError::gateway(platform, "Service unavailable"),
        504 => ChannelError::gateway(platform, "Gateway timeout"),
        _ if status >= 400 && status < 500 => {
            ChannelError::invalid_request(platform, message, Some(status.to_string()))
        }
        _ if status >= 500 => ChannelError::server(platform, message, None::<String>),
        _ => ChannelError::Generic {
            platform: platform.into(),
            message: message.into(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authentication_error() {
        let error = ChannelError::authentication("discord", "Invalid token");

        assert_eq!(error.platform(), Some("discord"));
        assert_eq!(error.message(), "Invalid token");
        assert!(error.is_authentication());
        assert!(!error.is_rate_limit());

        let display = format!("{}", error);
        assert!(display.contains("[discord]"));
        assert!(display.contains("Authentication error"));
        assert!(display.contains("Invalid token"));
    }

    #[test]
    fn test_rate_limit_error() {
        let error =
            ChannelError::rate_limit("signal", std::time::Duration::from_secs(30), Some(429));

        assert_eq!(error.platform(), Some("signal"));
        assert!(error.is_rate_limit());

        let display = format!("{}", error);
        assert!(display.contains("[signal]"));
        assert!(display.contains("Rate limit exceeded"));
    }

    #[test]
    fn test_network_error() {
        let error = ChannelError::network("telegram", "Connection timeout", Some(504));

        assert_eq!(error.platform(), Some("telegram"));
        assert!(error.is_network());

        let display = format!("{}", error);
        assert!(display.contains("[telegram]"));
        assert!(display.contains("Network error"));
        assert!(display.contains("Connection timeout"));
    }

    #[test]
    fn test_invalid_request_error() {
        let error = ChannelError::invalid_request(
            "whatsapp",
            "Invalid phone number",
            Some("invalid_parameter"),
        );

        assert_eq!(error.platform(), Some("whatsapp"));

        let display = format!("{}", error);
        assert!(display.contains("[whatsapp]"));
        assert!(display.contains("Invalid request"));
        assert!(display.contains("invalid_parameter"));
    }

    #[test]
    fn test_not_found_error() {
        let error = ChannelError::not_found("slack", "C123456", "channel");

        assert_eq!(error.platform(), Some("slack"));

        let display = format!("{}", error);
        assert!(display.contains("[slack]"));
        assert!(display.contains("channel"));
        assert!(display.contains("not found"));
    }

    #[test]
    fn test_server_error() {
        let error = ChannelError::server("discord", "Internal server error", Some("Error 500"));

        assert_eq!(error.platform(), Some("discord"));
        assert!(error.is_server_error());

        let display = format!("{}", error);
        assert!(display.contains("[discord]"));
        assert!(display.contains("Server error"));
        assert!(display.contains("Error 500"));
    }

    #[test]
    fn test_gateway_error() {
        let error = ChannelError::gateway("upstream-api", "Service unavailable");

        assert!(error.platform().is_none());
        assert!(error.is_gateway_error());

        let display = format!("{}", error);
        assert!(display.contains("[upstream-api]"));
        assert!(display.contains("Gateway error"));
    }

    #[test]
    fn test_generic_error() {
        let error = ChannelError::generic("irc", "Unknown error occurred");

        assert_eq!(error.platform(), Some("irc"));

        let display = format!("{}", error);
        assert!(display.contains("[irc]"));
        assert!(display.contains("Error: Unknown error occurred"));
    }

    #[test]
    fn test_error_from_http_status() {
        let error = error_from_http_status(429, "Too many requests", "discord");
        assert!(error.is_rate_limit());

        let error = error_from_http_status(401, "Unauthorized", "telegram");
        assert!(error.is_authentication());

        let error = error_from_http_status(500, "Server error", "slack");
        assert!(error.is_server_error());

        let error = error_from_http_status(502, "Bad gateway", "signal");
        assert!(error.is_gateway_error());
    }

    #[test]
    fn test_error_display_formatting() {
        let error = ChannelError::authentication("discord", "Token expired");
        let display = format!("{}", error);

        // Verify formatting includes all components
        assert!(display.contains("[discord]"));
        assert!(display.contains("Authentication error"));
        assert!(display.contains("Token expired"));
    }

    #[test]
    fn test_error_is_commutative() {
        let error1 = ChannelError::authentication("discord", "Token expired");
        let error2 = ChannelError::authentication("discord", "Token expired");

        assert_eq!(error1, error2);
        assert_eq!(error1.platform(), error2.platform());
        assert_eq!(error1.message(), error2.message());
    }
}
