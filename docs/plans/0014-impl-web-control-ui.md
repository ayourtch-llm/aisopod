# 0014 — Web Control UI

**Master Plan Reference:** Section 3.12 — Web Control UI  
**Phase:** 6 (User Interface)  
**Dependencies:** 0003 (Gateway Server)

---

## Objective

Provide a web-based control interface for aisopod, either by adapting the existing
Lit-based OpenClaw UI or building a new one, served from the Rust gateway.

---

## Deliverables

### 1. Strategy: Adapt Existing Lit UI

**Recommended approach:** Port the existing Lit web component UI with rebranding.
This minimizes effort while delivering a functional UI quickly.

**Steps:**
1. Copy the `ui/` directory from OpenClaw
2. Rebrand all references from "OpenClaw" to "Aisopod"
3. Update logos, icons, and color scheme
4. Update API endpoint URLs to match aisopod gateway
5. Build the UI with Vite
6. Embed built assets in the Rust binary (via `rust-embed`)

### 2. UI Rebranding

**Visual changes:**
- Replace "OpenClaw" with "Aisopod" in all text
- New favicon and app icon (isopod-themed)
- Updated color palette
- New splash/loading screen

**Component updates:**
- `<openclaw-app>` → `<aisopod-app>` root component
- All component name prefixes updated
- API client updated for aisopod endpoints

### 3. Static File Serving from Rust

```rust
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "ui/dist"]
struct UiAssets;

// Axum handler
async fn serve_ui(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');
    match UiAssets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            (StatusCode::OK, [(header::CONTENT_TYPE, mime.as_ref())], content.data)
        }
        None => {
            // SPA fallback: serve index.html
            let index = UiAssets::get("index.html").unwrap();
            (StatusCode::OK, [(header::CONTENT_TYPE, "text/html")], index.data)
        }
    }
}
```

### 4. Key UI Views to Maintain

Port all functional views from OpenClaw:

| View           | Purpose                                    |
|----------------|--------------------------------------------|
| Chat           | Messaging interface with markdown rendering |
| Agents         | Agent management and configuration         |
| Channels       | Channel status and configuration           |
| Config         | Form-based configuration editor            |
| Sessions       | Session listing and management             |
| Models         | Model selection and status                 |
| Usage          | Token usage metrics and analytics          |
| Skills         | Skill management and status                |
| Devices        | Device pairing and management              |
| Theme          | Light/dark mode toggle                     |

### 5. WebSocket Client (UI ↔ Gateway)

The UI communicates with the gateway via WebSocket:
- Connection establishment with auth token
- JSON-RPC method calls
- Real-time event subscription (agent events, chat events)
- Reconnection logic on disconnect

### 6. Build Integration

- `npm run build` (or `pnpm build`) as part of the Rust build process
- Cargo build script (`build.rs`) to trigger UI build
- Or: Pre-build UI and embed as static assets
- Development mode: Proxy to Vite dev server for hot reload

### 7. Future: Rust-Native UI (Optional)

For a fully Rust-native UI later:
- **Leptos** — Server-side rendered + hydration
- **Dioxus** — React-like with desktop/web/mobile targets
- **Yew** — Mature Rust WASM framework
- This would replace the Lit UI in a future phase

---

## Acceptance Criteria

- [ ] UI is accessible at gateway URL (e.g., `http://localhost:18789/`)
- [ ] All branding updated from OpenClaw to Aisopod
- [ ] Chat view sends and receives messages
- [ ] Agent management view works (list, create, update, delete)
- [ ] Channel status displays correctly
- [ ] Configuration editor loads and saves config
- [ ] Session listing and management works
- [ ] Usage metrics display token consumption
- [ ] Light/dark theme toggle works
- [ ] WebSocket connection is stable with reconnection
- [ ] UI builds and embeds correctly in release binary
- [ ] Development mode with hot reload works
