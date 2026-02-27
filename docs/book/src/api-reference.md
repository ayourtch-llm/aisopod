# REST & WebSocket API Reference

This document provides comprehensive documentation for the Aisopod gateway API, including REST endpoints, WebSocket RPC methods, authentication, error codes, and rate limiting.

## Authentication

All API requests require authentication unless `auth.mode = "none"` is configured.

### Bearer Token

Authenticate by including the `Authorization` header:

```http
Authorization: Bearer <your-token>
```

**Example:**
```bash
curl -H "Authorization: Bearer my-secret-token" \
     http://localhost:3080/v1/chat/completions
```

### Query Parameter (for webhooks)

For webhook endpoints, authenticate using a query parameter:

```http
GET /hooks/telegram?token=<your-token>
```

**Example:**
```bash
curl "http://localhost:3080/hooks/telegram?token=my-secret-token" \
     -X POST \
     -H "Content-Type: application/json" \
     -d '{"update_id": 123, "message": {"text": "Hello"}}'
```

### Authentication Failure

Unauthenticated requests return `401 Unauthorized`:

```json
{
  "error": "unauthorized",
  "message": "Missing or invalid authentication"
}
```

---

## REST API

**Base URL:** `http://localhost:3080`

### `POST /v1/chat/completions`

OpenAI-compatible chat completions endpoint.

#### Request

**Headers:**
```http
Content-Type: application/json
Authorization: Bearer <your-token>
```

**Body:**
```json
{
  "model": "gpt-4o",
  "messages": [
    {"role": "system", "content": "You are a helpful assistant."},
    {"role": "user", "content": "Hello!"}
  ],
  "stream": false,
  "temperature": 0.7,
  "max_tokens": 1024
}
```

**Parameters:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `model` | string | Yes | Model identifier (e.g., "gpt-4o", "claude-3-5-sonnet") |
| `messages` | array | Yes | Array of message objects with `role` and `content` |
| `stream` | boolean | No | If `true`, returns streaming response (default: `false`) |
| `temperature` | number | No | Sampling temperature (0.0-2.0, default: 1.0) |
| `max_tokens` | integer | No | Maximum completion tokens (default: unlimited) |

#### Response (Non-Streaming)

**Status:** `200 OK`

```json
{
  "id": "chatcmpl-abc123",
  "object": "chat.completion",
  "created": 1700000000,
  "model": "gpt-4o",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "Hello! How can I help you today?"
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 12,
    "completion_tokens": 8,
    "total_tokens": 20
  }
}
```

#### Response (Streaming)

When `stream: true`, returns Server-Sent Events (SSE) format:

```
data: {"id":"chatcmpl-abc123","object":"chat.completion.chunk","created":1700000000,"model":"gpt-4o","choices":[{"index":0,"delta":{"content":"Hello"},"finish_reason":null}]}

data: {"id":"chatcmpl-abc123","object":"chat.completion.chunk","created":1700000000,"model":"gpt-4o","choices":[{"index":0,"delta":{"content":"!"},"finish_reason":null}]}

data: [DONE]
```

#### Errors

| Status | Code | Description |
|--------|------|-------------|
| 400 | `invalid_request` | Malformed request body or invalid parameters |
| 401 | `unauthorized` | Missing or invalid authentication |
| 429 | `rate_limited` | Too many requests |
| 500 | `internal_error` | Internal server error |
| 502 | `upstream_error` | LLM provider returned an error |

---

### `POST /v1/responses`

Create a response using the Responses API format.

#### Request

**Headers:**
```http
Content-Type: application/json
Authorization: Bearer <your-token>
```

**Body:**
```json
{
  "model": "gpt-4o",
  "input": "Explain Rust ownership in one paragraph."
}
```

**Parameters:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `model` | string | Yes | Model identifier |
| `input` | string | Yes | Input text to process |

#### Response

**Status:** `200 OK`

```json
{
  "id": "resp-abc123",
  "object": "response",
  "created": 1700000000,
  "model": "gpt-4o",
  "output": [
    {
      "type": "message",
      "role": "assistant",
      "content": [
        {
          "type": "text",
          "text": "Rust ownership is a set of rules that govern how memory is managed. Every value in Rust has a single owner, and when the owner goes out of scope, the value is dropped. Ownership enables memory safety without garbage collection."
        }
      ]
    }
  ]
}
```

#### Errors

| Status | Code | Description |
|--------|------|-------------|
| 400 | `invalid_request` | Malformed request body or invalid parameters |
| 401 | `unauthorized` | Missing or invalid authentication |
| 429 | `rate_limited` | Too many requests |
| 500 | `internal_error` | Internal server error |
| 502 | `upstream_error` | LLM provider returned an error |

---

### `POST /hooks/{channel_type}`

Webhook receiver for channel integrations (Telegram, Slack, Discord, etc.).

#### Request

**Path Parameters:**

| Parameter | Description |
|-----------|-------------|
| `channel_type` | The channel type (e.g., "telegram", "slack", "discord") |

**Headers:**
```http
Content-Type: application/json
```

**Body (Telegram example):**
```json
{
  "update_id": 123,
  "message": {
    "message_id": 1,
    "from": {"id": 123456, "username": "user123"},
    "chat": {"id": 789, "type": "private"},
    "date": 1700000000,
    "text": "Hello!"
  }
}
```

#### Response

**Status:** `200 OK`

```json
{
  "status": "ok"
}
```

**Status:** `202 Accepted` (if processing asynchronously)

```json
{
  "status": "accepted",
  "message": "Webhook processed successfully"
}
```

#### Errors

| Status | Code | Description |
|--------|------|-------------|
| 400 | `invalid_request` | Invalid channel type or malformed body |
| 401 | `unauthorized` | Missing or invalid authentication token |
| 404 | `not_found` | Channel type not supported |
| 500 | `internal_error` | Internal server error |

---

### `GET /health`

Health check endpoint. Returns gateway status and version information.

#### Request

No authentication required.

**Headers:**
```http
Authorization: Bearer <your-token>  (optional for rate limiting)
```

#### Response

**Status:** `200 OK`

```json
{
  "status": "ok",
  "version": "0.1.0",
  "uptime_seconds": 3600,
  "agents": 3,
  "channels": 2
}
```

**Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `status` | string | Always "ok" when healthy |
| `version` | string | Aisopod version |
| `uptime_seconds` | integer | Seconds since gateway started |
| `agents` | integer | Number of configured agents |
| `channels` | integer | Number of active channels |

#### Errors

| Status | Code | Description |
|--------|------|-------------|
| 503 | `unavailable` | Gateway is starting or stopping |

---

### `GET /status`

Detailed gateway status including agents, channels, and resource usage.

#### Request

**Headers:**
```http
Authorization: Bearer <your-token>
```

#### Response

**Status:** `200 OK`

```json
{
  "agents": [
    {
      "name": "default",
      "model": "gpt-4o",
      "status": "active",
      "sessions": 5
    }
  ],
  "channels": [
    {
      "type": "telegram",
      "status": "connected",
      "connected_at": "2024-01-15T10:30:00Z"
    }
  ],
  "memory_mb": 128,
  "active_sessions": 42,
  "uptime_seconds": 3600,
  "version": "0.1.0"
}
```

**Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `agents` | array | List of configured agents with status |
| `agents[].name` | string | Agent identifier |
| `agents[].model` | string | Model used by the agent |
| `agents[].status` | string | "active", "inactive", or "error" |
| `agents[].sessions` | integer | Number of active sessions |
| `channels` | array | List of connected channels |
| `channels[].type` | string | Channel type (e.g., "telegram") |
| `channels[].status` | string | "connected", "disconnected", or "error" |
| `memory_mb` | integer | Current memory usage in MB |
| `active_sessions` | integer | Total number of active sessions |
| `uptime_seconds` | integer | Seconds since gateway started |
| `version` | string | Aisopod version |

#### Errors

| Status | Code | Description |
|--------|------|-------------|
| 401 | `unauthorized` | Missing or invalid authentication |
| 503 | `unavailable` | Gateway is starting or stopping |

---

## WebSocket API

**Connection URL:** `ws://localhost:3080/ws`

### Connection Handshake

**HTTP Request:**
```http
GET /ws HTTP/1.1
Host: localhost:3080
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==
Sec-WebSocket-Version: 13
Authorization: Bearer <your-token>
```

**Protocol Version Header (optional):**
```http
X-Aisopod-Protocol-Version: 1
```

### Message Format (JSON-RPC 2.0)

All WebSocket messages use JSON-RPC 2.0 format:

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "chat.send",
  "params": {...},
  "id": 1
}
```

**Success Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {...},
  "id": 1
}
```

**Error Response:**
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32601,
    "message": "Method not found"
  },
  "id": 1
}
```

**Notification (no response expected):**
```json
{
  "jsonrpc": "2.0",
  "method": "chat.send",
  "params": {...}
}
```

### RPC Methods

| Method | Scope Required | Description |
|--------|----------------|-------------|
| `system.ping` | none | Ping the gateway |
| `system.info` | `operator.read` | Get gateway system information |
| `agent.list` | `operator.read` | List all configured agents |
| `agent.get` | `operator.read` | Get agent details |
| `agent.start` | `operator.write` | Start an agent |
| `agent.stop` | `operator.write` | Stop an agent |
| `agent.create` | `operator.admin` | Create a new agent |
| `agent.update` | `operator.admin` | Update an agent |
| `agent.delete` | `operator.admin` | Delete an agent |
| `session.create` | `operator.write` | Create a new conversation session |
| `session.list` | `operator.read` | List active sessions |
| `session.get` | `operator.read` | Get session details |
| `session.close` | `operator.write` | Close a session |
| `chat.send` | `operator.write` | Send a message to an agent |
| `chat.stream` | `operator.write` | Stream a response from an agent |
| `chat.history` | `operator.read` | Get conversation history |
| `tools.list` | `operator.read` | List available tools |
| `models.list` | `operator.read` | List available models |
| `channels.list` | `operator.read` | List connected channels |
| `config.get` | `operator.read` | Get gateway configuration |
| `config.update` | `operator.admin` | Update gateway configuration |
| `health.check` | `operator.read` | Check gateway health |
| `memory.query` | `operator.read` | Query memory store |
| `approval.list` | `operator.read` | List pending approvals |
| `approval.request` | `operator.approvals` | Request approval |
| `approval.approve` | `operator.approvals` | Approve a request |
| `approval.deny` | `operator.approvals` | Deny a request |
| `pairing.initiate` | `operator.pairing` | Initiate device pairing |
| `pairing.confirm` | `operator.pairing` | Confirm device pairing |
| `admin.shutdown` | `operator.admin` | Shutdown the gateway |

#### Method Details

##### `system.ping`

Ping the gateway to verify connectivity.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "system.ping",
  "id": 1
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {"status": "ok", "ping": "pong"},
  "id": 1
}
```

##### `agent.list`

List all configured agents.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "agent.list",
  "id": 1
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "agents": [
      {
        "name": "default",
        "model": "gpt-4o",
        "status": "active",
        "created_at": "2024-01-15T10:00:00Z"
      }
    ],
    "count": 1
  },
  "id": 1
}
```

##### `chat.send`

Send a message to an agent and receive a response.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "chat.send",
  "params": {
    "text": "Hello, how are you?",
    "channel": "telegram-123",
    "agent": "default"
  },
  "id": 1
}
```

**Parameters:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `text` | string | Yes | Message text to send |
| `channel` | string | No | Channel ID for routing |
| `agent` | string | No | Agent ID to execute (uses default if not specified) |

**Response (immediate acknowledgment):**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "status": "accepted",
    "message": "Agent execution started"
  },
  "id": 1
}
```

**Streaming Responses (via same WebSocket connection):**

```json
{
  "jsonrpc": "2.0",
  "method": "chat.response",
  "params": {
    "text": "Hello",
    "done": false
  }
}
```

```json
{
  "jsonrpc": "2.0",
  "method": "chat.response",
  "params": {
    "text": "!",
    "done": false
  }
}
```

```json
{
  "jsonrpc": "2.0",
  "method": "chat.response",
  "params": {
    "text": "How can I help you today?",
    "usage": {
      "prompt_tokens": 12,
      "completion_tokens": 8,
      "total_tokens": 20
    },
    "done": true
  }
}
```

##### `session.create`

Create a new conversation session.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "session.create",
  "params": {
    "channel": "telegram-123",
    "metadata": {"user_id": "12345"}
  },
  "id": 1
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "session_id": "sess-abc123",
    "channel": "telegram-123",
    "created_at": "2024-01-15T11:00:00Z"
  },
  "id": 1
}
```

##### `session.list`

List active sessions.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "session.list",
  "id": 1
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "sessions": [
      {
        "session_id": "sess-abc123",
        "channel": "telegram-123",
        "created_at": "2024-01-15T11:00:00Z",
        "last_activity": "2024-01-15T11:05:00Z"
      }
    ],
    "count": 1
  },
  "id": 1
}
```

##### `admin.shutdown`

Shutdown the gateway (admin only).

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "admin.shutdown",
  "id": 1
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {"status": "shutdown initiated"},
  "id": 1
}
```

### Event Types (Server → Client)

The gateway can send events to clients over the WebSocket connection:

| Event | Description |
|-------|-------------|
| `chat.response` | Streaming response chunk or complete response |
| `chat.complete` | Response generation completed |
| `chat.error` | Error during response generation |
| `session.updated` | Session metadata changed |
| `gateway.event` | Gateway-level events (e.g., shutdown, config change) |

#### Event Format

**Chat Response Event:**
```json
{
  "jsonrpc": "2.0",
  "method": "chat.response",
  "params": {
    "text": "Hello!",
    "done": false
  }
}
```

**Chat Complete Event:**
```json
{
  "jsonrpc": "2.0",
  "method": "chat.complete",
  "params": {
    "session_id": "sess-abc123",
    "total_tokens": 20,
    "response_time_ms": 150
  }
}
```

**Chat Error Event:**
```json
{
  "jsonrpc": "2.0",
  "method": "chat.error",
  "params": {
    "error": "Model not found",
    "session_id": "sess-abc123"
  }
}
```

**Session Updated Event:**
```json
{
  "jsonrpc": "2.0",
  "method": "session.updated",
  "params": {
    "session_id": "sess-abc123",
    "updated_fields": {"metadata": {"user_id": "67890"}}
  }
}
```

**Gateway Event:**
```json
{
  "jsonrpc": "2.0",
  "method": "gateway.event",
  "params": {
    "event": "shutdown",
    "message": "Gateway is shutting down",
    "timestamp": "2024-01-15T12:00:00Z"
  }
}
```

---

## Error Codes

### HTTP Error Codes

| HTTP Code | Error Code | Description |
|-----------|------------|-------------|
| 400 | `bad_request` | Malformed request body or invalid parameters |
| 401 | `unauthorized` | Missing or invalid authentication |
| 403 | `forbidden` | Insufficient permissions for the requested action |
| 404 | `not_found` | Endpoint or resource not found |
| 405 | `method_not_allowed` | HTTP method not allowed for this endpoint |
| 429 | `rate_limited` | Too many requests |
| 500 | `internal_error` | Unexpected server error |
| 502 | `upstream_error` | LLM provider returned an error |
| 503 | `unavailable` | Gateway is starting or stopping |

### JSON-RPC Error Codes

| Code | Name | Description |
|------|------|-------------|
| -32700 | `parse_error` | Invalid JSON was received |
| -32600 | `invalid_request` | Invalid request object |
| -32601 | `method_not_found` | Method does not exist |
| -32602 | `invalid_params` | Invalid method parameters |
| -32603 | `internal_error` | Internal JSON-RPC error |
| -32003 | `auth_error` | Authentication error (not authorized) |
| -32004 | `not_found` | Resource not found |
| -32005 | `method_not_allowed` | Method not allowed for this scope |
| -32006 | `internal_error` | Internal gateway error |

### Rate Limiting Response Headers

When rate limited, the response includes:

```http
HTTP/1.1 429 Too Many Requests
Retry-After: 5
X-RateLimit-Limit: 60
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1700000060
Content-Type: application/json

{
  "error": "rate_limited",
  "message": "Too many requests",
  "retry_after": 5
}
```

**Headers:**

| Header | Description |
|--------|-------------|
| `Retry-After` | Seconds to wait before making another request |
| `X-RateLimit-Limit` | Maximum requests allowed in the window |
| `X-RateLimit-Remaining` | Requests remaining in current window |
| `X-RateLimit-Reset` | Unix timestamp when the rate limit resets |

---

## Rate Limiting

Rate limiting is applied to prevent abuse and ensure fair usage.

### Default Limits

| Scope | Limit | Window |
|-------|-------|--------|
| Per-token (HTTP) | 60 requests/minute | Sliding window |
| Per-IP (HTTP) | 120 requests/minute | Sliding window |
| WebSocket messages | 30 messages/minute | Sliding window |

### Configuration

Configure rate limits in `config.toml`:

```toml
[gateway]
rate_limit_enabled = true

[gateway.rate_limit]
per_token_requests = 60
per_ip_requests = 120
websocket_messages = 30
window_seconds = 60
```

### Rate Limit Bypass

For testing, you can bypass rate limiting using the header:

```http
X-Aisopod-Bypass-Rate-Limit: true
```

### Rate Limit Behavior

1. **Sliding Window:** Rate limits use a sliding window algorithm that tracks requests over the last N seconds.
2. **Per-Token vs Per-IP:** When authentication is enabled, limits are applied per-token; otherwise, limits are applied per-IP address.
3. **WebSocket vs HTTP:** WebSocket connections have separate rate limits from HTTP requests.
4. **Response Headers:** Rate-limited responses include headers to help clients understand the rate limit status.

### Exceeded Rate Limit Response

```json
{
  "error": "rate_limited",
  "message": "Too many requests",
  "retry_after": 5
}
```

---

## Examples

### cURL Examples

#### Chat Completions (Non-Streaming)

```bash
curl -X POST http://localhost:3080/v1/chat/completions \
  -H "Authorization: Bearer my-token" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-4o",
    "messages": [{"role": "user", "content": "Hello!"}],
    "stream": false
  }'
```

#### Chat Completions (Streaming)

```bash
curl -X POST http://localhost:3080/v1/chat/completions \
  -H "Authorization: Bearer my-token" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-4o",
    "messages": [{"role": "user", "content": "Hello!"}],
    "stream": true
  }' | while read line; do echo "$line" | sed -n 's/^data: //p'; done
```

#### WebSocket Connection (Node.js)

```javascript
const WebSocket = require('ws');

const ws = new WebSocket('ws://localhost:3080/ws', {
  headers: {
    'Authorization': 'Bearer my-token'
  }
});

ws.on('open', () => {
  ws.send(JSON.stringify({
    jsonrpc: '2.0',
    method: 'chat.send',
    params: { text: 'Hello!' },
    id: 1
  }));
});

ws.on('message', (data) => {
  console.log(JSON.parse(data));
});
```

### JavaScript Example

```javascript
// Chat completions
const response = await fetch('http://localhost:3080/v1/chat/completions', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer my-token',
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    model: 'gpt-4o',
    messages: [{ role: 'user', content: 'Hello!' }]
  })
});

const data = await response.json();
console.log(data.choices[0].message.content);
```

---

## API Versioning

The gateway uses a simple versioning scheme:

- **HTTP API:** Version is included in the `/status` response
- **WebSocket:** Protocol version is negotiated via `X-Aisopod-Protocol-Version` header

Current version: `0.1.0`

---

## Changelog

### 0.1.0 (Initial Release)

- REST API endpoints: `/v1/chat/completions`, `/v1/responses`, `/hooks/{channel_type}`, `/health`, `/status`
- WebSocket API with JSON-RPC 2.0
- RPC methods: `system.ping`, `agent.list`, `chat.send`, `session.create/list`
- Authentication: Bearer token and query parameter
- Rate limiting: Per-token and per-IP limits
- Error codes and response formats
