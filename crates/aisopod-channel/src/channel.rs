//! Channel registry for managing channel plugin instances.
//!
//! This module provides the [`ChannelRegistry`] struct, which serves as a
//! central registry for channel plugins. It allows registration, lookup,
//! and listing of channels, as well as registration and resolution of aliases.

use crate::plugin::ChannelPlugin;
use std::collections::HashMap;
use std::sync::Arc;

/// A channel alias mapping a friendly name to a specific channel ID.
///
/// Aliases allow users to refer to channels using convenient names that may
/// differ from the canonical channel ID.
#[derive(Debug, Clone, PartialEq)]
pub struct ChannelAlias {
    /// The canonical channel ID that this alias points to.
    pub channel_id: String,
}

/// A registry for managing channel plugin instances.
///
/// The `ChannelRegistry` serves as the central dispatch point for all
/// channel interactions. It stores channel instances keyed by their
/// unique ID and supports:
///
/// - Registering new channels
/// - Looking up channels by ID or alias
/// - Listing all registered channels
/// - Registering and resolving channel aliases
/// - Normalizing channel IDs
///
/// # Example
///
/// ```ignore
/// use std::sync::Arc;
/// use aisopod_channel::{ChannelPlugin, ChannelRegistry};
///
/// async fn example(channel: impl ChannelPlugin) -> anyhow::Result<()> {
///     let mut registry = ChannelRegistry::new();
///     registry.register(Arc::new(channel));
///
///     // Look up a channel
///     if let Some(retrieved) = registry.get("telegram") {
///         // Use the channel
///     }
///
///     // Register an alias for a channel
///     registry.add_alias("tg", "telegram");
///
///     // Resolve an alias to a channel
///     if let Some(channel) = registry.get("tg") {
///         // Uses the telegram channel
///     }
///
///     Ok(())
/// }
/// ```
pub struct ChannelRegistry {
    /// Mapping from channel ID to channel plugin instances.
    channels: HashMap<String, Arc<dyn ChannelPlugin>>,
    /// Mapping from alias to canonical channel ID.
    aliases: HashMap<String, ChannelAlias>,
}

impl ChannelRegistry {
    /// Creates a new empty `ChannelRegistry`.
    pub fn new() -> Self {
        Self {
            channels: HashMap::new(),
            aliases: HashMap::new(),
        }
    }

    /// Registers a channel with the registry.
    ///
    /// The channel is keyed by its `id()` method. If a channel with
    /// the same ID is already registered, it will be replaced.
    ///
    /// # Arguments
    ///
    /// * `channel` - An `Arc` wrapping the channel instance.
    pub fn register(&mut self, channel: Arc<dyn ChannelPlugin>) {
        let id = channel.id().to_string();
        self.channels.insert(id, channel);
    }

    /// Unregisters a channel from the registry.
    ///
    /// Removes the channel with the given ID. If the channel is not
    /// registered, this is a no-op.
    ///
    /// # Arguments
    ///
    /// * `channel_id` - The ID of the channel to remove.
    pub fn unregister(&mut self, channel_id: &str) {
        self.channels.remove(channel_id);
    }

    /// Looks up a channel by its ID or alias.
    ///
    /// Returns `Some` with an `Arc` to the channel if found, `None` otherwise.
    /// This method will first check if the given ID is a channel ID, and if not,
    /// it will check if it's an alias.
    ///
    /// # Arguments
    ///
    /// * `id` - The channel ID or alias to look up.
    pub fn get(&self, id: &str) -> Option<Arc<dyn ChannelPlugin>> {
        // First, try to get directly by channel ID
        if let Some(channel) = self.channels.get(id) {
            return Some(channel.clone());
        }

        // If not found, check if it's an alias
        if let Some(alias) = self.aliases.get(id) {
            return self.channels.get(&alias.channel_id).cloned();
        }

        None
    }

    /// Returns a list of all registered channel IDs.
    ///
    /// The order of channel IDs in the list is not guaranteed.
    pub fn list(&self) -> Vec<String> {
        self.channels.keys().cloned().collect()
    }

    /// Returns a list of all registered channel plugins.
    ///
    /// The order of channels in the list is not guaranteed.
    pub fn list_channels(&self) -> Vec<Arc<dyn ChannelPlugin>> {
        self.channels.values().cloned().collect()
    }

    /// Normalizes a channel ID.
    ///
    /// This method converts a channel ID or alias to its canonical form.
    /// If the input is an alias, it returns the canonical channel ID.
    /// If the input is already a channel ID (or doesn't match any alias),
    /// it returns the input as-is (if the channel exists) or None.
    ///
    /// # Arguments
    ///
    /// * `id` - The channel ID or alias to normalize.
    ///
    /// # Returns
    ///
    /// Returns `Some(normalized_id)` if the channel exists, `None` otherwise.
    pub fn normalize_id(&self, id: &str) -> Option<String> {
        // First, check if it's an alias
        if let Some(alias) = self.aliases.get(id) {
            return Some(alias.channel_id.clone());
        }

        // Check if it's a direct channel ID
        if self.channels.contains_key(id) {
            return Some(id.to_string());
        }

        None
    }

    /// Adds an alias for a channel.
    ///
    /// Creates a mapping from a friendly alias name to a canonical channel ID.
    /// If the alias already exists, it will be updated to point to the new channel.
    ///
    /// # Arguments
    ///
    /// * `alias` - The friendly name to register (e.g., `"tg"` for `"telegram"`).
    /// * `channel_id` - The canonical channel ID that this alias points to.
    pub fn add_alias(&mut self, alias: &str, channel_id: &str) {
        let channel_alias = ChannelAlias {
            channel_id: channel_id.to_string(),
        };
        self.aliases.insert(alias.to_string(), channel_alias);
    }

    /// Removes an alias from the registry.
    ///
    /// # Arguments
    ///
    /// * `alias` - The alias to remove.
    pub fn remove_alias(&mut self, alias: &str) {
        self.aliases.remove(alias);
    }

    /// Checks if a channel with the given ID or alias is registered.
    ///
    /// # Arguments
    ///
    /// * `id` - The channel ID or alias to check.
    pub fn contains(&self, id: &str) -> bool {
        self.channels.contains_key(id) || self.aliases.contains_key(id)
    }
}

impl Default for ChannelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::ChannelPlugin;
    use crate::types::{ChannelCapabilities, ChannelMeta};

    /// A mock channel implementation for testing.
    struct TestChannel {
        id: String,
        meta: ChannelMeta,
        capabilities: ChannelCapabilities,
    }

    impl TestChannel {
        fn new(id: &str) -> Self {
            Self {
                id: id.to_string(),
                meta: ChannelMeta {
                    label: format!("{} Channel", id.to_ascii_uppercase()),
                    docs_url: None,
                    ui_hints: serde_json::Value::Object(serde_json::Map::new()),
                },
                capabilities: ChannelCapabilities {
                    chat_types: vec![],
                    supports_media: false,
                    supports_reactions: false,
                    supports_threads: false,
                    supports_typing: false,
                    supports_voice: false,
                    max_message_length: None,
                    supported_media_types: vec![],
                },
            }
        }
    }

    #[async_trait::async_trait]
    impl ChannelPlugin for TestChannel {
        fn id(&self) -> &str {
            &self.id
        }

        fn meta(&self) -> &ChannelMeta {
            &self.meta
        }

        fn capabilities(&self) -> &ChannelCapabilities {
            &self.capabilities
        }

        fn config(&self) -> &dyn crate::adapters::ChannelConfigAdapter {
            // Return a dummy implementation for testing
            unimplemented!("TestChannel does not implement config")
        }
    }

    #[test]
    fn test_new_registry_is_empty() {
        let registry = ChannelRegistry::new();
        assert!(registry.channels.is_empty());
        assert!(registry.aliases.is_empty());
        assert!(registry.list().is_empty());
    }

    #[test]
    fn test_register_channel() {
        let mut registry = ChannelRegistry::new();
        let channel: Arc<dyn ChannelPlugin> = Arc::new(TestChannel::new("test-channel"));
        registry.register(Arc::clone(&channel));

        assert_eq!(registry.channels.len(), 1);
        assert!(registry.get("test-channel").is_some());
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_register_same_channel_twice_replaces() {
        let mut registry = ChannelRegistry::new();

        let channel1: Arc<dyn ChannelPlugin> = Arc::new(TestChannel::new("test"));
        registry.register(Arc::clone(&channel1));

        let channel2: Arc<dyn ChannelPlugin> = Arc::new(TestChannel::new("test"));
        registry.register(Arc::clone(&channel2));

        assert_eq!(registry.channels.len(), 1);
        // Both channels have the same ID, so get should return the second one
        assert!(registry.get("test").is_some());
    }

    #[test]
    fn test_register_different_channels() {
        let mut registry = ChannelRegistry::new();

        registry.register(Arc::new(TestChannel::new("channel1")) as Arc<dyn ChannelPlugin>);
        registry.register(Arc::new(TestChannel::new("channel2")) as Arc<dyn ChannelPlugin>);
        registry.register(Arc::new(TestChannel::new("channel3")) as Arc<dyn ChannelPlugin>);

        assert_eq!(registry.channels.len(), 3);
        assert!(registry.get("channel1").is_some());
        assert!(registry.get("channel2").is_some());
        assert!(registry.get("channel3").is_some());
    }

    #[test]
    fn test_unregister_channel() {
        let mut registry = ChannelRegistry::new();
        let channel: Arc<dyn ChannelPlugin> = Arc::new(TestChannel::new("test-channel"));
        registry.register(Arc::clone(&channel));

        assert!(registry.get("test-channel").is_some());

        registry.unregister("test-channel");
        assert!(registry.get("test-channel").is_none());
    }

    #[test]
    fn test_unregister_nonexistent_channel() {
        let mut registry = ChannelRegistry::new();
        registry.unregister("nonexistent");
        // Should be a no-op
        assert!(registry.channels.is_empty());
    }

    #[test]
    fn test_list_channels() {
        let mut registry = ChannelRegistry::new();

        registry.register(Arc::new(TestChannel::new("channel-a")) as Arc<dyn ChannelPlugin>);
        registry.register(Arc::new(TestChannel::new("channel-b")) as Arc<dyn ChannelPlugin>);
        registry.register(Arc::new(TestChannel::new("channel-c")) as Arc<dyn ChannelPlugin>);

        let ids = registry.list();
        assert_eq!(ids.len(), 3);
        assert!(ids.contains(&"channel-a".to_string()));
        assert!(ids.contains(&"channel-b".to_string()));
        assert!(ids.contains(&"channel-c".to_string()));
    }

    #[test]
    fn test_list_channel_plugins() {
        let mut registry = ChannelRegistry::new();

        registry.register(Arc::new(TestChannel::new("chan1")) as Arc<dyn ChannelPlugin>);
        registry.register(Arc::new(TestChannel::new("chan2")) as Arc<dyn ChannelPlugin>);

        let channels = registry.list_channels();
        assert_eq!(channels.len(), 2);
        let ids: Vec<&str> = channels.iter().map(|c| c.id()).collect();
        assert!(ids.contains(&"chan1"));
        assert!(ids.contains(&"chan2"));
    }

    #[test]
    fn test_add_alias() {
        let mut registry = ChannelRegistry::new();
        registry.register(Arc::new(TestChannel::new("telegram")) as Arc<dyn ChannelPlugin>);

        registry.add_alias("tg", "telegram");

        // Alias should be resolvable via get
        assert!(registry.get("tg").is_some());
        assert!(registry.contains("tg"));

        // But list() should only return the canonical ID
        let ids = registry.list();
        assert!(ids.contains(&"telegram".to_string()));
        assert!(!ids.contains(&"tg".to_string()));
    }

    #[test]
    fn test_add_alias_for_nonexistent_channel() {
        let mut registry = ChannelRegistry::new();

        registry.add_alias("tg", "telegram");

        // The alias exists, but get should return None since channel doesn't exist
        assert!(registry.get("tg").is_none());
    }

    #[test]
    fn test_add_same_alias_twice_replaces() {
        let mut registry = ChannelRegistry::new();
        registry.register(Arc::new(TestChannel::new("telegram")) as Arc<dyn ChannelPlugin>);
        registry.register(Arc::new(TestChannel::new("discord")) as Arc<dyn ChannelPlugin>);

        registry.add_alias("chat", "telegram");
        registry.add_alias("chat", "discord");

        // Should point to discord now
        let channel = registry.get("chat").unwrap();
        assert_eq!(channel.id(), "discord");
    }

    #[test]
    fn test_remove_alias() {
        let mut registry = ChannelRegistry::new();
        registry.register(Arc::new(TestChannel::new("telegram")));
        registry.add_alias("tg", "telegram");

        registry.remove_alias("tg");

        assert!(!registry.contains("tg"));
        assert!(registry.get("tg").is_none());
    }

    #[test]
    fn test_contains_channel() {
        let mut registry = ChannelRegistry::new();
        registry.register(Arc::new(TestChannel::new("telegram")));

        assert!(registry.contains("telegram"));
        assert!(!registry.contains("discord"));
    }

    #[test]
    fn test_contains_alias() {
        let mut registry = ChannelRegistry::new();
        registry.register(Arc::new(TestChannel::new("telegram")));
        registry.add_alias("tg", "telegram");

        assert!(registry.contains("tg"));
    }

    #[test]
    fn test_normalize_id_direct() {
        let mut registry = ChannelRegistry::new();
        registry.register(Arc::new(TestChannel::new("telegram")));

        let normalized = registry.normalize_id("telegram");
        assert_eq!(normalized, Some("telegram".to_string()));
    }

    #[test]
    fn test_normalize_id_via_alias() {
        let mut registry = ChannelRegistry::new();
        registry.register(Arc::new(TestChannel::new("telegram")));
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

    #[test]
    fn test_get_via_alias() {
        let mut registry = ChannelRegistry::new();
        registry.register(Arc::new(TestChannel::new("telegram")));
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
        registry.register(Arc::new(TestChannel::new("telegram")));

        let channel = registry.get("telegram").unwrap();
        assert_eq!(channel.id(), "telegram");
    }

    #[test]
    fn test_default_registry() {
        let registry = ChannelRegistry::default();
        assert!(registry.channels.is_empty());
        assert!(registry.aliases.is_empty());
    }
}
