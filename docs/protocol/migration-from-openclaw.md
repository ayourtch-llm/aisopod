# Migration Guide: OpenClaw → Aisopod Protocol

## Overview

This guide covers all breaking changes between the OpenClaw WebSocket protocol and the Aisopod protocol v1.0. Developers maintaining OpenClaw-based clients will need to update their implementations to work with Aisopod.

**Important Notes:**
- This is a **protocol migration**, not just a rename
- Aisopod introduces **new capabilities** and **removes some OpenClaw features**
- The protocol is now formally specified and versioned
- Device pairing is now required for device-specific operations

## Method Name Changes

OpenClaw used `claw.*` namespaces; Aisopod uses `node.*`, `chat.*`, and other namespaces.

| OpenClaw Method      | Aisopod Method       | Notes                                    |
|----------------------|----------------------|------------------------------------------|
| `claw.execute`       | `node.invoke`        | Params restructured (service/method)     |
| `claw.describe`      | `node.describe`      | Added `capabilities` array field         |
| `claw.send`          | `chat.send`          | Agent execution via chat                 |
| `claw.agent.list`    | `agent.list`         | Agent management                         |
| `claw.agent.create`  | `agent.create`       |                                          |
| `claw.agent.update`  | `agent.update`       |                                          |
| `claw.agent.delete`  | `agent.delete`       |                                          |
| `claw.agent.start`   | `agent.start`        |                                          |
| `claw.agent.stop`    | `agent.stop`         |                                          |
| `claw.canvas.update` | `canvas.update`      | Canvas/UI state management               |
| `claw.canvas.interact`| `canvas.interact`   | Canvas user interactions                 |
| -                    | `node.pair.request`  | **NEW**: Device pairing initiation       |
| -                    | `node.pair.confirm`  | **NEW**: Device pairing confirmation     |
| -                    | `node.pair.revoke`   | **NEW**: Device pairing revocation       |
| -                    | `approval.request`   | **NEW**: Approval workflow               |
| -                    | `approval.approve`   | **NEW**: Approval confirmation           |
| -                    | `approval.deny`      | **NEW**: Approval denial                 |
| -                    | `approval.list`      | **NEW**: List pending approvals          |
| -                    | `gateway.subscribe`  | **NEW**: Runtime subscription updates    |
| -                    | `system.welcome`     | **NEW**: Server sends on connect         |
| -                    | `system.ping`        | **NEW**: Keep-alive mechanism            |
| -                    | `system.version`     | **NEW**: Protocol version negotiation    |

## Parameter Changes

### node.invoke (was claw.execute)

**OpenClaw Format:**
```json
{
  "jsonrpc": "2.0",
  "id": "request-123",
  "method": "claw.execute",
  "params": {
    "target": "devices/123/services/camera/takePhoto",
    "args": {
      "resolution": "high",
      "flash": true
    }
  }
}
```

**Aisopod Format:**
```json
{
  "jsonrpc": "2.0",
  "id": "request-123",
  "method": "node.invoke",
  "params": {
    "service": "camera",
    "method": "takePhoto",
    "params": {
      "resolution": "high",
      "flash": true
    },
    "timeout_ms": 30000,
    "device_id": "device-123-uuid"
  }
}
```

**Changes:**
- Removed: `target` field (single path string)
- Added: `service` field (service name only)
- Added: `method` field (method name within service)
- Changed: `args` renamed to `params` (to match JSON-RPC convention)
- Added: `timeout_ms` field (optional, default 30000ms)
- Added: `device_id` field (optional, for routing to specific device)

### node.describe (was claw.describe)

**OpenClaw Format:**
```json
{
  "jsonrpc": "2.0",
  "id": "request-456",
  "method": "claw.describe",
  "params": {
    "capabilities": ["camera", "location", "contacts"]
  }
}
```

**Aisopod Format:**
```json
{
  "jsonrpc": "2.0",
  "id": "request-456",
  "method": "node.describe",
  "params": {
    "capabilities": [
      {
        "service": "camera",
        "methods": ["takePhoto", "listPhotos"],
        "metadata": {
          "max_resolution": "4k",
          "supports_flash": true
        }
      },
      {
        "service": "location",
        "methods": ["getCurrentLocation", "startTracking"],
        "metadata": {
          "accuracy": "high",
          "update_interval_ms": 5000
        }
      }
    ]
  }
}
```

**Changes:**
- The `capabilities` field now accepts an array of objects instead of simple strings
- Each capability object includes: `service`, `methods` array, and optional `metadata`
- This allows the server to understand exactly what each device can do

### chat.send

**OpenClaw Format:**
```json
{
  "jsonrpc": "2.0",
  "id": "request-789",
  "method": "claw.send",
  "params": {
    "message": "Hello, agent!",
    "agent_id": "agent-123"
  }
}
```

**Aisopod Format:**
```json
{
  "jsonrpc": "2.0",
  "id": "request-789",
  "method": "chat.send",
  "params": {
    "text": "Hello, agent!",
    "agent": "agent-123",
    "channel": "slack-channel-456"
  }
}
```

**Changes:**
- Changed: `message` renamed to `text`
- Changed: `agent_id` renamed to `agent`
- Added: `channel` field (optional, for multi-channel support)

## Authentication Changes

### Headers

OpenClaw used simple token headers; Aisopod uses standardized HTTP authentication with additional device/client metadata.

| OpenClaw Header      | Aisopod Header                      | Required | Format                          |
|----------------------|-------------------------------------|----------|---------------------------------|
| `X-OpenClaw-Token`   | `Authorization`                     | Yes      | `Bearer <token>`                |
| -                    | `Authorization`                     | Yes      | `DeviceToken <device_token>`    |
| -                    | `X-Aisopod-Client`                  | Yes      | `<client_name>/<version>`       |
| -                    | `X-Aisopod-Device-Id`               | Yes      | UUID (e.g., `550e8400-e29b-41d4`) |
| -                    | `X-Aisopod-Protocol-Version`        | No       | `1.0`                           |

### Connection Flow

**OpenClaw:**
1. Client connects with `X-OpenClaw-Token` header
2. Server authenticates token
3. Connection established

**Aisopod:**
1. Client connects with required headers:
   - `Authorization: Bearer <token>` (for API access)
   - OR `Authorization: DeviceToken <device_token>` (for device access)
   - `X-Aisopod-Client: aisopod-ios/1.0.0`
   - `X-Aisopod-Device-Id: 550e8400-e29b-41d4-a716-446655440000`
2. Server authenticates and validates headers
3. Server sends `system.welcome` message with capabilities
4. Device pairing flow may be initiated if needed

### Welcome Message

Aisopod sends a welcome message immediately after connection:

```json
{
  "jsonrpc": "2.0",
  "method": "system.welcome",
  "params": {
    "server_version": "0.1.0",
    "protocol_version": "1.0",
    "session_id": "session-uuid-here",
    "capabilities": ["chat", "canvas", "node", "approval"]
  }
}
```

**Purpose:**
- Inform client of server version and protocol version
- Provide session ID for tracking
- List available capabilities

## Environment Variable Renames

The following environment variables have been renamed to follow Aisopod naming conventions.

| OpenClaw Variable          | Aisopod Variable                       | Purpose                          |
|----------------------------|----------------------------------------|----------------------------------|
| `OPENCLAW_SERVER_PORT`     | `AISOPOD_GATEWAY_SERVER_PORT`          | Gateway server port              |
| `OPENCLAW_SERVER_HOST`     | `AISOPOD_GATEWAY_BIND_ADDRESS`         | Gateway bind address             |
| `OPENCLAW_SERVER_BIND`     | `AISOPOD_GATEWAY_BIND_ADDRESS`         | Gateway bind address (alias)     |
| `OPENCLAW_MODEL_API_KEY`   | `AISOPOD_MODELS_PROVIDERS_0_API_KEY`   | API key for first provider       |
| `OPENCLAW_MODEL_ENDPOINT`  | `AISOPOD_MODELS_PROVIDERS_0_ENDPOINT`  | Endpoint for first provider      |
| `OPENCLAW_MODEL_NAME`      | `AISOPOD_MODELS_PROVIDERS_0_NAME`      | Name for first provider          |
| `OPENCLAW_DEFAULT_PROVIDER`| `AISOPOD_MODELS_DEFAULT_PROVIDER`      | Default provider name            |
| `OPENCLAW_TOOLS_ENABLED`   | `AISOPOD_TOOLS_BASH_ENABLED`           | Enable bash tool                 |
| `OPENCLAW_SESSION_ENABLED` | `AISOPOD_SESSION_ENABLED`              | Enable session management        |
| `OPENCLAW_MEMORY_ENABLED`  | `AISOPOD_MEMORY_ENABLED`               | Enable memory system             |

### Configuration File Mappings

When migrating config files (JSON5 format), these key mappings apply:

| OpenClaw Config Path       | Aisopod Config Path                    |
|----------------------------|----------------------------------------|
| `server.port`              | `gateway.server.port`                  |
| `server.host`              | `gateway.bind.address`                 |
| `models.providers[].name`  | `models.providers[].name`              |
| `models.providers[].endpoint` | `models.providers[].endpoint`      |
| `models.providers[].api_key` | `models.providers[].api_key`        |
| `models.default_provider`  | `models.default_provider`              |
| `tools.bash.enabled`       | `tools.bash.enabled`                   |
| `session.enabled`          | `session.enabled`                      |
| `memory.enabled`           | `memory.enabled`                       |

## Protocol Version Mapping

### Version Numbering

**OpenClaw:**
- No formal versioning system
- Clients and servers were expected to be compatible by chance
- No way to negotiate versions

**Aisopod:**
- Formal protocol versioning: `1.0`
- Version negotiation via `X-Aisopod-Protocol-Version` header
- Graceful handling of version mismatches

### Version Header

Clients can indicate their protocol version:

```http
X-Aisopod-Protocol-Version: 1.0
```

### Server Response

If the server supports the requested version, it sends a welcome message.
If not, it closes the connection with an error.

### Compatibility Matrix

| Protocol Version | Server Support | Client Action                     |
|------------------|----------------|-----------------------------------|
| 1.0              | Full support   | Normal operation                  |
| (no header)      | Default to 1.0 | Use protocol 1.0                  |
| Other            | Version mismatch | Connection closed with error   |

## New Features (No OpenClaw Equivalent)

Aisopod introduces several features that did not exist in OpenClaw:

### Device Pairing

**Purpose:** Secure device registration with explicit user consent

**Methods:**
- `node.pair.request` - Initiate pairing from device
- `node.pair.confirm` - Confirm pairing on server
- `node.pair.revoke` - Remove device pairing

**Flow:**
1. Device sends `node.pair.request` with pairing code
2. User confirms pairing (via web UI, CLI, or API)
3. Server sends `node.pair.confirm` response
4. Device can now invoke capabilities

**Example - Request:**
```json
{
  "jsonrpc": "2.0",
  "id": "pair-1",
  "method": "node.pair.request",
  "params": {
    "device_name": "iPhone-14-Pro",
    "device_type": "mobile",
    "pairing_code": "ABCD-1234"
  }
}
```

**Example - Confirm:**
```json
{
  "jsonrpc": "2.0",
  "id": "pair-2",
  "method": "node.pair.confirm",
  "params": {
    "device_id": "device-uuid-here",
    "token": "device-token-here"
  }
}
```

### Canvas Protocol

**Purpose:** UI canvas state management for mobile/desktop apps

**Methods:**
- `canvas.update` - Push canvas state updates
- `canvas.interact` - Handle user interactions

**Use Cases:**
- Syncing UI state across devices
- Collaborative editing
- Remote UI control

### Protocol Version Negotiation

**Purpose:** Ensure client and server are compatible

**Implementation:**
- Client sends `X-Aisopod-Protocol-Version` header
- Server validates and accepts/rejects
- Version info in welcome message

### Welcome Message

**Purpose:** Inform clients of server capabilities on connect

**Content:**
- Server version
- Protocol version
- Session ID
- List of supported capabilities

### Approval Workflow

**Purpose:** Require user approval for potentially dangerous operations

**Methods:**
- `approval.request` - Request approval for an action
- `approval.approve` - Approve a pending request
- `approval.deny` - Deny a pending request
- `approval.list` - List pending approvals

**Use Cases:**
- Bash command execution approval
- File operation approval
- Agent execution approval

### System Ping/Pong

**Purpose:** Keep connection alive and detect disconnects

**Implementation:**
- Server sends `system.ping` periodically
- Client responds with `system.pong`
- Connection closed if pong not received

## Removed Features

The following OpenClaw features were intentionally not carried forward to Aisopod:

### Direct Device Targeting

**OpenClaw:**
- Devices could be targeted via path strings like `devices/123/services/camera`
- No explicit pairing requirement

**Aisopod:**
- Devices must be paired before invocation
- Targeting uses `device_id` UUID in `node.invoke` params
- More secure: pairing ensures explicit consent

### Flexible Response Streaming

**OpenClaw:**
- Responses could include streaming data in various formats
- No standardized streaming protocol

**Aisopod:**
- Streaming is handled via specific methods
- `chat.send` uses `chat.response` events for streaming
- More predictable and easier to implement clients

### Unstructured Metadata

**OpenClaw:**
- Metadata was passed ad-hoc in various fields
- No consistent format

**Aisopod:**
- Metadata is structured in capability definitions
- Consistent format across all features

## Migration Checklist

### Client Implementation

- [ ] Replace all `claw.*` method calls with `node.*`, `chat.*`, etc.
- [ ] Update authentication headers
  - Replace `X-OpenClaw-Token` with `Authorization: Bearer <token>`
  - Add `X-Aisopod-Client` header
  - Add `X-Aisopod-Device-Id` header
- [ ] Implement device pairing flow (if applicable)
- [ ] Handle welcome message on connect
- [ ] Update method parameters (service/method instead of target)
- [ ] Implement approval workflow (if needed)
- [ ] Update environment variables

### Server Configuration

- [ ] Update environment variables to AISOPOD_* prefix
- [ ] Verify protocol version header is sent
- [ ] Configure device pairing (if needed)
- [ ] Set up approval workflow (if needed)

### Testing

- [ ] Test WebSocket connection with new headers
- [ ] Test device pairing flow
- [ ] Test `node.invoke` with new params structure
- [ ] Test `node.describe` with capability objects
- [ ] Test welcome message handling
- [ ] Test protocol version negotiation

## Example Migration: Chat Application

### Before (OpenClaw)

```javascript
// Connect
const ws = new WebSocket('wss://api.openclaw.example.com/ws', {
  headers: {
    'X-OpenClaw-Token': openclawToken
  }
});

// Send message
ws.send(JSON.stringify({
  jsonrpc: '2.0',
  id: Date.now(),
  method: 'claw.send',
  params: {
    message: 'Hello!',
    agent_id: 'agent-123'
  }
}));
```

### After (Aisopod)

```javascript
// Connect
const ws = new WebSocket('wss://api.aisopod.example.com/ws', {
  headers: {
    'Authorization': `Bearer ${aisopodToken}`,
    'X-Aisopod-Client': 'my-app/1.0.0',
    'X-Aisopod-Device-Id': deviceUUID,
    'X-Aisopod-Protocol-Version': '1.0'
  }
});

// Handle welcome message
ws.onmessage = (event) => {
  const msg = JSON.parse(event.data);
  if (msg.method === 'system.welcome') {
    console.log('Connected to', msg.params.server_version);
    console.log('Capabilities:', msg.params.capabilities);
  }
};

// Send message
ws.send(JSON.stringify({
  jsonrpc: '2.0',
  id: Date.now(),
  method: 'chat.send',
  params: {
    text: 'Hello!',
    agent: 'agent-123',
    channel: 'slack-channel'
  }
}));
```

## Troubleshooting

### Connection Refused

**Symptoms:** WebSocket connection fails immediately

**Possible Causes:**
- Missing required headers (`Authorization`, `X-Aisopod-Client`, `X-Aisopod-Device-Id`)
- Invalid token format
- Protocol version mismatch

**Solution:**
- Verify all headers are present
- Check token validity
- Add `X-Aisopod-Protocol-Version: 1.0` header

### Method Not Found

**Symptoms:** Server returns error code -32601

**Possible Causes:**
- Using old `claw.*` method names
- Typo in method name

**Solution:**
- Update method names per the migration table
- Check spelling and namespace

### Invalid Parameters

**Symptoms:** Server returns error code -32602

**Possible Causes:**
- Using old parameter structure (`target` instead of `service`/`method`)
- Missing required fields

**Solution:**
- Update parameter structure per documentation
- Check required fields for each method

### Device Not Paired

**Symptoms:** `node.invoke` or `node.describe` fails

**Possible Causes:**
- Device not paired yet
- Device pairing expired

**Solution:**
- Implement device pairing flow
- Handle re-pairing scenarios

## See Also

- [WebSocket Protocol Specification](websocket-protocol.md) - Complete protocol reference
- [Issue #162 - WebSocket Protocol Specification](../issues/resolved/162-websocket-protocol-spec.md)
- [Configuration Migration Guide](../issues/resolved/161-openclaw-migration-deployment-tests.md)

---

*Created: 2026-02-26*
*Issue: #167*
