# Issue 029: Implement JSON-RPC 2.0 Message Parsing

## Summary
Implement strongly-typed parsing and serialization for JSON-RPC 2.0 messages flowing over WebSocket connections. Malformed messages must be rejected with the correct JSON-RPC error codes.

## Location
- Crate: `aisopod-gateway`
- File: `crates/aisopod-gateway/src/rpc/types.rs`

## Current Behavior
WebSocket messages are received as raw text or binary frames. There is no structured parsing or validation against the JSON-RPC 2.0 specification.

## Expected Behavior
- Incoming WebSocket text messages are deserialized into `RpcRequest` structs.
- Outgoing responses are serialized as `RpcResponse` structs.
- Errors are represented by an `RpcError` struct containing `code`, `message`, and optional `data`.
- Standard error codes are handled:
  - `-32700` — Parse error (invalid JSON)
  - `-32600` — Invalid request (missing required fields)
  - `-32601` — Method not found
- All types derive `serde::Serialize` and `serde::Deserialize`.

## Impact
Every RPC interaction between the gateway and its clients passes through this parsing layer. Correct, spec-compliant parsing is essential for interoperability with any JSON-RPC 2.0 client library.

## Suggested Implementation
1. Create `crates/aisopod-gateway/src/rpc/types.rs` with:
   ```rust
   #[derive(Deserialize)]
   pub struct RpcRequest {
       pub jsonrpc: String,      // must be "2.0"
       pub method: String,
       #[serde(default)]
       pub params: Option<serde_json::Value>,
       pub id: Option<serde_json::Value>,
   }

   #[derive(Serialize)]
   pub struct RpcResponse {
       pub jsonrpc: &'static str, // always "2.0"
       #[serde(skip_serializing_if = "Option::is_none")]
       pub result: Option<serde_json::Value>,
       #[serde(skip_serializing_if = "Option::is_none")]
       pub error: Option<RpcError>,
       pub id: Option<serde_json::Value>,
   }

   #[derive(Serialize)]
   pub struct RpcError {
       pub code: i32,
       pub message: String,
       #[serde(skip_serializing_if = "Option::is_none")]
       pub data: Option<serde_json::Value>,
   }
   ```
2. Add a `pub fn parse(raw: &str) -> Result<RpcRequest, RpcResponse>` function that:
   - Attempts `serde_json::from_str`. On failure, returns a `-32700` error response.
   - Validates that `jsonrpc == "2.0"`. On failure, returns a `-32600` error response.
3. Add helper constructors on `RpcResponse`:
   - `RpcResponse::success(id, result)`
   - `RpcResponse::error(id, code, message)`
4. Create `crates/aisopod-gateway/src/rpc/mod.rs` to re-export types.
5. Write unit tests covering valid requests, missing fields, invalid JSON, and wrong version strings.

## Dependencies
- Issue 028 (WebSocket upgrade and connection lifecycle)

## Acceptance Criteria
- [ ] `RpcRequest`, `RpcResponse`, and `RpcError` types are defined and public.
- [ ] Valid JSON-RPC 2.0 messages parse into `RpcRequest` correctly.
- [ ] Invalid JSON returns error code `-32700`.
- [ ] Missing or wrong `jsonrpc` field returns error code `-32600`.
- [ ] Unknown method detection is available for use by the router (error `-32601`).
- [ ] Unit tests cover all happy and error paths.

---
*Created: 2026-02-15*
