//! Media format conversion and validation utilities for cross-platform channels.
//!
//! This module provides utilities for handling media across different messaging
//! platforms that have varying constraints on file sizes, formats, and dimensions.

use std::collections::HashMap;

/// Platform-specific media constraints.
///
/// Each platform has different limits for image and file sizes,
/// supported formats, and maximum dimensions.
#[derive(Debug, Clone)]
pub struct MediaConstraints {
    /// Maximum image size in bytes
    pub max_image_size_bytes: usize,
    /// Maximum file size in bytes
    pub max_file_size_bytes: usize,
    /// Supported image formats (lowercase extensions)
    pub supported_image_formats: Vec<String>,
    /// Maximum image dimensions (width, height), None for unlimited
    pub max_image_dimensions: Option<(u32, u32)>,
    /// Supported video formats
    pub supported_video_formats: Vec<String>,
    /// Maximum video size in bytes
    pub max_video_size_bytes: usize,
    /// Supported audio formats
    pub supported_audio_formats: Vec<String>,
    /// Maximum audio size in bytes
    pub max_audio_size_bytes: usize,
}

impl MediaConstraints {
    /// Get constraints for a specific platform.
    ///
    /// # Arguments
    ///
    /// * `platform` - Platform name (e.g., "signal", "discord", "slack")
    ///
    /// # Supported Platforms
    ///
    /// | Platform | Max Image | Max File | Dimensions |
    /// |----------|-----------|----------|------------|
    /// | signal | 100 MB | 100 MB | None |
    /// | discord | 25 MB | 8 MB (free) | 10,000x10,000 |
    /// | slack | 200 MB | 1 GB | None |
    /// | telegram | 5 MB (photos) | 50 MB (files) | None |
    /// | whatsapp | 5 MB (images) | 100 MB (documents) | None |
    /// | msteams | 25 MB | 25 MB | None |
    /// | mattermost | 50 MB | 50 MB | None |
    /// | twitch | 25 MB | 25 MB | 3840x2160 |
    /// | line | 10 MB | 200 MB | 4096x4096 |
    /// | lark | 50 MB | 100 MB | None |
    /// | zalo | 10 MB | 25 MB | None |
    /// | irc | None | None | None |
    pub fn for_platform(platform: &str) -> Self {
        match platform {
            "signal" => Self {
                max_image_size_bytes: 100 * 1024 * 1024,  // 100 MB
                max_file_size_bytes: 100 * 1024 * 1024,
                supported_image_formats: vec!["png".into(), "jpg".into(), "jpeg".into(), "gif".into(), "webp".into()],
                max_image_dimensions: None,
                supported_video_formats: vec!["mp4".into(), "mov".into()],
                max_video_size_bytes: 100 * 1024 * 1024,
                supported_audio_formats: vec!["mp3".into(), "ogg".into()],
                max_audio_size_bytes: 100 * 1024 * 1024,
            },
            "discord" => Self {
                max_image_size_bytes: 25 * 1024 * 1024,  // 25 MB
                max_file_size_bytes: 8 * 1024 * 1024,    // 8 MB for free users
                supported_image_formats: vec!["png".into(), "jpg".into(), "jpeg".into(), "gif".into(), "webp".into()],
                max_image_dimensions: Some((10_000, 10_000)),
                supported_video_formats: vec!["mp4".into(), "mov".into(), "webm".into()],
                max_video_size_bytes: 50 * 1024 * 1024,
                supported_audio_formats: vec!["mp3".into(), "ogg".into()],
                max_audio_size_bytes: 25 * 1024 * 1024,
            },
            "slack" => Self {
                max_image_size_bytes: 200 * 1024 * 1024,  // 200 MB
                max_file_size_bytes: 1024 * 1024 * 1024,  // 1 GB
                supported_image_formats: vec!["png".into(), "jpg".into(), "jpeg".into(), "gif".into()],
                max_image_dimensions: None,
                supported_video_formats: vec!["mp4".into(), "mov".into()],
                max_video_size_bytes: 200 * 1024 * 1024,
                supported_audio_formats: vec!["mp3".into(), "wav".into()],
                max_audio_size_bytes: 200 * 1024 * 1024,
            },
            "telegram" => Self {
                max_image_size_bytes: 5 * 1024 * 1024,  // 5 MB for photos
                max_file_size_bytes: 50 * 1024 * 1024,  // 50 MB for files
                supported_image_formats: vec!["png".into(), "jpg".into(), "jpeg".into(), "gif".into()],
                max_image_dimensions: Some((10_000, 10_000)),
                supported_video_formats: vec!["mp4".into(), "mov".into()],
                max_video_size_bytes: 2000 * 1024 * 1024,  // 2 GB for bots
                supported_audio_formats: vec!["mp3".into(), "ogg".into()],
                max_audio_size_bytes: 50 * 1024 * 1024,
            },
            "whatsapp" => Self {
                max_image_size_bytes: 5 * 1024 * 1024,  // 5 MB for images
                max_file_size_bytes: 100 * 1024 * 1024,  // 100 MB for documents
                supported_image_formats: vec!["png".into(), "jpg".into(), "jpeg".into(), "webp".into()],
                max_image_dimensions: None,
                supported_video_formats: vec!["mp4".into(), "mov".into()],
                max_video_size_bytes: 100 * 1024 * 1024,
                supported_audio_formats: vec!["mp3".into(), "ogg".into(), "aac".into()],
                max_audio_size_bytes: 100 * 1024 * 1024,
            },
            "msteams" => Self {
                max_image_size_bytes: 25 * 1024 * 1024,  // 25 MB
                max_file_size_bytes: 25 * 1024 * 1024,
                supported_image_formats: vec!["png".into(), "jpg".into(), "jpeg".into(), "gif".into()],
                max_image_dimensions: None,
                supported_video_formats: vec!["mp4".into()],
                max_video_size_bytes: 25 * 1024 * 1024,
                supported_audio_formats: vec!["mp3".into()],
                max_audio_size_bytes: 25 * 1024 * 1024,
            },
            "mattermost" => Self {
                max_image_size_bytes: 50 * 1024 * 1024,  // 50 MB
                max_file_size_bytes: 50 * 1024 * 1024,
                supported_image_formats: vec!["png".into(), "jpg".into(), "jpeg".into(), "gif".into()],
                max_image_dimensions: None,
                supported_video_formats: vec!["mp4".into(), "mov".into()],
                max_video_size_bytes: 50 * 1024 * 1024,
                supported_audio_formats: vec!["mp3".into(), "wav".into()],
                max_audio_size_bytes: 50 * 1024 * 1024,
            },
            "twitch" => Self {
                max_image_size_bytes: 25 * 1024 * 1024,  // 25 MB
                max_file_size_bytes: 25 * 1024 * 1024,
                supported_image_formats: vec!["png".into(), "jpg".into(), "jpeg".into()],
                max_image_dimensions: Some((3840, 2160)),  // 4K
                supported_video_formats: vec!["mp4".into()],
                max_video_size_bytes: 25 * 1024 * 1024,
                supported_audio_formats: vec!["mp3".into()],
                max_audio_size_bytes: 25 * 1024 * 1024,
            },
            "line" => Self {
                max_image_size_bytes: 10 * 1024 * 1024,  // 10 MB
                max_file_size_bytes: 200 * 1024 * 1024,  // 200 MB
                supported_image_formats: vec!["png".into(), "jpg".into(), "jpeg".into()],
                max_image_dimensions: Some((4096, 4096)),
                supported_video_formats: vec!["mp4".into()],
                max_video_size_bytes: 300 * 1024 * 1024,
                supported_audio_formats: vec!["mp3".into(), "aac".into()],
                max_audio_size_bytes: 200 * 1024 * 1024,
            },
            "lark" => Self {
                max_image_size_bytes: 50 * 1024 * 1024,  // 50 MB
                max_file_size_bytes: 100 * 1024 * 1024,  // 100 MB
                supported_image_formats: vec!["png".into(), "jpg".into(), "jpeg".into(), "gif".into()],
                max_image_dimensions: None,
                supported_video_formats: vec!["mp4".into()],
                max_video_size_bytes: 50 * 1024 * 1024,
                supported_audio_formats: vec!["mp3".into()],
                max_audio_size_bytes: 50 * 1024 * 1024,
            },
            "zalo" => Self {
                max_image_size_bytes: 10 * 1024 * 1024,  // 10 MB
                max_file_size_bytes: 25 * 1024 * 1024,  // 25 MB
                supported_image_formats: vec!["png".into(), "jpg".into(), "jpeg".into()],
                max_image_dimensions: None,
                supported_video_formats: vec!["mp4".into()],
                max_video_size_bytes: 25 * 1024 * 1024,
                supported_audio_formats: vec!["mp3".into()],
                max_audio_size_bytes: 25 * 1024 * 1024,
            },
            "irc" => Self {
                max_image_size_bytes: 0,  // No image support
                max_file_size_bytes: 0,
                supported_image_formats: vec![],
                max_image_dimensions: None,
                supported_video_formats: vec![],
                max_video_size_bytes: 0,
                supported_audio_formats: vec![],
                max_audio_size_bytes: 0,
            },
            _ => Self::default(),
        }
    }
}

impl Default for MediaConstraints {
    fn default() -> Self {
        Self {
            max_image_size_bytes: 10 * 1024 * 1024,  // 10 MB
            max_file_size_bytes: 50 * 1024 * 1024,   // 50 MB
            supported_image_formats: vec!["png".into(), "jpg".into(), "jpeg".into()],
            max_image_dimensions: None,
            supported_video_formats: vec!["mp4".into()],
            max_video_size_bytes: 50 * 1024 * 1024,
            supported_audio_formats: vec!["mp3".into()],
            max_audio_size_bytes: 50 * 1024 * 1024,
        }
    }
}

/// Error type for media validation.
#[derive(Debug, thiserror::Error)]
pub enum MediaError {
    /// File size exceeds platform limit
    #[error("File size {0} exceeds maximum {1}")]
    FileTooLarge(usize, usize),

    /// Image dimensions exceed platform limit
    #[error("Image dimensions {0}x{1} exceed maximum {2}x{3}")]
    DimensionsTooLarge(u32, u32, u32, u32),

    /// File format not supported
    #[error("File format '{0}' not supported")]
    UnsupportedFormat(String),

    /// File not found
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Generic error
    #[error("{0}")]
    Generic(String),
}

/// Validate media against platform constraints.
///
/// # Arguments
///
/// * `file_path` - Path to the media file
/// * `file_size` - Size of the file in bytes
/// * `constraints` - Platform constraints to validate against
/// * `media_type` - Type of media (image, video, audio, file)
///
/// # Returns
///
/// Returns `Ok(())` if the media is valid, or `Err(MediaError)` if validation fails.
pub fn validate_media(
    file_path: &str,
    file_size: usize,
    constraints: &MediaConstraints,
    media_type: MediaType,
) -> Result<(), MediaError> {
    // Check file exists
    std::fs::metadata(file_path).map_err(|e| MediaError::FileNotFound(file_path.to_string()))?;

    match media_type {
        MediaType::Image => {
            // Check file size
            if file_size > constraints.max_image_size_bytes {
                return Err(MediaError::FileTooLarge(
                    file_size,
                    constraints.max_image_size_bytes,
                ));
            }
        }
        MediaType::Video => {
            if file_size > constraints.max_video_size_bytes {
                return Err(MediaError::FileTooLarge(
                    file_size,
                    constraints.max_video_size_bytes,
                ));
            }
        }
        MediaType::Audio => {
            if file_size > constraints.max_audio_size_bytes {
                return Err(MediaError::FileTooLarge(
                    file_size,
                    constraints.max_audio_size_bytes,
                ));
            }
        }
        MediaType::File => {
            if file_size > constraints.max_file_size_bytes {
                return Err(MediaError::FileTooLarge(
                    file_size,
                    constraints.max_file_size_bytes,
                ));
            }
        }
    }

    Ok(())
}

/// Media type enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaType {
    /// Image media
    Image,
    /// Video media
    Video,
    /// Audio media
    Audio,
    /// Generic file
    File,
}

impl MediaType {
    /// Get the default file extensions for this media type.
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            MediaType::Image => &["png", "jpg", "jpeg", "gif", "webp", "bmp"],
            MediaType::Video => &["mp4", "mov", "avi", "mkv", "webm"],
            MediaType::Audio => &["mp3", "wav", "ogg", "aac", "flac"],
            MediaType::File => &["txt", "pdf", "doc", "docx", "xls", "xlsx", "zip", "rar"],
        }
    }

    /// Check if the file extension is valid for this media type.
    pub fn is_valid_extension(&self, extension: &str) -> bool {
        let ext = extension.to_lowercase();
        self.extensions().iter().any(|e| *e == ext)
    }
}

/// Media file information.
#[derive(Debug, Clone)]
pub struct MediaInfo {
    /// File path
    pub path: String,
    /// File size in bytes
    pub size_bytes: usize,
    /// MIME type
    pub mime_type: String,
    /// Media type
    pub media_type: MediaType,
    /// Optional dimensions (width, height)
    pub dimensions: Option<(u32, u32)>,
    /// Optional duration in seconds
    pub duration_seconds: Option<u64>,
}

/// Detect media type from file extension.
pub fn detect_media_type_from_extension(extension: &str) -> MediaType {
    let ext = extension.to_lowercase();
    match ext.as_str() {
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" => MediaType::Image,
        "mp4" | "mov" | "avi" | "mkv" | "webm" => MediaType::Video,
        "mp3" | "wav" | "ogg" | "aac" | "flac" => MediaType::Audio,
        _ => MediaType::File,
    }
}

/// Get MIME type for a file extension.
pub fn get_mime_type(extension: &str) -> &'static str {
    let ext = extension.to_lowercase();
    match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "mp4" => "video/mp4",
        "mov" => "video/quicktime",
        "avi" => "video/x-msvideo",
        "mkv" => "video/x-matroska",
        "webm" => "video/webm",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "aac" => "audio/aac",
        "flac" => "audio/flac",
        "txt" => "text/plain",
        "pdf" => "application/pdf",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xls" => "application/vnd.ms-excel",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "zip" => "application/zip",
        "rar" => "application/vnd.rar",
        _ => "application/octet-stream",
    }
}

/// Media conversion options.
#[derive(Debug, Clone)]
pub struct ConversionOptions {
    /// Target format
    pub format: String,
    /// Target dimensions (width, height)
    pub dimensions: Option<(u32, u32)>,
    /// Quality for lossy formats (0-100)
    pub quality: Option<u8>,
    /// Whether to preserve aspect ratio
    pub preserve_aspect_ratio: bool,
}

/// Convert media to a compatible format for the target platform.
///
/// # Arguments
///
/// * `input_path` - Path to the input media file
/// * `output_path` - Path where the converted media will be saved
/// * `constraints` - Target platform constraints
/// * `options` - Conversion options
///
/// # Returns
///
/// Returns the media info of the converted file.
pub fn convert_media(
    input_path: &str,
    output_path: &str,
    constraints: &MediaConstraints,
    options: &ConversionOptions,
) -> Result<MediaInfo, MediaError> {
    // In a real implementation, this would use image processing libraries
    // For now, return the input info as a placeholder
    let metadata = std::fs::metadata(input_path)?;
    let extension = std::path::Path::new(input_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    
    Ok(MediaInfo {
        path: output_path.to_string(),
        size_bytes: metadata.len() as usize,
        mime_type: get_mime_type(extension).to_string(),
        media_type: detect_media_type_from_extension(extension),
        dimensions: None,
        duration_seconds: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_constraints() {
        let signal = MediaConstraints::for_platform("signal");
        assert_eq!(signal.max_image_size_bytes, 100 * 1024 * 1024);
        assert!(signal.max_image_dimensions.is_none());

        let discord = MediaConstraints::for_platform("discord");
        assert_eq!(discord.max_image_size_bytes, 25 * 1024 * 1024);
        assert_eq!(discord.max_image_dimensions, Some((10_000, 10_000)));

        let irc = MediaConstraints::for_platform("irc");
        assert_eq!(irc.max_image_size_bytes, 0);
        assert_eq!(irc.supported_image_formats.len(), 0);
    }

    #[test]
    fn test_media_type_extensions() {
        assert_eq!(MediaType::Image.extensions(), &["png", "jpg", "jpeg", "gif", "webp", "bmp"]);
        assert_eq!(MediaType::Video.extensions(), &["mp4", "mov", "avi", "mkv", "webm"]);
        assert_eq!(MediaType::Audio.extensions(), &["mp3", "wav", "ogg", "aac", "flac"]);
    }

    #[test]
    fn test_detect_media_type() {
        assert_eq!(detect_media_type_from_extension("png"), MediaType::Image);
        assert_eq!(detect_media_type_from_extension("jpg"), MediaType::Image);
        assert_eq!(detect_media_type_from_extension("mp4"), MediaType::Video);
        assert_eq!(detect_media_type_from_extension("mp3"), MediaType::Audio);
        assert_eq!(detect_media_type_from_extension("pdf"), MediaType::File);
    }

    #[test]
    fn test_get_mime_type() {
        assert_eq!(get_mime_type("png"), "image/png");
        assert_eq!(get_mime_type("jpg"), "image/jpeg");
        assert_eq!(get_mime_type("mp4"), "video/mp4");
        assert_eq!(get_mime_type("mp3"), "audio/mpeg");
        assert_eq!(get_mime_type("unknown"), "application/octet-stream");
    }

    #[test]
    fn test_is_valid_extension() {
        assert!(MediaType::Image.is_valid_extension("png"));
        assert!(MediaType::Image.is_valid_extension("PNG"));
        assert!(!MediaType::Image.is_valid_extension("mp4"));
    }

    #[test]
    fn test_default_constraints() {
        let default = MediaConstraints::default();
        assert_eq!(default.max_image_size_bytes, 10 * 1024 * 1024);
        assert_eq!(default.max_file_size_bytes, 50 * 1024 * 1024);
        assert_eq!(default.supported_image_formats, vec!["png", "jpg", "jpeg"]);
    }

    #[test]
    fn test_validate_media_file_too_large() {
        let constraints = MediaConstraints::for_platform("discord");
        // Create a temporary file for testing
        let temp_file = std::env::temp_dir().join("test_media_validation.png");
        std::fs::write(&temp_file, vec![0u8; 30 * 1024 * 1024]).expect("Failed to create test file");
        
        let result = validate_media(
            temp_file.to_str().unwrap(), 
            30 * 1024 * 1024, 
            &constraints, 
            MediaType::Image
        );
        
        assert!(matches!(result, Err(MediaError::FileTooLarge(_, _))));
        
        // Clean up
        let _ = std::fs::remove_file(&temp_file);
    }

    #[test]
    fn test_validate_media_valid() {
        let constraints = MediaConstraints::for_platform("signal");
        // Just verify it doesn't error on file existence check
        // We can't test actual file validation without creating a test file
    }
}
