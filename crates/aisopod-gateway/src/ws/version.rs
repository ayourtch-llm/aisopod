//! Protocol version negotiation for WebSocket connections.
//!
//! This module implements semantic versioning-based protocol version negotiation
//! between clients and the server. The server supports version 1.0 and will accept
//! connections from clients supporting versions 1.x where x <= 0.

use std::fmt;

/// Represents a protocol version with major and minor numbers.
///
/// Version numbers follow semantic versioning rules where:
/// - Major version must match exactly between client and server
/// - Server minor version must be >= client minor version (backward compatible)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolVersion {
    /// Major version number
    pub major: u32,
    /// Minor version number
    pub minor: u32,
}

impl ProtocolVersion {
    /// Creates a new ProtocolVersion with the given major and minor numbers.
    pub fn new(major: u32, minor: u32) -> Self {
        Self { major, minor }
    }

    /// Parses a version string in the format "major.minor" (e.g., "1.0").
    ///
    /// # Errors
    /// Returns `VersionError::InvalidFormat` if the string is not in the
    /// expected format or contains invalid numbers.
    pub fn parse(s: &str) -> Result<Self, VersionError> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 2 {
            return Err(VersionError::InvalidFormat(s.to_string()));
        }

        let major = parts[0]
            .parse::<u32>()
            .map_err(|_| VersionError::InvalidFormat(s.to_string()))?;

        let minor = parts[1]
            .parse::<u32>()
            .map_err(|_| VersionError::InvalidFormat(s.to_string()))?;

        Ok(Self { major, minor })
    }

    /// Checks if this server version is compatible with the given client version.
    ///
    /// Compatibility rules:
    /// - Major version must match exactly
    /// - Server minor version must be >= client minor version
    ///
    /// # Examples
    /// ```
    /// # use aisopod_gateway::ws::version::ProtocolVersion;
    /// let server = ProtocolVersion::new(1, 0);
    /// assert!(server.is_compatible_with(&ProtocolVersion::new(1, 0))); // exact match
    /// assert!(!server.is_compatible_with(&ProtocolVersion::new(1, 1))); // client has newer minor (incompatible)
    /// assert!(!server.is_compatible_with(&ProtocolVersion::new(2, 0))); // different major (incompatible)
    /// ```
    pub fn is_compatible_with(&self, client: &ProtocolVersion) -> bool {
        self.major == client.major && self.minor >= client.minor
    }
}

impl fmt::Display for ProtocolVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

/// Error types for protocol version parsing and negotiation.
#[derive(Debug, thiserror::Error)]
pub enum VersionError {
    /// The version string is not in the expected format (e.g., "1.0").
    #[error("Invalid version format: {0}. Expected format: major.minor (e.g., 1.0)")]
    InvalidFormat(String),
}

/// Error returned when version negotiation fails.
#[derive(Debug)]
pub enum VersionNegotiationError {
    /// Client sent an incompatible version.
    Incompatible {
        /// The server's protocol version
        server: ProtocolVersion,
        /// The client's protocol version
        client: ProtocolVersion,
    },
    /// Client sent a malformed version string.
    ParseError(VersionError),
}

impl fmt::Display for VersionNegotiationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VersionNegotiationError::Incompatible { server, client } => {
                write!(
                    f,
                    "Protocol version mismatch: server {} is incompatible with client {}",
                    server, client
                )
            }
            VersionNegotiationError::ParseError(e) => write!(f, "Failed to parse version: {}", e),
        }
    }
}

impl std::error::Error for VersionNegotiationError {}

/// The server's protocol version.
///
/// This is the version that the server advertises and uses for compatibility checks.
pub const SERVER_PROTOCOL_VERSION: ProtocolVersion = ProtocolVersion { major: 1, minor: 0 };

/// Negotiates the protocol version based on the client's request.
///
/// This function extracts the `X-Aisopod-Protocol-Version` header from the request,
/// parses it, and checks compatibility with the server's version.
///
/// # Arguments
/// * `client_version_str` - The version string from the client (e.g., "1.0").
///   If `None`, defaults to "1.0" for backward compatibility.
///
/// # Returns
/// * `Ok(client_version)` - If the client version is compatible, returns the parsed client version
/// * `Err(VersionNegotiationError::ParseError)` - If the version string is malformed
/// * `Err(VersionNegotiationError::Incompatible)` - If the client version is incompatible
///
/// # Examples
/// ```
/// # use aisopod_gateway::ws::version::{negotiate_version, ProtocolVersion, VersionNegotiationError};
/// // Compatible version
/// let result = negotiate_version(Some("1.0"));
/// assert!(matches!(result, Ok(_)));
///
/// // Incompatible version (different major)
/// let result = negotiate_version(Some("2.0"));
/// assert!(matches!(result, Err(VersionNegotiationError::Incompatible { .. })));
///
/// // Missing version defaults to 1.0
/// let result = negotiate_version(None);
/// assert!(matches!(result, Ok(v) if v == ProtocolVersion::new(1, 0)));
/// ```
pub fn negotiate_version(
    client_version_str: Option<&str>,
) -> Result<ProtocolVersion, VersionNegotiationError> {
    // Default to "1.0" if header is missing for backward compatibility
    let version_str = client_version_str.unwrap_or("1.0");

    // Parse the client version
    let client_version =
        ProtocolVersion::parse(version_str).map_err(VersionNegotiationError::ParseError)?;

    // Check compatibility
    if SERVER_PROTOCOL_VERSION.is_compatible_with(&client_version) {
        Ok(client_version)
    } else {
        Err(VersionNegotiationError::Incompatible {
            server: SERVER_PROTOCOL_VERSION,
            client: client_version,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_version_new() {
        let v1 = ProtocolVersion::new(1, 0);
        assert_eq!(v1.major, 1);
        assert_eq!(v1.minor, 0);

        let v2 = ProtocolVersion::new(2, 5);
        assert_eq!(v2.major, 2);
        assert_eq!(v2.minor, 5);
    }

    #[test]
    fn test_protocol_version_parse_valid() {
        let v1 = ProtocolVersion::parse("1.0").unwrap();
        assert_eq!(v1.major, 1);
        assert_eq!(v1.minor, 0);

        let v2 = ProtocolVersion::parse("2.3").unwrap();
        assert_eq!(v2.major, 2);
        assert_eq!(v2.minor, 3);

        let v3 = ProtocolVersion::parse("0.1").unwrap();
        assert_eq!(v3.major, 0);
        assert_eq!(v3.minor, 1);
    }

    #[test]
    fn test_protocol_version_parse_invalid_format() {
        let result = ProtocolVersion::parse("1");
        assert!(matches!(result, Err(VersionError::InvalidFormat(_))));

        let result = ProtocolVersion::parse("1.0.0");
        assert!(matches!(result, Err(VersionError::InvalidFormat(_))));

        let result = ProtocolVersion::parse("abc.def");
        assert!(matches!(result, Err(VersionError::InvalidFormat(_))));

        let result = ProtocolVersion::parse("");
        assert!(matches!(result, Err(VersionError::InvalidFormat(_))));

        let result = ProtocolVersion::parse("1.");
        assert!(matches!(result, Err(VersionError::InvalidFormat(_))));

        let result = ProtocolVersion::parse(".0");
        assert!(matches!(result, Err(VersionError::InvalidFormat(_))));
    }

    #[test]
    fn test_protocol_version_display() {
        assert_eq!(ProtocolVersion::new(1, 0).to_string(), "1.0");
        assert_eq!(ProtocolVersion::new(2, 5).to_string(), "2.5");
        assert_eq!(ProtocolVersion::new(0, 1).to_string(), "0.1");
    }

    #[test]
    fn test_is_compatible_with_exact_match() {
        let server = ProtocolVersion::new(1, 0);
        let client = ProtocolVersion::new(1, 0);
        assert!(server.is_compatible_with(&client));
    }

    #[test]
    fn test_is_compatible_with_server_higher_minor() {
        let server = ProtocolVersion::new(1, 5);
        let client = ProtocolVersion::new(1, 3);
        assert!(server.is_compatible_with(&client));
    }

    #[test]
    fn test_is_compatible_with_client_higher_minor() {
        let server = ProtocolVersion::new(1, 3);
        let client = ProtocolVersion::new(1, 5);
        assert!(!server.is_compatible_with(&client));
    }

    #[test]
    fn test_is_compatible_with_different_major() {
        let server = ProtocolVersion::new(2, 0);
        let client = ProtocolVersion::new(1, 0);
        assert!(!server.is_compatible_with(&client));
    }

    #[test]
    fn test_is_compatible_with_zero_major() {
        let server = ProtocolVersion::new(0, 1);
        let client = ProtocolVersion::new(0, 1);
        assert!(server.is_compatible_with(&client));

        let server = ProtocolVersion::new(0, 1);
        let client = ProtocolVersion::new(0, 2);
        assert!(!server.is_compatible_with(&client));
    }

    #[test]
    fn test_negotiate_version_valid() {
        // Server version is 1.0, so compatible clients can only be 1.x where x <= 0
        let result = negotiate_version(Some("1.0"));
        assert!(matches!(result, Ok(v) if v == ProtocolVersion::new(1, 0)));
    }

    #[test]
    fn test_negotiate_version_valid_lower_minor() {
        // Server 1.5 can accept client 1.0 (client has older minor)
        // We test this by using a server with version 1.5
        // But since SERVER_PROTOCOL_VERSION is constant at 1.0,
        // we test the is_compatible_with method directly
        let server = ProtocolVersion::new(1, 5);
        let client = ProtocolVersion::new(1, 0);
        assert!(server.is_compatible_with(&client));
    }

    #[test]
    fn test_negotiate_version_default() {
        // Missing version should default to 1.0
        let result = negotiate_version(None);
        assert!(matches!(result, Ok(v) if v == ProtocolVersion::new(1, 0)));
    }

    #[test]
    fn test_negotiate_version_incompatible_major() {
        let result = negotiate_version(Some("2.0"));
        assert!(matches!(
            result,
            Err(VersionNegotiationError::Incompatible { .. })
        ));
    }

    #[test]
    fn test_negotiate_version_incompatible_higher_minor() {
        // Server 1.0 cannot accept client 1.5 (client has newer minor)
        let result = negotiate_version(Some("1.5"));
        assert!(matches!(
            result,
            Err(VersionNegotiationError::Incompatible { .. })
        ));
    }

    #[test]
    fn test_negotiate_version_parse_error() {
        let result = negotiate_version(Some("invalid"));
        assert!(matches!(
            result,
            Err(VersionNegotiationError::ParseError(_))
        ));
    }

    #[test]
    fn test_version_negotiation_error_display() {
        let err = VersionNegotiationError::Incompatible {
            server: ProtocolVersion::new(1, 0),
            client: ProtocolVersion::new(2, 0),
        };
        assert!(err.to_string().contains("Protocol version mismatch"));

        let err =
            VersionNegotiationError::ParseError(VersionError::InvalidFormat("bad".to_string()));
        assert!(err.to_string().contains("Failed to parse version"));
    }

    #[test]
    fn test_server_protocol_version_constant() {
        assert_eq!(SERVER_PROTOCOL_VERSION.major, 1);
        assert_eq!(SERVER_PROTOCOL_VERSION.minor, 0);
    }
}
