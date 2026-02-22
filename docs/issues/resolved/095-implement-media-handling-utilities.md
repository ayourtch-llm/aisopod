# Issue 095: Implement Media Handling Utilities

## Summary
Implement media handling utilities for processing images, detecting media types, and providing integration points for audio and document handling within the channel abstraction layer.

## Location
- Crate: `aisopod-channel`
- File: `crates/aisopod-channel/src/media.rs`

## Current Behavior
There are no utilities for processing media attachments. Channel plugins have no standardized way to resize images, detect file types, or prepare media for delivery.

## Expected Behavior
A `MediaHandler` struct provides common media processing utilities:

- **Image processing** — resize images to channel-specific maximum dimensions, convert between formats (e.g., WebP to PNG), and generate thumbnails using the `image` crate.
- **Media type detection** — detect `MediaType` from file extension, MIME type string, or raw bytes (magic bytes).
- **Media validation** — validate that media meets channel constraints (max file size, supported types) based on `ChannelCapabilities`.
- **Audio handling integration point** — a trait method stub for audio transcription that future crates can implement.
- **Document handling integration point** — a trait method stub for document text extraction (e.g., PDF to text) that future crates can implement.

Key functions:
```rust
pub fn resize_image(data: &[u8], max_width: u32, max_height: u32) -> Result<Vec<u8>>;
pub fn convert_image_format(data: &[u8], target_format: ImageFormat) -> Result<Vec<u8>>;
pub fn detect_media_type(data: &[u8], filename: Option<&str>) -> MediaType;
pub fn detect_mime_type(data: &[u8], filename: Option<&str>) -> String;
pub fn validate_media(media: &Media, capabilities: &ChannelCapabilities) -> Result<()>;
```

Integration traits:
```rust
#[async_trait]
pub trait AudioTranscriber: Send + Sync {
    async fn transcribe(&self, audio_data: &[u8], mime_type: &str) -> Result<String>;
}

#[async_trait]
pub trait DocumentExtractor: Send + Sync {
    async fn extract_text(&self, doc_data: &[u8], mime_type: &str) -> Result<String>;
}
```

## Impact
Media handling is essential for channels that support images, audio, and documents (e.g., Telegram, Discord). Without these utilities, each channel plugin would need to implement its own media processing, leading to duplication and inconsistency.

## Resolution
The following was implemented:

- Created `MediaHandler` with `resize_image()`, `convert_image_format()`, `detect_media_type()`, `detect_mime_type()`, and `validate_media()` functions
- Created `AudioTranscriber` and `DocumentExtractor` async traits for future integration
- Added `image` crate dependency to `Cargo.toml` with support for PNG, JPEG, GIF, and WebP formats
- Added 14 comprehensive unit tests covering media type detection, MIME type detection, image resizing, and media validation
- All types and functions have doc-comments following Rust documentation conventions
- Media module is exported from `lib.rs` with all public items re-exported

## Suggested Implementation
1. Add the `image` crate to `crates/aisopod-channel/Cargo.toml` as a dependency.
2. Open `crates/aisopod-channel/src/media.rs`.
3. Implement `detect_media_type()`:
   - Check the first few bytes of `data` for known magic bytes (JPEG: `FF D8 FF`, PNG: `89 50 4E 47`, GIF: `47 49 46`, PDF: `25 50 44 46`).
   - Fall back to file extension if `filename` is provided (e.g., `.mp3` → `Audio`, `.pdf` → `Document`).
   - Return `MediaType::Other(String)` for unrecognized types.
4. Implement `detect_mime_type()`:
   - Similar to `detect_media_type()` but returns a MIME type string (e.g., `"image/jpeg"`, `"application/pdf"`).
5. Implement `resize_image()`:
   - Use `image::load_from_memory(data)` to decode the image.
   - If the image dimensions exceed `max_width` or `max_height`, use `image::imageops::resize` with `FilterType::Lanczos3` to scale it down while preserving aspect ratio.
   - Encode the result back to the original format and return the bytes.
6. Implement `convert_image_format()`:
   - Decode with `image::load_from_memory(data)`.
   - Encode to the target format using the `image` crate's format writers.
   - Return the new bytes.
7. Implement `validate_media()`:
   - Check if the media's `MediaType` is in `capabilities.supported_media_types`. If not, return an error.
   - If `capabilities.supports_media` is false, return an error.
   - Optionally check file size against any configured limits.
8. Define the `AudioTranscriber` trait with a single `transcribe()` method. Add a doc-comment noting this is an integration point for future implementation.
9. Define the `DocumentExtractor` trait with a single `extract_text()` method. Add a doc-comment noting this is an integration point for future implementation.
10. Add doc-comments to every function and trait.
11. Re-export all public items from `crates/aisopod-channel/src/lib.rs`.
12. Run `cargo check -p aisopod-channel` to verify everything compiles.

## Dependencies
- Issue 091 (define message types — provides `Media`, `MediaType`)

## Acceptance Criteria
- [x] `resize_image()` scales images down to specified maximum dimensions using the `image` crate
- [x] `convert_image_format()` converts between image formats
- [x] `detect_media_type()` correctly identifies images, audio, video, and documents from magic bytes and file extensions
- [x] `detect_mime_type()` returns correct MIME type strings
- [x] `validate_media()` checks media against `ChannelCapabilities` constraints
- [x] `AudioTranscriber` and `DocumentExtractor` traits are defined as integration points
- [x] Every public function and trait has a doc-comment
- [x] `cargo check -p aisopod-channel` compiles without errors
- [x] All 14 unit tests pass

---
*Created: 2026-02-15*
*Resolved: 2026-02-22*
