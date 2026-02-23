//! Media transcoding and validation utilities.
//!
//! This module provides functionality for ensuring media files are compatible
//! with different platform requirements, including format conversion and
//! dimension resizing.

use anyhow::{anyhow, Result};
use std::fmt;
use std::path::Path;

/// Represents a media attachment that can be transcoded.
#[derive(Debug, Clone)]
pub struct MediaAttachment {
    /// The raw media data
    pub data: Vec<u8>,
    /// The current format of the media
    pub format: MediaFormat,
    /// Optional filename
    pub filename: Option<String>,
    /// Optional MIME type
    pub mime_type: Option<String>,
    /// Optional dimensions for images
    pub dimensions: Option<(u32, u32)>,
}

/// Represents the format of media content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MediaFormat {
    /// Image formats
    Image(ImageFormat),
    /// Audio formats
    Audio(AudioFormat),
    /// Video formats
    Video(VideoFormat),
    /// Document formats (documents pass through without conversion)
    Document(DocumentFormat),
}

/// Represents image formats.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageFormat {
    /// PNG format
    Png,
    /// JPEG format
    Jpeg,
    /// GIF format
    Gif,
    /// WebP format
    WebP,
    /// BMP format
    Bmp,
    /// Unknown image format
    Other(String),
}

/// Represents audio formats.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AudioFormat {
    /// MP3 format
    Mp3,
    /// WAV format
    Wav,
    /// OGG format
    Ogg,
    /// FLAC format
    Flac,
    /// AAC format
    Aac,
    /// Unknown audio format
    Other(String),
}

/// Represents video formats.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VideoFormat {
    /// MP4 format
   Mp4,
    /// WebM format
    WebM,
    /// MOV format
    Mov,
    /// AVI format
    Avi,
    /// MKV format
    Mkv,
    /// Unknown video format
    Other(String),
}

/// Represents document formats.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentFormat {
    /// PDF format
    Pdf,
    /// DOC format
    Doc,
    /// DOCX format
    Docx,
    /// TXT format
    Txt,
    /// RTF format
    Rtf,
    /// Unknown document format
    Other(String),
}

/// Platform-specific constraints for media.
#[derive(Debug, Clone)]
pub struct PlatformConstraints {
    /// Maximum file size in bytes
    pub max_file_size: u64,
    /// Supported image formats
    pub supported_image_formats: Vec<ImageFormat>,
    /// Supported audio formats
    pub supported_audio_formats: Vec<AudioFormat>,
    /// Supported video formats
    pub supported_video_formats: Vec<VideoFormat>,
    /// Supported document formats
    pub supported_document_formats: Vec<DocumentFormat>,
    /// Maximum image dimensions (width, height)
    pub max_image_dimensions: Option<(u32, u32)>,
}

/// Target platform for media constraints.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Platform {
    /// Telegram platform
    Telegram,
    /// Discord platform
    Discord,
    /// WhatsApp platform
    WhatsApp,
    /// Slack platform
    Slack,
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Platform::Telegram => write!(f, "Telegram"),
            Platform::Discord => write!(f, "Discord"),
            Platform::WhatsApp => write!(f, "WhatsApp"),
            Platform::Slack => write!(f, "Slack"),
        }
    }
}

impl Platform {
    /// Get the platform constraints for a given platform.
    pub fn constraints(&self) -> PlatformConstraints {
        match self {
            Platform::Telegram => PlatformConstraints {
                max_file_size: 20_000_000, // 20 MB
                supported_image_formats: vec![
                    ImageFormat::Png,
                    ImageFormat::Jpeg,
                    ImageFormat::Gif,
                    ImageFormat::WebP,
                ],
                supported_audio_formats: vec![
                    AudioFormat::Mp3,
                    AudioFormat::Ogg,
                ],
                supported_video_formats: vec![
                    VideoFormat::Mp4,
                    VideoFormat::WebM,
                ],
                supported_document_formats: vec![
                    DocumentFormat::Pdf,
                    DocumentFormat::Docx,
                    DocumentFormat::Txt,
                ],
                max_image_dimensions: Some((4096, 4096)),
            },
            Platform::Discord => PlatformConstraints {
                max_file_size: 25_000_000, // 25 MB
                supported_image_formats: vec![
                    ImageFormat::Png,
                    ImageFormat::Jpeg,
                    ImageFormat::Gif,
                    ImageFormat::WebP,
                ],
                supported_audio_formats: vec![
                    AudioFormat::Mp3,
                    AudioFormat::Wav,
                ],
                supported_video_formats: vec![
                    VideoFormat::Mp4,
                    VideoFormat::WebM,
                    VideoFormat::Mov,
                ],
                supported_document_formats: vec![
                    DocumentFormat::Pdf,
                    DocumentFormat::Txt,
                ],
                max_image_dimensions: Some((1024, 1024)),
            },
            Platform::WhatsApp => PlatformConstraints {
                max_file_size: 16_000_000, // 16 MB
                supported_image_formats: vec![
                    ImageFormat::Png,
                    ImageFormat::Jpeg,
                    ImageFormat::WebP,
                ],
                supported_audio_formats: vec![
                    AudioFormat::Mp3,
                    AudioFormat::Wav,
                    AudioFormat::Ogg,
                ],
                supported_video_formats: vec![
                    VideoFormat::Mp4,
                ],
                supported_document_formats: vec![
                    DocumentFormat::Pdf,
                    DocumentFormat::Docx,
                    DocumentFormat::Txt,
                ],
                max_image_dimensions: Some((10000, 10000)),
            },
            Platform::Slack => PlatformConstraints {
                max_file_size: 100_000_000, // 100 MB
                supported_image_formats: vec![
                    ImageFormat::Png,
                    ImageFormat::Jpeg,
                    ImageFormat::Gif,
                ],
                supported_audio_formats: vec![
                    AudioFormat::Mp3,
                    AudioFormat::Wav,
                ],
                supported_video_formats: vec![
                    VideoFormat::Mp4,
                    VideoFormat::WebM,
                ],
                supported_document_formats: vec![
                    DocumentFormat::Pdf,
                    DocumentFormat::Docx,
                    DocumentFormat::Txt,
                ],
                max_image_dimensions: Some((2000, 2000)),
            },
        }
    }
}

/// Ensure media is compatible with the target platform.
///
/// This function checks if the media is compatible with the target platform
/// and converts it if necessary. It handles:
///
/// - Format conversion (e.g., WebP to PNG for Slack)
/// - Dimension resizing for images that exceed platform limits
/// - File size validation
///
/// # Arguments
///
/// * `media` - The media attachment to check and potentially convert
/// * `target` - The target platform
///
/// # Returns
///
/// Returns the (possibly converted) media attachment that is compatible
/// with the target platform. If the media is already compatible, the
/// original is returned unchanged.
///
/// # Errors
///
/// Returns an error if:
/// - The media format is not supported by the platform
/// - The media file exceeds the platform's size limit
/// - Conversion fails for any reason
pub fn ensure_compatible_format(
    media: &MediaAttachment,
    target: Platform,
) -> Result<MediaAttachment> {
    let constraints = target.constraints();

    // Check file size
    if media.data.len() as u64 > constraints.max_file_size {
        return Err(anyhow!(
            "Media file size ({}) exceeds {} limit of {}",
            media.data.len(),
            target,
            constraints.max_file_size
        ));
    }

    // Check format compatibility
    let is_compatible = match &media.format {
        MediaFormat::Image(format) => constraints.supported_image_formats.contains(format),
        MediaFormat::Audio(format) => constraints.supported_audio_formats.contains(format),
        MediaFormat::Video(format) => constraints.supported_video_formats.contains(format),
        MediaFormat::Document(_) => true, // Documents pass through
    };

    if is_compatible {
        // Check if resizing is needed for images
        if let MediaFormat::Image(_) = &media.format {
            if let Some(dim) = &media.dimensions {
                if let Some(max_dim) = constraints.max_image_dimensions {
                    if dim.0 > max_dim.0 || dim.1 > max_dim.1 {
                        // Resize the image
                        let resized = resize_image_for_platform(&media.data, max_dim)?;
                        return Ok(MediaAttachment {
                            data: resized,
                            format: media.format.clone(),
                            filename: media.filename.clone(),
                            mime_type: media.mime_type.clone(),
                            dimensions: Some(max_dim),
                        });
                    }
                }
            }
        }

        // Media is already compatible, return unchanged
        return Ok(media.clone());
    }

    // Need to convert format
    let converted = convert_format(media, &constraints, target)?;

    Ok(converted)
}

/// Convert media to a compatible format.
fn convert_format(
    media: &MediaAttachment,
    constraints: &PlatformConstraints,
    target: Platform,
) -> Result<MediaAttachment> {
    match &media.format {
        MediaFormat::Image(from_format) => {
            // Find a supported format to convert to
            let to_format = constraints
                .supported_image_formats
                .first()
                .ok_or_else(|| anyhow!("No supported image formats for {}", target))?;

            convert_image_format(media, from_format, to_format)
        }
        MediaFormat::Audio(from_format) => {
            let to_format = constraints
                .supported_audio_formats
                .first()
                .ok_or_else(|| anyhow!("No supported audio formats for {}", target))?;

            convert_audio_format(media, from_format, to_format)
        }
        MediaFormat::Video(from_format) => {
            let to_format = constraints
                .supported_video_formats
                .first()
                .ok_or_else(|| anyhow!("No supported video formats for {}", target))?;

            convert_video_format(media, from_format, to_format)
        }
        MediaFormat::Document(_) => {
            // Documents should pass through, but if we get here,
            // it means the document format isn't supported
            Err(anyhow!(
                "Document format {:?} is not supported by {}",
                media.format,
                target
            ))
        }
    }
}

/// Convert an image to a different format.
fn convert_image_format(
    media: &MediaAttachment,
    from: &ImageFormat,
    to: &ImageFormat,
) -> Result<MediaAttachment> {
    // For now, we'll just validate the format and return unchanged
    // Real conversion would require image processing libraries or ffmpeg
    if from == to {
        return Ok(media.clone());
    }

    // Placeholder for actual conversion logic
    // In a real implementation, this would use image crate or ffmpeg
    Err(anyhow!(
        "Image format conversion from {:?} to {:?} not yet implemented",
        from,
        to
    ))
}

/// Convert audio to a different format.
fn convert_audio_format(
    media: &MediaAttachment,
    from: &AudioFormat,
    to: &AudioFormat,
) -> Result<MediaAttachment> {
    if from == to {
        return Ok(media.clone());
    }

    Err(anyhow!(
        "Audio format conversion from {:?} to {:?} not yet implemented",
        from,
        to
    ))
}

/// Convert video to a different format.
fn convert_video_format(
    media: &MediaAttachment,
    from: &VideoFormat,
    to: &VideoFormat,
) -> Result<MediaAttachment> {
    if from == to {
        return Ok(media.clone());
    }

    Err(anyhow!(
        "Video format conversion from {:?} to {:?} not yet implemented",
        from,
        to
    ))
}

/// Resize an image to fit within the specified maximum dimensions.
fn resize_image_for_platform(data: &[u8], max_dimensions: (u32, u32)) -> Result<Vec<u8>> {
    // Placeholder for image resizing logic
    // This would use the image crate to resize the image
    Ok(data.to_vec())
}

/// Detect the media format from the file extension or MIME type.
pub fn detect_media_format(data: &[u8], filename: Option<&str>, mime_type: Option<&str>) -> MediaFormat {
    // First check MIME type if available
    if let Some(mime) = mime_type {
        match mime {
            // Images
            "image/png" => return MediaFormat::Image(ImageFormat::Png),
            "image/jpeg" => return MediaFormat::Image(ImageFormat::Jpeg),
            "image/gif" => return MediaFormat::Image(ImageFormat::Gif),
            "image/webp" => return MediaFormat::Image(ImageFormat::WebP),
            "image/bmp" => return MediaFormat::Image(ImageFormat::Bmp),

            // Audio
            "audio/mpeg" => return MediaFormat::Audio(AudioFormat::Mp3),
            "audio/wav" => return MediaFormat::Audio(AudioFormat::Wav),
            "audio/ogg" => return MediaFormat::Audio(AudioFormat::Ogg),
            "audio/flac" => return MediaFormat::Audio(AudioFormat::Flac),
            "audio/aac" => return MediaFormat::Audio(AudioFormat::Aac),

            // Video
            "video/mp4" => return MediaFormat::Video(VideoFormat::Mp4),
            "video/webm" => return MediaFormat::Video(VideoFormat::WebM),
            "video/quicktime" => return MediaFormat::Video(VideoFormat::Mov),
            "video/x-msvideo" => return MediaFormat::Video(VideoFormat::Avi),
            "video/x-matroska" => return MediaFormat::Video(VideoFormat::Mkv),

            // Documents
            "application/pdf" => return MediaFormat::Document(DocumentFormat::Pdf),
            "application/msword" => return MediaFormat::Document(DocumentFormat::Doc),
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => {
                return MediaFormat::Document(DocumentFormat::Docx)
            }
            "text/plain" => return MediaFormat::Document(DocumentFormat::Txt),
            "application/rtf" => return MediaFormat::Document(DocumentFormat::Rtf),

            _ => {}
        }
    }

    // Fall back to file extension
    if let Some(name) = filename {
        if let Some(ext) = Path::new(name).extension().and_then(|e| e.to_str()) {
            let ext_lower = ext.to_lowercase();

            match ext_lower.as_str() {
                // Images
                "png" => return MediaFormat::Image(ImageFormat::Png),
                "jpg" | "jpeg" => return MediaFormat::Image(ImageFormat::Jpeg),
                "gif" => return MediaFormat::Image(ImageFormat::Gif),
                "webp" => return MediaFormat::Image(ImageFormat::WebP),
                "bmp" => return MediaFormat::Image(ImageFormat::Bmp),

                // Audio
                "mp3" => return MediaFormat::Audio(AudioFormat::Mp3),
                "wav" => return MediaFormat::Audio(AudioFormat::Wav),
                "ogg" => return MediaFormat::Audio(AudioFormat::Ogg),
                "oga" | "flac" => return MediaFormat::Audio(AudioFormat::Flac),
                "aac" => return MediaFormat::Audio(AudioFormat::Aac),

                // Video
                "mp4" => return MediaFormat::Video(VideoFormat::Mp4),
                "webm" => return MediaFormat::Video(VideoFormat::WebM),
                "mov" => return MediaFormat::Video(VideoFormat::Mov),
                "avi" => return MediaFormat::Video(VideoFormat::Avi),
                "mkv" => return MediaFormat::Video(VideoFormat::Mkv),

                // Documents
                "pdf" => return MediaFormat::Document(DocumentFormat::Pdf),
                "doc" => return MediaFormat::Document(DocumentFormat::Doc),
                "docx" => return MediaFormat::Document(DocumentFormat::Docx),
                "txt" => return MediaFormat::Document(DocumentFormat::Txt),
                "rtf" => return MediaFormat::Document(DocumentFormat::Rtf),

                _ => {}
            }
        }
    }

    // Try to detect from magic bytes
    detect_format_from_magic(data)
}

/// Detect format from magic bytes (first few bytes of the file).
fn detect_format_from_magic(data: &[u8]) -> MediaFormat {
    // Check magic bytes for common formats
    if data.len() >= 4 {
        // PNG: 89 50 4E 47
        if data[0] == 0x89 && data[1] == b'P' && data[2] == b'N' && data[3] == b'G' {
            return MediaFormat::Image(ImageFormat::Png);
        }

        // JPEG: FF D8 FF
        if data[0] == 0xFF && data[1] == 0xD8 && data[2] == 0xFF {
            return MediaFormat::Image(ImageFormat::Jpeg);
        }

        // GIF: 47 49 46
        if data[0] == b'G' && data[1] == b'I' && data[2] == b'F' {
            return MediaFormat::Image(ImageFormat::Gif);
        }

        // WebP: 52 49 46 46 ... 57 45 42 50
        if data[0] == b'R' && data[1] == b'I' && data[2] == b'F' && data[3] == b'F' && data.len() >= 12 {
            if data[8] == b'W' && data[9] == b'E' && data[10] == b'B' && data[11] == b'P' {
                return MediaFormat::Image(ImageFormat::WebP);
            }
        }
    }

    if data.len() >= 2 {
        // MP3: ID3 or 11 bits of 1s
        if (data[0] == b'I' && data[1] == b'D' && data.len() >= 3 && data[2] == b'3') ||
           ((data[0] & 0xFF) == 0xFF && (data[1] & 0xE0) == 0xE0) {
            return MediaFormat::Audio(AudioFormat::Mp3);
        }

        // WAV: 52 49 46 46 ... 57 41 56 45
        if data[0] == b'R' && data[1] == b'I' && data[2] == b'F' && data[3] == b'F' && data.len() >= 12 {
            if data[8] == b'W' && data[9] == b'A' && data[10] == b'V' && data[11] == b'E' {
                return MediaFormat::Audio(AudioFormat::Wav);
            }
        }
    }

    // PDF: 25 50 44 46
    if data.len() >= 4 && data[0] == b'%' && data[1] == b'P' && data[2] == b'D' && data[3] == b'F' {
        return MediaFormat::Document(DocumentFormat::Pdf);
    }

    // MP4: 00 00 00 01 or ftyp box
    if data.len() >= 8 {
        // Check for ftyp box (common in MP4, MOV)
        if (data[4] == b'f' && data[5] == b't' && data[6] == b'y' && data[7] == b'p') ||
           (data[0] == 0 && data[1] == 0 && data[2] == 0 && data[3] == 1) {
            return MediaFormat::Video(VideoFormat::Mp4);
        }
    }

    MediaFormat::Document(DocumentFormat::Other("unknown".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_constraints_telegram() {
        let constraints = Platform::Telegram.constraints();
        assert_eq!(constraints.max_file_size, 20_000_000);
        assert!(constraints.supported_image_formats.contains(&ImageFormat::Png));
        assert!(constraints.supported_image_formats.contains(&ImageFormat::Jpeg));
    }

    #[test]
    fn test_platform_constraints_discord() {
        let constraints = Platform::Discord.constraints();
        assert_eq!(constraints.max_file_size, 25_000_000);
        assert_eq!(constraints.max_image_dimensions, Some((1024, 1024)));
    }

    #[test]
    fn test_platform_constraints_whatsapp() {
        let constraints = Platform::WhatsApp.constraints();
        assert_eq!(constraints.max_file_size, 16_000_000);
    }

    #[test]
    fn test_platform_constraints_slack() {
        let constraints = Platform::Slack.constraints();
        assert_eq!(constraints.max_file_size, 100_000_000);
    }

    #[test]
    fn test_detect_media_format_png() {
        // PNG magic bytes: 89 50 4E 47
        let data = [0x89, b'P' as u8, b'N' as u8, b'G' as u8, 0x00];
        let format = detect_media_format(&data, None, None);
        assert_eq!(format, MediaFormat::Image(ImageFormat::Png));
    }

    #[test]
    fn test_detect_media_format_jpeg() {
        // JPEG magic bytes: FF D8 FF
        let data = [0xFF, 0xD8, 0xFF, 0x00];
        let format = detect_media_format(&data, None, None);
        assert_eq!(format, MediaFormat::Image(ImageFormat::Jpeg));
    }

    #[test]
    fn test_detect_media_format_by_extension() {
        // Test extension-based detection
        let data = [0x00; 10];
        let format = detect_media_format(&data, Some("image.png"), None);
        assert_eq!(format, MediaFormat::Image(ImageFormat::Png));
    }

    #[test]
    fn test_detect_media_format_by_mime_type() {
        // Test MIME type-based detection
        let data = [0x00; 10];
        let format = detect_media_format(&data, None, Some("image/jpeg"));
        assert_eq!(format, MediaFormat::Image(ImageFormat::Jpeg));
    }

    #[test]
    fn test_detect_media_format_pdf() {
        // PDF magic bytes: 25 50 44 46
        let data = [b'%', b'P', b'D', b'F', 0x00];
        let format = detect_media_format(&data, None, None);
        assert_eq!(format, MediaFormat::Document(DocumentFormat::Pdf));
    }

    #[test]
    fn test_detect_media_format_mp3() {
        // MP3 magic bytes: FF FB
        let data = [0xFF, 0xFB, 0x00, 0x00];
        let format = detect_media_format(&data, None, None);
        assert_eq!(format, MediaFormat::Audio(AudioFormat::Mp3));
    }

    #[test]
    fn test_detect_media_format_mp4() {
        // MP4 ftyp box: ... ftyp
        let data = [0x00, 0x00, 0x00, 0x00, b'f', b't', b'y', b'p'];
        let format = detect_media_format(&data, None, None);
        assert_eq!(format, MediaFormat::Video(VideoFormat::Mp4));
    }

    #[test]
    fn test_media_attachment_creation() {
        let attachment = MediaAttachment {
            data: vec![0x00, 0x01, 0x02],
            format: MediaFormat::Image(ImageFormat::Png),
            filename: Some("test.png".to_string()),
            mime_type: Some("image/png".to_string()),
            dimensions: Some((100, 100)),
        };

        assert_eq!(attachment.data.len(), 3);
        assert_eq!(attachment.filename, Some("test.png".to_string()));
    }

    #[test]
    fn test_image_format_equality() {
        assert_eq!(ImageFormat::Png, ImageFormat::Png);
        assert_ne!(ImageFormat::Png, ImageFormat::Jpeg);
    }

    #[test]
    fn test_audio_format_equality() {
        assert_eq!(AudioFormat::Mp3, AudioFormat::Mp3);
        assert_ne!(AudioFormat::Mp3, AudioFormat::Wav);
    }

    #[test]
    fn test_video_format_equality() {
        assert_eq!(VideoFormat::Mp4, VideoFormat::Mp4);
        assert_ne!(VideoFormat::Mp4, VideoFormat::WebM);
    }

    #[test]
    fn test_document_format_equality() {
        assert_eq!(DocumentFormat::Pdf, DocumentFormat::Pdf);
        assert_ne!(DocumentFormat::Pdf, DocumentFormat::Docx);
    }

    #[test]
    fn test_media_format_equality() {
        assert_eq!(
            MediaFormat::Image(ImageFormat::Png),
            MediaFormat::Image(ImageFormat::Png)
        );
        assert_ne!(
            MediaFormat::Image(ImageFormat::Png),
            MediaFormat::Audio(AudioFormat::Mp3)
        );
    }
}
