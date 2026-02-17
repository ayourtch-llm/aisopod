# Issue 032: Implement Rate Limiting

## Summary
Add per-IP sliding-window rate limiting to the gateway so that abusive or misconfigured clients cannot overwhelm the server. Limits are configurable and enforced as Axum middleware.

## Location
- Crate: `aisopod-gateway`
- File: `crates/aisopod-gateway/src/middleware/rate_limit.rs`

## Current Behavior
The gateway accepts an unlimited number of requests from any IP address. There is no throttling mechanism.

## Expected Behavior
- Each client IP is tracked in a sliding time window.
- When the request count exceeds the configured limit within the window, the server returns HTTP 429 Too Many Requests with a `Retry-After` header indicating how many seconds until the window resets.
- Rate-limit state is stored in memory using a concurrent map (`DashMap`).
- A background task periodically evicts expired entries to prevent unbounded memory growth.
- Limits are read from `GatewayConfig`.

## Impact
Rate limiting is a basic operational safeguard that prevents denial-of-service conditions. It must be in place before the gateway is exposed to any untrusted network.

## Suggested Implementation
1. Add `dashmap` to `aisopod-gateway/Cargo.toml`.
2. Create `crates/aisopod-gateway/src/middleware/rate_limit.rs`:
   - Define a `RateLimiter` struct:
     ```rust
     pub struct RateLimiter {
         state: DashMap<IpAddr, Vec<Instant>>,
         max_requests: u64,
         window: Duration,
     }
     ```
   - Implement `pub fn check(&self, ip: IpAddr) -> Result<(), Duration>`:
     - Look up or insert the IP entry.
     - Remove timestamps older than `window`.
     - If the remaining count ≥ `max_requests`, return `Err(remaining_duration)`.
     - Otherwise, push the current timestamp and return `Ok(())`.
   - Implement a `pub async fn cleanup_loop(&self)` method that runs every `window` and removes stale entries.
3. Create an Axum middleware function:
   ```rust
   async fn rate_limit_middleware(
       State(limiter): State<Arc<RateLimiter>>,
       ConnectInfo(addr): ConnectInfo<SocketAddr>,
       request: Request,
       next: Next,
   ) -> Response { ... }
   ```
   - On `Err(retry_after)`, return 429 with `Retry-After` header.
4. Add the middleware to the router in `server.rs`, after authentication.
5. Spawn the cleanup task in the server startup function.
6. Write unit tests for under-limit, at-limit, and over-limit scenarios.

## Dependencies
- Issue 026 (Axum HTTP server skeleton)
- Issue 031 (Gateway authentication — rate limiting runs after auth)

## Acceptance Criteria
- [ ] Requests within the configured limit succeed normally.
- [ ] Requests exceeding the limit receive HTTP 429 with a `Retry-After` header.
- [ ] Rate-limit state is per-IP.
- [ ] Expired entries are cleaned up periodically (no memory leak).
- [ ] Limits are configurable via `GatewayConfig`.
- [ ] Unit tests cover under-limit, at-limit, and over-limit cases.

---
*Created: 2026-02-15*
