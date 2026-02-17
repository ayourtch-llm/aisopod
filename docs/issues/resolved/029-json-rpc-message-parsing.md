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
- [x] `RpcRequest`, `RpcResponse`, and `RpcError` types are defined and public.
- [x] Valid JSON-RPC 2.0 messages parse into `RpcRequest` correctly.
- [x] Invalid JSON returns error code `-32700`.
- [x] Missing or wrong `jsonrpc` field returns error code `-32600`.
- [x] Unknown method detection is available for use by the router (error `-32601`).
- [x] Unit tests cover all happy and error paths.

## Resolution
The JSON-RPC 2.0 message parsing layer was implemented in `crates/aisopod-gateway/src/rpc/types.rs`:

### Types Implemented
- **`RpcRequest`**: Deserialize incoming JSON-RPC requests with validation for required fields (`jsonrpc`, `method`) and optional fields (`params`, `id`)
- **`RpcResponse`**: Serialize responses with `result` or `error` fields, always including `jsonrpc: "2.0"` and `id`
- **`RpcError`**: Error structure with `code`, `message`, and optional `data` for additional context

### Error Codes
Standard JSON-RPC 2.0 error codes defined in `error_codes` module:
- `-32700` (PARSE_ERROR) - Invalid JSON
- `-32600` (INVALID_REQUEST) - Missing required fields or wrong version
- `-32601` (METHOD_NOT_FOUND) - Unknown method

### Parse Function
```rust
pub fn parse(raw: &str) -> Result<RpcRequest, RpcResponse>
```
- Attempts `serde_json::from_str` - returns `-32700` error on failure
- Validates `jsonrpc == "2.0"` - returns `-32600` error on failure
- Validates `method` is present and non-empty - returns `-32600` error on failure
- Returns `Ok(RpcRequest)` on success

### Helper Constructors
- `RpcResponse::success(id, result)` - Create successful response
- `RpcResponse::error(id, code, message)` - Create error response
- `RpcResponse::error_with_data(id, code, message, data)` - Create error with custom data

### Method Router
Implemented in `crates/aisopod-gateway/src/rpc/handler.rs`:
- `RpcMethod` trait for handler implementations
- `MethodRouter` for dispatching requests to appropriate handlers
- `PlaceholderHandler` returning `-32601` for unimplemented methods
- `default_router()` with placeholder handlers for 24 method namespaces

### Unit Tests
Comprehensive test coverage in `types.rs` and `handler.rs`:
- Valid request parsing
- Invalid JSON error (`-32700`)
- Wrong JSON-RPC version error (`-32600`)
- Missing/empty method field error (`-32600`)
- Empty method name validation
- Response serialization (success and error)
- Notification requests (no id)
- Method router dispatch (known and unknown methods)
- Default router with placeholder handlers

### Verification
- `cargo build` passes without errors
- `cargo test` passes without failures
- No compilation warnings (`RUSTFLAGS=-Awarnings`)

---
*Created: 2026-02-15*
*Resolved: 2026-02-17*
