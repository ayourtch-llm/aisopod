# 0017 — Mobile/Desktop App Protocol

**Master Plan Reference:** Section 3.17 — Mobile/Desktop App Protocol  
**Phase:** 7 (Production)  
**Dependencies:** 0003 (Gateway Server), 0009 (Channel Abstraction)

---

## Objective

Define and document the WebSocket protocol used by native clients (iOS, Android,
macOS) to communicate with the aisopod gateway, including device pairing,
authentication, and real-time messaging.

---

## Deliverables

### 1. Protocol Specification

Document the complete WebSocket protocol:

**Connection handshake:**
```
Client → Server: WebSocket upgrade request
  Headers:
    - Authorization: Bearer <device_token>
    - X-Aisopod-Client: <client_type>/<version>
    - X-Aisopod-Device-Id: <device_id>

Server → Client: 101 Switching Protocols
Server → Client: { "method": "welcome", "params": { "version": "...", "features": [...] } }
```

**Message format (JSON-RPC 2.0):**
```json
// Request
{ "id": "uuid", "method": "chat.send", "params": { "text": "Hello" } }

// Response
{ "id": "uuid", "result": { "status": "ok" } }

// Error
{ "id": "uuid", "error": { "code": -32600, "message": "Invalid request" } }

// Server event (broadcast)
{ "method": "agent.event", "params": { "type": "text_delta", "data": "..." } }
```

### 2. Device Pairing Protocol

Port the device pairing flow:

```
1. Client requests pairing code:
   → { "method": "node.pair.request", "params": { "device_info": {...} } }

2. Server generates code, displays in UI:
   ← { "result": { "code": "ABCD-1234", "expires_at": "..." } }

3. User enters code on paired device or confirms in UI:
   → { "method": "node.pair.confirm", "params": { "code": "ABCD-1234" } }

4. Server issues device token:
   ← { "result": { "device_token": "...", "device_id": "..." } }
```

### 3. Node Role Methods

Methods available to connected device nodes:

| Method              | Description                           |
|---------------------|---------------------------------------|
| `node.describe`     | Describe device capabilities          |
| `node.invoke`       | Invoke device service (camera, etc.)  |
| `node.pair.request` | Initiate pairing                      |
| `node.pair.confirm` | Confirm pairing code                  |
| `node.pair.revoke`  | Revoke device access                  |

### 4. Device Capabilities

Devices can expose services to the AI agent:

```json
{
  "method": "node.describe",
  "result": {
    "device_id": "...",
    "platform": "ios",
    "capabilities": [
      { "service": "camera", "methods": ["capture", "stream"] },
      { "service": "location", "methods": ["current", "track"] },
      { "service": "calendar", "methods": ["list", "create"] },
      { "service": "contacts", "methods": ["search", "get"] }
    ]
  }
}
```

### 5. Canvas Protocol

For interactive UI rendering on devices:

```json
// Server sends canvas content
{
  "method": "canvas.update",
  "params": {
    "canvas_id": "...",
    "content": "<html>...</html>",
    "type": "html"
  }
}

// Client sends canvas interaction
{
  "method": "canvas.interact",
  "params": {
    "canvas_id": "...",
    "event": { "type": "click", "target": "..." }
  }
}
```

### 6. Backward Compatibility

- Document breaking changes from OpenClaw protocol
- Provide protocol version negotiation
- Migration guide for existing native apps

### 7. Reference Implementation

- Rust WebSocket client library for protocol testing
- Protocol conformance test suite
- Example client implementation (CLI-based)

---

## Acceptance Criteria

- [ ] Protocol specification is complete and documented
- [ ] Device pairing flow works end-to-end
- [ ] Node capabilities describe device services accurately
- [ ] Canvas protocol renders interactive content
- [ ] Protocol version negotiation handles version mismatches
- [ ] Reference client connects and communicates correctly
- [ ] Conformance test suite validates protocol compliance
- [ ] Documentation covers all protocol messages and flows
