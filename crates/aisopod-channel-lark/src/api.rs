//! API client for Lark/Feishu Open Platform.
//!
//! This module provides the LarkApi struct for making API calls to the Lark API,
//! including sending messages, message cards, and other operations.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::auth::LarkAuth;

/// Error type for API operations.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Failed to send message: {0}")]
    SendMessageFailed(String),
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Auth error: {0}")]
    AuthError(#[from] anyhow::Error),
}

/// API response error.
#[derive(Debug, Deserialize)]
struct ApiErrorResponse {
    code: i64,
    msg: String,
}

/// API client for Lark/Feishu.
///
/// This struct provides methods for interacting with the Lark Open Platform API,
/// including sending text messages, message cards, and other operations.
pub struct LarkApi {
    /// Authentication manager
    auth: LarkAuth,
    /// HTTP client
    http: reqwest::Client,
    /// Base URL for the API
    base_url: String,
}

impl LarkApi {
    /// Creates a new LarkApi instance.
    ///
    /// # Arguments
    ///
    /// * `auth` - The LarkAuth instance with credentials
    pub fn new(auth: LarkAuth) -> Self {
        let base_url = auth.base_url().to_string();
        Self {
            auth,
            http: reqwest::Client::new(),
            base_url,
        }
    }

    /// Returns the base URL for the API.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Sends a message to a Lark chat.
    ///
    /// # Arguments
    ///
    /// * `receive_id` - The ID of the recipient (chat_id, open_id, etc.)
    /// * `receive_id_type` - The type of receive_id: "open_id", "chat_id", "user_id", "email"
    /// * `msg_type` - The message type: "text", "post", "image", "interactive", etc.
    /// * `content` - The message content (JSON string for complex types)
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message was sent successfully
    /// * `Err(ApiError)` - An error occurred
    pub async fn send_message(
        &mut self,
        receive_id: &str,
        receive_id_type: &str,
        msg_type: &str,
        content: &str,
    ) -> Result<(), ApiError> {
        let token = self.auth.get_tenant_access_token().await?;

        let url = format!(
            "{}/open-apis/im/v1/messages?receive_id_type={}",
            self.base_url, receive_id_type
        );

        let body = serde_json::json!({
            "receive_id": receive_id,
            "msg_type": msg_type,
            "content": content
        });

        debug!("Sending message to {}", receive_id);

        let response = self
            .http
            .post(&url)
            .bearer_auth(token)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            warn!("Message send failed: {} - {}", status, body);

            // Try to parse error response
            let api_error: Option<ApiErrorResponse> = serde_json::from_str(&body).ok();
            if let Some(err) = api_error {
                return Err(ApiError::SendMessageFailed(format!(
                    "API error {}: {}",
                    err.code, err.msg
                )));
            }

            return Err(ApiError::SendMessageFailed(format!(
                "HTTP {}: {}",
                status, body
            )));
        }

        info!("Message sent successfully to {}", receive_id);
        Ok(())
    }

    /// Sends a text message to a chat.
    ///
    /// # Arguments
    ///
    /// * `chat_id` - The chat ID to send to
    /// * `text` - The text content to send
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message was sent successfully
    /// * `Err(ApiError)` - An error occurred
    pub async fn send_text(&mut self, chat_id: &str, text: &str) -> Result<(), ApiError> {
        let content = serde_json::json!({ "text": text }).to_string();
        self.send_message(chat_id, "chat_id", "text", &content).await
    }

    /// Sends a rich message card to a chat.
    ///
    /// # Arguments
    ///
    /// * `chat_id` - The chat ID to send to
    /// * `card` - The message card as JSON
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message was sent successfully
    /// * `Err(ApiError)` - An error occurred
    pub async fn send_card(&mut self, chat_id: &str, card: serde_json::Value) -> Result<(), ApiError> {
        let content = card.to_string();
        self.send_message(chat_id, "chat_id", "interactive", &content).await
    }

    /// Gets user profile information.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID (open_id or user_id)
    /// * `user_id_type` - The type of user_id: "open_id", "user_id", "union_id", "email"
    ///
    /// # Returns
    ///
    /// * `Ok(UserProfile)` - User profile information
    /// * `Err(ApiError)` - An error occurred
    pub async fn get_user_profile(
        &mut self,
        user_id: &str,
        user_id_type: &str,
    ) -> Result<UserProfile, ApiError> {
        let token = self.auth.get_tenant_access_token().await?;

        let url = format!(
            "{}/open-apis/contact/v3/users/{}?user_id_type={}",
            self.base_url, user_id, user_id_type
        );

        let response = self.http.get(&url).bearer_auth(token).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            warn!("Get user profile failed: {} - {}", status, body);
            return Err(ApiError::SendMessageFailed(format!(
                "HTTP {}: {}",
                status, body
            )));
        }

        let user_profile: UserProfile = response.json().await?;
        Ok(user_profile)
    }
}

/// User profile information from Lark API.
#[derive(Debug, Deserialize)]
pub struct UserProfile {
    pub user_id: String,
    pub open_id: String,
    pub union_id: Option<String>,
    pub name: String,
    pub email: Option<String>,
    pub avatar: Option<String>,
    pub mobile: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lark_api_new() {
        let auth = crate::auth::LarkAuth::new(
            "test_app_id".to_string(),
            "test_app_secret".to_string(),
            false,
        );
        let api = LarkApi::new(auth);
        assert_eq!(api.base_url(), "https://open.larksuite.com");
    }
}
