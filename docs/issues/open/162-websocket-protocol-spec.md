# Issue 162: Write WebSocket Protocol Specification Document

## Summary
Create a comprehensive WebSocket protocol specification document that defines the complete JSON-RPC 2.0 message format, connection lifecycle, and all method namespaces used by aisopod mobile/desktop app clients.

## Location
- Crate: N/A (documentation only)
- File: `docs/protocol/websocket-protocol.md`

## Current Behavior
No formal protocol specification exists. Client implementations must reverse-engineer the expected message formats and connection flow from source code.

## Expected Behavior
A complete protocol specification document that any developer can use to build a conformant aisopod client. The document covers:

1. **Connection Handshake** — WebSocket upgrade with required headers:
   - `Authorization: Bearer <token>` or `Authorization: DeviceToken <device_token>`
   - `X-Aisopod-Client: <client_name>/<version>` (e.g. `aisopod-ios/1.0.0`)
   - `X-Aisopod-Device-Id: <uuid>` unique per device installation
2. **Welcome Message** — Server sends immediately after connection:
   ```json
   {
     "jsonrpc": "2.0",
     "method": "system.welcome",
     "params": {
       "server_version": "0.1.0",
       "protocol_version": "1.0",
       "session_id": "uuid",
       "capabilities": ["chat", "canvas", "node"]
     }
   }
   ```
3. **Request Format** (client → server):
   ```json
   {
     "jsonrpc": "2.0",
     "id": "unique-request-id",
     "method": "namespace.method",
     "params": { }
   }
   ```
4. **Response Format** (server → client):
   ```json
   {
     "jsonrpc": "2.0",
     "id": "unique-request-id",
     "result": { }
   }
   ```
5. **Error Format**:
   ```json
   {
     "jsonrpc": "2.0",
     "id": "unique-request-id",
     "error": {
       "code": -32600,
       "message": "Invalid request",
       "data": { }
     }
   }
   ```
6. **Broadcast/Event Format** (server → client, no `id`):
   ```json
   {
     "jsonrpc": "2.0",
     "method": "event.namespace.name",
     "params": { }
   }
   ```
7. **Method Namespaces** with full params/result schemas:
   - `chat.*` — send messages, receive streaming tokens
   - `agent.*` — list, create, update, delete agents
   - `node.*` — pair, describe, invoke device capabilities
   - `canvas.*` — update, interact with UI canvases
   - `system.*` — welcome, ping/pong, version info

## Impact
This specification is the foundation for all client implementations (iOS, Android, desktop) and the conformance test suite. Without it, protocol changes risk breaking clients silently.

## Suggested Implementation
1. Create the file `docs/protocol/websocket-protocol.md`.
2. Start with a header and table of contents:
   ```markdown
   # Aisopod WebSocket Protocol Specification v1.0

   ## Table of Contents
   1. Overview
   2. Connection Lifecycle
   3. Message Format
   4. Method Reference
   5. Error Codes
   6. Security Considerations
   ```
3. Write each section following the structures described in Expected Behavior above.
4. For each method namespace, create a subsection with a table:
   ```markdown
   ### chat.send
   | Field       | Type   | Required | Description          |
   |-------------|--------|----------|----------------------|
   | agent_id    | string | yes      | Target agent UUID    |
   | message     | string | yes      | User message content |
   ```
5. Define standard error codes (use JSON-RPC 2.0 standard codes plus custom ranges):
   - `-32700` Parse error
   - `-32600` Invalid request
   - `-32601` Method not found
   - `-32001` Unauthorized
   - `-32002` Rate limited
   - `-32003` Device not paired
6. Add a Security Considerations section covering TLS requirements, token rotation, and rate limiting.

## Dependencies
- Issue 029 (JSON-RPC message format)
- Issue 030 (RPC method definitions)

## Acceptance Criteria
- [ ] `docs/protocol/websocket-protocol.md` exists and is complete
- [ ] All method namespaces are documented with params and result schemas
- [ ] Connection handshake flow is fully specified with required headers
- [ ] Welcome message format is defined
- [ ] Error codes and their meanings are listed
- [ ] Document passes review by at least one other contributor

---
*Created: 2026-02-15*
