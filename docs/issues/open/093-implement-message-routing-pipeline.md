# Issue 093: Implement Message Routing Pipeline

## Summary
Implement the message routing pipeline that takes an incoming message from a channel, resolves the appropriate agent, and delivers it for processing. This is the core data flow connecting channels to the agent engine.

## Location
- Crate: `aisopod-channel`
- File: `crates/aisopod-channel/src/router.rs`

## Current Behavior
There is no message routing logic. Incoming messages from channels have no path to reach the agent engine.

## Expected Behavior
A `MessageRouter` struct implements the following pipeline for each incoming message:

1. **Normalize channel ID** — use `ChannelRegistry::normalize_id()` to resolve the channel identifier.
2. **Resolve account** — use `ChannelConfigAdapter::resolve_account()` to load the account configuration for the message's `account_id`.
3. **Build session key** — use session key generation (from Issue 077) combining channel, account, and peer info to produce a deterministic session key.
4. **Check security/allowlist** — use `SecurityAdapter::is_allowed_sender()` to verify the sender is permitted. Reject unauthorized senders with an appropriate error.
5. **Check mention requirement** — for group messages, use `SecurityAdapter::requires_mention_in_group()` to determine if the bot must be @mentioned. If required and not mentioned, silently ignore the message.
6. **Resolve agent** — use agent resolution (from Issue 063) to find the agent bound to this channel/account/peer combination.
7. **Route to agent runner** — pass the message and resolved agent to the agent execution pipeline, creating or loading the session as needed.

The router exposes an async method:
```rust
pub async fn route(&self, message: IncomingMessage) -> Result<()>;
```

## Impact
This is the central integration point between the channel layer and the agent engine. Without it, messages cannot flow from messaging platforms to agents.

## Suggested Implementation
1. Open `crates/aisopod-channel/src/router.rs`.
2. Define `MessageRouter` struct with fields:
   - `registry: Arc<ChannelRegistry>` — for channel lookup and ID normalization.
   - `agent_resolver: Arc<dyn AgentResolver>` — for resolving which agent handles the message (from Issue 063).
   - `session_manager: Arc<dyn SessionManager>` — for session key generation and session lifecycle (from Issue 077).
3. Implement `MessageRouter::new()` accepting the three dependencies.
4. Implement `async fn route(&self, message: IncomingMessage) -> Result<()>`:
   - Step 1: Call `self.registry.normalize_id(&message.channel)`. Return an error if the channel is unknown.
   - Step 2: Get the channel plugin via `self.registry.get()`. Call `plugin.config().resolve_account(&message.account_id)` to load the account. Return an error if the account is not found or disabled.
   - Step 3: Build a session key from the channel ID, account ID, and peer info. Use the session key builder from Issue 077.
   - Step 4: Attempt to get a `SecurityAdapter` from the channel plugin. If available, call `is_allowed_sender(&message.sender)`. If the sender is not allowed, return an `Unauthorized` error.
   - Step 5: If the message peer kind is `Group` and the security adapter's `requires_mention_in_group()` returns true, check if the message content contains an @mention of the bot. If not, return `Ok(())` silently (ignore the message).
   - Step 6: Call `self.agent_resolver.resolve(&session_key)` to find the bound agent. Return an error if no agent is bound.
   - Step 7: Create or load the session using the session manager, then pass the message to the agent runner for execution.
5. Add doc-comments to the struct, constructor, and `route()` method.
6. Re-export `MessageRouter` from `crates/aisopod-channel/src/lib.rs`.
7. Run `cargo check -p aisopod-channel` to verify everything compiles.

## Dependencies
- Issue 091 (define message types)
- Issue 092 (implement channel registry)
- Issue 063 (implement agent resolution and binding)
- Issue 077 (implement session key generation and routing)

## Acceptance Criteria
- [ ] `MessageRouter` struct is defined with channel registry, agent resolver, and session manager dependencies
- [ ] `route()` method implements the full pipeline: normalize → resolve account → build session key → check security → check mention → resolve agent → route to runner
- [ ] Unknown channels return an error
- [ ] Disabled or missing accounts return an error
- [ ] Unauthorized senders are rejected
- [ ] Group messages without required @mention are silently ignored
- [ ] Messages are delivered to the correct agent
- [ ] `cargo check -p aisopod-channel` compiles without errors

---
*Created: 2026-02-15*
