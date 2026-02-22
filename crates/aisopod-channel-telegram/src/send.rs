//! Text message sending with Markdown formatting and chunking.
//!
//! This module provides functionality for sending text messages to Telegram
//! with support for Markdown formatting and automatic chunking of long messages.

use crate::{TelegramAccount, TelegramChannel};
use aisopod_channel::message::{MessageContent, MessagePart, MessageTarget, OutgoingMessage};
use aisopod_channel::types::MediaType;
use anyhow::Result;
use std::cmp::Ordering;
use teloxide::prelude::*;

/// Options for sending messages to Telegram.
#[derive(Debug, Clone, Default)]
pub struct SendOptions {
    /// Parse mode for Markdown/HTML formatting
    pub parse_mode: Option<teloxide::types::ParseMode>,
    /// ID of the message to reply to
    pub reply_to_message_id: Option<i64>,
    /// Disable web page previews
    pub disable_web_page_preview: bool,
}

impl From<&TelegramAccount> for SendOptions {
    fn from(account: &TelegramAccount) -> Self {
        SendOptions {
            parse_mode: Some(account.config.parse_mode.clone()),
            ..Default::default()
        }
    }
}

/// Maximum Telegram message length
/// Maximum Telegram message length
pub const MAX_MESSAGE_LENGTH: usize = 4096;

/// Maximum length for a chunk when splitting messages
pub const MAX_CHUNK_LENGTH: usize = 4000; // Leave some buffer for formatting

/// Send a text message to a Telegram chat.
///
/// # Arguments
///
/// * `account` - The Telegram account to send from
/// * `chat_id` - The target chat ID
/// * `text` - The message text to send
/// * `options` - Optional sending options
///
/// # Returns
///
/// The ID of the sent message
pub async fn send_text(
    account: &TelegramAccount,
    chat_id: i64,
    text: &str,
    options: Option<SendOptions>,
) -> Result<i64> {
    use teloxide::types::{ChatId, MessageId};
    
    let options = options.unwrap_or_else(|| SendOptions::from(account));
    
    // Check if message needs chunking
    if text.len() > MAX_MESSAGE_LENGTH {
        let chunks = chunk_message(text, options.parse_mode.as_ref())?;
        let mut last_id: Option<MessageId> = None;
        
        for (i, chunk) in chunks.iter().enumerate() {
            let chunk_options = if i == 0 {
                // First message - can have reply_to
                Some(SendOptions {
                    reply_to_message_id: options.reply_to_message_id,
                    ..options.clone()
                })
            } else {
                // Subsequent messages - inherit parse_mode but no reply_to
                Some(SendOptions {
                    parse_mode: options.parse_mode.clone(),
                    ..Default::default()
                })
            };
            
            // Build the request with conditional reply_to_message_id
            let mut req = account.bot.send_message(ChatId(chat_id), chunk);
            req = req.parse_mode(options.parse_mode.unwrap_or(teloxide::types::ParseMode::MarkdownV2));
            
            // Only set reply_to_message_id if we have a value
            if let Some(id) = chunk_options.as_ref().and_then(|o| o.reply_to_message_id) {
                req = req.reply_to_message_id(MessageId(id as i32));
            }
            
            req = req.disable_web_page_preview(options.disable_web_page_preview);
            
            let sent = req.await?;
            last_id = Some(sent.id);
        }
        
        Ok(last_id.map(|id| id.0 as i64).unwrap_or(0))
    } else {
        // Build the request with conditional reply_to_message_id
        let mut req = account.bot.send_message(ChatId(chat_id), text);
        req = req.parse_mode(options.parse_mode.unwrap_or(teloxide::types::ParseMode::MarkdownV2));
        
        // Only set reply_to_message_id if we have a value
        if let Some(id) = options.reply_to_message_id {
            req = req.reply_to_message_id(MessageId(id as i32));
        }
        
        req = req.disable_web_page_preview(options.disable_web_page_preview);
        
        let sent = req.await?;
        
        Ok(sent.id.0 as i64)
    }
}

/// Split a long message into chunks while preserving Markdown formatting.
///
/// This function handles Markdown delimiters by tracking open/close pairs
/// and attempting to preserve them across chunk boundaries.
fn chunk_message(text: &str, parse_mode: Option<&teloxide::types::ParseMode>) -> Result<Vec<String>> {
    let chunks = match parse_mode.unwrap_or(&teloxide::types::ParseMode::MarkdownV2) {
        teloxide::types::ParseMode::MarkdownV2 => chunk_markdown_v2(text),
        teloxide::types::ParseMode::Html => chunk_html(text),
        teloxide::types::ParseMode::Markdown => chunk_markdown_v2(text), // Markdown is similar to MarkdownV2
    };
    
    Ok(chunks)
}

/// Chunk a message using MarkdownV2 formatting rules.
pub fn chunk_markdown_v2(text: &str) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current_chunk = String::new();
    let chars: Vec<char> = text.chars().collect();
    
    // Track MarkdownV2 special characters that need to be balanced
    let mut open_quotes = 0;      // __, **, ~~
    let mut open_parens = 0;      // []
    let mut open_braces = 0;      // {}
    let mut in_link = false;      // []
    let mut in_code = false;      // `
    let mut in_pre = false;       // ```
    
    let mut i = 0;
    while i < chars.len() {
        let chunk_start = current_chunk.len();
        
        // Process until we hit a chunk boundary
        while i < chars.len() && current_chunk.len() - chunk_start < MAX_CHUNK_LENGTH {
            let c = chars[i];
            
            // Check for code blocks
            if c == '`' && i + 2 < chars.len() && &chars[i..=i+2] == &['`', '`', '`'] {
                in_pre = !in_pre;
                current_chunk.push_str("```");
                i += 3;
                continue;
            }
            
            if c == '`' && !in_pre {
                in_code = !in_code;
            }
            
            // Track open quotes for formatting
            if c == '_' || c == '*' || c == '~' {
                if !in_code && !in_pre {
                    if c == '_' {
                        // Check for __ (bold/italic)
                        if i + 1 < chars.len() && chars[i+1] == '_' {
                            open_quotes += 1;
                            i += 2;
                            continue;
                        }
                    }
                    open_quotes += 1;
                }
            }
            
            current_chunk.push(c);
            i += 1;
        }
        
        // If we stopped exactly at a delimiter, try to balance it
        if current_chunk.len() >= MAX_CHUNK_LENGTH {
            // Try to find a good breaking point
            if let Some(break_point) = find_break_point(&current_chunk) {
                let first_part = current_chunk[..break_point].trim_end().to_string();
                if !first_part.is_empty() {
                    chunks.push(first_part);
                }
                current_chunk = current_chunk[break_point..].to_string();
            } else {
                // No good break point, just cut at the limit
                let first_part = current_chunk[..MAX_CHUNK_LENGTH].to_string();
                chunks.push(first_part);
                current_chunk = current_chunk[MAX_CHUNK_LENGTH..].to_string();
            }
        }
    }
    
    // Push remaining chunk if not empty
    if !current_chunk.is_empty() {
        chunks.push(current_chunk);
    }
    
    chunks
}

/// Find a good break point in a chunk (after whitespace or punctuation).
fn find_break_point(chunk: &str) -> Option<usize> {
    // Try to break at whitespace - find last whitespace from the end
    chunk.char_indices()
        .rfind(|(_, c)| c.is_whitespace())
        .map(|(i, _)| i + 1)
        .or_else(|| {
            // Try to break at punctuation
            chunk.char_indices()
                .rfind(|(_, c)| ".!?,".contains(*c))
                .map(|(i, _)| i + 1)
        })
}

/// Chunk a message using HTML formatting rules.
pub fn chunk_html(text: &str) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current_chunk = String::new();
    
    // Track HTML tags
    let mut in_tag = false;
    let mut tag_content = String::new();
    
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    
    while i < chars.len() {
        let c = chars[i];
        
        if c == '<' {
            in_tag = true;
            tag_content.push(c);
        } else if c == '>' && in_tag {
            in_tag = false;
            tag_content.push(c);
            current_chunk.push_str(&tag_content);
            tag_content.clear();
        } else if c == '&' && i + 1 < chars.len() && chars[i+1] == '#' {
            // HTML entity
            let end = text[i..].find(';').map(|e| i + e + 1).unwrap_or(i + 10);
            let entity = &text[i..end.min(i + 10)];
            current_chunk.push_str(entity);
            i = end - 1;
        } else {
            current_chunk.push(c);
        }
        
        if current_chunk.len() >= MAX_CHUNK_LENGTH {
            // Try to close any open tags
            if in_tag && !tag_content.is_empty() {
                current_chunk.push('>');
                in_tag = false;
            }
            
            chunks.push(current_chunk.clone());
            current_chunk.clear();
        }
        
        i += 1;
    }
    
    if !current_chunk.is_empty() {
        chunks.push(current_chunk);
    }
    
    chunks
}

/// Send an outgoing message using the appropriate account.
///
/// # Arguments
///
/// * `channel` - The Telegram channel
/// * `message` - The outgoing message to send
///
/// # Returns
///
/// The ID of the sent message
pub async fn send_message(
    channel: &TelegramChannel,
    message: &OutgoingMessage,
) -> Result<i64> {
    // Extract chat ID from target
    let chat_id = message.target.peer.id.parse::<i64>()
        .map_err(|e| anyhow::anyhow!("Invalid chat ID: {}: {}", message.target.peer.id, e))?;
    
    // Build the text content - use content_to_string() method on MessageContent
    let text = content_to_string_from_message(message);
    
    // Determine the account ID from the message target
    let account_id = &message.target.account_id;
    
    // Find the account for this message
    let account = channel.get_account(account_id)
        .ok_or_else(|| anyhow::anyhow!("Account not found: {}", account_id))?;
    
    // Create send options
    let options = SendOptions {
        parse_mode: Some(account.config.parse_mode.clone()),
        reply_to_message_id: message.reply_to.as_ref().and_then(|r| r.parse::<i64>().ok()),
        ..Default::default()
    };
    
    send_text(account, chat_id, &text, Some(options)).await
}

/// Helper function to convert message content to string.
fn content_to_string_from_message(message: &OutgoingMessage) -> String {
    match &message.content {
        MessageContent::Text(text) => text.clone(),
        MessageContent::Media(media) => {
            // Return a placeholder for media content
            match &media.media_type {
                MediaType::Image => format!("[Image: {}]", media.url.as_deref().unwrap_or("unknown")),
                MediaType::Audio => format!("[Audio: {}]", media.url.as_deref().unwrap_or("unknown")),
                MediaType::Video => format!("[Video: {}]", media.url.as_deref().unwrap_or("unknown")),
                MediaType::Document => format!("[Document: {}]", media.filename.as_deref().unwrap_or("unknown")),
                MediaType::Other(other) => format!("[{}: {}]", other, media.url.as_deref().unwrap_or("unknown")),
            }
        }
        MessageContent::Mixed(parts) => {
            parts
                .iter()
                .map(|part| match part {
                    MessagePart::Text(text) => text.clone(),
                    MessagePart::Media(media) => {
                        match &media.media_type {
                            MediaType::Image => format!("[Image: {}]", media.url.as_deref().unwrap_or("unknown")),
                            MediaType::Audio => format!("[Audio: {}]", media.url.as_deref().unwrap_or("unknown")),
                            MediaType::Video => format!("[Video: {}]", media.url.as_deref().unwrap_or("unknown")),
                            MediaType::Document => format!("[Document: {}]", media.filename.as_deref().unwrap_or("unknown")),
                            MediaType::Other(other) => format!("[{}: {}]", other, media.url.as_deref().unwrap_or("unknown")),
                        }
                    }
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
    }
}

// ============================================================================
// Extension methods on TelegramChannel
// ============================================================================

// Note: These methods have been moved to TelegramChannel in lib.rs
// to avoid duplicate implementations.
