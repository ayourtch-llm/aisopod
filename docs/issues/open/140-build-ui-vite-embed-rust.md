# Issue 140: Build UI with Vite and Embed in Rust Binary

## Summary
Set up the Vite build pipeline for the rebranded UI and use the `rust-embed` crate to embed the built `ui/dist/` assets into the aisopod Rust binary, integrating with the Axum static file serving handler.

## Location
- Crate: `aisopod-gateway`  
- File: `crates/aisopod-gateway/src/ui.rs` (new), `ui/vite.config.ts`, `ui/package.json`

## Current Behavior
The UI source exists in `ui/` but there is no build integration with the Rust binary. The gateway does not serve any static files for the web UI.

## Expected Behavior
Running `cargo build` produces a binary that includes the compiled UI assets. The gateway serves the UI at its root URL (e.g., `http://localhost:18789/`). SPA fallback serves `index.html` for unknown routes.

## Impact
This is the integration point that makes the UI actually accessible to users. Without this, the UI is just source code that cannot be served.

## Suggested Implementation

1. **Verify Vite build works:**
   ```bash
   cd ui/
   pnpm install
   pnpm build
   # Output should appear in ui/dist/
   ls ui/dist/
   ```

2. **Add `rust-embed` dependency to the gateway crate:**
   ```toml
   # crates/aisopod-gateway/Cargo.toml
   [dependencies]
   rust-embed = "8"
   mime_guess = "2"
   ```

3. **Create the UI asset embedding module** (`crates/aisopod-gateway/src/ui.rs`):
   ```rust
   use axum::{
       http::{header, StatusCode, Uri},
       response::IntoResponse,
   };
   use rust_embed::Embed;

   #[derive(Embed)]
   #[folder = "ui/dist"]
   struct UiAssets;

   pub async fn serve_ui(uri: Uri) -> impl IntoResponse {
       let path = uri.path().trim_start_matches('/');

       // Try to serve the requested file
       if let Some(content) = UiAssets::get(path) {
           let mime = mime_guess::from_path(path).first_or_octet_stream();
           return (
               StatusCode::OK,
               [(header::CONTENT_TYPE, mime.as_ref().to_string())],
               content.data.into(),
           );
       }

       // SPA fallback: serve index.html for unknown routes
       match UiAssets::get("index.html") {
           Some(index) => (
               StatusCode::OK,
               [(header::CONTENT_TYPE, "text/html".to_string())],
               index.data.into(),
           ),
           None => (
               StatusCode::NOT_FOUND,
               [(header::CONTENT_TYPE, "text/plain".to_string())],
               "UI not found".as_bytes().into(),
           ),
       }
   }
   ```

4. **Register the UI handler in the Axum router** (integrate with issue 035 static file serving):
   ```rust
   // In the gateway router setup
   use crate::ui::serve_ui;

   let app = Router::new()
       .route("/api/*path", api_routes())
       .route("/ws", ws_handler())
       .fallback(serve_ui);  // Serve UI for all other routes
   ```

5. **Add `mod ui;`** to the gateway's `lib.rs` or `main.rs`.

6. **Optional: Add `build.rs`** to trigger UI build during `cargo build`:
   ```rust
   // crates/aisopod-gateway/build.rs
   use std::process::Command;

   fn main() {
       println!("cargo:rerun-if-changed=ui/src");
       println!("cargo:rerun-if-changed=ui/public");

       let status = Command::new("pnpm")
           .args(["--dir", "../../ui", "build"])
           .status()
           .expect("Failed to build UI");

       if !status.success() {
           panic!("UI build failed");
       }
   }
   ```

## Dependencies
- Issue 137 (copy and rebrand UI)
- Issue 138 (update API endpoints)
- Issue 139 (update visual assets)
- Issue 035 (static file serving handler)

## Acceptance Criteria
- [ ] `pnpm build` in `ui/` produces `ui/dist/` with all assets
- [ ] `rust-embed` is added to gateway crate dependencies
- [ ] UI assets are embedded in the compiled aisopod binary
- [ ] Gateway serves `index.html` at root URL
- [ ] SPA fallback works (unknown routes serve `index.html`)
- [ ] Static assets (JS, CSS, images) served with correct MIME types
- [ ] `cargo build` completes successfully with embedded UI

---
*Created: 2026-02-15*
