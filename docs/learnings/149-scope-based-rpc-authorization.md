# Learnings: Scope-Based RPC Authorization (Issue 149)

## Overview

This document captures key learnings and implementation decisions from implementing scope-based authorization for RPC methods in the aisopod-gateway crate.

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

## Testing Strategy

### 1. Unit Tests

We wrote comprehensive unit tests covering:
- Scope enum operations (as_str, display, allows)
- Method to scope mapping
- Scope checking for various combinations
- Edge cases (empty scopes, unknown methods, admin access)

### 2. Integration Testing

The existing integration tests verify:
- WebSocket connections work with scope-based authorization
- Unauthorized methods are properly rejected
- Authorized methods work correctly

## Lessons Learned

### 1. Scope Granularity

The initial design had a very simple "exact match" check. We realized early that this wasn't flexible enough - admin users should be able to access all methods. The hierarchical approach with `Scope::allows()` provided the right level of flexibility.

### 2. Type Safety

Using a proper `Scope` enum with a custom `allows()` method is more type-safe than using string comparisons. This prevents typos and makes the scope relationships explicit.

### 3. Early Failure

By checking scopes at the router level, we fail fast without executing any handler logic. This is more efficient and provides clearer security boundaries.

### 4. Centralized Configuration

The `METHOD_SCOPES` static map serves as a single source of truth for scope requirements. When adding new methods, developers just need to update this map.

## Future Considerations

### 1. Dynamic Scope Configuration

Currently, scopes are hardcoded. In the future, scopes could be configured dynamically via the configuration file or database.

### 2. Custom Scope Definitions

Allowing users to define custom scopes with custom names could be useful for complex deployments with many roles.

### 3. Scope Inheritance

The current hierarchy is simple (admin > others). More complex inheritance patterns (e.g., "read + specific scope = additional access") might be needed for advanced use cases.

### 4. Auditing and Logging

It would be beneficial to log scope violations for security auditing. This could be added to the `check_scope()` function.

## Code Review Checklist

When reviewing similar scope-based authorization changes:
- [ ] Is the scope mapping centralized in one place?
- [ ] Are all methods in `METHOD_SCOPES` assigned a scope?
- [ ] Does `Scope::allows()` correctly represent the hierarchy?
- [ ] Are scope checks performed before handler execution?
- [ ] Are error codes consistent with JSON-RPC 2.0?
- [ ] Are unit tests comprehensive for scope combinations?
- [ ] Does the implementation handle edge cases (empty scopes, unknown methods)?
