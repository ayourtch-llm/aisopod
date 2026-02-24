# Web UI Integration Tests - Learnings

## Issue Reference
Issue #143: Add Web UI Integration Tests

## Implementation Summary

This implementation adds comprehensive integration tests for the embedded Web UI using `axum-test` and `axum::Router`.

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

## Conclusion

The key insight is that Axum applications should be designed with testability in mind - separating the app building logic from the server running logic makes testing much easier. The `build_app()` pattern allows tests to use the same app configuration as production, ensuring tests are meaningful and comprehensive.
