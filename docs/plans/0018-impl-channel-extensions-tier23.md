# 0018 — Channel Extensions (Tier 2 & 3)

**Master Plan Reference:** Section 3.8 — Channel Extensions (Tier 2 & 3)  
**Phase:** 7 (Production)  
**Dependencies:** 0009 (Channel Abstraction), 0010 (Tier 1 Channels)

---

## Objective

Implement remaining channel integrations beyond the Tier 1 channels,
covering Tier 2 (commonly requested) and Tier 3 (niche/specialized).

---

## Deliverables

### 1. Tier 2 Channels

#### Signal (`aisopod-channel-signal`)
- Integration via Signal CLI or libsignal
- DM and group messaging
- Media support (images, documents)
- Disappearing messages awareness
- Phone number-based identity

#### iMessage (`aisopod-channel-imessage`)
- macOS-only (uses AppleScript / Shortcuts / BlueBubbles bridge)
- DM and group messaging
- Media support
- Platform detection (macOS required)
- BlueBubbles API as alternative

#### Google Chat (`aisopod-channel-googlechat`)
- Google Chat API / webhook integration
- Space and DM messaging
- Card-based rich messages
- OAuth 2.0 authentication
- Service account support

#### Microsoft Teams (`aisopod-channel-msteams`)
- Microsoft Bot Framework integration
- Channel and DM messaging
- Adaptive Cards for rich content
- Azure AD authentication
- Webhook/connector support

### 2. Tier 3 Channels

#### Matrix (`aisopod-channel-matrix`)
- Matrix client-server API
- Room and DM messaging
- End-to-end encryption (optional)
- Homeserver configuration
- SSO/token authentication

#### IRC (`aisopod-channel-irc`)
- IRC protocol client
- Channel and DM messaging
- Multiple server support
- NickServ authentication
- TLS support

#### Mattermost (`aisopod-channel-mattermost`)
- Mattermost REST API
- Channel and DM messaging
- WebSocket event streaming
- Bot account support
- Self-hosted server configuration

#### Nextcloud Talk (`aisopod-channel-nextcloud`)
- Nextcloud Talk API
- Room-based messaging
- File sharing integration
- Nextcloud authentication

#### Twitch (`aisopod-channel-twitch`)
- Twitch IRC (TMI) integration
- Chat channel messaging
- Mod/subscriber status
- OAuth authentication
- Whisper support (DMs)

#### Nostr (`aisopod-channel-nostr`)
- Nostr protocol (NIP-01, NIP-04)
- Relay connection management
- Encrypted DMs
- Public channel posting
- Key management

#### LINE (`aisopod-channel-line`)
- LINE Messaging API
- User and group messaging
- Rich message types (Flex Messages)
- Webhook handling
- Channel access token management

#### Lark/Feishu (`aisopod-channel-lark`)
- Lark Open Platform API
- Group and DM messaging
- Rich message cards
- Event subscription
- App credentials management

#### Zalo (`aisopod-channel-zalo`)
- Zalo Official Account API
- User messaging
- Media support
- OAuth authentication

### 3. Channel Extension Template

Provide a template for community channel implementations:

```
aisopod-channel-template/
├── Cargo.toml
├── src/
│   ├── lib.rs          # Plugin registration
│   ├── channel.rs      # ChannelPlugin implementation
│   ├── config.rs       # Account configuration
│   ├── outbound.rs     # Message sending
│   ├── gateway.rs      # Connection management
│   └── runtime.rs      # Channel-specific state
└── README.md           # Setup guide
```

### 4. Shared Channel Utilities

- Rate limit handling per platform
- Markdown format conversion between platforms
- Media format conversion
- Connection retry with exponential backoff
- Platform-specific error mapping

---

## Acceptance Criteria

### Tier 2
- [ ] Signal channel sends/receives DMs and group messages
- [ ] iMessage channel works on macOS (with fallback note for other platforms)
- [ ] Google Chat channel handles spaces and DMs
- [ ] Teams channel handles channels and DMs with Adaptive Cards

### Tier 3
- [ ] Each Tier 3 channel has basic send/receive functionality
- [ ] Channel template generates valid scaffolding
- [ ] Shared utilities handle common cross-channel operations

### General
- [ ] All channels implement the ChannelPlugin trait correctly
- [ ] Integration tests verify connectivity for each channel
- [ ] Documentation covers setup for each channel
- [ ] Error handling provides clear diagnostics per platform
