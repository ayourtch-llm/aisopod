//! Azure AD authentication for Microsoft Teams.
//!
//! This module provides authentication functionality using Azure AD with the
//! client_credentials grant type for Bot Framework API access.

use anyhow::Result;
use base64::Engine;
use chrono::{DateTime, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

/// Azure AD authentication token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MsTeamsAuthToken {
    /// The access token
    pub token: String,
    /// Token type (typically "Bearer")
    pub token_type: String,
    /// Expiration timestamp
    pub expires_at: DateTime<Utc>,
    /// The scope of the token
    pub scope: String,
}

impl MsTeamsAuthToken {
    /// Check if the token is expired.
    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expires_at
    }

    /// Check if the token is about to expire (within 5 minutes).
    pub fn is_about_to_expire(&self) -> bool {
        let now = Utc::now();
        let margin = chrono::Duration::minutes(5);
        now + margin >= self.expires_at
    }

    /// Get remaining time until expiration.
    pub fn time_until_expiration(&self) -> Duration {
        self.expires_at
            .signed_duration_since(Utc::now())
            .to_std()
            .unwrap_or_default()
    }
}

/// Azure AD authentication configuration.
#[derive(Debug, Clone)]
pub struct AzureAuthConfig {
    /// Azure AD tenant ID
    pub tenant_id: String,
    /// Azure AD client ID
    pub client_id: String,
    /// Azure AD client secret
    pub client_secret: String,
    /// Resource or scope to request (default: bot framework)
    pub resource: Option<String>,
}

impl AzureAuthConfig {
    /// Creates a new Azure AD authentication configuration.
    pub fn new(tenant_id: &str, client_id: &str, client_secret: &str) -> Self {
        Self {
            tenant_id: tenant_id.to_string(),
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            resource: None,
        }
    }

    /// Creates a new Azure AD authentication configuration with custom resource.
    pub fn with_resource(
        tenant_id: &str,
        client_id: &str,
        client_secret: &str,
        resource: &str,
    ) -> Self {
        Self {
            tenant_id: tenant_id.to_string(),
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            resource: Some(resource.to_string()),
        }
    }
}

/// Microsoft Bot Framework authentication manager.
#[derive(Debug, Clone)]
pub struct MsTeamsAuth {
    /// Authentication configuration
    config: AzureAuthConfig,
    /// Cached token
    cached_token: Option<MsTeamsAuthToken>,
    /// Time when the token was cached
    token_cache_time: Option<Instant>,
}

impl MsTeamsAuth {
    /// Creates a new Microsoft Teams authentication manager.
    pub fn new(config: AzureAuthConfig) -> Self {
        Self {
            config,
            cached_token: None,
            token_cache_time: None,
        }
    }

    /// Creates a new Microsoft Teams authentication manager from account config.
    pub fn from_account_config(config: &crate::config::MsTeamsAccountConfig) -> Self {
        let azure_config =
            AzureAuthConfig::new(&config.tenant_id, &config.client_id, &config.client_secret);
        Self::new(azure_config)
    }

    /// Get or refresh the access token.
    pub async fn get_token(&mut self) -> Result<MsTeamsAuthToken> {
        // Check if we have a valid cached token
        if let Some((token, cache_time)) = self
            .cached_token
            .as_ref()
            .zip(self.token_cache_time.as_ref())
        {
            // Refresh token if it's about to expire or if more than 45 minutes have passed
            if !token.is_about_to_expire() && cache_time.elapsed() < Duration::from_secs(2700) {
                debug!("Using cached token");
                return Ok(token.clone());
            }
        }

        info!("Fetching new access token from Azure AD");
        self.refresh_token().await
    }

    /// Refresh the access token.
    pub async fn refresh_token(&mut self) -> Result<MsTeamsAuthToken> {
        let token = self.fetch_access_token().await?;
        self.cached_token = Some(token.clone());
        self.token_cache_time = Some(Instant::now());
        Ok(token)
    }

    /// Fetches a new access token from Azure AD.
    async fn fetch_access_token(&self) -> Result<MsTeamsAuthToken> {
        let client = reqwest::Client::new();

        // Determine the resource/scope
        let resource = self
            .config
            .resource
            .as_deref()
            .unwrap_or("https://api.botframework.com");
        let scope = format!("{}/.default", resource);

        // Build the token request
        let form_data = [
            ("grant_type", "client_credentials"),
            ("client_id", &self.config.client_id),
            ("client_secret", &self.config.client_secret),
            ("scope", &scope),
        ];

        let url = format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
            self.config.tenant_id
        );

        debug!("Requesting token from {}", url);

        let response = client.post(&url).form(&form_data).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Token request failed with status {}: {}", status, body);
            return Err(anyhow::anyhow!("Token request failed: {}", status));
        }

        let token_response: TokenResponse = response.json().await?;

        let expires_at = Utc::now() + chrono::Duration::seconds(token_response.expires_in as i64);

        Ok(MsTeamsAuthToken {
            token: token_response.access_token,
            token_type: token_response.token_type,
            expires_at,
            scope: token_response.scope,
        })
    }

    /// Creates a bot framework JWT token for outbound activities.
    /// This is used when the bot needs to send activities to users.
    pub fn create_bot_token(&self, app_id: &str) -> Result<String> {
        let now = Utc::now();
        let expiration = now + chrono::Duration::hours(24);

        let claims = BotClaims {
            aud: "botframework.com".to_string(),
            iss: app_id.to_string(),
            sub: app_id.to_string(),
            jti: uuid::Uuid::new_v4().to_string(),
            nbf: now.timestamp() as usize,
            exp: expiration.timestamp() as usize,
        };

        let header = Header::new(jsonwebtoken::Algorithm::HS256);
        let encoding_key = EncodingKey::from_secret(app_id.as_bytes());

        encode(&header, &claims, &encoding_key)
            .map_err(|e| anyhow::anyhow!("Failed to encode bot token: {}", e))
    }

    /// Validates a bot framework JWT token from incoming activities.
    pub fn validate_bot_token(&self, token: &str, app_id: &str) -> Result<BotClaims> {
        let decoding_key = DecodingKey::from_secret(app_id.as_bytes());
        let validation = Validation::new(jsonwebtoken::Algorithm::HS256);

        let decoded = decode::<BotClaims>(token, &decoding_key, &validation)
            .map_err(|e| anyhow::anyhow!("Failed to decode bot token: {}", e))?;

        Ok(decoded.claims)
    }
}

/// Token response from Azure AD.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TokenResponse {
    /// Access token
    access_token: String,
    /// Token type
    token_type: String,
    /// Expiration time in seconds
    expires_in: u64,
    /// Scope of the token
    scope: String,
}

/// Claims for bot framework JWT tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct BotClaims {
    /// Audience
    aud: String,
    /// Issuer
    iss: String,
    /// Subject
    sub: String,
    /// JWT ID
    jti: String,
    /// Not before timestamp
    nbf: usize,
    /// Expiration timestamp
    exp: usize,
}

/// Bot Framework token validation utilities.
pub mod bot_token {
    use super::*;

    /// Decodes a Bot Framework JWT token without validation.
    /// Use this only for inspection/debugging purposes.
    pub fn decode_inspect(token: &str) -> Result<serde_json::Value> {
        // JWT tokens have 3 parts separated by dots
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(anyhow::anyhow!("Invalid JWT token format"));
        }

        // Decode the payload (second part)
        let payload = parts[1];
        // Add padding if needed
        let padding = payload.len() % 4;
        let padded_payload = if padding > 0 {
            format!("{}{}", payload, "====".get(0..(4 - padding)).unwrap_or(""))
        } else {
            payload.to_string()
        };

        let decoded = base64::engine::general_purpose::STANDARD
            .decode(padded_payload)
            .map_err(|e| anyhow::anyhow!("Failed to decode base64: {}", e))?;

        let claims: serde_json::Value = serde_json::from_slice(&decoded)
            .map_err(|e| anyhow::anyhow!("Failed to parse JSON: {}", e))?;

        Ok(claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_config_creation() {
        let config = AzureAuthConfig::new("tenant123", "client123", "secret123");
        assert_eq!(config.tenant_id, "tenant123");
        assert_eq!(config.client_id, "client123");
        assert_eq!(config.client_secret, "secret123");
        assert!(config.resource.is_none());
    }

    #[test]
    fn test_auth_config_with_resource() {
        let config = AzureAuthConfig::with_resource(
            "tenant123",
            "client123",
            "secret123",
            "custom-resource",
        );
        assert_eq!(config.resource, Some("custom-resource".to_string()));
    }

    #[test]
    fn test_token_expiry() {
        let now = Utc::now();
        let token = MsTeamsAuthToken {
            token: "test_token".to_string(),
            token_type: "Bearer".to_string(),
            expires_at: now + chrono::Duration::minutes(10),
            scope: "test".to_string(),
        };

        assert!(!token.is_expired());
        assert!(!token.is_about_to_expire());

        let old_token = MsTeamsAuthToken {
            token: "test_token".to_string(),
            token_type: "Bearer".to_string(),
            expires_at: now - chrono::Duration::minutes(10),
            scope: "test".to_string(),
        };

        assert!(old_token.is_expired());
    }

    #[test]
    fn test_auth_from_account_config() {
        let account_config =
            crate::config::MsTeamsAccountConfig::new("test", "tenant123", "client123", "secret123");
        let auth = MsTeamsAuth::from_account_config(&account_config);

        assert_eq!(auth.config.tenant_id, "tenant123");
        assert_eq!(auth.config.client_id, "client123");
        assert_eq!(auth.config.client_secret, "secret123");
    }
}
