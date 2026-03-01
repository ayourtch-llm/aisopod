//! Media handling utilities for processing images, detecting media types, and more.
//!
//! This module provides common media processing utilities that can be used by channel
//! plugins to handle images, audio, and documents. It includes functions for:
//!
//! - Resizing images to channel-specific maximum dimensions
//! - Converting between image formats
//! - Detecting media types from magic bytes or file extensions
//! - Validating media against channel capabilities
//!
//! # Example
//!
//! ```rust,ignore
//! use aisopod_channel::media::{resize_image, detect_media_type, validate_media};
//! use aisopod_channel::{MediaType, ChannelCapabilities};
//!
//! let image_data = std::fs::read("example.jpg").unwrap();
//!
//! // Detect the media type
//! let media_type = detect_media_type(&image_data, Some("example.jpg"));
//! assert_eq!(media_type, MediaType::Image);
//!
//! // Resize the image to max 800x600
//! let resized = resize_image(&image_data, 800, 600).unwrap();
//!
//! // Validate against channel capabilities
//! let capabilities = ChannelCapabilities {
//!     supports_media: true,
//!     supported_media_types: vec![MediaType::Image],
//!     ..Default::default()
//! };
//! validate_media(&media_type, &capabilities).unwrap();
//! ```

use std::io::Cursor;

use anyhow::Result;
use image::{DynamicImage, GenericImageView, ImageFormat};

use crate::types::MediaType;

/// A trait for audio transcription services.
///
/// This trait provides an integration point for future implementation of audio
/// transcription functionality. Channel plugins or external services can implement
/// this trait to provide speech-to-text capabilities.
///
/// # Example
///
/// ```rust,ignore
/// use aisopod_channel::media::AudioTranscriber;
///
/// struct MyTranscriber;
///
/// #[async_trait::async_trait]
/// impl AudioTranscriber for MyTranscriber {
///     async fn transcribe(&self, audio_data: &[u8], mime_type: &str) -> Result<String> {
///         // Implementation for audio transcription
///         Ok("Transcribed text".to_string())
///     }
/// }
/// ```
#[async_trait::async_trait]
pub trait AudioTranscriber: Send + Sync {
    /// Transcribe audio data to text.
    ///
    /// # Arguments
    ///
    /// * `audio_data` - The raw audio bytes to transcribe
    /// * `mime_type` - The MIME type of the audio (e.g., "audio/mp3", "audio/wav")
    ///
    /// # Returns
    ///
    /// Returns the transcribed text on success.
    async fn transcribe(&self, audio_data: &[u8], mime_type: &str) -> Result<String>;
}

/// A trait for document text extraction services.
///
/// This trait provides an integration point for future implementation of document
/// text extraction functionality. Channel plugins or external services can implement
/// this trait to extract text from documents like PDFs or Office documents.
///
/// # Example
///
/// ```rust,ignore
/// use aisopod_channel::media::DocumentExtractor;
///
/// struct MyDocumentExtractor;
///
/// #[async_trait::async_trait]
/// impl DocumentExtractor for MyDocumentExtractor {
///     async fn extract_text(&self, doc_data: &[u8], mime_type: &str) -> Result<String> {
///         // Implementation for document text extraction
///         Ok("Extracted text".to_string())
///     }
/// }
/// ```
#[async_trait::async_trait]
pub trait DocumentExtractor: Send + Sync {
    /// Extract text from document data.
    ///
    /// # Arguments
    ///
    /// * `doc_data` - The raw document bytes to extract text from
    /// * `mime_type` - The MIME type of the document (e.g., "application/pdf", "application/docx")
    ///
    /// # Returns
    ///
    /// Returns the extracted text on success.
    async fn extract_text(&self, doc_data: &[u8], mime_type: &str) -> Result<String>;
}

/// Resize an image to fit within the specified maximum dimensions.
///
/// This function scales down an image while preserving its aspect ratio.
/// If the image is already smaller than the specified dimensions, it will
/// be returned unchanged.
///
/// # Arguments
///
/// * `data` - The raw image bytes to resize
/// * `max_width` - The maximum width in pixels
/// * `max_height` - The maximum height in pixels
///
/// # Returns
///
/// Returns the resized image bytes in the original format, or an error if
/// the image could not be decoded or encoded.
///
/// # Example
///
/// ```rust,ignore
/// let image_data = std::fs::read("photo.jpg").unwrap();
/// let resized = resize_image(&image_data, 800, 600).unwrap();
/// ```
pub fn resize_image(data: &[u8], max_width: u32, max_height: u32) -> Result<Vec<u8>> {
    let mut img = image::load_from_memory(data)?;

    let (width, height) = img.dimensions();

    // If the image is already within bounds, return it unchanged
    if width <= max_width && height <= max_height {
        // Re-encode in the original format
        let mut buf = Cursor::new(Vec::new());
        img.write_to(&mut buf, detect_image_format(data)?)?;
        return Ok(buf.into_inner());
    }

    // Calculate new dimensions while preserving aspect ratio
    let scale = f64::min(
        max_width as f64 / width as f64,
        max_height as f64 / height as f64,
    );

    let new_width = (width as f64 * scale) as u32;
    let new_height = (height as f64 * scale) as u32;

    // Resize using Lanczos3 filter for high quality
    img = DynamicImage::ImageRgba8(image::imageops::resize(
        &img.to_rgba8(),
        new_width,
        new_height,
        image::imageops::FilterType::Lanczos3,
    ));

    // Re-encode in the original format
    let mut buf = Cursor::new(Vec::new());
    img.write_to(&mut buf, detect_image_format(data)?)?;
    Ok(buf.into_inner())
}

/// Convert an image to a different format.
///
/// # Arguments
///
/// * `data` - The raw image bytes to convert
/// * `target_format` - The target image format
///
/// # Returns
///
/// Returns the image bytes in the target format, or an error if the image
/// could not be decoded or encoded.
///
/// # Example
///
/// ```rust,ignore
/// let png_data = std::fs::read("photo.png").unwrap();
/// let jpeg_data = convert_image_format(&png_data, ImageFormat::Jpeg).unwrap();
/// ```
pub fn convert_image_format(data: &[u8], target_format: ImageFormat) -> Result<Vec<u8>> {
    let img = image::load_from_memory(data)?;

    let mut buf = Cursor::new(Vec::new());
    img.write_to(&mut buf, target_format)?;
    Ok(buf.into_inner())
}

/// Detect the media type from raw bytes (magic bytes) or file extension.
///
/// This function first attempts to detect the media type by examining the
/// magic bytes at the beginning of the data. If that fails or if the data
/// doesn't match known formats, it falls back to checking the file extension.
///
/// # Arguments
///
/// * `data` - The raw bytes to analyze
/// * `filename` - Optional filename to check for extension-based detection
///
/// # Returns
///
/// Returns the detected `MediaType`.
///
/// # Magic Bytes Detected
///
/// - JPEG: `FF D8 FF`
/// - PNG: `89 50 4E 47`
/// - GIF: `47 49 46`
/// - PDF: `25 50 44 46`
pub fn detect_media_type(data: &[u8], filename: Option<&str>) -> MediaType {
    // Check magic bytes first
    if data.len() >= 3 {
        // JPEG: FF D8 FF
        if data[0] == 0xFF && data[1] == 0xD8 && data[2] == 0xFF {
            return MediaType::Image;
        }

        // GIF: 47 49 46 (GIF)
        if data[0] == b'G' && data[1] == b'I' && data[2] == b'F' {
            return MediaType::Image;
        }
    }

    if data.len() >= 4 {
        // PNG: 89 50 4E 47
        if data[0] == 0x89 && data[1] == b'P' && data[2] == b'N' && data[3] == b'G' {
            return MediaType::Image;
        }

        // PDF: 25 50 44 46 (%PDF)
        if data[0] == b'%' && data[1] == b'P' && data[2] == b'D' && data[3] == b'F' {
            return MediaType::Document;
        }
    }

    // Fall back to file extension
    if let Some(name) = filename {
        if let Some(ext) = std::path::Path::new(name)
            .extension()
            .and_then(|e| e.to_str())
        {
            let ext_lower = ext.to_lowercase();

            match ext_lower.as_str() {
                // Images
                "jpg" | "jpeg" => return MediaType::Image,
                "png" => return MediaType::Image,
                "gif" => return MediaType::Image,
                "webp" => return MediaType::Image,
                "bmp" => return MediaType::Image,

                // Audio
                "mp3" => return MediaType::Audio,
                "wav" => return MediaType::Audio,
                "ogg" => return MediaType::Audio,
                "flac" => return MediaType::Audio,
                "aac" => return MediaType::Audio,

                // Video
                "mp4" => return MediaType::Video,
                "webm" => return MediaType::Video,
                "mkv" => return MediaType::Video,
                "avi" => return MediaType::Video,

                // Documents
                "pdf" => return MediaType::Document,
                "doc" => return MediaType::Document,
                "docx" => return MediaType::Document,
                "txt" => return MediaType::Document,
                "rtf" => return MediaType::Document,

                _ => {}
            }
        }
    }

    // Unknown type
    MediaType::Other("unknown".to_string())
}

/// Detect the MIME type from raw bytes or file extension.
///
/// This function works similarly to `detect_media_type`, but returns
/// a MIME type string instead of a `MediaType` enum.
///
/// # Arguments
///
/// * `data` - The raw bytes to analyze
/// * `filename` - Optional filename to check for extension-based detection
///
/// # Returns
///
/// Returns the detected MIME type as a string.
pub fn detect_mime_type(data: &[u8], filename: Option<&str>) -> String {
    // Check magic bytes first for images
    if data.len() >= 3 {
        if data[0] == 0xFF && data[1] == 0xD8 && data[2] == 0xFF {
            return "image/jpeg".to_string();
        }

        if data[0] == b'G' && data[1] == b'I' && data[2] == b'F' {
            return "image/gif".to_string();
        }
    }

    if data.len() >= 4 {
        if data[0] == 0x89 && data[1] == b'P' && data[2] == b'N' && data[3] == b'G' {
            return "image/png".to_string();
        }

        if data[0] == b'%' && data[1] == b'P' && data[2] == b'D' && data[3] == b'F' {
            return "application/pdf".to_string();
        }
    }

    // Fall back to file extension
    if let Some(name) = filename {
        if let Some(ext) = std::path::Path::new(name)
            .extension()
            .and_then(|e| e.to_str())
        {
            let ext_lower = ext.to_lowercase();

            match ext_lower.as_str() {
                // Images
                "jpg" | "jpeg" => return "image/jpeg".to_string(),
                "png" => return "image/png".to_string(),
                "gif" => return "image/gif".to_string(),
                "webp" => return "image/webp".to_string(),
                "bmp" => return "image/bmp".to_string(),
                
                // Audio
                "mp3" => return "audio/mpeg".to_string(),
                "wav" => return "audio/wav".to_string(),
                "ogg" => return "audio/ogg".to_string(),
                "flac" => return "audio/flac".to_string(),
                "aac" => return "audio/aac".to_string(),
                
                // Video
                "mp4" => return "video/mp4".to_string(),
                "webm" => return "video/webm".to_string(),
                "mkv" => return "video/x-matroska".to_string(),
                "avi" => return "video/x-msvideo".to_string(),
                
                // Documents
                "pdf" => return "application/pdf".to_string(),
                "doc" => return "application/msword".to_string(),
                "docx" => return "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string(),
                "txt" => return "text/plain".to_string(),
                "rtf" => return "application/rtf".to_string(),
                
                _ => {}
            }
        }
    }

    // Unknown type
    "application/octet-stream".to_string()
}

/// Validate media against channel capabilities.
///
/// This function checks whether a given media type is supported by the
/// channel based on its capabilities. It verifies that:
///
/// 1. The channel supports media at all (`supports_media` is true)
/// 2. The media type is in the list of supported media types
///
/// # Arguments
///
/// * `media_type` - The media type to validate
/// * `capabilities` - The channel capabilities to validate against
///
/// # Returns
///
/// Returns `Ok(())` if the media is valid, or an error describing why
/// validation failed.
///
/// # Errors
///
/// Returns an error if:
/// - The channel doesn't support media
/// - The media type is not in the supported list
pub fn validate_media(
    media_type: &MediaType,
    capabilities: &crate::types::ChannelCapabilities,
) -> Result<()> {
    // Check if the channel supports media at all
    if !capabilities.supports_media {
        return Err(anyhow::anyhow!(
            "Channel does not support media transmission"
        ));
    }

    // Check if the media type is supported
    if !capabilities.supported_media_types.contains(media_type) {
        return Err(anyhow::anyhow!(
            "Media type {:?} is not supported by this channel",
            media_type
        ));
    }

    Ok(())
}

/// Detect the image format from raw bytes.
///
/// This is an internal helper function used to preserve the original format
/// when resizing or converting images.
///
/// # Arguments
///
/// * `data` - The raw image bytes
///
/// # Returns
///
/// Returns the detected `ImageFormat`, or an error if the format cannot be detected.
fn detect_image_format(data: &[u8]) -> Result<ImageFormat> {
    if data.len() >= 4 {
        // PNG: 89 50 4E 47
        if data[0] == 0x89 && data[1] == b'P' && data[2] == b'N' && data[3] == b'G' {
            return Ok(ImageFormat::Png);
        }

        // GIF: 47 49 46
        if data[0] == b'G' && data[1] == b'I' && data[2] == b'F' {
            return Ok(ImageFormat::Gif);
        }

        // JPEG: FF D8 FF
        if data[0] == 0xFF && data[1] == 0xD8 && data[2] == 0xFF {
            return Ok(ImageFormat::Jpeg);
        }
    }

    // Try to detect using the image crate's built-in detection
    if let Ok(img) = image::guess_format(data) {
        return Ok(img);
    }

    Err(anyhow::anyhow!("Could not detect image format"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_media_type_jpeg() {
        // JPEG magic bytes: FF D8 FF
        let data = [0xFF, 0xD8, 0xFF, 0x00];
        let media_type = detect_media_type(&data, None);
        assert_eq!(media_type, MediaType::Image);
    }

    #[test]
    fn test_detect_media_type_png() {
        // PNG magic bytes: 89 50 4E 47
        let data = [0x89, b'P' as u8, b'N' as u8, b'G' as u8, 0x00];
        let media_type = detect_media_type(&data, None);
        assert_eq!(media_type, MediaType::Image);
    }

    #[test]
    fn test_detect_media_type_gif() {
        // GIF magic bytes: 47 49 46
        let data = [b'G', b'I', b'F', 0x00];
        let media_type = detect_media_type(&data, None);
        assert_eq!(media_type, MediaType::Image);
    }

    #[test]
    fn test_detect_media_type_pdf() {
        // PDF magic bytes: 25 50 44 46
        let data = [b'%', b'P', b'D', b'F', 0x00];
        let media_type = detect_media_type(&data, None);
        assert_eq!(media_type, MediaType::Document);
    }

    #[test]
    fn test_detect_media_type_by_extension() {
        // Test extension-based detection
        let data = [0x00, 0x00, 0x00, 0x00]; // Unknown magic bytes
        let media_type = detect_media_type(&data, Some("photo.jpg"));
        assert_eq!(media_type, MediaType::Image);
    }

    #[test]
    fn test_detect_media_type_audio() {
        let data = [0x00, 0x00, 0x00, 0x00];
        let media_type = detect_media_type(&data, Some("music.mp3"));
        assert_eq!(media_type, MediaType::Audio);
    }

    #[test]
    fn test_detect_media_type_video() {
        let data = [0x00, 0x00, 0x00, 0x00];
        let media_type = detect_media_type(&data, Some("video.mp4"));
        assert_eq!(media_type, MediaType::Video);
    }

    #[test]
    fn test_detect_media_type_other() {
        let data = [0x00, 0x00, 0x00, 0x00];
        let media_type = detect_media_type(&data, Some("file.xyz"));
        assert_eq!(media_type, MediaType::Other("unknown".to_string()));
    }

    #[test]
    fn test_detect_mime_type_jpeg() {
        let data = [0xFF, 0xD8, 0xFF, 0x00];
        let mime_type = detect_mime_type(&data, None);
        assert_eq!(mime_type, "image/jpeg");
    }

    #[test]
    fn test_detect_mime_type_png() {
        let data = [0x89, b'P' as u8, b'N' as u8, b'G' as u8, 0x00];
        let mime_type = detect_mime_type(&data, None);
        assert_eq!(mime_type, "image/png");
    }

    #[test]
    fn test_resize_image() {
        // Create a simple 1000x1000 PNG image using the image crate
        use image::{ImageBuffer, Rgba};
        use std::io::Cursor;

        // Create a 1000x1000 image filled with white
        let img_buffer: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(1000, 1000);

        // Encode to PNG bytes using DynamicImage
        let dynamic_img = DynamicImage::ImageRgba8(img_buffer);
        let mut buf = Cursor::new(Vec::new());
        dynamic_img.write_to(&mut buf, ImageFormat::Png).unwrap();
        let data = buf.into_inner();

        // Resize to max 500x500
        let resized = resize_image(&data, 500, 500).unwrap();

        // Verify the result is a valid PNG
        let img = image::load_from_memory(&resized).unwrap();
        let (width, height) = img.dimensions();
        assert!(width <= 500);
        assert!(height <= 500);
    }

    #[test]
    fn test_validate_media_success() {
        let media_type = MediaType::Image;
        let capabilities = crate::types::ChannelCapabilities {
            supports_media: true,
            supported_media_types: vec![MediaType::Image, MediaType::Document],
            ..Default::default()
        };

        assert!(validate_media(&media_type, &capabilities).is_ok());
    }

    #[test]
    fn test_validate_media_not_supported() {
        let media_type = MediaType::Video;
        let capabilities = crate::types::ChannelCapabilities {
            supports_media: true,
            supported_media_types: vec![MediaType::Image, MediaType::Document],
            ..Default::default()
        };

        assert!(validate_media(&media_type, &capabilities).is_err());
    }

    #[test]
    fn test_validate_media_disabled() {
        let media_type = MediaType::Image;
        let capabilities = crate::types::ChannelCapabilities {
            supports_media: false,
            supported_media_types: vec![],
            ..Default::default()
        };

        assert!(validate_media(&media_type, &capabilities).is_err());
    }
}
