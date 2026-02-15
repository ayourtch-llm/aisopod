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
- [ ] All five endpoints are reachable at their documented paths.
- [ ] Each returns HTTP 501 with a JSON error body.
- [ ] Every request produces a `tracing` log line containing method, path, and response status.
- [ ] Existing `/health` endpoint is unaffected.

---
*Created: 2026-02-15*
