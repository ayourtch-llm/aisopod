//! LINE Messaging API client.
//!
//! This module provides the LINE Messaging API client for sending messages,
//! handling rich messages, and interacting with the LINE platform.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

/// BASE_URL for LINE Messaging API
pub const BASE_URL: &str = "https://api.line.me/v2/bot";

/// Error type for LINE API operations.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("API error: {code} - {message}")]
    Api { code: u16, message: String },

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Response from the LINE API.
#[derive(Debug, Deserialize)]
pub struct LineResponse {
    #[serde(default)]
    pub message: Option<String>,
}

/// LINE message types.
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum LineMessage {
    #[serde(rename = "text")]
    Text { text: String },

    #[serde(rename = "image")]
    Image {
        #[serde(rename = "originalContentUrl")]
        original_content_url: String,
        #[serde(rename = "previewImageUrl")]
        preview_image_url: String,
    },

    #[serde(rename = "video")]
    Video {
        #[serde(rename = "originalContentUrl")]
        original_content_url: String,
        #[serde(rename = "previewImageUrl")]
        preview_image_url: String,
    },

    #[serde(rename = "audio")]
    Audio {
        #[serde(rename = "originalContentUrl")]
        original_content_url: String,
        #[serde(rename = "duration")]
        duration: u64,
    },

    #[serde(rename = "location")]
    Location {
        title: String,
        address: String,
        #[serde(rename = "latitude")]
        latitude: f64,
        #[serde(rename = "longitude")]
        longitude: f64,
    },

    #[serde(rename = "sticker")]
    Sticker {
        #[serde(rename = "packageId")]
        package_id: String,
        #[serde(rename = "stickerId")]
        sticker_id: String,
    },

    #[serde(rename = "flex")]
    Flex {
        #[serde(rename = "altText")]
        alt_text: String,
        contents: serde_json::Value,
    },
}

/// Flex Message container types.
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum FlexContainerType {
    #[serde(rename = "bubble")]
    Bubble,
    #[serde(rename = "carousel")]
    Carousel,
}

/// Flex container component.
#[derive(Debug, Serialize)]
pub struct FlexContainer {
    #[serde(rename = "type")]
    pub container_type: FlexContainerType,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<FlexComponent>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<FlexComponent>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub footer: Option<FlexComponent>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub styles: Option<FlexStyles>,
}

/// Flex container styles.
#[derive(Debug, Serialize)]
pub struct FlexStyles {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<FlexBlockStyle>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<FlexBlockStyle>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub footer: Option<FlexBlockStyle>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub hero: Option<FlexBlockStyle>,
}

/// Flex block style.
#[derive(Debug, Serialize)]
pub struct FlexBlockStyle {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub separator: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub separator_color: Option<String>,
}

/// Flex component.
#[derive(Debug, Serialize)]
pub struct FlexComponent {
    #[serde(rename = "type")]
    pub component_type: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub contents: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<serde_json::Value>,
}

/// LINE API client.
#[derive(Clone)]
pub struct LineApi {
    token: String,
    http: reqwest::Client,
    base_url: String,
}

impl LineApi {
    /// Create a new LineApi instance.
    pub fn new(token: String) -> Self {
        Self {
            token,
            http: reqwest::Client::new(),
            base_url: BASE_URL.to_string(),
        }
    }

    /// Get the channel access token.
    pub fn token(&self) -> &str {
        &self.token
    }

    /// Send a reply message to a user or group.
    ///
    /// # Arguments
    ///
    /// * `reply_token` - The reply token from the webhook event
    /// * `messages` - The messages to send
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message sent successfully
    /// * `Err(ApiError)` - An error occurred
    pub async fn reply_message(
        &self,
        reply_token: &str,
        messages: Vec<LineMessage>,
    ) -> Result<(), ApiError> {
        let url = format!("{}/message/reply", self.base_url);

        debug!("Sending reply message to: {}", reply_token);

        let response = self
            .http
            .post(&url)
            .bearer_auth(&self.token)
            .json(&serde_json::json!({
                "replyToken": reply_token,
                "messages": messages
            }))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status().as_u16();
            let error_msg = response.text().await.unwrap_or_default();
            error!("API error {}: {}", status, error_msg);
            Err(ApiError::Api {
                code: status,
                message: error_msg,
            })
        }
    }

    /// Send a push message to a user or group.
    ///
    /// # Arguments
    ///
    /// * `to` - The destination user or group ID
    /// * `messages` - The messages to send
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message sent successfully
    /// * `Err(ApiError)` - An error occurred
    pub async fn push_message(&self, to: &str, messages: Vec<LineMessage>) -> Result<(), ApiError> {
        let url = format!("{}/message/push", self.base_url);

        debug!("Sending push message to: {}", to);

        let response = self
            .http
            .post(&url)
            .bearer_auth(&self.token)
            .json(&serde_json::json!({
                "to": to,
                "messages": messages
            }))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status().as_u16();
            let error_msg = response.text().await.unwrap_or_default();
            error!("API error {}: {}", status, error_msg);
            Err(ApiError::Api {
                code: status,
                message: error_msg,
            })
        }
    }

    /// Send a multicast message to multiple users or groups.
    ///
    /// # Arguments
    ///
    /// * `to` - The list of destination IDs
    /// * `messages` - The messages to send
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Messages sent successfully
    /// * `Err(ApiError)` - An error occurred
    pub async fn multicast(
        &self,
        to: Vec<String>,
        messages: Vec<LineMessage>,
    ) -> Result<(), ApiError> {
        let url = format!("{}/message/multicast", self.base_url);

        debug!("Sending multicast message to {} recipients", to.len());

        let response = self
            .http
            .post(&url)
            .bearer_auth(&self.token)
            .json(&serde_json::json!({
                "to": to,
                "messages": messages
            }))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status().as_u16();
            let error_msg = response.text().await.unwrap_or_default();
            error!("API error {}: {}", status, error_msg);
            Err(ApiError::Api {
                code: status,
                message: error_msg,
            })
        }
    }

    /// Get profile information for a user.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    ///
    /// # Returns
    ///
    /// * `Ok(UserProfile)` - The user profile
    /// * `Err(ApiError)` - An error occurred
    pub async fn get_profile(&self, user_id: &str) -> Result<UserProfile, ApiError> {
        let url = format!("{}/profile/{}", self.base_url, user_id);

        let response = self.http.get(&url).bearer_auth(&self.token).send().await?;

        if response.status().is_success() {
            let profile = response.json::<UserProfile>().await?;
            Ok(profile)
        } else {
            let status = response.status().as_u16();
            let error_msg = response.text().await.unwrap_or_default();
            error!("API error {}: {}", status, error_msg);
            Err(ApiError::Api {
                code: status,
                message: error_msg,
            })
        }
    }

    /// Get group member profile.
    ///
    /// # Arguments
    ///
    /// * `group_id` - The group ID
    /// * `user_id` - The user ID
    ///
    /// # Returns
    ///
    /// * `Ok(UserProfile)` - The user profile
    /// * `Err(ApiError)` - An error occurred
    pub async fn get_group_member_profile(
        &self,
        group_id: &str,
        user_id: &str,
    ) -> Result<UserProfile, ApiError> {
        let url = format!("{}/group/{}/member/{}", self.base_url, group_id, user_id);

        let response = self.http.get(&url).bearer_auth(&self.token).send().await?;

        if response.status().is_success() {
            let profile = response.json::<UserProfile>().await?;
            Ok(profile)
        } else {
            let status = response.status().as_u16();
            let error_msg = response.text().await.unwrap_or_default();
            error!("API error {}: {}", status, error_msg);
            Err(ApiError::Api {
                code: status,
                message: error_msg,
            })
        }
    }

    /// Get room member profile.
    ///
    /// # Arguments
    ///
    /// * `room_id` - The room ID
    /// * `user_id` - The user ID
    ///
    /// # Returns
    ///
    /// * `Ok(UserProfile)` - The user profile
    /// * `Err(ApiError)` - An error occurred
    pub async fn get_room_member_profile(
        &self,
        room_id: &str,
        user_id: &str,
    ) -> Result<UserProfile, ApiError> {
        let url = format!("{}/room/{}/member/{}", self.base_url, room_id, user_id);

        let response = self.http.get(&url).bearer_auth(&self.token).send().await?;

        if response.status().is_success() {
            let profile = response.json::<UserProfile>().await?;
            Ok(profile)
        } else {
            let status = response.status().as_u16();
            let error_msg = response.text().await.unwrap_or_default();
            error!("API error {}: {}", status, error_msg);
            Err(ApiError::Api {
                code: status,
                message: error_msg,
            })
        }
    }

    /// Leave a group.
    ///
    /// # Arguments
    ///
    /// * `group_id` - The group ID
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Left successfully
    /// * `Err(ApiError)` - An error occurred
    pub async fn leave_group(&self, group_id: &str) -> Result<(), ApiError> {
        let url = format!("{}/group/{}/leave", self.base_url, group_id);

        let response = self.http.post(&url).bearer_auth(&self.token).send().await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status().as_u16();
            let error_msg = response.text().await.unwrap_or_default();
            error!("API error {}: {}", status, error_msg);
            Err(ApiError::Api {
                code: status,
                message: error_msg,
            })
        }
    }

    /// Leave a room.
    ///
    /// # Arguments
    ///
    /// * `room_id` - The room ID
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Left successfully
    /// * `Err(ApiError)` - An error occurred
    pub async fn leave_room(&self, room_id: &str) -> Result<(), ApiError> {
        let url = format!("{}/room/{}/leave", self.base_url, room_id);

        let response = self.http.post(&url).bearer_auth(&self.token).send().await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status().as_u16();
            let error_msg = response.text().await.unwrap_or_default();
            error!("API error {}: {}", status, error_msg);
            Err(ApiError::Api {
                code: status,
                message: error_msg,
            })
        }
    }
}

/// User profile from LINE.
#[derive(Debug, Deserialize)]
pub struct UserProfile {
    #[serde(rename = "displayName")]
    pub display_name: String,

    #[serde(rename = "userId")]
    pub user_id: String,

    #[serde(rename = "pictureUrl")]
    pub picture_url: Option<String>,

    #[serde(rename = "statusMessage")]
    pub status_message: Option<String>,
}

/// Builder for Flex containers.
pub struct FlexBuilder {
    container: FlexContainer,
}

impl FlexBuilder {
    /// Create a new FlexBuilder with a bubble container.
    pub fn new_bubble() -> Self {
        Self {
            container: FlexContainer {
                container_type: FlexContainerType::Bubble,
                body: None,
                header: None,
                footer: None,
                styles: None,
            },
        }
    }

    /// Create a new FlexBuilder with a carousel container.
    pub fn new_carousel() -> Self {
        Self {
            container: FlexContainer {
                container_type: FlexContainerType::Carousel,
                body: None,
                header: None,
                footer: None,
                styles: None,
            },
        }
    }

    /// Set the header component.
    pub fn header(mut self, header: FlexComponent) -> Self {
        self.container.header = Some(header);
        self
    }

    /// Set the body component.
    pub fn body(mut self, body: FlexComponent) -> Self {
        self.container.body = Some(body);
        self
    }

    /// Set the footer component.
    pub fn footer(mut self, footer: FlexComponent) -> Self {
        self.container.footer = Some(footer);
        self
    }

    /// Set the styles.
    pub fn styles(mut self, styles: FlexStyles) -> Self {
        self.container.styles = Some(styles);
        self
    }

    /// Build the FlexContainer.
    pub fn build(self) -> FlexContainer {
        self.container
    }
}

/// Simple text component builder.
pub struct TextComponentBuilder {
    text: String,
    wrap: bool,
    size: Option<String>,
    weight: Option<String>,
    color: Option<String>,
}

impl TextComponentBuilder {
    /// Create a new TextComponentBuilder.
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            wrap: false,
            size: None,
            weight: None,
            color: None,
        }
    }

    /// Enable text wrapping.
    pub fn wrap(mut self) -> Self {
        self.wrap = true;
        self
    }

    /// Set the text size.
    pub fn size(mut self, size: &str) -> Self {
        self.size = Some(size.to_string());
        self
    }

    /// Set the text weight.
    pub fn weight(mut self, weight: &str) -> Self {
        self.weight = Some(weight.to_string());
        self
    }

    /// Set the text color.
    pub fn color(mut self, color: &str) -> Self {
        self.color = Some(color.to_string());
        self
    }

    /// Build the FlexComponent.
    pub fn build(self) -> FlexComponent {
        let mut properties = serde_json::json!({
            "type": "text",
            "text": self.text,
            "wrap": self.wrap,
        });

        if let Some(size) = self.size {
            properties["size"] = serde_json::Value::String(size);
        }
        if let Some(weight) = self.weight {
            properties["weight"] = serde_json::Value::String(weight);
        }
        if let Some(color) = self.color {
            properties["color"] = serde_json::Value::String(color);
        }

        FlexComponent {
            component_type: "text".to_string(),
            layout: None,
            contents: Some(properties),
            action: None,
        }
    }
}

/// Simple box component builder.
pub struct BoxComponentBuilder {
    layout: String,
    contents: Vec<FlexComponent>,
}

impl BoxComponentBuilder {
    /// Create a new BoxComponentBuilder.
    pub fn new(layout: &str) -> Self {
        Self {
            layout: layout.to_string(),
            contents: Vec::new(),
        }
    }

    /// Add a component to the box.
    pub fn add_component(mut self, component: FlexComponent) -> Self {
        self.contents.push(component);
        self
    }

    /// Build the FlexComponent.
    pub fn build(self) -> FlexComponent {
        let layout = self.layout.clone();
        FlexComponent {
            component_type: "box".to_string(),
            layout: Some(layout.clone()),
            contents: Some(serde_json::json!({
                "type": "box",
                "layout": layout,
                "contents": self.contents
            })),
            action: None,
        }
    }
}
