//! Subagent-related tests for agent engine.
//!
//! This module tests subagent spawning functionality including resource budget
//! management and depth limits.

use aisopod_agent::subagent::{ResourceBudget, SubagentSpawnParams};
use aisopod_agent::types::AgentRunParams;

#[test]
fn test_resource_budget_new() {
    let budget = ResourceBudget::new(1000, 1000);
    assert_eq!(budget.max_tokens, 1000);
    assert_eq!(budget.remaining_tokens, 1000);
}

#[test]
fn test_resource_budget_has_budget() {
    let budget = ResourceBudget::new(1000, 500);
    assert!(budget.has_budget(400));
    assert!(budget.has_budget(500));
    assert!(!budget.has_budget(600));
}

#[test]
fn test_resource_budget_deduct() {
    let mut budget = ResourceBudget::new(1000, 500);
    let remaining = budget.deduct(200).unwrap();
    assert_eq!(remaining, 300);
    assert_eq!(budget.remaining_tokens, 300);
}

#[test]
fn test_resource_budget_deduct_insufficient() {
    let mut budget = ResourceBudget::new(1000, 100);
    let result = budget.deduct(200);
    assert!(result.is_err());
    assert_eq!(budget.remaining_tokens, 100); // Should not deduct
}

#[test]
fn test_subagent_spawn_params_new() {
    let params = SubagentSpawnParams {
        agent_id: "child_agent".to_string(),
        messages: vec![],
        parent_session_key: "parent_session".to_string(),
        parent_depth: 0,
        thread_id: None,
        resource_budget: None,
    };
    assert_eq!(params.agent_id, "child_agent");
    assert_eq!(params.parent_depth, 0);
}

#[test]
fn test_subagent_spawn_params_with_thread_id() {
    let params = SubagentSpawnParams {
        agent_id: "child_agent".to_string(),
        messages: vec![],
        parent_session_key: "parent_session".to_string(),
        parent_depth: 0,
        thread_id: Some("thread_123".to_string()),
        resource_budget: Some(ResourceBudget::new(1000, 1000)),
    };
    assert_eq!(params.thread_id, Some("thread_123".to_string()));
}

#[test]
fn test_agent_run_params_with_depth() {
    let params = AgentRunParams::with_depth(
        "session_123",
        vec![],
        Some("agent_1"),
        2,
    );
    assert_eq!(params.depth, 2);
    assert_eq!(params.agent_id, Some("agent_1".to_string()));
}

#[test]
fn test_agent_run_params_default_depth() {
    let params = AgentRunParams::new(
        "session_123",
        vec![],
        Some("agent_1"),
    );
    assert_eq!(params.depth, 0);
}

#[test]
fn test_resource_budget_with_max_and_remaining() {
    let budget = ResourceBudget::new(2000, 1500);
    assert_eq!(budget.max_tokens, 2000);
    assert_eq!(budget.remaining_tokens, 1500);
}

#[test]
fn test_resource_budget_deduct_updates_remaining() {
    let mut budget = ResourceBudget::new(1000, 1000);
    let remaining = budget.deduct(300).unwrap();
    assert_eq!(remaining, 700);
    assert_eq!(budget.remaining_tokens, 700);
}

#[test]
fn test_agent_run_params_with_depth_and_agent() {
    let params = AgentRunParams::with_depth(
        "session_xyz",
        vec![],
        Some("test_agent_1"),
        5,
    );
    assert_eq!(params.depth, 5);
    assert_eq!(params.session_key, "session_xyz");
    assert_eq!(params.agent_id, Some("test_agent_1".to_string()));
}

#[test]
fn test_agent_run_params_with_thread_id() {
    let params = AgentRunParams::with_depth_and_thread_id(
        "session_123",
        vec![],
        Some("agent_1"),
        2,
        Some("thread_xyz"),
    );
    assert_eq!(params.depth, 2);
    assert_eq!(params.thread_id, Some("thread_xyz".to_string()));
}

#[test]
fn test_agent_run_params_with_thread_id_none() {
    let params = AgentRunParams::with_depth_and_thread_id_str(
        "session_123",
        vec![],
        Some("agent_1"),
        1,
        None,
    );
    assert_eq!(params.depth, 1);
    assert_eq!(params.thread_id, None);
}

#[test]
fn test_resource_budget_zero_budget() {
    let budget = ResourceBudget::new(1000, 0);
    assert!(!budget.has_budget(1));
    assert!(budget.has_budget(0));
}

#[test]
fn test_resource_budget_exact_budget() {
    let budget = ResourceBudget::new(1000, 1000);
    assert!(budget.has_budget(1000));
}

#[test]
fn test_resource_budget_multiple_deductions() {
    let mut budget = ResourceBudget::new(1000, 1000);
    
    budget.deduct(300).unwrap();
    assert_eq!(budget.remaining_tokens, 700);
    
    budget.deduct(200).unwrap();
    assert_eq!(budget.remaining_tokens, 500);
    
    budget.deduct(250).unwrap();
    assert_eq!(budget.remaining_tokens, 250);
}

#[test]
fn test_subagent_spawn_params_clone() {
    let params = SubagentSpawnParams {
        agent_id: "child_agent".to_string(),
        messages: vec![],
        parent_session_key: "parent_session".to_string(),
        parent_depth: 0,
        thread_id: None,
        resource_budget: None,
    };
    
    let cloned = params.clone();
    assert_eq!(params.agent_id, cloned.agent_id);
    assert_eq!(params.parent_session_key, cloned.parent_session_key);
}

#[test]
fn test_resource_budget_large_numbers() {
    let budget = ResourceBudget::new(1_000_000, 500_000);
    assert!(budget.has_budget(400_000));
    assert!(!budget.has_budget(600_000));
}

#[test]
fn test_agent_run_params_with_large_depth() {
    let params = AgentRunParams::with_depth(
        "session_123",
        vec![],
        Some("agent_1"),
        100,
    );
    assert_eq!(params.depth, 100);
}
