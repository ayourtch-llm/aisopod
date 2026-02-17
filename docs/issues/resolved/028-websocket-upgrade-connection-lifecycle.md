# Issue 028: Implement WebSocket Upgrade and Connection Lifecycle

## Summary
Add WebSocket support to the gateway so that clients can establish persistent, bidirectional connections for real-time RPC communication. Each connection receives a unique `connId`, maintains a heartbeat, and is cleaned up on disconnect.

## Location
- Crate: `aisopod-gateway`
- File: `crates/aisopod-gateway/src/ws.rs`

## Current Behavior
The gateway only serves HTTP request/response endpoints. There is no WebSocket upgrade path or persistent connection handling.

## Expected Behavior
- A client can upgrade to WebSocket at the `/ws` path.
- The handshake enforces a configurable timeout (e.g., 5 seconds).
- Once connected, the server sends periodic ping frames and expects pong responses (keep-alive).
- Each connection is assigned a unique `connId` (UUID or monotonic ID).
- When a client disconnects (cleanly or due to timeout), resources are released and the event is logged.

## Impact
WebSocket connections are the primary transport for JSON-RPC messages between operators, nodes, and the web UI. Without this, no real-time communication is possible.

## Suggested Implementation
1. Add `tokio-tungstenite` (or use Axum's built-in WebSocket support via the `ws` feature) to `Cargo.toml`.
2. Create `crates/aisopod-gateway/src/ws.rs`:
   - Define an Axum handler that performs the WebSocket upgrade:
     ```rust
     async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
         ws.on_upgrade(handle_connection)
     }
     ```
   - In `handle_connection`, generate a `connId`, log it, then enter a loop:
     - Read incoming messages.
     - On `Ping`, respond with `Pong`.
     - On `Close` or error, break out and clean up.
   - Spawn a background task that sends a `Ping` every N seconds (configurable).
   - Use `tokio::time::timeout` around the initial handshake to enforce the handshake timeout.
3. Register the route in `server.rs`: `.route("/ws", get(ws_handler))`.
4. On disconnect, log the `connId` and duration at `info` level.

## Dependencies
- Issue 026 (Axum HTTP server skeleton)

## Acceptance Criteria
- [ ] `GET /ws` upgrades the connection to WebSocket.
- [ ] Each connection is assigned a unique `connId` visible in logs.
- [ ] Server sends periodic ping frames; client pongs keep the connection alive.
- [ ] Connections that fail the handshake timeout are dropped with an appropriate log.
- [ ] Disconnected connections are cleaned up (no resource leaks).

## Resolution

The WebSocket upgrade and connection lifecycle was implemented with the following changes to `crates/aisopod-gateway/src/ws.rs`:

1. **WebSocket Upgrade**: Implemented `ws_handler` using Axum's `WebSocketUpgrade` extractor that performs the upgrade at `/ws` path with configurable handshake timeout.

2. **Connection ID**: Each connection receives a unique `connId` using `Uuid::new_v4()`, logged when connections establish and close.

3. **Heartbeat Mechanism**: 
   - Server sends periodic ping frames every 30 seconds
   - Client pongs are tracked to keep the connection alive
   - Connection times out if pong not received within 10 seconds after ping
   - Server also sends pong responses when receiving pings from clients

4. **Handshake Timeout**: Uses `tokio::time::timeout` to enforce configurable timeout (default 5 seconds) on the initial WebSocket handshake.

5. **Connection Cleanup**: When disconnecting (clean close or error), resources are properly released and connection duration is logged.

6. **Code Structure**: Simplified the implementation from using multiple tasks with broadcast channels to a single loop using `tokio::select!` for clean message handling.

---

*Created: 2026-02-15*
*Resolved: 2026-02-17*
