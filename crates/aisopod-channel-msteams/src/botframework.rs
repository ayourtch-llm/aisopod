//! Microsoft Bot Framework API client.
//!
//! This module provides a client for interacting with the Microsoft Bot Framework API,
//! including sending and receiving activities, managing conversations, and handling
//! various bot operations.

use crate::auth::{MsTeamsAuth, MsTeamsAuthToken};
use crate::config::MsTeamsAccountConfig;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

/// Bot Framework API client.
#[derive(Debug, Clone)]
pub struct BotFrameworkClient {
    /// Authentication manager
    auth: MsTeamsAuth,
    /// Base URL for Bot Framework API
    base_url: String,
    /// Microsoft App ID for authentication
    app_id: String,
}

impl BotFrameworkClient {
    /// Creates a new Bot Framework API client.
    pub fn new(auth: MsTeamsAuth, app_id: &str) -> Self {
        Self {
            auth,
            base_url: "https://smba.trafficmanager.net/amer".to_string(),
            app_id: app_id.to_string(),
        }
    }

    /// Creates a new Bot Framework API client from account config.
    pub fn from_account_config(config: &MsTeamsAccountConfig) -> Result<Self> {
        let auth = MsTeamsAuth::from_account_config(config);
        let app_id = config.bot_app_id_or_client_id().to_string();
        Ok(Self::new(auth, &app_id))
    }

    /// Sets the base URL for the Bot Framework API.
    /// This is useful for different regions or testing.
    pub fn with_base_url(mut self, base_url: &str) -> Self {
        self.base_url = base_url.to_string();
        self
    }

    /// Gets the current access token.
    async fn get_access_token(&mut self) -> Result<MsTeamsAuthToken> {
        self.auth.get_token().await
    }

    /// Creates the HTTP client with authentication.
    async fn create_client_with_auth(&mut self) -> Result<reqwest::Client> {
        let token = self.get_access_token().await?;
        let bearer_token = format!("Bearer {}", token.token);

        let client = reqwest::Client::new();
        Ok(client)
    }

    /// Sends an activity to a conversation.
    /// This is used to send messages, typing indicators, and other activities to users.
    pub async fn send_activity(
        &mut self,
        conversation_id: &str,
        activity: &Activity,
    ) -> Result<String> {
        let client = self.create_client_with_auth().await?;
        let url = format!(
            "{}/v3/conversations/{}/activities",
            self.base_url, conversation_id
        );

        debug!("Sending activity to {}", url);

        let response = client.post(&url).json(activity).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Failed to send activity: {} - {}", status, body);
            return Err(anyhow::anyhow!("Failed to send activity: {}", status));
        }

        let response_body: SendActivityResponse = response.json().await?;
        Ok(response_body.id)
    }

    /// Sends a message to a user or channel.
    pub async fn send_message(
        &mut self,
        conversation_id: &str,
        text: &str,
        reply_to_id: Option<&str>,
    ) -> Result<String> {
        let activity = Activity::create_message(text, reply_to_id);
        self.send_activity(conversation_id, &activity).await
    }

    /// Creates a new conversation with a user.
    pub async fn create_conversation(
        &mut self,
        activity: &Activity,
    ) -> Result<ConversationResponse> {
        let client = self.create_client_with_auth().await?;
        let url = format!("{}/v3/conversations", self.base_url);

        debug!("Creating new conversation at {}", url);

        let response = client.post(&url).json(activity).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Failed to create conversation: {} - {}", status, body);
            return Err(anyhow::anyhow!("Failed to create conversation: {}", status));
        }

        let response: ConversationResponse = response.json().await?;
        Ok(response)
    }

    /// Gets a message activity.
    pub async fn get_activity(
        &mut self,
        conversation_id: &str,
        activity_id: &str,
    ) -> Result<Activity> {
        let client = self.create_client_with_auth().await?;
        let url = format!(
            "{}/v3/conversations/{}/activities/{}",
            self.base_url, conversation_id, activity_id
        );

        let response = client.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Failed to get activity: {} - {}", status, body);
            return Err(anyhow::anyhow!("Failed to get activity: {}", status));
        }

        let activity: Activity = response.json().await?;
        Ok(activity)
    }

    /// Updates a message activity.
    pub async fn update_activity(
        &mut self,
        conversation_id: &str,
        activity_id: &str,
        activity: &Activity,
    ) -> Result<()> {
        let client = self.create_client_with_auth().await?;
        let url = format!(
            "{}/v3/conversations/{}/activities/{}",
            self.base_url, conversation_id, activity_id
        );

        let response = client.put(&url).json(activity).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Failed to update activity: {} - {}", status, body);
            return Err(anyhow::anyhow!("Failed to update activity: {}", status));
        }

        Ok(())
    }

    /// Deletes a message activity.
    pub async fn delete_activity(
        &mut self,
        conversation_id: &str,
        activity_id: &str,
    ) -> Result<()> {
        let client = self.create_client_with_auth().await?;
        let url = format!(
            "{}/v3/conversations/{}/activities/{}",
            self.base_url, conversation_id, activity_id
        );

        let response = client.delete(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Failed to delete activity: {} - {}", status, body);
            return Err(anyhow::anyhow!("Failed to delete activity: {}", status));
        }

        Ok(())
    }

    /// Gets the members of a conversation.
    pub async fn get_conversation_members(
        &mut self,
        conversation_id: &str,
    ) -> Result<Vec<ConversationMember>> {
        let client = self.create_client_with_auth().await?;
        let url = format!(
            "{}/v3/conversations/{}/members",
            self.base_url, conversation_id
        );

        let response = client.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Failed to get conversation members: {} - {}", status, body);
            return Err(anyhow::anyhow!(
                "Failed to get conversation members: {}",
                status
            ));
        }

        let members: Vec<ConversationMember> = response.json().await?;
        Ok(members)
    }

    /// Gets the conversation PSM (Persistent State Management) info.
    pub async fn get_conversation_psm_info(
        &mut self,
        conversation_id: &str,
    ) -> Result<ConversationPsmInfo> {
        let client = self.create_client_with_auth().await?;
        let url = format!(
            "{}/v3/conversations/{}/psmInfo",
            self.base_url, conversation_id
        );

        let response = client.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Failed to get PSM info: {} - {}", status, body);
            return Err(anyhow::anyhow!("Failed to get PSM info: {}", status));
        }

        let info: ConversationPsmInfo = response.json().await?;
        Ok(info)
    }

    /// Sends a typing indicator to a conversation.
    pub async fn send_typing(&mut self, conversation_id: &str) -> Result<()> {
        let activity = Activity::create_typing();
        self.send_activity(conversation_id, &activity).await?;
        Ok(())
    }

    /// Marks a message as read.
    pub async fn mark_as_read(&mut self, conversation_id: &str, activity_id: &str) -> Result<()> {
        let activity = Activity::create_read(activity_id);
        self.send_activity(conversation_id, &activity).await?;
        Ok(())
    }
}

/// Activity types for Bot Framework.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ActivityType {
    /// A message from a user
    Message,
    /// A typing indicator
    Typing,
    /// A conversation membership update
    ConversationUpdate,
    /// A message was added to the conversation
    MessageAdd,
    /// A message was deleted from the conversation
    MessageDelete,
    /// A message was updated in the conversation
    MessageUpdate,
    /// A reaction was added to a message
    MessageReaction,
    /// A synchronization action
    SynchronizeAction,
    /// A ping for health checking
    Ping,
    /// A end-of-conversation message
    EndOfConversation,
    /// A event activity
    Event,
    /// A invoke activity
    Invoke,
    /// A trace activity
    Trace,
    /// A handoff activity
    Handoff,
}

/// Represents an activity in the Bot Framework.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Activity {
    /// Unique identifier for this activity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Timestamp when the activity was created
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<DateTime<Utc>>,
    /// Local timestamp when the activity was created
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_timestamp: Option<DateTime<Utc>>,
    /// The channel ID where the activity originated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<String>,
    /// Conversation information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation: Option<ConversationReference>,
    /// The activity type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activity_type: Option<ActivityType>,
    /// The recipient of the activity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipient: Option<ChannelAccount>,
    /// The sender of the activity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<ChannelAccount>,
    /// The text content of the activity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// The text content for speech
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speak: Option<String>,
    /// The speech input hint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_hint: Option<String>,
    /// The summary text for the activity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// The text to display when the activity cannot be rendered
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_format: Option<String>,
    /// The attachment content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachment: Option<Vec<Attachment>>,
    /// The entity content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity: Option<Vec<Entity>>,
    /// The mark read activity ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mark_read: Option<String>,
    /// The reply to activity ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to_id: Option<String>,
    /// View state for the activity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub view_state: Option<String>,
    /// The action associated with the activity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    /// The name of the action
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The value of the action
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,
    /// The locale for the activity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    /// The channel data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_data: Option<serde_json::Value>,
    /// The origin
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
    /// The semantic action
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semantic_action: Option<SemanticAction>,
    /// The relates to conversation reference
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relates_to: Option<ConversationReference>,
}

impl Activity {
    /// Creates a new message activity.
    pub fn create_message(text: &str, reply_to_id: Option<&str>) -> Self {
        Self {
            activity_type: Some(ActivityType::Message),
            text: Some(text.to_string()),
            conversation: Some(ConversationReference::default()),
            reply_to_id: reply_to_id.map(|s| s.to_string()),
            ..Default::default()
        }
    }

    /// Creates a new typing activity.
    pub fn create_typing() -> Self {
        Self {
            activity_type: Some(ActivityType::Typing),
            conversation: Some(ConversationReference::default()),
            ..Default::default()
        }
    }

    /// Creates a new read activity.
    pub fn create_read(reply_to_id: &str) -> Self {
        Self {
            activity_type: Some(ActivityType::Message),
            conversation: Some(ConversationReference::default()),
            mark_read: Some(reply_to_id.to_string()),
            ..Default::default()
        }
    }

    /// Creates a new conversation update activity.
    pub fn create_conversation_update(action: &str, members_added: Vec<ChannelAccount>) -> Self {
        Self {
            activity_type: Some(ActivityType::ConversationUpdate),
            conversation: Some(ConversationReference::default()),
            action: Some(action.to_string()),
            ..Default::default()
        }
    }

    /// Creates a new event activity.
    pub fn create_event(name: &str, value: serde_json::Value) -> Self {
        Self {
            activity_type: Some(ActivityType::Event),
            conversation: Some(ConversationReference::default()),
            name: Some(name.to_string()),
            value: Some(value),
            ..Default::default()
        }
    }

    /// Creates a new invoke activity.
    pub fn create_invoke(name: &str, value: serde_json::Value) -> Self {
        Self {
            activity_type: Some(ActivityType::Invoke),
            conversation: Some(ConversationReference::default()),
            name: Some(name.to_string()),
            value: Some(value),
            ..Default::default()
        }
    }
}

/// Conversation reference for Bot Framework activities.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConversationReference {
    /// The channel ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<String>,
    /// Conversation information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation: Option<ChannelAccount>,
    /// Bot information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bot: Option<ChannelAccount>,
    /// Service URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_url: Option<String>,
    /// The user's ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<ChannelAccount>,
    /// Channel data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_data: Option<serde_json::Value>,
}

/// Channel account information.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChannelAccount {
    /// The channel ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The name of the account
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The role of the account
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// Channel data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_data: Option<serde_json::Value>,
}

/// Attachment content for activities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    /// The content type of the attachment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    /// The content of the attachment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<serde_json::Value>,
    /// The name of the attachment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The thumbnail URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
}

/// Entity in an activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// The type of entity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    /// The channel account
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_account: Option<ChannelAccount>,
}

/// Semantic action for activities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticAction {
    /// The name of the action
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The state of the action
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    /// The actions associated with this semantic action
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actions: Option<Vec<serde_json::Value>>,
}

/// Response for creating a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationResponse {
    /// The conversation ID
    pub conversation_id: String,
    /// The service URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_url: Option<String>,
    /// The activity ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activity_id: Option<String>,
    /// The PSM info
    #[serde(skip_serializing_if = "Option::is_none")]
    pub psm_info: Option<serde_json::Value>,
    /// Channel data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_data: Option<serde_json::Value>,
}

/// Response for sending an activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendActivityResponse {
    /// The activity ID
    pub id: String,
    /// The timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<DateTime<Utc>>,
    /// The ETag
    #[serde(skip_serializing_if = "Option::is_none")]
    pub e_tag: Option<String>,
}

/// Response for getting conversation members.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMember {
    /// The member ID
    pub id: String,
    /// The member name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The role
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

/// Response for getting PSM info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationPsmInfo {
    /// The PSM ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub psm_id: Option<String>,
    /// The state
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_activity_create_message() {
        let activity = Activity::create_message("Hello, World!", None);
        assert_eq!(activity.activity_type, Some(ActivityType::Message));
        assert_eq!(activity.text, Some("Hello, World!".to_string()));
        assert!(activity.reply_to_id.is_none());
    }

    #[test]
    fn test_activity_create_message_with_reply() {
        let activity = Activity::create_message("Hello, World!", Some("reply123"));
        assert_eq!(activity.reply_to_id, Some("reply123".to_string()));
    }

    #[test]
    fn test_activity_create_typing() {
        let activity = Activity::create_typing();
        assert_eq!(activity.activity_type, Some(ActivityType::Typing));
    }

    #[test]
    fn test_activity_create_conversation_update() {
        let activity = Activity::create_conversation_update(
            "membersAdded",
            vec![ChannelAccount {
                id: Some("user1".to_string()),
                name: Some("User One".to_string()),
                role: Some("user".to_string()),
                ..Default::default()
            }],
        );
        assert_eq!(
            activity.activity_type,
            Some(ActivityType::ConversationUpdate)
        );
        assert_eq!(activity.action, Some("membersAdded".to_string()));
    }

    #[test]
    fn test_activity_create_event() {
        let activity = Activity::create_event("testEvent", serde_json::json!({"key": "value"}));
        assert_eq!(activity.activity_type, Some(ActivityType::Event));
        assert_eq!(activity.name, Some("testEvent".to_string()));
        assert_eq!(activity.value, Some(serde_json::json!({"key": "value"})));
    }
}
