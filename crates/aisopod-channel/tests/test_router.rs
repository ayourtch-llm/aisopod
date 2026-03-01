//! Tests for the MessageRouter.

use std::sync::Arc;

use aisopod_channel::adapters::{AccountSnapshot, ChannelConfigAdapter, SecurityAdapter};
use aisopod_channel::message::{IncomingMessage, PeerInfo, PeerKind, SenderInfo};
use aisopod_channel::{AgentResolver, ChannelRegistry, MessageRouter};
use aisopod_config::AisopodConfig;
use aisopod_session::routing::resolve_session_key;
use aisopod_session::SessionKey;
use aisopod_tools::SessionManager;
use anyhow::Result;
use async_trait::async_trait;

// ============================================================================
// Mock Agent Resolver
// ============================================================================

struct MockAgentResolver {
    agent_id: String,
}

impl MockAgentResolver {
    fn new(agent_id: &str) -> Self {
        Self {
            agent_id: agent_id.to_string(),
        }
    }
}

impl AgentResolver for MockAgentResolver {
    fn resolve(&self, _session_key: &SessionKey) -> Result<String> {
        Ok(self.agent_id.clone())
    }
}

struct MockSessionManager;

#[async_trait]
impl SessionManager for MockSessionManager {
    async fn list_sessions(
        &self,
        _limit: Option<usize>,
    ) -> Result<Vec<aisopod_tools::builtins::session::SessionInfo>> {
        Ok(Vec::new())
    }

    async fn send_to_session(&self, _session_id: &str, _message: &str) -> Result<()> {
        Ok(())
    }

    async fn patch_metadata(&self, _session_id: &str, _metadata: serde_json::Value) -> Result<()> {
        Ok(())
    }

    async fn get_history(
        &self,
        _session_id: &str,
        _limit: Option<usize>,
    ) -> Result<Vec<serde_json::Value>> {
        Ok(Vec::new())
    }
}

// ============================================================================
// Mock Channel Plugin
// ============================================================================

struct MockChannelPlugin {
    id: String,
    security_adapter: Option<Arc<dyn SecurityAdapter>>,
}

impl MockChannelPlugin {
    fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            security_adapter: None,
        }
    }

    fn with_config_adapter(self, _adapter: Arc<dyn ChannelConfigAdapter>) -> Self {
        self
    }

    fn with_security_adapter(mut self, adapter: Arc<dyn SecurityAdapter>) -> Self {
        self.security_adapter = Some(adapter);
        self
    }
}

impl aisopod_channel::ChannelPlugin for MockChannelPlugin {
    fn id(&self) -> &str {
        &self.id
    }

    fn meta(&self) -> &aisopod_channel::ChannelMeta {
        static META: std::sync::OnceLock<aisopod_channel::ChannelMeta> = std::sync::OnceLock::new();
        META.get_or_init(|| aisopod_channel::ChannelMeta {
            label: "Mock Channel".to_string(),
            docs_url: None,
            ui_hints: serde_json::Value::Object(serde_json::Map::new()),
        })
    }

    fn capabilities(&self) -> &aisopod_channel::ChannelCapabilities {
        static CAPS: std::sync::OnceLock<aisopod_channel::ChannelCapabilities> =
            std::sync::OnceLock::new();
        CAPS.get_or_init(|| aisopod_channel::ChannelCapabilities {
            chat_types: vec![],
            supports_media: false,
            supports_reactions: false,
            supports_threads: false,
            supports_typing: false,
            supports_voice: false,
            max_message_length: None,
            supported_media_types: vec![],
        })
    }

    fn config(&self) -> &dyn ChannelConfigAdapter {
        unimplemented!("Mock for testing purposes")
    }

    fn security(&self) -> Option<&dyn SecurityAdapter> {
        self.security_adapter.as_ref().map(|a| a.as_ref())
    }
}

// ============================================================================
// Mock Security Adapter
// ============================================================================

struct MockSecurityAdapter {
    allowed_senders: Vec<String>,
    requires_mention: bool,
}

impl MockSecurityAdapter {
    fn new() -> Self {
        Self {
            allowed_senders: Vec::new(),
            requires_mention: false,
        }
    }

    fn add_allowed_sender(mut self, sender_id: &str) -> Self {
        self.allowed_senders.push(sender_id.to_string());
        self
    }

    fn with_mention_required(mut self, required: bool) -> Self {
        self.requires_mention = required;
        self
    }
}

impl SecurityAdapter for MockSecurityAdapter {
    fn is_allowed_sender(&self, sender: &SenderInfo) -> bool {
        self.allowed_senders.contains(&sender.id)
    }

    fn requires_mention_in_group(&self) -> bool {
        self.requires_mention
    }
}

// ============================================================================
// Mock Channel Config Adapter
// ============================================================================

struct TestChannelConfigAdapter {
    accounts: std::collections::HashMap<String, AccountSnapshot>,
}

impl TestChannelConfigAdapter {
    fn new_enabled(account_id: &str) -> Self {
        let mut accounts = std::collections::HashMap::new();
        accounts.insert(
            account_id.to_string(),
            AccountSnapshot {
                id: account_id.to_string(),
                channel: "telegram".to_string(),
                enabled: true,
                connected: true,
            },
        );
        Self { accounts }
    }

    fn new_disabled(account_id: &str) -> Self {
        let mut accounts = std::collections::HashMap::new();
        accounts.insert(
            account_id.to_string(),
            AccountSnapshot {
                id: account_id.to_string(),
                channel: "telegram".to_string(),
                enabled: false,
                connected: false,
            },
        );
        Self { accounts }
    }
}

impl ChannelConfigAdapter for TestChannelConfigAdapter {
    fn list_accounts(&self) -> Result<Vec<String>, anyhow::Error> {
        Ok(self.accounts.keys().cloned().collect())
    }

    fn resolve_account(&self, id: &str) -> Result<AccountSnapshot, anyhow::Error> {
        self.accounts
            .get(id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Account not found: {}", id))
    }

    fn enable_account(&self, _id: &str) -> Result<(), anyhow::Error> {
        Ok(())
    }

    fn disable_account(&self, _id: &str) -> Result<(), anyhow::Error> {
        Ok(())
    }

    fn delete_account(&self, _id: &str) -> Result<(), anyhow::Error> {
        Ok(())
    }
}

// ============================================================================
// Test Helper Functions
// ============================================================================

fn create_test_message(channel: &str, account_id: &str, sender_id: &str) -> IncomingMessage {
    IncomingMessage {
        id: "msg1".to_string(),
        channel: channel.to_string(),
        account_id: account_id.to_string(),
        sender: SenderInfo {
            id: sender_id.to_string(),
            display_name: Some("Test User".to_string()),
            username: Some("testuser".to_string()),
            is_bot: false,
        },
        peer: PeerInfo {
            id: "peer1".to_string(),
            kind: PeerKind::User,
            title: Some("Test".to_string()),
        },
        content: aisopod_channel::message::MessageContent::Text("Hello".to_string()),
        reply_to: None,
        timestamp: chrono::Utc::now(),
        metadata: serde_json::Value::Object(serde_json::Map::new()),
    }
}

// ============================================================================
// Mock Channel Factory
// ============================================================================

fn create_mock_channel(
    channel_id: &str,
    account_id: &str,
    enabled: bool,
    allowed_sender: Option<&str>,
) -> Arc<dyn aisopod_channel::ChannelPlugin> {
    let config_adapter = if enabled {
        Arc::new(TestChannelConfigAdapter::new_enabled(account_id))
    } else {
        Arc::new(TestChannelConfigAdapter::new_disabled(account_id))
    };

    let security_adapter = if let Some(sender_id) = allowed_sender {
        let adapter = MockSecurityAdapter::new()
            .add_allowed_sender(sender_id)
            .with_mention_required(false);
        Some(Arc::new(adapter))
    } else {
        None
    };

    let mut channel = MockChannelPlugin::new(channel_id).with_config_adapter(config_adapter);

    if let Some(adapter) = security_adapter {
        channel = channel.with_security_adapter(adapter);
    }

    Arc::new(channel)
}

// ============================================================================
// Router Tests
// ============================================================================

#[test]
fn test_router_debug() {
    let registry = Arc::new(ChannelRegistry::new());
    let agent_resolver = Arc::new(MockAgentResolver::new("default"));
    let session_manager = Arc::new(MockSessionManager);

    let router = MessageRouter::new(registry, agent_resolver, session_manager);

    // Just verify it compiles and can be created
    let _debug_str = format!("{:?}", router);
}

// ============================================================================
// Session Key Tests
// ============================================================================

#[test]
fn test_session_key_creation() {
    let key = SessionKey {
        agent_id: "agent_001".to_string(),
        channel: "discord".to_string(),
        account_id: "bot_123".to_string(),
        peer_kind: "dm".to_string(),
        peer_id: "user_456".to_string(),
    };

    assert_eq!(key.agent_id, "agent_001");
    assert_eq!(key.channel, "discord");
    assert_eq!(key.account_id, "bot_123");
    assert_eq!(key.peer_kind, "dm");
    assert_eq!(key.peer_id, "user_456");
}

#[test]
fn test_resolve_session_key() {
    let agent_id = "test_agent";
    let ctx = aisopod_session::ChannelContext {
        channel: "telegram".to_string(),
        account_id: "bot1".to_string(),
        peer_kind: aisopod_session::PeerKind::Dm,
        peer_id: "user1".to_string(),
    };

    let key = resolve_session_key(agent_id, &ctx);
    assert_eq!(key.agent_id, "test_agent");
    assert_eq!(key.channel, "telegram");
    assert_eq!(key.account_id, "bot1");
    assert_eq!(key.peer_id, "user1");
}

// ============================================================================
// Registry Integration Tests
// ============================================================================

#[test]
fn test_registry_with_mock_channel() {
    let mut registry = ChannelRegistry::new();

    let channel = create_mock_channel("telegram", "bot1", true, Some("user123"));
    registry.register(channel);

    // Test get
    let retrieved = registry.get("telegram");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id(), "telegram");

    // Test list
    let list = registry.list();
    assert!(list.contains(&"telegram".to_string()));

    // Test normalize (case-sensitive)
    let normalized = registry.normalize_id("telegram");
    assert_eq!(normalized, Some("telegram".to_string()));

    // Test normalize with wrong case (should return None)
    let normalized_wrong = registry.normalize_id("Telegram");
    assert_eq!(normalized_wrong, None);
}

#[test]
fn test_registry_with_disabled_account() {
    let mut registry = ChannelRegistry::new();

    let channel = create_mock_channel("telegram", "bot1", false, Some("user123"));
    registry.register(channel);

    let retrieved = registry.get("telegram");
    assert!(retrieved.is_some());
}

#[test]
fn test_registry_with_security_adapter() {
    let mut registry = ChannelRegistry::new();

    // Create channel with security adapter that allows specific sender
    let channel = create_mock_channel("telegram", "bot1", true, Some("allowed_user"));
    registry.register(channel);

    let retrieved = registry.get("telegram");
    assert!(retrieved.is_some());
}

#[test]
fn test_registry_add_alias() {
    let mut registry = ChannelRegistry::new();

    let channel = create_mock_channel("telegram", "bot1", true, None);
    registry.register(channel);

    registry.add_alias("tg", "telegram");

    // Test alias resolution
    let channel_by_alias = registry.get("tg");
    assert!(channel_by_alias.is_some());
    assert_eq!(channel_by_alias.unwrap().id(), "telegram");
}

#[test]
fn test_registry_contains() {
    let mut registry = ChannelRegistry::new();

    let channel = create_mock_channel("telegram", "bot1", true, None);
    registry.register(channel);
    registry.add_alias("tg", "telegram");

    assert!(registry.contains("telegram"));
    assert!(registry.contains("tg"));
    assert!(!registry.contains("discord"));
}

#[test]
fn test_registry_remove_alias() {
    let mut registry = ChannelRegistry::new();

    let channel = create_mock_channel("telegram", "bot1", true, None);
    registry.register(channel);
    registry.add_alias("tg", "telegram");

    assert!(registry.contains("tg"));

    registry.remove_alias("tg");

    assert!(!registry.contains("tg"));
}
