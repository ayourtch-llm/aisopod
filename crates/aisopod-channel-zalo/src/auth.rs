//! Zalo OA OAuth authentication and token management.
//!
//! This module provides functionality for managing Zalo OA access tokens,
//! including token refresh and validation.

use anyhow::Result;
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use std::time::{Duration as StdDuration, Instant};
use tracing::{debug, error, info};

/// Zalo OAuth2 token endpoint.
pub const TOKEN_ENDPOINT: &str = "https://oauth.zaloapp.com/v4/oa/access_token";

/// Zalo OAuth2 verification endpoint.
pub const VERIFY_ENDPOINT: &str = "https://oauth.zaloapp.com/v4/oa/verify";

/// Error type for Zalo authentication operations.
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Invalid token response: {0}")]
    InvalidResponse(String),

    #[error("Authentication failed: {0}")]
    Failed(String),
}

/// Response from Zalo token refresh endpoint.
#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    /// The access token
    pub access_token: String,
    /// The refresh token (may be rotated)
    #[serde(default)]
    pub refresh_token: Option<String>,
    /// Expiration time in seconds
    pub expires_in: i64,
    /// User ID (in some responses)
    #[serde(default)]
    pub user_id: Option<String>,
    /// Error code (in error responses)
    #[serde(default)]
    pub error: Option<String>,
    /// Error description (in error responses)
    #[serde(default)]
    pub error_description: Option<String>,
}

/// Zalo OAuth2 authentication manager.
///
/// This struct manages OAuth2 authentication with the Zalo OA API,
/// including token refresh and validation.
#[derive(Clone, Debug)]
pub struct ZaloAuth {
    /// Zalo OA App ID
    app_id: String,
    /// Zalo OA App Secret
    app_secret: String,
    /// Current access token
    access_token: Option<String>,
    /// OAuth refresh token
    refresh_token: String,
    /// Token expiration time
    token_expiry: Option<Instant>,
}

impl ZaloAuth {
    /// Create a new ZaloAuth instance.
    ///
    /// # Arguments
    ///
    /// * `app_id` - Zalo OA App ID from Zalo Developer Console
    /// * `app_secret` - Zalo OA App Secret from Zalo Developer Console
    /// * `refresh_token` - OAuth refresh token
    ///
    /// # Returns
    ///
    /// * `Ok(ZaloAuth)` - The authentication manager
    /// * `Err(AuthError)` - An error if initialization fails
    pub fn new(app_id: String, app_secret: String, refresh_token: String) -> Self {
        Self {
            app_id,
            app_secret,
            access_token: None,
            refresh_token,
            token_expiry: None,
        }
    }

    /// Check if the current access token is valid and not expired.
    ///
    /// # Returns
    ///
    /// * `true` - Token is valid
    /// * `false` - Token is expired or not set
    pub fn is_token_valid(&self) -> bool {
        if let Some(expiry) = self.token_expiry {
            // Refresh token 60 seconds before expiry for safety
            let now = Instant::now();
            let buffer = StdDuration::from_secs(60);
            return now < expiry - buffer;
        }
        false
    }

    /// Get the current access token, refreshing if necessary.
    ///
    /// This method will automatically refresh the token if it's expired
    /// or if no token is currently set.
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The access token
    /// * `Err(AuthError)` - An error if token refresh fails
    pub async fn get_access_token(&mut self) -> Result<String> {
        if self.is_token_valid() {
            if let Some(ref token) = self.access_token {
                debug!("Using cached access token");
                return Ok(token.clone());
            }
        }

        self.refresh_access_token().await
    }

    /// Refresh the access token using the refresh token.
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The new access token
    /// * `Err(AuthError)` - An error if refresh fails
    pub async fn refresh_access_token(&mut self) -> Result<String> {
        let client = reqwest::Client::new();

        let response = client
            .post(TOKEN_ENDPOINT)
            .header("secret_key", &self.app_secret)
            .json(&serde_json::json!({
                "app_id": self.app_id,
                "grant_type": "refresh_token",
                "refresh_token": self.refresh_token
            }))
            .send()
            .await?;

        if response.status().is_success() {
            let token_response: TokenResponse = response.json().await?;

            if let Some(error) = token_response.error {
                let error_description = token_response.error_description.unwrap_or_default();
                error!("Token refresh failed: {} - {}", error, error_description);
                return Err(anyhow::anyhow!(
                    "Token refresh failed: {} - {}",
                    error,
                    error_description
                ));
            }

            // Update access token
            self.access_token = Some(token_response.access_token.clone());

            // Calculate expiry time
            let expires_in = token_response.expires_in;
            let expiry = Instant::now() + StdDuration::from_secs(expires_in as u64);
            self.token_expiry = Some(expiry);

            // Update refresh token if it was rotated
            if let Some(new_refresh_token) = token_response.refresh_token {
                if !new_refresh_token.is_empty() {
                    info!("Refresh token rotated");
                    self.refresh_token = new_refresh_token;
                }
            }

            info!(
                "Access token refreshed, expires in {} seconds",
                expires_in
            );

            Ok(self.access_token.clone().unwrap())
        } else {
            let status = response.status().as_u16();
            let error_msg = response.text().await.unwrap_or_default();
            error!("Token refresh failed: {} - {}", status, error_msg);
            Err(anyhow::anyhow!(
                "Token refresh failed: {} - {}",
                status,
                error_msg
            ))
        }
    }

    /// Verify the current access token.
    ///
    /// # Returns
    ///
    /// * `Ok(bool)` - `true` if token is valid
    /// * `Err(AuthError)` - An error if verification fails
    pub async fn verify_token(&self) -> Result<bool> {
        let client = reqwest::Client::new();
        let token = self
            .access_token
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No access token available"))?;

        let response = client
            .post(VERIFY_ENDPOINT)
            .header("secret_key", &self.app_secret)
            .header("access_token", token)
            .send()
            .await?;

        Ok(response.status().is_success())
    }

    /// Get the app ID.
    pub fn app_id(&self) -> &str {
        &self.app_id
    }

    /// Get the refresh token.
    pub fn refresh_token(&self) -> &str {
        &self.refresh_token
    }

    /// Get the app secret.
    pub fn app_secret(&self) -> &str {
        &self.app_secret
    }

    /// Get the current access token (if any).
    pub fn access_token(&self) -> Option<&str> {
        self.access_token.as_deref()
    }
}

/// Validate an access token.
///
/// This is a standalone function for token validation.
///
/// # Arguments
///
/// * `access_token` - The access token to validate
/// * `app_secret` - The app secret for the secret_key header
///
/// # Returns
///
/// * `Ok(bool)` - `true` if token is valid
/// * `Err(AuthError)` - An error if validation fails
pub async fn validate_access_token(
    access_token: &str,
    app_secret: &str,
) -> Result<bool> {
    let client = reqwest::Client::new();

    let response = client
        .post(VERIFY_ENDPOINT)
        .header("secret_key", app_secret)
        .header("access_token", access_token)
        .send()
        .await?;

    Ok(response.status().is_success())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zalo_auth_initialization() {
        let auth = ZaloAuth::new(
            "test_app_id".to_string(),
            "test_app_secret".to_string(),
            "test_refresh_token".to_string(),
        );

        assert_eq!(auth.app_id(), "test_app_id");
        assert_eq!(auth.refresh_token(), "test_refresh_token");
        assert!(auth.is_token_valid() == false);
    }

    #[test]
    fn test_zalo_auth_token_expiry() {
        let auth = ZaloAuth::new(
            "test_app_id".to_string(),
            "test_app_secret".to_string(),
            "test_refresh_token".to_string(),
        );

        // Initially no token
        assert!(!auth.is_token_valid());

        // Set a token that expires in the future
        let expiry = Instant::now() + StdDuration::from_secs(3600);
        let auth_with_expiry = ZaloAuth {
            app_id: "test_app_id".to_string(),
            app_secret: "test_app_secret".to_string(),
            access_token: Some("test_token".to_string()),
            refresh_token: "test_refresh_token".to_string(),
            token_expiry: Some(expiry),
        };

        assert!(auth_with_expiry.is_token_valid());
    }
}
