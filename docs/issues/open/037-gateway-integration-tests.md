# Issue 037: Add Gateway Integration Tests

## Summary
Create a comprehensive integration test suite for the `aisopod-gateway` crate that exercises all major subsystems end-to-end: HTTP endpoints, WebSocket connections, authentication, rate limiting, JSON-RPC message flow, and event broadcasting.

## Location
- Crate: `aisopod-gateway`
- File: `crates/aisopod-gateway/tests/integration.rs`

## Current Behavior
Individual components may have unit tests, but there are no integration tests that start the full gateway server and interact with it as a real client would.

## Expected Behavior
A test binary starts the gateway on a random available port and runs a suite of tests covering every major feature. Tests are isolated and can run in parallel where possible.

## Impact
Integration tests are the primary safety net against regressions. They validate that all components work together correctly and that the gateway behaves as documented from a client's perspective.

## Suggested Implementation
1. Create `crates/aisopod-gateway/tests/integration.rs`.
2. Write a test helper that:
   - Builds a `GatewayConfig` with a random available port (bind to `127.0.0.1:0`).
   - Starts the gateway in a background `tokio::spawn` task.
   - Returns the bound address for use by test clients.
   - Provides a cleanup handle that shuts the server down after tests complete.
3. Write the following test functions (each as a `#[tokio::test]`):

   **HTTP Endpoint Tests:**
   - `test_health_returns_200` — `GET /health` returns 200 with `{"status":"ok"}`.
   - `test_stub_endpoints_return_501` — Each stub endpoint returns 501.
   - `test_static_file_fallback` — Unknown path returns `index.html`.

   **Authentication Tests:**
   - `test_valid_token_accepted` — Request with correct Bearer token succeeds.
   - `test_invalid_token_rejected` — Request with wrong token gets 401.
   - `test_no_auth_mode` — All requests succeed when auth mode is `none`.

   **Rate Limiting Tests:**
   - `test_under_limit_allowed` — Requests within the limit succeed.
   - `test_over_limit_returns_429` — Exceeding the limit returns 429 with `Retry-After`.

   **WebSocket Tests:**
   - `test_ws_connect_and_ping` — Client connects, receives pong to a ping.
   - `test_ws_auth_rejected` — Unauthenticated WebSocket upgrade fails.

   **JSON-RPC Tests:**
   - `test_valid_rpc_request` — Send a valid RPC request over WebSocket, receive a response.
   - `test_malformed_json_returns_parse_error` — Send invalid JSON, receive `-32700`.
   - `test_unknown_method_returns_not_found` — Send unknown method, receive `-32601`.

   **Broadcast Tests:**
   - `test_broadcast_event_received` — Two clients connect; one triggers an event; both receive the broadcast.

4. Use `reqwest` for HTTP tests and `tokio-tungstenite` for WebSocket tests.
5. Add `reqwest` and `tokio-tungstenite` as dev-dependencies.

## Dependencies
- Issue 026 (Axum HTTP server skeleton)
- Issue 027 (REST API endpoint stubs)
- Issue 028 (WebSocket upgrade and connection lifecycle)
- Issue 029 (JSON-RPC message parsing)
- Issue 030 (RPC method router and handler trait)
- Issue 031 (Gateway authentication system)
- Issue 032 (Rate limiting)
- Issue 033 (Client connection management)
- Issue 034 (Event broadcasting system)
- Issue 035 (Static file serving for web UI)
- Issue 036 (TLS/HTTPS support)

## Acceptance Criteria
- [ ] Integration test binary compiles and runs with `cargo test -p aisopod-gateway`.
- [ ] All HTTP endpoint tests pass.
- [ ] All authentication tests pass.
- [ ] All rate limiting tests pass.
- [ ] All WebSocket connection tests pass.
- [ ] All JSON-RPC message flow tests pass.
- [ ] All broadcast event tests pass.
- [ ] Tests are isolated and do not interfere with each other.

---
*Created: 2026-02-15*
