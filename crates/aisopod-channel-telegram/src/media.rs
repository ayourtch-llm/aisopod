//! Media sending and receiving for Telegram channel.
//!
//! This module provides functionality for sending and receiving media files
//! including photos, documents, audio, and video.

use crate::{send_text, TelegramAccount, TelegramChannel};
use aisopod_channel::message::{Media, MessageContent, MessageTarget, OutgoingMessage};
use aisopod_channel::types::MediaType;
use anyhow::Result;
use teloxide::prelude::*;

/// Send a photo to a Telegram chat.
///
/// # Arguments
///
/// * `account` - The Telegram account to send from
/// * `chat_id` - The target chat ID
/// * `photo` - The photo data (bytes) or URL
/// * `caption` - Optional caption for the photo
/// * `options` - Optional sending options
///
/// # Returns
///
/// The ID of the sent message
pub async fn send_photo(
    account: &TelegramAccount,
    chat_id: i64,
    photo: &[u8],
    caption: Option<&str>,
    options: Option<crate::send::SendOptions>,
) -> Result<i64> {
    use bytes::Bytes;
    use teloxide::types::InputFile;
    
    let options = options.unwrap_or_else(|| crate::send::SendOptions::from(account));
    
    let msg = account.bot
        .send_photo(ChatId(chat_id), InputFile::memory(Bytes::from(photo.to_vec())))
        .parse_mode(options.parse_mode.unwrap_or(teloxide::types::ParseMode::MarkdownV2))
        .caption(caption.unwrap_or(""))
        .await?;
    
    Ok(msg.id.0 as i64)
}

/// Send a document to a Telegram chat.
///
/// # Arguments
///
/// * `account` - The Telegram account to send from
/// * `chat_id` - The target chat ID
/// * `document` - The document data (bytes) or URL
/// * `filename` - Optional filename for the document
/// * `caption` - Optional caption for the document
/// * `options` - Optional sending options
///
/// # Returns
///
/// The ID of the sent message
pub async fn send_document(
    account: &TelegramAccount,
    chat_id: i64,
    document: &[u8],
    filename: Option<&str>,
    caption: Option<&str>,
    options: Option<crate::send::SendOptions>,
) -> Result<i64> {
    use bytes::Bytes;
    use teloxide::types::InputFile;
    
    let options = options.unwrap_or_else(|| crate::send::SendOptions::from(account));
    
    let file = InputFile::memory(Bytes::from(document.to_vec()));
    
    let msg = account.bot
        .send_document(ChatId(chat_id), file)
        .parse_mode(options.parse_mode.unwrap_or(teloxide::types::ParseMode::MarkdownV2))
        .caption(caption.unwrap_or(""))
        .await?;
    
    Ok(msg.id.0 as i64)
}

/// Send audio to a Telegram chat.
///
/// # Arguments
///
/// * `account` - The Telegram account to send from
/// * `chat_id` - The target chat ID
/// * `audio` - The audio data (bytes) or URL
/// * `filename` - Optional filename for the audio
/// * `caption` - Optional caption for the audio
/// * `options` - Optional sending options
///
/// # Returns
///
/// The ID of the sent message
pub async fn send_audio(
    account: &TelegramAccount,
    chat_id: i64,
    audio: &[u8],
    filename: Option<&str>,
    caption: Option<&str>,
    options: Option<crate::send::SendOptions>,
) -> Result<i64> {
    use bytes::Bytes;
    use teloxide::types::InputFile;
    
    let options = options.unwrap_or_else(|| crate::send::SendOptions::from(account));
    
    let file = InputFile::memory(Bytes::from(audio.to_vec()));
    
    let msg = account.bot
        .send_audio(ChatId(chat_id), file)
        .parse_mode(options.parse_mode.unwrap_or(teloxide::types::ParseMode::MarkdownV2))
        .caption(caption.unwrap_or(""))
        .await?;
    
    Ok(msg.id.0 as i64)
}

/// Send a video to a Telegram chat.
///
/// # Arguments
///
/// * `account` - The Telegram account to send from
/// * `chat_id` - The target chat ID
/// * `video` - The video data (bytes) or URL
/// * `filename` - Optional filename for the video
/// * `caption` - Optional caption for the video
/// * `options` - Optional sending options
///
/// # Returns
///
/// The ID of the sent message
pub async fn send_video(
    account: &TelegramAccount,
    chat_id: i64,
    video: &[u8],
    filename: Option<&str>,
    caption: Option<&str>,
    options: Option<crate::send::SendOptions>,
) -> Result<i64> {
    use bytes::Bytes;
    use teloxide::types::InputFile;
    
    let options = options.unwrap_or_else(|| crate::send::SendOptions::from(account));
    
    let file = InputFile::memory(Bytes::from(video.to_vec()));
    
    let msg = account.bot
        .send_video(ChatId(chat_id), file)
        .parse_mode(options.parse_mode.unwrap_or(teloxide::types::ParseMode::MarkdownV2))
        .caption(caption.unwrap_or(""))
        .await?;
    
    Ok(msg.id.0 as i64)
}

/// Map a MediaType to the corresponding Telegram file type handler.
pub fn map_media_type_to_handler(media_type: &MediaType) -> &'static str {
    match media_type {
        MediaType::Image => "send_photo",
        MediaType::Audio => "send_audio",
        MediaType::Video => "send_video",
        MediaType::Document => "send_document",
        MediaType::Other(_) => "send_document", // Default to document for unknown types
    }
}

/// Send media content to a Telegram chat.
///
/// # Arguments
///
/// * `account` - The Telegram account to send from
/// * `chat_id` - The target chat ID
/// * `media` - The media content to send
/// * `caption` - Optional caption for the media
/// * `options` - Optional sending options
///
/// # Returns
///
/// The ID of the sent message
pub async fn send_media(
    account: &TelegramAccount,
    chat_id: i64,
    media: &Media,
    caption: Option<&str>,
    options: Option<crate::send::SendOptions>,
) -> Result<i64> {
    let caption = caption.unwrap_or("");
    let options = options.unwrap_or_else(|| crate::send::SendOptions::from(account));
    
    match media.media_type {
        MediaType::Image => {
            if let Some(data) = &media.data {
                send_photo(account, chat_id, data, Some(caption), Some(options)).await
            } else if let Some(url) = &media.url {
                // For URLs, we need to download first (simplified for now)
                // In production, this would fetch from the URL
                Err(anyhow::anyhow!("URL-based media sending not yet implemented"))
            } else {
                Err(anyhow::anyhow!("Media data or URL required"))
            }
        }
        MediaType::Audio => {
            if let Some(data) = &media.data {
                send_audio(account, chat_id, data, media.filename.as_deref(), Some(caption), Some(options)).await
            } else if let Some(url) = &media.url {
                Err(anyhow::anyhow!("URL-based media sending not yet implemented"))
            } else {
                Err(anyhow::anyhow!("Media data or URL required"))
            }
        }
        MediaType::Video => {
            if let Some(data) = &media.data {
                send_video(account, chat_id, data, media.filename.as_deref(), Some(caption), Some(options)).await
            } else if let Some(url) = &media.url {
                Err(anyhow::anyhow!("URL-based media sending not yet implemented"))
            } else {
                Err(anyhow::anyhow!("Media data or URL required"))
            }
        }
        MediaType::Document => {
            if let Some(data) = &media.data {
                send_document(account, chat_id, data, media.filename.as_deref(), Some(caption), Some(options)).await
            } else if let Some(url) = &media.url {
                Err(anyhow::anyhow!("URL-based media sending not yet implemented"))
            } else {
                Err(anyhow::anyhow!("Media data or URL required"))
            }
        }
        MediaType::Other(_) => {
            // For unknown media types, try as document
            if let Some(data) = &media.data {
                send_document(account, chat_id, data, media.filename.as_deref(), Some(caption), Some(options)).await
            } else {
                Err(anyhow::anyhow!("Unknown media type requires data"))
            }
        }
    }
}

/// Extract media from an incoming Telegram message.
///
/// This function checks a Telegram message for any media content and
/// extracts it into the shared `Media` type.
pub fn extract_media_from_message(msg: &teloxide::types::Message) -> Option<Media> {
    // Check for photo (use last size for best quality)
    if let Some(photo) = &msg.photo() {
        if let Some(last_size) = photo.last() {
            return Some(Media {
                media_type: MediaType::Image,
                url: Some(last_size.file.id.clone()),
                data: None,
                filename: None,
                mime_type: Some("image/jpeg".to_string()), // Default for photos
                size_bytes: Some(last_size.file.size as u64),
            });
        }
    }
    
    // Check for audio
    if let Some(audio) = &msg.audio() {
        return Some(Media {
            media_type: MediaType::Audio,
            url: Some(audio.file.id.clone()),
            data: None,
            filename: audio.file_name.clone(),
            mime_type: audio.mime_type.as_ref().map(|m| m.essence_str().to_string()),
            size_bytes: Some(audio.file.size as u64),
        });
    }
    
    // Check for video
    if let Some(video) = &msg.video() {
        return Some(Media {
            media_type: MediaType::Video,
            url: Some(video.file.id.clone()),
            data: None,
            filename: video.file_name.clone(),
            mime_type: video.mime_type.as_ref().map(|m| m.essence_str().to_string()),
            size_bytes: Some(video.file.size as u64),
        });
    }
    
    // Check for document
    if let Some(document) = &msg.document() {
        return Some(Media {
            media_type: MediaType::Document,
            url: Some(document.file.id.clone()),
            data: None,
            filename: document.file_name.clone(),
            mime_type: document.mime_type.as_ref().map(|m| m.essence_str().to_string()),
            size_bytes: Some(document.file.size as u64),
        });
    }
    
    // Check for sticker (treat as image)
    if let Some(sticker) = &msg.sticker() {
        // Determine MIME type from sticker format
        let mime_type = match sticker.format {
            teloxide::types::StickerFormat::Raster => Some("image/webp".to_string()),
            teloxide::types::StickerFormat::Animated => Some("application/x-tgsticker".to_string()),
            teloxide::types::StickerFormat::Video => Some("video/webm".to_string()),
        };
        
        return Some(Media {
            media_type: MediaType::Image,
            url: Some(sticker.file.id.clone()),
            data: None,
            filename: sticker.emoji.clone(),
            mime_type,
            size_bytes: Some(sticker.file.size as u64),
        });
    }
    
    // No media found
    None
}

/// Convert an outgoing message to media-specific send calls.
///
/// # Arguments
///
/// * `channel` - The Telegram channel
/// * `message` - The outgoing message to send
///
/// # Returns
///
/// The ID of the sent message
pub async fn send_message_with_media(
    channel: &TelegramChannel,
    message: &OutgoingMessage,
) -> Result<i64> {
    // Extract chat ID from target
    let chat_id = message.target.peer.id.parse::<i64>()
        .map_err(|e| anyhow::anyhow!("Invalid chat ID: {}: {}", message.target.peer.id, e))?;
    
    // Determine the account ID from the message target
    let account_id = &message.target.account_id;
    
    // Find the account for this message
    let account = channel.get_account(account_id)
        .ok_or_else(|| anyhow::anyhow!("Account not found: {}", account_id))?;
    
    // Get the content
    let content = match &message.content {
        MessageContent::Text(text) => return send_text(account, chat_id, text, None).await,
        MessageContent::Media(media) => {
            return send_media(account, chat_id, media, None, None).await;
        }
        MessageContent::Mixed(_) => {
            // For mixed content, send each part separately
            // This is a simplified approach - in production, you might want to batch them
            match &message.content {
                MessageContent::Mixed(parts) => {
                    let mut last_id = None;
                    for part in parts {
                        match part {
                            aisopod_channel::message::MessagePart::Text(text) => {
                                last_id = Some(send_text(account, chat_id, text, None).await?);
                            }
                            aisopod_channel::message::MessagePart::Media(media) => {
                                last_id = Some(send_media(account, chat_id, media, None, None).await?);
                            }
                        }
                    }
                    return Ok(last_id.unwrap_or(0));
                }
                _ => unreachable!(),
            }
        }
    };
    
    Ok(content)
}
