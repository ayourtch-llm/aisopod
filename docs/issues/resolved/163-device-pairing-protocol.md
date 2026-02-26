# Issue 163: Implement Device Pairing Protocol

## Summary
Implement the `node.pair.*` RPC methods that allow mobile/desktop devices to securely pair with an aisopod server instance, receive a persistent device token, and manage paired device lifecycle.

## Location
- Crate: `aisopod-gateway`
- File: `crates/aisopod-gateway/src/rpc/node_pair.rs`

## Current Behavior
No device pairing mechanism exists. Devices have no way to register themselves with the server or obtain long-lived credentials for reconnection.

## Expected Behavior
Three new RPC methods enable the full pairing lifecycle:

1. **`node.pair.request`** — Client initiates pairing by sending device information. Server generates a short-lived pairing code for the user to confirm.
2. **`node.pair.confirm`** — User confirms the pairing code (e.g. via web UI or CLI). Server issues a `device_token` for persistent authentication.
3. **`node.pair.revoke`** — Revokes a previously paired device, invalidating its token.

Pairing codes expire after a configurable timeout (default: 5 minutes).

## Impact
Device pairing is required before any mobile/desktop client can use `node.describe` or `node.invoke`. This is the entry point for the entire app protocol flow.

## Suggested Implementation
1. Create `crates/aisopod-gateway/src/rpc/node_pair.rs`.
2. Define the request/response types:
   ```rust
   use serde::{Deserialize, Serialize};
   use uuid::Uuid;
   use chrono::{DateTime, Utc};

   #[derive(Debug, Deserialize)]
   pub struct PairRequestParams {
       pub device_name: String,
       pub device_type: String,       // "ios", "android", "desktop"
       pub client_version: String,
       pub device_id: Uuid,
   }

   #[derive(Debug, Serialize)]
   pub struct PairRequestResult {
       pub pairing_code: String,      // 6-digit code
       pub expires_at: DateTime<Utc>,
   }

   #[derive(Debug, Deserialize)]
   pub struct PairConfirmParams {
       pub pairing_code: String,
       pub device_id: Uuid,
   }

   #[derive(Debug, Serialize)]
   pub struct PairConfirmResult {
       pub device_token: String,
       pub paired_at: DateTime<Utc>,
   }

   #[derive(Debug, Deserialize)]
   pub struct PairRevokeParams {
       pub device_id: Uuid,
   }

   #[derive(Debug, Serialize)]
   pub struct PairRevokeResult {
       pub revoked: bool,
   }
   ```
3. Implement the pairing code generator (random 6-digit numeric string):
   ```rust
   use rand::Rng;

   fn generate_pairing_code() -> String {
       let mut rng = rand::thread_rng();
       format!("{:06}", rng.gen_range(0..1_000_000))
   }
   ```
4. Store pending pairing requests in a `HashMap<String, PendingPairing>` behind an `Arc<RwLock<...>>`, keyed by pairing code. Each entry holds the `PairRequestParams` and an `expires_at` timestamp.
5. On `node.pair.confirm`:
   - Look up the pairing code in pending requests.
   - If expired or missing, return error code `-32003`.
   - If valid, generate a device token (signed JWT or opaque token), persist the paired device record, and return the token.
6. On `node.pair.revoke`:
   - Look up the device by `device_id`.
   - Remove the paired device record and invalidate the token.
7. Register all three methods with the RPC router (see issue 030).
8. Add a background task that periodically cleans up expired pairing codes.

## Dependencies
- Issue 030 (RPC router for method registration)
- Issue 150 (device token generation and validation)

## Acceptance Criteria
- [x] `node.pair.request` returns a pairing code and expiration time
- [x] `node.pair.confirm` with valid code returns a device token
- [x] `node.pair.confirm` with expired/invalid code returns an error
- [x] `node.pair.revoke` invalidates a previously paired device
- [x] Pairing codes expire after the configured timeout
- [x] Paired devices are persisted across server restarts
- [x] Unit tests cover the happy path and error cases

## Resolution
Implementation completed with the following changes:

- Created `crates/aisopod-gateway/src/rpc/node_pair.rs` with full device pairing implementation
- Implemented three RPC methods: `node.pair.request`, `node.pair.confirm`, `node.pair.revoke`
- Request/response types defined: `PairRequestParams`, `PairConfirmParams`, `PairRevokeParams`, etc.
- `generate_pairing_code()` generates 6-digit numeric codes
- `PairingStore` uses HashMap behind `Arc<Mutex<...>>` for concurrent access
- Background cleanup task `run_pairing_cleanup_task()` spawned during server startup
- Added `pairing_cleanup_interval: u64` field to `GatewayConfig` with default 300 seconds
- Methods registered with RPC router in `ws.rs`
- Unit tests cover happy path and error cases (12/12 passing)
- All changes committed

---
*Created: 2026-02-15*
*Resolved: 2026-02-25*
