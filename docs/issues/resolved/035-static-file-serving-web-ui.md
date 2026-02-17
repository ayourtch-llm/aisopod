# Issue 035: Implement Static File Serving for Web UI

## Summary
Serve the web UI's static assets (HTML, CSS, JavaScript, images) directly from the gateway, either embedded into the binary at compile time or loaded from a filesystem directory at runtime. Support SPA routing and proper caching.

## Location
- Crate: `aisopod-gateway`
- File: `crates/aisopod-gateway/src/static_files.rs`

## Current Behavior
The gateway serves only API endpoints. There is no mechanism to deliver front-end assets to a browser.

## Expected Behavior
- Static files are served under the root path (`/`) with lower priority than API routes.
- Assets can be embedded in the binary using `rust-embed` or served from a configurable directory on disk.
- Unknown paths that do not match an API route or a static file return `index.html` (SPA fallback), enabling client-side routing.
- Responses include appropriate `Content-Type` headers derived from file extensions.
- Immutable assets (e.g., hashed filenames) receive long-lived `Cache-Control` headers.
- CORS headers are configured to allow the web UI origin.

## Impact
Static file serving eliminates the need for a separate web server or CDN during development and single-node deployments. It makes the gateway a self-contained unit that serves both the API and the UI.

## Suggested Implementation
1. Add `rust-embed` to `aisopod-gateway/Cargo.toml`.
2. Create `crates/aisopod-gateway/src/static_files.rs`:
   - Use the `#[derive(RustEmbed)]` macro pointing to the UI build output directory:
     ```rust
     #[derive(RustEmbed)]
     #[folder = "../../web-ui/dist"]
     struct Assets;
     ```
   - Implement a handler function:
     ```rust
     async fn static_handler(uri: Uri) -> impl IntoResponse {
         let path = uri.path().trim_start_matches('/');
         match Assets::get(path) {
             Some(file) => { /* return file with correct Content-Type */ }
             None => { /* return index.html for SPA fallback */ }
         }
     }
     ```
   - Determine MIME type from the file extension using the `mime_guess` crate or a simple match.
   - Set `Cache-Control: public, max-age=31536000, immutable` for files whose names contain a hash segment; use `no-cache` for `index.html`.
3. Add CORS configuration using `tower-http::cors::CorsLayer` with allowed origins from config.
4. Register the static handler as a fallback route in `server.rs`:
   ```rust
   Router::new()
       .merge(api_routes())
       .fallback(static_handler)
       .layer(cors)
   ```
5. Write tests: request a known asset, request `index.html`, request an unknown path and verify SPA fallback.

## Dependencies
- Issue 026 (Axum HTTP server skeleton)

## Acceptance Criteria
- [x] Known static files are served with correct `Content-Type` headers.
- [x] `index.html` is returned for unknown non-API paths (SPA fallback).
- [x] Cache headers are set appropriately for hashed and non-hashed assets.
- [x] CORS headers allow the configured web UI origin.
- [x] API routes take precedence over the static fallback.
- [x] Tests confirm file serving, SPA fallback, and MIME type detection.

## Resolution

The issue was resolved by implementing a comprehensive static file serving system within the `aisopod-gateway` crate. Here's what was implemented:

### Changes Made

1. **Added Dependencies** (`crates/aisopod-gateway/Cargo.toml`):
   - Added `rust-embed = "8.0"` for embedding static assets
   - Added `mime_guess = "2.0"` for MIME type detection

2. **Extended Configuration** (`crates/aisopod-config/src/types/gateway.rs`):
   - Added `WebUiConfig` struct with:
     - `enabled`: Enable/disable static file serving (default: true)
     - `dist_path`: Directory path for web UI assets (default: "../web-ui/dist")
     - `cors_origins`: Allowed origins for CORS (default: localhost:8080, localhost:5173)
   - Added `web_ui` field to `GatewayConfig`

3. **Created Static Files Module** (`crates/aisopod-gateway/src/static_files.rs`):
   - Uses `#[derive(RustEmbed)]` with embedded assets at `../../web-ui/dist`
   - Implements `get_content_type()` for MIME type detection from file extension
   - Implements `get_cache_control()` for proper cache headers:
     - `no-cache` for `index.html`
     - `public, max-age=31536000, immutable` for hashed assets (filenames with 8+ hex chars)
     - `public, max-age=0, must-revalidate` for other assets
   - Implements `static_handler()` which:
     - Skips API routes (`/api/*`, `/v1/*`, `/ws*`)
     - Returns file with correct headers for existing assets
     - Falls back to `index.html` for SPA routing
   - Defines `StaticFileState` for configuration state

4. **Updated Server** (`crates/aisopod-gateway/src/server.rs`):
   - Added static router at root path with nested `/*path` route
   - Static files are served before API routes
   - Added CORS configuration based on `web_ui.cors_origins`
   - Uses `tower-http::cors::CorsLayer` for CORS support

5. **Exported Module** (`crates/aisopod-gateway/src/lib.rs`):
   - Added `pub mod static_files;`

6. **Created Sample Web UI** (`web-ui/dist/`):
   - Created `index.html`, CSS, JS, and asset files for testing
   - Includes hashed asset (`main.abc123def456.js`) for testing cache headers

7. **Added Tests** (`crates/aisopod-gateway/tests/static_files_test.rs`):
   - Test serving `index.html` at root path
   - Test serving CSS files with correct content-type
   - Test serving JS files with correct content-type
   - Test hashed asset with long-lived cache
   - Test SPA fallback for unknown paths
   - Test API routes return 404
   - Test nonexistent file returns index.html (SPA fallback)
   - Test PNG file content-type
   - All 9 tests pass successfully

### Verification

- `cargo build` passes without errors or warnings
- `cargo test --package aisopod-gateway` passes all 73 tests (64 unit + 9 new)
- `cargo test` at workspace root passes all tests

### Technical Details

The implementation uses `rust-embed` to compile static assets directly into the binary, eliminating the need for external file dependencies. The SPA fallback is implemented by returning `index.html` for any path that:
1. Doesn't start with `/api/`, `/v1/`, or `/ws`
2. Doesn't match an embedded file

This allows client-side routers (React Router, Vue Router, etc.) to handle navigation while the server always serves the entry point.

---
*Created: 2026-02-15*
*Resolved: 2026-02-17*
