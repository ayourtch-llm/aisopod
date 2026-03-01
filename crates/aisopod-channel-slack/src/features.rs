//! Advanced features for Slack: typing indicators, thread management, reactions, and more.
//!
//! This module provides implementations for Slack's advanced messaging features
//! including typing indicators, thread management, reaction handling, and channel/user discovery.

use anyhow::Result;
use serde::{Deserialize, Serialize};

// Import types from send module for type inference
use crate::send::SendMessageResponse;

/// Channel information from the Slack API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelInfo {
    /// The channel ID
    pub id: String,
    /// The channel name (without the # prefix)
    pub name: Option<String>,
    /// The channel name with # prefix
    pub name_normalized: Option<String>,
    /// The channel's purpose
    pub purpose: Option<ChannelPurpose>,
    /// The channel's description
    pub topic: Option<ChannelPurpose>,
    /// The number of members
    pub num_members: Option<u64>,
    /// Whether the channel is public
    pub is_public: Option<bool>,
    /// Whether the channel is private
    pub is_private: Option<bool>,
    /// Whether the channel is a DM
    pub is_im: Option<bool>,
    /// Whether the channel is a DM with a bot
    pub is_mpim: Option<bool>,
    /// Whether the channel is archived
    pub is_archived: Option<bool>,
    /// Whether the channel is unarchived
    pub is_general: Option<bool>,
    /// The user IDs of members
    pub members: Option<Vec<String>>,
    /// The timestamp of the last message
    pub last_read: Option<String>,
    /// The latest message timestamp
    pub latest: Option<serde_json::Value>,
    /// The number of unread messages
    pub unread_count: Option<i64>,
    /// Whether there are unread messages
    pub unread_count_display: Option<bool>,
    /// The creation timestamp
    pub created: Option<i64>,
    /// The creator user ID
    pub creator: Option<String>,
}

/// Channel purpose information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelPurpose {
    /// The purpose text
    pub value: Option<String>,
    /// The last setter's user ID
    pub last_set: Option<i64>,
}

/// User information from the Slack API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    /// The user ID
    pub id: String,
    /// The username
    pub name: Option<String>,
    /// The display name
    pub display_name: Option<String>,
    /// The real name
    pub real_name: Option<String>,
    /// The email address
    pub profile: Option<UserProfile>,
    /// Whether the user is deleted
    pub deleted: Option<bool>,
    /// Whether the user is a bot
    pub is_bot: Option<bool>,
    /// The user's timezone
    pub tz: Option<String>,
    /// The user's timezone label
    pub tz_label: Option<String>,
    /// The user's timezone offset
    pub tz_offset: Option<i64>,
    /// The user's last activity timestamp
    pub last_active: Option<i64>,
    /// Whether the user is overall online
    pub online: Option<bool>,
    /// Whether the user is in the team
    pub is_admin: Option<bool>,
    /// Whether the user is the owner
    pub is_owner: Option<bool>,
    /// Whether the user is a primary owner
    pub is_primary_owner: Option<bool>,
    /// Whether the user is restricted
    pub is_restricted: Option<bool>,
    /// Whether the user is ultra restricted
    pub is_ultra_restricted: Option<bool>,
    /// The profile image URLs
    pub profile_image: Option<String>,
}

/// User profile information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    /// The display name
    pub display_name: Option<String>,
    /// The display name normalized
    pub display_name_normalized: Option<String>,
    /// The real name
    pub real_name: Option<String>,
    /// The real name normalized
    pub real_name_normalized: Option<String>,
    /// The email address
    pub email: Option<String>,
    /// The profile image URL
    pub image_24: Option<String>,
    /// The profile image URL (32px)
    pub image_32: Option<String>,
    /// The profile image URL (48px)
    pub image_48: Option<String>,
    /// The profile image URL (72px)
    pub image_72: Option<String>,
    /// The profile image URL (192px)
    pub image_192: Option<String>,
    /// The profile image URL (512px)
    pub image_512: Option<String>,
    /// The status emoji
    pub status_emoji: Option<String>,
    /// The status text
    pub status_text: Option<String>,
    /// The status expiration timestamp
    pub status_expiration: Option<i64>,
    /// Whether the user has a 2FA enabled
    pub two_factor_type: Option<String>,
    /// Whether the user has profile editing disabled
    pub is_custom_image: Option<bool>,
}

/// A message in a thread.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadMessage {
    /// The message text
    pub text: Option<String>,
    /// The user ID who sent the message
    pub user: Option<String>,
    /// The message timestamp
    pub ts: Option<String>,
    /// The message ID
    pub id: Option<String>,
    /// The bot ID if sent by a bot
    pub bot_id: Option<String>,
    /// The subtype
    pub subtype: Option<String>,
    /// Reactions on the message
    pub reactions: Option<Vec<Reaction>>,
    /// The parent message timestamp (thread_ts)
    pub thread_ts: Option<String>,
}

/// A reaction on a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reaction {
    /// The emoji name
    pub name: String,
    /// The list of user IDs who added this reaction
    pub users: Vec<String>,
    /// The number of users who added this reaction
    pub count: u64,
}

/// Reactions list response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactionsListResponse {
    /// Whether the request was successful
    pub ok: bool,
    /// The reactions
    pub reactions: Option<Vec<Reaction>>,
    /// Error information if the request failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl ReactionsListResponse {
    /// Check if the request was successful.
    pub fn is_ok(&self) -> bool {
        self.ok
    }
}

/// Build the payload for conversations.list.
///
/// # Arguments
///
/// * `cursor` - The cursor for pagination
/// * `limit` - Maximum number of results per page
/// * `types` - Channel types to filter by (public, private, im, mpim)
///
/// # Returns
///
/// A JSON value representing the list payload
pub fn build_list_channels_payload(
    cursor: Option<&str>,
    limit: Option<u32>,
    types: Option<&str>,
) -> serde_json::Value {
    let mut payload = serde_json::json!({});

    if let Some(c) = cursor {
        payload["cursor"] = serde_json::json!(c);
    }
    if let Some(l) = limit {
        payload["limit"] = serde_json::json!(l);
    }
    if let Some(t) = types {
        payload["types"] = serde_json::json!(t);
    }

    payload
}

/// List channels accessible to the bot.
///
/// # Arguments
///
/// * `client` - The Slack client handle
/// * `cursor` - Optional cursor for pagination
/// * `limit` - Optional maximum results per page
/// * `types` - Optional channel types to filter by
///
/// # Returns
///
/// * `Ok(serde_json::Value)` - The paginated channel list response
/// * `Err(anyhow::Error)` - An error if the request fails
pub async fn list_channels(
    client: &crate::connection::SlackClientHandle,
    cursor: Option<&str>,
    limit: Option<u32>,
    types: Option<&str>,
) -> Result<serde_json::Value> {
    let payload = build_list_channels_payload(cursor, limit, types);

    let response = client
        .post("https://slack.com/api/conversations.list", &payload)
        .await?;

    let json: serde_json::Value = response.json().await?;
    Ok(json)
}

/// Get information about a channel.
///
/// # Arguments
///
/// * `client` - The Slack client handle
/// * `channel_id` - The channel ID to get information for
///
/// # Returns
///
/// * `Ok(ChannelInfo)` - The channel information
/// * `Err(anyhow::Error)` - An error if the request fails
pub async fn get_channel_info(
    client: &crate::connection::SlackClientHandle,
    channel_id: &str,
) -> Result<ChannelInfo> {
    let payload = serde_json::json!({
        "channel": channel_id
    });

    let response = client
        .post("https://slack.com/api/conversations.info", &payload)
        .await?;

    let json: serde_json::Value = response.json().await?;

    if let Some(error) = json.get("error") {
        return Err(anyhow::anyhow!(
            "conversations.info failed: {}",
            error.as_str().unwrap_or("Unknown error")
        ));
    }

    let channel: ChannelInfo = serde_json::from_value(json["channel"].clone())?;
    Ok(channel)
}

/// Get information about a user.
///
/// # Arguments
///
/// * `client` - The Slack client handle
/// * `user_id` - The user ID to get information for
///
/// # Returns
///
/// * `Ok(UserInfo)` - The user information
/// * `Err(anyhow::Error)` - An error if the request fails
pub async fn get_user_info(
    client: &crate::connection::SlackClientHandle,
    user_id: &str,
) -> Result<UserInfo> {
    let payload = serde_json::json!({
        "user": user_id
    });

    let response = client
        .post("https://slack.com/api/users.info", &payload)
        .await?;

    let json: serde_json::Value = response.json().await?;

    if let Some(error) = json.get("error") {
        return Err(anyhow::anyhow!(
            "users.info failed: {}",
            error.as_str().unwrap_or("Unknown error")
        ));
    }

    let user: UserInfo = serde_json::from_value(json["user"].clone())?;
    Ok(user)
}

/// Get a list of users in a channel.
///
/// # Arguments
///
/// * `client` - The Slack client handle
/// * `channel_id` - The channel ID
/// * `cursor` - Optional cursor for pagination
/// * `limit` - Optional maximum results per page
///
/// # Returns
///
/// * `Ok(serde_json::Value)` - The paginated user list response
/// * `Err(anyhow::Error)` - An error if the request fails
pub async fn get_channel_members(
    client: &crate::connection::SlackClientHandle,
    channel_id: &str,
    cursor: Option<&str>,
    limit: Option<u32>,
) -> Result<serde_json::Value> {
    let mut payload = serde_json::json!({
        "channel": channel_id
    });

    if let Some(c) = cursor {
        payload["cursor"] = serde_json::json!(c);
    }
    if let Some(l) = limit {
        payload["limit"] = serde_json::json!(l);
    }

    let response = client
        .post("https://slack.com/api/conversations.members", &payload)
        .await?;

    let json: serde_json::Value = response.json().await?;
    Ok(json)
}

/// Get thread replies for a message.
///
/// # Arguments
///
/// * `client` - The Slack client handle
/// * `channel_id` - The channel ID
/// * `thread_ts` - The thread timestamp
/// * `cursor` - Optional cursor for pagination
/// * `limit` - Optional maximum results per page
///
/// # Returns
///
/// * `Ok(serde_json::Value)` - The thread replies response
/// * `Err(anyhow::Error)` - An error if the request fails
pub async fn get_thread_replies(
    client: &crate::connection::SlackClientHandle,
    channel_id: &str,
    thread_ts: &str,
    cursor: Option<&str>,
    limit: Option<u32>,
) -> Result<serde_json::Value> {
    let mut payload = serde_json::json!({
        "channel": channel_id,
        "ts": thread_ts
    });

    if let Some(c) = cursor {
        payload["cursor"] = serde_json::json!(c);
    }
    if let Some(l) = limit {
        payload["limit"] = serde_json::json!(l);
    }

    let response = client
        .post("https://slack.com/api/conversations.replies", &payload)
        .await?;

    let json: serde_json::Value = response.json().await?;
    Ok(json)
}

/// Get messages in a thread with full details.
///
/// # Arguments
///
/// * `client` - The Slack client handle
/// * `channel_id` - The channel ID
/// * `thread_ts` - The thread timestamp
///
/// # Returns
///
/// * `Ok(Vec<ThreadMessage>)` - The thread messages
/// * `Err(anyhow::Error)` - An error if the request fails
pub async fn get_thread_messages(
    client: &crate::connection::SlackClientHandle,
    channel_id: &str,
    thread_ts: &str,
) -> Result<Vec<ThreadMessage>> {
    let response = get_thread_replies(client, channel_id, thread_ts, None, None).await?;

    let messages: Vec<ThreadMessage> = serde_json::from_value(response["messages"].clone())?;
    Ok(messages)
}

/// Add a reaction to a message.
///
/// # Arguments
///
/// * `client` - The Slack client handle
/// * `channel_id` - The channel ID containing the message
/// * `timestamp` - The message timestamp
/// * `emoji` - The emoji name (without colons)
///
/// # Returns
///
/// * `Ok(())` - Reaction was added successfully
/// * `Err(anyhow::Error)` - An error if adding fails
pub async fn add_reaction(
    client: &crate::connection::SlackClientHandle,
    channel_id: &str,
    timestamp: &str,
    emoji: &str,
) -> Result<()> {
    let payload = serde_json::json!({
        "channel": channel_id,
        "timestamp": timestamp,
        "name": emoji
    });

    let response = client
        .post("https://slack.com/api/reactions.add", &payload)
        .await?;

    let json: serde_json::Value = response.json().await?;
    let success: bool = json["ok"].as_bool().unwrap_or(false);

    if !success {
        let error = json["error"].as_str().unwrap_or("Unknown error");
        return Err(anyhow::anyhow!("reactions.add failed: {}", error));
    }

    Ok(())
}

/// Remove a reaction from a message.
///
/// # Arguments
///
/// * `client` - The Slack client handle
/// * `channel_id` - The channel ID containing the message
/// * `timestamp` - The message timestamp
/// * `emoji` - The emoji name (without colons)
///
/// # Returns
///
/// * `Ok(())` - Reaction was removed successfully
/// * `Err(anyhow::Error)` - An error if removal fails
pub async fn remove_reaction(
    client: &crate::connection::SlackClientHandle,
    channel_id: &str,
    timestamp: &str,
    emoji: &str,
) -> Result<()> {
    let payload = serde_json::json!({
        "channel": channel_id,
        "timestamp": timestamp,
        "name": emoji
    });

    let response = client
        .post("https://slack.com/api/reactions.remove", &payload)
        .await?;

    let json: serde_json::Value = response.json().await?;
    let success: bool = json["ok"].as_bool().unwrap_or(false);

    if !success {
        let error = json["error"].as_str().unwrap_or("Unknown error");
        return Err(anyhow::anyhow!("reactions.remove failed: {}", error));
    }

    Ok(())
}

/// Get reactions on a message.
///
/// # Arguments
///
/// * `client` - The Slack client handle
/// * `channel_id` - The channel ID containing the message
/// * `timestamp` - The message timestamp
///
/// # Returns
///
/// * `Ok(ReactionsListResponse)` - The reactions list response
/// * `Err(anyhow::Error)` - An error if the request fails
pub async fn get_message_reactions(
    client: &crate::connection::SlackClientHandle,
    channel_id: &str,
    timestamp: &str,
) -> Result<ReactionsListResponse> {
    let payload = serde_json::json!({
        "channel": channel_id,
        "timestamp": timestamp
    });

    let response = client
        .post("https://slack.com/api/reactions.get", &payload)
        .await?;

    let json: serde_json::Value = response.json().await?;
    let parsed: ReactionsListResponse = serde_json::from_value(json)?;

    Ok(parsed)
}

/// Build the payload for a typing indicator.
///
/// Note: Slack doesn't have a native typing indicator API for bots.
/// This is a placeholder for future implementation or alternative approaches.
///
/// # Arguments
///
/// * `channel_id` - The channel ID
///
/// # Returns
///
/// A JSON value representing the typing payload
pub fn build_typing_payload(channel_id: &str) -> serde_json::Value {
    serde_json::json!({
        "channel": channel_id
    })
}

/// Send a "thinking..." ephemeral message to indicate processing.
///
/// This is a workaround for the lack of native typing indicators in Slack.
///
/// # Arguments
///
/// * `client` - The Slack client handle
/// * `channel_id` - The channel ID
/// * `user_id` - The user ID to show the message to
/// * `text` - The thinking message text
///
/// # Returns
///
/// * `Ok(())` - Message was sent successfully
/// * `Err(anyhow::Error)` - An error if sending fails
pub async fn send_thinking_message(
    client: &crate::connection::SlackClientHandle,
    channel_id: &str,
    user_id: &str,
    text: &str,
) -> Result<()> {
    let options = crate::send::SendOptions {
        thread_ts: None,
        reply_broadcast: None,
        mrkdwn: Some(true),
        ..Default::default()
    };

    let response: SendMessageResponse =
        crate::send::send_ephemeral_message(client, channel_id, user_id, text, Some(&options))
            .await?;

    if response.is_ok() {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "Failed to send thinking message: {:?}",
            response.get_error()
        ))
    }
}

/// Mark a channel as read.
///
/// # Arguments
///
/// * `client` - The Slack client handle
/// * `channel_id` - The channel ID to mark as read
///
/// # Returns
///
/// * `Ok(())` - Channel was marked as read
/// * `Err(anyhow::Error)` - An error if marking fails
pub async fn mark_channel_as_read(
    client: &crate::connection::SlackClientHandle,
    channel_id: &str,
) -> Result<()> {
    let payload = serde_json::json!({
        "channel": channel_id
    });

    let response = client
        .post("https://slack.com/api/conversations.mark", &payload)
        .await?;

    let json: serde_json::Value = response.json().await?;
    let success: bool = json["ok"].as_bool().unwrap_or(false);

    if !success {
        let error = json["error"].as_str().unwrap_or("Unknown error");
        return Err(anyhow::anyhow!("conversations.mark failed: {}", error));
    }

    Ok(())
}

/// Pin a message to a channel.
///
/// # Arguments
///
/// * `client` - The Slack client handle
/// * `channel_id` - The channel ID
/// * `timestamp` - The message timestamp to pin
///
/// # Returns
///
/// * `Ok(())` - Message was pinned successfully
/// * `Err(anyhow::Error)` - An error if pinning fails
pub async fn pin_message(
    client: &crate::connection::SlackClientHandle,
    channel_id: &str,
    timestamp: &str,
) -> Result<()> {
    let payload = serde_json::json!({
        "channel": channel_id,
        "timestamp": timestamp
    });

    let response = client
        .post("https://slack.com/api/pins.add", &payload)
        .await?;

    let json: serde_json::Value = response.json().await?;
    let success: bool = json["ok"].as_bool().unwrap_or(false);

    if !success {
        let error = json["error"].as_str().unwrap_or("Unknown error");
        return Err(anyhow::anyhow!("pins.add failed: {}", error));
    }

    Ok(())
}

/// Unpin a message from a channel.
///
/// # Arguments
///
/// * `client` - The Slack client handle
/// * `channel_id` - The channel ID
/// * `timestamp` - The message timestamp to unpin
///
/// # Returns
///
/// * `Ok(())` - Message was unpinned successfully
/// * `Err(anyhow::Error)` - An error if unpinning fails
pub async fn unpin_message(
    client: &crate::connection::SlackClientHandle,
    channel_id: &str,
    timestamp: &str,
) -> Result<()> {
    let payload = serde_json::json!({
        "channel": channel_id,
        "timestamp": timestamp
    });

    let response = client
        .post("https://slack.com/api/pins.remove", &payload)
        .await?;

    let json: serde_json::Value = response.json().await?;
    let success: bool = json["ok"].as_bool().unwrap_or(false);

    if !success {
        let error = json["error"].as_str().unwrap_or("Unknown error");
        return Err(anyhow::anyhow!("pins.remove failed: {}", error));
    }

    Ok(())
}

/// Archive a channel.
///
/// # Arguments
///
/// * `client` - The Slack client handle
/// * `channel_id` - The channel ID to archive
///
/// # Returns
///
/// * `Ok(())` - Channel was archived successfully
/// * `Err(anyhow::Error)` - An error if archiving fails
pub async fn archive_channel(
    client: &crate::connection::SlackClientHandle,
    channel_id: &str,
) -> Result<()> {
    let payload = serde_json::json!({
        "channel": channel_id
    });

    let response = client
        .post("https://slack.com/api/conversations.archive", &payload)
        .await?;

    let json: serde_json::Value = response.json().await?;
    let success: bool = json["ok"].as_bool().unwrap_or(false);

    if !success {
        let error = json["error"].as_str().unwrap_or("Unknown error");
        return Err(anyhow::anyhow!("conversations.archive failed: {}", error));
    }

    Ok(())
}

/// Unarchive a channel.
///
/// # Arguments
///
/// * `client` - The Slack client handle
/// * `channel_id` - The channel ID to unarchive
///
/// # Returns
///
/// * `Ok(())` - Channel was unarchived successfully
/// * `Err(anyhow::Error)` - An error if unarchiving fails
pub async fn unarchive_channel(
    client: &crate::connection::SlackClientHandle,
    channel_id: &str,
) -> Result<()> {
    let payload = serde_json::json!({
        "channel": channel_id
    });

    let response = client
        .post("https://slack.com/api/conversations.unarchive", &payload)
        .await?;

    let json: serde_json::Value = response.json().await?;
    let success: bool = json["ok"].as_bool().unwrap_or(false);

    if !success {
        let error = json["error"].as_str().unwrap_or("Unknown error");
        return Err(anyhow::anyhow!("conversations.unarchive failed: {}", error));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_list_channels_payload() {
        let payload =
            build_list_channels_payload(Some("cursor123"), Some(100), Some("public,private"));

        assert_eq!(payload["cursor"], "cursor123");
        assert_eq!(payload["limit"], 100);
        assert_eq!(payload["types"], "public,private");
    }

    #[test]
    fn test_build_typing_payload() {
        let payload = build_typing_payload("C123456");
        assert_eq!(payload["channel"], "C123456");
    }

    #[test]
    fn test_channel_info_serialization() {
        let channel = ChannelInfo {
            id: "C123456".to_string(),
            name: Some("general".to_string()),
            name_normalized: Some("general".to_string()),
            purpose: Some(ChannelPurpose {
                value: Some("For general discussion".to_string()),
                last_set: Some(1234567890),
            }),
            topic: None,
            num_members: Some(10),
            is_public: Some(true),
            is_private: Some(false),
            is_im: Some(false),
            is_mpim: Some(false),
            is_archived: Some(false),
            is_general: Some(true),
            members: Some(vec!["U1".to_string(), "U2".to_string()]),
            last_read: None,
            latest: None,
            unread_count: None,
            unread_count_display: None,
            created: Some(1234567890),
            creator: None,
        };

        let json = serde_json::to_string(&channel).unwrap();
        assert!(json.contains("C123456"));
        assert!(json.contains("general"));
    }

    #[test]
    fn test_user_info_serialization() {
        let user = UserInfo {
            id: "U123456".to_string(),
            name: Some("john".to_string()),
            display_name: Some("John Doe".to_string()),
            real_name: Some("John Doe".to_string()),
            profile: None,
            deleted: Some(false),
            is_bot: Some(false),
            tz: Some("America/Los_Angeles".to_string()),
            tz_label: None,
            tz_offset: None,
            last_active: None,
            online: None,
            is_admin: None,
            is_owner: None,
            is_primary_owner: None,
            is_restricted: None,
            is_ultra_restricted: None,
            profile_image: None,
        };

        let json = serde_json::to_string(&user).unwrap();
        assert!(json.contains("U123456"));
        assert!(json.contains("john"));
    }
}
