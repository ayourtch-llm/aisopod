//! Message receiving and normalization for Slack.
//!
//! This module provides utilities for parsing incoming Slack events
//! and normalizing them to the shared IncomingMessage type.

use aisopod_channel::message::{IncomingMessage, Media, MessageContent, PeerInfo, PeerKind, SenderInfo};
use aisopod_channel::types::MediaType;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::socket_mode::{MessageEvent, EventsApiEvent, EventsApiPayload, SocketModePayload};

/// Check if a Slack message should be filtered based on channel/user settings.
///
/// # Arguments
///
/// * `config` - The Slack account configuration
/// * `channel_id` - The channel ID where the message was posted
/// * `user_id` - The user ID who sent the message
/// * `bot_user_id` - The bot's own user ID (for filtering self-messages)
///
/// # Returns
///
/// * `true` - The message should be filtered out
/// * `false` - The message should be processed
pub fn should_filter_message(
    config: &crate::SlackAccountConfig,
    channel_id: &str,
    user_id: Option<&str>,
    bot_user_id: Option<&str>,
) -> bool {
    // Filter out messages from the bot itself
    if let Some(bot_id) = bot_user_id {
        if user_id == Some(bot_id) {
            debug!("Filtering self-message from bot user {}", bot_id);
            return true;
        }
    }

    // Filter messages from channels not in the allowed list
    if let Some(ref allowed_channels) = config.allowed_channels {
        if !allowed_channels.contains(&channel_id.to_string()) {
            debug!("Filtering message from channel {} (not in allowed list)", channel_id);
            return true;
        }
    }

    // Filter messages from users not in the allowed list
    if let Some(ref allowed_users) = config.allowed_users {
        if let Some(user_id) = user_id {
            if !allowed_users.contains(&user_id.to_string()) {
                debug!("Filtering message from user {} (not in allowed list)", user_id);
                return true;
            }
        }
    }

    // Filter messages that don't contain a mention if mention_required is true
    if config.mention_required {
        if let Some(user_id) = user_id {
            if let Some(bot_id) = bot_user_id {
                // Check if the message contains a mention of the bot
                // This is a simplified check - in production, you'd parse the text for mentions
                warn!("mention_required is true but mention detection is simplified");
            }
        }
    }

    false
}

/// Normalize a Slack message event to the shared IncomingMessage type.
///
/// # Arguments
///
/// * `event` - The Slack message event to normalize
/// * `account_id` - The account ID for this channel
/// * `channel` - The channel identifier
/// * `bot_user_id` - The bot's user ID (for filtering)
///
/// # Returns
///
/// * `Ok(IncomingMessage)` - The normalized message
/// * `Err(anyhow::Error)` - An error if normalization fails
pub fn normalize_message(
    event: &MessageEvent,
    account_id: &str,
    channel: &str,
    bot_user_id: Option<&str>,
) -> Result<IncomingMessage> {
    // Extract sender info
    let sender_id = event.user.as_deref()
        .ok_or_else(|| anyhow!("Message event missing user ID"))?;
    
    let sender = SenderInfo {
        id: sender_id.to_string(),
        display_name: None,
        username: None,
        is_bot: event.bot_id.is_some(),
    };

    // Extract channel ID and determine message type (DM, channel, group, thread)
    let channel_id = &event.channel;
    
    // Determine peer info based on channel type
    let peer = match channel_id.chars().next() {
        Some('D') => PeerInfo {
            id: channel_id.to_string(),
            kind: PeerKind::User,
            title: None,
        },
        Some('C') => PeerInfo {
            id: channel_id.to_string(),
            kind: PeerKind::Channel,
            title: None,
        },
        Some('G') => PeerInfo {
            id: channel_id.to_string(),
            kind: PeerKind::Group,
            title: None,
        },
        _ => PeerInfo {
            id: channel_id.to_string(),
            kind: PeerKind::User,
            title: None,
        },
    };

    // Build the message content
    let content = match event.text.as_deref() {
        Some(text) => MessageContent::Text(text.to_string()),
        None => MessageContent::Text("[Empty message]".to_string()),
    };

    // Parse timestamp
    // Slack timestamps are in seconds with decimal places for sub-second precision
    let ts_parts: Vec<&str> = event.ts.split('.').collect();
    let secs = ts_parts[0].parse::<i64>()
        .map_err(|_| anyhow!("Invalid timestamp seconds: {}", event.ts))?;
    let nanos = ts_parts.get(1)
        .map(|s| s.parse::<u32>().unwrap_or(0))
        .unwrap_or(0);
    
    // Scale to nanoseconds (pad with zeros if needed)
    let nanos = match ts_parts.get(1) {
        Some(s) if s.len() <= 9 => {
            let padded = format!("{:<09}", s);
            padded[..9].parse::<u32>().unwrap_or(0)
        }
        _ => 0,
    };
    
    let timestamp = DateTime::from_timestamp(secs, nanos)
        .ok_or_else(|| anyhow!("Invalid timestamp: {}", event.ts))?
        .with_timezone(&Utc);

    // Check for thread context
    let thread_ts = event.thread_ts.as_deref();

    // Build the incoming message
    let mut incoming = IncomingMessage {
        id: format!("{}-{}", channel_id, event.ts),
        channel: channel.to_string(),
        account_id: account_id.to_string(),
        sender,
        peer,
        content,
        reply_to: None, // Would need additional processing to determine this
        timestamp,
        metadata: serde_json::json!({
            "thread_ts": thread_ts,
            "channel_type": event.channel_type.as_deref().unwrap_or("unknown"),
        }),
    };

    // If this is a thread reply, store the parent message ID in reply_to
    if let Some(thread_ts) = thread_ts {
        if thread_ts != event.ts {
            // This is a reply to a thread - in a full implementation, 
            // you'd lookup the original message ID from the thread
            incoming.reply_to = Some(thread_ts.to_string());
        }
    }

    Ok(incoming)
}

/// Process a Slack Socket Mode event and return normalized messages if applicable.
///
/// This function handles different types of Socket Mode events and normalizes
/// message events to IncomingMessage.
///
/// # Arguments
///
/// * `payload` - The Socket Mode payload
/// * `account_id` - The account ID for this channel
/// * `channel` - The channel identifier
/// * `bot_user_id` - The bot's user ID (for filtering)
///
/// # Returns
///
/// * `Ok(Vec<IncomingMessage>)` - Normalized messages (may be empty)
/// * `Err(anyhow::Error)` - An error if processing fails
pub fn process_slack_message(
    payload: &SocketModePayload,
    account_id: &str,
    channel: &str,
    bot_user_id: Option<&str>,
) -> Result<Vec<IncomingMessage>> {
    match payload {
        SocketModePayload::EventsApi(events_api) => {
            match &events_api.event {
                EventsApiEvent::Message(message) => {
                    // Apply filtering
                    let should_filter = should_filter_message(
                        &crate::SlackAccountConfig::default(),
                        &message.channel,
                        message.user.as_deref(),
                        bot_user_id,
                    );

                    if should_filter {
                        info!("Filtering message from channel {}", message.channel);
                        return Ok(vec![]);
                    }

                    // Normalize the message
                    let incoming = normalize_message(message, account_id, channel, bot_user_id)?;
                    Ok(vec![incoming])
                }
                EventsApiEvent::AppMention(mention) => {
                    // Similar to message but for app mentions
                    let message = MessageEvent {
                        event_type: "app_mention".to_string(),
                        channel: mention.channel.clone(),
                        thread_ts: None,
                        user: Some(mention.user.clone()),
                        text: Some(mention.text.clone()),
                        bot_id: None,
                        ts: mention.ts.clone(),
                        subtype: None,
                        channel_type: None,
                        files: None,
                    };
                    
                    let incoming = normalize_message(&message, account_id, channel, bot_user_id)?;
                    Ok(vec![incoming])
                }
                EventsApiEvent::Other(_) => {
                    // Other event types - not supported yet
                    debug!("Skipping unsupported event type");
                    Ok(vec![])
                }
            }
        }
        SocketModePayload::Hello(_hello) => {
            // Hello events are connection status, not messages
            Ok(vec![])
        }
        SocketModePayload::Disconnect(_disconnect) => {
            // Disconnect events are connection status, not messages
            Ok(vec![])
        }
        SocketModePayload::Other(_value) => {
            // Unknown event type
            debug!("Skipping unknown event type");
            Ok(vec![])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_message_dm() {
        let event = MessageEvent {
            event_type: "message".to_string(),
            channel: "D123456".to_string(),
            thread_ts: None,
            user: Some("U123456".to_string()),
            text: Some("Hello, world!".to_string()),
            bot_id: None,
            ts: "1234567890.123456".to_string(),
            subtype: None,
            channel_type: Some("im".to_string()),
            files: None,
        };

        let result = normalize_message(&event, "test-account", "slack", Some("U999999"));
        assert!(result.is_ok());
        let message = result.unwrap();
        assert_eq!(message.channel, "slack");
        assert_eq!(message.account_id, "test-account");
        assert_eq!(message.sender.id, "U123456");
        assert_eq!(message.peer.kind, PeerKind::User);
        assert_eq!(message.peer.id, "D123456");
    }

    #[test]
    fn test_normalize_message_channel() {
        let event = MessageEvent {
            event_type: "message".to_string(),
            channel: "C123456".to_string(),
            thread_ts: None,
            user: Some("U123456".to_string()),
            text: Some("Channel message".to_string()),
            bot_id: None,
            ts: "1234567890.123456".to_string(),
            subtype: None,
            channel_type: Some("channel".to_string()),
            files: None,
        };

        let result = normalize_message(&event, "test-account", "slack", None);
        assert!(result.is_ok());
        let message = result.unwrap();
        assert_eq!(message.peer.kind, PeerKind::Channel);
        assert_eq!(message.peer.id, "C123456");
    }

    #[test]
    fn test_normalize_message_thread() {
        let event = MessageEvent {
            event_type: "message".to_string(),
            channel: "C123456".to_string(),
            thread_ts: Some("1234567890.000000".to_string()),
            user: Some("U123456".to_string()),
            text: Some("Thread reply".to_string()),
            bot_id: None,
            ts: "1234567890.123456".to_string(),
            subtype: None,
            channel_type: Some("channel".to_string()),
            files: None,
        };

        let result = normalize_message(&event, "test-account", "slack", None);
        assert!(result.is_ok());
        let message = result.unwrap();
        // Thread reply should have reply_to set
        assert!(message.reply_to.is_some());
    }

    #[test]
    fn test_filter_self_message() {
        let config = crate::SlackAccountConfig::default();
        let filtered = should_filter_message(&config, "C123456", Some("U999999"), Some("U999999"));
        assert!(filtered, "Self-message should be filtered");
    }

    #[test]
    fn test_filter_allowed_channel() {
        let config = crate::SlackAccountConfig {
            allowed_channels: Some(vec!["C111111".to_string(), "C222222".to_string()]),
            ..Default::default()
        };
        
        // Should filter channel not in allowed list
        let filtered = should_filter_message(&config, "C999999", Some("U123456"), None);
        assert!(filtered, "Channel not in allowed list should be filtered");
        
        // Should not filter channel in allowed list
        let filtered = should_filter_message(&config, "C111111", Some("U123456"), None);
        assert!(!filtered, "Channel in allowed list should not be filtered");
    }

    #[test]
    fn test_process_message_event() {
        let payload = SocketModePayload::EventsApi(EventsApiPayload {
            event: EventsApiEvent::Message(MessageEvent {
                event_type: "message".to_string(),
                channel: "D123456".to_string(),
                thread_ts: None,
                user: Some("U123456".to_string()),
                text: Some("Hello".to_string()),
                bot_id: None,
                ts: "1234567890.123456".to_string(),
                subtype: None,
                channel_type: Some("im".to_string()),
                files: None,
            }),
            event_id: "Ev123456".to_string(),
            event_time: "1234567890".to_string(),
            token: "verification-token".to_string(),
        });

        let result = process_slack_message(&payload, "test-account", "slack", Some("U999999"));
        assert!(result.is_ok());
        let messages = result.unwrap();
        assert_eq!(messages.len(), 1);
    }
}
