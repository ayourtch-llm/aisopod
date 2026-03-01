//! Security enforcement layer for message routing.
//!
//! This module provides the [`SecurityEnforcer`] struct that implements
//! security checks for incoming messages, including sender allowlist
//! enforcement, @mention requirements for group messages, and DM security policies.
//!
//! # Overview
//!
//! The security enforcer integrates with the [`SecurityAdapter`] trait to enforce
//! channel-specific security policies:
//!
//! - **Sender allowlist** - Verifies senders are authorized before processing messages
//! - **Mention requirement** - For group messages, checks if the bot must be @mentioned
//! - **DM security policies** - Enforces access control for direct messages
//!
//! # Usage
//!
//! ```rust,ignore
//! use aisopod_channel::{IncomingMessage, SenderInfo, SecurityEnforcer, PeerKind};
//! use aisopod_channel::adapters::SecurityAdapter;
//!
//! // Create the enforcer
//! let enforcer = SecurityEnforcer::new();
//!
//! // Check sender (with optional adapter)
//! let sender = SenderInfo {
//!     id: "user123".to_string(),
//!     display_name: Some("John Doe".to_string()),
//!     username: Some("johndoe".to_string()),
//!     is_bot: false,
//! };
//!
//! // adapter: Option<&dyn SecurityAdapter>
//! let result = enforcer.check_sender(adapter, &sender);
//! match result {
//!     Ok(()) => println!("Sender is allowed"),
//!     Err(e) => println!("Sender not allowed: {}", e),
//! }
//! ```
//!
//! # Example: Group Message with Mention Requirement
//!
//! ```rust,ignore
//! use aisopod_channel::{IncomingMessage, MessageContent, SecurityEnforcer, PeerInfo, PeerKind, SenderInfo};
//!
//! let enforcer = SecurityEnforcer::new();
//! let bot_id = "bot123";
//!
//! let message = IncomingMessage {
//!     id: "msg123".to_string(),
//!     channel: "discord".to_string(),
//!     account_id: "bot123".to_string(),
//!     sender: SenderInfo {
//!         id: "user456".to_string(),
//!         display_name: Some("Jane Doe".to_string()),
//!         username: Some("janedoe".to_string()),
//!         is_bot: false,
//!     },
//!     peer: PeerInfo {
//!         id: "channel789".to_string(),
//!         kind: PeerKind::Channel,
//!         title: Some("General".to_string()),
//!     },
//!     content: MessageContent::Text("Hello @bot123".to_string()),
//!     reply_to: None,
//!     timestamp: std::time::SystemTime::now().into(),
//!     metadata: serde_json::Value::Object(serde_json::Map::new()),
//! };
//!
//! let result = enforcer.check_mention(adapter, &message, &[bot_id.to_string()]);
//! match result {
//!     MentionCheckResult::Allowed => println!("Message allowed"),
//!     MentionCheckResult::SkipSilently => println!("Skipping (no mention)"),
//!     MentionCheckResult::Blocked(msg) => println!("Blocked: {}", msg),
//! }
//! ```

use crate::adapters::SecurityAdapter;
use crate::message::{IncomingMessage, MessageContent, MessagePart, PeerKind, SenderInfo};

/// Result of a mention check for a message.
///
/// This enum indicates whether a message passes the @mention requirement
/// in group/channel conversations, and if not, whether it should be skipped
/// silently or explicitly blocked.
///
/// # Variants
///
/// * `Allowed` - The message passes the mention check (contains bot mention).
/// * `SkipSilently` - The message does not contain a required mention and should be silently ignored.
/// * `Blocked(String)` - The message should be explicitly blocked with a reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MentionCheckResult {
    /// The message contains the required @mention and is allowed.
    Allowed,
    /// The message does not contain a required mention and should be silently skipped.
    SkipSilently,
    /// The message should be blocked with a descriptive reason.
    Blocked(String),
}

/// Enforcer for security and allowlist policies.
///
/// This struct provides reusable security checks that the message routing
/// pipeline calls before passing messages to agents. It integrates with
/// the [`SecurityAdapter`] trait to enforce channel-specific security policies.
///
/// # Methods
///
/// * [`SecurityEnforcer::check_sender()`] - Verify sender is on the allowlist
/// * [`SecurityEnforcer::check_mention()`] - Check @mention requirement for group messages
/// * [`SecurityEnforcer::check_dm_policy()`] - Enforce DM security policies
///
/// # Example
///
/// ```rust,ignore
/// use aisopod_channel::SecurityEnforcer;
///
/// let enforcer = SecurityEnforcer::new();
/// // Use enforcer.check_sender(), enforcer.check_mention(), etc.
/// ```
#[derive(Debug, Clone)]
pub struct SecurityEnforcer {
    /// Optional default policy for DM access.
    /// If None, uses the SecurityAdapter's is_allowed_sender() result directly.
    dm_policy: Option<DmPolicy>,
}

/// Policy for DM access control.
///
/// This enum specifies how direct message access should be controlled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DmPolicy {
    /// Use the same allowlist as for regular messages.
    Allowlist,
    /// Allow all DMs (no restrictions).
    Open,
    /// Custom policy - implementation defined.
    Custom,
}

impl SecurityEnforcer {
    /// Creates a new `SecurityEnforcer` with default settings.
    ///
    /// The enforcer will use the [`SecurityAdapter`] policies when available,
    /// or allow all access when no adapter is provided.
    pub fn new() -> Self {
        Self { dm_policy: None }
    }

    /// Creates a new `SecurityEnforcer` with a custom DM policy.
    ///
    /// # Arguments
    ///
    /// * `dm_policy` - The policy for DM access control.
    pub fn with_dm_policy(dm_policy: DmPolicy) -> Self {
        Self {
            dm_policy: Some(dm_policy),
        }
    }

    /// Checks if the sender is allowed to send messages.
    ///
    /// This method verifies that the sender is on the allowlist by calling
    /// [`SecurityAdapter::is_allowed_sender()`]. If the channel does not
    /// implement the `SecurityAdapter`, all senders are allowed by default.
    ///
    /// # Arguments
    ///
    /// * `adapter` - Optional reference to the [`SecurityAdapter`] implementation.
    /// * `sender` - Information about the sender to check.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - The sender is allowed.
    /// * `Err(anyhow::Error)` - The sender is not allowed, with a descriptive error message.
    ///
    /// # Errors
    ///
    /// Returns an error with a message including the sender's ID if:
    /// * The adapter is provided and `is_allowed_sender()` returns `false`
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use aisopod_channel::{SecurityEnforcer, SenderInfo};
    ///
    /// let enforcer = SecurityEnforcer::new();
    /// let sender = SenderInfo {
    ///     id: "user123".to_string(),
    ///     display_name: Some("John Doe".to_string()),
    ///     username: Some("johndoe".to_string()),
    ///     is_bot: false,
    /// };
    ///
    /// // With adapter
    /// let result = enforcer.check_sender(Some(&my_security_adapter), &sender);
    ///
    /// // Without adapter (allows all)
    /// let result = enforcer.check_sender(None, &sender);
    /// ```
    pub fn check_sender(
        &self,
        adapter: Option<&dyn SecurityAdapter>,
        sender: &SenderInfo,
    ) -> Result<(), anyhow::Error> {
        match adapter {
            None => {
                // No security adapter means open access
                Ok(())
            }
            Some(adapter) => {
                if adapter.is_allowed_sender(sender) {
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("Unauthorized sender: {}", sender.id))
                }
            }
        }
    }

    /// Checks if a message contains a required @mention in group/channel conversations.
    ///
    /// This method verifies that messages in group/channel conversations contain
    /// a bot mention if the channel requires it via [`SecurityAdapter::requires_mention_in_group()`].
    /// For direct messages, mentions are not required.
    ///
    /// The bot is considered mentioned if the message content contains any of the
    /// provided `bot_identifiers` in the format `@identifier` or `<@identifier>`.
    ///
    /// # Arguments
    ///
    /// * `adapter` - Optional reference to the [`SecurityAdapter`] implementation.
    /// * `message` - The incoming message to check.
    /// * `bot_identifiers` - List of bot identifier strings to look for.
    ///
    /// # Returns
    ///
    /// * `MentionCheckResult::Allowed` - The message contains a required mention or mentions are not required.
    /// * `MentionCheckResult::SkipSilently` - The message does not contain a required mention.
    /// * `MentionCheckResult::Blocked(String)` - The message should be explicitly blocked.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use aisopod_channel::{SecurityEnforcer, IncomingMessage};
    ///
    /// let enforcer = SecurityEnforcer::new();
    /// let bot_id = "bot123";
    ///
    /// let result = enforcer.check_mention(Some(&my_security_adapter), &message, &[bot_id.to_string()]);
    ///
    /// match result {
    ///     MentionCheckResult::Allowed => { /* process message */ }
    ///     MentionCheckResult::SkipSilently => { /* ignore message */ }
    ///     MentionCheckResult::Blocked(reason) => { /* log and handle */ }
    /// }
    /// ```
    pub fn check_mention(
        &self,
        adapter: Option<&dyn SecurityAdapter>,
        message: &IncomingMessage,
        bot_identifiers: &[String],
    ) -> MentionCheckResult {
        // If no adapter, mentions are not required
        let adapter = match adapter {
            None => return MentionCheckResult::Allowed,
            Some(a) => a,
        };

        // Mention checks only apply to group/channel messages
        match message.peer.kind {
            PeerKind::User | PeerKind::Thread => return MentionCheckResult::Allowed,
            PeerKind::Group | PeerKind::Channel => {
                // If adapter doesn't require mentions, allow the message
                if !adapter.requires_mention_in_group() {
                    return MentionCheckResult::Allowed;
                }
            }
        }

        // Check if any bot identifier is mentioned in the message
        if self.contains_bot_mention(message, bot_identifiers) {
            MentionCheckResult::Allowed
        } else {
            MentionCheckResult::SkipSilently
        }
    }

    /// Checks if the sender is allowed to send direct messages.
    ///
    /// This method enforces security policies for direct messages.
    /// If no adapter is provided, all senders are allowed by default.
    /// If an adapter is provided, it uses the same allowlist check as `check_sender()`.
    ///
    /// # Arguments
    ///
    /// * `adapter` - Optional reference to the [`SecurityAdapter`] implementation.
    /// * `sender` - Information about the sender to check.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - The sender is allowed to DM the bot.
    /// * `Err(anyhow::Error)` - The sender is not allowed, with a descriptive error message.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use aisopod_channel::{SecurityEnforcer, SenderInfo};
    ///
    /// let enforcer = SecurityEnforcer::new();
    /// let sender = SenderInfo {
    ///     id: "user123".to_string(),
    ///     display_name: Some("John Doe".to_string()),
    ///     username: Some("johndoe".to_string()),
    ///     is_bot: false,
    /// };
    ///
    /// let result = enforcer.check_dm_policy(Some(&my_security_adapter), &sender);
    /// ```
    pub fn check_dm_policy(
        &self,
        adapter: Option<&dyn SecurityAdapter>,
        sender: &SenderInfo,
    ) -> Result<(), anyhow::Error> {
        match adapter {
            None => {
                // No security adapter means open access
                Ok(())
            }
            Some(adapter) => {
                if adapter.is_allowed_sender(sender) {
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("Unauthorized DM sender: {}", sender.id))
                }
            }
        }
    }

    /// Checks if the message content contains a bot mention.
    ///
    /// Scans the message content for any of the `bot_identifiers` in formats
    /// like `@identifier` or `<@identifier>`.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to check.
    /// * `bot_identifiers` - List of bot identifiers to search for.
    ///
    /// # Returns
    ///
    /// `true` if any bot identifier is found as a mention, `false` otherwise.
    fn contains_bot_mention(&self, message: &IncomingMessage, bot_identifiers: &[String]) -> bool {
        let content = message.content_to_string();
        bot_identifiers.iter().any(|bot_id| {
            content.contains(&format!("@{}", bot_id))
                || content.contains(&format!("<@{}>", bot_id))
                || content.contains(&format!("<@{}", bot_id))
        })
    }
}

impl Default for SecurityEnforcer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PeerInfo;

    /// Mock SecurityAdapter for testing.
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
            timestamp: std::time::SystemTime::now().into(),
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

    #[test]
    fn test_check_sender_with_adapter_allows_allowed_sender() {
        let adapter = MockSecurityAdapter::new(vec!["user123".to_string()], false);
        let enforcer = SecurityEnforcer::new();
        let sender = create_sender("user123");

        let result = enforcer.check_sender(Some(&adapter), &sender);

        assert!(result.is_ok());
    }

    #[test]
    fn test_check_sender_with_adapter_rejects_unauthorized_sender() {
        let adapter = MockSecurityAdapter::new(vec!["user123".to_string()], false);
        let enforcer = SecurityEnforcer::new();
        let sender = create_sender("user456");

        let result = enforcer.check_sender(Some(&adapter), &sender);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Unauthorized sender: user456"));
    }

    #[test]
    fn test_check_sender_without_adapter_allows_all() {
        let enforcer = SecurityEnforcer::new();
        let sender = create_sender("anyone");

        let result = enforcer.check_sender(None, &sender);

        assert!(result.is_ok());
    }

    #[test]
    fn test_check_mention_without_adapter_allows_all() {
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
    fn test_check_dm_policy_with_adapter_allows_allowed_sender() {
        let adapter = MockSecurityAdapter::new(vec!["user123".to_string()], false);
        let enforcer = SecurityEnforcer::new();
        let sender = create_sender("user123");

        let result = enforcer.check_dm_policy(Some(&adapter), &sender);

        assert!(result.is_ok());
    }

    #[test]
    fn test_check_dm_policy_with_adapter_rejects_unauthorized_sender() {
        let adapter = MockSecurityAdapter::new(vec!["user123".to_string()], false);
        let enforcer = SecurityEnforcer::new();
        let sender = create_sender("user456");

        let result = enforcer.check_dm_policy(Some(&adapter), &sender);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Unauthorized DM sender: user456"));
    }

    #[test]
    fn test_check_dm_policy_without_adapter_allows_all() {
        let enforcer = SecurityEnforcer::new();
        let sender = create_sender("anyone");

        let result = enforcer.check_dm_policy(None, &sender);

        assert!(result.is_ok());
    }

    #[test]
    fn test_contains_bot_mention_with_at() {
        let enforcer = SecurityEnforcer::new();
        let message = create_message("user123", PeerKind::Group, "Hello @bot123");

        let result = enforcer.contains_bot_mention(&message, &["bot123".to_string()]);

        assert!(result);
    }

    #[test]
    fn test_contains_bot_mention_with_angle_brackets() {
        let enforcer = SecurityEnforcer::new();
        let message = create_message("user123", PeerKind::Group, "Hello <@bot123>");

        let result = enforcer.contains_bot_mention(&message, &["bot123".to_string()]);

        assert!(result);
    }

    #[test]
    fn test_contains_bot_mention_no_mention() {
        let enforcer = SecurityEnforcer::new();
        let message = create_message("user123", PeerKind::Group, "Hello world");

        let result = enforcer.contains_bot_mention(&message, &["bot123".to_string()]);

        assert!(!result);
    }

    #[test]
    fn test_contains_bot_mention_multiple_bots() {
        let enforcer = SecurityEnforcer::new();
        let message = create_message("user123", PeerKind::Group, "Hello @bot456");

        let result =
            enforcer.contains_bot_mention(&message, &["bot123".to_string(), "bot456".to_string()]);

        assert!(result);
    }
}
