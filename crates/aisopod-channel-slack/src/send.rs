//! Text and file message sending with mrkdwn formatting.
//!
//! This module provides functionality for sending messages to Slack
//! with support for mrkdwn formatting, file uploads, and message options.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Options for sending a message to Slack.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SendOptions {
    /// Optional thread timestamp to reply in a thread
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_ts: Option<String>,
    /// Whether to broadcast thread reply to the channel (default: false)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_broadcast: Option<bool>,
    /// Whether to use mrkdwn formatting (default: true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mrkdwn: Option<bool>,
    /// Optional icon emoji for the message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_emoji: Option<String>,
    /// Optional username for the message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// Optional message timestamp to edit an existing message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub as_user: Option<bool>,
}

/// A Slack message response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageResponse {
    /// Whether the request was successful
    pub ok: bool,
    /// The channel ID where the message was sent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    /// The timestamp of the sent message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ts: Option<String>,
    /// The message ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
    /// Error information if the request failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Response metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_metadata: Option<ResponseMetadata>,
}

/// Response metadata from the Slack API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetadata {
    /// Warning messages
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<Vec<String>>,
    /// Next cursor for pagination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

impl SendMessageResponse {
    /// Check if the message was sent successfully.
    pub fn is_ok(&self) -> bool {
        self.ok
    }

    /// Get the message timestamp if available.
    pub fn get_ts(&self) -> Option<&str> {
        self.ts.as_deref()
    }

    /// Get the error message if the request failed.
    pub fn get_error(&self) -> Option<&str> {
        self.error.as_deref()
    }
}

/// Convert common markdown to Slack mrkdwn format.
///
/// # Supported conversions:
/// - `**bold**` or `__bold__` → `*bold*`
/// - `*italic*` or `_italic_` → `_italic_`
/// - `~~strikethrough~~` → `~strikethrough~`
/// - `` `inline code` `` → `` `inline code` ``
/// - ```` ```code block``` ```` → ```` ```code block``` ````
/// - `> quote` → `> quote`
/// - `# Header` → `*Header*` (converted to bold header)
/// - `## Subheader` → `*Subheader*`
/// - `1. List` → `• List` (converted to bullet points)
/// - `- List` → `• List`
/// - `* List` → `• List`
///
/// # Arguments
///
/// * `text` - The markdown text to convert
///
/// # Returns
///
/// The converted mrkdwn text
pub fn markdown_to_mrkdwn(text: &str) -> String {
    let mut result = text.to_string();

    // Bold: **text** or __text__ → *text*
    result = regex::Regex::new(r"\*\*(.+?)\*\*")
        .map(|re: regex::Regex| re.replace_all(&result, "*$1*").to_string())
        .unwrap_or(result.clone());
    result = regex::Regex::new(r"__(.+?)__")
        .map(|re: regex::Regex| re.replace_all(&result, "*$1*").to_string())
        .unwrap_or(result.clone());

    // Italic: _text_ → _text_ (single underscore is already valid Slack mrkdwn)
    // Note: **bold** is converted to *bold* first, so we only need to handle _text_
    // Single *text* patterns are already valid Slack mrkdwn (from bold conversion)
    // and should not be modified

    // Strikethrough: ~~text~~ → ~text~
    result = regex::Regex::new(r"~~(.+?)~~")
        .map(|re: regex::Regex| re.replace_all(&result, "~$1~").to_string())
        .unwrap_or(result.clone());

    // Inline code: `code` → `code`
    // Code blocks: ```code``` → ```code```

    // Quotes: > text → > text (already compatible)

    // Headers: # Header → *Header* (multiline)
    result = regex::Regex::new(r"(?m)^#\s+(.+)$")
        .map(|re: regex::Regex| re.replace_all(&result, "*$1*").to_string())
        .unwrap_or(result.clone());

    // Subheaders: ## Subheader → *Subheader* (multiline)
    result = regex::Regex::new(r"(?m)^##\s+(.+)$")
        .map(|re: regex::Regex| re.replace_all(&result, "*$1*").to_string())
        .unwrap_or(result.clone());

    // Lists: 1. item, - item, * item → • item (multiline)
    result = regex::Regex::new(r"(?m)^\s*[\d]+\.\s+(.+)$")
        .map(|re: regex::Regex| re.replace_all(&result, "• $1").to_string())
        .unwrap_or(result.clone());
    result = regex::Regex::new(r"(?m)^\s*-\s+(.+)$")
        .map(|re: regex::Regex| re.replace_all(&result, "• $1").to_string())
        .unwrap_or(result.clone());
    result = regex::Regex::new(r"(?m)^\s*\*\s+(.+)$")
        .map(|re: regex::Regex| re.replace_all(&result, "• $1").to_string())
        .unwrap_or(result.clone());

    // Links: [text](url) → <url|text>
    result = regex::Regex::new(r"\[([^\]]+)\]\(([^)]+)\)")
        .map(|re: regex::Regex| re.replace_all(&result, "<$2|$1>").to_string())
        .unwrap_or(result.clone());

    result
}

/// Split a long message into chunks that fit within Slack's limits.
///
/// Slack has a 4000 character limit for text in chat.postMessage.
/// This function splits messages at natural boundaries (newlines, spaces).
///
/// # Arguments
///
/// * `text` - The text to split
/// * `max_length` - Maximum length per chunk (default: 3900 for safety)
///
/// # Returns
///
/// A vector of message chunks
pub fn split_message(text: &str, max_length: usize) -> Vec<String> {
    let max_len = max_length.min(3900); // Safety margin

    if text.len() <= max_len {
        return vec![text.to_string()];
    }

    let mut chunks = Vec::new();
    let mut remaining = text.to_string();

    while remaining.len() > max_len {
        // Try to find a natural break point (newline, space, or punctuation)
        let chunk = if let Some(newline_pos) = remaining[..max_len].rfind('\n') {
            let after_start = newline_pos + 1;
            let before = remaining[..newline_pos].to_string();
            remaining = remaining[after_start..].to_string();
            before
        } else if let Some(space_pos) = remaining[..max_len].rfind(' ') {
            let after_start = space_pos + 1;
            let before = remaining[..space_pos].to_string();
            remaining = remaining[after_start..].to_string();
            before
        } else {
            // No natural break point, just split at max_len
            let after_start = max_len;
            let chunk = remaining[..max_len].to_string();
            remaining = remaining[after_start..].to_string();
            chunk
        };

        chunks.push(chunk);
    }

    if !remaining.is_empty() {
        chunks.push(remaining);
    }

    chunks
}

/// Build the payload for chat.postMessage.
///
/// # Arguments
///
/// * `channel_id` - The Slack channel ID
/// * `text` - The message text (with mrkdwn formatting)
/// * `options` - Additional send options
/// * `blocks` - Optional Block Kit blocks
///
/// # Returns
///
/// A JSON value representing the message payload
pub fn build_send_message_payload(
    channel_id: &str,
    text: &str,
    options: Option<&SendOptions>,
    blocks: Option<&serde_json::Value>,
) -> serde_json::Value {
    let mut payload = serde_json::json!({
        "channel": channel_id,
        "text": text
    });

    if let Some(opts) = options {
        if let Some(ts) = &opts.thread_ts {
            payload["thread_ts"] = serde_json::json!(ts);
        }
        if let Some(rb) = opts.reply_broadcast {
            payload["reply_broadcast"] = serde_json::json!(rb);
        }
        if let Some(m) = opts.mrkdwn {
            payload["mrkdwn"] = serde_json::json!(m);
        }
        if let Some(emoji) = &opts.icon_emoji {
            payload["icon_emoji"] = serde_json::json!(emoji);
        }
        if let Some(username) = &opts.username {
            payload["username"] = serde_json::json!(username);
        }
        if let Some(as_user) = opts.as_user {
            payload["as_user"] = serde_json::json!(as_user);
        }
    }

    if let Some(b) = blocks {
        payload["blocks"] = b.clone();
    }

    payload
}

/// Send a text message to Slack.
///
/// This is a convenience function that builds the payload and sends it.
///
/// # Arguments
///
/// * `client` - The Slack client handle
/// * `channel_id` - The channel ID to send to
/// * `text` - The message text (supports mrkdwn formatting)
/// * `options` - Optional send options
///
/// # Returns
///
/// * `Ok(SendMessageResponse)` - The response from Slack
/// * `Err(anyhow::Error)` - An error if sending fails
pub async fn send_text_message(
    client: &crate::connection::SlackClientHandle,
    channel_id: &str,
    text: &str,
    options: Option<&SendOptions>,
) -> Result<SendMessageResponse> {
    let payload = build_send_message_payload(channel_id, text, options, None);

    let response = client
        .post("https://slack.com/api/chat.postMessage", &payload)
        .await?;

    let json: serde_json::Value = response.json().await?;
    let parsed: SendMessageResponse = serde_json::from_value(json)?;

    Ok(parsed)
}

/// Send a message with Block Kit support.
///
/// # Arguments
///
/// * `client` - The Slack client handle
/// * `channel_id` - The channel ID to send to
/// * `text` - Fallback text for the message
/// * `blocks` - The Block Kit blocks
/// * `options` - Optional send options
///
/// # Returns
///
/// * `Ok(SendMessageResponse)` - The response from Slack
/// * `Err(anyhow::Error)` - An error if sending fails
pub async fn send_message_with_blocks(
    client: &crate::connection::SlackClientHandle,
    channel_id: &str,
    text: &str,
    blocks: &serde_json::Value,
    options: Option<&SendOptions>,
) -> Result<SendMessageResponse> {
    let payload = build_send_message_payload(channel_id, text, options, Some(blocks));

    let response = client
        .post("https://slack.com/api/chat.postMessage", &payload)
        .await?;

    let json: serde_json::Value = response.json().await?;
    let parsed: SendMessageResponse = serde_json::from_value(json)?;

    Ok(parsed)
}

/// Edit an existing message in Slack.
///
/// # Arguments
///
/// * `client` - The Slack client handle
/// * `channel_id` - The channel ID containing the message
/// * `ts` - The timestamp of the message to edit
/// * `text` - The new text for the message
/// * `blocks` - Optional new Block Kit blocks
///
/// # Returns
///
/// * `Ok(())` - Message was edited successfully
/// * `Err(anyhow::Error)` - An error if editing fails
pub async fn edit_message(
    client: &crate::connection::SlackClientHandle,
    channel_id: &str,
    ts: &str,
    text: &str,
    blocks: Option<&serde_json::Value>,
) -> Result<()> {
    let mut payload = serde_json::json!({
        "channel": channel_id,
        "ts": ts,
        "text": text
    });

    if let Some(b) = blocks {
        payload["blocks"] = b.clone();
    }

    let response = client
        .post("https://slack.com/api/chat.update", &payload)
        .await?;

    let json: serde_json::Value = response.json().await?;
    let success: bool = json["ok"].as_bool().unwrap_or(false);

    if !success {
        let error = json["error"].as_str().unwrap_or("Unknown error");
        return Err(anyhow::anyhow!("chat.update failed: {}", error));
    }

    Ok(())
}

/// Delete a message from Slack.
///
/// # Arguments
///
/// * `client` - The Slack client handle
/// * `channel_id` - The channel ID containing the message
/// * `ts` - The timestamp of the message to delete
///
/// # Returns
///
/// * `Ok(())` - Message was deleted successfully
/// * `Err(anyhow::Error)` - An error if deletion fails
pub async fn delete_message(
    client: &crate::connection::SlackClientHandle,
    channel_id: &str,
    ts: &str,
) -> Result<()> {
    let payload = serde_json::json!({
        "channel": channel_id,
        "ts": ts
    });

    let response = client
        .post("https://slack.com/api/chat.delete", &payload)
        .await?;

    let json: serde_json::Value = response.json().await?;
    let success: bool = json["ok"].as_bool().unwrap_or(false);

    if !success {
        let error = json["error"].as_str().unwrap_or("Unknown error");
        return Err(anyhow::anyhow!("chat.delete failed: {}", error));
    }

    Ok(())
}

/// Send an ephemeral message to a user in a channel.
///
/// Ephemeral messages are visible only to the specified user.
///
/// # Arguments
///
/// * `client` - The Slack client handle
/// * `channel_id` - The channel ID
/// * `user_id` - The user ID to send the message to
/// * `text` - The message text
/// * `options` - Optional send options
///
/// # Returns
///
/// * `Ok(SendMessageResponse)` - The response from Slack
/// * `Err(anyhow::Error)` - An error if sending fails
pub async fn send_ephemeral_message(
    client: &crate::connection::SlackClientHandle,
    channel_id: &str,
    user_id: &str,
    text: &str,
    options: Option<&SendOptions>,
) -> Result<SendMessageResponse> {
    let mut payload = serde_json::json!({
        "channel": channel_id,
        "user": user_id,
        "text": text
    });

    if let Some(opts) = options {
        if let Some(ts) = &opts.thread_ts {
            payload["thread_ts"] = serde_json::json!(ts);
        }
    }

    let response = client
        .post("https://slack.com/api/chat.postEphemeral", &payload)
        .await?;

    let json: serde_json::Value = response.json().await?;
    let parsed: SendMessageResponse = serde_json::from_value(json)?;

    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_to_mrkdwn_basic() {
        let md = "**bold** text";
        let mrkdwn = markdown_to_mrkdwn(md);
        assert!(mrkdwn.contains("*bold*"));
    }

    #[test]
    fn test_markdown_to_mrkdwn_italic() {
        let md = "_italic_ text";
        let mrkdwn = markdown_to_mrkdwn(md);
        assert!(mrkdwn.contains("_italic_"));
    }

    #[test]
    fn test_markdown_to_mrkdwn_strikethrough() {
        let md = "~~strikethrough~~ text";
        let mrkdwn = markdown_to_mrkdwn(md);
        assert!(mrkdwn.contains("~strikethrough~"));
    }

    #[test]
    fn test_markdown_to_mrkdwn_link() {
        let md = "[example](https://example.com)";
        let mrkdwn = markdown_to_mrkdwn(md);
        assert!(mrkdwn.contains("<https://example.com|example>"));
    }

    #[test]
    fn test_markdown_to_mrkdwn_header() {
        let md = "# Header\n## Subheader";
        let mrkdwn = markdown_to_mrkdwn(md);
        assert!(mrkdwn.contains("*Header*"));
        assert!(mrkdwn.contains("*Subheader*"));
    }

    #[test]
    fn test_split_message_short() {
        let text = "Short message";
        let chunks = split_message(text, 100);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], "Short message");
    }

    #[test]
    fn test_split_message_long() {
        let text = "a".repeat(5000);
        let chunks = split_message(&text, 1000);
        assert!(chunks.len() > 1);

        // Verify all chunks are within the limit
        for chunk in &chunks {
            assert!(chunk.len() <= 1000);
        }

        // Verify all characters are preserved
        let combined = chunks.join("");
        assert_eq!(combined.len(), text.len());
    }

    #[test]
    fn test_build_send_message_payload() {
        let payload = build_send_message_payload("C123456", "Hello, world!", None, None);

        assert_eq!(payload["channel"], "C123456");
        assert_eq!(payload["text"], "Hello, world!");
    }

    #[test]
    fn test_send_options_serialization() {
        let options = SendOptions {
            thread_ts: Some("1234567890.123456".to_string()),
            reply_broadcast: Some(true),
            mrkdwn: Some(true),
            ..Default::default()
        };

        let json = serde_json::to_string(&options).unwrap();
        assert!(json.contains("thread_ts"));
        assert!(json.contains("reply_broadcast"));
        assert!(json.contains("true"));
    }

    #[test]
    fn test_send_message_response() {
        let response = SendMessageResponse {
            ok: true,
            channel: Some("C123456".to_string()),
            ts: Some("1234567890.123456".to_string()),
            message_id: None,
            error: None,
            response_metadata: None,
        };

        assert!(response.is_ok());
        assert_eq!(response.get_ts(), Some("1234567890.123456"));
        assert_eq!(response.get_error(), None);
    }
}
