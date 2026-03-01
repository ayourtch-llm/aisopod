//! Media attachment handling for Discord channel.
//!
//! This module handles sending and receiving media attachments (files, images, etc.)
//! through Discord messages. It also provides utilities for mapping between
//! Discord attachment types and the shared MediaType enum.

use aisopod_channel::message::Media;
use aisopod_channel::types::MediaType;
use anyhow::{anyhow, Result};
use serenity::{
    all::{Attachment, ChannelId, CreateAttachment, Message},
    client::Context,
    http::Http,
};
use std::path::Path;
use tracing::{debug, info};

/// Maximum file size for Discord uploads (typically 8MB for verified bots, 50MB for Nitro)
pub const MAX_FILE_SIZE: u64 = 25 * 1024 * 1024; // 25MB for safety

/// Discord attachment content type mappings to MediaType.
fn map_discord_content_type(content_type: &str) -> MediaType {
    match content_type {
        ct if ct.starts_with("image/") => MediaType::Image,
        ct if ct.starts_with("audio/") => MediaType::Audio,
        ct if ct.starts_with("video/") => MediaType::Video,
        ct if ct.starts_with("text/") => MediaType::Document,
        // Only specific known application types are mapped to Document
        ct if ct == "application/pdf" => MediaType::Document,
        ct if ct == "application/json" => MediaType::Document,
        ct if ct.starts_with("application/json") => MediaType::Document,
        // Unknown application types become Other
        ct if ct.starts_with("application/") => MediaType::Other(content_type.to_string()),
        _ => MediaType::Other(content_type.to_string()),
    }
}

/// Extract attachments from a Discord message and convert to Media.
///
/// # Arguments
///
/// * `attachments` - The Discord attachment objects from a message
/// * `base_url` - Optional base URL for constructing attachment URLs
///
/// # Returns
///
/// A vector of Media objects extracted from the attachments.
pub fn extract_media_from_attachments(
    attachments: &[Attachment],
    base_url: Option<&str>,
) -> Vec<Media> {
    attachments
        .iter()
        .map(|att| {
            let media_type = map_discord_content_type(
                att.content_type
                    .as_deref()
                    .unwrap_or("application/octet-stream"),
            );

            // Construct the URL - Discord attachments use a specific URL pattern
            let url = base_url
                .map(|base| format!("{}{}", base, att.url))
                .or_else(|| Some(format!("https://cdn.discordapp.com{}", att.url)))
                .unwrap_or_else(|| att.url.clone());

            Media {
                media_type,
                url: Some(url),
                data: None,
                filename: Some(att.filename.clone()),
                mime_type: att.content_type.clone(),
                size_bytes: Some(att.size as u64),
            }
        })
        .collect()
}

/// Create a CreateAttachment from a file path.
///
/// # Arguments
///
/// * `file_path` - The path to the file to attach
/// * `filename` - Optional custom filename for the attachment
///
/// # Returns
///
/// * `Ok(CreateAttachment)` - The attachment ready to send
/// * `Err(anyhow::Error)` - An error if reading the file fails
pub async fn create_attachment_from_path(
    file_path: &str,
    filename: Option<&str>,
) -> Result<CreateAttachment> {
    use tokio::fs;

    let path = Path::new(file_path);

    // Read the file content
    let data = fs::read(path)
        .await
        .map_err(|e| anyhow!("Failed to read file '{}': {}", file_path, e))?;

    // Determine the filename
    let fname = filename
        .map(|s| s.to_string())
        .or_else(|| path.file_name().map(|f| f.to_string_lossy().to_string()))
        .ok_or_else(|| anyhow!("Could not determine filename for file '{}'", file_path))?;

    // Create the attachment
    let attachment = CreateAttachment::bytes(data, fname);

    Ok(attachment)
}

/// Create a CreateAttachment from in-memory data.
///
/// # Arguments
///
/// * `data` - The file data
/// * `filename` - The filename for the attachment
/// * `content_type` - Optional MIME type
pub fn create_attachment_from_bytes(
    data: Vec<u8>,
    filename: &str,
    content_type: Option<&str>,
) -> Result<CreateAttachment> {
    let mut attachment = CreateAttachment::bytes(data, filename);

    if let Some(ct) = content_type {
        // Note: serenity 0.12 doesn't expose content_type setter directly
        // The content type is typically inferred from the filename extension
    }

    Ok(attachment)
}

/// Download media from a Discord attachment URL.
///
/// # Arguments
///
/// * `http` - The serenity HTTP client
/// * `url` - The attachment URL
///
/// # Returns
///
/// * `Ok((Vec<u8>, String))` - The file data and content type
/// * `Err(anyhow::Error)` - An error if downloading fails
pub async fn download_attachment(_http: &Http, url: &str) -> Result<(Vec<u8>, String)> {
    // In serenity v0.12, direct HTTP access to the internal client is not possible
    // Use reqwest directly for attachment downloads
    let client = reqwest::Client::new();

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| anyhow!("Failed to download attachment from '{}': {}", url, e))?;

    // Get content type from response headers BEFORE consuming the response
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "application/octet-stream".to_string());

    let bytes = response
        .bytes()
        .await
        .map_err(|e| anyhow!("Failed to read attachment bytes: {}", e))?;

    Ok((bytes.to_vec(), content_type))
}

/// Download and convert a Discord message's media to Media objects.
///
/// # Arguments
///
/// * `ctx` - The serenity context
/// * `message` - The Discord message containing attachments
///
/// # Returns
///
/// A vector of Media objects with downloaded data.
pub async fn download_attachments(ctx: &Context, message: &Message) -> Result<Vec<Media>> {
    let mut result = Vec::new();

    for attachment in &message.attachments {
        let (data, content_type) = download_attachment(&ctx.http, &attachment.url).await?;

        result.push(Media {
            media_type: map_discord_content_type(content_type.as_str()),
            url: Some(attachment.url.clone()),
            data: Some(data),
            filename: Some(attachment.filename.clone()),
            mime_type: Some(content_type),
            size_bytes: Some(attachment.size as u64),
        });
    }

    Ok(result)
}

/// Send media to a Discord channel.
///
/// # Arguments
///
/// * `ctx` - The serenity context
/// * `channel_id` - The target channel ID
/// * `media` - The media to send
/// * `content` - Optional message content to include with the media
///
/// # Returns
///
/// * `Ok(Message)` - The sent message
/// * `Err(anyhow::Error)` - An error if sending fails
pub async fn send_media(
    ctx: &Context,
    channel_id: ChannelId,
    media: &Media,
    content: Option<&str>,
) -> Result<serenity::all::Message> {
    // Create attachment from media data
    let attachment = if let Some(data) = &media.data {
        create_attachment_from_bytes(
            data.clone(),
            media.filename.as_deref().unwrap_or("attachment"),
            media.mime_type.as_deref(),
        )?
    } else if let Some(url) = &media.url {
        // Download from URL
        let (data, _) = download_attachment(&ctx.http, url).await?;
        create_attachment_from_bytes(
            data,
            media.filename.as_deref().unwrap_or("attachment"),
            media.mime_type.as_deref(),
        )?
    } else {
        return Err(anyhow!("Media must have either data or URL"));
    };

    // Build the message
    let mut msg_builder = serenity::all::CreateMessage::new();

    if let Some(c) = content {
        msg_builder = msg_builder.content(c);
    }

    msg_builder = msg_builder.add_file(attachment);

    let message = channel_id
        .send_message(&ctx.http, msg_builder)
        .await
        .map_err(|e| anyhow!("Failed to send media: {}", e))?;

    Ok(message)
}

/// Send multiple media attachments to a Discord channel.
///
/// # Arguments
///
/// * `ctx` - The serenity context
/// * `channel_id` - The target channel ID
/// * `media_list` - The list of media to send
/// * `content` - Optional message content
///
/// # Returns
///
/// * `Ok(Message)` - The sent message (first in batch)
/// * `Err(anyhow::Error)` - An error if sending fails
pub async fn send_media_batch(
    ctx: &Context,
    channel_id: ChannelId,
    media_list: &[Media],
    content: Option<&str>,
) -> Result<serenity::all::Message> {
    if media_list.is_empty() {
        return Err(anyhow!("No media to send"));
    }

    let mut msg_builder = serenity::all::CreateMessage::new();

    if let Some(c) = content {
        msg_builder = msg_builder.content(c);
    }

    for media in media_list {
        let attachment = if let Some(data) = &media.data {
            create_attachment_from_bytes(
                data.clone(),
                media.filename.as_deref().unwrap_or("attachment"),
                media.mime_type.as_deref(),
            )?
        } else if let Some(url) = &media.url {
            let (data, _) = download_attachment(&ctx.http, url).await?;
            create_attachment_from_bytes(
                data,
                media.filename.as_deref().unwrap_or("attachment"),
                media.mime_type.as_deref(),
            )?
        } else {
            continue;
        };

        msg_builder = msg_builder.add_file(attachment);
    }

    let message = channel_id
        .send_message(&ctx.http, msg_builder)
        .await
        .map_err(|e| anyhow!("Failed to send media batch: {}", e))?;

    Ok(message)
}

/// Validate media before sending.
///
/// Checks file size and other constraints.
///
/// # Arguments
///
/// * `media` - The media to validate
///
/// # Returns
///
/// * `Ok(())` - Media is valid
/// * `Err(anyhow::Error)` - Media validation failed
pub fn validate_media(media: &Media) -> Result<()> {
    // Check file size if data is present
    if let Some(data) = &media.data {
        if data.len() as u64 > MAX_FILE_SIZE {
            return Err(anyhow!(
                "Media file size ({}) exceeds maximum ({})",
                data.len(),
                MAX_FILE_SIZE
            ));
        }
    }

    // Check filename if present
    if let Some(filename) = &media.filename {
        if filename.is_empty() {
            return Err(anyhow!("Filename cannot be empty"));
        }

        // Check for potentially malicious characters
        if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
            return Err(anyhow!("Filename contains invalid characters"));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_discord_content_type_image() {
        assert_eq!(map_discord_content_type("image/png"), MediaType::Image);
        assert_eq!(map_discord_content_type("image/jpeg"), MediaType::Image);
    }

    #[test]
    fn test_map_discord_content_type_audio() {
        assert_eq!(map_discord_content_type("audio/mpeg"), MediaType::Audio);
        assert_eq!(map_discord_content_type("audio/wav"), MediaType::Audio);
    }

    #[test]
    fn test_map_discord_content_type_video() {
        assert_eq!(map_discord_content_type("video/mp4"), MediaType::Video);
        assert_eq!(map_discord_content_type("video/webm"), MediaType::Video);
    }

    #[test]
    fn test_map_discord_content_type_document() {
        assert_eq!(
            map_discord_content_type("application/pdf"),
            MediaType::Document
        );
        assert_eq!(map_discord_content_type("text/plain"), MediaType::Document);
        assert_eq!(
            map_discord_content_type("application/json"),
            MediaType::Document
        );
    }

    #[test]
    fn test_map_discord_content_type_other() {
        let result = map_discord_content_type("application/x-custom");
        assert_eq!(result, MediaType::Other("application/x-custom".to_string()));
    }

    #[test]
    #[ignore = "Attachment struct is non-exhaustive in serenity v0.12, only available from API responses"]
    fn test_extract_media_from_attachments() {
        // Skip test - can't construct Attachment directly in v0.12
    }

    #[test]
    #[ignore = "Attachment struct is non-exhaustive in serenity v0.12, only available from API responses"]
    fn test_extract_media_multiple() {
        // Skip test - can't construct Attachment directly in v0.12
    }

    #[test]
    fn test_validate_media_file_size() {
        let media = Media {
            media_type: MediaType::Image,
            url: None,
            data: Some(vec![0u8; (MAX_FILE_SIZE as usize) + 1]),
            filename: Some("test.png".to_string()),
            mime_type: Some("image/png".to_string()),
            size_bytes: Some(MAX_FILE_SIZE + 1),
        };

        let result = validate_media(&media);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_media_empty_filename() {
        let media = Media {
            media_type: MediaType::Image,
            url: None,
            data: Some(vec![0u8; 100]),
            filename: Some("".to_string()),
            mime_type: Some("image/png".to_string()),
            size_bytes: Some(100),
        };

        let result = validate_media(&media);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_media_valid() {
        let media = Media {
            media_type: MediaType::Image,
            url: None,
            data: Some(vec![0u8; 100]),
            filename: Some("test.png".to_string()),
            mime_type: Some("image/png".to_string()),
            size_bytes: Some(100),
        };

        let result = validate_media(&media);
        assert!(result.is_ok());
    }
}
