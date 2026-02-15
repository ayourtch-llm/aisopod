# 0009 — Channel Abstraction & Messaging

**Master Plan Reference:** Section 3.7 — Channel Abstraction & Messaging  
**Phase:** 4 (Messaging)  
**Dependencies:** 0001 (Project Structure), 0002 (Configuration), 0003 (Gateway Server)

---

## Objective

Implement the channel abstraction layer that defines how messaging platforms
integrate with aisopod, including the plugin interface, message routing,
and common messaging features.

---

## Deliverables

### 1. Channel Trait (`aisopod-channel`)

Define the core channel abstraction:

```rust
#[async_trait]
pub trait ChannelPlugin: Send + Sync {
    /// Channel identifier (e.g., "telegram", "discord")
    fn id(&self) -> &str;

    /// Channel metadata (label, docs, capabilities)
    fn meta(&self) -> &ChannelMeta;

    /// Channel capabilities
    fn capabilities(&self) -> &ChannelCapabilities;

    /// Account management
    fn config(&self) -> &dyn ChannelConfigAdapter;
}
```

### 2. Adapter Interfaces

Port the 13 optional adapter interfaces as traits:

```rust
/// CLI onboarding wizard
#[async_trait]
pub trait OnboardingAdapter: Send + Sync {
    async fn setup_wizard(&self, ctx: &OnboardingContext) -> Result<AccountConfig>;
}

/// Message delivery
#[async_trait]
pub trait OutboundAdapter: Send + Sync {
    async fn send_text(&self, target: &MessageTarget, text: &str) -> Result<()>;
    async fn send_media(&self, target: &MessageTarget, media: &Media) -> Result<()>;
}

/// WebSocket/polling gateway connection
#[async_trait]
pub trait GatewayAdapter: Send + Sync {
    async fn connect(&self, account: &AccountConfig) -> Result<()>;
    async fn disconnect(&self, account: &AccountConfig) -> Result<()>;
    fn is_connected(&self, account: &AccountConfig) -> bool;
}

/// Health monitoring
#[async_trait]
pub trait StatusAdapter: Send + Sync {
    async fn health_check(&self, account: &AccountConfig) -> Result<ChannelHealth>;
}

/// Typing indicators
#[async_trait]
pub trait TypingAdapter: Send + Sync {
    async fn send_typing(&self, target: &MessageTarget) -> Result<()>;
}

/// Message reactions
#[async_trait]
pub trait MessagingAdapter: Send + Sync {
    async fn react(&self, message_id: &str, emoji: &str) -> Result<()>;
    async fn unreact(&self, message_id: &str, emoji: &str) -> Result<()>;
}

/// Thread/reply support
#[async_trait]
pub trait ThreadingAdapter: Send + Sync {
    async fn create_thread(&self, parent_id: &str, title: &str) -> Result<String>;
    async fn reply_in_thread(&self, thread_id: &str, text: &str) -> Result<()>;
}

/// Group/user discovery
#[async_trait]
pub trait DirectoryAdapter: Send + Sync {
    async fn list_groups(&self, account: &AccountConfig) -> Result<Vec<GroupInfo>>;
    async fn list_members(&self, group_id: &str) -> Result<Vec<MemberInfo>>;
}

/// Security and DM policies
#[async_trait]
pub trait SecurityAdapter: Send + Sync {
    fn is_allowed_sender(&self, sender: &SenderInfo) -> bool;
    fn requires_mention_in_group(&self) -> bool;
}

/// Keep-alive mechanism
#[async_trait]
pub trait HeartbeatAdapter: Send + Sync {
    async fn heartbeat(&self, account: &AccountConfig) -> Result<()>;
    fn heartbeat_interval(&self) -> Duration;
}

/// Account management
#[async_trait]
pub trait ChannelConfigAdapter: Send + Sync {
    async fn list_accounts(&self) -> Result<Vec<String>>;
    async fn resolve_account(&self, id: &str) -> Result<AccountSnapshot>;
    async fn enable_account(&self, id: &str) -> Result<()>;
    async fn disable_account(&self, id: &str) -> Result<()>;
    async fn delete_account(&self, id: &str) -> Result<()>;
}

/// Token/credential management
#[async_trait]
pub trait AuthAdapter: Send + Sync {
    async fn authenticate(&self, config: &AccountConfig) -> Result<AuthToken>;
    async fn refresh_token(&self, token: &AuthToken) -> Result<AuthToken>;
}

/// Device pairing
#[async_trait]
pub trait PairingAdapter: Send + Sync {
    async fn initiate_pairing(&self) -> Result<PairingCode>;
    async fn complete_pairing(&self, code: &str) -> Result<AccountConfig>;
}
```

### 3. Channel Registry

Port the channel registration and discovery system:

```rust
pub struct ChannelRegistry {
    channels: HashMap<String, Arc<dyn ChannelPlugin>>,
    order: Vec<String>,
    aliases: HashMap<String, String>,
}

impl ChannelRegistry {
    pub fn register(&mut self, plugin: Arc<dyn ChannelPlugin>);
    pub fn get(&self, id: &str) -> Option<&Arc<dyn ChannelPlugin>>;
    pub fn list(&self) -> Vec<&Arc<dyn ChannelPlugin>>;
    pub fn normalize_id(&self, id: &str) -> Option<String>;
}
```

### 4. Channel Capabilities

```rust
pub struct ChannelCapabilities {
    pub chat_types: Vec<ChatType>,      // dm, group, channel, thread
    pub supports_media: bool,
    pub supports_reactions: bool,
    pub supports_threads: bool,
    pub supports_typing: bool,
    pub supports_voice: bool,
    pub max_message_length: Option<usize>,
    pub supported_media_types: Vec<MediaType>,
}
```

### 5. Message Types

```rust
pub struct IncomingMessage {
    pub id: String,
    pub channel: String,
    pub account_id: String,
    pub sender: SenderInfo,
    pub peer: PeerInfo,
    pub content: MessageContent,
    pub reply_to: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

pub struct OutgoingMessage {
    pub target: MessageTarget,
    pub content: MessageContent,
    pub reply_to: Option<String>,
}

pub enum MessageContent {
    Text(String),
    Media(Media),
    Mixed(Vec<MessagePart>),
}

pub struct MessageTarget {
    pub channel: String,
    pub account_id: String,
    pub peer: PeerInfo,
    pub thread_id: Option<String>,
}
```

### 6. Message Routing

Implement the message routing pipeline:

```
Incoming message from channel
  → Normalize channel ID
  → Resolve account
  → Build session key
  → Check security/allowlist
  → Check mention requirement (groups)
  → Resolve agent via bindings
  → Create/load session
  → Pass to agent runner
  → Stream response back
  → Deliver via outbound adapter
```

### 7. Media Handling

- Image processing (resize, format conversion) via `image` crate
- Audio transcription integration
- Document handling (PDF text extraction)
- Media upload/download for each channel
- Media type detection and validation

---

## Acceptance Criteria

- [ ] Channel trait is well-defined with all adapter interfaces
- [ ] Channel registry supports registration, lookup, and listing
- [ ] Message types handle text, media, and mixed content
- [ ] Message routing resolves channels → agents correctly
- [ ] Security adapter enforces allowlists and mention requirements
- [ ] Media handling processes images, audio, and documents
- [ ] Channel capabilities accurately describe each channel's features
- [ ] Unit tests verify routing logic and security enforcement
- [ ] Integration tests verify message flow end-to-end
