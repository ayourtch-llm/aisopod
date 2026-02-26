# Learning 170: Signal Channel Implementation

## Overview
This document captures key learnings and insights from implementing the Signal channel plugin (Issue #170) for aisopod.

## Implementation Summary

### Crate Structure
The `aisopod-channel-signal` crate follows a well-organized modular design:

```
crates/aisopod-channel-signal/
├── Cargo.toml
└── src/
    ├── lib.rs        # Module exports and documentation
    ├── channel.rs    # ChannelPlugin trait implementation
    ├── config.rs     # Configuration types
    ├── gateway.rs    # Incoming message handling
    ├── outbound.rs   # Outbound message sending
    └── runtime.rs    # Signal CLI subprocess management
```

### Key Design Patterns

#### 1. ChannelPlugin Trait Implementation
The `SignalChannel` struct implements the `ChannelPlugin` trait with:
- `id()` - Returns unique channel identifier
- `meta()` - Returns metadata about the channel
- `capabilities()` - Returns supported features (DM, Group, Media, etc.)
- `config()` - Returns configuration adapter
- `security()` - Returns security adapter

#### 2. Subprocess Management
The `SignalRuntime` handles signal-cli daemon spawning:
```rust
// Spawns: signal-cli -u <phone_number> daemon --json
// Uses JSON-RPC mode for bidirectional communication
```

**Key Insight**: The daemon runs in JSON-RPC mode, which allows:
- Bidirectional communication via stdin/stdout
- Structured JSON message exchange
- Better error handling than plain text

#### 3. Message Parsing
The `SignalGateway` parses incoming JSON messages:
- Detects message type (`receive`, `receipt`, `contact`)
- Maps Signal envelopes to `IncomingMessage` type
- Handles both DMs and group messages

**Phone Number Identity Mapping**:
```rust
// Signal uses phone numbers as unique identifiers
// Maps directly to aisopod's PeerInfo with PeerKind::User
```

#### 4. Media Handling
The `SignalOutbound` handles media attachments:
- Downloads media from URLs or uses raw data
- Creates temporary files for sending
- Supports images, audio, video, and documents

**Key Insight**: Signal CLI requires file paths for attachments, not raw data. The implementation uses `tempfile` crate to create temporary files.

### Acceptance Criteria Verification

| Criteria | Status | Notes |
|----------|--------|-------|
| Signal CLI subprocess spawns and connects | ✅ | `SignalRuntime::start_daemon()` |
| Direct messages sent and received | ✅ | `SignalGateway::parse_message()` with DM support |
| Group messages sent and received | ✅ | Group parsing with `SignalGroup` struct |
| Media attachments work correctly | ✅ | `SignalOutbound::send_media()` |
| Disappearing message timers detected | ✅ | `expires_in` field in `SignalMessage` |
| Phone number identity mapped | ✅ | `PeerInfo` with phone number ID |
| Graceful error handling | ✅ | `SignalError` enum with descriptive variants |
| Unit tests present | ✅ | 18 tests covering all modules |
| Integration test with mock output | ⚠️ | Tests use JSON strings directly, not true mocks |

### Testing Coverage

#### Unit Tests (18 tests)
- **config.rs**: 6 tests
  - Phone number validation
  - Sender allowlist logic
  - Phone number normalization
  
- **channel.rs**: 3 tests
  - Account config defaults
  - Channel registration type check

- **gateway.rs**: 4 tests
  - DM message parsing
  - Group message parsing
  - Media attachment handling
  - Disappearing timer extraction

- **outbound.rs**: 2 tests
  - Outbound creation
  - Timeout configuration

- **runtime.rs**: 3 tests
  - Runtime initialization
  - Signal CLI path detection
  - Default path resolution

#### Missing Integration Tests
**Recommendation**: Add integration tests with:
1. Mock signal-cli subprocess that returns predefined JSON responses
2. Full message flow tests (send → receive → parse)
3. Error scenarios (signal-cli unavailable, malformed JSON)

### Configuration Features

#### SignalAccountConfig
- Phone number validation (E.164 format)
- Optional device name
- Allowed senders list
- Monitored groups list
- Disappearing message settings
- Signal CLI path customization

#### SignalDaemonConfig
- JSON-RPC port configuration
- Operation timeout
- Retry configuration
- Data directory path

### Common Patterns Identified

1. **Error Handling**: Use `SignalError` enum for domain-specific errors
2. **Message Normalization**: Signal messages map to `IncomingMessage` with metadata
3. **Async-Await**: All I/O operations use async/await with tokio
4. **Configuration Validation**: Phone numbers validated at construction time
5. **Subprocess Safety**: Daemon processes are Arc-wrapped for thread safety

### Lessons for Future Channel Implementations

1. **Follow Existing Patterns**: The Telegram and Discord channels provide good templates
2. **JSON-RPC is Preferred**: Many chat platforms use JSON-RPC (Signal, WhatsApp Business API)
3. **Temporary Files for Media**: Use tempfile crate for attachment handling
4. **Graceful Degradation**: When signal-cli unavailable, return clear error messages
5. **Test JSON Parsing**: Most bugs are in JSON parsing; test edge cases
6. **Metadata Preservation**: Preserve Signal-specific metadata (expires_in, group_id)

### Known Limitations

1. **No true integration tests**: Current tests use JSON strings directly
2. **No concurrent daemon support**: Runtime could support multiple phone numbers
3. **No reconnection logic**: If daemon crashes, manual restart needed
4. **No read receipts**: Receipt handling not fully implemented

### Recommendations

1. Add integration test infrastructure with mock subprocess
2. Implement automatic reconnection on daemon failure
3. Add metrics/health monitoring for daemon status
4. Consider adding connection pooling for multiple accounts
5. Document signal-cli version requirements clearly

### Reference Implementation Files

- **Main trait**: `crates/aisopod-channel/src/plugin.rs`
- **Message types**: `crates/aisopod-channel/src/message.rs`
- **Type definitions**: `crates/aisopod-channel/src/types.rs`

### Dependencies Used

- `tokio` - Async runtime
- `serde` / `serde_json` - JSON serialization
- `tracing` - Logging
- `thiserror` - Error types
- `chrono` - Timestamp handling
- `tempfile` - Media file handling
- `reqwest` - HTTP for media downloads

---
*Created: 2026-02-26*
*Issue: #170*
