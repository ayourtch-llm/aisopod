# Channel Abstraction

**Crate:** `aisopod-channel`

## Overview

The channel subsystem defines how messaging platforms (Telegram, Discord, WhatsApp,
Slack, Signal, etc.) integrate with aisopod. It provides a plugin trait with 13
optional adapter interfaces, a central registry, message types, and a routing
pipeline that connects incoming messages to the correct agent.

## Key Types

- **`ChannelPlugin` trait** — Core interface: `id()`, `meta()`, `capabilities()`,
  `config()`. Each messaging platform implements this trait.
- **`ChannelRegistry`** — Central registration and lookup with ordering, aliases,
  and ID normalization.
- **`ChannelCapabilities`** — Feature matrix: chat types (dm/group/channel/thread),
  media support, reactions, threads, typing, voice, max message length.
- **`IncomingMessage`** / **`OutgoingMessage`** — Normalized message types with
  channel, account, sender/target, content, reply context, and metadata.
- **`MessageContent`** — Enum: `Text`, `Media`, `Mixed(Vec<MessagePart>)`.
- **`MessageTarget`** — Delivery address: channel, account_id, peer, thread_id.

## Adapter Interfaces (13 traits)

| Adapter            | Purpose                                |
|--------------------|----------------------------------------|
| `OnboardingAdapter`  | CLI setup wizard for new accounts    |
| `PairingAdapter`     | Device/security pairing              |
| `OutboundAdapter`    | Send text and media messages         |
| `StatusAdapter`      | Health monitoring                    |
| `DirectoryAdapter`   | Group/user discovery                 |
| `SecurityAdapter`    | DM policies, allowlists, elevation   |
| `GatewayAdapter`     | WebSocket/polling connections        |
| `ThreadingAdapter`   | Thread/reply support                 |
| `MessagingAdapter`   | Reactions and message actions        |
| `HeartbeatAdapter`   | Keep-alive mechanisms                |
| `TypingAdapter`      | Typing indicators                    |
| `AuthAdapter`        | Token/credential management          |
| `ChannelConfigAdapter` | Account CRUD operations            |

Channel implementations adopt only the adapters relevant to their platform.

## Message Routing Pipeline

1. Receive incoming message from channel adapter
2. Normalize channel ID via registry
3. Resolve account and build `SessionKey`
4. Check security/allowlist via `SecurityAdapter`
5. Check mention requirement for group messages
6. Resolve agent via bindings
7. Create or load session
8. Pass to agent engine
9. Stream response back and deliver via `OutboundAdapter`

## Media Handling

Images are processed (resize, format conversion) via the `image` crate. Audio
transcription and document text extraction integrate through provider APIs.
Each channel adapter handles platform-specific upload/download mechanics.

## Dependencies

- **aisopod-config** — `ChannelsConfig`, per-channel account settings.
- **aisopod-session** — Session creation during message routing.
- **aisopod-agent** — Agent execution for routed messages.

## Design Decisions

- **Optional adapter traits over a monolithic interface:** Channels only implement
  what they support, avoiding stub methods and making capability introspection
  explicit via `ChannelCapabilities`.
- **Registry with aliases:** Handles common name variations (e.g., "wa" → "whatsapp")
  for a friendlier CLI and config experience.
- **Normalized message types:** `IncomingMessage`/`OutgoingMessage` decouple the
  agent engine from channel-specific wire formats, keeping routing logic uniform.
