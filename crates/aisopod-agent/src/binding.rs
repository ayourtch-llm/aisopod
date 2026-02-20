//! Agent binding system for routing agents to channels and sessions.
//!
//! This module provides the infrastructure for matching sessions to agents
//! based on configurable binding rules. It supports matching by channel,
//! account ID, peer, and guild ID.

use crate::types::SessionMetadata;
use serde::{Deserialize, Serialize};

/// Peer match rule for agent binding.
///
/// This type defines how to match a session based on peer information.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PeerMatch {
    /// Match any peer (wildcard)
    Any,
    /// Match a specific peer ID
    Id(String),
    /// Match peers by pattern (regex)
    Pattern(String),
}

impl PeerMatch {
    /// Creates a new PeerMatch that matches any peer.
    pub fn any() -> Self {
        PeerMatch::Any
    }

    /// Creates a new PeerMatch that matches a specific peer ID.
    pub fn id(id: impl Into<String>) -> Self {
        PeerMatch::Id(id.into())
    }

    /// Creates a new PeerMatch that matches peers by regex pattern.
    pub fn pattern(pattern: impl Into<String>) -> Self {
        PeerMatch::Pattern(pattern.into())
    }

    /// Checks if this peer match matches the given peer ID.
    pub fn matches(&self, peer_id: &str) -> bool {
        match self {
            PeerMatch::Any => true,
            PeerMatch::Id(expected) => peer_id == expected,
            PeerMatch::Pattern(pattern) => regex::Regex::new(pattern)
                .map(|re| re.is_match(peer_id))
                .unwrap_or(false),
        }
    }
}

/// A match result for agent binding.
///
/// Contains the matched session metadata fields.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BindingMatch {
    /// The channel ID that matched
    pub channel: Option<String>,
    /// The account ID that matched
    pub account_id: Option<String>,
    /// The peer information that matched
    pub peer: Option<PeerMatch>,
    /// The guild ID that matched
    pub guild_id: Option<String>,
}

impl BindingMatch {
    /// Creates a new BindingMatch with the given session metadata.
    pub fn new(
        channel: Option<String>,
        account_id: Option<String>,
        peer: Option<PeerMatch>,
        guild_id: Option<String>,
    ) -> Self {
        Self {
            channel,
            account_id,
            peer,
            guild_id,
        }
    }

    /// Checks if this match satisfies the given session metadata.
    pub fn matches(&self, session_metadata: &SessionMetadata) -> bool {
        // Check channel if specified
        if let Some(ref expected_channel) = self.channel {
            if session_metadata.channel != Some(expected_channel.clone()) {
                return false;
            }
        }

        // Check account_id if specified
        if let Some(ref expected_account_id) = self.account_id {
            if session_metadata.account_id != Some(expected_account_id.clone()) {
                return false;
            }
        }

        // Check peer if specified
        if let Some(ref expected_peer) = self.peer {
            let session_peer = session_metadata.peer.as_deref().unwrap_or("");
            if !expected_peer.matches(session_peer) {
                return false;
            }
        }

        // Check guild_id if specified
        if let Some(ref expected_guild_id) = self.guild_id {
            if session_metadata.guild_id != Some(expected_guild_id.clone()) {
                return false;
            }
        }

        true
    }
}

/// Agent binding configuration.
///
/// Defines how to route sessions to specific agents based on matching rules.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentBinding {
    /// The agent ID to bind to
    pub agent_id: String,
    /// The match rule for determining if this binding applies
    pub match_rule: BindingMatch,
    /// Priority of this binding (higher = more specific, takes precedence)
    #[serde(default)]
    pub priority: u32,
}

impl AgentBinding {
    /// Creates a new AgentBinding with the given agent ID and match rule.
    pub fn new(agent_id: impl Into<String>, match_rule: BindingMatch) -> Self {
        Self {
            agent_id: agent_id.into(),
            match_rule,
            priority: 0,
        }
    }

    /// Creates a new AgentBinding with all fields.
    pub fn with_priority(
        agent_id: impl Into<String>,
        match_rule: BindingMatch,
        priority: u32,
    ) -> Self {
        Self {
            agent_id: agent_id.into(),
            match_rule,
            priority,
        }
    }

    /// Evaluates if this binding matches the given session metadata.
    ///
    /// Returns `Some(agent_id)` if the binding matches, `None` otherwise.
    pub fn evaluate(&self, session_metadata: &SessionMetadata) -> Option<&str> {
        if self.match_rule.matches(session_metadata) {
            Some(&self.agent_id)
        } else {
            None
        }
    }

    /// Returns the priority of this binding.
    pub fn priority(&self) -> u32 {
        self.priority
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_match_any() {
        let match_any = PeerMatch::any();
        assert!(match_any.matches("any_peer_id"));
        assert!(match_any.matches("another_peer"));
    }

    #[test]
    fn test_peer_match_id() {
        let match_id = PeerMatch::id("peer_123");
        assert!(match_id.matches("peer_123"));
        assert!(!match_id.matches("peer_456"));
    }

    #[test]
    fn test_peer_match_pattern() {
        let match_pattern = PeerMatch::pattern(r"^peer_\d+$");
        assert!(match_pattern.matches("peer_123"));
        assert!(match_pattern.matches("peer_456"));
        assert!(!match_pattern.matches("peer_abc"));
        assert!(!match_pattern.matches("other_peer_123"));
    }

    #[test]
    fn test_binding_match_new() {
        let match_result = BindingMatch::new(
            Some("channel_1".to_string()),
            Some("account_1".to_string()),
            Some(PeerMatch::id("peer_1")),
            Some("guild_1".to_string()),
        );

        assert_eq!(match_result.channel, Some("channel_1".to_string()));
        assert_eq!(match_result.account_id, Some("account_1".to_string()));
        assert_eq!(match_result.peer, Some(PeerMatch::id("peer_1")));
        assert_eq!(match_result.guild_id, Some("guild_1".to_string()));
    }

    #[test]
    fn test_binding_match_matches_all_fields() {
        let match_result = BindingMatch::new(
            Some("channel_1".to_string()),
            Some("account_1".to_string()),
            Some(PeerMatch::id("peer_1")),
            Some("guild_1".to_string()),
        );

        let metadata = SessionMetadata {
            channel: Some("channel_1".to_string()),
            account_id: Some("account_1".to_string()),
            peer: Some("peer_1".to_string()),
            guild_id: Some("guild_1".to_string()),
            ..Default::default()
        };

        assert!(match_result.matches(&metadata));
    }

    #[test]
    fn test_binding_match_matches_partial_fields() {
        let match_result = BindingMatch::new(
            Some("channel_1".to_string()),
            None, // Not checking account_id
            Some(PeerMatch::id("peer_1")),
            None, // Not checking guild_id
        );

        let metadata = SessionMetadata {
            channel: Some("channel_1".to_string()),
            account_id: Some("different_account".to_string()),
            peer: Some("peer_1".to_string()),
            guild_id: Some("different_guild".to_string()),
            ..Default::default()
        };

        assert!(match_result.matches(&metadata));
    }

    #[test]
    fn test_binding_match_no_match() {
        let match_result = BindingMatch::new(Some("channel_1".to_string()), None, None, None);

        let metadata = SessionMetadata {
            channel: Some("channel_2".to_string()), // Different channel
            ..Default::default()
        };

        assert!(!match_result.matches(&metadata));
    }

    #[test]
    fn test_agent_binding_new() {
        let match_rule = BindingMatch::new(Some("channel_1".to_string()), None, None, None);
        let binding = AgentBinding::new("agent_1", match_rule);

        assert_eq!(binding.agent_id, "agent_1");
        assert_eq!(binding.priority(), 0);
    }

    #[test]
    fn test_agent_binding_with_priority() {
        let match_rule = BindingMatch::new(None, None, None, None);
        let binding = AgentBinding::with_priority("agent_1", match_rule, 10);

        assert_eq!(binding.agent_id, "agent_1");
        assert_eq!(binding.priority(), 10);
    }

    #[test]
    fn test_agent_binding_evaluate_match() {
        let match_rule = BindingMatch::new(Some("channel_1".to_string()), None, None, None);
        let binding = AgentBinding::new("agent_1", match_rule);

        let metadata = SessionMetadata {
            channel: Some("channel_1".to_string()),
            ..Default::default()
        };

        let result = binding.evaluate(&metadata);
        assert_eq!(result, Some("agent_1"));
    }

    #[test]
    fn test_agent_binding_evaluate_no_match() {
        let match_rule = BindingMatch::new(Some("channel_1".to_string()), None, None, None);
        let binding = AgentBinding::new("agent_1", match_rule);

        let metadata = SessionMetadata {
            channel: Some("channel_2".to_string()),
            ..Default::default()
        };

        let result = binding.evaluate(&metadata);
        assert_eq!(result, None);
    }

    #[test]
    fn test_agent_binding_evaluate_with_peer_match() {
        let match_rule = BindingMatch::new(None, None, Some(PeerMatch::id("peer_123")), None);
        let binding = AgentBinding::new("agent_1", match_rule);

        let metadata = SessionMetadata {
            peer: Some("peer_123".to_string()),
            ..Default::default()
        };

        let result = binding.evaluate(&metadata);
        assert_eq!(result, Some("agent_1"));
    }
}
