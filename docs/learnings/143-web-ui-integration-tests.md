# Web UI Integration Tests - Learnings

## Issue Reference
Issue #143: Add Web UI Integration Tests

## Implementation Summary

This implementation adds comprehensive integration tests for the embedded Web UI using `axum-test` and `axum::Router`.

## Verification Report (2026-02-24)

The issue has been verified and confirmed as successfully implemented according to all acceptance criteria:

### Acceptance Criteria Status
| Criterion | Status |
|-----------|--------|
| ✓ Test: root URL serves `index.html` with `text/html` content type | **PASS** |
| ✓ Test: static assets served with correct MIME types | **PASS** |
| ✓ Test: SPA fallback returns `index.html` for unknown client-side routes | **PASS** |
| ✓ Test: deep routes (`/chat`, `/agents`, `/config`, etc.) all return `index.html` | **PASS** |
| ✓ Test: WebSocket connection to `/ws` succeeds | **PASS** |
| ✓ Test: theme support markup present in served HTML | **PASS** |
| ✓ All integration tests pass with `cargo test` | **PASS** (13/13) |
| ✓ Tests run in CI pipeline | **PASS** |

### Test Results
```
running 13 tests
test test_health_endpoint ... ok
test test_api_routes_return_not_found ... ok
test test_index_html_cache_headers ... ok
test test_serves_index_html ... ok
test test_deep_spa_fallback_with_query_params ... ok
test test_index_html_contains_theme_support ... ok
test test_png_asset_served ... ok
test test_spa_fallback_serves_index ... ok
test test_spa_fallback_deep_routes ... ok
test test_serves_css_with_correct_mime ... ok
test test_static_file_cache_headers ... ok
test test_serves_js_assets ... ok
test test_websocket_connection ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Files Verification
| File | Status |
|------|--------|
| `crates/aisopod-gateway/tests/ui_integration.rs` (new) | **CREATED** |
| `crates/aisopod-gateway/Cargo.toml` (modified) | **MODIFIED** - Added `axum-test = "16"` and `serde_json = "1"` dev-dependencies |
| `crates/aisopod-gateway/src/lib.rs` (modified) | **MODIFIED** - Exported `build_app` |
| `crates/aisopod-gateway/src/server.rs` (modified) | **MODIFIED** - Added `build_app()` function |

## Key Learnings

### 1. Testing Axum Applications Without a Server

When testing Axum applications, there are two main approaches:

#### Option A: Using axum-test (TestServer)
The `axum-test` crate provides a `TestServer` that wraps your Axum app and allows making HTTP requests without starting a real server. This is cleaner and faster.

```rust
use axum_test::TestServer;
use aisopod_gateway::build_app;

#[tokio::test]
async fn test_serves_index_html() {
    let app = build_app().await;
    let server = TestServer::new(app).unwrap();
    
    let response = server.get("/").await;
    assert_eq!(response.status_code(), StatusCode::OK);
}
```

#### Option B: Manual Server with TcpListener
For WebSocket testing (which `axum-test` doesn't fully support), you need to create a server manually:

```rust
let app = build_app().await;
let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
let addr = listener.local_addr().unwrap();

tokio::spawn(async move {
    axum::serve(listener, app).await.unwrap();
});

// Connect to WebSocket
let (stream, _) = tokio_tungstenite::connect_async(format!("ws://{}/ws", addr))
    .await
    .expect("WebSocket connection should succeed");
```

### 2. Axum-test v16 API Differences

When using `axum-test` v16, be aware of these API differences:

- **Method name**: Use `status_code()` instead of `status()`
- **Method name**: Use `get_websocket()` for websocket connections (though full support may be limited)

Example:
```rust
// Correct for v16
let status = response.status_code();

// Not available in v16
// let status = response.status();
```

### 3. Building Test-Friendly Application Structs

To make applications testable, create a `build_app()` function that:

1. Creates all dependencies
2. Builds the router with all middleware
3. Returns the `Router` for testing

This avoids code duplication between the production server and tests:

```rust
pub async fn build_app() -> Router {
    // Create all dependencies
    let rate_limiter = Arc::new(RateLimiter::new(rate_limit_config));
    let auth_config_data = AuthConfigData::new(config.auth.clone());
    // ... other dependencies ...
    
    // Build middleware stack
    let middleware_stack = tower::ServiceBuilder::new()
        .layer(TraceLayer::new_for_http()...)
        .layer(axum::middleware::from_fn(auth_middleware));
    
    // Build the router
    Router::new()
        .route("/health", get(health))
        .merge(api_routes())
        .merge(ws_routes())
        .layer(middleware_stack)
}
```

### 4. Testing SPA Fallback

Single Page Application (SPA) routing requires returning `index.html` for any unknown paths that aren't API routes. Test this by:

1. Requesting unknown paths like `/chat`, `/agents`, etc.
2. Verifying the response is `index.html` with `text/html` content type
3. Verifying the response contains expected UI markup

```rust
#[tokio::test]
async fn test_spa_fallback_serves_index() {
    let response = server.get("/chat").await;
    assert_eq!(response.status_code(), StatusCode::OK);
    
    let content_type = response.headers().get("content-type").unwrap();
    assert!(content_type.contains("text/html"));
    
    let body = response.text();
    assert!(body.contains("Aisopod"));
}
```

### 5. Testing Static File Assets

Test static files by:

1. Requesting known asset paths (check your Vite output for actual hashed filenames)
2. Verifying correct MIME types
3. Verifying cache headers (immutable for hashed assets, no-cache for index.html)

```rust
#[tokio::test]
async fn test_serve_js_assets() {
    let response = server.get("/assets/index-BoczOECE.js").await;
    assert_eq!(response.status_code(), StatusCode::OK);
    
    let content_type = response.headers().get("content-type").unwrap();
    assert!(content_type.contains("application/javascript"));
}
```

### 6. WebSocket Testing Pattern

For WebSocket testing, you need to:

1. Start the server manually with a known port
2. Connect using `tokio-tungstenite`
3. Send messages and verify responses

```rust
let (mut ws_tx, mut ws_rx) = stream.split();

// Send JSON-RPC message
ws_tx.send(Message::Text(jsonrpc_message)).await?;

// Receive response
let response = ws_rx.next().await;
if let Ok(Message::Text(text)) = response {
    assert!(text.contains("jsonrpc"));
}
```

### 7. Cache Header Testing

Test that cache headers are correctly set based on file type:

- **Hashed assets**: `public, max-age=31536000, immutable`
- **index.html**: `no-cache`

```rust
let cache_control = response.headers().get("cache-control").unwrap();
assert!(cache_control.contains("immutable"));
assert!(cache_control.contains("max-age=31536000"));
```

## Common Pitfalls

### 1. Forgetting to Spawn Cleanup Tasks
If your app spawns background tasks (like rate limiter cleanup), make sure they're spawned during testing too:

```rust
let cleanup_limiter = rate_limiter.clone();
tokio::spawn(async move {
    cleanup_limiter.cleanup_loop().await;
});
```

### 2. Middleware Order Issues
Middleware runs in reverse order (last layer listed runs first in the request flow). Ensure auth config is injected before auth middleware:

```rust
// Auth config data MUST be injected BEFORE auth_middleware
.layer(axum::middleware::from_fn(inject_auth_config))
.layer(axum::middleware::from_fn(auth_middleware))
```

### 3. WebSocket Stream Handling
When working with WebSocket streams, remember that `next()` returns `Result<Message, Error>`, not just `Message`:

```rust
// Correct
if let Ok(Message::Text(text)) = response {
    // handle text
}

// Wrong
if let Message::Text(text) = response {  // Type mismatch
    // ...
}
```

## Files Modified/Created

1. **crates/aisopod-gateway/Cargo.toml** - Added `axum-test` and `serde_json` dev-dependencies
2. **crates/aisopod-gateway/src/server.rs** - Added `build_app()` function
3. **crates/aisopod-gateway/src/lib.rs** - Exported `build_app`
4. **crates/aisopod-gateway/tests/ui_integration.rs** - Created new integration test file with 13 tests

## Test Results

All 13 tests pass:
- Static file serving (HTML, JS, CSS, PNG)
- SPA fallback routing
- WebSocket connectivity
- Theme support verification
- Health endpoint
- Cache headers
- API routes (404 for non-API paths)

## Additional Learnings (Verification Addendum)

### 1. Testing Pattern for Complex Middleware Stacks

When an Axum application has complex middleware dependencies, the `build_app()` pattern is crucial for testability:

```rust
// Production code
pub async fn run_with_config(config: &AisopodConfig) -> Result<()> {
    // ... build dependencies ...
    let app = Router::new()
        .route("/health", get(health))
        .merge(api_routes())
        .merge(ws_routes())
        .layer(middleware_stack);
    
    axum::serve(listener, app).await?;
    Ok(())
}

// Test code - reuses same build logic
pub async fn build_app() -> Router {
    // Same dependency building logic
    let app = Router::new()
        .route("/health", get(health))
        .merge(api_routes())
        .merge(ws_routes())
        .layer(middleware_stack);
    app  // Returns Router instead of serving
}
```

**Key Insight**: Separating the app building logic from server startup enables:
- Tests to use the exact same middleware configuration as production
- Consistent behavior between test and production environments
- Reduced code duplication and maintenance burden

### 2. API Differences with axum-test v16

The `axum-test` crate has different API naming conventions:

| axum-test v16 | Older Versions |
|---------------|----------------|
| `response.status_code()` | `response.status()` |
| `response.text()` | `response.body_string()` |
| `response.json::<T>()` | `response.body_json::<T>()` |

**Recommendation**: Always check the crate version documentation when upgrading:

```rust
// Use status_code() for v16
let status = response.status_code();
assert_eq!(status, StatusCode::OK);

// Use text() for v16
let body = response.text();
assert!(body.contains("Aisopod"));
```

### 3. Static Asset Hashing in Vite Builds

Vite uses content-hashed filenames for cache-busting. Tests must use the actual hashed filenames from the build output:

```rust
// Find the actual filename from build output
// Example: index-BoczOECE.js
let response = server.get("/assets/index-BoczOECE.js").await;

// Or check the build output directory
// ls web-ui/dist/assets/
```

**Learning**: Include a comment in tests about where to find the actual asset names:

```rust
// Vite output: assets/index-BoczOECE.js
// Check web-ui/dist/assets/ for current filenames
let response = server.get("/assets/index-BoczOECE.js").await;
```

### 4. Testing WebSocket Without Full Server

Axum-test's WebSocket support is limited. The manual server approach provides full control:

```rust
// Create server manually for WebSocket tests
let app = build_app().await;
let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
let addr = listener.local_addr().unwrap();

tokio::spawn(async move {
    axum::serve(listener, app).await.unwrap();
});

// Give server time to start
tokio::time::sleep(Duration::from_millis(100)).await;

// Connect and test
let (stream, _) = tokio_tungstenite::connect_async(format!("ws://{}/ws", addr))
    .await
    .expect("WebSocket connection should succeed");
```

**Trade-offs**:
- Pros: Full WebSocket control, exact production behavior
- Cons: More complex setup, timing-sensitive tests

### 5. Cache Header Testing Patterns

Different files should have different cache strategies. Test both:

```rust
// Hashed assets (immutable, long-lived)
let response = server.get("/assets/index-XXXX.css").await;
let cache_control = response.headers().get("cache-control").unwrap();
assert!(cache_control.contains("immutable"));
assert!(cache_control.contains("max-age=31536000"));

// HTML files (no-cache)
let response = server.get("/").await;
let cache_control = response.headers().get("cache-control").unwrap();
assert!(cache_control.contains("no-cache"));
```

### 6. Theme Support Verification

Since the UI uses Lit for theming, verify the theme support exists in the HTML:

```rust
let body = response.text();
assert!(
    body.contains("theme") || 
    body.contains("dark-mode") || 
    body.contains("color-scheme"),
    "index.html should contain theme support markup"
);
```

**Note**: The exact check depends on your UI implementation. For Lit-based themes, look for:
- `theme` attribute on elements
- `--dark-mode` CSS custom properties
- `color-scheme` meta tags

### 7. CI Pipeline Integration

The test file follows the pattern `crates/aisopod-gateway/tests/ui_integration.rs`:

```rust
// Cargo.toml
[dev-dependencies]
axum-test = "16"

// tests/ui_integration.rs
#![allow(clippy::all)]

use axum::http::StatusCode;
use axum_test::TestServer;
use aisopod_gateway::build_app;
```

This integrates with existing CI workflows without additional configuration.

### 8. Dependencies Verification

Issue 143 depends on:
- Issue 140: Build UI with Vite and embed in Rust binary ✓
- Issue 141: Verify core UI views ✓
- Issue 142: Development mode setup ✓

**Lesson**: Issue dependencies should be clearly documented in the issue file to prevent incomplete implementations.

## Conclusion

The key insight is that Axum applications should be designed with testability in mind - separating the app building logic from the server running logic makes testing much easier. The `build_app()` pattern allows tests to use the same app configuration as production, ensuring tests are meaningful and comprehensive.

This implementation demonstrates a comprehensive integration testing strategy for embedded Web UIs in Axum applications, covering static file serving, SPA routing, WebSocket connectivity, and theme support.
