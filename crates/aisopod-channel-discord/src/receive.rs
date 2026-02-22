//! Message receiving and normalization for Discord channel.
//!
//! This module handles incoming Discord messages, filtering, and normalization
//! to the shared `IncomingMessage` type.

use crate::DiscordAccountConfig;
use aisopod_channel::message::{IncomingMessage, Media, MessageContent, MessagePart, PeerInfo, PeerKind, SenderInfo};
use anyhow::Result;
use chrono::{DateTime, Utc};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Filter a message based on allowlist settings.
///
/// # Arguments
///
/// * `config` - The Discord account configuration
/// * `guild_id` - Optional guild ID where the message was sent
/// * `channel_id` - Channel ID where the message was sent
///
/// # Returns
///
/// * `true` - Message should be filtered out
/// * `false` - Message should be processed
pub fn should_filter_message(
    config: &DiscordAccountConfig,
    guild_id: Option<u64>,
    channel_id: u64,
) -> bool {
    // Check if guild is allowed
    if let Some(ref allowed_guilds) = config.allowed_guilds {
        if let Some(gid) = guild_id {
            if !allowed_guilds.contains(&gid) {
                debug!("Message filtered: guild {} not in allowed list", gid);
                return true;
            }
        }
    }

    // Check if channel is allowed
    if let Some(ref allowed_channels) = config.allowed_channels {
        if !allowed_channels.contains(&channel_id) {
            debug!("Message filtered: channel {} not in allowed list", channel_id);
            return true;
        }
    }

    false
}

/// Check if a message should be processed based on mention requirements.
///
/// # Arguments
///
/// * `config` - The Discord account configuration
/// * `mentions` - List of users mentioned in the message
/// * `bot_user_id` - The bot's user ID
///
/// # Returns
///
/// * `true` - Message should be processed
/// * `false` - Message should be filtered due to missing mention
pub fn check_mention_requirement(
    config: &DiscordAccountConfig,
    mentions: &[serenity::all::User],
    bot_user_id: Option<u64>,
) -> bool {
    if !config.mention_required_in_channels {
        return true;
    }

    // Check if the bot is mentioned in the message
    let bot_id = bot_user_id.unwrap_or(0);
    mentions.iter().any(|user| user.id.get() == bot_id)
}

/// Normalize a Discord message to the shared IncomingMessage type.
///
/// # Arguments
///
/// * `discord_msg` - The raw Discord message
/// * `account_id` - The account ID that received the message
/// * `channel_id` - Optional canonical channel ID override
///
/// # Returns
///
/// * `Ok(IncomingMessage)` - The normalized message
/// * `Err(anyhow::Error)` - An error if normalization fails
pub fn normalize_message(
    discord_msg: &serenity::all::Message,
    account_id: &str,
    _channel_id: Option<String>,
) -> Result<IncomingMessage> {
    let msg = discord_msg;

    // Get channel and guild IDs using public methods
    let channel_id_val = msg.channel_id.get();
    let guild_id = msg.guild_id.map(|g| g.get());

    // Determine peer kind based on guild presence and flags
    let peer_kind = if msg.guild_id.is_some() {
        // Message in a guild
        // Check thread status using flags field
        if let Some(flags) = msg.flags {
            if flags.contains(serenity::all::MessageFlags::HAS_THREAD) {
                // Message in a thread
                PeerKind::Thread
            } else {
                // Regular guild channel
                PeerKind::Group
            }
        } else {
            // Regular guild channel
            PeerKind::Group
        }
    } else {
        // Direct message
        PeerKind::User
    };

    // For DMs, use the recipient's name; for guilds, we don't have guild_name in v0.12
    // so we use the channel name or a generic label
    let peer_title = if msg.guild_id.is_some() {
        // In guilds, we can't easily get the guild name without additional API calls
        // Use a generic label based on channel type
        "Server Channel".to_string()
    } else {
        // For DMs, use the recipient's name
        msg.author.name.clone()
    };

    let peer = PeerInfo {
        id: msg.channel_id.to_string(),
        kind: peer_kind,
        title: Some(peer_title),
    };

    // Extract sender information
    let display_name = msg.author.global_name.as_ref().map(|s| s.as_str()).or(Some(msg.author.name.as_str()));
    let sender = SenderInfo {
        id: msg.author.id.to_string(),
        display_name: display_name.map(|s| s.to_string()),
        username: Some(msg.author.tag()),
        is_bot: msg.author.bot,
    };

    // Extract message content
    let content = if !msg.content.is_empty() {
        MessageContent::Text(msg.content.clone())
    } else if !msg.embeds.is_empty() {
        // Extract text from embeds if message content is empty
        let embed_texts: Vec<String> = msg
            .embeds
            .iter()
            .filter_map(|embed| embed.description.as_ref().map(|s| s.clone()))
            .collect();
        if embed_texts.is_empty() {
            MessageContent::Text("[No text content]".to_string())
        } else {
            MessageContent::Mixed(
                embed_texts
                    .into_iter()
                    .map(|s| MessagePart::Text(s))
                    .collect(),
            )
        }
    } else {
        MessageContent::Text("[Empty message]".to_string())
    };

    // Extract timestamp - convert from serenity::Timestamp to DateTime<Utc>
    let timestamp = (*msg.timestamp).into();

    // Extract reply_to if present
    let reply_to = msg
        .referenced_message
        .as_ref()
        .map(|msg| msg.id.to_string())
        .or_else(|| msg.message_reference.as_ref().and_then(|r| r.message_id.map(|id| id.get().to_string())));

    // Create unique message ID
    let message_id = format!(
        "discord:{}:{}",
        guild_id.unwrap_or(0),
        msg.id.get()
    );

    // Build metadata
    let metadata = serde_json::json!({
        "discord": {
            "message_id": msg.id.to_string(),
            "channel_id": msg.channel_id.to_string(),
            "guild_id": guild_id.map(|g| g.to_string()),
            "author_id": msg.author.id.to_string(),
            "author_tag": msg.author.tag(),
        }
    });

    Ok(IncomingMessage {
        id: message_id,
        channel: "discord".to_string(),
        account_id: account_id.to_string(),
        sender,
        peer,
        content,
        reply_to,
        timestamp,
        metadata,
    })
}

/// Process a Discord message through the filtering pipeline.
///
/// # Arguments
///
/// * `config` - The Discord account configuration
/// * `discord_msg` - The raw Discord message
/// * `account_id` - The account ID that received the message
/// * `bot_user_id` - The bot's user ID for mention checking
///
/// # Returns
///
/// * `Ok(Some(IncomingMessage))` - Message passed filters and was normalized
/// * `Ok(None)` - Message was filtered out
/// * `Err(anyhow::Error)` - An error occurred during processing
pub fn process_discord_message(
    config: &DiscordAccountConfig,
    discord_msg: &serenity::all::Message,
    account_id: &str,
    bot_user_id: Option<u64>,
) -> Result<Option<IncomingMessage>> {
    // Check if message is from the bot itself
    if discord_msg.author.bot {
        debug!("Message filtered: from bot itself");
        return Ok(None);
    }

    // Check mention requirement
    if !check_mention_requirement(config, &discord_msg.mentions, bot_user_id) {
        debug!("Message filtered: mention required but bot not mentioned");
        return Ok(None);
    }

    // Check allowlist
    let guild_id = discord_msg.guild_id.map(|g| g.get());
    let channel_id = discord_msg.channel_id.get();

    if should_filter_message(config, guild_id, channel_id) {
        return Ok(None);
    }

    // Normalize the message
    normalize_message(discord_msg, account_id, None).map(Some)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serenity::model::{channel::Message, id::{ChannelId, GuildId, MessageId, UserId}, user::User};
    use std::num::NonZeroU16;

    fn create_test_message(content: &str, is_bot: bool) -> Message {
        let mut author = User::default();
        author.id = UserId::new(if is_bot { 999 } else { 100 });
        author.name = "testuser".to_string();
        author.discriminator = Some(NonZeroU16::new(1).unwrap());
        author.bot = is_bot;

        let channel_id = ChannelId::new(456);
        let message_id = MessageId::new(123);
        let mut message = Message::default();
        message.id = message_id;
        message.channel_id = channel_id;
        message.guild_id = Some(GuildId::new(789));
        message.author = author;
        message.content = content.to_string();
        message.timestamp = serenity::model::Timestamp::now();
        message
    }

    #[test]
    fn test_should_filter_message_no_allowlist() {
        let config = DiscordAccountConfig {
            allowed_guilds: None,
            allowed_channels: None,
            ..Default::default()
        };

        assert!(!should_filter_message(&config, Some(123), 456));
        assert!(!should_filter_message(&config, None, 456));
    }

    #[test]
    fn test_should_filter_message_with_guild_allowlist() {
        let config = DiscordAccountConfig {
            allowed_guilds: Some(vec![123, 456]),
            allowed_channels: None,
            ..Default::default()
        };

        // Should not filter - guild is in allowlist
        assert!(!should_filter_message(&config, Some(123), 789));

        // Should filter - guild is not in allowlist
        assert!(should_filter_message(&config, Some(999), 789));
    }

    #[test]
    fn test_should_filter_message_with_channel_allowlist() {
        let config = DiscordAccountConfig {
            allowed_guilds: None,
            allowed_channels: Some(vec![111, 222]),
            ..Default::default()
        };

        // Should not filter - channel is in allowlist
        assert!(!should_filter_message(&config, Some(123), 111));

        // Should filter - channel is not in allowlist
        assert!(should_filter_message(&config, Some(123), 999));
    }

    #[test]
    fn test_should_filter_message_with_both_allowlists() {
        let config = DiscordAccountConfig {
            allowed_guilds: Some(vec![123]),
            allowed_channels: Some(vec![111, 222]),
            ..Default::default()
        };

        // Both match - should not filter
        assert!(!should_filter_message(&config, Some(123), 111));

        // Guild matches, channel doesn't - should filter
        assert!(should_filter_message(&config, Some(123), 999));

        // Channel matches, guild doesn't - should filter
        assert!(should_filter_message(&config, Some(999), 111));
    }

    #[test]
    fn test_should_filter_message_bot_itself() {
        let config = DiscordAccountConfig::default();
        let bot_user_id = Some(999);

        let message = create_test_message("test", true); // is_bot = true
        let result = process_discord_message(&config, &message, "test_account", bot_user_id);

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_should_filter_message_mention_required() {
        let config = DiscordAccountConfig {
            mention_required_in_channels: true,
            ..Default::default()
        };
        let bot_user_id = Some(999);

        // Message without bot mention
        let message = create_test_message("test without mention", false);
        let result = process_discord_message(&config, &message, "test_account", bot_user_id);

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_should_filter_message_not_mention_required() {
        let config = DiscordAccountConfig {
            mention_required_in_channels: false,
            ..Default::default()
        };
        let bot_user_id = Some(999);

        // Even without mention, should not be filtered
        let message = create_test_message("test without mention", false);
        let result = process_discord_message(&config, &message, "test_account", bot_user_id);

        assert!(result.is_ok());
        let msg = result.unwrap();
        assert!(msg.is_some());
        assert_eq!(msg.unwrap().content_to_string(), "test without mention");
    }

    #[test]
    fn test_mention_checking() {
        let config = DiscordAccountConfig {
            mention_required_in_channels: true,
            ..Default::default()
        };
        let bot_user_id = Some(999);

        // Create message with bot mentioned
        let mut user1 = User::default();
        user1.id = UserId::new(100);
        user1.name = "test1".to_string();
        user1.discriminator = Some(NonZeroU16::new(1).unwrap());
        user1.bot = false;
        
        let mut user2 = User::default();
        user2.id = UserId::new(999);
        user2.name = "test2".to_string();
        user2.discriminator = Some(NonZeroU16::new(2).unwrap());
        user2.bot = false;

        let mentions = vec![user1, user2];

        assert!(check_mention_requirement(&config, &mentions, bot_user_id));

        // Create message without bot mentioned
        let mut user3 = User::default();
        user3.id = UserId::new(100);
        user3.name = "test3".to_string();
        user3.discriminator = Some(NonZeroU16::new(3).unwrap());
        user3.bot = false;
        
        let mut user4 = User::default();
        user4.id = UserId::new(200);
        user4.name = "test4".to_string();
        user4.discriminator = Some(NonZeroU16::new(4).unwrap());
        user4.bot = false;

        let mentions = vec![user3, user4];

        assert!(!check_mention_requirement(&config, &mentions, bot_user_id));
    }
}
