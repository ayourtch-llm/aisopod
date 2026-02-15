# Issue 031: Implement Gateway Authentication System

## Summary
Add authentication to the gateway supporting three modes — Bearer token, password, and no-auth — for both HTTP routes and WebSocket handshakes. Authenticated requests carry a role (operator or node) and a set of permission scopes.

## Location
- Crate: `aisopod-gateway`
- Files:
  - `crates/aisopod-gateway/src/auth.rs`
  - `crates/aisopod-gateway/src/middleware/auth.rs`

## Current Behavior
All HTTP and WebSocket endpoints are open with no authentication or authorization checks.

## Expected Behavior
- An Axum middleware layer inspects incoming HTTP requests for an `Authorization` header.
- Three auth modes are supported, selected via `GatewayConfig`:
  - **token** — Expects `Authorization: Bearer <token>`. Validates against configured tokens.
  - **password** — Expects HTTP Basic auth. Validates against configured credentials.
  - **none** — All requests are allowed without credentials.
- WebSocket upgrade requests are authenticated during the HTTP handshake phase before the upgrade completes.
- On successful auth, the request is annotated with a role (`operator` or `node`) and a list of scopes (e.g., `chat:write`, `agent:admin`).
- Unauthorized requests receive HTTP 401 with a JSON error body.

## Impact
Authentication is a security-critical gate. Without it, any client can invoke any RPC method or REST endpoint. This issue also provides the role and scope information that downstream authorization checks rely on.

## Suggested Implementation
1. Create `crates/aisopod-gateway/src/auth.rs`:
   - Define an `AuthInfo` struct: `{ role: String, scopes: Vec<String> }`.
   - Implement `validate_token(token: &str, config: &AuthConfig) -> Option<AuthInfo>`.
   - Implement `validate_basic(user: &str, pass: &str, config: &AuthConfig) -> Option<AuthInfo>`.
2. Create `crates/aisopod-gateway/src/middleware/auth.rs`:
   - Implement an Axum middleware (using `axum::middleware::from_fn`) that:
     1. Reads the `Authorization` header.
     2. Calls the appropriate validate function based on the configured auth mode.
     3. On success, inserts `AuthInfo` into request extensions.
     4. On failure, returns `(StatusCode::UNAUTHORIZED, Json(...))`.
3. Apply the middleware to all routes except `GET /health`.
4. In the WebSocket upgrade handler, extract `AuthInfo` from request extensions before upgrading.
5. Write unit tests for each auth mode and for the rejection path.

## Dependencies
- Issue 026 (Axum HTTP server skeleton)
- Issue 028 (WebSocket upgrade and connection lifecycle)
- Issue 016 (Configuration loading — provides `AuthConfig`)

## Acceptance Criteria
- [ ] Bearer token auth validates correct tokens and rejects incorrect ones.
- [ ] Password auth validates correct credentials and rejects incorrect ones.
- [ ] No-auth mode allows all requests through.
- [ ] WebSocket handshakes are authenticated before upgrade.
- [ ] Authenticated requests carry `role` and `scopes` in request extensions.
- [ ] Unauthorized requests receive HTTP 401 with a JSON error body.
- [ ] `GET /health` remains accessible without authentication.

---
*Created: 2026-02-15*
