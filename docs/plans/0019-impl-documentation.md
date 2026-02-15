# 0019 — Documentation

**Master Plan Reference:** Section 3.18 — Documentation  
**Phase:** 1 (Foundation, initial) → 7 (Production, complete)  
**Dependencies:** All other plans (documentation evolves alongside implementation)

---

## Objective

Create comprehensive documentation for aisopod covering user guides, API reference,
developer documentation, and migration guides from OpenClaw.

---

## Deliverables

### 1. Getting Started Guide

- System requirements (Rust not needed for binary users)
- Installation methods:
  - Pre-built binary download
  - Homebrew (`brew install aisopod`)
  - Cargo install (`cargo install aisopod`)
  - Docker (`docker pull aisopod/aisopod`)
- First-run onboarding wizard walkthrough
- Quick start: send your first message
- Architecture overview (what is a gateway, agent, channel)

### 2. User Guide

**Configuration:**
- Config file format (TOML/JSON5)
- Environment variables reference
- Config sections explained
- Example configurations for common setups

**Agents:**
- Agent concepts and lifecycle
- Creating and managing agents
- Model selection and fallback configuration
- System prompt customization
- Workspace configuration

**Channels:**
- Channel setup guides (one per channel):
  - Telegram: Create bot, configure token
  - Discord: Create bot, configure permissions
  - WhatsApp: Business API setup or bridge setup
  - Slack: Create app, configure tokens
  - Signal, iMessage, Google Chat, Teams, etc.
- Multi-channel configuration
- Agent binding (route channels to agents)

**Skills:**
- Available skills reference
- Enabling/disabling skills
- Skill configuration
- Creating custom skills

**CLI Reference:**
- All commands with examples
- Environment variables
- Configuration file reference
- Shell completion setup

### 3. API Reference

**REST API:**
- `POST /v1/chat/completions` — OpenAI-compatible endpoint
- `POST /v1/responses` — OpenResponses endpoint
- `POST /hooks` — Webhook ingestion
- `GET /health` — Health check
- Authentication methods

**WebSocket API:**
- Connection handshake
- All RPC methods with request/response schemas
- Event types and payloads
- Error codes and handling
- Rate limiting

**Generated from Rust doc comments:**
- `cargo doc --workspace --no-deps`
- Published to documentation site

### 4. Developer Guide

**Architecture:**
- Crate dependency graph
- Module organization
- Key design patterns (traits, async, streaming)

**Contributing:**
- Development environment setup
- Build and test commands
- Code style (rustfmt, clippy)
- PR process and review guidelines

**Plugin Development:**
- Plugin trait implementation guide
- Manifest format specification
- Hook system reference
- Example plugin walkthrough

**Channel Development:**
- Channel trait implementation guide
- Adapter interfaces reference
- Example channel implementation walkthrough

### 5. Migration Guide (OpenClaw → Aisopod)

- Configuration format migration
- Environment variable mapping (`OPENCLAW_*` → `AISOPOD_*`)
- Feature parity checklist
- Breaking changes and workarounds
- Data migration (sessions, memories)

### 6. Security Documentation

- Authentication modes explained
- Authorization scopes reference
- Sandbox configuration guide
- Best practices for production deployment
- API key management

### 7. Deployment Guides

- Docker deployment (standalone, compose)
- Fly.io deployment
- Render deployment
- VPS deployment (Ubuntu, Debian, etc.)
- Raspberry Pi deployment
- systemd service setup
- macOS launchctl setup
- Reverse proxy configuration (nginx, Caddy)

### 8. Troubleshooting

- Common errors and solutions
- Diagnostic commands (`aisopod doctor`)
- Log analysis
- Channel-specific troubleshooting
- Performance tuning

### 9. Documentation Infrastructure

- Static site generator (mdBook or similar Rust-native tool)
- Automated build from markdown source
- Version-tagged documentation
- Search functionality
- Dark/light theme

---

## Acceptance Criteria

- [ ] Getting started guide enables a new user to have aisopod running in <10 minutes
- [ ] All CLI commands are documented with examples
- [ ] All API endpoints are documented with request/response schemas
- [ ] All WebSocket methods are documented
- [ ] Each channel has a setup guide
- [ ] Migration guide covers all OpenClaw → aisopod differences
- [ ] Plugin development guide enables creating a basic plugin
- [ ] Deployment guides cover Docker, Fly.io, Render, and VPS
- [ ] Documentation site builds and deploys automatically
- [ ] Search works across all documentation pages
