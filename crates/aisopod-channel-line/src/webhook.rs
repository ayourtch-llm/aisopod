//! LINE webhook handling.
//!
//! This module provides webhook support for receiving events from LINE,
//! including signature verification and event parsing.

use anyhow::Result;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::HashMap;
use tracing::{debug, error};

/// LINE webhook signature header name.
pub const LINE_SIGNATURE_HEADER: &str = "X-Line-Signature";

/// Webhook event types.
#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum WebhookEventType {
    #[serde(rename = "message")]
    Message(MessageEvent),
    
    #[serde(rename = "follow")]
    Follow(FollowEvent),
    
    #[serde(rename = "unfollow")]
    Unfollow(UnfollowEvent),
    
    #[serde(rename = "join")]
    Join(JoinEvent),
    
    #[serde(rename = "leave")]
    Leave(LeaveEvent),
    
    #[serde(rename = "postback")]
    Postback(PostbackEvent),
    
    #[serde(rename = "beacon")]
    Beacon(BeaconEvent),
}

/// Message event.
#[derive(Debug, Deserialize, Clone)]
pub struct MessageEvent {
    #[serde(rename = "replyToken")]
    pub reply_token: String,
    
    #[serde(rename = "timestamp")]
    pub timestamp: u64,
    
    #[serde(rename = "source")]
    pub source: EventSource,
    
    pub message: MessageContent,
}

/// Follow event.
#[derive(Debug, Deserialize, Clone)]
pub struct FollowEvent {
    #[serde(rename = "replyToken")]
    pub reply_token: String,
    
    #[serde(rename = "timestamp")]
    pub timestamp: u64,
    
    #[serde(rename = "source")]
    pub source: EventSource,
}

/// Unfollow event.
#[derive(Debug, Deserialize, Clone)]
pub struct UnfollowEvent {
    #[serde(rename = "replyToken")]
    pub reply_token: String,
    
    #[serde(rename = "timestamp")]
    pub timestamp: u64,
    
    #[serde(rename = "source")]
    pub source: EventSource,
}

/// Join event.
#[derive(Debug, Deserialize, Clone)]
pub struct JoinEvent {
    #[serde(rename = "replyToken")]
    pub reply_token: String,
    
    #[serde(rename = "timestamp")]
    pub timestamp: u64,
    
    #[serde(rename = "source")]
    pub source: EventSource,
}

/// Leave event.
#[derive(Debug, Deserialize, Clone)]
pub struct LeaveEvent {
    #[serde(rename = "replyToken")]
    pub reply_token: String,
    
    #[serde(rename = "timestamp")]
    pub timestamp: u64,
    
    #[serde(rename = "source")]
    pub source: EventSource,
}

/// Postback event.
#[derive(Debug, Deserialize, Clone)]
pub struct PostbackEvent {
    #[serde(rename = "replyToken")]
    pub reply_token: String,
    
    #[serde(rename = "timestamp")]
    pub timestamp: u64,
    
    #[serde(rename = "source")]
    pub source: EventSource,
    
    pub postback: PostbackContent,
}

/// Beacon event.
#[derive(Debug, Deserialize, Clone)]
pub struct BeaconEvent {
    #[serde(rename = "replyToken")]
    pub reply_token: String,
    
    #[serde(rename = "timestamp")]
    pub timestamp: u64,
    
    #[serde(rename = "source")]
    pub source: EventSource,
    
    pub beacon: BeaconContent,
}

/// Event source.
#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum EventSource {
    #[serde(rename = "user")]
    User { #[serde(rename = "userId")] user_id: String },
    
    #[serde(rename = "group")]
    Group { 
        #[serde(rename = "groupId")] group_id: String,
        #[serde(rename = "userId")] user_id: Option<String>,
    },
    
    #[serde(rename = "room")]
    Room {
        #[serde(rename = "roomId")] room_id: String,
        #[serde(rename = "userId")] user_id: Option<String>,
    },
}

/// Message content.
#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum MessageContent {
    #[serde(rename = "text")]
    Text { id: String, text: String },
    
    #[serde(rename = "image")]
    Image { id: String },
    
    #[serde(rename = "video")]
    Video { id: String },
    
    #[serde(rename = "audio")]
    Audio { id: String, duration: u64 },
    
    #[serde(rename = "file")]
    File { id: String, file_name: String, file_size: u64 },
    
    #[serde(rename = "location")]
    Location {
        id: String,
        title: String,
        address: String,
        #[serde(rename = "latitude")]
        latitude: f64,
        #[serde(rename = "longitude")]
        longitude: f64,
    },
    
    #[serde(rename = "sticker")]
    Sticker {
        id: String,
        package_id: String,
        sticker_id: String,
    },
}

/// Postback content.
#[derive(Debug, Deserialize, Clone)]
pub struct PostbackContent {
    pub data: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<HashMap<String, String>>,
}

/// Beacon content.
#[derive(Debug, Deserialize, Clone)]
pub struct BeaconContent {
    pub hwid: String,
    #[serde(rename = "type")]
    pub beacon_type: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dm: Option<String>,
}

/// Webhook request body.
#[derive(Debug, Deserialize)]
pub struct WebhookRequestBody {
    #[serde(rename = "destination")]
    pub destination: String,
    
    pub events: Vec<WebhookEventType>,
}

/// Verify the webhook signature.
///
/// # Arguments
///
/// * `channel_secret` - The channel secret from LINE
/// * `body` - The raw request body
/// * `signature` - The X-Line-Signature header value
///
/// # Returns
///
/// * `Ok(true)` - Signature is valid
/// * `Ok(false)` - Signature is invalid
/// * `Err(anyhow::Error)` - An error occurred
pub fn verify_signature(channel_secret: &str, body: &str, signature: &str) -> Result<bool> {
    let mut mac = Hmac::<Sha256>::new_from_slice(channel_secret.as_bytes())
        .map_err(|e| anyhow::anyhow!("Failed to create HMAC: {}", e))?;
    
    mac.update(body.as_bytes());
    
    let expected = base64::encode(mac.finalize().into_bytes());
    
    Ok(expected == signature)
}

/// Parse webhook request body from raw bytes.
///
/// # Arguments
///
/// * `body` - The raw request body as bytes
///
/// # Returns
///
/// * `Ok(WebhookRequestBody)` - Parsed webhook request
/// * `Err(anyhow::Error)` - An error occurred
pub fn parse_webhook_body(body: &[u8]) -> Result<WebhookRequestBody> {
    let body_str = String::from_utf8(body.to_vec())
        .map_err(|e| anyhow::anyhow!("Failed to parse body as UTF-8: {}", e))?;
    
    serde_json::from_str(&body_str)
        .map_err(|e| anyhow::anyhow!("Failed to parse webhook body: {}", e))
}

/// Extract the destination from a webhook request.
///
/// # Arguments
///
/// * `body` - The raw request body as bytes
///
/// # Returns
///
/// * `Ok(String)` - The destination user or group ID
/// * `Err(anyhow::Error)` - An error occurred
pub fn extract_destination(body: &[u8]) -> Result<String> {
    let webhook: WebhookRequestBody = parse_webhook_body(body)?;
    Ok(webhook.destination)
}

/// Extract the first event from a webhook request.
///
/// # Arguments
///
/// * `body` - The raw request body as bytes
///
/// # Returns
///
/// * `Ok(WebhookEventType)` - The first event
/// * `Err(anyhow::Error)` - An error occurred
pub fn extract_first_event(body: &[u8]) -> Result<WebhookEventType> {
    let webhook: WebhookRequestBody = parse_webhook_body(body)?;
    
    webhook.events.into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("No events in webhook body"))
}

/// Get the reply token from a message event.
///
/// # Arguments
///
/// * `body` - The raw request body as bytes
///
/// # Returns
///
/// * `Ok(String)` - The reply token
/// * `Err(anyhow::Error)` - An error occurred
pub fn extract_reply_token(body: &[u8]) -> Result<String> {
    let webhook: WebhookRequestBody = parse_webhook_body(body)?;
    
    for event in webhook.events {
        if let WebhookEventType::Message(message_event) = event {
            return Ok(message_event.reply_token);
        }
    }
    
    Err(anyhow::anyhow!("No message event found in webhook"))
}

/// Get the source from a webhook event.
///
/// # Arguments
///
/// * `body` - The raw request body as bytes
///
/// # Returns
///
/// * `Ok(EventSource)` - The event source
/// * `Err(anyhow::Error)` - An error occurred
pub fn extract_source(body: &[u8]) -> Result<EventSource> {
    let webhook: WebhookRequestBody = parse_webhook_body(body)?;
    
    if let Some(event) = webhook.events.first() {
        match event {
            WebhookEventType::Message(msg) => Ok(msg.source.clone()),
            WebhookEventType::Follow(follow) => Ok(follow.source.clone()),
            WebhookEventType::Unfollow(unfollow) => Ok(unfollow.source.clone()),
            WebhookEventType::Join(join) => Ok(join.source.clone()),
            WebhookEventType::Leave(leave) => Ok(leave.source.clone()),
            WebhookEventType::Postback(postback) => Ok(postback.source.clone()),
            WebhookEventType::Beacon(beacon) => Ok(beacon.source.clone()),
        }
    } else {
        Err(anyhow::anyhow!("No events in webhook body"))
    }
}

/// Get the user ID from the source.
///
/// # Arguments
///
/// * `source` - The event source
///
/// # Returns
///
/// * `Ok(String)` - The user ID
/// * `Err(anyhow::Error)` - An error occurred
pub fn get_user_id(source: &EventSource) -> Result<String> {
    match source {
        EventSource::User { user_id } => Ok(user_id.clone()),
        EventSource::Group { group_id, user_id } => {
            user_id.clone()
                .ok_or_else(|| anyhow::anyhow!("No user_id in group source"))
        }
        EventSource::Room { room_id, user_id } => {
            user_id.clone()
                .ok_or_else(|| anyhow::anyhow!("No user_id in room source"))
        }
    }
}

/// Get the group ID from the source.
///
/// # Arguments
///
/// * `source` - The event source
///
/// # Returns
///
/// * `Ok(String)` - The group ID
/// * `Err(anyhow::Error)` - An error occurred
pub fn get_group_id(source: &EventSource) -> Result<String> {
    match source {
        EventSource::Group { group_id, .. } => Ok(group_id.clone()),
        _ => Err(anyhow::anyhow!("Source is not a group")),
    }
}

/// Get the room ID from the source.
///
/// # Arguments
///
/// * `source` - The event source
///
/// # Returns
///
/// * `Ok(String)` - The room ID
/// * `Err(anyhow::Error)` - An error occurred
pub fn get_room_id(source: &EventSource) -> Result<String> {
    match source {
        EventSource::Room { room_id, .. } => Ok(room_id.clone()),
        _ => Err(anyhow::anyhow!("Source is not a room")),
    }
}

/// Get the destination from a webhook request.
///
/// # Arguments
///
/// * `webhook` - The parsed webhook request
///
/// # Returns
///
/// * `Ok(String)` - The destination
pub fn get_destination(webhook: &WebhookRequestBody) -> String {
    webhook.destination.clone()
}

/// Check if the webhook contains a message event.
///
/// # Arguments
///
/// * `body` - The raw request body as bytes
///
/// # Returns
///
/// * `true` - Contains a message event
/// * `false` - Does not contain a message event
pub fn contains_message_event(body: &[u8]) -> Result<bool> {
    let webhook: WebhookRequestBody = parse_webhook_body(body)?;
    Ok(webhook.events.iter().any(|e| matches!(e, WebhookEventType::Message(_))))
}

/// Check if the webhook contains a follow event.
///
/// # Arguments
///
/// * `body` - The raw request body as bytes
///
/// # Returns
///
/// * `true` - Contains a follow event
/// * `false` - Does not contain a follow event
pub fn contains_follow_event(body: &[u8]) -> Result<bool> {
    let webhook: WebhookRequestBody = parse_webhook_body(body)?;
    Ok(webhook.events.iter().any(|e| matches!(e, WebhookEventType::Follow(_))))
}

/// Check if the webhook contains a join event.
///
/// # Arguments
///
/// * `body` - The raw request body as bytes
///
/// # Returns
///
/// * `true` - Contains a join event
/// * `false` - Does not contain a join event
pub fn contains_join_event(body: &[u8]) -> Result<bool> {
    let webhook: WebhookRequestBody = parse_webhook_body(body)?;
    Ok(webhook.events.iter().any(|e| matches!(e, WebhookEventType::Join(_))))
}
