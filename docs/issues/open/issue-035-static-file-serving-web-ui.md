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
- [ ] Known static files are served with correct `Content-Type` headers.
- [ ] `index.html` is returned for unknown non-API paths (SPA fallback).
- [ ] Cache headers are set appropriately for hashed and non-hashed assets.
- [ ] CORS headers allow the configured web UI origin.
- [ ] API routes take precedence over the static fallback.
- [ ] Tests confirm file serving, SPA fallback, and MIME type detection.

---
*Created: 2026-02-15*
