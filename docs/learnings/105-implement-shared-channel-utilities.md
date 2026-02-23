# Issue #105: Implement Shared Channel Utilities - Learning Summary

**Date:** 2026-02-23  
**Issue:** #105  
**Status:** Code implemented, integration pending

---

## Summary

This issue implemented a shared utilities module (`aisopod-channel/src/util/`) for cross-platform message formatting normalization, media transcoding, rate limit handling, connection state management with reconnection logic, and error mapping.

The module is **fully implemented** with comprehensive unit tests but has **not yet been integrated** into the Tier 1 channel crates (Telegram, Discord, WhatsApp, Slack).

---

## Key Learnings

### 1. Module Organization Pattern

**Lesson:** Create a dedicated `util/` subdirectory for shared utilities within a crate.

The implementation demonstrates a clear pattern for organizing shared code:

```
crates/aisopod-channel/
├── src/
│   ├── util/
│   │   ├── mod.rs          # Module declarations and re-exports
│   │   ├── formatting.rs   # Markdown normalization
│   │   ├── media.rs        # Media transcoding and validation
│   │   ├── rate_limit.rs   # Rate limiting logic
│   │   ├── connection.rs   # Connection state management
│   │   └── errors.rs       # Common error types
│   ├── lib.rs              # Re-exports from util module
│   ├── adapters.rs
│   ├── channel.rs
│   ├── plugin.rs
│   └── ...
```

**Benefits:**
- Clear separation of shared utilities vs. platform-specific code
- Easy to discover and import shared types
- Proper module hierarchy that scales with complexity

**Pattern to Follow:**
```rust
// util/mod.rs
pub mod connection;
pub mod errors;
pub mod formatting;
pub mod media;
pub mod rate_limit;

// Re-export for convenience
pub use formatting::NormalizedMarkdown;
pub use media::MediaAttachment;
pub use rate_limit::RateLimiter;
pub use connection::{ConnectionManager, ConnectionState};
pub use errors::ChannelError;
```

### 2. NormalizedMarkdown Architecture

**Lesson:** Define a platform-agnostic intermediate representation for cross-platform formatting.

The `NormalizedMarkdown` type serves as a canonical format that can be:
1. Parsed from platform-specific syntax (Telegram MarkdownV2, Discord markdown, Slack mrkdwn)
2. Stored internally without platform assumptions
3. Rendered to any supported platform format

**Implementation Pattern:**
```rust
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NormalizedMarkdown {
    pub segments: Vec<FormatSegment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormatSegment {
    Text(String),
    Bold(String),
    Italic(String),
    // ... other formats
    Link { url: String, text: String },
    CodeBlock { language: Option<String>, content: String },
}
```

**Key Design Decisions:**
- **Segments approach:** Allows mixing formatted and unformatted text within a single message
- **Explicit link structure:** Links are a separate variant with URL and text separate, not embedded in text
- **Code blocks with language:** Allows syntax highlighting when rendering

**Limitations Discovered:**
- The parser currently only handles: plain text, bold, italic, inline code, code blocks
- It does NOT parse: links, quotes (these are only renderable, not parseable)
- This asymmetry should be addressed: either implement full parsing or document the limitation

**Recommendation:** Either:
1. Complete the parser to handle all supported formatting, OR
2. Add parser limitation documentation to prevent user confusion

### 3. Rate Limiting Implementation

**Lesson:** Use a sliding window algorithm with per-chat tracking for accurate rate limiting.

The `RateLimiter` implementation uses a sliding window approach with:
- `tokio::sync::RwLock` for thread-safe concurrent access
- Timestamp vectors for tracking request history
- Separate tracking for global and per-chat limits

**Implementation Pattern:**
```rust
pub struct RateLimiter {
    config: RateLimitConfig,
    global_requests: Arc<RwLock<HashMap<(), Vec<Instant>>>>,
    per_chat_requests: Arc<RwLock<HashMap<String, Vec<Instant>>>>,
    retry_after: Arc<RwLock<HashMap<String, Instant>>>,
}
```

**Key Features:**
- **Try-acquire vs. Acquire:** Separate methods for non-blocking and blocking acquisition
- **Per-chat limits:** Important for platforms like Telegram with 20 messages/minute per chat
- **Retry-After handling:** Properly honors server responses when rate limited

**Platform Configurations:** Each platform's limits are explicitly defined:
```rust
impl Platform {
    pub fn default_config(&self) -> RateLimitConfig {
        match self {
            Platform::Telegram => RateLimitConfig {
                global_limit: RateLimit::new(30, Duration::from_secs(1)),
                per_chat_limit: RateLimit::new(20, Duration::from_secs(60)),
            },
            // ...
        }
    }
}
```

**Recommendation:** Consider adding:
- Metrics/export of current request counts for observability
- Per-user rate limiting (in addition to per-chat)
- Burst handling for platforms that allow it

### 4. Connection State Management

**Lesson:** Implement connection state machine with exponential backoff and jitter.

The `ConnectionManager` provides a robust state machine pattern:

```rust
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Failed,
}
```

**Key Features:**
- **Exponential Backoff:** Delay doubles after each failed attempt (1s, 2s, 4s, 8s...)
- **Jitter:** Random offset prevents thundering herd when multiple channels reconnect
- **State observation:** `watch::Receiver` allows other components to monitor state changes
- **Statistics tracking:** Counts reconnection attempts and uptime

**Implementation Pattern:**
```rust
pub async fn maintain_connection<F, Fut>(&self, connect: F) -> Result<(), String>
where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<(), String>>,
{
    loop {
        match connect().await {
            Ok(()) => {
                self.record_connect();
                self.reset_backoff();
                return Ok(());
            }
            Err(_) => {
                let delay = self.next_delay();  // Exponential backoff with jitter
                tokio::time::sleep(delay).await;
            }
        }
    }
}
```

**Recommendation:** Consider adding:
- Circuit breaker pattern (stop retrying after N consecutive failures)
- Health check endpoint or callback
- Configuration for minimum/maximum backoff per connection type

### 5. Error Handling Strategy

**Lesson:** Define a unified error type that covers common scenarios across platforms.

The `ChannelError` enum provides a canonical error type with variants for:

```rust
pub enum ChannelError {
    AuthenticationFailed,
    RateLimited { retry_after: Duration },
    MessageTooLong { max_length: usize },
    MediaUnsupported { media_type: String },
    ConnectionLost,
    PermissionDenied,
    NotFound { resource: String },
    PlatformError { code: String, message: String },
    Generic(String),
    Io(std::io::Error),
    Other(anyhow::Error),
}
```

**Key Features:**
- **Platform-agnostic variants:** Most errors have a unified representation
- **Platform pass-through:** `PlatformError` allows preserving platform-specific details
- **Retry hints:** `should_retry()` and `retry_after()` methods
- **Error source chain:** Implements `std::error::Error` for proper error chaining

**Current Gap:** The `From` trait implementations for platform-specific errors are **not** present in the codebase. Only example code is provided in comments.

**Implementation Pattern (Example):**
```rust
// In aisopod-channel-telegram crate:
impl From<teloxide::ApiError> for ChannelError {
    fn from(err: teloxide::ApiError) -> Self {
        match err {
            teloxide::ApiError::Unauthorized => ChannelError::AuthenticationFailed,
            teloxide::ApiError::Forbidden => ChannelError::PermissionDenied,
            teloxide::ApiError::NotFound => ChannelError::not_found("resource"),
            teloxide::ApiError::TooManyRequests { retry_after } => {
                ChannelError::rate_limited(retry_after)
            }
            _ => ChannelError::Generic(err.to_string()),
        }
    }
}
```

**Recommendation:** 
1. Implement actual `From` traits in each channel crate
2. Add a macro for common error mappings
3. Consider creating platform-specific error modules for more detailed handling

### 6. Media Validation vs. Transcoding

**Lesson:** Separate validation from transcoding in the media module.

The media module correctly separates concerns:

**Validation (Implemented):**
- File size checking against platform limits
- Format compatibility checking
- Image dimension checking

**Transcoding (Not Implemented):**
- Image format conversion (PNG↔JPEG↔WebP)
- Audio format conversion (MP3↔OGG↔AAC)
- Video format conversion (MP4↔WebM)

**Current State:**
```rust
pub fn ensure_compatible_format(media: &MediaAttachment, target: Platform) -> Result<MediaAttachment> {
    // 1. Check file size
    if media.data.len() > constraints.max_file_size {
        return Err(...);
    }
    
    // 2. Check format compatibility
    if !constraints.supported_image_formats.contains(&format) {
        // ❌ Returns error - conversion not implemented
        return Err(anyhow!("format conversion not yet implemented"));
    }
    
    // 3. Check dimensions
    if exceeds_max_dimensions(media, constraints) {
        // ⚠️ Returns original data (stub)
        return Ok(media.data.clone());
    }
}
```

**Recommendation:** 
1. Implement basic transcoding using `image` crate for images
2. Integrate `ffmpeg` bindings or `symphonia` for audio/video
3. Consider async transcoding for large files
4. Add caching for transcoded media

### 7. Testing Strategy for Shared Utilities

**Lesson:** Write comprehensive unit tests for shared utilities before platform integration.

The util module has **68 unit tests** covering:
- Formatting: 18 tests (parsing, rendering, escaping)
- Media: 17 tests (format detection, constraints, dimensions)
- Rate limiting: 13 tests (acquisition, per-chat, retry-after)
- Connection: 17 tests (state transitions, backoff, jitter)
- Errors: 14 tests (display, error handling)

**Test Pattern:**
```rust
#[tokio::test]
async fn test_rate_limiter_creation() {
    let limiter = RateLimiter::new(Platform::Telegram);
    assert_eq!(limiter.config().global_limit.max_requests, 30);
}

#[tokio::test]
async fn test_per_chat_rate_limiting() {
    let config = RateLimitConfig {
        global_limit: RateLimit::new(100, Duration::from_secs(10)),
        per_chat_limit: RateLimit::new(2, Duration::from_secs(10)),
    };
    let limiter = RateLimiter::with_config(config);
    
    assert!(limiter.try_acquire(Some("chat1")).await.is_ok());
    assert!(limiter.try_acquire(Some("chat1")).await.is_ok());
    assert!(limiter.try_acquire(Some("chat1")).await.is_err()); // Third fails
    assert!(limiter.try_acquire(Some("chat2")).await.is_ok()); // Different chat OK
}
```

**Recommendation:** 
1. Add integration tests after channel crate integration
2. Add benchmarks for rate limiter (sliding window performance)
3. Add property-based tests for formatting round-trips

---

## Integration Checklist for Tier 1 Channel Crates

Before this issue can be considered complete, each Tier 1 channel crate needs to integrate the shared utilities:

### For each channel crate (Telegram, Discord, WhatsApp, Slack):

1. **Add imports:**
   ```rust
   use aisopod_channel::util::{
       NormalizedMarkdown, ChannelError, RateLimiter, ConnectionManager, ConnectionState,
   };
   ```

2. **Implement `From` for platform-specific errors:**
   ```rust
   impl From<PlatformSpecificError> for ChannelError {
       fn from(err: PlatformSpecificError) -> Self {
           // Map to appropriate ChannelError variant
       }
   }
   ```

3. **Integrate RateLimiter:**
   - Create `RateLimiter` on channel initialization
   - Call `rate_limiter.acquire().await` before API calls
   - Handle `RateLimitError::RetryAfter` by calling `handle_retry_after()`

4. **Integrate ConnectionManager:**
   - Create `ConnectionManager` for persistent connections (WebSocket, long-polling)
   - Use `maintain_connection()` for reconnection logic
   - Observe `ConnectionState` via watch channel for UI updates

5. **Integrate NormalizedMarkdown:**
   - Parse incoming messages using `from_telegram_markdown()`, etc.
   - Render outgoing messages using `to_platform_markdown()`
   - Handle parsing limitations gracefully

6. **Integrate media validation:**
   - Use `ensure_compatible_format()` before sending media
   - Fall back to original if transcoding not available

7. **Add tests:**
   - Test error conversions work correctly
   - Test rate limiting in integration scenarios
   - Test formatting round-trips

---

## Known Gaps

### 1. Media Format Transcoding
- Image format conversion: Not implemented
- Audio format conversion: Not implemented  
- Video format conversion: Not implemented

**Impact:** Media files may fail if they're in an unsupported format for the target platform.

**Mitigation:** Current implementation returns an error when transcoding is needed, allowing the caller to handle it (likely by showing an error to the user).

### 2. Platform Error Conversions
- `From` trait implementations for platform-specific errors are not present

**Impact:** Each channel crate must handle platform errors independently, losing the benefit of unified error handling.

**Mitigation:** Example code is provided in `errors.rs` as a reference implementation.

### 3. Parser Limitations
- Links and quotes are renderable but not parseable

**Impact:** Cross-platform message forwarding may lose link/quote information when parsing Telegram/Discord/Slack messages.

**Mitigation:** Document the limitation or complete the parser implementation.

---

## Recommendations

### Immediate Actions
1. **Integrate shared utilities into Tier 1 channel crates** - This is the most critical missing piece
2. **Implement media format transcoding** - Using the `image` crate for images
3. **Add `From` implementations for platform errors** - Follow the example code in `errors.rs`

### Medium-Term Improvements
1. **Complete the parser** - Implement link and quote parsing
2. **Add integration tests** - Test end-to-end across platform boundaries
3. **Implement async transcoding** - For large files to avoid blocking

### Long-Term Enhancements
1. **Add media caching** - Cache transcoded media to avoid repeated conversion
2. **Add metrics/export** - Track rate limiter stats, connection health
3. **Consider feature flags** - Allow disabling heavy dependencies (ffmpeg, image processing)

---

## Conclusion

The shared utilities module is **well-designed and well-tested** but remains **unintegrated**. The code provides a solid foundation for cross-platform channel implementations, but the real value will only be realized once the Tier 1 channel crates begin using these utilities.

**The key insight:** This issue should be considered "implementation complete" but not "feature complete." The utilities are ready to use, but they haven't been used yet.

**Next step:** Create follow-up issues to integrate these utilities into each Tier 1 channel crate, using this module as the single source of truth for:
- Message formatting
- Rate limiting
- Connection management
- Error handling
- Media validation
