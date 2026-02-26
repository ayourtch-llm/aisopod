# Learning 168: Reference WebSocket Client Implementation

## Summary

This learning captures the key insights and implementation details from building the aisopod WebSocket client library (issue #168), particularly focusing on the critical fixes applied for the tungstenite WebSocket library integration.

## Critical Issues Encountered and Fixed

### 1. tungstenite Request::new() Signature Mismatch

**Problem**: The `tungstenite::handshake::client::Request::new()` signature changed from accepting parameters directly to using the builder pattern.

**Original Code (Incorrect)**:
```rust
let mut request = tungstenite::handshake::client::Request::new();
```

**Fixed Code**:
```rust
let mut request = tungstenite::handshake::client::Request::new(());
*request.uri_mut() = url_str.parse().map_err(...)?;
```

**Key Takeaway**: The tungstenite 0.21 library uses a unit tuple `()` for `Request::new()` and requires setting the URI via the `uri_mut()` method. This is a common breaking change in WebSocket libraries between versions.

### 2. ClientError::Closed Variant Change

**Problem**: The `tungstenite::Error::Closed` variant changed from a tuple variant to a unit variant.

**Original Code (Incorrect)**:
```rust
Err(ClientError::Closed(e))  // Expected tuple
```

**Fixed Code**:
```rust
Err(ClientError::Closed)  // Unit variant
```

**Key Takeaway**: When upgrading tungstenite or similar WebSocket libraries, variant definitions may change. Use `cargo expand` or `cargo doc --open` to verify current API definitions.

### 3. parse_response() Lifetime Issue

**Problem**: Using `&str` in `parse_response()` caused lifetime issues when parsing JSON-RPC responses.

**Original Code (Incorrect)**:
```rust
pub fn parse_response(json_str: &str) -> std::result::Result<RpcResponse, ParseResponseError>
```

**Fixed Code**:
```rust
pub fn parse_response(json_str: &str) -> std::result::Result<RpcResponse, ParseResponseError>
```

**Note**: The function actually kept using `&str`, but the issue was resolved by ensuring the `RpcResponse` struct stores owned `String` values rather than references.

**Key Takeaway**: JSON parsing with `serde` typically requires owned data (`String`) rather than borrowed references (`&str`) unless using lifetime-annotated deserialization with `serde_json::from_str` and `PhantomData`.

### 4. uuid Crate Missing "std" Feature

**Problem**: The `uuid` crate requires the "std" feature for methods like `is_v4()` (or `get_version()` in newer versions).

**Fix Applied in Workspace Cargo.toml**:
```toml
uuid = { version = "1", features = ["v4", "serde", "std"] }
```

**Usage in aisopod-client**:
```toml
uuid = { workspace = true }
```

**Verification**:
```rust
use uuid::Uuid;

// For newer uuid versions (1.6+)
let id = Uuid::new_v4();
assert_eq!(id.get_version(), Some(uuid::Version::Random));

// For older uuid versions
// assert!(id.is_v4());
```

**Key Takeaway**: The `uuid` crate's `std` feature is required for random number generation (UUID v4). Without it, methods like `new_v4()` are not available. Always specify `features = ["std"]` when using uuid in non-no_std contexts.

## Implementation Checklist

Based on issue #168 requirements, the following was implemented:

- [x] `crates/aisopod-client` crate created and compiles
- [x] Client connects to aisopod server via WebSocket with proper handshake headers
- [x] Handshake sends correct upgrade headers (`Authorization`, `X-Aisopod-Client`, `X-Aisopod-Device-Id`, `X-Aisopod-Protocol-Version`)
- [x] Client receives and parses the welcome message
- [x] `request()` sends JSON-RPC and matches response by ID
- [x] Server events are received and dispatched to the event channel
- [x] Helper methods exist for chat, node pairing, node describe, and node invoke
- [x] Basic integration tests demonstrate connect → authenticate → request → disconnect flow

## Workspace Dependencies Pattern

The project follows a workspace dependency pattern for consistent versions:

1. Define dependencies in root `Cargo.toml` `[workspace.dependencies]`
2. Reference with `workspace = true` in member crates
3. Add crate-specific features as needed

**Example**:
```toml
# Root Cargo.toml
[workspace.dependencies]
uuid = { version = "1", features = ["v4", "serde", "std"] }

# Member crate Cargo.toml
[dependencies]
uuid = { workspace = true }
```

This ensures version consistency and makes feature management easier.

## Testing Strategy

The client library includes:

1. **Unit tests** in `src/client.rs`:
   - `test_client_config_defaults`
   - `test_build_auth_request`
   - `test_auth_request_serialization`

2. **Integration tests** in `tests/integration_test.rs`:
   - Connection lifecycle
   - JSON-RPC request/response serialization
   - Error response handling
   - Device capability/information serialization

3. **Test pattern**: Tests verify message serialization matches the protocol specification.

## Recommendations for Future Reference Client Development

1. **Always check tungstenite version compatibility** before implementing WebSocket handshake
2. **Use `cargo expand`** to see expanded macros and verify API usage
3. **Review changelogs** when upgrading `uuid`, `tungstenite`, or similar low-level crates
4. **Use workspace dependencies** for consistent version management across crates
5. **Include both unit and integration tests** to catch both API changes and protocol compliance issues
6. **Test with actual server implementations** early to catch protocol mismatch issues
7. **Document API breaking changes** in issue resolution files for future reference

## Related Files

- `crates/aisopod-client/Cargo.toml` - Dependencies including uuid with std feature
- `crates/aisopod-client/src/client.rs` - Main client implementation with handshake fix
- `crates/aisopod-client/src/error.rs` - ClientError enum with unit variant Closed
- `crates/aisopod-client/src/message.rs` - JSON-RPC parsing with owned String types
- `crates/aisopod-client/src/types.rs` - Data types for protocol messages
- `crates/aisopod-client/tests/integration_test.rs` - Integration tests
- `docs/issues/open/168-reference-websocket-client.md` - Original issue description

## Conclusion

The reference WebSocket client library has been successfully built and tested. The critical fixes for tungstenite integration were applied and verified through the test suite. The implementation follows the project's design patterns and serves as a solid foundation for future client development.
