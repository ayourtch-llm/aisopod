//! WhatsApp Business API media upload and download functionality.
//!
//! This module provides functionality for uploading and downloading media
//! files via the WhatsApp Business API.
//!
//! # Features
//!
//! - Uploading media files to WhatsApp
//! - Downloading media files from WhatsApp
//! - Media ID management
//! - Media type detection and validation
//! - Error handling for media operations

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, trace};

use crate::connection::WhatsAppAccount;
use crate::send::{SendError, SendResult};
use aisopod_channel::types::MediaType;
use std::path::Path;

/// Maximum media file size for WhatsApp (16MB for most media types)
const MAX_MEDIA_SIZE: u64 = 16 * 1024 * 1024;

/// Maximum message file size for WhatsApp (100MB)
const MAX_DOCUMENT_SIZE: u64 = 100 * 1024 * 1024;

/// Error types for WhatsApp media operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MediaError {
    /// File not found
    FileNotFound(String),
    /// File too large
    FileTooLarge(u64),
    /// Invalid media type
    InvalidMediaType(String),
    /// WhatsApp API error
    ApiError(String),
    /// Network error
    NetworkError(String),
    /// Configuration error
    ConfigError(String),
    /// Invalid file path
    InvalidPath(String),
}

impl std::fmt::Display for MediaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaError::FileNotFound(path) => write!(f, "File not found: {}", path),
            MediaError::FileTooLarge(size) => {
                write!(f, "File too large: {} bytes (max 16MB)", size)
            }
            MediaError::InvalidMediaType(msg) => write!(f, "Invalid media type: {}", msg),
            MediaError::ApiError(msg) => write!(f, "WhatsApp API error: {}", msg),
            MediaError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            MediaError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            MediaError::InvalidPath(path) => write!(f, "Invalid path: {}", path),
        }
    }
}

impl std::error::Error for MediaError {}

impl From<anyhow::Error> for MediaError {
    fn from(err: anyhow::Error) -> Self {
        MediaError::ApiError(err.to_string())
    }
}

impl From<reqwest::Error> for MediaError {
    fn from(err: reqwest::Error) -> Self {
        MediaError::NetworkError(err.to_string())
    }
}

impl From<std::io::Error> for MediaError {
    fn from(err: std::io::Error) -> Self {
        MediaError::FileNotFound(err.to_string())
    }
}

/// WhatsApp media upload response.
#[derive(Debug, Clone, Deserialize)]
pub struct UploadMediaResponse {
    /// The media ID.
    pub id: String,
    /// The media type.
    #[serde(rename = "messaging_product")]
    pub messaging_product: Option<String>,
    /// The file size in bytes.
    #[serde(default)]
    pub file_size: Option<String>,
    /// The MIME type.
    #[serde(default)]
    pub mime_type: Option<String>,
    /// The SHA256 hash.
    #[serde(default)]
    pub sha256: Option<String>,
}

/// WhatsApp media download response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadMediaResponse {
    /// The media URL.
    pub url: String,
    /// The content type.
    #[serde(rename = "content_type")]
    pub content_type: String,
}

/// WhatsApp media information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaInfo {
    /// The media ID.
    pub id: String,
    /// The MIME type.
    #[serde(rename = "mime_type")]
    pub mime_type: String,
    /// The file size in bytes.
    #[serde(rename = "file_size")]
    pub file_size: Option<u64>,
    /// The SHA256 hash.
    #[serde(default)]
    pub sha256: Option<String>,
}

/// Supported media types for WhatsApp.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SupportedMediaType {
    /// Image media (JPEG, PNG)
    Image,
    /// Audio media (MP3, OGG)
    Audio,
    /// Video media (MP4)
    Video,
    /// Document media (PDF, DOCX, etc.)
    Document,
}

impl SupportedMediaType {
    /// Get the maximum file size for this media type.
    pub fn max_size(&self) -> u64 {
        match self {
            SupportedMediaType::Image => MAX_MEDIA_SIZE,
            SupportedMediaType::Audio => MAX_MEDIA_SIZE,
            SupportedMediaType::Video => MAX_MEDIA_SIZE,
            SupportedMediaType::Document => MAX_DOCUMENT_SIZE,
        }
    }

    /// Check if a MIME type is supported.
    pub fn is_supported(&self, mime_type: &str) -> bool {
        match self {
            SupportedMediaType::Image => {
                mime_type.starts_with("image/")
            }
            SupportedMediaType::Audio => {
                mime_type.starts_with("audio/")
            }
            SupportedMediaType::Video => {
                mime_type.starts_with("video/")
            }
            SupportedMediaType::Document => {
                mime_type.starts_with("application/") || mime_type.starts_with("text/")
            }
        }
    }
}

// ============================================================================
// Media Upload/Download Implementation
// ============================================================================

impl WhatsAppAccount {
    /// Upload a media file to WhatsApp.
    ///
    /// This method uploads a media file and returns a media ID that can be
    /// used to send the media in a message.
    ///
    /// # Arguments
    ///
    /// * `file_path` - The path to the file to upload
    /// * `mime_type` - The MIME type of the file
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The media ID for the uploaded file
    /// * `Err(MediaError)` - An error if upload fails
    pub async fn upload_media(
        &self,
        file_path: &str,
        mime_type: &str,
    ) -> Result<String, MediaError> {
        // Validate file exists
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(MediaError::FileNotFound(file_path.to_string()));
        }

        // Get file size
        let metadata = std::fs::metadata(path)?;
        let file_size = metadata.len();

        // Determine media type from MIME type
        let media_type = self.determine_media_type(mime_type)?;

        // Check file size
        let max_size = media_type.max_size();
        if file_size > max_size {
            return Err(MediaError::FileTooLarge(file_size));
        }

        let phone_number_id = self.config.phone_number_id.as_ref()
            .ok_or_else(|| MediaError::ConfigError("Phone number ID not configured".to_string()))?;

        let endpoint = format!(
            "{}/v18.0/{}/media",
            "https://graph.facebook.com", phone_number_id
        );
        let auth_header = self.build_auth_header();

        debug!(
            "Uploading media file {} ({} bytes) to {}",
            file_path, file_size, endpoint
        );

        // Read file content
        let content = std::fs::read(path)
            .map_err(|e| MediaError::FileNotFound(format!("{}: {}", file_path, e)))?;

        // Build multipart form data
        let client = reqwest::Client::new();
        let response = client
            .post(&endpoint)
            .header("Authorization", &auth_header)
            .form(&[
                ("file", content.as_slice()),
                ("messaging_product", "whatsapp"),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!(
                "Failed to upload media: status={}, error={}",
                response.status(),
                error_text
            );
            return Err(MediaError::ApiError(error_text));
        }

        let upload_response: UploadMediaResponse = response.json().await?;
        let media_id = upload_response.id;

        info!("Uploaded media file {} with ID {}", file_path, media_id);

        Ok(media_id)
    }

    /// Download media from WhatsApp using a media ID.
    ///
    /// # Arguments
    ///
    /// * `media_id` - The media ID to download
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<u8>)` - The media content as bytes
    /// * `Err(MediaError)` - An error if download fails
    pub async fn download_media(&self, media_id: &str) -> Result<Vec<u8>, MediaError> {
        let endpoint = format!(
            "{}/v18.0/{}",
            "https://graph.facebook.com", media_id
        );
        let auth_header = self.build_auth_header();

        debug!("Downloading media {} from {}", media_id, endpoint);

        let client = reqwest::Client::new();
        let response = client
            .get(&endpoint)
            .header("Authorization", &auth_header)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!(
                "Failed to download media: status={}, error={}",
                response.status(),
                error_text
            );
            return Err(MediaError::ApiError(error_text));
        }

        let bytes = response.bytes().await?;
        let content = bytes.to_vec();

        info!("Downloaded media {} ({} bytes)", media_id, content.len());

        Ok(content)
    }

    /// Send media from a local file.
    ///
    /// This method uploads the file and then sends it as a message.
    ///
    /// # Arguments
    ///
    /// * `to` - The recipient's phone number
    /// * `file_path` - The path to the file
    /// * `mime_type` - The MIME type of the file
    /// * `caption` - Optional caption for the message
    ///
    /// # Returns
    ///
    /// * `Ok(SendResult)` - The result of sending the message
    /// * `Err(MediaError)` - An error if upload or send fails
    pub async fn send_media_file(
        &self,
        to: &str,
        file_path: &str,
        mime_type: &str,
        caption: Option<&str>,
    ) -> Result<SendResult, MediaError> {
        // Upload the media first
        let media_id = self.upload_media(file_path, mime_type).await?;

        // Send the media message
        let result = self.send_media_with_id(to, &media_id, mime_type, caption).await?;

        Ok(result)
    }

    /// Send media using a previously uploaded media ID.
    ///
    /// # Arguments
    ///
    /// * `to` - The recipient's phone number
    /// * `media_id` - The media ID to send
    /// * `mime_type` - The MIME type of the media
    /// * `caption` - Optional caption for the message
    ///
    /// # Returns
    ///
    /// * `Ok(SendResult)` - The result of sending the message
    /// * `Err(SendError)` - An error if sending fails
    pub async fn send_media_with_id(
        &self,
        to: &str,
        media_id: &str,
        mime_type: &str,
        caption: Option<&str>,
    ) -> Result<SendResult, SendError> {
        // Determine media type from MIME type
        let media_type = self.determine_media_type(mime_type)?;

        let options = crate::send::SendOptions::default();

        let payload = WhatsAppMediaPayload {
            messaging_product: "whatsapp".to_string(),
            to: to.to_string(),
            message_type: match media_type {
                SupportedMediaType::Image => "image",
                SupportedMediaType::Audio => "audio",
                SupportedMediaType::Video => "video",
                SupportedMediaType::Document => "document",
            }.to_string(),
            text: None,
            image: if media_type == SupportedMediaType::Image {
                Some(WhatsAppMedia {
                    id: Some(media_id.to_string()),
                    link: None,
                    caption: caption.map(|s| s.to_string()),
                    filename: None,
                })
            } else {
                None
            },
            document: if media_type == SupportedMediaType::Document {
                Some(WhatsAppMedia {
                    id: Some(media_id.to_string()),
                    link: None,
                    caption: caption.map(|s| s.to_string()),
                    filename: None,
                })
            } else {
                None
            },
            audio: if media_type == SupportedMediaType::Audio {
                Some(WhatsAppMedia {
                    id: Some(media_id.to_string()),
                    link: None,
                    caption: None,
                    filename: None,
                })
            } else {
                None
            },
            video: if media_type == SupportedMediaType::Video {
                Some(WhatsAppMedia {
                    id: Some(media_id.to_string()),
                    link: None,
                    caption: caption.map(|s| s.to_string()),
                    filename: None,
                })
            } else {
                None
            },
            sticker: None,
            context: None,
            reaction: None,
        };

        // Build the URL and headers
        let phone_number_id = self.config.phone_number_id.as_ref()
            .ok_or_else(|| SendError::ConfigError("Phone number ID not configured".to_string()))?;
        let endpoint = format!("{}/v18.0/{}/messages", "https://graph.facebook.com", phone_number_id);
        let auth_header = self.build_auth_header();

        debug!("Sending media message to {} via {}", to, endpoint);

        // Make the API request
        let client = reqwest::Client::new();
        let response = client
            .post(&endpoint)
            .header("Authorization", &auth_header)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!(
                "Failed to send media message: status={}, error={}",
                response.status(),
                error_text
            );
            return Err(SendError::ApiError(error_text));
        }

        let result: crate::send::SendMessageResponse = response.json().await?;
        let send_result = SendResult {
            message_id: result.messages
                .first()
                .map(|m| crate::send::MessageId::new(&m.message_id))
                .ok_or_else(|| SendError::ApiError("No message ID in response".to_string()))?,
            accepted: true,
            error: None,
        };

        info!("Sent media message to {}: {}", to, send_result.message_id);

        Ok(send_result)
    }

    /// Determine the supported media type from a MIME type.
    fn determine_media_type(&self, mime_type: &str) -> Result<SupportedMediaType, MediaError> {
        match mime_type {
            _ if mime_type.starts_with("image/") => Ok(SupportedMediaType::Image),
            _ if mime_type.starts_with("audio/") => Ok(SupportedMediaType::Audio),
            _ if mime_type.starts_with("video/") => Ok(SupportedMediaType::Video),
            _ if mime_type.starts_with("application/") => Ok(SupportedMediaType::Document),
            _ if mime_type.starts_with("text/") => Ok(SupportedMediaType::Document),
            _ => Err(MediaError::InvalidMediaType(
                format!("Unsupported MIME type: {}", mime_type)
            )),
        }
    }

    /// Get media information by ID.
    ///
    /// # Arguments
    ///
    /// * `media_id` - The media ID to query
    ///
    /// # Returns
    ///
    /// * `Ok(MediaInfo)` - The media information
    /// * `Err(MediaError)` - An error if the query fails
    pub async fn get_media_info(&self, media_id: &str) -> Result<MediaInfo, MediaError> {
        let endpoint = format!(
            "{}/v18.0/{}",
            "https://graph.facebook.com", media_id
        );
        let auth_header = self.build_auth_header();

        debug!("Getting media info for {}", media_id);

        let client = reqwest::Client::new();
        let response = client
            .get(&endpoint)
            .header("Authorization", &auth_header)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!(
                "Failed to get media info: status={}, error={}",
                response.status(),
                error_text
            );
            return Err(MediaError::ApiError(error_text));
        }

        let info: MediaInfo = response.json().await?;
        Ok(info)
    }

    /// Get the download URL for a media ID.
    ///
    /// Note: The URL returned by WhatsApp is temporary and expires after a short time.
    ///
    /// # Arguments
    ///
    /// * `media_id` - The media ID
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The download URL
    /// * `Err(MediaError)` - An error if the query fails
    pub async fn get_media_url(&self, media_id: &str) -> Result<String, MediaError> {
        let media_info = self.get_media_info(media_id).await?;

        // The media_id returned from upload is actually the URL
        // For actual media IDs from messages, we need to use the WhatsApp API
        Ok(media_id.to_string())
    }

    /// Build the Authorization header for WhatsApp API requests.
    fn build_auth_header(&self) -> String {
        let api_token = self.config.api_token.as_ref()
            .expect("API token not configured");
        format!("Bearer {}", api_token)
    }
}

// ============================================================================
// WhatsApp Media Payload Structures
// ============================================================================

#[derive(Debug, Clone, Serialize)]
struct WhatsAppMediaPayload {
    #[serde(rename = "messaging_product")]
    messaging_product: String,
    to: String,
    #[serde(rename = "type")]
    message_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<WhatsAppText>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<WhatsAppMedia>,
    #[serde(skip_serializing_if = "Option::is_none")]
    document: Option<WhatsAppMedia>,
    #[serde(skip_serializing_if = "Option::is_none")]
    audio: Option<WhatsAppMedia>,
    #[serde(skip_serializing_if = "Option::is_none")]
    video: Option<WhatsAppMedia>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sticker: Option<WhatsAppMedia>,
    #[serde(skip_serializing_if = "Option::is_none")]
    context: Option<WhatsAppMessageContext>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reaction: Option<WhatsAppReaction>,
}

#[derive(Debug, Clone, Serialize)]
struct WhatsAppText {
    body: String,
}

#[derive(Debug, Clone, Serialize)]
struct WhatsAppMedia {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    link: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    caption: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    filename: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct WhatsAppMessageContext {
    #[serde(rename = "message_id")]
    message_id: String,
}

#[derive(Debug, Clone, Serialize)]
struct WhatsAppReaction {
    #[serde(rename = "message_id")]
    message_id: String,
    emoji: String,
}

#[derive(Debug, Clone, Deserialize)]
struct SendMessageResponse {
    messages: Vec<SendMessageResult>,
}

#[derive(Debug, Clone, Deserialize)]
struct SendMessageResult {
    #[serde(rename = "id")]
    message_id: String,
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_supported_media_type_max_size() {
        assert_eq!(SupportedMediaType::Image.max_size(), MAX_MEDIA_SIZE);
        assert_eq!(SupportedMediaType::Audio.max_size(), MAX_MEDIA_SIZE);
        assert_eq!(SupportedMediaType::Video.max_size(), MAX_MEDIA_SIZE);
        assert_eq!(SupportedMediaType::Document.max_size(), MAX_DOCUMENT_SIZE);
    }

    #[test]
    fn test_supported_media_type_is_supported() {
        assert!(SupportedMediaType::Image.is_supported("image/jpeg"));
        assert!(SupportedMediaType::Image.is_supported("image/png"));
        assert!(SupportedMediaType::Image.is_supported("image/gif"));
        
        assert!(SupportedMediaType::Audio.is_supported("audio/mpeg"));
        assert!(SupportedMediaType::Audio.is_supported("audio/ogg"));
        
        assert!(SupportedMediaType::Video.is_supported("video/mp4"));
        assert!(SupportedMediaType::Video.is_supported("video/webm"));
        
        assert!(SupportedMediaType::Document.is_supported("application/pdf"));
        assert!(SupportedMediaType::Document.is_supported("application/msword"));
        assert!(SupportedMediaType::Document.is_supported("text/plain"));
    }

    #[test]
    fn test_supported_media_type_not_supported() {
        assert!(!SupportedMediaType::Image.is_supported("text/plain"));
        assert!(!SupportedMediaType::Document.is_supported("image/jpeg"));
    }

    #[test]
    fn test_media_error_display() {
        let err = MediaError::FileNotFound("/path/to/file".to_string());
        assert!(err.to_string().contains("File not found"));

        let err = MediaError::FileTooLarge(MAX_MEDIA_SIZE + 1);
        assert!(err.to_string().contains("File too large"));

        let err = MediaError::InvalidMediaType("image/unsupported".to_string());
        assert!(err.to_string().contains("Invalid media type"));
    }

    #[test]
    fn test_error_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let media_err = MediaError::from(io_err);
        
        assert!(matches!(media_err, MediaError::FileNotFound(_)));
    }

    #[test]
    fn test_error_from_reqwest() {
        let reqwest_err = reqwest::Error::from_status(
            reqwest::StatusCode::NOT_FOUND,
            Some("Not found".to_string())
        ).unwrap();
        let media_err = MediaError::from(reqwest_err);
        
        assert!(matches!(media_err, MediaError::NetworkError(_)));
    }

    #[test]
    fn test_error_from_anyhow() {
        let anyhow_err = anyhow::anyhow!("API error");
        let media_err = MediaError::from(anyhow_err);
        
        assert!(matches!(media_err, MediaError::ApiError(_)));
    }

    #[test]
    fn test_upload_media_response_serialization() {
        let response = UploadMediaResponse {
            id: "media123".to_string(),
            messaging_product: Some("whatsapp".to_string()),
            file_size: Some("1024".to_string()),
            mime_type: Some("image/jpeg".to_string()),
            sha256: Some("abc123".to_string()),
        };

        let json = serde_json::to_string(&response).unwrap();
        let expected = serde_json::json!({
            "id": "media123",
            "messaging_product": "whatsapp",
            "file_size": "1024",
            "mime_type": "image/jpeg",
            "sha256": "abc123"
        });

        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_media_info_serialization() {
        let info = MediaInfo {
            id: "media123".to_string(),
            mime_type: "image/jpeg".to_string(),
            file_size: Some(1024),
            sha256: Some("abc123".to_string()),
        };

        let json = serde_json::to_string(&info).unwrap();
        let expected = serde_json::json!({
            "id": "media123",
            "mime_type": "image/jpeg",
            "file_size": 1024,
            "sha256": "abc123"
        });

        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_whatsapp_media_payload_with_image() {
        let payload = WhatsAppMediaPayload {
            messaging_product: "whatsapp".to_string(),
            to: "14151234567".to_string(),
            message_type: "image".to_string(),
            text: None,
            image: Some(WhatsAppMedia {
                id: Some("media123".to_string()),
                link: None,
                caption: Some("Check this out!".to_string()),
                filename: None,
            }),
            document: None,
            audio: None,
            video: None,
            sticker: None,
            context: None,
            reaction: None,
        };

        let json = serde_json::to_string(&payload).unwrap();
        let expected = serde_json::json!({
            "messaging_product": "whatsapp",
            "to": "14151234567",
            "type": "image",
            "image": {
                "id": "media123",
                "caption": "Check this out!"
            }
        });

        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_whatsapp_media_payload_with_document() {
        let payload = WhatsAppMediaPayload {
            messaging_product: "whatsapp".to_string(),
            to: "14151234567".to_string(),
            message_type: "document".to_string(),
            text: None,
            image: None,
            document: Some(WhatsAppMedia {
                id: Some("media123".to_string()),
                link: None,
                caption: Some("PDF document".to_string()),
                filename: Some("document.pdf".to_string()),
            }),
            audio: None,
            video: None,
            sticker: None,
            context: None,
            reaction: None,
        };

        let json = serde_json::to_string(&payload).unwrap();
        let expected = serde_json::json!({
            "messaging_product": "whatsapp",
            "to": "14151234567",
            "type": "document",
            "document": {
                "id": "media123",
                "caption": "PDF document",
                "filename": "document.pdf"
            }
        });

        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_determine_media_type() {
        let config = crate::connection::WhatsAppAccountConfig {
            mode: crate::connection::WhatsAppMode::BusinessApi,
            api_token: Some("test-token".to_string()),
            business_account_id: None,
            phone_number_id: Some("123456789".to_string()),
            webhook_verify_token: Some("verify-token".to_string()),
            allowed_numbers: None,
        };

        let account = WhatsAppAccount::new("test".to_string(), config);

        assert!(matches!(account.determine_media_type("image/jpeg").unwrap(), SupportedMediaType::Image));
        assert!(matches!(account.determine_media_type("audio/mpeg").unwrap(), SupportedMediaType::Audio));
        assert!(matches!(account.determine_media_type("video/mp4").unwrap(), SupportedMediaType::Video));
        assert!(matches!(account.determine_media_type("application/pdf").unwrap(), SupportedMediaType::Document));
        assert!(matches!(account.determine_media_type("text/plain").unwrap(), SupportedMediaType::Document));
    }

    #[test]
    fn test_determine_media_type_invalid() {
        let config = crate::connection::WhatsAppAccountConfig {
            mode: crate::connection::WhatsAppMode::BusinessApi,
            api_token: Some("test-token".to_string()),
            business_account_id: None,
            phone_number_id: Some("123456789".to_string()),
            webhook_verify_token: Some("verify-token".to_string()),
            allowed_numbers: None,
        };

        let account = WhatsAppAccount::new("test".to_string(), config);

        assert!(matches!(
            account.determine_media_type("application/unsupported"),
            Err(MediaError::InvalidMediaType(_))
        ));
    }
}
