# Issue 184: Implement Shared Tier 2 & 3 Channel Utilities

## Summary
Create shared utility modules for Tier 2 and Tier 3 channel implementations, including rate limit handling, markdown format conversion between platforms, media format conversion, connection retry with exponential backoff, and platform-specific error mapping.

## Location
- Crate: `aisopod-channel-utils` (extend existing from Tier 1)
- File: `crates/aisopod-channel-utils/src/lib.rs`

## Current Behavior
Basic shared utilities exist from Tier 1 channel implementation (Issue 105). However, Tier 2 and Tier 3 channels require additional cross-platform utilities that are not yet implemented, such as multi-format markdown conversion and platform-specific rate limiting.

## Expected Behavior
After implementation:
- Rate limit handling adapts to each platform's limits and headers.
- Markdown conversion translates between Discord, Slack, Telegram, HTML, and plain text formats.
- Media format conversion handles platform-specific constraints.
- Connection retry with exponential backoff is shared across all channels.
- Platform-specific errors map to a common ChannelError type.

## Impact
Reduces code duplication across 13+ channel implementations, ensures consistent behavior for error handling and rate limiting, and provides battle-tested utilities that community channel developers can also use.

## Suggested Implementation

1. **Extend crate structure:**
   ```
   crates/aisopod-channel-utils/src/
   ├── lib.rs
   ├── rate_limit.rs      (new)
   ├── markdown.rs         (new)
   ├── media.rs            (new)
   ├── retry.rs            (extend existing)
   └── error_mapping.rs    (new)
   ```

2. **Rate limit handling** in `rate_limit.rs`:
   ```rust
   use std::collections::HashMap;
   use std::time::{Duration, Instant};

   /// Platform-specific rate limiter.
   pub struct RateLimiter {
       /// Max requests per window
       max_requests: u32,
       /// Time window duration
       window: Duration,
       /// Current request count per key
       counters: HashMap<String, (u32, Instant)>,
   }

   impl RateLimiter {
       pub fn new(max_requests: u32, window: Duration) -> Self {
           Self {
               max_requests,
               window,
               counters: HashMap::new(),
           }
       }

       /// Predefined rate limiters for known platforms.
       pub fn for_platform(platform: &str) -> Self {
           match platform {
               "signal" => Self::new(60, Duration::from_secs(60)),
               "googlechat" => Self::new(60, Duration::from_secs(60)),
               "msteams" => Self::new(50, Duration::from_secs(60)),
               "matrix" => Self::new(30, Duration::from_secs(60)),
               "irc" => Self::new(5, Duration::from_secs(10)),
               "mattermost" => Self::new(60, Duration::from_secs(60)),
               "twitch" => Self::new(20, Duration::from_secs(30)),
               "line" => Self::new(500, Duration::from_secs(60)),
               "lark" => Self::new(100, Duration::from_secs(60)),
               "zalo" => Self::new(200, Duration::from_secs(60)),
               _ => Self::new(30, Duration::from_secs(60)), // conservative default
           }
       }

       /// Check if a request can proceed, or return wait duration.
       pub fn check(&mut self, key: &str) -> RateLimitResult {
           let now = Instant::now();
           let entry = self.counters.entry(key.to_string()).or_insert((0, now));

           if now.duration_since(entry.1) >= self.window {
               *entry = (1, now);
               return RateLimitResult::Allowed;
           }

           if entry.0 < self.max_requests {
               entry.0 += 1;
               RateLimitResult::Allowed
           } else {
               let wait = self.window - now.duration_since(entry.1);
               RateLimitResult::Limited { retry_after: wait }
           }
       }

       /// Parse rate limit headers from HTTP responses.
       pub fn update_from_headers(&mut self, key: &str, headers: &reqwest::header::HeaderMap) {
           // Parse X-RateLimit-Remaining, X-RateLimit-Reset, Retry-After
           // Adjust internal counters accordingly
           if let Some(remaining) = headers.get("x-ratelimit-remaining") {
               // Update counter
           }
           if let Some(retry_after) = headers.get("retry-after") {
               // Set backoff timer
           }
       }
   }

   pub enum RateLimitResult {
       Allowed,
       Limited { retry_after: Duration },
   }
   ```

3. **Markdown format conversion** in `markdown.rs`:
   ```rust
   /// Target format for markdown conversion.
   pub enum MarkdownFormat {
       Discord,   // **bold**, *italic*, ~~strike~~, `code`, ```block```
       Slack,     // *bold*, _italic_, ~strike~, `code`, ```block```
       Telegram,  // **bold**, __italic__, ~~strike~~, `code`, ```block```
       Html,      // <b>, <i>, <s>, <code>, <pre>
       Plain,     // strip all formatting
       Matrix,    // Standard markdown + HTML subset
       IRC,       // mIRC color codes: \x02bold\x02, \x1Ditalic\x1D
   }

   pub fn convert(input: &str, from: MarkdownFormat, to: MarkdownFormat) -> String {
       // Step 1: Parse input to intermediate AST
       let ast = parse_markdown(input, &from);
       // Step 2: Render AST to target format
       render_markdown(&ast, &to)
   }

   #[derive(Debug)]
   enum MarkdownNode {
       Text(String),
       Bold(Vec<MarkdownNode>),
       Italic(Vec<MarkdownNode>),
       Strikethrough(Vec<MarkdownNode>),
       Code(String),
       CodeBlock { language: Option<String>, code: String },
       Link { text: String, url: String },
       Newline,
   }

   fn parse_markdown(input: &str, format: &MarkdownFormat) -> Vec<MarkdownNode> {
       // Parse based on source format rules
       todo!()
   }

   fn render_markdown(nodes: &[MarkdownNode], format: &MarkdownFormat) -> String {
       let mut output = String::new();
       for node in nodes {
           match (node, format) {
               (MarkdownNode::Bold(children), MarkdownFormat::Discord) => {
                   output.push_str("**");
                   output.push_str(&render_markdown(children, format));
                   output.push_str("**");
               }
               (MarkdownNode::Bold(children), MarkdownFormat::Slack) => {
                   output.push('*');
                   output.push_str(&render_markdown(children, format));
                   output.push('*');
               }
               (MarkdownNode::Bold(children), MarkdownFormat::Html) => {
                   output.push_str("<b>");
                   output.push_str(&render_markdown(children, format));
                   output.push_str("</b>");
               }
               // ... handle all combinations
               _ => todo!()
           }
       }
       output
   }
   ```

4. **Media format conversion** in `media.rs`:
   ```rust
   /// Platform media constraints.
   pub struct MediaConstraints {
       pub max_image_size_bytes: usize,
       pub max_file_size_bytes: usize,
       pub supported_image_formats: Vec<String>,
       pub max_image_dimensions: Option<(u32, u32)>,
   }

   impl MediaConstraints {
       pub fn for_platform(platform: &str) -> Self {
           match platform {
               "signal" => Self {
                   max_image_size_bytes: 100 * 1024 * 1024,
                   max_file_size_bytes: 100 * 1024 * 1024,
                   supported_image_formats: vec!["png", "jpg", "gif", "webp"].into_iter().map(String::from).collect(),
                   max_image_dimensions: None,
               },
               "line" => Self {
                   max_image_size_bytes: 10 * 1024 * 1024,
                   max_file_size_bytes: 200 * 1024 * 1024,
                   supported_image_formats: vec!["png", "jpg"].into_iter().map(String::from).collect(),
                   max_image_dimensions: Some((4096, 4096)),
               },
               // ... other platforms
               _ => Self::default(),
           }
       }
   }

   impl Default for MediaConstraints {
       fn default() -> Self {
           Self {
               max_image_size_bytes: 10 * 1024 * 1024,
               max_file_size_bytes: 50 * 1024 * 1024,
               supported_image_formats: vec!["png".into(), "jpg".into()],
               max_image_dimensions: None,
           }
       }
   }

   /// Validate media against platform constraints.
   pub fn validate_media(
       file_path: &str,
       file_size: usize,
       constraints: &MediaConstraints,
   ) -> Result<(), MediaError> {
       // Check file size, format, dimensions
       todo!()
   }
   ```

5. **Enhanced retry with exponential backoff** in `retry.rs`:
   ```rust
   use std::time::Duration;

   pub struct RetryConfig {
       pub max_retries: u32,
       pub initial_delay: Duration,
       pub max_delay: Duration,
       pub multiplier: f64,
       pub jitter: bool,
   }

   impl Default for RetryConfig {
       fn default() -> Self {
           Self {
               max_retries: 5,
               initial_delay: Duration::from_millis(500),
               max_delay: Duration::from_secs(30),
               multiplier: 2.0,
               jitter: true,
           }
       }
   }

   pub async fn retry_with_backoff<F, Fut, T, E>(
       config: &RetryConfig,
       mut operation: F,
   ) -> Result<T, E>
   where
       F: FnMut() -> Fut,
       Fut: std::future::Future<Output = Result<T, E>>,
       E: std::fmt::Display,
   {
       let mut delay = config.initial_delay;
       for attempt in 0..=config.max_retries {
           match operation().await {
               Ok(result) => return Ok(result),
               Err(e) if attempt == config.max_retries => return Err(e),
               Err(e) => {
                   tracing::warn!("Attempt {} failed: {}. Retrying in {:?}", attempt + 1, e, delay);
                   tokio::time::sleep(delay).await;
                   delay = Duration::from_secs_f64(
                       (delay.as_secs_f64() * config.multiplier).min(config.max_delay.as_secs_f64())
                   );
                   if config.jitter {
                       // Add random jitter up to 25% of delay
                   }
               }
           }
       }
       unreachable!()
   }
   ```

6. **Platform error mapping** in `error_mapping.rs`:
   ```rust
   use aisopod_channel_core::ChannelError;

   /// Map HTTP status codes to ChannelError.
   pub fn map_http_error(status: reqwest::StatusCode, body: &str, platform: &str) -> ChannelError {
       match status.as_u16() {
           401 => ChannelError::Authentication(format!("{}: unauthorized - {}", platform, body)),
           403 => ChannelError::Permission(format!("{}: forbidden - {}", platform, body)),
           404 => ChannelError::NotFound(format!("{}: resource not found - {}", platform, body)),
           429 => ChannelError::RateLimited(format!("{}: rate limited - {}", platform, body)),
           500..=599 => ChannelError::ServerError(format!("{}: server error {} - {}", platform, status, body)),
           _ => ChannelError::Unknown(format!("{}: HTTP {} - {}", platform, status, body)),
       }
   }
   ```

## Dependencies
- Issue 105: Shared Tier 1 channel utilities (base utilities to extend)

## Acceptance Criteria
- [ ] Rate limiter works for all Tier 2 and Tier 3 platforms
- [ ] Rate limiter parses HTTP rate limit headers
- [ ] Markdown converts correctly between Discord, Slack, Telegram, HTML, plain, Matrix, and IRC formats
- [ ] Media validation checks against platform-specific constraints
- [ ] Retry with exponential backoff handles transient failures
- [ ] Platform error mapping covers common HTTP error codes
- [ ] All utilities are used by at least one Tier 2/3 channel implementation
- [ ] Unit tests for each utility module with edge cases
- [ ] Documentation with usage examples

---
*Created: 2026-02-15*
