//! Google Chat API client.
//!
//! This module provides a client for interacting with the Google Chat API,
//! including sending and receiving messages, and managing spaces.

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error, info};

use crate::auth::{GoogleChatAuth, GoogleChatAuthToken};

/// Base URL for the Google Chat API.
pub const GOOGLE_CHAT_API_BASE_URL: &str = "https://chat.googleapis.com/v1";

/// Google Chat API client.
pub struct GoogleChatClient {
    /// Authentication handler
    auth: Box<dyn GoogleChatAuth>,
    /// HTTP client
    client: reqwest::Client,
}

impl GoogleChatClient {
    /// Create a new Google Chat API client.
    pub fn new(auth: Box<dyn GoogleChatAuth>) -> Self {
        Self {
            auth,
            client: reqwest::Client::new(),
        }
    }

    /// Get the current access token.
    async fn get_access_token(&self) -> Result<GoogleChatAuthToken> {
        self.auth.get_token().await
    }

    /// Send a GET request to the Google Chat API.
    async fn get<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T> {
        let token = self.get_access_token().await?;
        let url = format!("{}{}", GOOGLE_CHAT_API_BASE_URL, path);

        debug!("GET {}", url);

        let response = self
            .client
            .get(&url)
            .bearer_auth(&token.token)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Send a POST request to the Google Chat API.
    async fn post<T: for<'de> Deserialize<'de>, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let token = self.get_access_token().await?;
        let url = format!("{}{}", GOOGLE_CHAT_API_BASE_URL, path);

        debug!("POST {}", url);

        let response = self
            .client
            .post(&url)
            .bearer_auth(&token.token)
            .json(body)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Send a PUT request to the Google Chat API.
    async fn put<T: for<'de> Deserialize<'de>, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let token = self.get_access_token().await?;
        let url = format!("{}{}", GOOGLE_CHAT_API_BASE_URL, path);

        debug!("PUT {}", url);

        let response = self
            .client
            .put(&url)
            .bearer_auth(&token.token)
            .json(body)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Send a DELETE request to the Google Chat API.
    async fn delete<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T> {
        let token = self.get_access_token().await?;
        let url = format!("{}{}", GOOGLE_CHAT_API_BASE_URL, path);

        debug!("DELETE {}", url);

        let response = self
            .client
            .delete(&url)
            .bearer_auth(&token.token)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Handle API response and extract the result.
    async fn handle_response<T: for<'de> Deserialize<'de>>(
        &self,
        response: reqwest::Response,
    ) -> Result<T> {
        if response.status().is_success() {
            let result = response.json::<T>().await?;
            Ok(result)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            error!("API request failed: {}", error_text);
            Err(anyhow::anyhow!("API request failed: {}", error_text))
        }
    }

    /// Get information about a space.
    pub async fn get_space(&self, space_id: &str) -> Result<Space> {
        self.get(&format!("/spaces/{}", space_id)).await
    }

    /// List spaces the bot is a member of.
    pub async fn list_spaces(&self) -> Result<ListSpacesResponse> {
        self.get("/spaces").await
    }

    /// Create a new space.
    pub async fn create_space(&self, space: &CreateSpaceRequest) -> Result<Space> {
        self.post("/spaces", space).await
    }

    /// Send a message to a space.
    pub async fn create_message(
        &self,
        space_id: &str,
        message: &CreateMessageRequest,
    ) -> Result<Message> {
        self.post(&format!("/spaces/{}/messages", space_id), message)
            .await
    }

    /// Update an existing message.
    pub async fn update_message(
        &self,
        message_name: &str,
        message: &UpdateMessageRequest,
        update_mask: &str,
    ) -> Result<Message> {
        let url = format!(
            "/{}?update_mask={}",
            message_name,
            urlencoding::encode(update_mask)
        );
        self.put(&url, message).await
    }

    /// Delete a message.
    pub async fn delete_message(&self, message_name: &str) -> Result<()> {
        self.delete::<()>(&format!("/{}", message_name)).await?;
        Ok(())
    }

    /// Get a message by name.
    pub async fn get_message(&self, message_name: &str) -> Result<Message> {
        self.get(&format!("/{}", message_name)).await
    }

    /// List messages in a space.
    pub async fn list_messages(
        &self,
        space_id: &str,
        filter: Option<&str>,
    ) -> Result<ListMessagesResponse> {
        let mut path = format!("/spaces/{}/messages", space_id);

        let mut query_params = Vec::new();
        if let Some(filter) = filter {
            query_params.push(("filter", filter));
        }
        query_params.push(("show_deleted", "true")); // Include deleted messages

        if !query_params.is_empty() {
            let query: String = query_params
                .iter()
                .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
                .collect::<Vec<_>>()
                .join("&");
            path = format!("{}?{}", path, query);
        }

        self.get(&path).await
    }

    /// Add a reaction to a message.
    pub async fn add_reaction(&self, message_name: &str, emoji: &str) -> Result<()> {
        let reaction = Reaction {
            emoji: Some(emoji.to_string()),
        };

        let _: () = self
            .post(&format!("/{}?fields=name", message_name), &reaction)
            .await?;
        Ok(())
    }

    /// Remove a reaction from a message.
    pub async fn remove_reaction(&self, message_name: &str, emoji: &str) -> Result<()> {
        let _reaction = Reaction {
            emoji: Some(emoji.to_string()),
        };

        self.delete::<()>(&format!("/{}/reactions/{}", message_name, emoji))
            .await?;
        Ok(())
    }

    /// Get members of a space.
    pub async fn list_members(&self, space_id: &str) -> Result<ListMembersResponse> {
        self.get(&format!("/spaces/{}/members", space_id)).await
    }

    /// Get member information.
    pub async fn get_member(&self, member_name: &str) -> Result<Member> {
        self.get(&member_name).await
    }

    /// Create a user invitation.
    pub async fn create_user_invitation(
        &self,
        space_id: &str,
        invitation: &CreateUserInvitationRequest,
    ) -> Result<Invitation> {
        self.post(&format!("/spaces/{}/userInvitations", space_id), invitation)
            .await
    }
}

/// Space information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Space {
    /// Resource name of the space, in the format "spaces/SPACE_ID".
    pub name: String,
    /// Display name of the space.
    pub display_name: Option<String>,
    /// Space type.
    #[serde(rename = "type")]
    pub space_type: Option<SpaceType>,
    /// Single type of space.
    #[serde(rename = "singleUserBotDm")]
    pub single_user_bot_dm: Option<bool>,
    /// Bot type.
    #[serde(rename = "bot")]
    pub bot: Option<Bot>,
    /// Creation timestamp.
    #[serde(default)]
    pub create_time: Option<DateTime<Utc>>,
    /// Last update timestamp.
    #[serde(default)]
    pub update_time: Option<DateTime<Utc>>,
    /// Deprecated: Use space_type instead.
    #[serde(default)]
    pub space_thread_read_state: Option<Vec<SpaceThreadReadState>>,
}

/// Space type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SpaceType {
    /// A direct message between the bot and a user.
    Direct,
    /// A group chat with multiple participants.
    Group,
}

/// Bot information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bot {
    /// Resource name of the bot, in the format "users/BOT_ID".
    pub name: String,
    /// Display name of the bot.
    pub display_name: Option<String>,
    /// Avatar URL of the bot.
    pub avatar_url: Option<String>,
    /// Bot type.
    #[serde(rename = "type")]
    pub bot_type: Option<BotType>,
}

/// Bot type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BotType {
    /// Human user bot.
    Human,
    /// Google Workspace bot.
    Workspace,
}

/// Space thread read state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceThreadReadState {
    /// Thread name.
    pub thread: Option<String>,
    /// Read state.
    pub thread_read_state: Option<String>,
}

/// Create space request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSpaceRequest {
    /// Space type.
    #[serde(rename = "type")]
    pub space_type: SpaceType,
    /// Display name of the space.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

/// Message information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Resource name of the message, in the format "spaces/SPACE_ID/messages/MESSAGE_ID".
    pub name: String,
    /// User who created the message.
    pub sender: Option<User>,
    /// Time when the message was created.
    #[serde(default)]
    pub create_time: Option<DateTime<Utc>>,
    /// Text content of the message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Card-based messages.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cards: Option<Vec<serde_json::Value>>,
    /// Deprecated: Use cards instead.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cards_v2: Option<Vec<serde_json::Value>>,
    /// Message type.
    #[serde(rename = "type")]
    pub message_type: Option<MessageType>,
    /// Thread information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread: Option<Thread>,
    /// Message format.
    #[serde(default)]
    pub argument_text: Option<String>,
    /// Reaction emojis.
    #[serde(default)]
    pub reaction: Option<Vec<String>>,
    /// Last edit timestamp.
    #[serde(default)]
    pub edit_time: Option<DateTime<Utc>>,
    /// Message ID.
    #[serde(default)]
    pub message_id: Option<String>,
    /// Deleted status.
    #[serde(default)]
    pub deleted: Option<bool>,
}

/// User information.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct User {
    /// Resource name of the user, in the format "users/USER_ID".
    pub name: String,
    /// Display name of the user.
    pub display_name: Option<String>,
    /// Avatar URL of the user.
    pub avatar_url: Option<String>,
    /// Email address of the user.
    pub email: Option<String>,
    /// User type.
    #[serde(rename = "type")]
    pub user_type: Option<UserType>,
    /// Bot information (if the user is a bot).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bot: Option<Bot>,
}

/// User type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UserType {
    /// Human user.
    Human,
    /// Google bot.
    Bot,
    /// Unknown user type.
    UserUnspecified,
}

/// Thread information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thread {
    /// Resource name of the thread, in the format "spaces/SPACE_ID/threads/THREAD_ID".
    pub name: String,
}

/// Message type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MessageType {
    /// Regular message.
    Default,
    /// System message.
    System,
}

/// Create message request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMessageRequest {
    /// Text content of the message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Card-based messages.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cards: Option<Vec<serde_json::Value>>,
    /// Deprecated: Use cards instead.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cards_v2: Option<Vec<serde_json::Value>>,
    /// Thread name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread: Option<Thread>,
    /// Message ID for replying to a specific message.
    #[serde(rename = "replyMessageId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_message_id: Option<String>,
}

/// Update message request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMessageRequest {
    /// Text content of the message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Card-based messages.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cards: Option<Vec<serde_json::Value>>,
    /// Deprecated: Use cards instead.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cards_v2: Option<Vec<serde_json::Value>>,
}

/// List spaces response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListSpacesResponse {
    /// List of spaces.
    pub spaces: Option<Vec<Space>>,
    /// Token for next page of results.
    #[serde(default)]
    pub next_page_token: Option<String>,
}

/// List messages response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListMessagesResponse {
    /// List of messages.
    pub messages: Option<Vec<Message>>,
    /// Token for next page of results.
    #[serde(default)]
    pub next_page_token: Option<String>,
}

/// List members response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListMembersResponse {
    /// List of members.
    pub members: Option<Vec<Member>>,
    /// Token for next page of results.
    #[serde(default)]
    pub next_page_token: Option<String>,
}

/// Member information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Member {
    /// Resource name of the member, in the format "spaces/SPACE_ID/members/USER_ID".
    pub name: String,
    /// User information.
    pub user: Option<User>,
    /// Member role.
    #[serde(rename = "role")]
    pub member_role: Option<MemberRole>,
    /// Member state.
    #[serde(rename = "state")]
    pub member_state: Option<MemberState>,
}

/// Member role.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MemberRole {
    /// Member role not specified.
    RoleUnspecified,
    /// Reader role.
    Reader,
    /// Member role.
    Member,
    /// Manager role.
    Manager,
}

/// Member state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MemberState {
    /// Member state not specified.
    StateUnspecified,
    /// Member has joined.
    Joined,
    /// Member has left.
    Left,
}

/// Create user invitation request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserInvitationRequest {
    /// User ID to invite.
    #[serde(rename = "userId")]
    pub user_id: String,
}

/// Invitation information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invitation {
    /// Resource name of the invitation.
    pub name: String,
    /// User information of the invited user.
    pub user: Option<User>,
    /// Invitation state.
    #[serde(rename = "state")]
    pub invitation_state: Option<InvitationState>,
    /// Space name.
    #[serde(rename = "space")]
    pub space: Option<String>,
}

/// Invitation state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InvitationState {
    /// Invitation state not specified.
    StateUnspecified,
    /// Invitation sent.
    Sent,
    /// Invitation accepted.
    Accepted,
}

/// Reaction information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reaction {
    /// Emoji to add as a reaction.
    pub emoji: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_space_default() {
        let space = Space {
            name: "spaces/test-space".to_string(),
            display_name: Some("Test Space".to_string()),
            space_type: Some(SpaceType::Group),
            single_user_bot_dm: Some(false),
            bot: None,
            create_time: None,
            update_time: None,
            space_thread_read_state: None,
        };

        assert_eq!(space.name, "spaces/test-space");
        assert_eq!(space.display_name, Some("Test Space".to_string()));
        assert_eq!(space.space_type, Some(SpaceType::Group));
    }

    #[test]
    fn test_message_default() {
        let message = Message {
            name: "spaces/test-space/messages/test-message".to_string(),
            sender: Some(User {
                name: "users/test-user".to_string(),
                display_name: Some("Test User".to_string()),
                ..Default::default()
            }),
            create_time: None,
            text: Some("Hello, world!".to_string()),
            cards: None,
            cards_v2: None,
            message_type: None,
            thread: None,
            argument_text: None,
            reaction: None,
            edit_time: None,
            message_id: None,
            deleted: None,
        };

        assert_eq!(message.name, "spaces/test-space/messages/test-message");
        assert_eq!(message.text, Some("Hello, world!".to_string()));
    }

    #[test]
    fn test_create_message_request_serialization() {
        let request = CreateMessageRequest {
            text: Some("Hello, world!".to_string()),
            cards: None,
            cards_v2: None,
            thread: None,
            reply_message_id: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("Hello, world!"));
    }

    #[test]
    fn test_space_type_serialization() {
        let json = serde_json::to_string(&SpaceType::Direct).unwrap();
        assert_eq!(json, "\"DIRECT\"");

        let json = serde_json::to_string(&SpaceType::Group).unwrap();
        assert_eq!(json, "\"GROUP\"");
    }

    #[test]
    fn test_message_type_serialization() {
        let json = serde_json::to_string(&MessageType::Default).unwrap();
        assert_eq!(json, "\"DEFAULT\"");

        let json = serde_json::to_string(&MessageType::System).unwrap();
        assert_eq!(json, "\"SYSTEM\"");
    }

    #[test]
    fn test_member_role_serialization() {
        let json = serde_json::to_string(&MemberRole::Reader).unwrap();
        assert_eq!(json, "\"READER\"");

        let json = serde_json::to_string(&MemberRole::Manager).unwrap();
        assert_eq!(json, "\"MANAGER\"");
    }

    #[test]
    fn test_member_state_serialization() {
        let json = serde_json::to_string(&MemberState::Joined).unwrap();
        assert_eq!(json, "\"JOINED\"");

        let json = serde_json::to_string(&MemberState::Left).unwrap();
        assert_eq!(json, "\"LEFT\"");
    }

    #[test]
    fn test_user_serialization() {
        let user = User {
            name: "users/123456789".to_string(),
            display_name: Some("John Doe".to_string()),
            email: Some("john@example.com".to_string()),
            user_type: Some(UserType::Human),
            ..Default::default()
        };

        let json = serde_json::to_string(&user).unwrap();
        assert!(json.contains("John Doe"));
        assert!(json.contains("john@example.com"));
    }
}
