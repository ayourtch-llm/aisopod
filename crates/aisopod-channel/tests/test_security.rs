//! Tests for the SecurityEnforcer and security enforcement.

use aisopod_channel::adapters::SecurityAdapter;
use aisopod_channel::message::{IncomingMessage, MessageContent, PeerInfo, PeerKind, SenderInfo};
use aisopod_channel::security::{MentionCheckResult, SecurityEnforcer};

// ============================================================================
// Mock SecurityAdapter for Testing
// ============================================================================

/// A mock security adapter for testing.
struct MockSecurityAdapter {
    allowed_senders: Vec<String>,
    requires_mention: bool,
}

impl MockSecurityAdapter {
    fn new(allowed_senders: Vec<String>, requires_mention: bool) -> Self {
        Self {
            allowed_senders,
            requires_mention,
        }
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
// Helper Functions
// ============================================================================

fn create_message(sender_id: &str, peer_kind: PeerKind, content: &str) -> IncomingMessage {
    IncomingMessage {
        id: "msg123".to_string(),
        channel: "test".to_string(),
        account_id: "bot123".to_string(),
        sender: SenderInfo {
            id: sender_id.to_string(),
            display_name: Some("Test User".to_string()),
            username: Some("testuser".to_string()),
            is_bot: false,
        },
        peer: PeerInfo {
            id: "peer123".to_string(),
            kind: peer_kind,
            title: Some("Test".to_string()),
        },
        content: MessageContent::Text(content.to_string()),
        reply_to: None,
        timestamp: chrono::Utc::now(),
        metadata: serde_json::Value::Object(serde_json::Map::new()),
    }
}

fn create_sender(id: &str) -> SenderInfo {
    SenderInfo {
        id: id.to_string(),
        display_name: Some("Test User".to_string()),
        username: Some("testuser".to_string()),
        is_bot: false,
    }
}

// ============================================================================
// check_sender Tests
// ============================================================================

#[test]
fn test_check_sender_allowed_sender() {
    let adapter = MockSecurityAdapter::new(vec!["user123".to_string()], false);
    let enforcer = SecurityEnforcer::new();
    let sender = create_sender("user123");

    let result = enforcer.check_sender(Some(&adapter), &sender);

    assert!(result.is_ok());
}

#[test]
fn test_check_sender_blocked_sender() {
    let adapter = MockSecurityAdapter::new(vec!["user123".to_string()], false);
    let enforcer = SecurityEnforcer::new();
    let sender = create_sender("user456");

    let result = enforcer.check_sender(Some(&adapter), &sender);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Unauthorized sender: user456"));
}

#[test]
fn test_check_sender_no_adapter() {
    let enforcer = SecurityEnforcer::new();
    let sender = create_sender("anyone");

    let result = enforcer.check_sender(None, &sender);

    assert!(result.is_ok());
}

#[test]
fn test_check_sender_multiple_allowed() {
    let adapter = MockSecurityAdapter::new(
        vec![
            "user1".to_string(),
            "user2".to_string(),
            "user3".to_string(),
        ],
        false,
    );
    let enforcer = SecurityEnforcer::new();

    // Test each allowed sender
    for sender_id in &["user1", "user2", "user3"] {
        let sender = create_sender(sender_id);
        assert!(enforcer.check_sender(Some(&adapter), &sender).is_ok());
    }

    // Test blocked sender
    let sender = create_sender("user4");
    assert!(enforcer.check_sender(Some(&adapter), &sender).is_err());
}

// ============================================================================
// check_mention Tests
// ============================================================================

#[test]
fn test_check_mention_without_adapter() {
    let enforcer = SecurityEnforcer::new();
    let message = create_message("user123", PeerKind::Group, "Hello");

    let result = enforcer.check_mention(None, &message, &["bot123".to_string()]);

    assert_eq!(result, MentionCheckResult::Allowed);
}

#[test]
fn test_check_mention_dm_with_requires_mention() {
    let adapter = MockSecurityAdapter::new(vec!["user123".to_string()], true);
    let enforcer = SecurityEnforcer::new();
    // DM messages don't require mentions
    let message = create_message("user123", PeerKind::User, "Hello");

    let result = enforcer.check_mention(Some(&adapter), &message, &["bot123".to_string()]);

    assert_eq!(result, MentionCheckResult::Allowed);
}

#[test]
fn test_check_mention_thread_with_requires_mention() {
    let adapter = MockSecurityAdapter::new(vec!["user123".to_string()], true);
    let enforcer = SecurityEnforcer::new();
    // Thread messages don't require mentions
    let message = create_message("user123", PeerKind::Thread, "Hello");

    let result = enforcer.check_mention(Some(&adapter), &message, &["bot123".to_string()]);

    assert_eq!(result, MentionCheckResult::Allowed);
}

#[test]
fn test_check_mention_group_without_mention() {
    let adapter = MockSecurityAdapter::new(vec!["user123".to_string()], true);
    let enforcer = SecurityEnforcer::new();
    let message = create_message("user123", PeerKind::Group, "Hello");

    let result = enforcer.check_mention(Some(&adapter), &message, &["bot123".to_string()]);

    assert_eq!(result, MentionCheckResult::SkipSilently);
}

#[test]
fn test_check_mention_group_with_mention() {
    let adapter = MockSecurityAdapter::new(vec!["user123".to_string()], true);
    let enforcer = SecurityEnforcer::new();
    let message = create_message("user123", PeerKind::Group, "Hello @bot123");

    let result = enforcer.check_mention(Some(&adapter), &message, &["bot123".to_string()]);

    assert_eq!(result, MentionCheckResult::Allowed);
}

#[test]
fn test_check_mention_group_without_requirement() {
    let adapter = MockSecurityAdapter::new(vec!["user123".to_string()], false);
    let enforcer = SecurityEnforcer::new();
    let message = create_message("user123", PeerKind::Group, "Hello");

    let result = enforcer.check_mention(Some(&adapter), &message, &["bot123".to_string()]);

    assert_eq!(result, MentionCheckResult::Allowed);
}

#[test]
fn test_check_mention_channel_with_mention() {
    let adapter = MockSecurityAdapter::new(vec!["user123".to_string()], true);
    let enforcer = SecurityEnforcer::new();
    // Channel messages also require mentions
    let message = create_message("user123", PeerKind::Channel, "Hello @bot123");

    let result = enforcer.check_mention(Some(&adapter), &message, &["bot123".to_string()]);

    assert_eq!(result, MentionCheckResult::Allowed);
}

#[test]
fn test_check_mention_channel_without_mention() {
    let adapter = MockSecurityAdapter::new(vec!["user123".to_string()], true);
    let enforcer = SecurityEnforcer::new();
    let message = create_message("user123", PeerKind::Channel, "Hello");

    let result = enforcer.check_mention(Some(&adapter), &message, &["bot123".to_string()]);

    assert_eq!(result, MentionCheckResult::SkipSilently);
}

// ============================================================================
// check_dm_policy Tests
// ============================================================================

#[test]
fn test_check_dm_policy_allowed_sender() {
    let adapter = MockSecurityAdapter::new(vec!["user123".to_string()], false);
    let enforcer = SecurityEnforcer::new();
    let sender = create_sender("user123");

    let result = enforcer.check_dm_policy(Some(&adapter), &sender);

    assert!(result.is_ok());
}

#[test]
fn test_check_dm_policy_blocked_sender() {
    let adapter = MockSecurityAdapter::new(vec!["user123".to_string()], false);
    let enforcer = SecurityEnforcer::new();
    let sender = create_sender("user456");

    let result = enforcer.check_dm_policy(Some(&adapter), &sender);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Unauthorized DM sender: user456"));
}

#[test]
fn test_check_dm_policy_no_adapter() {
    let enforcer = SecurityEnforcer::new();
    let sender = create_sender("anyone");

    let result = enforcer.check_dm_policy(None, &sender);

    assert!(result.is_ok());
}

// ============================================================================
// DM Policy Enforcement Tests
// ============================================================================

#[test]
fn test_dm_policy_allows_allowed_sender() {
    let adapter = MockSecurityAdapter::new(vec!["user123".to_string()], false);
    let enforcer = SecurityEnforcer::new();
    let sender = create_sender("user123");

    let result = enforcer.check_dm_policy(Some(&adapter), &sender);
    assert!(result.is_ok());

    // Also check via check_sender
    let result2 = enforcer.check_sender(Some(&adapter), &sender);
    assert!(result2.is_ok());
}

#[test]
fn test_dm_policy_blocks_blocked_sender() {
    let adapter = MockSecurityAdapter::new(vec!["user123".to_string()], false);
    let enforcer = SecurityEnforcer::new();
    let sender = create_sender("user456");

    let result = enforcer.check_dm_policy(Some(&adapter), &sender);
    assert!(result.is_err());

    // Also check via check_sender
    let result2 = enforcer.check_sender(Some(&adapter), &sender);
    assert!(result2.is_err());
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_check_sender_empty_allowlist() {
    let adapter = MockSecurityAdapter::new(vec![], false);
    let enforcer = SecurityEnforcer::new();
    let sender = create_sender("anyone");

    let result = enforcer.check_sender(Some(&adapter), &sender);

    assert!(result.is_err());
}

#[test]
fn test_check_mention_empty_bot_identifiers() {
    let adapter = MockSecurityAdapter::new(vec!["user123".to_string()], true);
    let enforcer = SecurityEnforcer::new();
    let message = create_message("user123", PeerKind::Group, "Hello @bot123");

    // Empty bot identifiers list means no mention will be found
    let result = enforcer.check_mention(Some(&adapter), &message, &[]);

    assert_eq!(result, MentionCheckResult::SkipSilently);
}

#[test]
fn test_check_mention_case_sensitive_mention() {
    let adapter = MockSecurityAdapter::new(vec!["user123".to_string()], true);
    let enforcer = SecurityEnforcer::new();
    // Mention is case-sensitive
    let message = create_message("user123", PeerKind::Group, "Hello @BOT123");

    let result = enforcer.check_mention(Some(&adapter), &message, &["bot123".to_string()]);

    assert_eq!(result, MentionCheckResult::SkipSilently);
}

#[test]
fn test_check_dm_policy_with_default_policy() {
    let enforcer = SecurityEnforcer::new();
    let adapter = MockSecurityAdapter::new(vec![], false);
    let sender = create_sender("anyone");

    let result = enforcer.check_dm_policy(Some(&adapter), &sender);

    // With empty allowlist, sender should be blocked
    assert!(result.is_err());
}

#[test]
fn test_check_sender_with_bot_sender() {
    let adapter = MockSecurityAdapter::new(vec!["bot123".to_string()], false);
    let enforcer = SecurityEnforcer::new();
    let sender = SenderInfo {
        id: "bot123".to_string(),
        display_name: Some("Bot User".to_string()),
        username: Some("botuser".to_string()),
        is_bot: true,
    };

    let result = enforcer.check_sender(Some(&adapter), &sender);

    assert!(result.is_ok());
}
