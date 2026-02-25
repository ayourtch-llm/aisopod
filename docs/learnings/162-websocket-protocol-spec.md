# WebSocket Protocol Specification Documentation (Issue #162)

## Summary

This issue implemented a comprehensive WebSocket protocol specification document for the Aisopod project. The documentation defines the complete JSON-RPC 2.0 message format, connection lifecycle, and all method namespaces used by aisopod mobile/desktop app clients.

## Learning: Documentation-Driven Development

### Key Insight

Writing a protocol specification **before** or **alongside** implementation provides several benefits:

1. **Clear Contract**: The specification serves as the contract between client and server
2. **Implementation Guidance**: Developers can implement clients based on the spec
3. **Testing Foundation**: The spec can be used to create conformance test suites
4. **Onboarding**: New team members can quickly understand the protocol

### Approach Used

For this issue, we:

1. Examined the existing codebase to understand the current protocol implementation
2. Read Issue 029 (JSON-RPC message parsing) and Issue 030 (RPC method router) for context
3. Reviewed the gateway code to understand:
   - WebSocket upgrade flow
   - Authentication headers
   - Message formats
   - Method namespaces
   - Error handling

### Documentation Structure

The specification is organized as follows:

```
docs/protocol/websocket-protocol.md
├── Overview
├── Connection Lifecycle (Handshake Headers)
├── Message Format
│   ├── JSON-RPC 2.0 Request
│   ├── JSON-RPC 2.0 Response
│   ├── JSON-RPC 2.0 Error
│   └── Broadcast/Event Notification
├── Welcome Message
├── Method Reference
│   ├── chat.* (Chat Methods)
│   ├── agent.* (Agent Management)
│   ├── node.* (Node/Device Methods)
│   ├── canvas.* (Canvas/UI Methods)
│   ├── system.* (System Methods)
│   ├── approval.* (Approval Methods)
│   └── gateway.* (Gateway Methods)
├── Error Codes
│   ├── Standard JSON-RPC 2.0 Codes
│   └── Custom Aisopod Codes
├── Security Considerations
└── Example Sessions
    ├── Authentication Flow
    ├── Chat Interaction
    └── Agent Management
```

## Learning: JSON-RPC 2.0 Protocol Details

### Message Types

1. **Request** (`jsonrpc`, `id`, `method`, `params`)
2. **Response** (`jsonrpc`, `id`, `result` or `error`)
3. **Error** (Response with `error` field)
4. **Notification** (Request without `id` or Response without `id`)

### Key Constraints

- `jsonrpc` must be exactly `"2.0"`
- `method` uses `namespace.method` naming convention
- `id` can be any JSON value (string, number, or null for notifications)
- Notifications have no `id` field

## Learning: Method Namespace Organization

The protocol uses a hierarchical naming convention:

```
namespace.method
```

Where namespace groups related functionality:

| Namespace | Purpose |
|-----------|---------|
| `chat.*` | Chat messaging and streaming |
| `agent.*` | Agent lifecycle management |
| `node.*` | Device/node management |
| `canvas.*` | UI canvas interactions |
| `system.*` | System-level operations |
| `approval.*` | Approval workflow |
| `gateway.*` | Gateway subscription management |

## Learning: Authentication Flow

### WebSocket Upgrade Headers

| Header | Required | Format |
|--------|----------|--------|
| `Authorization` | Yes | `Bearer <token>` or `DeviceToken <device_token>` |
| `X-Aisopod-Client` | Yes | `<client_name>/<version>` |
| `X-Aisopod-Device-Id` | Yes | UUID |

### Server Welcome

After successful connection, server sends:

```json
{
  "jsonrpc": "2.0",
  "method": "system.welcome",
  "params": {
    "server_version": "0.1.0",
    "protocol_version": "1.0",
    "session_id": "uuid",
    "capabilities": ["chat", "canvas", "node", "approval"]
  }
}
```

## Learning: Error Code Conventions

### Standard JSON-RPC 2.0 Codes

| Code | Name |
|------|------|
| -32700 | Parse Error |
| -32600 | Invalid Request |
| -32601 | Method Not Found |
| -32602 | Invalid Params |
| -32603 | Internal Error |

### Custom Aisopod Codes

| Code | Name |
|------|------|
| -32001 | Unauthorized |
| -32002 | Rate Limited |
| -32003 | Device Not Paired |
| -32004 | Agent Not Found |
| -32005 | Session Not Found |
| -32006 | Permission Denied |
| -32007 | Conflict |
| -32008 | Timeout |
| -32009 | Resource Exhausted |
| -32010 | Unsupported |

## Learning: Streaming and Notifications

### Chat Streaming

The `chat.send` method demonstrates streaming:
1. Client sends request
2. Server returns immediate acknowledgment
3. Server streams `chat.response` events
4. Events include text deltas, tool calls, and usage stats
5. Final event marks `done: true`

### Broadcast Events

Server sends events without `id` field:
```json
{
  "jsonrpc": "2.0",
  "method": "gateway.event",
  "params": {
    "type": "agent",
    "agent_id": "agent-123",
    "event": "created"
  }
}
```

## Learning: Security Considerations

### TLS Requirements

- Production must use `wss://` (WebSocket Secure)
- Server certificates must be validated by clients

### Token Management

- Bearer tokens: Sent in `Authorization: Bearer <token>`
- Device tokens: Sent in `Authorization: DeviceToken <device_token>`
- Both must expire and support rotation

### Rate Limiting

- Connection rate limits per IP
- Request rate limits per connection
- Return `-32002 Rate Limited` with `retry_after`

### Audit Logging

- Log authentication attempts
- Log authorization decisions
- Log rate limit violations
- Log malformed requests

## Learning: Testing Strategy

### Library Tests

All library tests pass with `RUSTFLAGS=-Awarnings`:

```bash
cargo test -p aisopod-gateway --lib
```

### Integration Tests

Two pre-existing integration tests fail due to authentication endpoint implementation issues (unrelated to this documentation change). These tests expect 401 UNAUTHORIZED but get 501 NOT_IMPLEMENTED because the endpoint isn't fully implemented.

## Conclusion

This documentation provides a complete reference for:
1. Client implementations (iOS, Android, desktop)
2. Server verification and compliance testing
3. New feature development aligned with protocol
4. Protocol evolution tracking

The specification was derived from examining the actual implementation, ensuring the documentation accurately reflects the running code.

---

*Created: 2026-02-25*
*Issue: #162*
