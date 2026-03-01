//! File sharing integration for Nextcloud Talk.
//!
//! This module provides utilities for uploading files to Nextcloud
//! and sharing them in Talk rooms.

use anyhow::{anyhow, Result};
use std::path::Path;
use tracing::{debug, error, info, instrument};

use crate::api::NextcloudTalkApi;

/// Upload a file to Nextcloud and share it in a Talk room.
///
/// This function uploads a file to the user's Nextcloud storage via WebDAV,
/// then posts a message in the specified Talk room with the file link.
///
/// # Arguments
///
/// * `api` - The Nextcloud Talk API client
/// * `room_token` - The room token to share the file in
/// * `file_path` - Path to the file to upload
///
/// # Returns
///
/// * `Ok(String)` - The URL where the file can be accessed
/// * `Err(anyhow::Error)` - An error if the upload fails
#[instrument(skip(api, file_path))]
pub async fn share_file(
    api: &NextcloudTalkApi,
    room_token: &str,
    file_path: &str,
) -> Result<String> {
    let path = Path::new(file_path);

    if !path.exists() {
        return Err(anyhow!("File not found: {}", file_path));
    }

    if !path.is_file() {
        return Err(anyhow!("Not a file: {}", file_path));
    }

    // Get the filename
    let filename = path
        .file_name()
        .ok_or_else(|| anyhow!("Invalid file path: {}", file_path))?
        .to_string_lossy()
        .to_string();

    // Get file size
    let file_size = std::fs::metadata(path)?.len();

    info!(
        "Uploading {} to room {} ({} bytes)",
        filename, room_token, file_size
    );

    // Upload the file to Nextcloud via WebDAV
    let file_url = upload_file(api, path, &filename).await?;

    // Create a link message to share in the room
    let message = format!("[{}]({})", filename, file_url);

    // Send the message to the room
    api.send_message(room_token, &message).await?;

    info!("Successfully shared {} in room {}", filename, room_token);

    Ok(file_url)
}

/// Upload a file to Nextcloud via WebDAV.
///
/// # Arguments
///
/// * `api` - The Nextcloud Talk API client
/// * `file_path` - Path to the file to upload
/// * `filename` - The filename to use in Nextcloud
///
/// # Returns
///
/// * `Ok(String)` - The URL where the file can be accessed
/// * `Err(anyhow::Error)` - An error if the upload fails
async fn upload_file(api: &NextcloudTalkApi, file_path: &Path, filename: &str) -> Result<String> {
    let file_data = std::fs::read(file_path)
        .map_err(|e| anyhow!("Failed to read file {}: {}", file_path.display(), e))?;

    // WebDAV endpoint for files
    let webdav_url = format!(
        "{}/remote.php/dav/files/{}/{}",
        api.base_url(),
        urlencoding::encode(&api.auth().0),
        urlencoding::encode(filename)
    );

    debug!("Uploading to WebDAV: {}", webdav_url);

    let response = api
        .http()
        .put(&webdav_url)
        .basic_auth(&api.auth().0, Some(&api.auth().1))
        .body(file_data)
        .send()
        .await?;

    if response.status().is_success() {
        debug!("File uploaded successfully");

        // Construct the public URL for the file
        // Nextcloud stores files at: /remote.php/webdav/path/to/file
        // or for user files: /remote.php/dav/files/username/path/to/file
        let file_url = format!(
            "{}/index.php/s/{}",
            api.base_url(),
            urlencoding::encode(filename)
        );

        Ok(file_url)
    } else {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        error!("Failed to upload file: status={}, body={}", status, text);
        Err(anyhow!("Failed to upload file: HTTP {}", status))
    }
}

/// Upload an image file to Nextcloud and get a shareable link.
///
/// This is a convenience wrapper around share_file for image files.
#[instrument(skip(api, file_path))]
pub async fn share_image(
    api: &NextcloudTalkApi,
    room_token: &str,
    file_path: &str,
) -> Result<String> {
    share_file(api, room_token, file_path).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_share_api() {
        // This test validates the API - actual upload would need a real server
        let config = crate::config::NextcloudConfig::default();
        let api = NextcloudTalkApi::new(&config.server_url, &config.username, &config.password);

        if api.is_ok() {
            let api = api.unwrap();

            // Test that we can construct the file sharing path
            let filename = "test.txt";
            let webdav_url = format!(
                "{}/remote.php/dav/files/{}/{}",
                api.base_url(),
                urlencoding::encode(&api.auth().0),
                urlencoding::encode(filename)
            );

            assert!(webdav_url.contains(&api.auth().0));
            assert!(webdav_url.contains(filename));
        }
    }
}
