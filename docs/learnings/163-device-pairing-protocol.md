# Learning: Device Pairing Protocol Implementation

## Summary

Issue #163 implemented a device pairing protocol with three RPC methods (`node.pair.request`, `node.pair.confirm`, `node.pair.revoke`) in the `aisopod-gateway` crate. This document captures key learnings and observations from the verification and fix process.

## Implementation Overview

The device pairing protocol enables mobile/desktop devices to securely register with the aisopod server and obtain persistent device tokens. The implementation follows a three-step flow:

1. **Pairing Request**: Client sends device information, server generates a 6-digit pairing code
2. **Pairing Confirmation**: Server validates the pairing code and issues a device token
3. **Pairing Revocation**: Server invalidates a previously paired device's token

## Key Implementation Details

### 1. Pairing Code Generation

```rust
pub fn generate_pairing_code() -> String {
    let mut rng = rand::thread_rng();
    let code: u32 = rng.gen_range(0..1_000_000);
    format!("{:06}", code)
}
```

- Generates 6-digit numeric codes (000000-999999)
- Uses `rand::thread_rng()` for randomness
- Format ensures leading zeros are preserved

### 2. Storage Architecture

The `PairingStore` uses a thread-safe implementation:

```rust
pub struct PairingStore {
    pending_pairings: Mutex<HashMap<String, PendingPairing>>,
    device_to_code: Mutex<HashMap<String, String>>,
}
```

- `pending_pairings`: Maps pairing code → PendingPairing
- `device_to_code`: Maps device_id → pairing code (for bidirectional lookup)
- Both maps wrapped in `Mutex<HashMap<...>>` (not `RwLock` as originally suggested)
- Uses `Arc<PairingStore>` for shared ownership across handlers

### 3. Device Token Persistence

The `DeviceTokenManager` from `crates/aisopod-gateway/src/auth/device_tokens.rs` handles:
- Token issuance with argon2id hashing
- Token validation with hash comparison
- Token revocation (marking as revoked in storage)
- **Persistent storage**: Uses `toml::to_string_pretty()` to save to file

Key observations:
- Tokens are persisted to disk using TOML format
- Storage path is configurable (e.g., `device-tokens-{address}.toml`)
- Load/Save methods handle persistence automatically

### 4. Background Cleanup Task (FIXED)

**Original Issue**: The cleanup function existed but was never spawned, causing expired pairing codes to accumulate in memory.

**Fix Applied**:
```rust
// In server.rs run_with_config()
let pairing_cleanup_interval = Duration::from_secs(gateway_config.pairing_cleanup_interval);
let pairing_store_for_cleanup = pairing_store.clone();
tokio::spawn(async move {
    run_pairing_cleanup_task(pairing_store_for_cleanup, pairing_cleanup_interval).await;
});
```

**Configuration**:
```rust
// In gateway.rs
pub struct GatewayConfig {
    #[serde(default = "default_pairing_cleanup_interval")]
    pub pairing_cleanup_interval: u64,
}

fn default_pairing_cleanup_interval() -> u64 {
    300  // 5 minutes
}
```

**Verification**: Cleanup task now spawns with configured interval on every server startup.

### 5. Method Registration

The pairing methods are registered in `crates/aisopod-gateway/src/ws.rs` within the WebSocket connection handler:

```rust
if let Some(pairing_store_ref) = &pairing_store {
    let pairing_store_for_request = pairing_store_ref.clone();
    let pairing_store_for_confirm = pairing_store_ref.clone();
    let pairing_store_for_revoke = pairing_store_ref.clone();
    
    let token_manager = Arc::new(Mutex::new(DeviceTokenManager::new(...)));
    let token_manager_for_request = token_manager.clone();
    let token_manager_for_confirm = token_manager.clone();
    
    method_router.register("node.pair.request", 
        PairRequestHandler::with_deps(pairing_store_for_request, token_manager_for_request));
    method_router.register("node.pair.confirm", 
        PairConfirmHandler::with_deps(pairing_store_for_confirm, token_manager_for_confirm));
    method_router.register("node.pair.revoke", 
        PairRevokeHandler::with_deps(token_manager));
}
```

**Note**: The handlers receive their dependencies via `with_deps()` rather than storing them in a shared state, which is appropriate for the WebSocket architecture.

## Test Coverage

### Test Results

| Test Name | Status | Description |
|-----------|--------|-------------|
| `test_generate_pairing_code_format` | ✅ Pass | Verifies 6-digit format |
| `test_generate_pairing_code_unique` | ✅ Pass | Verifies uniqueness |
| `test_is_valid_device_type` | ✅ Pass | Validates ios/android/desktop |
| `test_pair_request_success` | ✅ Pass | Happy path for request |
| `test_pair_request_invalid_device_type` | ✅ Pass | Error case |
| `test_pair_request_invalid_device_id` | ✅ Pass | Error case |
| `test_pair_request_missing_params` | ✅ Pass | Error case |
| `test_pair_confirm_success` | ✅ Pass | Happy path for confirm |
| `test_pair_confirm_invalid_code` | ✅ Pass | Invalid code error |
| `test_pair_confirm_device_id_mismatch` | ✅ Pass | Device ID validation |
| `test_pairing_store_cleanup_expired` | ✅ Pass | Cleanup functionality |
| `test_pair_revoke_nonexistent_device` | ✅ Pass | Revoke nonexistent |

**Total**: 12/12 tests passing

### Test Architecture

Tests use a dependency injection pattern:

```rust
fn create_handlers() -> (PairRequestHandler, PairConfirmHandler, PairRevokeHandler) {
    let pairing_store = Arc::new(PairingStore::new());
    let token_manager = Arc::new(Mutex::new(DeviceTokenManager::new(...)));

    let request_handler = PairRequestHandler::with_deps(pairing_store.clone(), token_manager.clone());
    let confirm_handler = PairConfirmHandler::with_deps(pairing_store.clone(), token_manager.clone());
    let revoke_handler = PairRevokeHandler::with_deps(token_manager);

    (request_handler, confirm_handler, revoke_handler)
}
```

This pattern allows:
- Independent test isolation
- Controlled state management
- Proper Arc cloning for shared state

## Code Quality Observations

### Strengths

1. **Clear separation of concerns**: Each handler is a separate struct with distinct responsibilities
2. **Comprehensive parameter validation**: Device type, device_id (UUID), pairing code existence
3. **Proper error handling**: Uses standard JSON-RPC error codes (-32602 for invalid params, -32003 for application errors)
4. **Thread-safe storage**: Uses `Arc<Mutex<...>>` pattern consistently
5. **Well-structured data models**: Request/response types match RPC interface
6. **Background task pattern**: Cleanup task follows tokio spawning best practices

### Areas for Future Improvement

1. **Make expiry configurable**: The 5-minute expiry is hardcoded as `PAIRING_CODE_EXPIRY`
2. **Add API endpoint to list paired devices**: Currently only revocation is supported
3. **Add metrics for pairing operations**: Track success/failure rates
4. **Token refresh functionality**: For long-lived devices

## Key Lessons Learned

### Lesson 1: Configuration for Background Tasks

**Pattern**: Make background task intervals configurable via config file.

**Implementation**:
```rust
// 1. Add field to config struct
pub struct GatewayConfig {
    #[serde(default = "default_pairing_cleanup_interval")]
    pub pairing_cleanup_interval: u64,
}

// 2. Read from config and spawn task
let interval = Duration::from_secs(gateway_config.pairing_cleanup_interval);
tokio::spawn(async move {
    run_pairing_cleanup_task(store, interval).await;
});
```

**Why**: Allows operators to adjust cleanup frequency without code changes.

### Lesson 2: Arc Cloning for Spawned Tasks

When spawning a task that needs access to shared state:

```rust
// 1. Create Arc
let store = Arc::new(PairingStore::new());

// 2. Clone for spawned task
let store_for_task = store.clone();
tokio::spawn(async move {
    run_cleanup_task(store_for_task, interval).await;
});
```

**Why**: The spawned task owns its copy of the Arc, preventing lifetime issues.

### Lesson 3: Cleanup Task Should Be Spawns, Not Called Directly

**Mistake (Before Fix)**:
```rust
// Cleanup function exists but is never called
pub async fn run_pairing_cleanup_task(...) { ... }

// No code calls this function
```

**Fix**:
```rust
tokio::spawn(async move {
    run_pairing_cleanup_task(store, interval).await;
});
```

**Why**: Background tasks must be spawned to run concurrently with main application logic.

### Lesson 4: Test Dependency Injection Pattern

```rust
fn create_handlers() -> (Handler1, Handler2, Handler3) {
    let shared_state = Arc::new(Shared::new());
    let shared_for_h1 = shared_state.clone();
    let shared_for_h2 = shared_state.clone();
    
    (Handler1::with_deps(shared_for_h1), Handler2::with_deps(shared_for_h2))
}
```

**Why**: Tests can control shared state while handlers receive their dependencies.

## Conclusion

The device pairing protocol implementation is functionally complete and production-ready. The fix for the cleanup task spawn ensures expired pairing codes are regularly removed, preventing memory leaks.

The implementation demonstrates:
- Clean separation of concerns
- Proper async patterns with tokio
- Comprehensive error handling
- Configurable background tasks
- Well-structured test patterns

---
*Document created: 2026-02-26*
*Based on verification of Issue #163*
