# Signal Channel Implementation

## Overview

This document captures key learnings and design decisions from implementing the Signal channel plugin for aisopod.

## Architecture

The Signal channel plugin follows the standard aisopod channel plugin architecture:

```
crates/aisopod-channel-signal/
├── Cargo.toml                      # Dependencies and package metadata
└── src/
    ├── lib.rs                      # Main entry point and re-exports
    ├── channel.rs                  # ChannelPlugin implementation
    ├── config.rs                   # Configuration types
    ├── outbound.rs                 # Message sending logic
    ├── gateway.rs                  # Message receiving and parsing
    └── runtime.rs                  # signal-cli daemon management
```

## Key Design Decisions

### 1. signal-cli Integration

The channel uses `signal-cli` as the underlying communication layer. signal-cli is a command-line tool for Signal that supports:
- Message sending and receiving
- Media attachment handling
- Group management
- JSON output format

We communicate with signal-cli via subprocess calls to commands like:
- `signal-cli -u <phone> send <recipient>`
- `signal-cli -u <phone> daemon --json` (for background daemon)

### 2. ChannelPlugin Trait Implementation

The `SignalChannel` struct implements the `ChannelPlugin` trait with:

- **id()**: Returns the channel identifier (`signal-<account_id>`)
- **meta()**: Returns channel metadata (label, docs URL, UI hints)
- **capabilities()**: Returns supported features (DMs, groups, media)
- **config()**: Returns `ChannelConfigAdapter` for account management
- **security()**: Returns `SecurityAdapter` for sender filtering

### 3. State Management

The channel maintains:
- **Accounts**: Vector of `SignalAccount` structs with connection state
- **Runtime**: `Arc<tokio::sync::Mutex<SignalRuntime>>` for daemon management
- **Gateway**: `SignalGateway` for incoming message parsing
- **Outbound**: `SignalOutbound` for message sending

### 4. Async Design

All operations use async/await for:
- Non-blocking I/O for subprocess communication
- Concurrent message handling
- Graceful shutdown with `tokio::sync::Notify`

## Features Implemented

### Direct Messages
- Phone number-based addressing
- Message body parsing
- Media attachment support

### Group Messages
- Group ID extraction from message metadata
- Group name parsing
- Member listing (via signal-cli)

### Media Attachments
- Download from URLs to temporary files
- Support for images, audio, video, documents
- MIME type detection

### Disappearing Messages
- Timer extraction from message metadata
- Notification when messages expire
- Configurable per-account

### Security
- Sender allowlist via `allowed_senders` config
- Group monitoring via `monitored_groups` config

## Configuration

```toml
[[channels]]
type = "signal"
account_id = "signal-main"
enabled = true

[channels.credentials]
phone_number = "+1234567890"
device_name = "aisopod-bot"
disappearing_enabled = true
disappearing_timer = 2592000  # 30 days
include_media = true

[channels.config]
allowed_senders = ["+1234567890", "+0987654321"]
monitored_groups = ["group-id-1", "group-id-2"]
```

## Testing

Unit tests cover:
- Phone number validation
- Sender allowlist filtering
- Phone number normalization
- Message parsing (DMs and groups)
- Media content handling
- Disappearing message timer extraction

Run tests with:
```bash
cargo test --package aisopod-channel-signal
```

## Integration Points

### aisopod-channel-core
- Implements `ChannelPlugin` trait
- Uses `IncomingMessage` and `MessageTarget` types
- Integrates with `ChannelRegistry`

### aisopod-gateway
- Channel configuration loaded from config
- Message routing via channel ID
- Account lifecycle management

## Known Limitations

1. **Subprocess Overhead**: Each send/receive operation spawns a subprocess. This could be optimized with persistent daemon connections.

2. **Signal-cli Dependency**: Requires signal-cli to be installed and configured separately.

3. **Media Download**: Media files are downloaded to temporary storage; in-memory handling could be more efficient.

## Future Enhancements

- WebSocket-based communication with signal-cli daemon
- Message queue for outgoing messages
- Improved error handling and retry logic
- Group member discovery
- Message editing and deletion
- Read receipts and typing indicators

## Debugging

Enable tracing for detailed logs:
```bash
RUST_LOG=trace ./aisopod
```

## References

- Signal CLI: https://github.com/AsamK/signal-cli
- aisopod Channel Plugin: `crates/aisopod-channel/src/plugin.rs`
- Similar implementations: `aisopod-channel-discord`, `aisopod-channel-whatsapp`
