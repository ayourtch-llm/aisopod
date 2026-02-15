# Issue 142: Set Up Development Mode with Vite Hot Reload Proxy

## Summary
Configure the Vite dev server to proxy API and WebSocket requests to the aisopod gateway, add an optional build.rs trigger for UI builds, and document the development workflow.

## Location
- Crate: `aisopod-gateway`  
- File: `ui/vite.config.ts`, `crates/aisopod-gateway/build.rs`, `docs/development.md`

## Current Behavior
There is no development workflow for working on the UI with hot reload. Developers must manually rebuild the UI and restart the gateway to see changes.

## Expected Behavior
Developers can run the Vite dev server alongside the aisopod gateway for hot module replacement (HMR). API and WebSocket requests are proxied to the gateway automatically. The workflow is documented.

## Impact
A smooth development workflow dramatically speeds up UI iteration. Without this, UI development requires a full rebuild cycle for every change, which is slow and frustrating.

## Suggested Implementation

1. **Configure Vite dev server proxy** (`ui/vite.config.ts`):
   ```typescript
   import { defineConfig } from 'vite';

   export default defineConfig({
     server: {
       port: 5173,
       proxy: {
         // Proxy API requests to the aisopod gateway
         '/api': {
           target: 'http://localhost:18789',
           changeOrigin: true,
         },
         // Proxy WebSocket connections to the gateway
         '/ws': {
           target: 'ws://localhost:18789',
           ws: true,
         },
       },
     },
     build: {
       outDir: 'dist',
       emptyOutDir: true,
     },
   });
   ```

2. **Add optional cargo build.rs script** (`crates/aisopod-gateway/build.rs`):
   ```rust
   use std::process::Command;

   fn main() {
       // Only build UI if the AISOPOD_BUILD_UI env var is set
       // Skip in development mode to avoid rebuilding on every cargo build
       if std::env::var("AISOPOD_BUILD_UI").is_ok() {
           println!("cargo:rerun-if-changed=../../ui/src");
           println!("cargo:rerun-if-changed=../../ui/public");
           println!("cargo:rerun-if-changed=../../ui/index.html");

           let status = Command::new("pnpm")
               .args(["--dir", "../../ui", "build"])
               .status()
               .expect("Failed to run pnpm build. Is pnpm installed?");

           if !status.success() {
               panic!("UI build failed");
           }
       }
   }
   ```

3. **Document the development workflow** (add to `docs/development.md` or `CONTRIBUTING.md`):
   ```markdown
   ## UI Development

   ### Prerequisites
   - Node.js 20+
   - pnpm (`npm install -g pnpm`)

   ### Hot Reload Development

   Run the aisopod gateway and Vite dev server in separate terminals:

   **Terminal 1 — Gateway:**
   ```bash
   cargo run
   ```

   **Terminal 2 — Vite Dev Server:**
   ```bash
   cd ui/
   pnpm install
   pnpm dev
   ```

   Open `http://localhost:5173` in your browser. Changes to UI source
   files will hot-reload instantly. API and WebSocket requests are
   proxied to the gateway on port 18789.

   ### Production Build

   To build the UI and embed it in the binary:
   ```bash
   cd ui/ && pnpm build && cd ..
   AISOPOD_BUILD_UI=1 cargo build --release
   ```
   ```

4. **Add a `dev` script to `ui/package.json`** if not already present:
   ```json
   {
     "scripts": {
       "dev": "vite",
       "build": "vite build",
       "preview": "vite preview"
     }
   }
   ```

## Dependencies
- Issue 140 (build UI with Vite and embed in Rust binary)

## Acceptance Criteria
- [ ] Vite dev server starts on port 5173 with `pnpm dev`
- [ ] API requests from the dev server are proxied to the gateway on port 18789
- [ ] WebSocket connections are proxied correctly
- [ ] Hot module replacement (HMR) works — UI updates without full page reload
- [ ] `build.rs` optionally triggers UI build when `AISOPOD_BUILD_UI` is set
- [ ] Development workflow is documented with clear step-by-step instructions
- [ ] Production build process is documented

---
*Created: 2026-02-15*
