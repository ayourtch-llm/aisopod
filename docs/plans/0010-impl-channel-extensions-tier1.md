# 0010 — Channel Extensions (Tier 1)

**Master Plan Reference:** Section 3.8 — Channel Extensions  
**Phase:** 4 (Messaging)  
**Dependencies:** 0009 (Channel Abstraction)

---

## Objective

Implement the four most commonly used channel integrations (Tier 1):
Telegram, Discord, WhatsApp, and Slack.

---

## Deliverables

### 1. Telegram Channel (`aisopod-channel-telegram`)

**Rust library:** `teloxide` or `grammyrs`

**Features to port:**
- Bot token authentication
- Receive messages (DM, group, supergroup)
- Send text messages with Markdown formatting
- Send/receive media (photos, documents, audio, video)
- Typing indicators
- Reply to specific messages
- Group mention detection (`@botname`)
- Inline keyboards (for interactive responses)
- Webhook or long-polling mode
- Multiple bot account support
- Message editing and deletion

**Account configuration:**
```rust
pub struct TelegramAccountConfig {
    pub bot_token: String,
    pub webhook_url: Option<String>,
    pub allowed_users: Option<Vec<i64>>,
    pub allowed_groups: Option<Vec<i64>>,
    pub parse_mode: ParseMode,
}
```

### 2. Discord Channel (`aisopod-channel-discord`)

**Rust library:** `serenity` or `twilight`

**Features to port:**
- Bot token authentication
- Receive messages (DM, server channel, thread)
- Send text messages with Discord markdown
- Send/receive media (attachments, embeds)
- Typing indicators
- Reply to specific messages
- Thread creation and management
- Reaction handling
- Slash command support (optional)
- Guild/channel discovery
- Multiple bot account support
- Message editing and deletion
- Embed support for rich responses

**Account configuration:**
```rust
pub struct DiscordAccountConfig {
    pub bot_token: String,
    pub application_id: Option<String>,
    pub allowed_guilds: Option<Vec<u64>>,
    pub allowed_channels: Option<Vec<u64>>,
    pub mention_required_in_channels: bool,
}
```

### 3. WhatsApp Channel (`aisopod-channel-whatsapp`)

**Challenge:** The OpenClaw project uses Baileys (Node.js WhatsApp Web library).
There is no mature Rust equivalent.

**Options:**
1. **Bridge approach:** Run a small Node.js/Python subprocess for WhatsApp Web protocol
2. **WhatsApp Business API:** Use the official REST API (requires business account)
3. **Rust reimplementation:** Port Baileys protocol (complex, not recommended initially)

**Recommended: Option 2 (WhatsApp Business API) for initial release, with Option 1 as fallback**

**Features to port:**
- QR code pairing (Baileys mode) or API key (Business API)
- Receive messages (DM, group)
- Send text messages
- Send/receive media (photos, documents, audio, video, stickers)
- Typing indicators ("recording" state)
- Reply to specific messages
- Read receipts
- Contact/group management
- Message status tracking

**Account configuration:**
```rust
pub struct WhatsAppAccountConfig {
    pub mode: WhatsAppMode, // "baileys_bridge" or "business_api"
    // Baileys bridge
    pub bridge_endpoint: Option<String>,
    // Business API
    pub api_token: Option<String>,
    pub phone_number_id: Option<String>,
    pub business_account_id: Option<String>,
    pub webhook_verify_token: Option<String>,
    // Common
    pub allowed_numbers: Option<Vec<String>>,
}
```

### 4. Slack Channel (`aisopod-channel-slack`)

**Rust library:** Custom HTTP client (Slack has a REST API)

**Features to port:**
- Bot token + app token authentication (Socket Mode)
- Receive messages (DM, channel, thread)
- Send text messages with Slack mrkdwn formatting
- Send/receive media (files)
- Typing indicators
- Thread management (reply in thread)
- Reaction handling
- Block Kit support for rich messages
- Slash command handling
- Channel/user discovery
- Multiple workspace support
- Message editing and deletion
- Interactive components (buttons, menus)

**Account configuration:**
```rust
pub struct SlackAccountConfig {
    pub bot_token: String,
    pub app_token: Option<String>,     // For Socket Mode
    pub signing_secret: Option<String>, // For webhook verification
    pub allowed_channels: Option<Vec<String>>,
    pub mention_required: bool,
}
```

### 5. Shared Channel Utilities

- Message formatting normalization (each platform's markdown variant)
- Media transcoding for cross-platform compatibility
- Rate limit handling per platform
- Connection state management and reconnection logic
- Error mapping to common error types

---

## Acceptance Criteria

### Telegram
- [ ] Bot connects and receives messages
- [ ] Text and media messages are sent and received
- [ ] Group mention detection works
- [ ] Typing indicators display
- [ ] Multiple accounts supported

### Discord
- [ ] Bot connects to gateway and receives messages
- [ ] Text and media messages are sent and received
- [ ] Thread management works
- [ ] Reactions can be added/removed
- [ ] Guild and channel discovery works

### WhatsApp
- [ ] Connection established (bridge or Business API)
- [ ] Text and media messages are sent and received
- [ ] DM and group messaging works
- [ ] Read receipts function
- [ ] Allowed number filtering works

### Slack
- [ ] Bot connects via Socket Mode or webhooks
- [ ] Text and media messages are sent and received
- [ ] Thread management works
- [ ] Reactions can be added/removed
- [ ] Block Kit messages render correctly

### All channels
- [ ] Integration tests verify message send/receive cycle
- [ ] Error handling provides clear diagnostics
- [ ] Reconnection logic handles network disruptions
- [ ] Rate limiting is respected per platform
