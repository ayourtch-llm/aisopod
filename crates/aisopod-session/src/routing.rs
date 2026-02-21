//! Session key generation and routing logic.
//!
//! This module provides utilities for generating `SessionKey` values from
//! channel context and agent binding information.

use crate::types::SessionKey;

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
