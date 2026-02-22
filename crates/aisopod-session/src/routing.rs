//! Session key generation and routing logic.
//!
//! This module provides utilities for generating `SessionKey` values from
//! channel context and agent binding information.

use crate::types::SessionKey;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_basic() {
        assert_eq!(normalize("  HELLO  "), "hello");
        assert_eq!(normalize("  World  "), "world");
        assert_eq!(normalize("  "), "");
        assert_eq!(normalize(""), "");
    }

    #[test]
    fn test_normalize_edge_cases() {
        assert_eq!(normalize("   "), "");
        assert_eq!(normalize("   a   "), "a");
        assert_eq!(normalize("HELLO"), "hello");
        assert_eq!(normalize("hello"), "hello");
        assert_eq!(normalize("HeLLo"), "hello");
    }

    #[test]
    fn test_session_key_canonical_string() {
        let key = SessionKey {
            agent_id: "agent_001".to_string(),
            channel: "discord".to_string(),
            account_id: "bot_123".to_string(),
            peer_kind: "dm".to_string(),
            peer_id: "user_456".to_string(),
        };

        assert_eq!(
            key.canonical_string(),
            "agent_001:discord:bot_123:dm:user_456"
        );
    }

    #[test]
    fn test_resolve_session_key_dm() {
        let agent_id = "AGENT_001";
        let ctx = ChannelContext {
            channel: "DISCORD".to_string(),
            account_id: "BOT_123".to_string(),
            peer_kind: PeerKind::Dm,
            peer_id: "USER_456".to_string(),
        };

        let key = resolve_session_key(agent_id, &ctx);

        assert_eq!(key.agent_id, "agent_001");
        assert_eq!(key.channel, "discord");
        assert_eq!(key.account_id, "bot_123");
        assert_eq!(key.peer_kind, "dm");
        assert_eq!(key.peer_id, "user_456");
    }

    #[test]
    fn test_resolve_session_key_group() {
        let agent_id = "AGENT_001";
        let ctx = ChannelContext {
            channel: "SLACK".to_string(),
            account_id: "BOT_456".to_string(),
            peer_kind: PeerKind::Group,
            peer_id: "CHANNEL_789".to_string(),
        };

        let key = resolve_session_key(agent_id, &ctx);

        assert_eq!(key.agent_id, "agent_001");
        assert_eq!(key.channel, "slack");
        assert_eq!(key.account_id, "bot_456");
        assert_eq!(key.peer_kind, "group");
        assert_eq!(key.peer_id, "channel_789");
    }

    #[test]
    fn test_resolve_session_key_with_whitespace() {
        let agent_id = "  AGENT_001  ";
        let ctx = ChannelContext {
            channel: "  DISCORD  ".to_string(),
            account_id: "  BOT_123  ".to_string(),
            peer_kind: PeerKind::Dm,
            peer_id: "  USER_456  ".to_string(),
        };

        let key = resolve_session_key(agent_id, &ctx);

        // All components should be normalized
        assert_eq!(key.agent_id, "agent_001");
        assert_eq!(key.channel, "discord");
        assert_eq!(key.account_id, "bot_123");
        assert_eq!(key.peer_kind, "dm");
        assert_eq!(key.peer_id, "user_456");
    }

    #[test]
    fn test_resolve_session_key_case_sensitivity() {
        // Test various case combinations
        let agent_id = "Agent123";
        let ctx = ChannelContext {
            channel: "Discord".to_string(),
            account_id: "Bot123".to_string(),
            peer_kind: PeerKind::Dm,
            peer_id: "User456".to_string(),
        };

        let key = resolve_session_key(agent_id, &ctx);

        assert_eq!(key.agent_id, "agent123");
        assert_eq!(key.channel, "discord");
        assert_eq!(key.account_id, "bot123");
        assert_eq!(key.peer_id, "user456");
    }

    #[test]
    fn test_peer_kind_equality() {
        assert_eq!(PeerKind::Dm, PeerKind::Dm);
        assert_eq!(PeerKind::Group, PeerKind::Group);
        assert_ne!(PeerKind::Dm, PeerKind::Group);
    }

    #[test]
    fn test_peer_kind_hash() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        PeerKind::Dm.hash(&mut hasher);
        let hash1 = hasher.finish();

        let mut hasher = DefaultHasher::new();
        PeerKind::Dm.hash(&mut hasher);
        let hash2 = hasher.finish();

        assert_eq!(hash1, hash2);

        let mut hasher = DefaultHasher::new();
        PeerKind::Group.hash(&mut hasher);
        let hash3 = hasher.finish();

        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_channel_context_equality() {
        let ctx1 = ChannelContext {
            channel: "discord".to_string(),
            account_id: "bot_123".to_string(),
            peer_kind: PeerKind::Dm,
            peer_id: "user_456".to_string(),
        };
        let ctx2 = ChannelContext {
            channel: "discord".to_string(),
            account_id: "bot_123".to_string(),
            peer_kind: PeerKind::Dm,
            peer_id: "user_456".to_string(),
        };
        let ctx3 = ChannelContext {
            channel: "slack".to_string(),
            ..ctx1.clone()
        };

        assert_eq!(ctx1, ctx2);
        assert_ne!(ctx1, ctx3);
        assert_ne!(ctx2, ctx3);
    }

    #[test]
    fn test_channel_context_hash() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let ctx = ChannelContext {
            channel: "discord".to_string(),
            account_id: "bot_123".to_string(),
            peer_kind: PeerKind::Dm,
            peer_id: "user_456".to_string(),
        };

        let mut hasher = DefaultHasher::new();
        ctx.hash(&mut hasher);
        let hash1 = hasher.finish();

        let mut hasher = DefaultHasher::new();
        ctx.hash(&mut hasher);
        let hash2 = hasher.finish();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_resolve_session_key_roundtrip() {
        // Test that resolve_session_key produces consistent results
        let agent_id = "Agent1";
        let ctx = ChannelContext {
            channel: "Discord".to_string(),
            account_id: "Bot1".to_string(),
            peer_kind: PeerKind::Dm,
            peer_id: "User1".to_string(),
        };

        let key1 = resolve_session_key(agent_id, &ctx);
        let key2 = resolve_session_key(agent_id, &ctx);

        assert_eq!(key1, key2);
        assert_eq!(key1.canonical_string(), key2.canonical_string());
    }

    #[test]
    fn test_same_user_same_key() {
        // Test that the same user always produces the same DM session key for a given agent
        let agent_id = "agent_001";
        let ctx = ChannelContext {
            channel: "discord".to_string(),
            account_id: "bot_123".to_string(),
            peer_kind: PeerKind::Dm,
            peer_id: "user_456".to_string(),
        };

        // Generate the same key multiple times
        let key1 = resolve_session_key(agent_id, &ctx);
        let key2 = resolve_session_key(agent_id, &ctx);
        let key3 = resolve_session_key(agent_id, &ctx);

        // All should be equal
        assert_eq!(key1, key2);
        assert_eq!(key2, key3);
        assert_eq!(key1.canonical_string(), key2.canonical_string());
        assert_eq!(key2.canonical_string(), key3.canonical_string());
    }
}

/// The kind of peer in a conversation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PeerKind {
    /// Direct message to a single user.
    Dm,
    /// Group conversation or channel.
    Group,
}

/// Context information for a channel conversation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChannelContext {
    /// The channel type identifier (e.g., "discord", "slack", "telegram").
    pub channel: String,
    /// The bot account identifier on the channel.
    pub account_id: String,
    /// The kind of peer (DM or group).
    pub peer_kind: PeerKind,
    /// The peer identifier (user ID for DMs, group ID for groups).
    pub peer_id: String,
}

/// Normalizes a string by trimming whitespace and converting to lowercase.
///
/// # Arguments
///
/// * `s` - The string to normalize.
///
/// # Returns
///
/// A normalized string with leading/trailing whitespace removed and all characters lowercase.
pub fn normalize(s: &str) -> String {
    s.trim().to_lowercase()
}

impl SessionKey {
    /// Returns a deterministic string representation of this session key.
    ///
    /// The canonical string format is: "agent_id:channel:account_id:peer_kind:peer_id"
    ///
    /// # Returns
    ///
    /// A string representation suitable for logging and debugging.
    pub fn canonical_string(&self) -> String {
        format!(
            "{}:{}:{}:{}:{}",
            self.agent_id, self.channel, self.account_id, self.peer_kind, self.peer_id
        )
    }
}

/// Resolves a `SessionKey` from an agent ID and channel context.
///
/// This function normalizes all components and applies routing rules:
/// - DM conversations are normalized with peer_kind="dm" and the normalized user ID as peer_id
/// - Group conversations are normalized with peer_kind="group" and the normalized group ID as peer_id
///
/// # Arguments
///
/// * `agent_id` - The agent identifier.
/// * `ctx` - The channel context containing channel, account_id, peer_kind, and peer_id.
///
/// # Returns
///
/// A `SessionKey` with normalized components.
pub fn resolve_session_key(agent_id: &str, ctx: &ChannelContext) -> SessionKey {
    let normalized_agent_id = normalize(agent_id);
    let normalized_channel = normalize(&ctx.channel);
    let normalized_account_id = normalize(&ctx.account_id);
    let normalized_peer_id = normalize(&ctx.peer_id);

    let (peer_kind, peer_id) = match ctx.peer_kind {
        PeerKind::Dm => ("dm".to_string(), normalized_peer_id),
        PeerKind::Group => ("group".to_string(), normalized_peer_id),
    };

    SessionKey {
        agent_id: normalized_agent_id,
        channel: normalized_channel,
        account_id: normalized_account_id,
        peer_kind,
        peer_id,
    }
}
