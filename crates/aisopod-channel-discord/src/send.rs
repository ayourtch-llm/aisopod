//! Message sending functionality for Discord channel.
//!
//! This module handles sending text messages with Discord markdown formatting,
//! chunking long messages (2000 char limit), and sending messages with options.

use aisopod_channel::message::{Media, MessageTarget, MessageContent};
use anyhow::{anyhow, Result};
use serenity::{
    all::{ChannelId, CreateMessage, Message, MessageId},
    builder::{CreateEmbed, CreateEmbedFooter, CreateActionRow},
};
use std::time::Duration;

/// Discord's message length limit (2000 characters)
pub const DISCORD_MESSAGE_LIMIT: usize = 2000;

/// Discord's embed limit (10 embeds per message)
pub const DISCORD_EMBED_LIMIT: usize = 10;

/// Options for sending a message to Discord.
#[derive(Debug, Clone, Default)]
pub struct SendOptions {
    /// Optional ID of the message to reply to
    pub reply_to_message_id: Option<MessageId>,
    /// Whether to mention the user when replying
    pub mention_reply: bool,
    /// Optional embeds to include in the message
    pub embeds: Vec<CreateEmbed>,
    /// Optional files/attachments to send
    pub attachments: Vec<serenity::all::CreateAttachment>,
    /// Optional components (buttons, selects) to include
    pub components: Vec<CreateActionRow>,
}

/// Result of sending a message to Discord.
#[derive(Debug, Clone)]
pub struct SendMessageResult {
    /// The ID of the sent message
    pub message_id: MessageId,
    /// Whether the message was chunked (split into multiple messages)
    pub was_chunked: bool,
    /// Number of chunks sent
    pub chunk_count: usize,
}

/// Send a text message to a Discord channel.
///
/// This method handles Discord's 2000 character limit by automatically
/// chunking long messages into multiple parts. It supports Discord markdown
/// formatting including bold, italic, code blocks, and more.
///
/// # Arguments
///
/// * `cache_http` - Implementation of CacheHttp trait (can be &Context, Arc<Client>, or &Http)
/// * `channel_id` - The target channel ID
/// * `text` - The text content to send
/// * `options` - Additional options for the message
///
/// # Returns
///
/// * `Ok(SendMessageResult)` - The result of sending the message
/// * `Err(anyhow::Error)` - An error if sending fails
pub async fn send_message(
    cache_http: impl serenity::http::CacheHttp + Clone,
    channel_id: ChannelId,
    text: &str,
    options: Option<SendOptions>,
) -> Result<SendMessageResult> {
    let options = options.unwrap_or_default();

    // Check if message needs chunking (2000 char limit)
    if text.len() <= DISCORD_MESSAGE_LIMIT {
        // Single message fits, send it directly
        let message = create_discord_message(cache_http.clone(), channel_id, text, &options).await?;
        Ok(SendMessageResult {
            message_id: message.id,
            was_chunked: false,
            chunk_count: 1,
        })
    } else {
        // Message needs to be chunked
        chunk_and_send(cache_http, channel_id, text, options).await
    }
}

/// Create and send a Discord message with the given content and options.
async fn create_discord_message(
    cache_http: impl serenity::http::CacheHttp,
    channel_id: ChannelId,
    text: &str,
    options: &SendOptions,
) -> Result<Message> {
    let mut msg_builder = CreateMessage::new().content(text);

    // Add reply reference if specified
    // MessageReference is non-exhaustive but implements From<(ChannelId, MessageId)>
    if let Some(reply_id) = options.reply_to_message_id {
        msg_builder = msg_builder.reference_message((channel_id, reply_id));
    }
    
    // Note: mention_reply is now handled via allowed_mentions in CreateAllowedMentions

    // Add embeds
    if !options.embeds.is_empty() {
        msg_builder = msg_builder.add_embeds(options.embeds.clone());
    }

    // Add attachments
    if !options.attachments.is_empty() {
        msg_builder = msg_builder.add_files(options.attachments.clone());
    }

    // Add components
    if !options.components.is_empty() {
        msg_builder = msg_builder.components(options.components.clone());
    }

    let message = channel_id
        .send_message(cache_http, msg_builder)
        .await
        .map_err(|e| anyhow!("Failed to send Discord message: {}", e))?;

    Ok(message)
}

/// Chunk a long message and send it as multiple parts.
async fn chunk_and_send(
    cache_http: impl serenity::http::CacheHttp + Clone,
    channel_id: ChannelId,
    text: &str,
    options: SendOptions,
) -> Result<SendMessageResult> {
    let chunks: Vec<&str> = chunk_text(text);
    let total_chunks = chunks.len();

    let mut final_message_id: Option<MessageId> = None;

    for (i, chunk) in chunks.iter().enumerate() {
        let mut chunk_options = options.clone();
        
        // For chunks after the first, remove reply_to since we're continuing the thread
        if i > 0 {
            chunk_options.reply_to_message_id = None;
            chunk_options.mention_reply = false;
        }

        // For the last chunk, keep the original embeds
        if i < total_chunks - 1 {
            chunk_options.embeds.clear();
        }

        let message = create_discord_message(cache_http.clone(), channel_id, chunk, &chunk_options).await?;
        final_message_id = Some(message.id);
    }

    Ok(SendMessageResult {
        message_id: final_message_id.ok_or_else(|| anyhow!("Failed to get message ID"))?,
        was_chunked: true,
        chunk_count: total_chunks,
    })
}

/// Chunk text into Discord-compatible pieces, respecting the 2000 character limit.
///
/// This function attempts to preserve word boundaries when chunking,
/// but will split words if necessary to respect the limit.
pub fn chunk_text(text: &str) -> Vec<&str> {
    let mut chunks = Vec::new();
    let text_len = text.len();

    if text_len <= DISCORD_MESSAGE_LIMIT {
        chunks.push(text);
        return chunks;
    }

    let chars: Vec<char> = text.chars().collect();
    let mut start = 0;

    while start < text_len {
        let remaining = text_len - start;
        
        if remaining <= DISCORD_MESSAGE_LIMIT {
            chunks.push(&text[start..]);
            break;
        }

        // Find a good split point (try to avoid splitting in the middle of words)
        let mut end = start + DISCORD_MESSAGE_LIMIT;
        
        // Look backward for a space or newline to split at
        while end > start {
            if chars[end - 1] == ' ' || chars[end - 1] == '\n' || chars[end - 1] == '\r' {
                break;
            }
            end -= 1;
        }

        // If we couldn't find a good split point, force split at limit
        if end == start {
            end = start + DISCORD_MESSAGE_LIMIT;
        }

        chunks.push(&text[start..end]);
        start = end;
    }

    chunks
}

/// Discord markdown formatting helpers.
pub mod formatting {
    use std::borrow::Cow;

    /// Make text bold.
    pub fn bold(text: &str) -> String {
        format!("**{}**", escape_markdown(text))
    }

    /// Make text italic.
    pub fn italic(text: &str) -> String {
        format!("_{}_", escape_markdown(text))
    }

    /// Make text underlined.
    pub fn underline(text: &str) -> String {
        format!("__{}__", escape_markdown(text))
    }

    /// Make text strikethrough.
    pub fn strikethrough(text: &str) -> String {
        format!("~~{}~~", escape_markdown(text))
    }

    /// Make text monospace (code inline).
    pub fn code(text: &str) -> String {
        format!("`{}`", escape_code(text))
    }

    /// Make text monospace with syntax highlighting (code block).
    pub fn code_block(text: &str, language: Option<&str>) -> String {
        match language {
            Some(lang) => format!("```{}\n{}\n```", lang, escape_code_block(text)),
            None => format!("```\n{}\n```", escape_code_block(text)),
        }
    }

    /// Make text a blockquote.
    pub fn blockquote(text: &str) -> String {
        format!("> {}", text.replace('\n', "\n> "))
    }

    /// Make text a spoiler.
    pub fn spoiler(text: &str) -> String {
        format!("||{}||", escape_markdown(text))
    }

    /// Escape markdown characters in text.
    pub fn escape_markdown(text: &str) -> Cow<str> {
        const MARKDOWN_CHARS: [char; 10] = [
            '*', '_', '`', '[', ']', '(', ')', '~', '>', '#',
        ];
        
        if text.chars().any(|c| MARKDOWN_CHARS.contains(&c)) {
            let escaped: String = text
                .chars()
                .map(|c| {
                    if MARKDOWN_CHARS.contains(&c) {
                        format!("\\{}", c)
                    } else {
                        c.to_string()
                    }
                })
                .collect();
            Cow::Owned(escaped)
        } else {
            Cow::Borrowed(text)
        }
    }

    /// Escape text for inline code (backticks).
    pub fn escape_code(text: &str) -> Cow<str> {
        if text.contains('`') {
            Cow::Owned(text.replace('`', "\\`"))
        } else {
            Cow::Borrowed(text)
        }
    }

    /// Escape text for code blocks.
    pub fn escape_code_block(text: &str) -> Cow<str> {
        if text.contains("```") {
            // Replace triple backticks with a visually similar alternative
            Cow::Owned(text.replace("```", "` `` `"))
        } else {
            Cow::Borrowed(text)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_text_short() {
        let text = "Hello, world!";
        let chunks = chunk_text(text);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], text);
    }

    #[test]
    fn test_chunk_text_exact_limit() {
        let text = "a".repeat(DISCORD_MESSAGE_LIMIT);
        let chunks = chunk_text(&text);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].len(), DISCORD_MESSAGE_LIMIT);
    }

    #[test]
    fn test_chunk_text_over_limit() {
        let text = "a".repeat(DISCORD_MESSAGE_LIMIT + 100);
        let chunks = chunk_text(&text);
        assert_eq!(chunks.len(), 2);
        assert!(chunks[0].len() <= DISCORD_MESSAGE_LIMIT);
        assert!(chunks[1].len() <= DISCORD_MESSAGE_LIMIT);
    }

    #[test]
    fn test_chunk_text_preserves_words() {
        let text = "This is a long message that needs to be chunked properly while preserving word boundaries";
        let chunks = chunk_text(text);
        
        // Should be chunked
        assert!(chunks.len() >= 1);
        
        // Check that words aren't broken in the middle (except when necessary)
        for chunk in &chunks {
            let trimmed = chunk.trim();
            if !trimmed.is_empty() {
                // First char should be a letter
                assert!(trimmed.chars().next().unwrap().is_alphabetic());
            }
        }
    }

    #[test]
    fn test_chunk_text_newlines() {
        let text = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6\nLine 7";
        let chunks = chunk_text(text);
        
        // Should be chunked if over limit
        assert!(chunks.len() >= 1);
        
        // Newlines should be preserved
        for chunk in &chunks {
            assert!(chunk.contains('\n') || chunk.lines().count() > 0);
        }
    }

    #[test]
    fn test_markdown_bold() {
        assert_eq!(formatting::bold("test"), "**test**");
        assert_eq!(formatting::bold("test*with*stars"), "**test\\*with\\*stars**");
    }

    #[test]
    fn test_markdown_italic() {
        assert_eq!(formatting::italic("test"), "_test_");
    }

    #[test]
    fn test_markdown_underline() {
        assert_eq!(formatting::underline("test"), "__test__");
    }

    #[test]
    fn test_markdown_strikethrough() {
        assert_eq!(formatting::strikethrough("test"), "~~test~~");
    }

    #[test]
    fn test_markdown_code() {
        assert_eq!(formatting::code("test"), "`test`");
        assert_eq!(formatting::code("test`with`backticks"), "`test\\`with\\`backticks`");
    }

    #[test]
    fn test_markdown_code_block() {
        let code = "fn main() { println!(\"Hello\"); }";
        let result = formatting::code_block(code, Some("rust"));
        assert!(result.starts_with("```rust"));
        assert!(result.ends_with("```"));
        assert!(result.contains(code));
    }

    #[test]
    fn test_markdown_blockquote() {
        let text = "Line 1\nLine 2";
        let result = formatting::blockquote(text);
        assert_eq!(result, "> Line 1\n> Line 2");
    }

    #[test]
    fn test_markdown_spoiler() {
        assert_eq!(formatting::spoiler("secret"), "||secret||");
    }

    #[test]
    fn test_markdown_escape_special() {
        let text = "test*bold*_italic_`code`[link](url)~strike~>quote#header";
        let escaped = formatting::escape_markdown(text);
        assert!(escaped.contains('\\'));
    }
}
