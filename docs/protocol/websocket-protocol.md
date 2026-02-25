# Aisopod WebSocket Protocol Specification v1.0

## Table of Contents

1. [Overview](#overview)
2. [Connection Lifecycle](#connection-lifecycle)
   - [Handshake Headers](#handshake-headers)
3. [Message Format](#message-format)
   - [JSON-RPC 2.0 Request](#json-rpc-20-request)
   - [JSON-RPC 2.0 Response](#json-rpc-20-response)
   - [JSON-RPC 2.0 Error](#json-rpc-20-error)
   - [Broadcast/Event Notification](#broadcastevent-notification)
4. [Welcome Message](#welcome-message)
5. [Method Reference](#method-reference)
   - [chat.* - Chat Methods](#chat---chat-methods)
   - [agent.* - Agent Management Methods](#agent---agent-management-methods)
   - [node.* - Node/Device Methods](#node---node-device-methods)
   - [canvas.* - Canvas/UI Methods](#canvas---canvasui-methods)
   - [system.* - System Methods](#system---system-methods)
   - [approval.* - Approval Methods](#approval---approval-methods)
   - [gateway.* - Gateway Methods](#gateway---gateway-methods)
6. [Error Codes](#error-codes)
   - [Standard JSON-RPC 2.0 Error Codes](#standard-json-rpc-20-error-codes)
   - [Custom Aisopod Error Codes](#custom-aisopod-error-codes)
7. [Security Considerations](#security-considerations)
8. [Example Sessions](#example-sessions)
   - [Authentication Flow](#authentication-flow)
   - [Chat Interaction](#chat-interaction)
   - [Agent Management](#agent-management)

---

## Overview

The Aisopod WebSocket protocol defines a JSON-RPC 2.0-based communication channel between client applications (mobile, desktop, web) and the Aisopod gateway server. This protocol enables real-time interaction with AI agents, device management, and system control through a persistent WebSocket connection.

Key characteristics:
- **Transport**: WebSocket (ws:// or wss://)
- **Message Format**: JSON-RPC 2.0
- **Authentication**: Bearer tokens or device tokens via HTTP upgrade headers
- **Bidirectional**: Full-duplex communication for real-time interactions

---

## Connection Lifecycle

### Handshake Headers

When upgrading to a WebSocket connection, the client must provide the following HTTP headers in the upgrade request:

| Header | Required | Description | Example |
|--------|----------|-------------|---------|
| `Authorization` | Yes | Authentication token (Bearer or DeviceToken) | `Bearer eyJhbGciOiJIUzI1NiJ9...` or `DeviceToken device-uuid-here` |
| `X-Aisopod-Client` | Yes | Client identification | `aisopod-ios/1.0.0` or `aisopod-desktop/0.5.0` |
| `X-Aisopod-Device-Id` | Yes | Unique device identifier | `550e8400-e29b-41d4-a716-446655440000` |

#### Authorization Header Formats

The `Authorization` header supports two formats:

1. **Bearer Token** (for authenticated users):
   ```
   Authorization: Bearer <token>
   ```
   Where `<token>` is a JWT or session token issued by the authentication server.

2. **Device Token** (for device-based authentication):
   ```
   Authorization: DeviceToken <device_token>
   ```
   Where `<device_token>` is a unique identifier for the device installation.

#### Client Identification Headers

- **X-Aisopod-Client**: Identifies the client application and version
  - Format: `<client_name>/<version>`
  - Example: `aisopod-android/2.1.0`, `aisopod-web/1.0.0`
  - This helps server-side analytics and compatibility tracking

- **X-Aisopod-Device-Id**: Unique identifier for the device
  - Format: UUID (v4 recommended)
  - Example: `550e8400-e29b-41d4-a716-446655440000`
  - Used for tracking device-specific sessions and rate limiting

---

## Message Format

All messages on the WebSocket connection are JSON-encoded strings. There are four primary message types:

### JSON-RPC 2.0 Request

Client → Server: A request that expects a response.

```json
{
  "jsonrpc": "2.0",
  "id": "unique-request-id",
  "method": "namespace.method",
  "params": { }
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `jsonrpc` | string | Yes | Must be exactly `"2.0"` |
| `id` | string/number/null | Yes | Unique request identifier. Can be any JSON value. |
| `method` | string | Yes | Method name in `namespace.method` format |
| `params` | object/array/null | No | Method parameters. Structure depends on the method. |

**Example:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-001",
  "method": "chat.send",
  "params": {
    "text": "Hello, agent!",
    "agent": "agent-123"
  }
}
```

### JSON-RPC 2.0 Response

Server → Client: Response to a request.

```json
{
  "jsonrpc": "2.0",
  "id": "unique-request-id",
  "result": { }
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `jsonrpc` | string | Yes | Must be exactly `"2.0"` |
| `id` | string/number/null | Yes | Must match the request's `id` |
| `result` | any | No | Result of the operation. Omitted on error. |

**Example:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-001",
  "result": {
    "status": "success",
    "message_id": "msg-456"
  }
}
```

### JSON-RPC 2.0 Error

Server → Client: Response indicating an error occurred.

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

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `jsonrpc` | string | Yes | Must be exactly `"2.0"` |
| `id` | string/number/null | Yes | Must match the request's `id`. `null` for notification errors. |
| `error.code` | integer | Yes | Error code (see [Error Codes](#error-codes)) |
| `error.message` | string | Yes | Human-readable error description |
| `error.data` | any | No | Additional error context. Optional. |

**Example:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-001",
  "error": {
    "code": -32601,
    "message": "Method chat.send is not implemented",
    "data": null
  }
}
```

### Broadcast/Event Notification

Server → Client: Asynchronous event from the server (no `id` field).

```json
{
  "jsonrpc": "2.0",
  "method": "event.namespace",
  "params": { }
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `jsonrpc` | string | Yes | Must be exactly `"2.0"` |
| `id` | - | No | **Not present** in notifications |
| `method` | string | Yes | Event name (always starts with `gateway.` or method namespace) |
| `params` | any | Yes | Event data. Structure depends on the event type. |

**Example (Agent Created Event):**
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

**Example (Chat Response Stream):**
```json
{
  "jsonrpc": "2.0",
  "method": "chat.response",
  "params": {
    "text": "Hello! How can I help you today?",
    "done": false
  }
}
```

---

## Welcome Message

When a client establishes a WebSocket connection, the server sends an initial **welcome message** as a JSON-RPC notification.

```json
{
  "jsonrpc": "2.0",
  "method": "system.welcome",
  "params": {
    "server_version": "0.1.0",
    "protocol_version": "1.0",
    "session_id": "550e8400-e29b-41d4-a716-446655440000",
    "capabilities": ["chat", "canvas", "node", "approval"]
  }
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `jsonrpc` | string | Yes | Must be `"2.0"` |
| `method` | string | Yes | Always `"system.welcome"` |
| `params.server_version` | string | Yes | Version of the Aisopod server (e.g., `"0.1.0"`) |
| `params.protocol_version` | string | Yes | Version of the WebSocket protocol (e.g., `"1.0"`) |
| `params.session_id` | string | Yes | Unique session identifier for this connection |
| `params.capabilities` | array | Yes | List of enabled capabilities: `chat`, `canvas`, `node`, `approval` |

**Purpose:**
- Informs the client about server capabilities
- Establishes the session ID for this connection
- Allows client to adapt based on server version

---

## Method Reference

All methods follow the `namespace.method` naming convention.

### chat.* - Chat Methods

Methods for sending messages and receiving streaming responses.

#### chat.send

Send a message to an agent. Returns immediately with an acknowledgment, then streams responses via `chat.response` events.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `text` | string | Yes | The message content to send |
| `channel` | string | No | Session/channel ID for conversation continuity |
| `agent` | string | No | Agent ID to use. Defaults to default agent if not specified |

**Request Example:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-001",
  "method": "chat.send",
  "params": {
    "text": "What is the weather in Tokyo?",
    "channel": "chat-session-123",
    "agent": "agent-weather-bot"
  }
}
```

**Initial Response:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-001",
  "result": {
    "status": "accepted",
    "message": "Agent execution started"
  }
}
```

**Streaming Response (Text Delta):**
```json
{
  "jsonrpc": "2.0",
  "method": "chat.response",
  "params": {
    "text": "Tokyo",
    "done": false
  }
}
```

**Streaming Response (Tool Call Start):**
```json
{
  "jsonrpc": "2.0",
  "method": "chat.response",
  "params": {
    "tool_call_start": {
      "tool_name": "weather_api",
      "call_id": "call-abc123"
    },
    "done": false
  }
}
```

**Streaming Response (Tool Call Result):**
```json
{
  "jsonrpc": "2.0",
  "method": "chat.response",
  "params": {
    "tool_call_result": {
      "call_id": "call-abc123",
      "result": {"temp": 22, "unit": "celsius"},
      "is_error": false
    },
    "done": false
  }
}
```

**Streaming Response (Complete):**
```json
{
  "jsonrpc": "2.0",
  "method": "chat.response",
  "params": {
    "text": "The current temperature in Tokyo is 22°C.",
    "tool_calls": [],
    "usage": {
      "prompt_tokens": 150,
      "completion_tokens": 45,
      "total_tokens": 195
    },
    "done": true
  }
}
```

#### chat.create

Create a new chat session.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `title` | string | No | Session title |
| `agent_id` | string | No | Default agent for the session |

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-002",
  "method": "chat.create",
  "params": {
    "title": "Project Planning",
    "agent_id": "agent-planner"
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-002",
  "result": {
    "session_id": "chat-789",
    "title": "Project Planning",
    "created_at": "2026-02-25T10:30:00Z"
  }
}
```

#### chat.history

Retrieve chat history for a session.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `session_id` | string | Yes | Chat session ID |
| `limit` | integer | No | Maximum messages to return (default: 50) |
| `offset` | integer | No | Pagination offset (default: 0) |

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-003",
  "method": "chat.history",
  "params": {
    "session_id": "chat-789",
    "limit": 20,
    "offset": 0
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-003",
  "result": {
    "session_id": "chat-789",
    "messages": [
      {
        "role": "user",
        "content": "Hello",
        "timestamp": "2026-02-25T10:30:00Z"
      },
      {
        "role": "assistant",
        "content": "Hi there!",
        "timestamp": "2026-02-25T10:30:01Z"
      }
    ],
    "total": 2
  }
}
```

#### chat.delete

Delete a chat session.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `session_id` | string | Yes | Chat session ID to delete |

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-004",
  "method": "chat.delete",
  "params": {
    "session_id": "chat-789"
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-004",
  "result": {
    "status": "deleted",
    "session_id": "chat-789"
  }
}
```

---

### agent.* - Agent Management Methods

Methods for listing, creating, updating, and deleting agents.

#### agent.create

Create a new agent.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Agent name |
| `description` | string | No | Agent description |
| `model` | string | No | Model ID to use (e.g., "gpt-4") |
| `system_prompt` | string | No | System prompt/instructions |
| `tools` | array | No | List of tool names to enable |

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-005",
  "method": "agent.create",
  "params": {
    "name": "Weather Assistant",
    "description": "Helps users check weather information",
    "model": "gpt-4-turbo",
    "system_prompt": "You are a helpful weather assistant. Provide accurate weather information.",
    "tools": ["weather_api", "location_search"]
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-005",
  "result": {
    "agent_id": "agent-weather-bot",
    "name": "Weather Assistant",
    "description": "Helps users check weather information",
    "model": "gpt-4-turbo",
    "system_prompt": "You are a helpful weather assistant...",
    "tools": ["weather_api", "location_search"],
    "created_at": "2026-02-25T10:30:00Z",
    "updated_at": "2026-02-25T10:30:00Z"
  }
}
```

#### agent.update

Update an existing agent.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `agent_id` | string | Yes | Agent ID to update |
| `name` | string | No | New name (optional) |
| `description` | string | No | New description (optional) |
| `model` | string | No | New model ID (optional) |
| `system_prompt` | string | No | New system prompt (optional) |
| `tools` | array | No | New tools list (optional) |

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-006",
  "method": "agent.update",
  "params": {
    "agent_id": "agent-weather-bot",
    "description": "Updated: Weather information assistant with additional features"
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-006",
  "result": {
    "agent_id": "agent-weather-bot",
    "name": "Weather Assistant",
    "description": "Updated: Weather information assistant with additional features",
    "updated_at": "2026-02-25T10:35:00Z"
  }
}
```

#### agent.delete

Delete an agent.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `agent_id` | string | Yes | Agent ID to delete |

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-007",
  "method": "agent.delete",
  "params": {
    "agent_id": "agent-weather-bot"
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-007",
  "result": {
    "status": "deleted",
    "agent_id": "agent-weather-bot"
  }
}
```

#### agent.list

List all agents.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `limit` | integer | No | Maximum agents to return |
| `offset` | integer | No | Pagination offset |

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-008",
  "method": "agent.list",
  "params": {
    "limit": 10,
    "offset": 0
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-008",
  "result": {
    "agents": [
      {
        "agent_id": "agent-weather-bot",
        "name": "Weather Assistant",
        "description": "Helps users check weather information",
        "model": "gpt-4-turbo",
        "tools": ["weather_api", "location_search"],
        "created_at": "2026-02-25T10:30:00Z",
        "updated_at": "2026-02-25T10:35:00Z"
      },
      {
        "agent_id": "agent-planner",
        "name": "Planning Assistant",
        "description": "Helps with project planning",
        "model": "gpt-4",
        "tools": [],
        "created_at": "2026-02-25T09:00:00Z",
        "updated_at": "2026-02-25T09:00:00Z"
      }
    ],
    "total": 2
  }
}
```

#### agent.start

Start an agent instance.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `agent_id` | string | Yes | Agent ID to start |

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-009",
  "method": "agent.start",
  "params": {
    "agent_id": "agent-weather-bot"
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-009",
  "result": {
    "status": "started",
    "agent_id": "agent-weather-bot",
    "instance_id": "instance-abc123"
  }
}
```

#### agent.stop

Stop an agent instance.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `agent_id` | string | Yes | Agent ID to stop |

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-010",
  "method": "agent.stop",
  "params": {
    "agent_id": "agent-weather-bot"
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-010",
  "result": {
    "status": "stopped",
    "agent_id": "agent-weather-bot"
  }
}
```

---

### node.* - Node/Device Methods

Methods for device/node management and interaction.

#### node.pair

Initiate device pairing.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `device_id` | string | Yes | Device ID to pair |
| `device_name` | string | No | Display name for the device |

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-011",
  "method": "node.pair",
  "params": {
    "device_id": "device-123",
    "device_name": "My Laptop"
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-011",
  "result": {
    "status": "pairing initiated",
    "device_id": "device-123",
    "verification_code": "ABC-123-DEF",
    "expires_at": "2026-02-25T11:00:00Z"
  }
}
```

#### node.describe

Get device/node information.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `node_id` | string | Yes | Node ID to describe |

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-012",
  "method": "node.describe",
  "params": {
    "node_id": "node-456"
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-012",
  "result": {
    "node_id": "node-456",
    "name": "Main Server",
    "type": "server",
    "status": "online",
    "capabilities": ["compute", "storage", "network"],
    "last_seen": "2026-02-25T10:30:00Z",
    "metadata": {
      "cpu": "8 cores",
      "memory": "32GB",
      "storage": "1TB SSD"
    }
  }
}
```

#### node.invoke

Invoke a capability on a device.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `node_id` | string | Yes | Node ID |
| `capability` | string | Yes | Capability name (e.g., "compute/run", "storage/read") |
| `params` | object | Yes | Capability-specific parameters |

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-013",
  "method": "node.invoke",
  "params": {
    "node_id": "node-456",
    "capability": "compute/run",
    "params": {
      "command": "ls -la /var/log",
      "timeout": 30
    }
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-013",
  "result": {
    "status": "completed",
    "output": "total 1234\ndrwxr-xr-x 1 root root 4096 Feb 25 10:00 .\n...",
    "exit_code": 0,
    "duration_ms": 150
  }
}
```

---

### canvas.* - Canvas/UI Methods

Methods for interacting with UI canvases (whiteboards, diagrams, etc.).

#### canvas.update

Update a canvas with new content.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `canvas_id` | string | Yes | Canvas ID |
| `elements` | array | Yes | Array of canvas elements |
| `action` | string | Yes | Action type: "add", "update", "delete", "clear" |

**Request (Add Elements):**
```json
{
  "jsonrpc": "2.0",
  "id": "req-014",
  "method": "canvas.update",
  "params": {
    "canvas_id": "canvas-789",
    "action": "add",
    "elements": [
      {
        "id": "elem-001",
        "type": "text",
        "x": 100,
        "y": 50,
        "content": "Project Goals",
        "font_size": 24
      }
    ]
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-014",
  "result": {
    "status": "updated",
    "canvas_id": "canvas-789",
    "elements_updated": 1
  }
}
```

#### canvas.interact

Interact with canvas elements (select, move, resize).

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `canvas_id` | string | Yes | Canvas ID |
| `element_id` | string | Yes | Element ID to interact with |
| `action` | string | Yes | Action: "select", "deselect", "move", "resize", "delete" |
| `position` | object | No | New position {x, y} for move |
| `size` | object | No | New size {width, height} for resize |

**Request (Move Element):**
```json
{
  "jsonrpc": "2.0",
  "id": "req-015",
  "method": "canvas.interact",
  "params": {
    "canvas_id": "canvas-789",
    "element_id": "elem-001",
    "action": "move",
    "position": {
      "x": 150,
      "y": 100
    }
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-015",
  "result": {
    "status": "updated",
    "canvas_id": "canvas-789",
    "element_id": "elem-001",
    "new_position": {"x": 150, "y": 100}
  }
}
```

---

### system.* - System Methods

System-level methods for connection management.

#### system.welcome

Server-initiated welcome message (see [Welcome Message](#welcome-message)).

#### system.ping

Client → Server: Keep-alive ping.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-016",
  "method": "system.ping",
  "params": {}
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-016",
  "result": {
    "status": "pong",
    "timestamp": "2026-02-25T10:30:00Z"
  }
}
```

#### system.version

Client → Server: Get server version information.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-017",
  "method": "system.version",
  "params": {}
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-017",
  "result": {
    "server_version": "0.1.0",
    "protocol_version": "1.0",
    "features": ["chat", "canvas", "node", "approval"]
  }
}
```

---

### approval.* - Approval Methods

Methods for managing approval workflows.

#### approval.request

Request approval for a critical operation.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `agent_id` | string | Yes | Agent ID requesting approval |
| `operation` | string | Yes | Description of the operation |
| `risk_level` | string | Yes | Risk level: "low", "medium", "high", "critical" |

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-018",
  "method": "approval.request",
  "params": {
    "agent_id": "agent-123",
    "operation": "rm -rf /data/old-logs",
    "risk_level": "high"
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-018",
  "result": {
    "status": "created",
    "id": "approval-abc123",
    "message": "Approval request has been created and broadcast to 2 operator(s)"
  }
}
```

#### approval.approve

Approve a pending approval request.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | Yes | Approval request ID |

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-019",
  "method": "approval.approve",
  "params": {
    "id": "approval-abc123"
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-019",
  "result": {
    "status": "approved",
    "id": "approval-abc123",
    "message": "Approval request approval-abc123 has been approved"
  }
}
```

#### approval.deny

Deny a pending approval request.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | Yes | Approval request ID |
| `reason` | string | No | Reason for denial (optional) |

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-020",
  "method": "approval.deny",
  "params": {
    "id": "approval-abc123",
    "reason": "Operation is too risky without backup"
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-020",
  "result": {
    "status": "denied",
    "id": "approval-abc123",
    "reason": "Operation is too risky without backup",
    "message": "Approval request approval-abc123 has been denied"
  }
}
```

#### approval.list

List pending approval requests.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-021",
  "method": "approval.list",
  "params": {}
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-021",
  "result": {
    "approvals": [
      {
        "id": "approval-abc123",
        "agent_id": "agent-123",
        "operation": "rm -rf /data/old-logs",
        "risk_level": "high",
        "requested_at": 1740481800,
        "status": "pending"
      }
    ],
    "count": 1,
    "message": "Found 1 approval request(s)"
  }
}
```

---

### gateway.* - Gateway Methods

Gateway-level methods for subscription management.

#### gateway.subscribe

Update client event subscription filter.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `events` | array | Yes | Array of event types to subscribe to |

Event Types:
- `presence` - Client presence status changes
- `health` - Gateway health snapshots
- `agent` - Agent lifecycle events
- `chat` - Chat-related events
- `approval` - Approval request events

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-022",
  "method": "gateway.subscribe",
  "params": {
    "events": ["presence", "chat", "approval"]
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-022",
  "result": {
    "status": "subscribed",
    "events": ["presence", "chat", "approval"]
  }
}
```

---

## Error Codes

### Standard JSON-RPC 2.0 Error Codes

| Code | Name | Description |
|------|------|-------------|
| -32700 | Parse Error | Invalid JSON was received by the server |
| -32600 | Invalid Request | JSON is not a valid request object |
| -32601 | Method Not Found | The method does not exist / is not available |
| -32602 | Invalid Params | Invalid method parameters |
| -32603 | Internal Error | Internal JSON-RPC error |

### Custom Aisopod Error Codes

| Code | Name | Description |
|------|------|-------------|
| -32001 | Unauthorized | Authentication failed or insufficient permissions |
| -32002 | Rate Limited | Client has exceeded the rate limit |
| -32003 | Device Not Paired | Device must be paired before performing this operation |
| -32004 | Agent Not Found | The specified agent ID does not exist |
| -32005 | Session Not Found | The specified session/channel does not exist |
| -32006 | Permission Denied | Operation not allowed for the current user/role |
| -32007 | Conflict | Request conflicts with current state (e.g., double pairing) |
| -32008 | Timeout | Operation timed out |
| -32009 | Resource Exhausted | Server is out of resources (memory, connections, etc.) |
| -32010 | Unsupported | Feature or capability not supported by this server version |

### Error Response Examples

**Parse Error (-32700):**
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32700,
    "message": "Failed to parse JSON: expected value at line 1 column 1",
    "data": null
  },
  "id": null
}
```

**Invalid Request (-32600):**
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32600,
    "message": "Missing or empty 'method' field",
    "data": null
  },
  "id": null
}
```

**Method Not Found (-32601):**
```json
{
  "jsonrpc": "2.0",
  "id": "req-001",
  "error": {
    "code": -32601,
    "message": "Method unknown.method not found",
    "data": null
  }
}
```

**Unauthorized (-32001):**
```json
{
  "jsonrpc": "2.0",
  "id": "req-001",
  "error": {
    "code": -32001,
    "message": "Authentication failed: invalid token",
    "data": null
  }
}
```

**Rate Limited (-32002):**
```json
{
  "jsonrpc": "2.0",
  "id": "req-001",
  "error": {
    "code": -32002,
    "message": "Rate limit exceeded: maximum 100 requests per minute",
    "data": {
      "retry_after": 45
    }
  }
}
```

**Device Not Paired (-32003):**
```json
{
  "jsonrpc": "2.0",
  "id": "req-001",
  "error": {
    "code": -32003,
    "message": "Device not paired. Please complete device pairing first.",
    "data": {
      "pairing_url": "https://app.aisopod.io/pair?code=ABC123"
    }
  }
}
```

---

## Security Considerations

### Transport Security

1. **TLS Required**: Production deployments must use `wss://` (WebSocket Secure) with valid TLS certificates
2. **Certificate Validation**: Clients must validate server certificates against trusted CAs
3. **Cipher Suites**: Use only strong cipher suites (AES-GCM, ChaCha20-Poly1305)

### Authentication

1. **Token Security**:
   - Bearer tokens must be sent in the `Authorization: Bearer <token>` header
   - Device tokens must be sent in the `Authorization: DeviceToken <device_token>` header
   - Tokens must have appropriate expiration times
   - Tokens should be rotated periodically

2. **Token Storage**:
   - Never store tokens in local storage (web clients)
   - Use secure storage mechanisms (Keychain, Keystore, encrypted preferences)
   - Implement secure token refresh before expiration

3. **Session Management**:
   - Each WebSocket connection should have a unique session ID
   - Server should track active sessions and enforce session limits
   - Implement session timeout for inactive connections

### Authorization

1. **Scope-Based Access Control**:
   - Methods should require specific scopes (e.g., `operator.read`, `operator.write`)
   - Check scopes before executing any method
   - Return `-32001 Unauthorized` for unauthorized access

2. **Principle of Least Privilege**:
   - Default to minimal permissions
   - Grant additional permissions only when explicitly requested
   - Audit all privilege escalations

### Rate Limiting

1. **Connection Rate Limits**:
   - Limit new connections per IP address
   - Implement connection cooldown after disconnections

2. **Request Rate Limits**:
   - Limit requests per connection per minute (e.g., 100 requests/minute)
   - Return `-32002 Rate Limited` with `retry_after` in error data

3. **Broadcast Rate Limits**:
   - Limit broadcast event frequency
   - Implement backpressure for slow clients

### Input Validation

1. **All Input Must Be Validated**:
   - Validate JSON-RPC message structure
   - Validate method names against whitelist
   - Validate parameter types and ranges
   - Sanitize user-provided strings (no XSS)

2. **Error Handling**:
   - Never leak internal error details to clients
   - Log detailed errors server-side only
   - Return generic error messages to clients

### Audit Logging

1. **Log Security Events**:
   - Authentication attempts (success/failure)
   - Authorization decisions
   - Rate limit violations
   - Malformed requests

2. **Log Format**:
   ```json
   {
     "timestamp": "2026-02-25T10:30:00Z",
     "event": "auth_failure",
     "client_ip": "192.168.1.100",
     "details": {
       "method": "agent.list",
       "token_hash": "sha256:abc123...",
       "reason": "token_expired"
     }
   }
   ```

### Device Security

1. **Device Pairing**:
   - Verify device identity during pairing
   - Use time-limited pairing codes
   - Support remote device wipe

2. **Device Tokens**:
   - Store device tokens securely
   - Implement device token revocation
   - Support multiple concurrent device tokens per user

---

## Example Sessions

### Authentication Flow

**Step 1: WebSocket Upgrade Request**
```
GET /ws HTTP/1.1
Host: api.aisopod.io
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==
Sec-WebSocket-Version: 13
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJ1c2VyX2lkIjoidXNlci0xMjMifQ...
X-Aisopod-Client: aisopod-ios/2.1.0
X-Aisopod-Device-Id: 550e8400-e29b-41d4-a716-446655440000
```

**Step 2: Server Welcome Message**
```json
{
  "jsonrpc": "2.0",
  "method": "system.welcome",
  "params": {
    "server_version": "0.1.0",
    "protocol_version": "1.0",
    "session_id": "session-abc123",
    "capabilities": ["chat", "canvas", "node", "approval"]
  }
}
```

**Step 3: Client Subscribes to Events**
```json
{
  "jsonrpc": "2.0",
  "id": "req-001",
  "method": "gateway.subscribe",
  "params": {
    "events": ["presence", "chat", "agent"]
  }
}
```

**Step 4: Server Acknowledges Subscription**
```json
{
  "jsonrpc": "2.0",
  "id": "req-001",
  "result": {
    "status": "subscribed",
    "events": ["presence", "chat", "agent"]
  }
}
```

### Chat Interaction

**Step 1: Send Message**
```json
{
  "jsonrpc": "2.0",
  "id": "req-002",
  "method": "chat.send",
  "params": {
    "text": "What are the project requirements?",
    "channel": "project-discussion",
    "agent": "agent-planner"
  }
}
```

**Step 2: Immediate Acknowledgment**
```json
{
  "jsonrpc": "2.0",
  "id": "req-002",
  "result": {
    "status": "accepted",
    "message": "Agent execution started"
  }
}
```

**Step 3: Streaming Response (Partial)**
```json
{
  "jsonrpc": "2.0",
  "method": "chat.response",
  "params": {
    "text": "Based on our previous conversation, the project requirements are:",
    "done": false
  }
}
```

**Step 4: Streaming Response (Complete)**
```json
{
  "jsonrpc": "2.0",
  "method": "chat.response",
  "params": {
    "text": "1. User authentication system\n2. Agent management dashboard\n3. Real-time chat\n4. Device node management\n\nTotal tokens used: 45",
    "tool_calls": [],
    "usage": {
      "prompt_tokens": 150,
      "completion_tokens": 45,
      "total_tokens": 195
    },
    "done": true
  }
}
```

### Agent Management

**Step 1: Create Agent**
```json
{
  "jsonrpc": "2.0",
  "id": "req-003",
  "method": "agent.create",
  "params": {
    "name": "Customer Support Bot",
    "description": "Automated customer support agent",
    "model": "gpt-4",
    "system_prompt": "You are a helpful customer support agent. Be polite and professional.",
    "tools": ["knowledge_base_search", "ticket_creation"]
  }
}
```

**Step 2: Server Response**
```json
{
  "jsonrpc": "2.0",
  "id": "req-003",
  "result": {
    "agent_id": "agent-support-bot",
    "name": "Customer Support Bot",
    "description": "Automated customer support agent",
    "model": "gpt-4",
    "system_prompt": "You are a helpful customer support agent...",
    "tools": ["knowledge_base_search", "ticket_creation"],
    "created_at": "2026-02-25T11:00:00Z",
    "updated_at": "2026-02-25T11:00:00Z"
  }
}
```

**Step 3: List Agents**
```json
{
  "jsonrpc": "2.0",
  "id": "req-004",
  "method": "agent.list",
  "params": {
    "limit": 10,
    "offset": 0
  }
}
```

**Step 4: Server Lists Agents**
```json
{
  "jsonrpc": "2.0",
  "id": "req-004",
  "result": {
    "agents": [
      {
        "agent_id": "agent-support-bot",
        "name": "Customer Support Bot",
        "description": "Automated customer support agent",
        "model": "gpt-4",
        "tools": ["knowledge_base_search", "ticket_creation"],
        "created_at": "2026-02-25T11:00:00Z",
        "updated_at": "2026-02-25T11:00:00Z"
      },
      {
        "agent_id": "agent-weather-bot",
        "name": "Weather Assistant",
        "description": "Weather information assistant",
        "model": "gpt-4-turbo",
        "tools": ["weather_api"],
        "created_at": "2026-02-25T10:00:00Z",
        "updated_at": "2026-02-25T10:00:00Z"
      }
    ],
    "total": 2
  }
}
```

**Step 5: Delete Agent**
```json
{
  "jsonrpc": "2.0",
  "id": "req-005",
  "method": "agent.delete",
  "params": {
    "agent_id": "agent-support-bot"
  }
}
```

**Step 6: Confirmation**
```json
{
  "jsonrpc": "2.0",
  "id": "req-005",
  "result": {
    "status": "deleted",
    "agent_id": "agent-support-bot"
  }
}
```

---

## Revision History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-02-25 | Initial release |

---

## Related Documents

- [Issue 029: JSON-RPC Message Parsing](../issues/resolved/029-json-rpc-message-parsing.md)
- [Issue 030: RPC Method Router](../issues/resolved/030-rpc-method-router-handler-trait.md)

---

## Feedback

For questions or suggestions about this protocol specification, please file an issue in the main repository.
