//! Authentication and token management for Lark/Feishu.
//!
//! This module provides the LarkAuth struct for managing tenant access tokens.
//! Lark uses a token-based authentication system where apps must exchange
//! app_id and app_secret for a tenant_access_token.

use anyhow::{anyhow, Result};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// Error type for authentication operations.
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Failed to get access token: {0}")]
    TokenRequestFailed(String),
    #[error("Token expired")]
    TokenExpired,
    #[error("Invalid response from API: {0}")]
    InvalidResponse(String),
}

/// Lark authentication manager.
///
/// This struct manages the tenant access token required for API calls.
/// It handles token retrieval, caching, and automatic refresh when expired.
pub struct LarkAuth {
    /// App ID from Lark Developer Console
    app_id: String,
    /// App Secret from Lark Developer Console
    app_secret: String,
    /// Base URL for the API (Lark or Feishu)
    base_url: String,
    /// Cached tenant access token
    tenant_access_token: Option<String>,
    /// Token expiration time
    token_expiry: Option<chrono::DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    tenant_access_token: String,
    expire: i64,
}

impl LarkAuth {
    /// Creates a new LarkAuth instance.
    ///
    /// # Arguments
    ///
    /// * `app_id` - The app ID from Lark Developer Console
    /// * `app_secret` - The app secret from Lark Developer Console
    /// * `use_feishu` - Whether to use Feishu domain (for China region)
    pub fn new(app_id: String, app_secret: String, use_feishu: bool) -> Self {
        let base_url = if use_feishu {
            "https://open.feishu.cn".to_string()
        } else {
            "https://open.larksuite.com".to_string()
        };

        Self {
            app_id,
            app_secret,
            base_url,
            tenant_access_token: None,
            token_expiry: None,
        }
    }

    /// Returns the base URL for the API.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Checks if the current token is valid and not expired.
    fn is_token_valid(&self) -> bool {
        if let Some(expiry) = self.token_expiry {
            // Refresh token 5 minutes before expiry
            let now = Utc::now();
            let buffer = Duration::minutes(5);
            now + buffer < expiry
        } else {
            false
        }
    }

    /// Gets a valid tenant access token.
    ///
    /// If a valid token exists, it's returned immediately.
    /// Otherwise, a new token is requested from the Lark API.
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The tenant access token
    /// * `Err(anyhow::Error)` - An error if token retrieval fails
    pub async fn get_tenant_access_token(&mut self) -> Result<String> {
        if self.is_token_valid() {
            debug!("Using cached tenant access token");
            return Ok(self.tenant_access_token.clone().unwrap());
        }

        info!("Requesting new tenant access token");
        let token = self.refresh_token().await?;
        Ok(token)
    }

    /// Refreshes the tenant access token from the Lark API.
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The new tenant access token
    /// * `Err(anyhow::Error)` - An error if the request fails
    async fn refresh_token(&mut self) -> Result<String> {
        let url = format!("{}/open-apis/auth/v3/tenant_access_token/internal", self.base_url);

        let client = reqwest::Client::new();
        let body = serde_json::json!({
            "app_id": self.app_id,
            "app_secret": self.app_secret
        });

        let response = client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| AuthError::TokenRequestFailed(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            warn!("Token request failed: {} - {}", status, body);
            return Err(anyhow::anyhow!("Token request failed: HTTP {}: {}", status, body));
        }

        let token_response: TokenResponse = response.json().await.map_err(|e| {
            AuthError::InvalidResponse(format!("Failed to parse token response: {}", e))
        })?;

        // Store the token with expiry
        let expire_duration = Duration::seconds(token_response.expire);
        self.tenant_access_token = Some(token_response.tenant_access_token.clone());
        self.token_expiry = Some(Utc::now() + expire_duration - Duration::minutes(5));

        debug!(
            "Token refreshed, expires at: {:?}",
            self.token_expiry
        );

        Ok(token_response.tenant_access_token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_lark_auth() {
        let auth = LarkAuth::new(
            "test_app_id".to_string(),
            "test_app_secret".to_string(),
            false,
        );
        assert_eq!(auth.base_url(), "https://open.larksuite.com");
        assert!(auth.tenant_access_token.is_none());
    }

    #[test]
    fn test_new_feishu_auth() {
        let auth = LarkAuth::new(
            "test_app_id".to_string(),
            "test_app_secret".to_string(),
            true,
        );
        assert_eq!(auth.base_url(), "https://open.feishu.cn");
    }

    #[test]
    fn test_is_token_valid_no_token() {
        let auth = LarkAuth::new(
            "test_app_id".to_string(),
            "test_app_secret".to_string(),
            false,
        );
        assert!(!auth.is_token_valid());
    }
}
