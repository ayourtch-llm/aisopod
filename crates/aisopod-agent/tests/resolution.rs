//! Resolution tests for agent engine.
//!
//! This module tests agent resolution functionality including:
//! - Session agent ID resolution with multiple bindings
//! - Default agent fallback when no bindings match
//! - Agent configuration resolution
//! - Model chain resolution
//! - Listing all agent IDs

#[path = "helpers.rs"]
mod helpers;

use std::sync::Arc;

use aisopod_agent::resolution::ModelChain;
use aisopod_agent::{
    list_agent_ids, resolve_agent_config, resolve_agent_model, resolve_session_agent_id,
};
use aisopod_config::types::AgentBinding;

use helpers::{test_config, test_config_with_fallbacks};

// ============================================================================
// Binding Resolution Tests
// ============================================================================

#[test]
fn test_resolve_session_agent_id_with_binding() {
    let config = test_config();

    let agent_id = resolve_session_agent_id(&config, "session_123").unwrap();

    // Should return test-agent based on binding priority
    assert_eq!(agent_id, "test-agent");
}

#[test]
fn test_resolve_session_agent_id_no_bindings() {
    let mut config = test_config();
    config.bindings = Vec::new();

    // With no bindings, should return first agent
    let agent_id = resolve_session_agent_id(&config, "session_123").unwrap();

    assert_eq!(agent_id, "default");
}

#[test]
fn test_resolve_session_agent_id_default_fallback() {
    let mut config = test_config();
    config.bindings = Vec::new();
    // Keep at least one agent so resolve_session_agent_id can return it
    // Use the default agent which has id "default"

    let agent_id = resolve_session_agent_id(&config, "session_123").unwrap();

    assert_eq!(agent_id, "default");
}

#[test]
fn test_resolve_session_agent_id_no_agents() {
    let mut config = test_config();
    config.bindings = Vec::new();
    config.agents.agents = Vec::new();

    // Should return error when no agents configured
    let result = resolve_session_agent_id(&config, "session_123");

    assert!(result.is_err());
}

// ============================================================================
// Agent Configuration Resolution Tests
// ============================================================================

#[test]
fn test_resolve_agent_config_success() {
    let config = test_config();

    let agent = resolve_agent_config(&config, "test-agent").unwrap();

    assert_eq!(agent.id, "test-agent");
    assert_eq!(agent.system_prompt, "You are a test agent.");
    assert_eq!(agent.model, "mock/test-model");
}

#[test]
fn test_resolve_agent_config_not_found() {
    let config = test_config();

    let result = resolve_agent_config(&config, "nonexistent-agent");

    assert!(result.is_err());
}

#[test]
fn test_resolve_agent_config_default() {
    let config = test_config();

    let agent = resolve_agent_config(&config, "default").unwrap();

    assert_eq!(agent.id, "default");
    assert_eq!(agent.system_prompt, "You are a helpful assistant.");
}

// ============================================================================
// Model Chain Resolution Tests
// ============================================================================

#[test]
fn test_resolve_agent_model_success() {
    let config = test_config();

    // Use a model that doesn't have fallbacks configured
    // Create a config with no fallbacks for testing
    let mut config_no_fallbacks = test_config();
    config_no_fallbacks.models.fallbacks = Vec::new();

    let model_chain = resolve_agent_model(&config_no_fallbacks, "default").unwrap();

    assert_eq!(model_chain.primary(), "mock/test-model");
    assert!(model_chain.fallbacks().is_empty());
}

#[test]
fn test_resolve_agent_model_with_fallbacks() {
    let config = test_config_with_fallbacks();

    let model_chain = resolve_agent_model(&config, "test-agent").unwrap();

    assert_eq!(model_chain.primary(), "mock/test-model");
    assert_eq!(model_chain.fallbacks().len(), 2);
    assert_eq!(model_chain.fallbacks()[0], "mock/fallback-model");
    assert_eq!(model_chain.fallbacks()[1], "mock/another-model");
}

#[test]
fn test_resolve_agent_model_not_found() {
    let config = test_config();

    let result = resolve_agent_model(&config, "nonexistent-agent");

    assert!(result.is_err());
}

#[test]
fn test_model_chain_primary() {
    let chain = ModelChain::new("gpt-4");

    assert_eq!(chain.primary(), "gpt-4");
}

#[test]
fn test_model_chain_fallbacks() {
    let chain = ModelChain::with_fallbacks(
        "gpt-4",
        vec!["gpt-3.5-turbo".to_string(), "claude-3-opus".to_string()],
    );

    assert_eq!(chain.fallbacks().len(), 2);
    assert_eq!(chain.fallbacks()[0], "gpt-3.5-turbo");
    assert_eq!(chain.fallbacks()[1], "claude-3-opus");
}

#[test]
fn test_model_chain_all_models() {
    let chain = ModelChain::with_fallbacks("gpt-4", vec!["gpt-3.5-turbo".to_string()]);

    let all = chain.all_models();

    assert_eq!(all.len(), 2);
    assert_eq!(all[0], "gpt-4");
    assert_eq!(all[1], "gpt-3.5-turbo");
}

// ============================================================================
// List Agent IDs Tests
// ============================================================================

#[test]
fn test_list_agent_ids() {
    let config = test_config();

    let agent_ids = list_agent_ids(&config);

    // Should include default, test-agent, and fallback-agent
    assert_eq!(agent_ids.len(), 3);
    assert!(agent_ids.contains(&"default".to_string()));
    assert!(agent_ids.contains(&"test-agent".to_string()));
    assert!(agent_ids.contains(&"fallback-agent".to_string()));
}

#[test]
fn test_list_agent_ids_empty() {
    let mut config = test_config();
    config.agents.agents = Vec::new();

    let agent_ids = list_agent_ids(&config);

    assert!(agent_ids.is_empty());
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_full_resolution_flow() {
    let config = test_config();

    // Step 1: Resolve session to agent
    let session_key = "session_123";
    let agent_id = resolve_session_agent_id(&config, session_key).unwrap();
    assert_eq!(agent_id, "test-agent");

    // Step 2: Resolve agent config
    let agent_config = resolve_agent_config(&config, &agent_id).unwrap();
    assert_eq!(agent_config.id, agent_id);

    // Step 3: Resolve model chain
    let model_chain = resolve_agent_model(&config, &agent_id).unwrap();
    assert_eq!(model_chain.primary(), agent_config.model);
}

#[test]
fn test_resolution_with_priority() {
    // Test that bindings are properly prioritized
    let mut config = test_config();
    config.bindings.push(AgentBinding {
        agent_id: "fallback-agent".to_string(),
        channels: vec![],
        priority: 10, // Lower priority
        sandbox: None,
    });

    let agent_id = resolve_session_agent_id(&config, "session_123").unwrap();

    // Should still return test-agent (higher priority)
    assert_eq!(agent_id, "test-agent");
}
