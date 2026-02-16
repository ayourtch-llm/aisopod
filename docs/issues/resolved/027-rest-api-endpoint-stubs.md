# Issue 027: Add REST API Endpoint Stubs

## Summary
Register route stubs for all planned REST API endpoints so that the gateway has a complete URL surface area from the start. Each stub returns HTTP 501 Not Implemented with a descriptive JSON body, and every request is logged via `tracing`.

## Location
- Crate: `aisopod-gateway`
- File: `crates/aisopod-gateway/src/routes.rs`

## Current Behavior
Only the `GET /health` endpoint exists. No other REST routes are defined.

## Expected Behavior
The following routes are registered and reachable:
- `POST /v1/chat/completions`
- `POST /v1/responses`
- `POST /hooks`
- `GET  /tools/invoke`
- `GET  /status`

Each returns HTTP 501 with a JSON body such as `{"error":"not implemented"}`. Every incoming request is logged at the `info` level with method, path, and client IP.

## Impact
Having the full route table in place early lets front-end developers, tests, and other crates discover the API surface. It also establishes the request-logging pattern used by all future handlers.

## Suggested Implementation
1. Create `crates/aisopod-gateway/src/routes.rs`.
2. Define one handler function per stub:
   ```rust
   async fn not_implemented() -> impl IntoResponse {
       (StatusCode::NOT_IMPLEMENTED, Json(serde_json::json!({"error":"not implemented"})))
   }
   ```
3. Build the router in a `pub fn api_routes() -> Router` function, chaining `.route(path, method(handler))` for each endpoint.
4. Add a `tracing` layer using `tower_http::trace::TraceLayer` (or a manual middleware) that logs each request with method, URI, and status code.
5. Merge `api_routes()` into the main router in `server.rs`.
6. Verify with `curl -X POST http://localhost:<port>/v1/chat/completions` — expect 501.

## Dependencies
- Issue 026 (Axum HTTP server skeleton)

## Acceptance Criteria
- [x] All five endpoints are reachable at their documented paths.
- [x] Each returns HTTP 501 with a JSON error body.
- [x] Every request produces a `tracing` log line containing method, path, and response status.
- [x] Existing `/health` endpoint is unaffected.

## Resolution
Fixed visibility issues in the implementation:
1. Made `ws_handler` function public in `crates/aisopod-gateway/src/ws.rs` so it can be accessed from `routes.rs`
2. Removed the private `into_inner()` call in `api_routes()` function in `crates/aisopod-gateway/src/routes.rs`
3. The `not_implemented` handler function already properly logs requests with method, path, and client IP using `tracing::info!`
4. The `api_routes()` function properly returns `Router` directly without needing to unwrap
5. All five endpoints are registered: `/v1/chat/completions`, `/v1/responses`, `/hooks`, `/tools/invoke`, `/status`
6. The `TraceLayer` is applied in `server.rs` to log all requests
7. `cargo build` and `cargo test` pass successfully

---
*Created: 2026-02-15*
*Resolved: 2026-02-16*
