//! Google Chat authentication support.
//!
//! This module provides authentication for Google Chat API using both
//! OAuth 2.0 and Service Account authentication methods.

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::{Arc, Mutex};
use tracing::{debug, error, info};

use crate::config::{OAuth2Config as OAuth2ConfigType, ServiceAccountConfig as ServiceAccountConfigType};

/// Authentication token for Google Chat API access.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleChatAuthToken {
    /// The access token string
    pub token: String,
    /// Token type (typically "Bearer")
    #[serde(default = "default_token_type")]
    pub token_type: String,
    /// Optional expiration timestamp
    pub expires_at: Option<DateTime<Utc>>,
    /// Optional refresh token for OAuth 2.0
    pub refresh_token: Option<String>,
}

fn default_token_type() -> String {
    "Bearer".to_string()
}

impl Default for GoogleChatAuthToken {
    fn default() -> Self {
        Self {
            token: String::new(),
            token_type: default_token_type(),
            expires_at: None,
            refresh_token: None,
        }
    }
}

/// Google Chat API authentication interface.
#[async_trait]
pub trait GoogleChatAuth: Send + Sync {
    /// Get the current access token.
    async fn get_token(&self) -> Result<GoogleChatAuthToken>;

    /// Refresh the access token if expired.
    async fn refresh_token(&self, token: &GoogleChatAuthToken) -> Result<GoogleChatAuthToken>;

    /// Check if the token is expired.
    fn is_token_expired(&self, token: &GoogleChatAuthToken) -> bool;
}

/// OAuth 2.0 authentication for Google Chat API.
pub struct OAuth2Auth {
    /// OAuth 2.0 configuration
    config: OAuth2ConfigType,
    /// Cached access token
    cached_token: Arc<Mutex<Option<GoogleChatAuthToken>>>,
}

impl OAuth2Auth {
    /// Create a new OAuth 2.0 authenticator.
    pub fn new(config: OAuth2ConfigType) -> Self {
        Self {
            config,
            cached_token: Arc::new(Mutex::new(None)),
        }
    }

    /// Get the token URI from the configuration.
    fn token_uri(&self) -> &str {
        &self.config.token_uri
    }
}

#[async_trait]
impl GoogleChatAuth for OAuth2Auth {
    async fn get_token(&self) -> Result<GoogleChatAuthToken> {
        // Check if we have a valid cached token
        if let Some(ref token) = self.cached_token.lock().unwrap().as_ref() {
            if !self.is_token_expired(token) {
                debug!("Using cached OAuth 2.0 token");
                let token_clone = (**token).clone();
                return Ok(token_clone);
            }
            info!("OAuth 2.0 token expired, refreshing...");
        }

        // Refresh the token
        self.refresh_token(&GoogleChatAuthToken {
            refresh_token: Some(self.config.refresh_token.clone()),
            ..Default::default()
        }).await
    }

    async fn refresh_token(&self, token: &GoogleChatAuthToken) -> Result<GoogleChatAuthToken> {
        let refresh_token = token.refresh_token.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No refresh token available"))?;

        // Prepare the token refresh request
        let client = reqwest::Client::new();
        let response = client
            .post(self.token_uri())
            .form(&[
                ("grant_type", "refresh_token"),
                ("client_id", &self.config.client_id),
                ("client_secret", &self.config.client_secret),
                ("refresh_token", refresh_token),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("Token refresh failed: {}", error_text);
            return Err(anyhow::anyhow!("Token refresh failed: {}", error_text));
        }

        // Parse the response
        let token_response: TokenResponse = response.json().await?;

        let new_token = GoogleChatAuthToken {
            token: token_response.access_token,
            token_type: token_response.token_type,
            expires_at: Some(Utc::now() + chrono::Duration::seconds(token_response.expires_in)),
            refresh_token: Some(refresh_token.to_string()),
        };

        // Cache the new token
        *self.cached_token.lock().unwrap() = Some(new_token.clone());

        info!("OAuth 2.0 token refreshed successfully");
        Ok(new_token)
    }

    fn is_token_expired(&self, token: &GoogleChatAuthToken) -> bool {
        if let Some(expires_at) = token.expires_at {
            expires_at <= Utc::now() + chrono::Duration::seconds(60) // 60 seconds buffer
        } else {
            false
        }
    }
}

/// Service Account authentication for Google Chat API.
pub struct ServiceAccountAuth {
    /// Service account configuration
    config: ServiceAccountConfigType,
    /// Cached access token
    cached_token: Arc<Mutex<Option<GoogleChatAuthToken>>>,
}

impl ServiceAccountAuth {
    /// Create a new Service Account authenticator.
    pub fn new(config: ServiceAccountConfigType) -> Result<Self> {
        // Validate the private key format
        if config.private_key.trim().is_empty() {
            return Err(anyhow::anyhow!("Private key cannot be empty"));
        }

        // Validate the client email
        if config.client_email.trim().is_empty() {
            return Err(anyhow::anyhow!("Client email cannot be empty"));
        }

        Ok(Self {
            config,
            cached_token: Arc::new(Mutex::new(None)),
        })
    }

    /// Load service account configuration from a JSON key file.
    pub fn from_key_file(key_file: &str) -> Result<Self> {
        let content = fs::read_to_string(key_file)
            .map_err(|e| anyhow::anyhow!("Failed to read key file '{}': {}", key_file, e))?;

        let key: ServiceAccountKeyFile = serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Invalid key file format: {}", e))?;

        Ok(Self::new(ServiceAccountConfigType {
            key_file: key_file.to_string(),
            client_email: key.client_email,
            private_key: key.private_key,
            project_number: key.project_id,
        })?)
    }
}

/// Service account key file structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ServiceAccountKeyFile {
    #[serde(rename = "type")]
    key_type: String,
    #[serde(rename = "project_id")]
    project_id: Option<String>,
    #[serde(rename = "private_key_id")]
    private_key_id: String,
    #[serde(rename = "private_key")]
    private_key: String,
    #[serde(rename = "client_email")]
    client_email: String,
    #[serde(rename = "client_id")]
    client_id: String,
    #[serde(rename = "auth_uri")]
    auth_uri: String,
    #[serde(rename = "token_uri")]
    token_uri: String,
    #[serde(rename = "auth_provider_x509_cert_url")]
    auth_provider_x509_cert_url: String,
    #[serde(rename = "client_x509_cert_url")]
    client_x509_cert_url: String,
}

#[async_trait]
impl GoogleChatAuth for ServiceAccountAuth {
    async fn get_token(&self) -> Result<GoogleChatAuthToken> {
        // Check if we have a valid cached token
        if let Some(ref token) = self.cached_token.lock().unwrap().as_ref() {
            if !self.is_token_expired(token) {
                debug!("Using cached Service Account token");
                let token_clone = (**token).clone();
                return Ok(token_clone);
            }
            info!("Service Account token expired, refreshing...");
        }

        // Generate a new JWT token
        self.generate_jwt_token().await
    }

    async fn refresh_token(&self, token: &GoogleChatAuthToken) -> Result<GoogleChatAuthToken> {
        // For service accounts, we regenerate the JWT token on each refresh
        self.generate_jwt_token().await
    }

    fn is_token_expired(&self, token: &GoogleChatAuthToken) -> bool {
        if let Some(expires_at) = token.expires_at {
            expires_at <= Utc::now() + chrono::Duration::seconds(60) // 60 seconds buffer
        } else {
            false
        }
    }
}

impl ServiceAccountAuth {
    /// Generate a JWT token for service account authentication.
    async fn generate_jwt_token(&self) -> Result<GoogleChatAuthToken> {
        use jsonwebtoken::{encode, EncodingKey, Header};
        use jsonwebtoken::Algorithm::RS256;

        // Create the JWT claims
        let now = Utc::now().timestamp();
        let claims = JwtClaims {
            iss: self.config.client_email.clone(),
            scope: "https://chat.googleapis.com/".to_string(),
            aud: "https://oauth2.googleapis.com/token".to_string(),
            iat: now,
            exp: now + 3600, // 1 hour expiration
        };

        // Encode the JWT
        let encoding_key = EncodingKey::from_rsa_pem(self.config.private_key.as_bytes())
            .map_err(|e| anyhow::anyhow!("Failed to create encoding key: {}", e))?;

        let token = encode(&Header::new(RS256), &claims, &encoding_key)
            .map_err(|e| anyhow::anyhow!("Failed to encode JWT: {}", e))?;

        // Exchange JWT for access token
        let client = reqwest::Client::new();
        let response = client
            .post("https://oauth2.googleapis.com/token")
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
                ("assertion", &token),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("JWT token exchange failed: {}", error_text);
            return Err(anyhow::anyhow!("JWT token exchange failed: {}", error_text));
        }

        // Parse the response
        let token_response: TokenResponse = response.json().await?;

        let new_token = GoogleChatAuthToken {
            token: token_response.access_token,
            token_type: token_response.token_type,
            expires_at: Some(Utc::now() + chrono::Duration::seconds(token_response.expires_in)),
            refresh_token: None,
        };

        // Cache the new token
        *self.cached_token.lock().unwrap() = Some(new_token.clone());

        info!("Service Account token generated successfully");
        Ok(new_token)
    }
}

/// JWT claims for service account authentication.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct JwtClaims {
    iss: String,      // Issuer
    scope: String,    // Scopes
    aud: String,      // Audience
    iat: i64,         // Issued at
    exp: i64,         // Expiration time
}

/// Token response from Google OAuth 2.0 endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TokenResponse {
    access_token: String,
    #[serde(rename = "token_type")]
    token_type: String,
    expires_in: i64,
    #[serde(default)]
    scope: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_google_chat_auth_token_default() {
        let token = GoogleChatAuthToken::default();
        assert!(token.token.is_empty());
        assert_eq!(token.token_type, "Bearer");
        assert!(token.expires_at.is_none());
        assert!(token.refresh_token.is_none());
    }

    #[test]
    fn test_oauth2_config_default() {
        let config = OAuth2ConfigType::default();
        assert!(config.client_id.is_empty());
        assert!(config.client_secret.is_empty());
        assert!(config.refresh_token.is_empty());
        assert_eq!(config.token_uri, "https://oauth2.googleapis.com/token");
        assert!(config.project_number.is_none());
    }

    #[test]
    fn test_service_account_config_default() {
        let config = ServiceAccountConfigType::default();
        assert!(config.key_file.is_empty());
        assert!(config.client_email.is_empty());
        assert!(config.private_key.is_empty());
        assert!(config.project_number.is_none());
    }

    #[test]
    fn test_oauth2_auth_new() {
        let config = OAuth2ConfigType {
            client_id: "test-client-id".to_string(),
            client_secret: "test-client-secret".to_string(),
            refresh_token: "test-refresh-token".to_string(),
            ..Default::default()
        };

        let auth = OAuth2Auth::new(config);
        // Just verify creation succeeds
        assert!(true); // If we got here, creation succeeded
    }

    #[test]
    fn test_service_account_config_from_key_file() {
        // Create a temporary key file
        let temp_dir = std::env::temp_dir();
        let key_file = temp_dir.join("test-key.json");
        let key_content = r#"
        {
            "type": "service_account",
            "project_id": "test-project",
            "private_key_id": "test-key-id",
            "private_key": "-----BEGIN PRIVATE KEY-----\nMIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQC7W0XcVw3t\n-----END PRIVATE KEY-----\n",
            "client_email": "test@test.iam.gserviceaccount.com",
            "client_id": "123456789",
            "auth_uri": "https://accounts.google.com/o/oauth2/auth",
            "token_uri": "https://oauth2.googleapis.com/token",
            "auth_provider_x509_cert_url": "https://www.googleapis.com/oauth2/v1/certs",
            "client_x509_cert_url": "https://www.googleapis.com/robot/v1/metadata/x509/test%40test.iam.gserviceaccount.com"
        }
        "#;

        std::fs::write(&key_file, key_content).unwrap();

        let result = ServiceAccountAuth::from_key_file(key_file.to_str().unwrap());

        // Clean up
        let _ = std::fs::remove_file(&key_file);

        // Just verify the function exists and can be called
        // Actual parsing would fail without a valid key, but we're testing structure
        assert!(result.is_ok());
    }
}

// Re-export config types
pub use crate::config::OAuth2Config;
pub use crate::config::ServiceAccountConfig;
