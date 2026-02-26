# Protocol Version Negotiation Implementation

## Summary

This document captures the learnings and implementation details for protocol version negotiation in the aisopod gateway WebSocket protocol.

## Problem Statement

Previously, the WebSocket server assumed all clients spoke the same protocol version, which would break when the protocol evolved. There was no mechanism to:
1. Negotiate compatible protocol versions between client and server
2. Gracefully reject connections with incompatible versions
3. Provide helpful error messages when version mismatch occurs

## Implementation Details

### Version Module Structure

Created `crates/aisopod-gateway/src/ws/version.rs` with:

1. **ProtocolVersion struct** - Represents a semantic version with major and minor numbers
   ```rust
   #[derive(Debug, Clone, PartialEq, Eq)]
   pub struct ProtocolVersion {
       pub major: u32,
       pub minor: u32,
   }
   ```

2. **parse() method** - Parses version strings like "1.0" or "2.3"
   - Returns `VersionError::InvalidFormat` for malformed versions
   - Requires exactly two dot-separated parts

3. **is_compatible_with() method** - Checks compatibility using semantic versioning rules:
   - Major version must match exactly
   - Server minor version must be >= client minor version

4. **VersionError enum** - Error types for version parsing

5. **VersionNegotiationError enum** - Error types for negotiation failures:
   - `Incompatible` - Client version is not compatible with server
   - `ParseError` - Client sent a malformed version string

6. **negotiate_version() function** - Central negotiation function:
   - Extracts version from `X-Aisopod-Protocol-Version` header
   - Defaults to "1.0" if header is missing (backward compatibility)
   - Returns compatible client version or error

### Server Configuration

- Server protocol version: `ProtocolVersion { major: 1, minor: 0 }`
- Version header: `X-Aisopod-Protocol-Version`

### WebSocket Integration

Modified `crates/aisopod-gateway/src/ws.rs`:

1. **Version check before upgrade** - Extract version from request headers
2. **Compatible path** - Proceed with WebSocket upgrade
3. **Incompatible path** - Send JSON-RPC error message then close

Error response format for incompatible version:
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32010,
    "message": "Protocol version mismatch",
    "data": {
      "server_version": "1.0",
      "client_version": "2.0",
      "hint": "Please upgrade/downgrade your client to match the server version"
    }
  }
}
```

### Error Handling Flow

1. Client sends WebSocket upgrade with `X-Aisopod-Protocol-Version` header
2. Server extracts and parses version
3. If version is valid and compatible:
   - Proceed with WebSocket upgrade
   - Version information is available in connection context
4. If version is invalid (parse error):
   - Send JSON-RPC error response with code -32010
   - Close connection
5. If version is incompatible:
   - Send JSON-RPC error response with code -32010
   - Include server and client versions in error data
   - Provide hint for client action
   - Close connection

## Compatibility Rules

The semantic versioning rules ensure:
- Server can accept clients with older or equal minor versions (backward compatible)
- Server rejects clients with newer minor versions (might have features server doesn't support)
- Server rejects clients with different major versions (breaking changes)

### Examples with Server Version 1.0

| Client Version | Result | Reason |
|---------------|--------|--------|
| 1.0 | ✓ Compatible | Exact match |
| 0.9 | ✓ Compatible | Older major (future-proof for 0.x series) |
| 1.1 | ✗ Incompatible | Client has newer minor |
| 2.0 | ✗ Incompatible | Different major (breaking changes) |

## Testing

Created comprehensive unit tests covering:
- Valid version parsing ("1.0", "2.3", "0.1")
- Invalid version formats ("1", "1.0.0", "abc.def")
- Version display formatting
- Compatibility checks:
  - Exact match
  - Server higher minor
  - Client higher minor
  - Different major
  - Zero major handling
- Negotiation tests:
  - Valid versions
  - Invalid versions (parse error)
  - Incompatible versions
  - Missing version (defaults to 1.0)

## Documentation

Added doc tests with working examples for:
- `ProtocolVersion::is_compatible_with()`
- `negotiate_version()`

## Files Modified

1. **crates/aisopod-gateway/src/ws/version.rs** - New file
2. **crates/aisopod-gateway/src/ws.rs** - Modified
   - Added `pub mod version;`
   - Modified `ws_handler()` to check version before upgrade
   - Added `handle_connection_with_error()` for error responses

## Lessons Learned

1. **Placement of version check** - The version check happens inside the `on_upgrade` closure because:
   - The WebSocketUpgrade extractor must be used to perform the upgrade
   - We need access to the WebSocket connection to send error messages
   - The version check must happen before the connection is fully established

2. **Error response format** - Using JSON-RPC format for errors allows:
   - Consistency with the rest of the protocol
   - Clients can parse errors the same way as RPC responses
   - Provides structured error data with hints

3. **Backward compatibility** - Defaulting to version "1.0" when header is missing:
   - Ensures existing clients continue to work
   - Allows gradual adoption of the version header

4. **Testing approach** - Unit tests verify:
   - Version parsing behavior
   - Compatibility logic
   - Error handling
   - Doc tests verify working examples

## Future Considerations

1. **Version caching** - Consider caching parsed versions for performance
2. **Multiple version support** - Server might support multiple version ranges
3. **Deprecation warnings** - Could warn clients about outdated versions
4. **Version negotiation protocol** - For more complex cases, could implement a full negotiation handshake
