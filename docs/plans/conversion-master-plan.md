# Aisopod Conversion Master Plan

## From OpenClaw (TypeScript/Node.js) to Aisopod (Rust)

This document outlines the high-level conversion plan from the legacy OpenClaw project
to the new Rust-based **aisopod** project. The goal is to retain all functionality while
leveraging Rust's performance, safety, and reliability advantages.

---

## 1. Naming & Branding

- **Project name:** aisopod (from "isopod" — resilient crustaceans + "ai")
- **CLI binary:** `aisopod`
- **Config file:** `~/.aisopod/aisopod.json` (or `.toml`)
- **UI title:** "Aisopod" with isopod-themed branding
- **Package/crate name:** `aisopod`
- **Environment variable prefix:** `AISOPOD_`

---

## 2. Technology Stack

| Area                | OpenClaw (Legacy)           | Aisopod (Target)                      |
|---------------------|-----------------------------|---------------------------------------|
| Language            | TypeScript / Node.js ≥ 22   | Rust (stable)                         |
| HTTP server         | Express                     | Axum                                  |
| WebSocket           | ws library                  | tokio-tungstenite / axum WS           |
| Async runtime       | Node.js event loop          | Tokio                                 |
| Serialization       | JSON (custom)               | serde / serde_json                    |
| Schema validation   | Zod                         | serde + custom validation             |
| Config format       | JSON5                       | TOML primary, JSON5 for compat        |
| Database            | SQLite-Vec                  | rusqlite + sqlite-vec                  |
| UI framework        | Lit web components + Vite   | Leptos, Dioxus, or Yew (or keep Lit)  |
| CLI framework       | Custom commander-like       | clap                                  |
| Testing             | Vitest                      | Rust built-in tests + integration     |
| Build               | tsdown / pnpm               | Cargo workspace                       |
| Linting             | Oxlint                      | clippy                                |
| Formatting          | Oxfmt                       | rustfmt                               |
| Image processing    | Sharp                       | image crate                           |
| PDF processing      | PDF.js                      | pdf-extract or lopdf                  |

---

## 3. High-Level Conversion Areas

### 3.1 Project Structure & Build System
- Set up Cargo workspace with crates for each major subsystem
- Establish CI/CD pipeline (GitHub Actions)
- Configure linting (clippy), formatting (rustfmt), and testing
- Create workspace layout mirroring the logical module separation

### 3.2 Configuration System
- Port the Zod-based schema to Rust structs with serde deserialization
- Support JSON5 (or TOML) config format
- Implement environment variable substitution
- Implement `@include` directive support for modular configs
- Implement config validation with detailed error messages
- Support legacy config migration from OpenClaw format
- Sensitive field marking for UI redaction

### 3.3 Gateway Server (HTTP + WebSocket)
- Implement HTTP server with Axum
- REST endpoints: `/v1/chat/completions`, `/v1/responses`, `/hooks`, `/tools/invoke`
- WebSocket upgrade and JSON-RPC 2.0 message handling
- Role-based authentication (token, password, device token, none)
- Rate limiting per IP
- Client connection management with presence tracking
- Event broadcasting to subscribed clients
- Health endpoint and status reporting
- TLS/HTTPS support

### 3.4 Agent Execution Engine
- Port agent lifecycle management (create, run, abort)
- Implement streaming execution with async channels
- Model selection with primary → fallback chain
- Auth profile rotation (round-robin + cooldown)
- Session compaction (adaptive chunking, summary, hard-clear)
- Context window guard (warn/fail thresholds)
- Subagent spawning with depth limits

### 3.5 AI Model Provider Integrations
- Implement provider abstraction trait
- Anthropic Claude API (streaming chat completions)
- OpenAI API (streaming chat completions, compatible endpoint)
- Google Gemini API
- AWS Bedrock API
- Ollama (local models)
- OAuth flow implementations for provider auth
- API key management and rotation

### 3.6 Tool System
- Define tool trait/interface for extensible tool registration
- Port built-in tools:
  - Bash/shell execution with approval workflow
  - File operations (read, write, search)
  - Message sending to channels
  - Subagent spawning
  - Session management
  - Cron/scheduled tasks
  - Canvas/visual output
  - Channel-specific actions
- Tool policy enforcement (allow/deny lists)

### 3.7 Channel Abstraction & Messaging
- Define channel plugin trait with adapter interfaces
- Port channel registry with metadata and capabilities
- Implement account management (multi-account per channel)
- Message routing (agent bindings → channel routing)
- Typing indicators, reactions, threading support
- Media handling (images, audio, documents)

### 3.8 Channel Extensions
- **Tier 1** (most common): Telegram, Discord, WhatsApp, Slack
- **Tier 2**: Signal, iMessage, Google Chat, MS Teams
- **Tier 3**: Matrix, IRC, Mattermost, Nextcloud Talk, Twitch, Nostr, LINE, Lark/Feishu, Zalo
- Each channel as a separate crate or feature-gated module
- Use existing Rust libraries where available (teloxide for Telegram, serenity for Discord, etc.)

### 3.9 Plugin System
- Define plugin trait with lifecycle hooks
- Plugin manifest format (aisopod.plugin.json or .toml)
- Dynamic plugin loading (shared libraries or WASM)
- Plugin-contributed CLI commands with security hardening
- Plugin-contributed agent tools
- Hook system: pre/post agent, message, session, gateway events

### 3.10 Skills System
- Port skill module interface
- Implement skill discovery and loading
- Port essential skills (or define how community skills will work)
- Skill creator tooling

### 3.11 CLI Application
- Implement using clap with subcommand structure
- Port key command groups:
  - Agent management (add, delete, list, identity)
  - Authentication and credential management
  - Configuration wizard and setup
  - Status, health, dashboard, diagnostics
  - Sandbox management
  - Model selection and management
  - Channel configuration
  - Daemon management (systemd/launchctl)
  - Gateway control
  - Session management
  - Message sending

### 3.12 Web Control UI
- Option A: Keep Lit-based UI, serve from Rust backend (minimal port)
- Option B: Rewrite in Rust-based framework (Leptos/Dioxus/Yew)
- Option C: Hybrid — keep existing UI components, new Rust-powered backend API
- **Recommended: Option A** for initial conversion (serve existing Lit UI from Axum)
- Update branding from OpenClaw to Aisopod
- Rebrand all UI components, views, and assets

### 3.13 Memory System
- Port QMD (query-memory database) functionality
- SQLite-based vector storage using rusqlite + sqlite-vec
- Memory retrieval and context injection
- LanceDB alternative integration

### 3.14 Session Management
- Persistent session state across disconnects
- Session key generation and routing
- History storage and retrieval
- Session compaction and pruning
- Multi-agent session isolation

### 3.15 Sandbox & Security
- Per-agent sandbox configuration (Docker/Podman containers)
- Workspace access controls (none/read/write)
- Tool policy enforcement
- Execution approval workflow
- Non-root container execution
- Auth scope system

### 3.16 Deployment & Packaging
- Docker multi-stage build (Rust compile → minimal runtime)
- Fly.io deployment config
- Render deployment config
- systemd/launchctl service files
- Platform-specific packaging (deb, rpm, homebrew, etc.)
- Cross-compilation support for multiple architectures

### 3.17 Mobile/Desktop App Protocol
- Define and document the WebSocket protocol for native clients
- Ensure backward compatibility or provide migration path
- Node pairing protocol for iOS/Android/macOS
- Device authentication flow
- Canvas rendering protocol

### 3.18 Documentation
- Port and adapt all documentation from OpenClaw
- Update for Rust-specific setup and configuration
- New getting-started guide for aisopod
- API reference generated from Rust doc comments
- Plugin/skill development guide
- Channel integration guides

---

## 4. Conversion Order & Dependencies

The recommended implementation order, based on dependency analysis:

```
Phase 1: Foundation
  ├── 3.1  Project Structure & Build System
  ├── 3.2  Configuration System
  └── 3.18 Documentation (initial)

Phase 2: Core Runtime
  ├── 3.3  Gateway Server
  ├── 3.5  AI Model Provider Integrations
  └── 3.6  Tool System

Phase 3: Agent Engine
  ├── 3.4  Agent Execution Engine
  ├── 3.14 Session Management
  └── 3.13 Memory System

Phase 4: Messaging
  ├── 3.7  Channel Abstraction
  └── 3.8  Channel Extensions (Tier 1)

Phase 5: Extensibility
  ├── 3.9  Plugin System
  └── 3.10 Skills System

Phase 6: User Interface
  ├── 3.11 CLI Application
  └── 3.12 Web Control UI

Phase 7: Production
  ├── 3.15 Sandbox & Security
  ├── 3.16 Deployment & Packaging
  ├── 3.17 Mobile/Desktop App Protocol
  ├── 3.8  Channel Extensions (Tier 2 & 3)
  └── 3.18 Documentation (complete)
```

---

## 5. Key Design Decisions

### 5.1 Crate Organization
```
aisopod/
├── Cargo.toml                 # Workspace root
├── crates/
│   ├── aisopod/               # Main binary crate (CLI + gateway)
│   ├── aisopod-config/        # Configuration types & validation
│   ├── aisopod-gateway/       # HTTP + WebSocket server
│   ├── aisopod-agent/         # Agent execution engine
│   ├── aisopod-provider/      # AI model provider abstractions
│   ├── aisopod-provider-anthropic/
│   ├── aisopod-provider-openai/
│   ├── aisopod-provider-gemini/
│   ├── aisopod-provider-bedrock/
│   ├── aisopod-provider-ollama/
│   ├── aisopod-channel/       # Channel abstraction traits
│   ├── aisopod-channel-telegram/
│   ├── aisopod-channel-discord/
│   ├── aisopod-channel-whatsapp/
│   ├── aisopod-channel-slack/
│   ├── aisopod-tools/         # Agent tool implementations
│   ├── aisopod-plugin/        # Plugin system
│   ├── aisopod-session/       # Session management
│   ├── aisopod-memory/        # Memory/vector DB
│   └── aisopod-shared/        # Shared utilities
├── ui/                        # Web UI (Lit, served by gateway)
└── docs/                      # Documentation
```

### 5.2 Error Handling
- Use `thiserror` for library error types
- Use `anyhow` for application-level errors
- Structured error codes matching the JSON-RPC error spec

### 5.3 Async Model
- Full async with Tokio runtime
- Channels (tokio::mpsc, tokio::broadcast) for event streaming
- Graceful shutdown with signal handling

### 5.4 Configuration
- Prefer TOML as the primary config format (Rust ecosystem standard)
- Support JSON5 for backward compatibility with OpenClaw configs
- Environment variable expansion using a simple template syntax

### 5.5 Plugin Loading
- Phase 1: Compiled-in plugins (feature flags)
- Phase 2: Dynamic loading via shared libraries (libloading)
- Phase 3: WASM plugin support for sandboxed extensions

---

## 6. Risk Areas & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| WhatsApp integration complexity | High — Baileys is Node.js only | Investigate Rust alternatives or bridge via subprocess |
| iMessage integration | High — Requires macOS APIs | macOS-only conditional compilation |
| Lit UI serving from Rust | Low — Static file serving | Standard asset embedding (rust-embed) |
| Plugin ecosystem compatibility | Medium — Breaking change | Version plugin API, migration guide |
| Mobile app protocol changes | Medium — Breaking change | Maintain protocol compatibility layer |
| SQLite-vec availability | Low — C library bindings | Well-supported via rusqlite |

---

## 7. Success Criteria

- [ ] All gateway API endpoints functional and tested
- [ ] All 24 WebSocket RPC methods implemented
- [ ] At least Tier 1 channels (Telegram, Discord, WhatsApp, Slack) operational
- [ ] Agent execution with streaming, failover, and compaction working
- [ ] CLI with all essential commands
- [ ] Web UI accessible and branded as Aisopod
- [ ] Docker deployment working
- [ ] Configuration migration from OpenClaw format
- [ ] Performance: Lower memory, faster startup than Node.js version
- [ ] Documentation covering setup, configuration, and migration
