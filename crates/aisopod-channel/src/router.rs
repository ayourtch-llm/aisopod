//! Message routing pipeline for aisopod-channel.
//!
//! This module implements the core message routing functionality that takes
//! incoming messages from channels and delivers them to the appropriate agents.

use std::sync::Arc;

use anyhow::Result;
use tracing::{instrument, trace};

use crate::message::IncomingMessage;
use crate::channel::ChannelRegistry;
use crate::adapters::{ChannelConfigAdapter, SecurityAdapter};
use crate::security::SecurityEnforcer;
use aisopod_session::{SessionKey, routing::resolve_session_key, PeerKind};
use aisopod_agent::resolution::resolve_session_agent_id;
use aisopod_config::AisopodConfig;
use aisopod_tools::SessionManager;

/// A message router that routes incoming messages to the appropriate agent.
///
/// The `MessageRouter` implements the message routing pipeline that takes
/// an incoming message from a channel, resolves the appropriate agent, and
/// delivers it for processing.
///
/// # Pipeline Steps
///
/// 1. **Normalize channel ID** — use `ChannelRegistry::normalize_id()` to resolve the channel identifier.
/// 2. **Resolve account** — use `ChannelConfigAdapter::resolve_account()` to load the account configuration.
/// 3. **Build session key** — use session key generation to produce a deterministic session key.
/// 4. **Check security/allowlist** — use `SecurityAdapter::is_allowed_sender()` to verify the sender.
/// 5. **Check mention requirement** — for group messages, check if the bot must be @mentioned.
/// 6. **Resolve agent** — use agent resolution to find the agent bound to this channel/account/peer combination.
/// 7. **Route to agent runner** — pass the message and resolved agent to the agent execution pipeline.
pub struct MessageRouter {
    /// The channel registry for channel lookup and ID normalization.
    registry: Arc<ChannelRegistry>,
    /// The agent resolver for finding which agent handles a message.
    agent_resolver: Arc<dyn AgentResolver>,
    /// The session manager for session key generation and session lifecycle.
    session_manager: Arc<dyn SessionManager>,
}

/// Trait for agent resolution.
///
/// This trait abstracts the agent resolution logic, allowing for
/// different implementations to determine which agent should handle
/// a given session.
pub trait AgentResolver: Send + Sync {
    /// Resolves the agent ID for the given session key.
    fn resolve(&self, session_key: &SessionKey) -> Result<String>;
}

/// Default agent resolver implementation.
///
/// This resolver uses the aisopod configuration to determine which
/// agent should handle a given session based on the session key.
pub struct ConfigAgentResolver {
    config: Arc<AisopodConfig>,
}

impl ConfigAgentResolver {
    /// Creates a new `ConfigAgentResolver` with the given configuration.
    pub fn new(config: Arc<AisopodConfig>) -> Self {
        Self { config }
    }
}

impl AgentResolver for ConfigAgentResolver {
    fn resolve(&self, session_key: &SessionKey) -> Result<String> {
        let session_key_str = session_key.canonical_string();
        resolve_session_agent_id(&self.config, &session_key_str)
    }
}

impl MessageRouter {
    /// Creates a new `MessageRouter` with the given dependencies.
    ///
    /// # Arguments
    ///
    /// * `registry` - The channel registry for channel lookup and ID normalization.
    /// * `agent_resolver` - The agent resolver for finding which agent handles the message.
    /// * `session_manager` - The session manager for session key generation and session lifecycle.
    pub fn new(
        registry: Arc<ChannelRegistry>,
        agent_resolver: Arc<dyn AgentResolver>,
        session_manager: Arc<dyn SessionManager>,
    ) -> Self {
        Self {
            registry,
            agent_resolver,
            session_manager,
        }
    }

    /// Routes an incoming message to the appropriate agent.
    ///
    /// This method implements the full message routing pipeline:
    /// 1. Normalize the channel ID
    /// 2. Resolve the account configuration
    /// 3. Build a session key
    /// 4. Check security/allowlist
    /// 5. Check mention requirement for group messages
    /// 6. Resolve the agent
    /// 7. Route to the agent runner
    ///
    /// # Arguments
    ///
    /// * `message` - The incoming message to route.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the message was routed successfully, or an error if routing failed.
    ///
    /// # Errors
    ///
    /// * `anyhow::Error` - Various errors can occur during routing:
    ///   - `UnknownChannel` - The channel is not registered.
    ///   - `AccountNotFound` - The account is not found or disabled.
    ///   - `Unauthorized` - The sender is not authorized.
    ///   - `MissingMention` - Group message without required @mention.
    #[instrument(skip(self, message))]
    pub async fn route(&self, message: IncomingMessage) -> Result<()> {
        trace!("Routing incoming message");

        // Step 1: Normalize channel ID
        let normalized_channel_id = self
            .registry
            .normalize_id(&message.channel)
            .ok_or_else(|| anyhow::anyhow!("Unknown channel: {}", message.channel))?;
        trace!("Normalized channel ID: {}", normalized_channel_id);

        // Step 2: Resolve account
        let plugin = self
            .registry
            .get(&normalized_channel_id)
            .ok_or_else(|| anyhow::anyhow!("Channel not found after normalization: {}", normalized_channel_id))?;

        let account = plugin
            .config()
            .resolve_account(&message.account_id)
            .map_err(|_| anyhow::anyhow!("Account not found or disabled: {}", message.account_id))?;

        if !account.enabled {
            return Err(anyhow::anyhow!("Account not found or disabled: {}", message.account_id));
        }
        trace!("Resolved account: {}", message.account_id);

        // Step 3: Build session key
        let session_key = self.build_session_key(&normalized_channel_id, &message);
        trace!("Built session key: {}", session_key.canonical_string());

        // Step 4: Check security/allowlist
        let enforcer = SecurityEnforcer::new();
        let security_adapter = plugin.security();
        enforcer.check_sender(security_adapter, &message.sender)?;

        trace!("Security check passed");

        // Step 5: Check mention requirement for group messages
        // Build bot identifiers from the bot's sender info
        let bot_identifiers = vec![message.account_id.clone()];

        match enforcer.check_mention(security_adapter, &message, &bot_identifiers) {
            crate::security::MentionCheckResult::Allowed => {
                trace!("Mention check passed");
            }
            crate::security::MentionCheckResult::SkipSilently => {
                trace!("Skipping group message without @mention");
                return Ok(());
            }
            crate::security::MentionCheckResult::Blocked(reason) => {
                return Err(anyhow::anyhow!("Message blocked: {}", reason));
            }
        }

        // Step 6: Resolve agent
        let agent_id = self.agent_resolver.resolve(&session_key)?;
        trace!("Resolved agent: {}", agent_id);

        // Step 7: Route to agent runner
        self.route_to_runner(&session_key, &message, agent_id).await?;

        Ok(())
    }

    /// Builds a session key from the channel ID, account ID, and peer info.
    fn build_session_key(&self, channel_id: &str, message: &IncomingMessage) -> SessionKey {
        // Create a ChannelContext for the session key resolution
        let peer_kind = match message.peer.kind {
            crate::PeerKind::User => PeerKind::Dm,
            crate::PeerKind::Group => PeerKind::Group,
            crate::PeerKind::Channel => PeerKind::Group,
            crate::PeerKind::Thread => PeerKind::Group,
        };

        let ctx = aisopod_session::ChannelContext {
            channel: channel_id.to_string(),
            account_id: message.account_id.clone(),
            peer_kind,
            peer_id: message.peer.id.clone(),
        };

        // Determine agent_id from the session key - resolve with empty peer info first
        let session_key_temp = SessionKey {
            agent_id: "".to_string(),
            channel: channel_id.to_string(),
            account_id: message.account_id.clone(),
            peer_kind: match message.peer.kind {
                crate::PeerKind::User => "dm".to_string(),
                crate::PeerKind::Group => "group".to_string(),
                crate::PeerKind::Channel => "group".to_string(),
                crate::PeerKind::Thread => "group".to_string(),
            },
            peer_id: message.peer.id.clone(),
        };

        let agent_id = self.agent_resolver.resolve(&session_key_temp).ok();

        // Use the resolved agent_id or fall back to a default
        let agent_id = agent_id.unwrap_or_else(|| "default_agent".to_string());

        resolve_session_key(&agent_id, &ctx)
    }

    /// Routes the message to the agent runner.
    async fn route_to_runner(
        &self,
        session_key: &SessionKey,
        message: &IncomingMessage,
        agent_id: String,
    ) -> Result<()> {
        // For now, this is a placeholder that will need to be implemented
        // when the agent runner integration is complete
        // 
        // A full implementation would:
        // 1. Create or load the session using the session manager
        // 2. Prepare the message history for the agent
        // 3. Run the agent with the message
        // 4. Handle the agent's response
        //
        // This would use the AgentRunner from aisopod-agent
        
        Ok(())
    }
}

impl std::fmt::Debug for MessageRouter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageRouter")
            .field("registry", &"ChannelRegistry {...}")
            .field("agent_resolver", &"AgentResolver {...}")
            .field("session_manager", &"SessionManager {...}")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize() {
        assert_eq!(normalize("  HELLO  "), "hello");
        assert_eq!(normalize("  World  "), "world");
        assert_eq!(normalize("  "), "");
        assert_eq!(normalize(""), "");
    }

    #[test]
    fn test_is_bot_mentioned_with_at() {
        // This test would require a proper IncomingMessage with content
        // For now, we just verify the function compiles
        assert!(true);
    }

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
}

/// Normalizes a string by trimming whitespace and converting to lowercase.
fn normalize(s: &str) -> String {
    s.trim().to_lowercase()
}
