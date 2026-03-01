//! Mattermost authentication helpers.
//!
//! This module provides utilities for Mattermost authentication
//! and token management.

use crate::api::ApiError;
use crate::config::MattermostAuth;
use anyhow::Result;
use serde::Serialize;

/// Authentication result containing credentials.
#[derive(Debug, Clone, Serialize)]
pub struct AuthResult {
    /// The authentication token
    pub token: String,
    /// The user ID (if available)
    pub user_id: Option<String>,
    /// The username (if available)
    pub username: Option<String>,
}

/// Error types for Mattermost authentication.
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    /// Authentication method not supported
    #[error("Authentication method not supported: {0}")]
    UnsupportedMethod(String),
    /// Token authentication failed
    #[error("Token authentication failed: {0}")]
    TokenAuth(String),
    /// Password authentication failed
    #[error("Password authentication failed: {0}")]
    PasswordAuth(String),
    /// Invalid credentials
    #[error("Invalid credentials")]
    InvalidCredentials,
}

/// Authenticate with Mattermost using the provided configuration.
///
/// # Arguments
///
/// * `auth` - The authentication configuration
/// * `server_url` - The base URL of the Mattermost server
///
/// # Returns
///
/// * `Ok(AuthResult)` - The authentication result with token
/// * `Err(AuthError)` - An error if authentication fails
pub async fn authenticate(
    auth: &MattermostAuth,
    server_url: &str,
) -> Result<AuthResult, AuthError> {
    match auth {
        MattermostAuth::BotToken { token } => authenticate_token(token, server_url).await,
        MattermostAuth::PersonalToken { token } => authenticate_token(token, server_url).await,
        MattermostAuth::Password { username, password } => {
            authenticate_password(username, password, server_url).await
        }
    }
}

/// Authenticate using a token.
async fn authenticate_token(token: &str, server_url: &str) -> Result<AuthResult, AuthError> {
    // For token-based authentication, we can validate by making a test request
    // In production, this would typically be validated against the API
    if token.is_empty() {
        return Err(AuthError::InvalidCredentials);
    }

    // Return the token (in production, you would validate it against the API)
    Ok(AuthResult {
        token: token.to_string(),
        user_id: None,
        username: None,
    })
}

/// Authenticate using username and password.
///
/// Note: Password authentication requires an additional API call to obtain a token.
/// This implementation assumes the caller will handle the actual API authentication.
async fn authenticate_password(
    username: &str,
    password: &str,
    server_url: &str,
) -> Result<AuthResult, AuthError> {
    if username.is_empty() || password.is_empty() {
        return Err(AuthError::InvalidCredentials);
    }

    // For password authentication, we need to:
    // 1. Call POST /api/v4/users/login to get a session token
    // 2. Use that token for subsequent API calls

    // In a full implementation, this would make an HTTP request to login
    // For now, we return a placeholder result
    // The actual API call would be handled by the MattermostApi

    Ok(AuthResult {
        token: String::new(), // Will be set after API call
        user_id: None,
        username: Some(username.to_string()),
    })
}

/// Validate that the token is properly formatted.
pub fn validate_token(token: &str) -> Result<(), AuthError> {
    if token.is_empty() {
        return Err(AuthError::InvalidCredentials);
    }

    // Mattermost tokens are typically base64-encoded or UUID-based
    // For a basic check, ensure it's not empty and has reasonable length
    if token.len() < 10 {
        return Err(AuthError::TokenAuth(
            "Token appears to be too short".to_string(),
        ));
    }

    Ok(())
}

/// Extract the token from authentication configuration.
pub fn extract_token(auth: &MattermostAuth) -> Option<String> {
    match auth {
        MattermostAuth::BotToken { token } => Some(token.clone()),
        MattermostAuth::PersonalToken { token } => Some(token.clone()),
        MattermostAuth::Password { .. } => None, // Password auth requires login
    }
}

/// Determine if the authentication method requires login.
pub fn requires_login(auth: &MattermostAuth) -> bool {
    matches!(auth, MattermostAuth::Password { .. })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_token_success() {
        let token = "test-token-1234567890".to_string();
        assert!(validate_token(&token).is_ok());
    }

    #[test]
    fn test_validate_token_empty() {
        let token = "".to_string();
        assert!(matches!(
            validate_token(&token),
            Err(AuthError::InvalidCredentials)
        ));
    }

    #[test]
    fn test_validate_token_too_short() {
        let token = "abc".to_string();
        assert!(matches!(
            validate_token(&token),
            Err(AuthError::TokenAuth(_))
        ));
    }

    #[test]
    fn test_extract_token_bot() {
        let auth = MattermostAuth::BotToken {
            token: "bot-token".to_string(),
        };
        assert_eq!(extract_token(&auth), Some("bot-token".to_string()));
    }

    #[test]
    fn test_extract_token_personal() {
        let auth = MattermostAuth::PersonalToken {
            token: "personal-token".to_string(),
        };
        assert_eq!(extract_token(&auth), Some("personal-token".to_string()));
    }

    #[test]
    fn test_extract_token_password() {
        let auth = MattermostAuth::Password {
            username: "user".to_string(),
            password: "pass".to_string(),
        };
        assert_eq!(extract_token(&auth), None);
    }

    #[test]
    fn test_requires_login_bot() {
        let auth = MattermostAuth::BotToken {
            token: "token".to_string(),
        };
        assert!(!requires_login(&auth));
    }

    #[test]
    fn test_requires_login_personal() {
        let auth = MattermostAuth::PersonalToken {
            token: "token".to_string(),
        };
        assert!(!requires_login(&auth));
    }

    #[test]
    fn test_requires_login_password() {
        let auth = MattermostAuth::Password {
            username: "user".to_string(),
            password: "pass".to_string(),
        };
        assert!(requires_login(&auth));
    }

    #[test]
    fn test_auth_result_serialization() {
        let result = AuthResult {
            token: "test-token".to_string(),
            user_id: Some("user123".to_string()),
            username: Some("testuser".to_string()),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("test-token"));
        assert!(json.contains("user123"));
        assert!(json.contains("testuser"));
    }
}
