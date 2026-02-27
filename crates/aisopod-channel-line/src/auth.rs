//! LINE channel authentication and token management.
//!
//! This module provides functionality for managing LINE channel access tokens,
//! including issuing stateless tokens and token validation.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

/// LINE OAuth2 token endpoint.
pub const TOKEN_ENDPOINT: &str = "https://api.line.me/oauth2/v3/token";

/// Issue a stateless channel access token.
///
/// This method uses client credentials to issue a stateless token.
/// Stateless tokens are valid for 30 days and can be used to send messages
/// without requiring user authorization.
///
/// # Arguments
///
/// * `client_id` - The channel's client ID (also known as channel access token)
/// * `client_secret` - The channel's client secret
///
/// # Returns
///
/// * `Ok(String)` - The issued access token
/// * `Err(anyhow::Error)` - An error occurred
pub async fn issue_stateless_token(client_id: &str, client_secret: &str) -> Result<String> {
    let client = reqwest::Client::new();
    
    let response = client
        .post(TOKEN_ENDPOINT)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&[
            ("grant_type", "client_credentials"),
            ("client_id", client_id),
            ("client_secret", client_secret),
        ])
        .send()
        .await?;
    
    if response.status().is_success() {
        let token_response: TokenResponse = response.json().await?;
        debug!("Successfully issued stateless token");
        Ok(token_response.access_token)
    } else {
        let status = response.status().as_u16();
        let error_msg = response.text().await.unwrap_or_default();
        error!("Token issuance failed: {} - {}", status, error_msg);
        Err(anyhow::anyhow!(
            "Failed to issue stateless token: {}",
            error_msg
        ))
    }
}

/// Validate a channel access token.
///
/// # Arguments
///
/// * `access_token` - The access token to validate
///
/// # Returns
///
/// * `Ok(TokenValidation)` - Token validation result
/// * `Err(anyhow::Error)` - An error occurred
pub async fn validate_token(access_token: &str) -> Result<TokenValidation> {
    let client = reqwest::Client::new();
    
    let response = client
        .get("https://api.line.me/oauth2/v3/token")
        .bearer_auth(access_token)
        .send()
        .await?;
    
    if response.status().is_success() {
        let validation: TokenValidation = response.json().await?;
        Ok(validation)
    } else {
        let status = response.status().as_u16();
        let error_msg = response.text().await.unwrap_or_default();
        error!("Token validation failed: {} - {}", status, error_msg);
        Err(anyhow::anyhow!(
            "Failed to validate token: {}",
            error_msg
        ))
    }
}

/// Revoke a channel access token.
///
/// # Arguments
///
/// * `access_token` - The access token to revoke
///
/// # Returns
///
/// * `Ok(())` - Token revoked successfully
/// * `Err(anyhow::Error)` - An error occurred
pub async fn revoke_token(access_token: &str) -> Result<()> {
    let client = reqwest::Client::new();
    
    let response = client
        .post("https://api.line.me/oauth2/v3/revoke")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&[
            ("access_token", access_token),
        ])
        .send()
        .await?;
    
    if response.status().is_success() {
        Ok(())
    } else {
        let status = response.status().as_u16();
        let error_msg = response.text().await.unwrap_or_default();
        error!("Token revocation failed: {} - {}", status, error_msg);
        Err(anyhow::anyhow!(
            "Failed to revoke token: {}",
            error_msg
        ))
    }
}

/// Issue a stateful channel access token using authorization code.
///
/// This method is used in the OAuth2 authorization code flow to exchange
/// an authorization code for an access token and refresh token.
///
/// # Arguments
///
/// * `client_id` - The channel's client ID
/// * `client_secret` - The channel's client secret
/// * `code` - The authorization code received from LINE
/// * `redirect_uri` - The redirect URI used in the authorization request
///
/// # Returns
///
/// * `Ok(TokenExchangeResponse)` - Token exchange result
/// * `Err(anyhow::Error)` - An error occurred
pub async fn issue_stateful_token(
    client_id: &str,
    client_secret: &str,
    code: &str,
    redirect_uri: &str,
) -> Result<TokenExchangeResponse> {
    let client = reqwest::Client::new();
    
    let response = client
        .post(TOKEN_ENDPOINT)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&[
            ("grant_type", "authorization_code"),
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("code", code),
            ("redirect_uri", redirect_uri),
        ])
        .send()
        .await?;
    
    if response.status().is_success() {
        let response: TokenExchangeResponse = response.json().await?;
        Ok(response)
    } else {
        let status = response.status().as_u16();
        let error_msg = response.text().await.unwrap_or_default();
        error!("Token exchange failed: {} - {}", status, error_msg);
        Err(anyhow::anyhow!(
            "Failed to exchange token: {}",
            error_msg
        ))
    }
}

/// Refresh a stateful channel access token using refresh token.
///
/// # Arguments
///
/// * `client_id` - The channel's client ID
/// * `client_secret` - The channel's client secret
/// * `refresh_token` - The refresh token
///
/// # Returns
///
/// * `Ok(TokenExchangeResponse)` - Token refresh result
/// * `Err(anyhow::Error)` - An error occurred
pub async fn refresh_token(
    client_id: &str,
    client_secret: &str,
    refresh_token: &str,
) -> Result<TokenExchangeResponse> {
    let client = reqwest::Client::new();
    
    let response = client
        .post(TOKEN_ENDPOINT)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&[
            ("grant_type", "refresh_token"),
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("refresh_token", refresh_token),
        ])
        .send()
        .await?;
    
    if response.status().is_success() {
        let response: TokenExchangeResponse = response.json().await?;
        Ok(response)
    } else {
        let status = response.status().as_u16();
        let error_msg = response.text().await.unwrap_or_default();
        error!("Token refresh failed: {} - {}", status, error_msg);
        Err(anyhow::anyhow!(
            "Failed to refresh token: {}",
            error_msg
        ))
    }
}

/// Token response from LINE OAuth2.
#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    #[serde(rename = "token_type")]
    pub token_type: String,
    #[serde(rename = "expires_in")]
    pub expires_in: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

/// Token exchange response from LINE OAuth2.
#[derive(Debug, Deserialize)]
pub struct TokenExchangeResponse {
    pub access_token: String,
    #[serde(rename = "token_type")]
    pub token_type: String,
    #[serde(rename = "expires_in")]
    pub expires_in: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

/// Token validation response from LINE.
#[derive(Debug, Deserialize)]
pub struct TokenValidation {
    #[serde(rename = "client_id")]
    pub client_id: String,
    #[serde(rename = "expires_in")]
    pub expires_in: u64,
    pub scope: Option<String>,
    #[serde(rename = "expires_at")]
    pub expires_at: u64,
}

/// Check if a token is expired or about to expire.
///
/// # Arguments
///
/// * `token_validation` - The token validation result
///
/// # Returns
///
/// * `true` - Token is expired or expiring soon
/// * `false` - Token is still valid
pub fn is_token_expired(token_validation: &TokenValidation) -> bool {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // Consider token expired if it will expire within the next 5 minutes
    now + 300 >= token_validation.expires_at
}

/// Token manager for automatic token refresh.
#[derive(Clone)]
pub struct TokenManager {
    access_token: String,
    refresh_token: Option<String>,
    client_id: String,
    client_secret: String,
    expires_at: u64,
}

impl TokenManager {
    /// Create a new TokenManager with an initial token.
    pub fn new(
        access_token: String,
        refresh_token: Option<String>,
        client_id: String,
        client_secret: String,
        expires_at: u64,
    ) -> Self {
        Self {
            access_token,
            refresh_token,
            client_id,
            client_secret,
            expires_at,
        }
    }

    /// Get the current access token.
    pub fn access_token(&self) -> &str {
        &self.access_token
    }

    /// Check if the token needs to be refreshed.
    pub fn needs_refresh(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Refresh if token will expire within the next 5 minutes
        now + 300 >= self.expires_at
    }

    /// Refresh the access token if needed.
    pub async fn refresh_if_needed(&mut self) -> Result<&str> {
        if self.needs_refresh() {
            if let Some(refresh_token_val) = &self.refresh_token {
                let response = crate::auth::refresh_token(
                    &self.client_id,
                    &self.client_secret,
                    refresh_token_val,
                ).await?;
                
                self.access_token = response.access_token;
                self.expires_at = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() + response.expires_in;
                
                // Update refresh token if provided
                self.refresh_token = response.refresh_token;
                
                debug!("Token refreshed successfully");
            } else {
                return Err(anyhow::anyhow!("No refresh token available"));
            }
        }
        
        Ok(&self.access_token)
    }
}
