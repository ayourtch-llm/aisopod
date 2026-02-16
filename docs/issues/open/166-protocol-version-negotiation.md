# Issue 166: Implement Protocol Version Negotiation

## Summary
Implement version negotiation during the WebSocket handshake so that clients and servers can agree on a compatible protocol version, and gracefully reject connections when versions are incompatible.

## Location
- Crate: `aisopod-gateway`
- File: `crates/aisopod-gateway/src/ws/version.rs`

## Current Behavior
No protocol versioning exists. All clients are assumed to speak the same protocol, which will break when the protocol evolves.

## Expected Behavior
1. The client sends its supported protocol version in the WebSocket upgrade request via the `X-Aisopod-Protocol-Version` header (e.g. `1.0`).
2. The server checks compatibility using semantic versioning rules:
   - **Major** version must match exactly.
   - **Minor** version: server must be ≥ client (server is backward-compatible).
3. If compatible, the server includes the negotiated version in the welcome message.
4. If incompatible, the server sends an error message and closes the connection with a descriptive close reason.

## Impact
Version negotiation prevents silent protocol mismatches that cause cryptic runtime errors. It enables safe, incremental protocol evolution.

## Suggested Implementation
1. Create `crates/aisopod-gateway/src/ws/version.rs`.
2. Define the version type:
   ```rust
   #[derive(Debug, Clone, PartialEq, Eq)]
   pub struct ProtocolVersion {
       pub major: u32,
       pub minor: u32,
   }

   impl ProtocolVersion {
       pub fn new(major: u32, minor: u32) -> Self {
           Self { major, minor }
       }

       /// Parse from a string like "1.0" or "2.3"
       pub fn parse(s: &str) -> Result<Self, VersionError> {
           let parts: Vec<&str> = s.split('.').collect();
           if parts.len() != 2 {
               return Err(VersionError::InvalidFormat(s.to_string()));
           }
           let major = parts[0].parse::<u32>()
               .map_err(|_| VersionError::InvalidFormat(s.to_string()))?;
           let minor = parts[1].parse::<u32>()
               .map_err(|_| VersionError::InvalidFormat(s.to_string()))?;
           Ok(Self { major, minor })
       }

       /// Check if this server version is compatible with a client version.
       pub fn is_compatible_with(&self, client: &ProtocolVersion) -> bool {
           self.major == client.major && self.minor >= client.minor
       }
   }

   impl std::fmt::Display for ProtocolVersion {
       fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
           write!(f, "{}.{}", self.major, self.minor)
       }
   }

   #[derive(Debug, thiserror::Error)]
   pub enum VersionError {
       #[error("Invalid version format: {0}")]
       InvalidFormat(String),
   }
   ```
3. Integrate into the WebSocket upgrade handler:
   ```rust
   const SERVER_PROTOCOL_VERSION: ProtocolVersion = ProtocolVersion { major: 1, minor: 0 };

   pub fn negotiate_version(
       headers: &HeaderMap,
   ) -> Result<ProtocolVersion, VersionNegotiationError> {
       let client_version_str = headers
           .get("X-Aisopod-Protocol-Version")
           .and_then(|v| v.to_str().ok())
           .unwrap_or("1.0"); // Default to 1.0 if header missing

       let client_version = ProtocolVersion::parse(client_version_str)?;

       if SERVER_PROTOCOL_VERSION.is_compatible_with(&client_version) {
           Ok(client_version)
       } else {
           Err(VersionNegotiationError::Incompatible {
               server: SERVER_PROTOCOL_VERSION.clone(),
               client: client_version,
           })
       }
   }
   ```
4. On incompatible version, send an error JSON-RPC message before closing:
   ```json
   {
     "jsonrpc": "2.0",
     "error": {
       "code": -32010,
       "message": "Protocol version mismatch",
       "data": {
         "server_version": "1.0",
         "client_version": "2.0",
         "hint": "Please upgrade/downgrade your client"
       }
     }
   }
   ```
5. Include `protocol_version` in the welcome message params (see issue 162).

## Dependencies
- Issue 028 (WebSocket connection lifecycle)

## Acceptance Criteria
- [ ] Client can send protocol version via `X-Aisopod-Protocol-Version` header
- [ ] Server checks major/minor version compatibility
- [ ] Compatible versions result in a successful welcome message with negotiated version
- [ ] Incompatible versions result in an error message and connection close
- [ ] Missing version header defaults to `1.0`
- [ ] Unit tests cover compatible, incompatible, missing, and malformed version strings

---
*Created: 2026-02-15*
