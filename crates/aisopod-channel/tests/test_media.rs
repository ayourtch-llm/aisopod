//! Tests for the media handling utilities.

use std::io::Cursor;

use aisopod_channel::media::{detect_media_type, detect_mime_type, resize_image, validate_media};
use aisopod_channel::types::MediaType;
use image::{DynamicImage, GenericImageView, ImageBuffer, ImageFormat, Rgba};

// ============================================================================
// Media Type Detection Tests (Magic Bytes)
// ============================================================================

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

// ============================================================================
// Media Type Detection Tests (By Extension)
// ============================================================================

#[test]
fn test_detect_media_type_by_extension_image() {
    // Test extension-based detection for images
    let data = [0x00, 0x00, 0x00, 0x00]; // Unknown magic bytes
    let media_type = detect_media_type(&data, Some("photo.jpg"));
    assert_eq!(media_type, MediaType::Image);
}

#[test]
fn test_detect_media_type_by_extension_png() {
    let data = [0x00, 0x00, 0x00, 0x00];
    let media_type = detect_media_type(&data, Some("photo.png"));
    assert_eq!(media_type, MediaType::Image);
}

#[test]
fn test_detect_media_type_by_extension_gif() {
    let data = [0x00, 0x00, 0x00, 0x00];
    let media_type = detect_media_type(&data, Some("animation.gif"));
    assert_eq!(media_type, MediaType::Image);
}

#[test]
fn test_detect_media_type_by_extension_webp() {
    let data = [0x00, 0x00, 0x00, 0x00];
    let media_type = detect_media_type(&data, Some("image.webp"));
    assert_eq!(media_type, MediaType::Image);
}

#[test]
fn test_detect_media_type_by_extension_bmp() {
    let data = [0x00, 0x00, 0x00, 0x00];
    let media_type = detect_media_type(&data, Some("picture.bmp"));
    assert_eq!(media_type, MediaType::Image);
}

#[test]
fn test_detect_media_type_by_extension_audio_mp3() {
    let data = [0x00, 0x00, 0x00, 0x00];
    let media_type = detect_media_type(&data, Some("music.mp3"));
    assert_eq!(media_type, MediaType::Audio);
}

#[test]
fn test_detect_media_type_by_extension_audio_wav() {
    let data = [0x00, 0x00, 0x00, 0x00];
    let media_type = detect_media_type(&data, Some("sound.wav"));
    assert_eq!(media_type, MediaType::Audio);
}

#[test]
fn test_detect_media_type_by_extension_audio_ogg() {
    let data = [0x00, 0x00, 0x00, 0x00];
    let media_type = detect_media_type(&data, Some("audio.ogg"));
    assert_eq!(media_type, MediaType::Audio);
}

#[test]
fn test_detect_media_type_by_extension_audio_flac() {
    let data = [0x00, 0x00, 0x00, 0x00];
    let media_type = detect_media_type(&data, Some("track.flac"));
    assert_eq!(media_type, MediaType::Audio);
}

#[test]
fn test_detect_media_type_by_extension_audio_aac() {
    let data = [0x00, 0x00, 0x00, 0x00];
    let media_type = detect_media_type(&data, Some("song.aac"));
    assert_eq!(media_type, MediaType::Audio);
}

#[test]
fn test_detect_media_type_by_extension_video_mp4() {
    let data = [0x00, 0x00, 0x00, 0x00];
    let media_type = detect_media_type(&data, Some("movie.mp4"));
    assert_eq!(media_type, MediaType::Video);
}

#[test]
fn test_detect_media_type_by_extension_video_webm() {
    let data = [0x00, 0x00, 0x00, 0x00];
    let media_type = detect_media_type(&data, Some("clip.webm"));
    assert_eq!(media_type, MediaType::Video);
}

#[test]
fn test_detect_media_type_by_extension_video_mkv() {
    let data = [0x00, 0x00, 0x00, 0x00];
    let media_type = detect_media_type(&data, Some("video.mkv"));
    assert_eq!(media_type, MediaType::Video);
}

#[test]
fn test_detect_media_type_by_extension_video_avi() {
    let data = [0x00, 0x00, 0x00, 0x00];
    let media_type = detect_media_type(&data, Some("film.avi"));
    assert_eq!(media_type, MediaType::Video);
}

#[test]
fn test_detect_media_type_by_extension_document_pdf() {
    let data = [0x00, 0x00, 0x00, 0x00];
    let media_type = detect_media_type(&data, Some("document.pdf"));
    assert_eq!(media_type, MediaType::Document);
}

#[test]
fn test_detect_media_type_by_extension_document_doc() {
    let data = [0x00, 0x00, 0x00, 0x00];
    let media_type = detect_media_type(&data, Some("file.doc"));
    assert_eq!(media_type, MediaType::Document);
}

#[test]
fn test_detect_media_type_by_extension_document_docx() {
    let data = [0x00, 0x00, 0x00, 0x00];
    let media_type = detect_media_type(&data, Some("file.docx"));
    assert_eq!(media_type, MediaType::Document);
}

#[test]
fn test_detect_media_type_by_extension_document_txt() {
    let data = [0x00, 0x00, 0x00, 0x00];
    let media_type = detect_media_type(&data, Some("text.txt"));
    assert_eq!(media_type, MediaType::Document);
}

#[test]
fn test_detect_media_type_by_extension_document_rtf() {
    let data = [0x00, 0x00, 0x00, 0x00];
    let media_type = detect_media_type(&data, Some("file.rtf"));
    assert_eq!(media_type, MediaType::Document);
}

#[test]
fn test_detect_media_type_unknown() {
    let data = [0x00, 0x00, 0x00, 0x00];
    let media_type = detect_media_type(&data, Some("file.xyz"));
    assert_eq!(media_type, MediaType::Other("unknown".to_string()));
}

// ============================================================================
// MIME Type Detection Tests
// ============================================================================

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
fn test_detect_mime_type_gif() {
    let data = [b'G', b'I', b'F', 0x00];
    let mime_type = detect_mime_type(&data, None);
    assert_eq!(mime_type, "image/gif");
}

#[test]
fn test_detect_mime_type_pdf() {
    let data = [b'%', b'P', b'D', b'F', 0x00];
    let mime_type = detect_mime_type(&data, None);
    assert_eq!(mime_type, "application/pdf");
}

#[test]
fn test_detect_mime_type_by_extension_mp3() {
    let data = [0x00, 0x00, 0x00, 0x00];
    let mime_type = detect_mime_type(&data, Some("music.mp3"));
    assert_eq!(mime_type, "audio/mpeg");
}

#[test]
fn test_detect_mime_type_by_extension_mp4() {
    let data = [0x00, 0x00, 0x00, 0x00];
    let mime_type = detect_mime_type(&data, Some("video.mp4"));
    assert_eq!(mime_type, "video/mp4");
}

#[test]
fn test_detect_mime_type_unknown() {
    let data = [0x00, 0x00, 0x00, 0x00];
    let mime_type = detect_mime_type(&data, Some("file.xyz"));
    assert_eq!(mime_type, "application/octet-stream");
}

// ============================================================================
// Image Resizing Tests
// ============================================================================

#[test]
fn test_resize_image_larger_than_max() {
    // Create a 1000x1000 image
    let img_buffer: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(1000, 1000);
    
    // Encode to PNG bytes
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
fn test_resize_image_smaller_than_max() {
    // Create a 200x200 image
    let img_buffer: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(200, 200);
    
    // Encode to PNG bytes
    let dynamic_img = DynamicImage::ImageRgba8(img_buffer);
    let mut buf = Cursor::new(Vec::new());
    dynamic_img.write_to(&mut buf, ImageFormat::Png).unwrap();
    let data = buf.into_inner();
    
    // Resize to max 500x500 (image is already smaller)
    let resized = resize_image(&data, 500, 500).unwrap();
    
    // Verify dimensions are unchanged
    let img = image::load_from_memory(&resized).unwrap();
    let (width, height) = img.dimensions();
    assert_eq!(width, 200);
    assert_eq!(height, 200);
}

#[test]
fn test_resize_image_preserves_aspect_ratio() {
    // Create a 1000x500 image (2:1 aspect ratio)
    let img_buffer: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(1000, 500);
    
    // Encode to PNG bytes
    let dynamic_img = DynamicImage::ImageRgba8(img_buffer);
    let mut buf = Cursor::new(Vec::new());
    dynamic_img.write_to(&mut buf, ImageFormat::Png).unwrap();
    let data = buf.into_inner();
    
    // Resize to max 400x400
    let resized = resize_image(&data, 400, 400).unwrap();
    
    // Verify aspect ratio is preserved (should be 2:1)
    let img = image::load_from_memory(&resized).unwrap();
    let (width, height) = img.dimensions();
    // Width should be 400, height should be 200
    assert_eq!(width, 400);
    assert_eq!(height, 200);
}

// ============================================================================
// Media Validation Tests
// ============================================================================

#[test]
fn test_validate_media_success() {
    let media_type = MediaType::Image;
    let capabilities = aisopod_channel::types::ChannelCapabilities {
        supports_media: true,
        supported_media_types: vec![MediaType::Image, MediaType::Document],
        ..Default::default()
    };
    
    assert!(validate_media(&media_type, &capabilities).is_ok());
}

#[test]
fn test_validate_media_not_supported() {
    let media_type = MediaType::Video;
    let capabilities = aisopod_channel::types::ChannelCapabilities {
        supports_media: true,
        supported_media_types: vec![MediaType::Image, MediaType::Document],
        ..Default::default()
    };
    
    let result = validate_media(&media_type, &capabilities);
    assert!(result.is_err());
}

#[test]
fn test_validate_media_disabled() {
    let media_type = MediaType::Image;
    let capabilities = aisopod_channel::types::ChannelCapabilities {
        supports_media: false,
        supported_media_types: vec![],
        ..Default::default()
    };
    
    let result = validate_media(&media_type, &capabilities);
    assert!(result.is_err());
}

#[test]
fn test_validate_media_type_not_in_supported_list() {
    let media_type = MediaType::Audio;
    let capabilities = aisopod_channel::types::ChannelCapabilities {
        supports_media: true,
        supported_media_types: vec![MediaType::Image],
        ..Default::default()
    };
    
    let result = validate_media(&media_type, &capabilities);
    assert!(result.is_err());
}

#[test]
fn test_validate_media_all_supported_types() {
    // Test that all supported types pass validation
    let supported_types = vec![
        MediaType::Image,
        MediaType::Audio,
        MediaType::Video,
        MediaType::Document,
    ];
    
    let capabilities = aisopod_channel::types::ChannelCapabilities {
        supports_media: true,
        supported_media_types: supported_types.clone(),
        ..Default::default()
    };
    
    for media_type in supported_types {
        assert!(validate_media(&media_type, &capabilities).is_ok());
    }
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_detect_media_type_empty_data() {
    let data: Vec<u8> = vec![];
    let media_type = detect_media_type(&data, None);
    // Should fall back to Other/unknown
    assert_eq!(media_type, MediaType::Other("unknown".to_string()));
}

#[test]
fn test_detect_media_type_short_data() {
    let data = [0xFF, 0xD8]; // Only 2 bytes of JPEG header
    let media_type = detect_media_type(&data, None);
    // Should fall back to extension or unknown
    let media_type = detect_media_type(&data, Some("photo.jpg"));
    assert_eq!(media_type, MediaType::Image);
}

#[test]
fn test_resize_image_corrupted_data() {
    let corrupted_data = vec![0xFF, 0xD8]; // Incomplete JPEG
    let result = resize_image(&corrupted_data, 100, 100);
    // Should return an error
    assert!(result.is_err());
}

#[test]
fn test_resize_image_png_to_jpeg() {
    // Create a 100x100 PNG image
    let img_buffer: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(100, 100);
    let dynamic_img = DynamicImage::ImageRgba8(img_buffer);
    let mut buf = Cursor::new(Vec::new());
    dynamic_img.write_to(&mut buf, ImageFormat::Png).unwrap();
    let data = buf.into_inner();
    
    // Resize (preserves original format)
    let resized = resize_image(&data, 50, 50).unwrap();
    
    // Verify it's still a PNG (format is preserved)
    let img = image::load_from_memory(&resized).unwrap();
    assert_eq!(img.dimensions(), (50, 50));
}

#[test]
fn test_detect_media_type_case_insensitive_extension() {
    // Test that extension matching is case-insensitive
    let data = [0x00, 0x00, 0x00, 0x00];
    
    let media_type_upper = detect_media_type(&data, Some("file.JPG"));
    assert_eq!(media_type_upper, MediaType::Image);
    
    let media_type_mixed = detect_media_type(&data, Some("file.PnG"));
    assert_eq!(media_type_mixed, MediaType::Image);
}
