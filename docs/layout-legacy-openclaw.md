# Legacy OpenClaw Codebase Layout & Functioning

This document provides a detailed reference of the OpenClaw project structure and functionality,
intended as a foundation for planning the Rust rewrite (aisopod).

---

## 1. Project Overview

OpenClaw is a **self-hosted, personal AI assistant gateway** built in **TypeScript/Node.js** (Node ≥ 22).
It connects multiple messaging platforms (WhatsApp, Telegram, Discord, Slack, Signal, iMessage, Google Chat,
Matrix, and more) to AI model providers (Anthropic Claude, OpenAI, Google Gemini, AWS Bedrock, Ollama, etc.)
through a unified WebSocket-based gateway.

**Key value proposition:** A single daemon process bridges all messaging channels to configurable AI agents,
with persistent sessions, extensible skills/plugins, and cross-platform native apps.

---

## 2. Repository Root Structure

```
openclaw/
├── openclaw.mjs              # CLI entry point (Node.js bootstrap)
├── package.json               # Root workspace config (pnpm monorepo)
├── pnpm-workspace.yaml        # Workspace member declarations
├── tsconfig.json              # TypeScript project config
├── tsdown.config.ts           # Build bundler config
├── vitest.config.ts           # Unit test config
├── vitest.e2e.config.ts       # End-to-end test config
├── vitest.extensions.config.ts
├── vitest.gateway.config.ts
├── vitest.live.config.ts
├── vitest.unit.config.ts
├── Dockerfile                 # Production Docker image
├── Dockerfile.sandbox*        # Agent sandbox containers
├── docker-compose.yml         # Local dev compose
├── fly.toml                   # Fly.io deployment
├── render.yaml                # Render deployment
├── .env.example               # Environment variable reference
│
├── src/                       # Core TypeScript source
├── ui/                        # Web control UI (Lit + Vite)
├── apps/                      # Native mobile/desktop apps
├── packages/                  # npm compatibility shims
├── extensions/                # Channel & feature extensions (plugins)
├── skills/                    # AI skill modules
├── scripts/                   # Build, deploy, and utility scripts
├── test/                      # End-to-end tests
├── docs/                      # Documentation site
├── assets/                    # Static assets
└── patches/                   # npm dependency patches
```

---

## 3. Core Source (`src/`)

### 3.1 Gateway (`src/gateway/`)

The heart of OpenClaw — an HTTP + WebSocket server providing the RPC control plane.

**Key files:**
- `server.ts`, `server.impl.ts` — Gateway initialization, HTTPS/HTTP server creation
- `server-http.ts` — REST endpoints:
  - `POST /v1/chat/completions` — OpenAI-compatible chat API
  - `POST /v1/responses` — OpenResponses API
  - `POST /hooks` — Webhook ingestion
  - `GET /tools/invoke` — Tool invocation
  - Canvas UI paths, control UI paths
- `server-ws-runtime.ts` — WebSocket upgrade handling and client lifecycle
- `server-methods.ts` — RPC method routing with role-based authorization

**WebSocket RPC Methods** (24 handler modules in `server-methods/`):
| Namespace      | Methods                                    |
|----------------|--------------------------------------------|
| `agent`        | `agent`, `agent.wait`, `agents.list/create/update/delete` |
| `chat`         | `chat.send`, `chat.abort`, `chat.history`, `chat.inject` |
| `node`         | `node.list`, `node.describe`, `node.invoke`, `node.pair.*` |
| `config`       | `config.get`, `config.set`, `config.apply`, `config.patch` |
| `skills`       | `skills.status`, `skills.bins`             |
| `sessions`     | `sessions.list`, `sessions.send`, `sessions.patch` |
| `system`       | `health`, `status`, `logs.tail`, `system-presence` |
| `cron`         | `cron.add/list/run/remove`                 |
| `models`       | `models.list`, `models.voicewake`          |
| `devices`      | Pairing, token management, revocation      |
| `approvals`    | Exec approval request/wait/resolve         |
| `updates`      | Software updates, plugins, wizard          |

**Protocol:** JSON-RPC 2.0 style — `{ id, method, params }` / `{ id, result }` / broadcast events.

**Authentication** (`auth.ts`, `auth-rate-limit.ts`):
- Bearer token, password, Tailscale identity, device tokens, trusted proxy, or none (loopback)
- Per-IP rate limiting with sliding windows
- Role-based authorization: `operator` (admin/read/write/approvals/pairing scopes) and `node`

**WebSocket Client Management** (`server/ws-connection.ts`):
- Tracks active clients with presence, health snapshots, broadcast filtering
- Handshake timeout validation
- Event broadcasting to subscribed clients

### 3.2 Agents (`src/agents/`)

The AI execution engine — manages agent lifecycle, model selection, tool execution, and session state.

**Core execution:**
- `runEmbeddedPiAgent()` — Main agent entry point
- `subscribeEmbeddedPiSession()` — Stream results from agent runs
- `runEmbeddedAttempt()` — Single attempt with model selection and error handling

**Agent scope & binding** (`agent-scope.ts`):
- `resolveSessionAgentId()` — Map sessions to agents via channel bindings
- `resolveAgentConfig()` — Get per-agent configuration
- `resolveAgentModelPrimary()` — Resolve model with fallback chain

**Agent binding** routes channels to agents:
```
AgentBinding { agentId, match: { channel, accountId?, peer?, guildId? } }
```

**Model selection & failover** (`model-selection.ts`, `model-auth.ts`):
- Primary model → fallback 1 → fallback 2 → fallback 3
- Auth profile rotation (round-robin + cooldown)
- Handles auth errors, rate limits, context overflow, timeouts

**Tools** (`agents/tools/`):
- `bash-tools` — Shell execution with approval workflow
- `pi-tools` — File operations, code execution
- `message-tool` — Send messages to channels
- `subagents-tool` — Spawn child agents
- `sessions-tool` — Manage conversation threads
- `cron-tool` — Schedule tasks
- `canvas-tool` — Visual output
- `channel-tools` — Channel-specific actions

**Session management:**
- `compaction.ts` — Adaptive history chunking, summary + hard-clear strategies
- `context-window-guard.ts` — Warn/fail at context thresholds, adaptive pruning
- Subagent spawning with depth limits and allowlist control

### 3.3 Channels (`src/channels/`)

Channel abstraction layer for multi-platform messaging.

**Core files:**
- `registry.ts` — Central channel registry with metadata, ordering, normalization
- `typing.ts` — Typing indicator callbacks
- `plugins/` — Channel plugin system (types, catalog, loaders, config)

**Channel Plugin Interface** (`ChannelPlugin<ResolvedAccount>`):
- `id` — Channel identifier (e.g., "discord", "telegram")
- `meta` — Metadata (label, docs, capabilities, UI hints)
- `capabilities` — Feature support (chat types, polls, reactions, media, threads)
- `config` — Account management (list, resolve, enable, delete)

**Optional adapter interfaces** (13 adapter types):
- `onboarding` — CLI setup wizard
- `pairing` — Security/allowlist integration
- `outbound` — Message delivery
- `status` — Health monitoring
- `directory` — Group/user discovery
- `security` — DM policies, elevation
- `gateway` — WebSocket/polling connections
- `threading` — Thread/reply support
- `messaging` — Reactions and actions
- `heartbeat` — Keep-alive
- `auth` — Token/credential handling

### 3.4 Configuration (`src/config/`)

Zod-schema-based configuration system with runtime validation.

**Root config** (`OpenClawConfig`):
```
meta          → agent versioning metadata
auth          → API keys & auth profiles
env           → environment variables
agents        → agent list + defaults
models        → model definitions & providers
channels      → channel configurations
tools         → bash, exec, file system tools
skills        → custom agent capabilities
plugins       → extension system
session       → message handling
bindings      → agent-to-channel routing
memory        → QMD memory system
gateway       → HTTP server config
```

**Loading pipeline:**
```
readConfigFileSnapshot()
  → Parse JSON5
  → Environment variable substitution
  → @include directive scanning
  → OpenClawSchema.safeParse() (Zod validation)
  → Legacy migration
  → Return OpenClawConfig
```

**File conventions:**
- `zod-schema.*.ts` — Zod validators
- `types.*.ts` — TypeScript interfaces
- `config.*.test.ts` — Validation tests
- Sensitive fields marked for UI redaction

### 3.5 Commands (`src/commands/`)

Extensive CLI with ~100 commands organized into groups:

| Group            | Commands                                                |
|------------------|---------------------------------------------------------|
| Agent management | add, delete, identity, list agents                     |
| Authentication   | OAuth, API keys, device codes, custom flows            |
| Configuration    | Wizard-based setup, channels, models                   |
| Status & health  | Status, health, dashboard, diagnostics                 |
| Sandbox          | Container management & inspection                      |
| Models           | Model selection, switching, allowlists                 |
| Channels         | Multi-protocol messaging setup                         |
| Daemon           | Background service management (systemd/launchctl)      |
| Gateway          | API gateway configuration & status                     |
| Sessions         | Session management, reset                              |
| Utilities        | Message send, docs, help formatting                    |

### 3.6 Plugins (`src/plugins/`)

Extensible plugin system with security hardening.

**Plugin lifecycle:**
- Manifest: `openclaw.plugin.json` — metadata, config schema, supported channels/providers
- Discovery: Auto-discovery with manifest validation
- Loader: Dynamic ES module loading with SDK alias resolution
- Registry: Central lifecycle management

**Plugin capabilities:**
- Register custom CLI commands (with 72 reserved command names protected)
- Register custom agent tools with context (workspace, agent, sandbox info)
- Pre/post lifecycle hooks (agent start/end, message events, tool calls, session lifecycle, gateway events)
- Auth & model provider extensions
- Custom channel implementations

**Security:**
- Command name validation, conflict prevention
- Argument sanitization (max 4KB, control char removal)
- Authorization checks (requireAuth flag)
- Registry locking during execution

### 3.7 Providers (`src/providers/`)

AI model provider integrations:
- GitHub Copilot, Google Gemini, Qwen Portal auth
- Provider-specific model discovery and token handling
- OAuth flow implementations

### 3.8 Shared Utilities (`src/shared/`)

Cross-cutting utilities:
- `config-eval.ts` — Path resolution, binary detection, PATH resolution
- `requirements.ts` — System requirement validation (bins, env vars, OS)
- `chat-envelope.ts` — Message metadata stripping
- `usage-aggregates.ts` — Token usage aggregation
- `frontmatter.ts` — YAML frontmatter parsing
- `device-auth.ts` — Device code flow utilities

---

## 4. Web UI (`ui/`)

Browser-based control interface built with **Lit** (v3.3.2) web components and **Vite** (v7.3.1).

**Architecture:**
- Root component: `<openclaw-app>` (defined in `app.ts`)
- Views (47 view files): Agents, channels, chat, config, sessions, usage metrics, device auth
- Controllers (20+ modules): State/logic managers for different domains
- Components: Reusable UI elements (resizable-divider)
- Styling: CSS with markdown support (`marked`)

**Key views:**
- Agent management and skill handling
- Multi-channel configuration (Discord, Slack, Telegram, Signal, iMessage, Nostr, WhatsApp, Google Chat)
- Chat messaging interface with markdown rendering
- Form-based configuration system
- Gateway API communication layer
- Usage metrics and analytics
- Theme support (light/dark mode)

---

## 5. Native Apps (`apps/`)

### 5.1 iOS (`apps/ios/`)
- SwiftUI app connecting to Gateway as a node role
- Exposes phone services (camera, location, calendar)
- Talk + Chat surfaces

### 5.2 Android (`apps/android/`)
- Kotlin + Jetpack Compose (minSdk 31+)
- Connects to Gateway WebSocket
- Canvas, Chat, and Camera surfaces

### 5.3 macOS (`apps/macos/`)
- SwiftUI app with code-signing and notarization support
- Dev/signing utilities, DMG packaging

### 5.4 Shared (`apps/shared/OpenClawKit/`)
- Shared Swift transport/types library used by iOS and macOS apps

---

## 6. Extensions (`extensions/`)

36 extension modules providing channel integrations and features:

**Messaging channels:** Discord, Telegram, WhatsApp, Slack, Signal, iMessage, IRC,
Google Chat, MS Teams, Matrix, Mattermost, Nextcloud Talk, Twitch, Nostr, Zalo, LINE, Lark/Feishu

**Specialized extensions:**
- `device-pair` — Mobile/node device pairing
- `talk-voice` — Voice integration
- `phone-control` — Phone services
- `thread-ownership` — Thread management
- `memory-core`, `memory-lancedb` — Memory systems
- `llm-task` — LLM task execution
- `diagnostics-otel` — OpenTelemetry diagnostics

**Authentication extensions:**
- `google-gemini-cli-auth`
- `google-antigravity-auth`
- `minimax-portal-auth`
- `qwen-portal-auth`
- `copilot-proxy-auth`

**Extension structure pattern:**
```
extensions/<name>/
├── openclaw.plugin.json   # Manifest
├── index.ts               # Entry point with register(api) function
└── src/
    ├── channel.ts         # Channel plugin implementation
    └── runtime.ts         # Channel-specific state & operations
```

---

## 7. Skills (`skills/`)

43 skill modules extending AI agent capabilities:

**Categories:**
- **Messaging:** apple-messages, bluebubbles, discord, slack, telegram
- **Productivity:** notion, obsidian, trello, things-mac, bear-notes, apple-notes
- **AI/ML:** openai-image-gen, openai-whisper, gemini, coding-agent, summarize
- **Integration:** 1password, github, spotify-player, weather
- **Utilities:** tmux, video-frames, healthcheck, skill-creator
- **System:** session-logs, model-usage

---

## 8. Testing (`test/`)

**Test infrastructure:**
- Vitest framework with multiple config files for different test scopes
- End-to-end tests: `gateway.multi.e2e.test.ts`, `media-understanding.auto.e2e.test.ts`, `provider-timeout.e2e.test.ts`
- Support: `setup.ts`, `global-setup.ts`, `test-env.ts`, `helpers/`, `fixtures/`, `mocks/`
- Unit tests co-located with source (`*.test.ts` files)

---

## 9. Build & Deployment

### Build tools
- **pnpm** — Package manager (workspace monorepo)
- **Tsdown** — TypeScript bundler
- **Vite** — UI bundler
- **Oxlint/Oxfmt** — Linting & formatting

### Deployment targets
- **Docker** — Multi-stage build, non-root user, loopback binding
- **Fly.io** — shared-cpu-2x, 2GB RAM, persistent volume at /data
- **Render** — Docker web service, 1GB persistent disk
- **Local** — systemd/launchctl daemon management
- **macOS app** — DMG packaging with codesign/notarize

### Scripts (`scripts/`)
60+ scripts covering:
- Docker smoke tests and e2e installs
- macOS app packaging (codesign, notarize, DMG)
- Changelog generation, release checks
- Documentation building and i18n
- Auth system setup, log collection, model benchmarks

---

## 10. Configuration & Environment

### Config file
- Location: `~/.openclaw/openclaw.json` (JSON5 format)
- Supports environment variable substitution
- `@include` directives for modular config
- Legacy migration support

### Environment variables
- `OPENCLAW_GATEWAY_TOKEN` / `OPENCLAW_GATEWAY_PASSWORD` — Auth
- Provider API keys: OpenAI, Anthropic, Gemini, OpenRouter, etc.
- Channel tokens: Telegram, Discord, Slack, etc.
- Tool API keys: Brave Search, Perplexity, Firecrawl, ElevenLabs, Deepgram

**Precedence:** Process env → `./.env` → `~/.openclaw/.env` → `openclaw.json`

---

## 11. Key Architectural Patterns

1. **Config-driven** — Everything configurable (tools, models, channels, plugins)
2. **Plugin-based channels** — Adapter pattern with optional interfaces for different behaviors
3. **Zod runtime validation** — Schema-first config with TypeScript type safety
4. **WebSocket RPC** — JSON-RPC 2.0 style control plane
5. **Streaming execution** — Real-time agent result subscription
6. **Multi-model failover** — Primary → fallback chain with auth profile rotation
7. **Session compaction** — Adaptive history management for context windows
8. **Extension ecosystem** — Skills, plugins, and channels as separate loadable modules
9. **Security-first** — Non-root Docker, rate limiting, auth scopes, sandbox isolation
10. **Cross-platform** — Web, iOS, Android, macOS native apps sharing common protocol
