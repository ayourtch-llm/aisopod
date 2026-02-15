# 0003 — Gateway Server (HTTP + WebSocket)

**Master Plan Reference:** Section 3.3 — Gateway Server  
**Phase:** 2 (Core Runtime)  
**Dependencies:** 0001 (Project Structure), 0002 (Configuration System)

---

## Objective

Implement the core gateway server providing HTTP REST endpoints and WebSocket
RPC communication, including authentication, rate limiting, client management,
and event broadcasting.

---

## Deliverables

### 1. HTTP Server (`aisopod-gateway`)

Build on **Axum** framework:

**REST Endpoints:**
- `POST /v1/chat/completions` — OpenAI-compatible chat completions API
- `POST /v1/responses` — OpenResponses API
- `POST /hooks` — Webhook ingestion for external integrations
- `GET /tools/invoke` — Tool invocation handler
- `GET /health` — Health check endpoint
- `GET /status` — System status
- Canvas UI paths (`/canvas`, `/a2ui`)
- Control UI static file serving

**Server features:**
- Configurable bind address and port (from `GatewayConfig`)
- TLS/HTTPS support (optional, via `rustls` or `native-tls`)
- Graceful shutdown with signal handling (SIGINT, SIGTERM)
- Request logging with `tracing`
- CORS configuration

### 2. WebSocket Server

**Connection lifecycle:**
- WebSocket upgrade from HTTP (`/ws` or configurable path)
- Handshake with timeout validation
- Client registration with `connId`, presence key, client IP
- Heartbeat/ping-pong keep-alive
- Graceful disconnect and cleanup

**Message protocol (JSON-RPC 2.0 style):**
```rust
// Request
{ "id": "uuid", "method": "chat.send", "params": { ... } }

// Response
{ "id": "uuid", "result": { ... } }

// Error
{ "id": "uuid", "error": { "code": -32600, "message": "..." } }

// Broadcast event (no id)
{ "method": "agent.event", "params": { ... } }
```

### 3. RPC Method Router

Implement method dispatch for all 24 WebSocket RPC method namespaces:

| Namespace    | Methods to implement                                  |
|--------------|-------------------------------------------------------|
| `agent`      | `agent`, `agent.wait`, `agents.list/create/update/delete` |
| `chat`       | `chat.send`, `chat.abort`, `chat.history`, `chat.inject` |
| `node`       | `node.list`, `node.describe`, `node.invoke`, `node.pair.*` |
| `config`     | `config.get`, `config.set`, `config.apply`, `config.patch` |
| `skills`     | `skills.status`, `skills.bins`                        |
| `sessions`   | `sessions.list`, `sessions.send`, `sessions.patch`    |
| `system`     | `health`, `status`, `logs.tail`, `system-presence`    |
| `cron`       | `cron.add/list/run/remove`                            |
| `models`     | `models.list`, `models.voicewake`                     |
| `devices`    | Pairing, token management, revocation                 |
| `approvals`  | Exec approval request/wait/resolve                    |
| `updates`    | Software updates, plugins, wizard                     |

**Method handler trait:**
```rust
#[async_trait]
pub trait RpcMethod {
    async fn handle(&self, ctx: &RequestContext, params: Value) -> Result<Value>;
}
```

### 4. Authentication System

**Auth modes** (from config):
- `token` — Bearer token in WebSocket upgrade header or HTTP Authorization
- `password` — Shared secret
- `device_token` — Mobile/node device credentials
- `none` — Loopback-only or open access

**Implementation:**
- Auth middleware for HTTP routes (Axum extractors)
- Auth validation on WebSocket handshake
- Role extraction: `operator` (with scopes) or `node`

**Authorization scopes:**
- `operator.admin` — Admin methods
- `operator.read` — Status queries
- `operator.write` — State changes
- `operator.approvals` — Exec approval workflow
- `operator.pairing` — Device/node pairing

### 5. Rate Limiting

- Per-IP rate limiting with sliding window
- Configurable attempt limits and cooldown
- Returns HTTP 429 with `Retry-After` header
- In-memory state with periodic cleanup (using `dashmap` or similar)

### 6. Client Connection Management

**State tracking:**
```rust
pub struct GatewayClient {
    pub conn_id: String,
    pub socket: WebSocketSender,
    pub presence_key: String,
    pub client_ip: IpAddr,
    pub role: ClientRole,
    pub scopes: Vec<Scope>,
    pub connected_at: Instant,
}
```

**Features:**
- Active client set (`DashMap` or `RwLock<HashMap>`)
- Presence tracking and health snapshots
- Broadcast filtering (send events to matching clients)
- Connection lifecycle hooks

### 7. Event Broadcasting

- Broadcast system for real-time events to all/filtered clients
- Event types: presence, health, agent events, chat events, node events
- Per-client subscription filtering
- Async broadcast using `tokio::broadcast` channel

### 8. Static File Serving

- Serve Web UI from embedded assets (using `rust-embed` or `include_dir`)
- Or from filesystem directory (configurable)
- Cache headers for static assets
- SPA fallback (serve index.html for unknown routes)

---

## Acceptance Criteria

- [ ] HTTP server starts and binds to configured address/port
- [ ] All REST endpoints respond correctly
- [ ] WebSocket connections establish with handshake
- [ ] JSON-RPC messages route to correct method handlers
- [ ] Authentication validates tokens/passwords correctly
- [ ] Unauthorized requests are rejected with proper error codes
- [ ] Rate limiting enforces per-IP limits
- [ ] Client connections are tracked and cleaned up on disconnect
- [ ] Event broadcasting delivers to subscribed clients
- [ ] Static files are served correctly
- [ ] Graceful shutdown completes without dropping connections
- [ ] Integration tests cover all endpoints and WebSocket methods
