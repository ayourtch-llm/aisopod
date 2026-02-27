# aisopod-channel-test-channel

Test Channel channel plugin for aisopod.

## Overview

This crate provides a Test Channel channel implementation for aisopod, enabling the bot to
receive and send messages via the Test Channel platform.

## Features

- Incoming message reception
- Outgoing message sending
- Connection management
- Error handling

## Setup

1. Add this crate to your aisopod workspace members in `Cargo.toml`:
   ```toml
   members = [
       # ...
       "crates/aisopod-channel-test-channel",
   ]
   ```

2. Enable the plugin in `aisopod-plugin`:
   ```toml
   [features]
   plugin-test-channel = ["dep:aisopod-channel-test-channel"]
   
   [dependencies]
   aisopod-channel-test-channel = { path = "../aisopod-channel-test-channel", optional = true }
   ```

3. Build: `cargo build -p aisopod-channel-test-channel`

## Configuration

Add the following to your aisopod config file:

```toml
[[channels]]
id = "test-channel"
name = "Test Channel"
channel_type = "test-channel"
connection = { endpoint = "", token = "" }
```

## Development

### Prerequisites

- Rust 1.70 or later
- Cargo (comes with Rust)

### Building

```bash
cargo build -p aisopod-channel-test-channel
```

### Running Tests

```bash
cargo test -p aisopod-channel-test-channel
```

## Structure

```
aisopod-channel-test-channel/
├── src/
│   ├── lib.rs        # Main entry point and exports
│   ├── channel.rs    # ChannelPlugin implementation
│   ├── config.rs     # Configuration types
│   ├── outbound.rs   # Outbound message handling
│   ├── gateway.rs    # Gateway adapter
│   └── runtime.rs    # Runtime utilities
├── Cargo.toml        # Package manifest
└── README.md         # This file
```

## Next Steps

1. Edit `src/config.rs` to add your configuration fields
2. Implement `connect()` and `disconnect()` in `src/channel.rs`
3. Implement `send()` and `receive()` in `src/channel.rs`
4. Update `src/outbound.rs` to format messages for Test Channel
5. Run `cargo build` to verify compilation

## Documentation

For more information on channel development, see the [Channel Development Guide](../../docs/channel-development.md).

## License

MIT
