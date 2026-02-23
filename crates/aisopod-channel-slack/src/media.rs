//! File upload and download for Slack.
//!
//! This module provides functionality for uploading and downloading
//! files to and from Slack channels.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

// Import MediaType from aisopod_channel::types (not crate::types)
use aisopod_channel::types::MediaType;

/// Information about a Slack file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    /// The file ID
    pub id: String,
    /// The file name
    pub name: Option<String>,
    /// The file title
    pub title: Option<String>,
    /// The file type
    pub file_type: Option<String>,
    /// The MIME type
    pub mimetype: Option<String>,
    /// The file size in bytes
    pub size: Option<u64>,
    /// A URL where the file can be downloaded
    pub url_private_download: Option<String>,
    /// The number of shares
    pub shares: Option<serde_json::Value>,
    /// The timestamp when the file was created
    pub created: Option<i64>,
    /// The timestamp when the file was last modified
    pub modified: Option<i64>,
    /// Whether the file is public
    pub public: Option<bool>,
    /// Whether the file is visible to the user
    pub editable: Option<bool>,
    /// The number of lines (for text files)
    pub lines: Option<i64>,
    /// The number of pages (for PDF files)
    pub num_pages: Option<i64>,
    /// The channel IDs where the file is shared
    pub channels: Option<Vec<String>>,
    /// The user ID of the file owner
    pub user_id: Option<String>,
    /// The original URL of the file if uploaded from a URL
    pub url_private: Option<String>,
    /// The permalink of the file
    pub permalink: Option<String>,
    /// The public permalink of the file
    pub permalink_public: Option<String>,
}

impl FileInfo {
    /// Check if the file has been successfully created.
    pub fn is_ok(&self) -> bool {
        !self.id.is_empty()
    }

    /// Get the file ID.
    pub fn get_id(&self) -> &str {
        &self.id
    }

    /// Get the download URL if available.
    pub fn get_download_url(&self) -> Option<&str> {
        self.url_private_download.as_deref()
    }
}

/// Response from a file upload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadResponse {
    /// Whether the request was successful
    pub ok: bool,
    /// The file information if upload succeeded
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<FileInfo>,
    /// Error information if the request failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl UploadResponse {
    /// Check if the upload was successful.
    pub fn is_ok(&self) -> bool {
        self.ok
    }

    /// Get the uploaded file information if available.
    pub fn get_file(&self) -> Option<&FileInfo> {
        self.file.as_ref()
    }

    /// Get the error message if the request failed.
    pub fn get_error(&self) -> Option<&str> {
        self.error.as_deref()
    }
}

/// Response from a file download.
#[derive(Debug, Clone)]
pub struct DownloadResponse {
    /// The downloaded file data
    pub data: Vec<u8>,
    /// The MIME type of the file
    pub mime_type: Option<String>,
    /// The filename
    pub filename: Option<String>,
}

impl DownloadResponse {
    /// Create a new download response.
    pub fn new(data: Vec<u8>, mime_type: Option<String>, filename: Option<String>) -> Self {
        Self {
            data,
            mime_type,
            filename,
        }
    }

    /// Save the downloaded file to disk.
    ///
    /// # Arguments
    ///
    /// * `path` - The path where the file should be saved
    ///
    /// # Returns
    ///
    /// * `Ok(())` - File was saved successfully
    /// * `Err(anyhow::Error)` - An error if saving fails
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        fs::write(path, &self.data)?;
        Ok(())
    }
}

/// Build the payload for files.uploadV2.
///
/// # Arguments
///
/// * `channel_ids` - The channel IDs to share the file with
/// * `file_content` - Optional file content as Base64 string
/// * `filename` - The filename
/// * `title` - The file title
/// * `initial_comment` - Optional initial comment
/// * `thread_ts` - Optional thread timestamp
///
/// # Returns
///
/// A JSON value representing the upload payload
pub fn build_upload_payload(
    channel_ids: &[String],
    file_content: Option<&str>,
    filename: &str,
    title: Option<&str>,
    initial_comment: Option<&str>,
    thread_ts: Option<&str>,
) -> serde_json::Value {
    let mut payload = serde_json::json!({
        "channel_ids": channel_ids,
        "filename": filename,
    });
    
    if let Some(content) = file_content {
        payload["content"] = serde_json::json!(content);
    }
    
    if let Some(t) = title {
        payload["title"] = serde_json::json!(t);
    }
    
    if let Some(comment) = initial_comment {
        payload["initial_comment"] = serde_json::json!(comment);
    }
    
    if let Some(ts) = thread_ts {
        payload["thread_ts"] = serde_json::json!(ts);
    }
    
    payload
}

/// Upload a file to Slack.
///
/// This uses the files.uploadV2 endpoint which allows uploading files
/// and sharing them with multiple channels in a single call.
///
/// # Arguments
///
/// * `client` - The Slack client handle
/// * `channel_ids` - The channel IDs to share the file with
/// * `file_data` - The file data
/// * `filename` - The filename
/// * `title` - Optional file title
/// * `initial_comment` - Optional initial comment
/// * `thread_ts` - Optional thread timestamp
///
/// # Returns
///
/// * `Ok(UploadResponse)` - The response from Slack
/// * `Err(anyhow::Error)` - An error if upload fails
pub async fn upload_file(
    client: &crate::connection::SlackClientHandle,
    channel_ids: &[String],
    file_data: &[u8],
    filename: &str,
    title: Option<&str>,
    initial_comment: Option<&str>,
    thread_ts: Option<&str>,
) -> Result<UploadResponse> {
    // Convert file data to Base64
    let content = base64::encode(file_data);
    
    let payload = build_upload_payload(
        channel_ids,
        Some(&content),
        filename,
        title,
        initial_comment,
        thread_ts,
    );
    
    let response = client
        .post("https://slack.com/api/files.uploadV2", &payload)
        .await?;
    
    let json: serde_json::Value = response.json().await?;
    let parsed: UploadResponse = serde_json::from_value(json)?;
    
    Ok(parsed)
}

/// Upload a file from a local path to Slack.
///
/// # Arguments
///
/// * `client` - The Slack client handle
/// * `channel_ids` - The channel IDs to share the file with
/// * `file_path` - The local path to the file
/// * `title` - Optional file title
/// * `initial_comment` - Optional initial comment
/// * `thread_ts` - Optional thread timestamp
///
/// # Returns
///
/// * `Ok(UploadResponse)` - The response from Slack
/// * `Err(anyhow::Error)` - An error if upload fails
pub async fn upload_file_from_path(
    client: &crate::connection::SlackClientHandle,
    channel_ids: &[String],
    file_path: &Path,
    title: Option<&str>,
    initial_comment: Option<&str>,
    thread_ts: Option<&str>,
) -> Result<UploadResponse> {
    let filename = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file")
        .to_string();
    
    let file_data = fs::read(file_path)?;
    
    upload_file(
        client,
        channel_ids,
        &file_data,
        &filename,
        title,
        initial_comment,
        thread_ts,
    )
    .await
}

/// Download a file from Slack.
///
/// # Arguments
///
/// * `client` - The Slack client handle
/// * `file_id` - The file ID to download
///
/// # Returns
///
/// * `Ok(DownloadResponse)` - The downloaded file data
/// * `Err(anyhow::Error)` - An error if download fails
pub async fn download_file(
    client: &crate::connection::SlackClientHandle,
    file_id: &str,
) -> Result<DownloadResponse> {
    // First, get file info to get the download URL
    let file_info = get_file_info(client, file_id).await?;
    
    let url = file_info
        .get_download_url()
        .ok_or_else(|| anyhow::anyhow!("No download URL available for file {}", file_id))?;
    
    // Download the file using the private URL
    let response = client.get(url).await?;
    
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await?;
        return Err(anyhow::anyhow!("File download failed with status {}: {}", status, body));
    }
    
    // Get mime_type before consuming response with bytes()
    let mime_type = response
        .headers()
        .get("content-type")
        .and_then(|v: &reqwest::header::HeaderValue| v.to_str().ok())
        .map(|s| s.to_string());
    
    // Now get the bytes
    let bytes = response.bytes().await?;
    let bytes_vec = bytes.to_vec();
    
    let filename = file_info
        .name
        .or(file_info.title)
        .unwrap_or_else(|| "downloaded_file".to_string());
    
    Ok(DownloadResponse::new(
        bytes_vec,
        mime_type,
        Some(filename),
    ))
}

/// Get information about a file.
///
/// # Arguments
///
/// * `client` - The Slack client handle
/// * `file_id` - The file ID to get information for
///
/// # Returns
///
/// * `Ok(FileInfo)` - The file information
/// * `Err(anyhow::Error)` - An error if the request fails
pub async fn get_file_info(
    client: &crate::connection::SlackClientHandle,
    file_id: &str,
) -> Result<FileInfo> {
    let payload = serde_json::json!({
        "file": file_id
    });
    
    let response = client
        .post("https://slack.com/api/files.info", &payload)
        .await?;
    
    let json: serde_json::Value = response.json().await?;
    
    if let Some(error) = json.get("error") {
        return Err(anyhow::anyhow!("files.info failed: {}", error.as_str().unwrap_or("Unknown error")));
    }
    
    let file: FileInfo = serde_json::from_value(json["file"].clone())?;
    Ok(file)
}

/// List files in a channel or conversation.
///
/// # Arguments
///
/// * `client` - The Slack client handle
/// * `channel` - The channel ID to list files from
/// * `ts_from` - Optional start timestamp
/// * `ts_to` - Optional end timestamp
/// * `types` - Optional file types to filter by
/// * `count` - Number of results per page
/// * `page` - Page number for pagination
///
/// # Returns
///
/// * `Ok(serde_json::Value)` - The paginated file list response
/// * `Err(anyhow::Error)` - An error if the request fails
pub async fn list_files(
    client: &crate::connection::SlackClientHandle,
    channel: &str,
    ts_from: Option<i64>,
    ts_to: Option<i64>,
    types: Option<&str>,
    count: Option<u32>,
    page: Option<u32>,
) -> Result<serde_json::Value> {
    let mut payload = serde_json::json!({
        "channel": channel
    });
    
    if let Some(ts) = ts_from {
        payload["ts_from"] = serde_json::json!(ts);
    }
    if let Some(ts) = ts_to {
        payload["ts_to"] = serde_json::json!(ts);
    }
    if let Some(t) = types {
        payload["types"] = serde_json::json!(t);
    }
    if let Some(c) = count {
        payload["count"] = serde_json::json!(c);
    }
    if let Some(p) = page {
        payload["page"] = serde_json::json!(p);
    }
    
    let response = client
        .post("https://slack.com/api/files.list", &payload)
        .await?;
    
    let json: serde_json::Value = response.json().await?;
    Ok(json)
}

/// Delete a file from Slack.
///
/// # Arguments
///
/// * `client` - The Slack client handle
/// * `file_id` - The file ID to delete
///
/// # Returns
///
/// * `Ok(())` - File was deleted successfully
/// * `Err(anyhow::Error)` - An error if deletion fails
pub async fn delete_file(
    client: &crate::connection::SlackClientHandle,
    file_id: &str,
) -> Result<()> {
    let payload = serde_json::json!({
        "file": file_id
    });
    
    let response = client
        .post("https://slack.com/api/files.delete", &payload)
        .await?;
    
    let json: serde_json::Value = response.json().await?;
    let success: bool = json["ok"].as_bool().unwrap_or(false);
    
    if !success {
        let error = json["error"].as_str().unwrap_or("Unknown error");
        return Err(anyhow::anyhow!("files.delete failed: {}", error));
    }
    
    Ok(())
}

/// Convert a Slack MediaType to a MIME type string.
///
/// # Arguments
///
/// * `media_type` - The Slack media type
///
/// # Returns
///
/// The corresponding MIME type string
pub fn media_type_to_mime(media_type: &MediaType) -> String {
    match media_type {
        MediaType::Image => "image/*".to_string(),
        MediaType::Audio => "audio/*".to_string(),
        MediaType::Video => "video/*".to_string(),
        MediaType::Document => "application/*".to_string(),
        MediaType::Other(other) => other.clone(),
    }
}

/// Convert a MIME type string to a Slack MediaType.
///
/// # Arguments
///
/// * `mime_type` - The MIME type string
///
/// # Returns
///
/// The corresponding Slack media type
pub fn mime_to_media_type(mime_type: &str) -> MediaType {
    let mime_lower = mime_type.to_lowercase();
    
    if mime_lower.starts_with("image/") {
        MediaType::Image
    } else if mime_lower.starts_with("audio/") {
        MediaType::Audio
    } else if mime_lower.starts_with("video/") {
        MediaType::Video
    } else if mime_lower.starts_with("application/pdf") {
        MediaType::Document
    } else if mime_lower.starts_with("application/") {
        MediaType::Document
    } else {
        MediaType::Other(mime_type.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_upload_payload() {
        let payload = build_upload_payload(
            &["C123456".to_string()],
            Some("SGVsbG8="),
            "test.txt",
            Some("Test File"),
            Some("Check this out!"),
            Some("1234567890.123456"),
        );
        
        assert_eq!(payload["channel_ids"][0], "C123456");
        assert_eq!(payload["filename"], "test.txt");
        assert_eq!(payload["title"], "Test File");
        assert_eq!(payload["initial_comment"], "Check this out!");
        assert_eq!(payload["thread_ts"], "1234567890.123456");
    }

    #[test]
    fn test_media_type_to_mime() {
        assert_eq!(media_type_to_mime(&MediaType::Image), "image/*");
        assert_eq!(media_type_to_mime(&MediaType::Audio), "audio/*");
        assert_eq!(media_type_to_mime(&MediaType::Video), "video/*");
        assert_eq!(media_type_to_mime(&MediaType::Document), "application/*");
        assert_eq!(media_type_to_mime(&MediaType::Other("custom".to_string())), "custom");
    }

    #[test]
    fn test_mime_to_media_type() {
        assert_eq!(mime_to_media_type("image/png"), MediaType::Image);
        assert_eq!(mime_to_media_type("audio/mpeg"), MediaType::Audio);
        assert_eq!(mime_to_media_type("video/mp4"), MediaType::Video);
        assert_eq!(mime_to_media_type("application/pdf"), MediaType::Document);
        assert_eq!(
            mime_to_media_type("application/custom"),
            MediaType::Document
        );
        assert_eq!(
            mime_to_media_type("text/plain"),
            MediaType::Other("text/plain".to_string())
        );
    }

    #[test]
    fn test_upload_response() {
        let response = UploadResponse {
            ok: true,
            file: Some(FileInfo {
                id: "F123456".to_string(),
                name: Some("test.txt".to_string()),
                title: None,
                file_type: None,
                mimetype: None,
                size: None,
                url_private_download: None,
                shares: None,
                created: None,
                modified: None,
                public: None,
                editable: None,
                lines: None,
                num_pages: None,
                channels: None,
                user_id: None,
                url_private: None,
                permalink: None,
                permalink_public: None,
            }),
            error: None,
        };
        
        assert!(response.is_ok());
        assert!(response.get_file().is_some());
        assert_eq!(response.get_file().unwrap().id, "F123456");
    }
}
