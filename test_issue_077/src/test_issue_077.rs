//! Verification script for Issue 077 implementation

use aisopod_session::{resolve_session_key, ChannelContext, PeerKind, SessionKey};

/// Test helper: Create a ChannelContext for DM
fn dm_context(channel: &str, account_id: &str, peer_id: &str) -> ChannelContext {
    ChannelContext {
        channel: channel.to_string(),
        account_id: account_id.to_string(),
        peer_kind: PeerKind::Dm,
        peer_id: peer_id.to_string(),
    }
}

/// Test helper: Create a ChannelContext for Group
fn group_context(channel: &str, account_id: &str, peer_id: &str) -> ChannelContext {
    ChannelContext {
        channel: channel.to_string(),
        account_id: account_id.to_string(),
        peer_kind: PeerKind::Group,
        peer_id: peer_id.to_string(),
    }
}

fn main() {
    println!("=== Issue 077 Implementation Verification ===\n");

    // 1. Verify exports from lib.rs
    println!("1. Checking exports from lib.rs:");
    let ctx = dm_context("discord", "bot123", "user456");
    let key = resolve_session_key("agent1", &ctx);
    println!("   ✓ resolve_session_key function is exported");
    println!("   ✓ ChannelContext struct is usable");
    println!("   ✓ PeerKind enum is usable");
    println!("   ✓ SessionKey is returned with canonical_string method\n");

    // 2. DM sessions for same user/agent produce same key
    println!("2. DM sessions for same user/agent produce same key:");
    let ctx1 = dm_context("discord", "bot123", "user456");
    let ctx2 = dm_context("discord", "bot123", "user456");
    let key1 = resolve_session_key("agent1", &ctx1);
    let key2 = resolve_session_key("agent1", &ctx2);
    let same = key1 == key2;
    println!("   DM context 1: {:?}", key1);
    println!("   DM context 2: {:?}", key2);
    println!("   Keys equal: {}", same);
    if same {
        println!("   ✓ PASS: Same DM session produces same key\n");
    } else {
        println!("   ✗ FAIL: DM session keys should be identical\n");
    }

    // 3. Group sessions for different groups produce different keys
    println!("3. Group sessions for different groups produce different keys:");
    let ctx_a = group_context("discord", "bot123", "group1");
    let ctx_b = group_context("discord", "bot123", "group2");
    let key_a = resolve_session_key("agent1", &ctx_a);
    let key_b = resolve_session_key("agent1", &ctx_b);
    let different = key_a != key_b;
    println!("   Group A: {:?}", key_a);
    println!("   Group B: {:?}", key_b);
    println!("   Keys different: {}", different);
    if different {
        println!("   ✓ PASS: Different group sessions produce different keys\n");
    } else {
        println!("   ✗ FAIL: Different group sessions should have different keys\n");
    }

    // 4. All key components are normalized (trimmed, lowercased)
    println!("4. All key components are normalized:");
    let ctx_dirty = ChannelContext {
        channel: "  DISCORD  ".to_string(),
        account_id: "  BOT_123  ".to_string(),
        peer_kind: PeerKind::Dm,
        peer_id: "  USER_456  ".to_string(),
    };
    let key_dirty = resolve_session_key("  AGENT_1  ", &ctx_dirty);
    println!("   Original channel: '  DISCORD  ' -> '{}'", key_dirty.channel);
    println!("   Original account: '  BOT_123  ' -> '{}'", key_dirty.account_id);
    println!("   Original agent: '  AGENT_1  ' -> '{}'", key_dirty.agent_id);
    println!("   Normalized peer_kind: '{}'", key_dirty.peer_kind);
    println!("   Original peer_id: '  USER_456  ' -> '{}'", key_dirty.peer_id);

    let all_normalized = 
        key_dirty.channel == "discord" &&
        key_dirty.account_id == "bot_123" &&  // BOT_123 -> bot_123 (underscore preserved)
        key_dirty.agent_id == "agent_1" &&    // AGENT_1 -> agent_1 (underscore preserved)
        key_dirty.peer_kind == "dm" &&
        key_dirty.peer_id == "user_456";      // USER_456 -> user_456 (underscore preserved)
    
    if all_normalized {
        println!("   ✓ PASS: All components are trimmed and lowercased\n");
    } else {
        println!("   ✗ FAIL: Components not properly normalized");
        println!("      Expected: channel=discord, account_id=bot_123, agent_id=agent_1, peer_id=user_456");
        println!("      Got: channel={}, account_id={}, agent_id={}, peer_id={}", 
                 key_dirty.channel, key_dirty.account_id, key_dirty.agent_id, key_dirty.peer_id);
        println!();
    }

    // 5. canonical_string returns deterministic, human-readable representation
    println!("5. canonical_string returns deterministic, human-readable representation:");
    let ctx_simple = dm_context("discord", "bot123", "user456");
    let key_simple = resolve_session_key("agent1", &ctx_simple);
    let canonical = key_simple.canonical_string();
    println!("   canonical_string: '{}'", canonical);
    
    // Verify deterministic
    let canonical2 = key_simple.canonical_string();
    let deterministic = canonical == canonical2;
    println!("   Deterministic (same call twice): {}", deterministic);
    
    // Verify human-readable format
    let parts: Vec<&str> = canonical.split(':').collect();
    let has_correct_format = parts.len() == 5 && 
        parts[0] == "agent1" && 
        parts[1] == "discord" && 
        parts[2] == "bot123" && 
        parts[3] == "dm" && 
        parts[4] == "user456";
    println!("   Format is 'agent_id:channel:account_id:peer_kind:peer_id': {}", has_correct_format);
    
    if deterministic && has_correct_format {
        println!("   ✓ PASS: canonical_string is deterministic and human-readable\n");
    } else {
        println!("   ✗ FAIL: canonical_string format issue\n");
    }

    // 6. PeerKind enum distinguishes DM from group
    println!("6. PeerKind enum distinguishes DM from group:");
    println!("   PeerKind::Dm variant: {:?}", PeerKind::Dm);
    println!("   PeerKind::Group variant: {:?}", PeerKind::Group);
    
    // Test that the enum correctly maps to session key
    let dm_ctx = dm_context("discord", "bot123", "user456");
    let group_ctx = group_context("discord", "bot123", "group1");
    let dm_key = resolve_session_key("agent1", &dm_ctx);
    let group_key = resolve_session_key("agent1", &group_ctx);
    
    let dm_correct = dm_key.peer_kind == "dm";
    let group_correct = group_key.peer_kind == "group";
    
    println!("   DM session peer_kind: '{}' (expected 'dm')", dm_key.peer_kind);
    println!("   Group session peer_kind: '{}' (expected 'group')", group_key.peer_kind);
    
    if dm_correct && group_correct {
        println!("   ✓ PASS: PeerKind correctly maps to session keys\n");
    } else {
        println!("   ✗ FAIL: PeerKind mapping incorrect\n");
    }

    // 7. Edge cases
    println!("7. Edge cases:");
    
    // Whitespace variations
    let ctx_spaces = dm_context("  DISCORD  ", "  bot_123  ", "  user_456  ");
    let key_spaces = resolve_session_key("  agent_1  ", &ctx_spaces);
    let normalized_spaces = key_spaces.channel == "discord" && 
                           key_spaces.account_id == "bot_123" && 
                           key_spaces.agent_id == "agent_1" && 
                           key_spaces.peer_id == "user_456";
    println!("   Whitespace handling: {}", if normalized_spaces { "✓" } else { "✗" });
    
    // Case insensitivity
    let ctx_case = dm_context("Discord", "Bot123", "User456");
    let key_case = resolve_session_key("Agent1", &ctx_case);
    let normalized_case = key_case.channel == "discord" && 
                         key_case.account_id == "bot123" && 
                         key_case.agent_id == "agent1" && 
                         key_case.peer_id == "user456";
    println!("   Case normalization: {}", if normalized_case { "✓" } else { "✗" });
    
    // Same user in different channels should produce DIFFERENT keys
    let ctx_discord = dm_context("discord", "bot123", "user456");
    let ctx_slack = dm_context("slack", "bot123", "user456");
    let key_discord = resolve_session_key("agent1", &ctx_discord);
    let key_slack = resolve_session_key("agent1", &ctx_slack);
    let different_channels = key_discord != key_slack;
    println!("   Same user, different channels produce different keys: {}", if different_channels { "✓" } else { "✗" });

    // Summary
    println!("\n=== Verification Complete ===");
}
