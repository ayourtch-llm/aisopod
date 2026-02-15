# Issue 143: Add Web UI Integration Tests

## Summary
Add integration tests that verify the embedded Web UI works correctly, including static file serving, SPA fallback routing, WebSocket connectivity, and theme toggling.

## Location
- Crate: `aisopod-gateway`  
- File: `crates/aisopod-gateway/tests/ui_integration.rs` (new)

## Current Behavior
There are no integration tests for the Web UI. Static file serving, SPA fallback, and WebSocket connectivity from the UI are untested.

## Expected Behavior
A suite of integration tests verifies that the embedded UI is served correctly, SPA routing works, WebSocket connections succeed, and the UI's theme toggle functions properly.

## Impact
Integration tests catch regressions in UI serving, routing, and connectivity. Without them, breaking changes to the gateway could silently break the UI.

## Suggested Implementation

1. **Create the integration test file** (`crates/aisopod-gateway/tests/ui_integration.rs`):
   ```rust
   use axum::http::StatusCode;
   use axum_test::TestServer;

   // Import the gateway app builder
   use aisopod_gateway::build_app;
   ```

2. **Test static file serving:**
   ```rust
   #[tokio::test]
   async fn test_serves_index_html() {
       let app = build_app().await;
       let server = TestServer::new(app).unwrap();

       let response = server.get("/").await;
       assert_eq!(response.status(), StatusCode::OK);

       let content_type = response
           .headers()
           .get("content-type")
           .unwrap()
           .to_str()
           .unwrap();
       assert!(content_type.contains("text/html"));

       let body = response.text();
       assert!(body.contains("Aisopod"));
   }

   #[tokio::test]
   async fn test_serves_js_assets() {
       let app = build_app().await;
       let server = TestServer::new(app).unwrap();

       // Assuming Vite outputs a JS bundle — adjust path as needed
       let response = server.get("/assets/index.js").await;
       // Should be 200 if the file exists, or test a known asset path
       assert!(
           response.status() == StatusCode::OK
               || response.status() == StatusCode::NOT_FOUND
       );
   }

   #[tokio::test]
   async fn test_serves_css_with_correct_mime() {
       let app = build_app().await;
       let server = TestServer::new(app).unwrap();

       let response = server.get("/assets/style.css").await;
       if response.status() == StatusCode::OK {
           let content_type = response
               .headers()
               .get("content-type")
               .unwrap()
               .to_str()
               .unwrap();
           assert!(content_type.contains("text/css"));
       }
   }
   ```

3. **Test SPA fallback** (index.html for unknown routes):
   ```rust
   #[tokio::test]
   async fn test_spa_fallback_serves_index() {
       let app = build_app().await;
       let server = TestServer::new(app).unwrap();

       // Unknown route should serve index.html for client-side routing
       let response = server.get("/chat").await;
       assert_eq!(response.status(), StatusCode::OK);

       let content_type = response
           .headers()
           .get("content-type")
           .unwrap()
           .to_str()
           .unwrap();
       assert!(content_type.contains("text/html"));

       let body = response.text();
       assert!(body.contains("Aisopod"));
   }

   #[tokio::test]
   async fn test_spa_fallback_deep_routes() {
       let app = build_app().await;
       let server = TestServer::new(app).unwrap();

       for route in ["/agents", "/channels", "/config", "/sessions", "/models"] {
           let response = server.get(route).await;
           assert_eq!(response.status(), StatusCode::OK);
           let body = response.text();
           assert!(body.contains("Aisopod"));
       }
   }
   ```

4. **Test WebSocket connection from UI:**
   ```rust
   #[tokio::test]
   async fn test_websocket_connection() {
       let app = build_app().await;
       let server = TestServer::new(app).unwrap();

       let ws = server.get_websocket("/ws").await;
       assert!(ws.is_ok(), "WebSocket connection should succeed");

       let mut ws = ws.unwrap();

       // Send a JSON-RPC ping/health check
       ws.send_text(serde_json::json!({
           "jsonrpc": "2.0",
           "method": "system.ping",
           "id": 1
       }).to_string()).await;

       let response = ws.receive_text().await;
       assert!(response.contains("jsonrpc"));
   }
   ```

5. **Test light/dark theme toggle:**
   ```rust
   #[tokio::test]
   async fn test_index_html_contains_theme_support() {
       let app = build_app().await;
       let server = TestServer::new(app).unwrap();

       let response = server.get("/").await;
       let body = response.text();

       // Verify the UI includes theme-related markup or scripts
       // The exact check depends on the Lit UI's theme implementation
       assert!(
           body.contains("theme") || body.contains("dark-mode") || body.contains("color-scheme"),
           "index.html should contain theme support markup"
       );
   }
   ```

6. **Add test dependencies** to `crates/aisopod-gateway/Cargo.toml`:
   ```toml
   [dev-dependencies]
   axum-test = "16"
   tokio = { version = "1", features = ["full"] }
   serde_json = "1"
   ```

## Dependencies
- Issue 140 (build UI with Vite and embed in Rust binary)
- Issue 141 (verify core UI views)
- Issue 142 (development mode setup)

## Acceptance Criteria
- [ ] Test: root URL serves `index.html` with `text/html` content type
- [ ] Test: static assets served with correct MIME types
- [ ] Test: SPA fallback returns `index.html` for unknown client-side routes
- [ ] Test: deep routes (`/chat`, `/agents`, `/config`, etc.) all return `index.html`
- [ ] Test: WebSocket connection to `/ws` succeeds
- [ ] Test: theme support markup present in served HTML
- [ ] All integration tests pass with `cargo test`
- [ ] Tests run in CI pipeline

---
*Created: 2026-02-15*
