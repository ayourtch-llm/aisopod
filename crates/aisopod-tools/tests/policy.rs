//! Policy enforcement tests

use aisopod_tools::{ToolPolicy, ToolPolicyEngine};

#[tokio::test]
async fn test_tool_policy_default_is_unrestricted() {
    let policy = ToolPolicy::new();
    assert!(policy.allow.is_none());
    assert!(policy.deny.is_none());
}

#[tokio::test]
async fn test_tool_policy_allow_list() {
    let policy = ToolPolicy::allow_list(vec!["read_file".to_string(), "list_files".to_string()]);

    // Check that tools in the allow list are allowed
    assert!(ToolPolicyEngine::with_global_policy(policy.clone())
        .is_allowed("agent-1", "read_file")
        .is_ok());
    assert!(ToolPolicyEngine::with_global_policy(policy.clone())
        .is_allowed("agent-1", "list_files")
        .is_ok());
    
    // Check that tools not in the allow list are denied
    assert!(ToolPolicyEngine::with_global_policy(policy)
        .is_allowed("agent-1", "bash")
        .is_err());
}

#[tokio::test]
async fn test_tool_policy_deny_list() {
    let policy = ToolPolicy::deny_list(vec!["bash".to_string(), "python".to_string()]);

    // Tools in deny list should be denied
    assert!(ToolPolicyEngine::with_global_policy(policy.clone())
        .is_allowed("agent-1", "bash")
        .is_err());
    assert!(ToolPolicyEngine::with_global_policy(policy.clone())
        .is_allowed("agent-1", "python")
        .is_err());
    
    // Tools not in deny list should be allowed
    assert!(ToolPolicyEngine::with_global_policy(policy)
        .is_allowed("agent-1", "read_file")
        .is_ok());
}

#[tokio::test]
async fn test_engine_default_is_unrestricted() {
    let engine = ToolPolicyEngine::new();
    assert!(engine.is_allowed("any-agent", "any-tool").is_ok());
}

#[tokio::test]
async fn test_global_deny_list_blocks_tools() {
    let mut engine = ToolPolicyEngine::new();
    engine.set_global_policy(ToolPolicy::deny_list(vec![
        "bash".to_string(),
        "python".to_string(),
    ]));

    assert!(engine.is_allowed("agent-1", "read_file").is_ok());
    assert!(engine.is_allowed("agent-1", "bash").is_err());
    assert!(engine.is_allowed("agent-1", "python").is_err());
}

#[tokio::test]
async fn test_global_allow_list_allows_only_whitelisted_tools() {
    let mut engine = ToolPolicyEngine::new();
    engine.set_global_policy(ToolPolicy::allow_list(vec![
        "read_file".to_string(),
        "list_files".to_string(),
    ]));

    assert!(engine.is_allowed("agent-1", "read_file").is_ok());
    assert!(engine.is_allowed("agent-1", "list_files").is_ok());
    assert!(engine.is_allowed("agent-1", "bash").is_err());
}

#[tokio::test]
async fn test_agent_policy_overrides_global_deny() {
    let mut engine = ToolPolicyEngine::new();
    engine.set_global_policy(ToolPolicy::deny_list(vec!["bash".to_string()]));

    // Agent-1 uses global policy - bash is denied
    assert!(engine.is_allowed("agent-1", "bash").is_err());

    // Agent-2 has override that allows bash
    engine.set_agent_policy(
        "agent-2".to_string(),
        ToolPolicy::allow_list(vec!["bash".to_string()]),
    );
    assert!(engine.is_allowed("agent-2", "bash").is_ok());
}

#[tokio::test]
async fn test_agent_deny_overrides_global_allow() {
    let mut engine = ToolPolicyEngine::new();
    engine.set_global_policy(ToolPolicy::allow_list(vec![
        "read_file".to_string(),
        "bash".to_string(),
    ]));

    // Agent-1 uses global policy - both tools allowed
    assert!(engine.is_allowed("agent-1", "read_file").is_ok());
    assert!(engine.is_allowed("agent-1", "bash").is_ok());

    // Agent-2 has deny list that blocks bash even though global allows it
    engine.set_agent_policy(
        "agent-2".to_string(),
        ToolPolicy::deny_list(vec!["bash".to_string()]),
    );
    assert!(engine.is_allowed("agent-2", "read_file").is_ok());
    assert!(engine.is_allowed("agent-2", "bash").is_err());
}

#[tokio::test]
async fn test_remove_agent_policy_falls_back_to_global() {
    let mut engine = ToolPolicyEngine::new();
    engine.set_global_policy(ToolPolicy::deny_list(vec!["bash".to_string()]));

    // Set agent policy - agent-1 is now allowed bash
    engine.set_agent_policy(
        "agent-1".to_string(),
        ToolPolicy::allow_list(vec!["bash".to_string()]),
    );
    assert!(engine.is_allowed("agent-1", "bash").is_ok());

    // Remove agent policy - should fall back to global (deny)
    engine.remove_agent_policy("agent-1");
    // After removing agent policy, agent-1 falls back to global deny policy
    // so bash should now be denied
    assert!(engine.is_allowed("agent-1", "bash").is_err());
}

#[tokio::test]
async fn test_deny_takes_precedence_over_allow() {
    let mut engine = ToolPolicyEngine::new();
    let policy = ToolPolicy {
        allow: Some(vec!["read_file".to_string(), "bash".to_string()]),
        deny: Some(vec!["bash".to_string()]),
    };
    engine.set_global_policy(policy);

    // bash is in both lists, but deny takes precedence
    assert!(engine.is_allowed("agent-1", "read_file").is_ok());
    assert!(engine.is_allowed("agent-1", "bash").is_err());
}

#[tokio::test]
async fn test_deny_precedence_message() {
    let mut engine = ToolPolicyEngine::new();
    engine.set_global_policy(ToolPolicy::deny_list(vec!["bash".to_string()]));

    let result = engine.is_allowed("agent-1", "bash");
    assert!(result.is_err());
    let error_msg = result.unwrap_err();
    assert!(error_msg.contains("bash"));
    assert!(error_msg.contains("global"));
}

#[tokio::test]
async fn test_agent_deny_message() {
    let mut engine = ToolPolicyEngine::new();
    engine.set_global_policy(ToolPolicy::new());
    engine.set_agent_policy(
        "agent-1".to_string(),
        ToolPolicy::deny_list(vec!["bash".to_string()]),
    );

    let result = engine.is_allowed("agent-1", "bash");
    assert!(result.is_err());
    let error_msg = result.unwrap_err();
    assert!(error_msg.contains("bash"));
    assert!(error_msg.contains("agent-1"));
}

#[tokio::test]
async fn test_agent_allow_message() {
    let mut engine = ToolPolicyEngine::new();
    engine.set_global_policy(ToolPolicy::new());
    engine.set_agent_policy(
        "agent-1".to_string(),
        ToolPolicy::allow_list(vec!["read_file".to_string()]),
    );

    // Allowed tool
    let result = engine.is_allowed("agent-1", "read_file");
    assert!(result.is_ok());

    // Disallowed tool
    let result = engine.is_allowed("agent-1", "bash");
    assert!(result.is_err());
    let error_msg = result.unwrap_err();
    assert!(error_msg.contains("bash"));
    assert!(error_msg.contains("agent-1"));
}

#[tokio::test]
async fn test_multiple_agent_policies_independent() {
    let mut engine = ToolPolicyEngine::new();
    
    // Set different policies for different agents
    engine.set_global_policy(ToolPolicy::deny_list(vec!["bash".to_string()]));
    engine.set_agent_policy(
        "agent-alpha".to_string(),
        ToolPolicy::allow_list(vec!["bash".to_string()]),
    );
    engine.set_agent_policy(
        "agent-beta".to_string(),
        ToolPolicy::allow_list(vec!["python".to_string()]),
    );

    // Agent-alpha can use bash (overrides global)
    assert!(engine.is_allowed("agent-alpha", "bash").is_ok());

    // Agent-beta cannot use bash (global deny applies)
    assert!(engine.is_allowed("agent-beta", "bash").is_err());

    // Agent-beta can use python (explicitly allowed)
    assert!(engine.is_allowed("agent-beta", "python").is_ok());

    // Agent-alpha cannot use python (not in allow list)
    assert!(engine.is_allowed("agent-alpha", "python").is_err());
}

#[tokio::test]
async fn test_global_deny_with_agent_allowlist_allows() {
    let mut engine = ToolPolicyEngine::new();
    engine.set_global_policy(ToolPolicy::deny_list(vec!["bash".to_string()]));

    // Agent with allow list that includes bash
    engine.set_agent_policy(
        "agent-special".to_string(),
        ToolPolicy::allow_list(vec!["bash".to_string(), "read_file".to_string()]),
    );

    // Agent can use bash because agent allow list overrides global deny
    assert!(engine.is_allowed("agent-special", "bash").is_ok());
    assert!(engine.is_allowed("agent-special", "read_file").is_ok());

    // Other agent is blocked by global deny
    assert!(engine.is_allowed("agent-regular", "bash").is_err());
}
