# Issue 192: Write REST and WebSocket API Reference

## Summary
Create a complete API reference covering all REST endpoints and WebSocket RPC methods exposed by the Aisopod gateway, including request/response schemas, authentication, error codes, and rate limiting.

## Location
- Crate: N/A (documentation)
- File: `docs/book/src/api-reference.md`

## Current Behavior
API endpoints and WebSocket methods are implemented in code but lack user-facing reference documentation. Consumers must read handler source code or integration tests to understand request formats and response shapes.

## Expected Behavior
A comprehensive API reference at `docs/book/src/api-reference.md` that documents every REST endpoint and WebSocket method with schemas, examples, error codes, and authentication details — enabling third-party integrations without reading source code.

## Impact
The API is the programmatic interface for all integrations, front-ends, and automations built on Aisopod. Complete API documentation is essential for ecosystem growth and third-party adoption.

## Suggested Implementation

1. **Create** `docs/book/src/api-reference.md` with the following sections:

2. **Authentication section:**
   ```markdown
   ## Authentication

   All API requests require authentication (unless `auth.mode = "none"`).

   ### Bearer Token
   \```
   Authorization: Bearer <your-token>
   \```

   ### Query Parameter (for webhooks)
   \```
   GET /hooks/telegram?token=<your-token>
   \```

   Unauthenticated requests return `401 Unauthorized`.
   ```

3. **REST API Endpoints section:**
   ```markdown
   ## REST API

   Base URL: `http://localhost:3080`

   ### `POST /v1/chat/completions`

   OpenAI-compatible chat completions endpoint.

   **Request:**
   \```json
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
   \```

   **Response (non-streaming):**
   \```json
   {
     "id": "chatcmpl-abc123",
     "object": "chat.completion",
     "created": 1700000000,
     "model": "gpt-4o",
     "choices": [
       {
         "index": 0,
         "message": {"role": "assistant", "content": "Hello! How can I help?"},
         "finish_reason": "stop"
       }
     ],
     "usage": {"prompt_tokens": 12, "completion_tokens": 8, "total_tokens": 20}
   }
   \```

   **Response (streaming, `stream: true`):**
   Server-Sent Events (SSE) with `data: {...}` chunks.

   ---

   ### `POST /v1/responses`

   Create a response using the Responses API format.

   **Request:**
   \```json
   {
     "model": "gpt-4o",
     "input": "Explain Rust ownership in one paragraph."
   }
   \```

   **Response:**
   \```json
   {
     "id": "resp-abc123",
     "output": [
       {
         "type": "message",
         "content": [{"type": "text", "text": "Rust ownership..."}]
       }
     ]
   }
   \```

   ---

   ### `POST /hooks/{channel_type}`

   Webhook receiver for channel integrations (Telegram, Slack, etc.).

   **Telegram example:**
   \```
   POST /hooks/telegram
   Content-Type: application/json

   { "update_id": 123, "message": { ... } }
   \```

   Returns `200 OK` on success.

   ---

   ### `GET /health`

   Health check endpoint.

   **Response:**
   \```json
   {"status": "ok", "version": "0.1.0", "uptime_seconds": 3600}
   \```

   ---

   ### `GET /status`

   Detailed gateway status.

   **Response:**
   \```json
   {
     "agents": [{"name": "default", "model": "gpt-4o", "status": "active"}],
     "channels": [{"type": "telegram", "status": "connected"}],
     "memory_mb": 128,
     "active_sessions": 42
   }
   \```
   ```

4. **WebSocket API section:**
   ```markdown
   ## WebSocket API

   Connect to: `ws://localhost:3080/ws`

   ### Connection Handshake
   \```
   GET /ws HTTP/1.1
   Upgrade: websocket
   Authorization: Bearer <token>
   \```

   ### Message Format (JSON-RPC 2.0)
   \```json
   {"jsonrpc": "2.0", "method": "chat.send", "params": {...}, "id": 1}
   \```

   ### RPC Methods

   | Method              | Description                          |
   |---------------------|--------------------------------------|
   | `chat.send`         | Send a message to an agent           |
   | `chat.stream`       | Stream a response from an agent      |
   | `session.create`    | Create a new conversation session    |
   | `session.list`      | List active sessions                 |
   | `session.delete`    | Delete a session                     |
   | `agent.list`        | List configured agents               |
   | `agent.status`      | Get agent status                     |

   ### Event Types (server → client)

   | Event               | Description                          |
   |---------------------|--------------------------------------|
   | `chat.chunk`        | Streaming response chunk             |
   | `chat.complete`     | Response complete                    |
   | `chat.error`        | Error during generation              |
   | `session.updated`   | Session metadata changed             |
   ```

5. **Error Codes Reference section:**
   ```markdown
   ## Error Codes

   | HTTP Code | Error               | Description                    |
   |-----------|---------------------|--------------------------------|
   | 400       | `bad_request`       | Malformed request body         |
   | 401       | `unauthorized`      | Missing or invalid auth        |
   | 403       | `forbidden`         | Insufficient permissions       |
   | 404       | `not_found`         | Endpoint or resource not found |
   | 429       | `rate_limited`      | Too many requests              |
   | 500       | `internal_error`    | Unexpected server error        |
   | 502       | `upstream_error`    | LLM provider returned error   |
   | 503       | `unavailable`       | Gateway is starting/stopping   |
   ```

6. **Rate Limiting section:**
   ```markdown
   ## Rate Limiting

   Default limits (configurable in `config.toml`):

   | Scope       | Limit                |
   |-------------|----------------------|
   | Per-token   | 60 requests/minute   |
   | Per-IP      | 120 requests/minute  |
   | WebSocket   | 30 messages/minute   |

   Rate-limited responses include:
   \```
   Retry-After: 5
   X-RateLimit-Remaining: 0
   X-RateLimit-Reset: 1700000060
   \```
   ```

7. **Update `SUMMARY.md`** to link to this page.

## Dependencies
- Issue 187 (documentation infrastructure)
- Issue 037 (gateway tests validate REST endpoints)
- Issue 169 (protocol tests validate WebSocket methods)

## Acceptance Criteria
- [ ] `docs/book/src/api-reference.md` exists and is linked from `SUMMARY.md`
- [ ] All REST endpoints documented: `/v1/chat/completions`, `/v1/responses`, `/hooks/{type}`, `/health`, `/status`
- [ ] Request and response schemas shown with JSON examples
- [ ] Authentication methods documented
- [ ] WebSocket connection handshake documented
- [ ] All WebSocket RPC methods and event types listed with descriptions
- [ ] Error codes reference is complete
- [ ] Rate limiting behavior documented
- [ ] `mdbook build` succeeds with this page included

---
*Created: 2026-02-15*
