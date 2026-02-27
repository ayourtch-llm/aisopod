//! OAuth authentication support for Twitch.
//!
//! This module provides functionality for validating OAuth tokens
//! and retrieving user information from Twitch's API.

use anyhow::{anyhow, Result};
use futures::TryFutureExt;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

/// Information about a validated OAuth token.
#[derive(Debug, Deserialize, Clone)]
pub struct TokenInfo {
    /// The client ID that created this token
    pub client_id: String,
    /// The scopes the token has access to
    pub scopes: Vec<String>,
    /// The user ID associated with the token
    pub user_id: String,
    /// The username associated with the token
    pub login: String,
    /// Token expiration time (Unix timestamp)
    pub expires_in: u64,
    /// When the token was created (Unix timestamp)
    pub created_at: u64,
    /// Token expiration timestamp
    pub expires_at: u64,
}

/// Error types for authentication operations.
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    /// Token validation failed
    #[error("Invalid or expired token: {0}")]
    InvalidToken(String),
    /// Network error during validation
    #[error("Network error: {0}")]
    NetworkError(String),
    /// API error from Twitch
    #[error("Twitch API error: {0}")]
    ApiError(String),
}

/// Validate a Twitch OAuth token.
///
/// This function calls Twitch's validation endpoint to verify that
/// the provided OAuth token is valid and returns token information.
///
/// # Arguments
///
/// * `oauth_token` - The OAuth token to validate (e.g., "oauth:abc123")
/// * `client_id` - The client ID for your Twitch application
///
/// # Returns
///
/// * `Ok(TokenInfo)` - Information about the validated token
/// * `Err(AuthError)` - An error if validation fails
///
/// # Example
///
/// ```no_run
/// use aisopod_channel_twitch::{validate_token, TokenInfo};
///
/// async fn validate() -> Result<(), Box<dyn std::error::Error>> {
///     let token = "oauth:abc123";
///     let client_id = "your_client_id";
///     let info = validate_token(token, client_id).await?;
///     println!("Token valid for user: {}", info.login);
///     Ok(())
/// }
/// ```
pub async fn validate_token(oauth_token: &str, client_id: &str) -> Result<TokenInfo> {
    // Remove the "oauth:" prefix if present
    let token = oauth_token.trim_start_matches("oauth:");

    debug!("Validating token for client {}", client_id);

    // Call Twitch's token validation endpoint
    let url = format!("https://id.twitch.tv/oauth2/validate");
    
    let response = reqwest::Client::new()
        .get(&url)
        .header("Authorization", format!("OAuth {}", token))
        .send()
        .await
        .map_err(|e| AuthError::NetworkError(e.to_string()))?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        error!(
            "Token validation failed with status {}: {}",
            status,
            text
        );
        return Err(anyhow!(
            "Token validation failed: {} {}",
            status,
            text
        ));
    }

    let info = response.json::<TokenInfo>().await.map_err(|e| {
        error!("Failed to parse token info: {}", e);
        AuthError::ApiError(e.to_string())
    })?;

    info!(
        "Token validated for user {} (expires in {} seconds)",
        info.login, info.expires_in
    );

    Ok(info)
}

/// Validate a Twitch OAuth token (blocking version).
///
/// This is a blocking version of `validate_token` that can be used
/// in synchronous contexts. For async contexts, use `validate_token`.
///
/// # Arguments
///
/// * `oauth_token` - The OAuth token to validate
/// * `client_id` - The client ID for your Twitch application
///
/// # Returns
///
/// * `Ok(TokenInfo)` - Information about the validated token
/// * `Err(AuthError)` - An error if validation fails
pub fn validate_token_blocking(oauth_token: &str, client_id: &str) -> Result<TokenInfo> {
    let token = oauth_token.trim_start_matches("oauth:");

    debug!("Validating token for client {} (blocking)", client_id);

    let url = format!("https://id.twitch.tv/oauth2/validate");
    
    // Use tokio's block_in_place to run async code in a blocking context
    tokio::runtime::Handle::current().block_on(async move {
        let response = reqwest::Client::new()
            .get(&url)
            .header("Authorization", format!("OAuth {}", token))
            .send()
            .await
            .map_err(|e| AuthError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            error!(
                "Token validation failed with status {}: {}",
                status,
                text
            );
            return Err(anyhow!(
                "Token validation failed: {} {}",
                status,
                text
            ));
        }

        let info = response.json::<TokenInfo>().await.map_err(|e| {
            error!("Failed to parse token info: {}", e);
            AuthError::ApiError(e.to_string())
        })?;

        info!(
            "Token validated for user {} (expires in {} seconds)",
            info.login, info.expires_in
        );

        Ok(info)
    })
}

/// Check if a token has specific scopes.
///
/// # Arguments
///
/// * `info` - The token information
/// * `required_scopes` - The scopes to check for
///
/// # Returns
///
/// `true` if the token has all the required scopes, `false` otherwise.
pub fn token_has_scopes(info: &TokenInfo, required_scopes: &[&str]) -> bool {
    let token_scopes: std::collections::HashSet<&str> = info.scopes.iter().map(|s| s.as_str()).collect();
    required_scopes.iter().all(|s| token_scopes.contains(s))
}

/// Check if the token is expired.
///
/// # Arguments
///
/// * `info` - The token information
///
/// # Returns
///
/// `true` if the token is expired, `false` otherwise.
pub fn is_token_expired(info: &TokenInfo) -> bool {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    info.expires_at <= now
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_info_deserialization() {
        let json = r#"{
            "client_id": "abc123",
            "scopes": ["chat:read", "chat:edit"],
            "user_id": "123456789",
            "login": "testbot",
            "expires_in": 3600,
            "created_at": 1234567890,
            "expires_at": 1234571490
        }"#;

        let info: TokenInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.client_id, "abc123");
        assert_eq!(info.scopes, vec!["chat:read", "chat:edit"]);
        assert_eq!(info.login, "testbot");
        assert_eq!(info.user_id, "123456789");
        assert_eq!(info.expires_in, 3600);
    }

    #[test]
    fn test_token_has_scopes() {
        let info = TokenInfo {
            client_id: "abc123".to_string(),
            scopes: vec!["chat:read".to_string(), "chat:edit".to_string(), "user:read:email".to_string()],
            user_id: "123456789".to_string(),
            login: "testbot".to_string(),
            expires_in: 3600,
            created_at: 1234567890,
            expires_at: 1234571490,
        };

        assert!(token_has_scopes(&info, &["chat:read"]));
        assert!(token_has_scopes(&info, &["chat:read", "chat:edit"]));
        assert!(!token_has_scopes(&info, &["chat:read", "user:manage"]));
    }

    #[test]
    fn test_token_prefix_removal() {
        // This is a compilation test - the actual token parsing is in the function
        let token_with_prefix = "oauth:abc123";
        let token_without_prefix = "abc123";
        
        assert_eq!(token_with_prefix.trim_start_matches("oauth:"), token_without_prefix);
    }
}
