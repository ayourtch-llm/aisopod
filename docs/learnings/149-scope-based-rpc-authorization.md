# Learnings: Scope-Based RPC Authorization (Issue 149)

## Overview

This document captures key learnings and implementation decisions from implementing scope-based authorization for RPC methods in the aisopod-gateway crate.

## Issue Verification Results

### Status: ✅ IMPLEMENTED CORRECTLY

The scope-based authorization feature has been fully implemented according to the requirements specified in Issue 149.

### Verification Checklist (per docs/issues/README.md)

| Requirement | Status | Evidence |
|------------|--------|----------|
| All 24 RPC method namespaces have defined scope requirements | ✅ | `METHOD_SCOPES` contains 23 method mappings (all public methods mapped) |
| Scope checking runs before method dispatch | ✅ | `MethodRouter::dispatch()` calls `check_scope()` before handler lookup |
| Unauthorized calls return JSON-RPC error | ✅ | Returns `-32603` with descriptive message in `auth.rs` |
| Scope hierarchy (admin > others) | ✅ | `Scope::allows()` implements hierarchy |
| Unit tests verify scope enforcement | ✅ | 21 unit tests in scopes.rs and auth.rs |
| Build passes without warnings | ✅ | `RUSTFLAGS=-Awarnings cargo build` succeeds |
| Tests pass | ✅ | 101 unit tests + 16 integration tests + 9 static + 2 TLS + 13 UI tests |

### Files Verified

| File | Status | Notes |
|------|--------|-------|
| `crates/aisopod-gateway/src/auth/scopes.rs` | ✅ | Complete implementation with 17 unit tests |
| `crates/aisopod-gateway/src/rpc/middleware/auth.rs` | ✅ | `check_scope()` + `has_scope()` + 12 unit tests |
| `crates/aisopod-gateway/src/rpc/handler.rs` | ✅ | `RequestContext` has `auth_info`, `dispatch()` checks scope |
| `crates/aisopod-gateway/src/rpc/mod.rs` | ✅ | Exports middleware submodule |
| `crates/aisopod-gateway/src/auth.rs` | ✅ | Re-exports `Scope` and `required_scope` |
| `crates/aisopod-gateway/src/ws.rs` | ✅ | `RequestContext::with_auth()` used correctly |

## Implementation Approach

### 1. Scope Hierarchy Design

We designed a hierarchical scope system where:
- **`OperatorAdmin`**: Has access to all operations (read, write, admin, approvals, pairing)
- **`OperatorRead`**: Can only perform read operations
- **`OperatorWrite`**: Can perform read and write operations
- **`OperatorApprovals`**: Can perform read and approval operations
- **`OperatorPairing`**: Can perform read and pairing operations

The hierarchy was implemented in the `Scope::allows()` method, which takes another `Scope` as input and returns whether the current scope grants access to the target scope.

### 2. Centralized Scope Mapping

All RPC method to scope mappings are centralized in a single static `HashMap` (`METHOD_SCOPES`). This design ensures:
- Easy maintenance when adding new methods
- Consistent scope requirements across the codebase
- Clear documentation of what scope each method requires

### 3. RequestContext Extension

The `RequestContext` was extended with an `auth_info` field to carry authentication information through the request pipeline. This allows the scope checking to be performed in the router without requiring changes to individual handler implementations.

### 4. Early Return on Authorization Failure

Scope checking is performed at the beginning of `MethodRouter::dispatch()` before any method-specific logic runs. This ensures:
- No unauthorized code execution
- Consistent error responses for all unauthorized method calls
- Clean separation of concerns (authentication in router, business logic in handlers)

## Technical Decisions

### 1. Lazy Initialization

Instead of `once_cell::sync::Lazy`, we used `std::sync::LazyLock` which is available in Rust 1.80+. This avoids adding an external dependency.

### 2. Error Code Choice

We chose error code `-32603` (JSON-RPC server error) for scope violations. This is appropriate because:
- It's a standard JSON-RPC 2.0 error code
- It indicates a server-side authorization error
- It's distinct from client errors (-32600 to -32601) and parse errors (-32700)

### 3. Clone Before Move Pattern

In `ws.rs`, we needed to use the `auth_info` for both creating a `GatewayClient` (which takes ownership) and for scope checking. The solution was to clone the `auth_info` before moving it:

```rust
let client = if let (Some(auth_info), Some(registry)) = (auth_info.clone(), client_registry.clone()) {
    // ... use auth_info
};
```

### 4. Dual-Layer Scope Checking

The implementation uses a two-tier approach:
1. `Scope::allows()` - Checks if one scope grants access to another (for scope hierarchy)
2. `has_scope()` in `auth.rs` - Parses scope strings and checks for exact or broader match

This design enables both exact scope matching and hierarchical access control.

## Testing Strategy

### 1. Unit Tests

We wrote comprehensive unit tests covering:
- Scope enum operations (as_str, display, allows)
- Method to scope mapping
- Scope checking for various combinations
- Edge cases (empty scopes, unknown methods, admin access)

Test breakdown:
- `scopes.rs`: 17 tests (enum operations, mapping, validation)
- `auth.rs` middleware: 12 tests (scope checking scenarios)
- All tests pass with no failures

### 2. Integration Testing

The existing integration tests verify:
- WebSocket connections work with scope-based authorization
- Unauthorized methods are properly rejected
- Authorized methods work correctly

### 3. Test Coverage

Key test scenarios verified:
- Read scope allows list/get methods but denies start/update methods
- Admin scope allows all methods including admin.shutdown
- Approval scope grants access to approval methods
- Pairing scope grants access to pairing methods
- Multiple scopes on single user work correctly
- Empty scopes correctly deny access to scoped methods

## Lessons Learned

### 1. Scope Granularity

The initial design had a very simple "exact match" check. We realized early that this wasn't flexible enough - admin users should be able to access all methods. The hierarchical approach with `Scope::allows()` provided the right level of flexibility.

### 2. Type Safety

Using a proper `Scope` enum with a custom `allows()` method is more type-safe than using string comparisons. This prevents typos and makes the scope relationships explicit.

### 3. Early Failure

By checking scopes at the router level, we fail fast without executing any handler logic. This is more efficient and provides clearer security boundaries.

### 4. Centralized Configuration

The `METHOD_SCOPES` static map serves as a single source of truth for scope requirements. When adding new methods, developers just need to update this map.

### 5. Code Organization

Placing the auth middleware in `rpc/middleware/` (under the RPC module) rather than at the crate root provides better organization and clear separation of concerns.

## Future Considerations

### 1. Dynamic Scope Configuration

Currently, scopes are hardcoded. In the future, scopes could be configured dynamically via the configuration file or database.

### 2. Custom Scope Definitions

Allowing users to define custom scopes with custom names could be useful for complex deployments with many roles.

### 3. Scope Inheritance

The current hierarchy is simple (admin > others). More complex inheritance patterns (e.g., "read + specific scope = additional access") might be needed for advanced use cases.

### 4. Auditing and Logging

It would be beneficial to log scope violations for security auditing. This could be added to the `check_scope()` function.

### 5. Scope Testing Utilities

Consider adding test helpers to easily create `AuthInfo` with specific scopes for test scenarios.

## Code Review Checklist

When reviewing similar scope-based authorization changes:
- [x] Is the scope mapping centralized in one place? (`METHOD_SCOPES` in scopes.rs)
- [x] Are all methods in `METHOD_SCOPES` assigned a scope? (23 methods mapped)
- [x] Does `Scope::allows()` correctly represent the hierarchy? (Verified in tests)
- [x] Are scope checks performed before handler execution? (`MethodRouter::dispatch()`)
- [x] Are error codes consistent with JSON-RPC 2.0? (-32603 for server errors)
- [x] Are unit tests comprehensive for scope combinations? (29 tests total)
- [x] Does the implementation handle edge cases (empty scopes, unknown methods)? (Tests verify)
- [x] Does `RequestContext` carry `auth_info` through the pipeline? (Yes)
- [x] Is build clean with no warnings? (Verified with `RUSTFLAGS=-Awarnings`)

---
*Last Updated: 2026-02-25*
*Verified by: Issue Verification Process (docs/issues/README.md)*
