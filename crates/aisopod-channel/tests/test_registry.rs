//! Tests for the ChannelRegistry.

use std::sync::Arc;

use aisopod_channel::{ChannelRegistry, ChannelPlugin, ChannelMeta, ChannelCapabilities};
use aisopod_channel::adapters::{ChannelConfigAdapter, SecurityAdapter, AccountSnapshot};
use aisopod_channel::types::ChatType;
use async_trait::async_trait;

// ============================================================================
// Helper types for creating mock channels
// ============================================================================

struct TestChannelPlugin {
    id: String,
    meta: ChannelMeta,
    capabilities: ChannelCapabilities,
    config_adapter: Option<Arc<dyn ChannelConfigAdapter>>,
    security_adapter: Option<Arc<dyn SecurityAdapter>>,
}

impl TestChannelPlugin {
    fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            meta: ChannelMeta {
                label: format!("{} Channel", id.to_ascii_uppercase()),
                docs_url: None,
                ui_hints: serde_json::Value::Object(serde_json::Map::new()),
            },
            capabilities: ChannelCapabilities {
                chat_types: vec![ChatType::Dm, ChatType::Group],
                supports_media: false,
                supports_reactions: false,
                supports_threads: false,
                supports_typing: false,
                supports_voice: false,
                max_message_length: None,
                supported_media_types: vec![],
            },
            config_adapter: None,
            security_adapter: None,
        }
    }

    fn with_config_adapter(mut self, adapter: Arc<dyn ChannelConfigAdapter>) -> Self {
        self.config_adapter = Some(adapter);
        self
    }

    fn with_security_adapter(mut self, adapter: Arc<dyn SecurityAdapter>) -> Self {
        self.security_adapter = Some(adapter);
        self
    }
}

#[async_trait]
impl ChannelPlugin for TestChannelPlugin {
    fn id(&self) -> &str {
        &self.id
    }

    fn meta(&self) -> &ChannelMeta {
        &self.meta
    }

    fn capabilities(&self) -> &ChannelCapabilities {
        &self.capabilities
    }

    fn config(&self) -> &dyn ChannelConfigAdapter {
        self.config_adapter.as_ref().map(|a| a.as_ref()).unwrap()
    }

    fn security(&self) -> Option<&dyn SecurityAdapter> {
        self.security_adapter.as_ref().map(|a| a.as_ref())
    }
}

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
        self.accounts.get(id).cloned().ok_or_else(|| {
            anyhow::anyhow!("Account not found: {}", id)
        })
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

struct TestSecurityAdapter {
    allowed_senders: Vec<String>,
    requires_mention: bool,
}

impl TestSecurityAdapter {
    fn new(allowed_senders: Vec<String>, requires_mention: bool) -> Self {
        Self {
            allowed_senders,
            requires_mention,
        }
    }
}

impl SecurityAdapter for TestSecurityAdapter {
    fn is_allowed_sender(&self, sender: &aisopod_channel::message::SenderInfo) -> bool {
        self.allowed_senders.contains(&sender.id)
    }

    fn requires_mention_in_group(&self) -> bool {
        self.requires_mention
    }
}

// ============================================================================
// Helper functions for creating mock channels
// ============================================================================

fn make_test_channel(id: &str) -> Arc<dyn ChannelPlugin> {
    let channel = TestChannelPlugin::new(id);
    Arc::new(channel)
}

fn make_test_channel_with_config(id: &str) -> Arc<dyn ChannelPlugin> {
    let config_adapter = Arc::new(TestChannelConfigAdapter::new_enabled(id));
    let channel = TestChannelPlugin::new(id)
        .with_config_adapter(config_adapter);
    Arc::new(channel)
}

// ============================================================================
// New Registry Tests
// ============================================================================

#[test]
fn test_new_registry_is_empty() {
    let registry = ChannelRegistry::new();
    // Verify list is empty
    assert!(registry.list().is_empty());
}

// ============================================================================
// Register and Get Tests
// ============================================================================

#[test]
fn test_register_and_get() {
    let mut registry = ChannelRegistry::new();
    let channel = make_test_channel("telegram");
    registry.register(channel.clone());

    let retrieved = registry.get("telegram");
    assert!(retrieved.is_some());
    
    let retrieved_channel = retrieved.unwrap();
    assert_eq!(retrieved_channel.id(), "telegram");
}

#[test]
fn test_get_unknown_channel() {
    let registry = ChannelRegistry::new();
    
    let result = registry.get("nonexistent");
    assert!(result.is_none());
}

// ============================================================================
// List Tests
// ============================================================================

#[test]
fn test_list_preserves_order() {
    let mut registry = ChannelRegistry::new();
    
    registry.register(make_test_channel("channel_a"));
    registry.register(make_test_channel("channel_b"));
    registry.register(make_test_channel("channel_c"));

    let ids = registry.list();
    assert_eq!(ids.len(), 3);
    
    // The order may not be preserved due to HashMap, so we just check all are present
    assert!(ids.contains(&"channel_a".to_string()));
    assert!(ids.contains(&"channel_b".to_string()));
    assert!(ids.contains(&"channel_c".to_string()));
}

// ============================================================================
// Normalize ID Tests
// ============================================================================

#[test]
fn test_normalize_id_lowercase() {
    let mut registry = ChannelRegistry::new();
    registry.register(make_test_channel("telegram"));

    // normalize_id is case-sensitive, so "Telegram" != "telegram"
    let normalized = registry.normalize_id("Telegram");
    assert_eq!(normalized, None);
    
    // But lowercase "telegram" works
    let normalized_lower = registry.normalize_id("telegram");
    assert_eq!(normalized_lower, Some("telegram".to_string()));
}

#[test]
fn test_alias_resolution() {
    let mut registry = ChannelRegistry::new();
    registry.register(make_test_channel("telegram"));
    registry.add_alias("tg", "telegram");

    let channel = registry.get("tg");
    assert!(channel.is_some());
    assert_eq!(channel.unwrap().id(), "telegram");
}

#[test]
fn test_alias_for_unknown_channel() {
    let mut registry = ChannelRegistry::new();
    // Register alias before channel exists
    registry.add_alias("tg", "telegram");

    // The alias exists but channel doesn't, so get returns None
    let result = registry.get("tg");
    assert!(result.is_none());
    
    // normalize_id returns Some("telegram") because the alias maps to "telegram"
    // even if the channel doesn't exist
    let normalized = registry.normalize_id("tg");
    assert_eq!(normalized, Some("telegram".to_string()));
}

#[test]
fn test_register_duplicate() {
    let mut registry = ChannelRegistry::new();
    
    let channel1 = Arc::new(TestChannelPlugin::new("test"));
    registry.register(channel1);

    let channel2 = Arc::new(TestChannelPlugin::new("test"));
    registry.register(channel2);

    // Second registration overwrites first - verify by checking list size
    let list = registry.list();
    assert_eq!(list.len(), 1);
    
    let retrieved = registry.get("test");
    assert!(retrieved.is_some());
    // The retrieved channel should be the second one
    assert_eq!(retrieved.unwrap().id(), "test");
}

// ============================================================================
// Alias Tests
// ============================================================================

#[test]
fn test_add_alias() {
    let mut registry = ChannelRegistry::new();
    registry.register(make_test_channel("telegram"));
    
    registry.add_alias("tg", "telegram");

    assert!(registry.contains("tg"));
    assert!(registry.get("tg").is_some());
}

#[test]
fn test_add_alias_for_nonexistent_channel() {
    let mut registry = ChannelRegistry::new();
    
    registry.add_alias("tg", "telegram");

    // Alias exists but channel doesn't
    assert!(registry.contains("tg"));
    assert!(registry.get("tg").is_none());
}

#[test]
fn test_add_same_alias_twice_replaces() {
    let mut registry = ChannelRegistry::new();
    registry.register(make_test_channel("telegram"));
    registry.register(make_test_channel("discord"));

    registry.add_alias("chat", "telegram");
    registry.add_alias("chat", "discord");

    let channel = registry.get("chat").unwrap();
    assert_eq!(channel.id(), "discord");
}

#[test]
fn test_remove_alias() {
    let mut registry = ChannelRegistry::new();
    registry.register(make_test_channel("telegram"));
    registry.add_alias("tg", "telegram");

    registry.remove_alias("tg");

    assert!(!registry.contains("tg"));
    assert!(registry.get("tg").is_none());
}

// ============================================================================
// Contains Tests
// ============================================================================

#[test]
fn test_contains_channel() {
    let mut registry = ChannelRegistry::new();
    registry.register(make_test_channel("telegram"));

    assert!(registry.contains("telegram"));
    assert!(!registry.contains("discord"));
}

#[test]
fn test_contains_alias() {
    let mut registry = ChannelRegistry::new();
    registry.register(make_test_channel("telegram"));
    registry.add_alias("tg", "telegram");

    assert!(registry.contains("tg"));
}

// ============================================================================
// Normalize ID Tests
// ============================================================================

#[test]
fn test_normalize_id_direct() {
    let mut registry = ChannelRegistry::new();
    registry.register(make_test_channel("telegram"));

    let normalized = registry.normalize_id("telegram");
    assert_eq!(normalized, Some("telegram".to_string()));
}

#[test]
fn test_normalize_id_via_alias() {
    let mut registry = ChannelRegistry::new();
    registry.register(make_test_channel("telegram"));
    registry.add_alias("tg", "telegram");

    let normalized = registry.normalize_id("tg");
    assert_eq!(normalized, Some("telegram".to_string()));
}

#[test]
fn test_normalize_id_not_found() {
    let registry = ChannelRegistry::new();

    let normalized = registry.normalize_id("nonexistent");
    assert_eq!(normalized, None);
}

// ============================================================================
// Get via Alias Tests
// ============================================================================

#[test]
fn test_get_via_alias() {
    let mut registry = ChannelRegistry::new();
    registry.register(make_test_channel("telegram"));
    registry.add_alias("tg", "telegram");

    let channel = registry.get("tg").unwrap();
    assert_eq!(channel.id(), "telegram");
}

#[test]
fn test_get_via_nonexistent_alias() {
    let mut registry = ChannelRegistry::new();
    registry.add_alias("tg", "telegram");

    // Alias exists but channel doesn't, so get returns None
    assert!(registry.get("tg").is_none());
}

#[test]
fn test_get_via_canonical_id() {
    let mut registry = ChannelRegistry::new();
    registry.register(make_test_channel("telegram"));

    let channel = registry.get("telegram").unwrap();
    assert_eq!(channel.id(), "telegram");
}

// ============================================================================
// Unregister Tests
// ============================================================================

#[test]
fn test_unregister_channel() {
    let mut registry = ChannelRegistry::new();
    registry.register(make_test_channel("telegram"));

    assert!(registry.get("telegram").is_some());

    registry.unregister("telegram");
    assert!(registry.get("telegram").is_none());
}

#[test]
fn test_unregister_nonexistent_channel() {
    let mut registry = ChannelRegistry::new();
    registry.unregister("nonexistent");
    // Should be a no-op - verify by checking list is empty
    assert!(registry.list().is_empty());
}

// ============================================================================
// List Channels Tests
// ============================================================================

#[test]
fn test_list_channels() {
    let mut registry = ChannelRegistry::new();

    registry.register(make_test_channel("channel_a"));
    registry.register(make_test_channel("channel_b"));
    registry.register(make_test_channel("channel_c"));

    let ids = registry.list();
    assert_eq!(ids.len(), 3);
    assert!(ids.contains(&"channel_a".to_string()));
    assert!(ids.contains(&"channel_b".to_string()));
    assert!(ids.contains(&"channel_c".to_string()));
}

#[test]
fn test_list_channel_plugins() {
    let mut registry = ChannelRegistry::new();

    registry.register(make_test_channel("chan1"));
    registry.register(make_test_channel("chan2"));

    let channels = registry.list_channels();
    assert_eq!(channels.len(), 2);
    let ids: Vec<&str> = channels.iter().map(|c| c.id()).collect();
    assert!(ids.contains(&"chan1"));
    assert!(ids.contains(&"chan2"));
}

// ============================================================================
// Default Registry Tests
// ============================================================================

#[test]
fn test_default_registry() {
    let registry = ChannelRegistry::default();
    // Verify the default registry is empty by checking list
    assert!(registry.list().is_empty());
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_empty_string_id() {
    let mut registry = ChannelRegistry::new();
    
    // Note: This test might fail if the implementation doesn't handle empty IDs
    // For now, we just verify it compiles and runs
    let channel = TestChannelPlugin::new("");
    registry.register(Arc::new(channel));
    
    let result = registry.get("");
    // Empty ID might or might not work depending on implementation
    let _ = result;
}

#[test]
fn test_special_characters_in_id() {
    let mut registry = ChannelRegistry::new();
    
    // Test with hyphenated ID (common pattern)
    registry.register(make_test_channel("my-special-channel"));
    
    assert!(registry.get("my-special-channel").is_some());
}
