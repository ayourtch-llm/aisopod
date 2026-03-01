//! Zalo OA API client.
//!
//! This module provides the Zalo OA API client for sending messages,
//! handling media attachments, and interacting with the Zalo platform.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

use crate::auth::{validate_access_token, ZaloAuth};

/// Base URL for Zalo OA API.
pub const BASE_URL: &str = "https://openapi.zalo.me/v3.0/oa";

/// Zalo message types.
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum MessageType {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image {
        #[serde(rename = "url")]
        url: String,
    },
    #[serde(rename = "file")]
    File {
        #[serde(rename = "token")]
        token: String,
    },
}

/// Message payload for Zalo API.
#[derive(Debug, Serialize)]
pub struct MessagePayload {
    /// Recipient information
    pub recipient: Recipient,
    /// Message content
    pub message: MessageContent,
}

/// Recipient of a message.
#[derive(Debug, Serialize)]
pub struct Recipient {
    /// User ID
    #[serde(rename = "user_id")]
    pub user_id: String,
}

/// Message content wrapper.
#[derive(Debug, Serialize)]
pub struct MessageContent {
    /// The message attachment/structure
    pub attachment: Attachment,
}

/// Message attachment.
#[derive(Debug, Serialize)]
pub struct Attachment {
    /// The type of attachment
    #[serde(rename = "type")]
    pub type_field: String,
    /// The payload
    pub payload: serde_json::Value,
}

/// Zalo API client for sending messages.
///
/// This struct provides methods for sending text, image, and file messages
/// to Zalo users via the OA API.
#[derive(Clone, Debug)]
pub struct ZaloApi {
    /// Authentication manager
    auth: ZaloAuth,
    /// HTTP client
    http: reqwest::Client,
    /// Base URL for the Zalo OA API
    base_url: String,
}

impl ZaloApi {
    /// Create a new ZaloApi instance.
    ///
    /// # Arguments
    ///
    /// * `auth` - The authentication manager
    ///
    /// # Returns
    ///
    /// * `Ok(ZaloApi)` - The API client
    pub fn new(auth: ZaloAuth) -> Self {
        Self {
            auth,
            http: reqwest::Client::new(),
            base_url: BASE_URL.to_string(),
        }
    }

    /// Get the current access token.
    pub async fn get_access_token(&mut self) -> Result<String> {
        self.auth.get_access_token().await
    }

    /// Send a text message to a user.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The recipient's user ID
    /// * `text` - The text message to send
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message sent successfully
    /// * `Err(anyhow::Error)` - An error if sending fails
    pub async fn send_text_message(&mut self, user_id: &str, text: &str) -> Result<()> {
        info!("Sending text message to {}", user_id);
        let token = self.auth.get_access_token().await?;

        let payload = MessagePayload {
            recipient: Recipient {
                user_id: user_id.to_string(),
            },
            message: MessageContent {
                attachment: Attachment {
                    type_field: "text".to_string(),
                    payload: serde_json::json!({
                        "text": text
                    }),
                },
            },
        };

        self.post_message(&token, &payload).await
    }

    /// Send an image message to a user.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The recipient's user ID
    /// * `image_url` - The URL of the image to send
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message sent successfully
    /// * `Err(anyhow::Error)` - An error if sending fails
    pub async fn send_image_message(&mut self, user_id: &str, image_url: &str) -> Result<()> {
        info!("Sending image message to {} at {}", user_id, image_url);
        let token = self.auth.get_access_token().await?;

        let payload = MessagePayload {
            recipient: Recipient {
                user_id: user_id.to_string(),
            },
            message: MessageContent {
                attachment: Attachment {
                    type_field: "image".to_string(),
                    payload: serde_json::json!({
                        "url": image_url
                    }),
                },
            },
        };

        self.post_message(&token, &payload).await
    }

    /// Send a file message to a user.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The recipient's user ID
    /// * `file_token` - The token for the uploaded file
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message sent successfully
    /// * `Err(anyhow::Error)` - An error if sending fails
    pub async fn send_file_message(&mut self, user_id: &str, file_token: &str) -> Result<()> {
        info!(
            "Sending file message to {} with token {}",
            user_id, file_token
        );
        let token = self.auth.get_access_token().await?;

        let payload = MessagePayload {
            recipient: Recipient {
                user_id: user_id.to_string(),
            },
            message: MessageContent {
                attachment: Attachment {
                    type_field: "file".to_string(),
                    payload: serde_json::json!({
                        "token": file_token
                    }),
                },
            },
        };

        self.post_message(&token, &payload).await
    }

    /// Upload a file to Zalo's CDN.
    ///
    /// # Arguments
    ///
    /// * `file_data` - The file data to upload
    /// * `filename` - The filename for the uploaded file
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The file token
    /// * `Err(anyhow::Error)` - An error if upload fails
    pub async fn upload_file(&mut self, file_data: Vec<u8>, filename: &str) -> Result<String> {
        info!("Uploading file: {}", filename);
        let token = self.auth.get_access_token().await?;

        let url = format!("{}/media/upload", self.base_url);

        let response = self
            .http
            .post(&url)
            .bearer_auth(&token)
            .header(
                "Content-Disposition",
                format!("attachment; filename=\"{}\"", filename),
            )
            .body(file_data)
            .send()
            .await?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            debug!("File upload response: {:?}", result);

            // Extract the media_id or file_token from the response
            if let Some(media_id) = result.get("media_id").and_then(|v| v.as_str()) {
                Ok(media_id.to_string())
            } else if let Some(token) = result.get("token").and_then(|v| v.as_str()) {
                Ok(token.to_string())
            } else {
                Err(anyhow::anyhow!(
                    "Upload response missing media_id or token: {:?}",
                    result
                ))
            }
        } else {
            let status = response.status().as_u16();
            let error_msg = response.text().await.unwrap_or_default();
            error!("File upload failed: {} - {}", status, error_msg);
            Err(anyhow::anyhow!(
                "File upload failed: {} - {}",
                status,
                error_msg
            ))
        }
    }

    /// Get user profile information.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID to look up
    ///
    /// # Returns
    ///
    /// * `Ok(UserProfile)` - The user's profile information
    /// * `Err(anyhow::Error)` - An error if lookup fails
    pub async fn get_user_profile(&mut self, user_id: &str) -> Result<UserProfile> {
        debug!("Fetching user profile for {}", user_id);
        let token = self.auth.get_access_token().await?;

        let url = format!("{}/user/{}", self.base_url, user_id);

        let response = self.http.get(&url).bearer_auth(&token).send().await?;

        if response.status().is_success() {
            let profile: UserProfile = response.json().await?;
            Ok(profile)
        } else {
            let status = response.status().as_u16();
            let error_msg = response.text().await.unwrap_or_default();
            error!("User profile fetch failed: {} - {}", status, error_msg);
            Err(anyhow::anyhow!(
                "User profile fetch failed: {} - {}",
                status,
                error_msg
            ))
        }
    }

    /// Create a message template.
    ///
    /// # Arguments
    ///
    /// * `name` - The template name
    /// * `language` - The language code (e.g., "vi", "en")
    /// * `components` - The template components
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The template ID
    /// * `Err(anyhow::Error)` - An error if creation fails
    pub async fn create_template(
        &mut self,
        name: &str,
        language: &str,
        components: &[serde_json::Value],
    ) -> Result<String> {
        debug!("Creating template: {}", name);
        let token = self.auth.get_access_token().await?;

        let url = format!("{}/message/template", self.base_url);

        let response = self
            .http
            .post(&url)
            .bearer_auth(&token)
            .json(&serde_json::json!({
                "name": name,
                "language": language,
                "components": components
            }))
            .send()
            .await?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            if let Some(template_id) = result.get("template_id").and_then(|v| v.as_str()) {
                Ok(template_id.to_string())
            } else {
                Err(anyhow::anyhow!(
                    "Template creation response missing template_id: {:?}",
                    result
                ))
            }
        } else {
            let status = response.status().as_u16();
            let error_msg = response.text().await.unwrap_or_default();
            error!("Template creation failed: {} - {}", status, error_msg);
            Err(anyhow::anyhow!(
                "Template creation failed: {} - {}",
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
    /// * `Err(anyhow::Error)` - An error if verification fails
    pub async fn verify_token(&self) -> Result<bool> {
        validate_access_token(
            self.auth.access_token().unwrap_or(""),
            self.auth.app_secret(),
        )
        .await
    }

    /// Internal method to POST a message.
    async fn post_message(&self, token: &str, payload: &MessagePayload) -> Result<()> {
        let url = format!("{}/message/cs", self.base_url);

        let response = self
            .http
            .post(&url)
            .bearer_auth(token)
            .json(payload)
            .send()
            .await?;

        if response.status().is_success() {
            debug!("Message sent successfully");
            Ok(())
        } else {
            let status = response.status().as_u16();
            let error_msg = response.text().await.unwrap_or_default();
            error!("Message send failed: {} - {}", status, error_msg);
            Err(anyhow::anyhow!(
                "Message send failed: {} - {}",
                status,
                error_msg
            ))
        }
    }
}

/// User profile information from Zalo.
#[derive(Debug, Deserialize)]
pub struct UserProfile {
    /// User ID
    pub id: String,
    /// User's name
    pub name: Option<String>,
    /// User's avatar URL
    #[serde(default)]
    pub avatar: Option<String>,
    /// User's gender
    pub gender: Option<i32>,
    /// User's phone number
    #[serde(default)]
    pub phone: Option<String>,
    /// User's email
    #[serde(default)]
    pub email: Option<String>,
    /// User's date of birth
    #[serde(default)]
    pub birthday: Option<String>,
    /// User's address
    #[serde(default)]
    pub address: Option<String>,
    /// User's language
    #[serde(default)]
    pub language: Option<String>,
    /// User's zone info
    #[serde(default)]
    pub zone_info: Option<String>,
    /// User's birthday in YYYY-MM-DD format
    #[serde(default)]
    pub birthday_date: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zalo_api_initialization() {
        let auth = ZaloAuth::new(
            "test_app_id".to_string(),
            "test_app_secret".to_string(),
            "test_refresh_token".to_string(),
        );
        let api = ZaloApi::new(auth);

        assert_eq!(api.base_url, BASE_URL);
    }

    #[test]
    fn test_message_payload_serialization() {
        let payload = MessagePayload {
            recipient: Recipient {
                user_id: "123456789".to_string(),
            },
            message: MessageContent {
                attachment: Attachment {
                    type_field: "text".to_string(),
                    payload: serde_json::json!({
                        "text": "Hello, world!"
                    }),
                },
            },
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("123456789"));
        assert!(json.contains("Hello, world!"));
    }
}
