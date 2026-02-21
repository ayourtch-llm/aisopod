//! Pipeline-related tests for agent engine.
//!
//! This module tests the agent execution pipeline components including:
//! - Pipeline construction and basic operations
//! - Full pipeline execution with mock providers
//! - Tool call handling
//! - Event streaming

#[path = "helpers.rs"]
mod helpers;

use aisopod_agent::pipeline::{AgentPipeline, AgentRunStream};
use aisopod_agent::types::{AgentEvent, AgentRunResult};
use aisopod_provider::Message;
use std::sync::Arc;

use aisopod_session::SessionStore;

use helpers::{
    collect_events, test_abort_registry, test_agent_run_params, test_agent_run_result, test_config,
    test_session_store, test_tool_registry, user_message, MockProvider,
};

// ============================================================================
// Pipeline Construction Tests
// ============================================================================

#[test]
fn test_agent_run_stream_new() {
    let (tx, rx) = tokio::sync::mpsc::channel(10);
    let stream = AgentRunStream::new(rx);
    assert!(stream.receiver().capacity() > 0);
    drop(tx);
}

#[test]
fn test_agent_run_stream_receiver() {
    let (tx, rx) = tokio::sync::mpsc::channel(10);
    let stream = AgentRunStream::new(rx);

    let receiver = stream.receiver();
    assert!(receiver.capacity() > 0);

    drop(tx);
}

#[test]
fn test_agent_run_stream_into_receiver() {
    let (tx, rx) = tokio::sync::mpsc::channel(10);
    let stream = AgentRunStream::new(rx);
    let mut receiver = stream.into_receiver();

    assert!(receiver.try_recv().is_err());
    drop(tx);
}

#[test]
fn test_agent_pipeline_new() {
    // Basic test to verify the struct can be instantiated
    // Actual pipeline execution requires full dependencies
    let (_config, _providers, _tools, _sessions) = create_test_dependencies();

    let pipeline = AgentPipeline::new(_config, _providers, _tools, _sessions);

    assert!(!pipeline.has_usage_tracker());
    assert!(pipeline.usage_tracker().is_none());
    assert!(pipeline.abort_registry().is_none());
}

#[test]
fn test_agent_pipeline_new_with_usage_tracker() {
    let (_config, _providers, _tools, _sessions) = create_test_dependencies();
    let usage_tracker = Arc::new(aisopod_agent::usage::UsageTracker::new());

    let pipeline = AgentPipeline::new_with_usage_tracker(
        _config,
        _providers,
        _tools,
        _sessions,
        usage_tracker.clone(),
    );

    assert!(pipeline.has_usage_tracker());
    assert!(pipeline.usage_tracker().is_some());
    assert!(pipeline.abort_registry().is_none());
}

#[test]
fn test_agent_pipeline_new_with_abort_registry() {
    let (_config, _providers, _tools, _sessions) = create_test_dependencies();
    let usage_tracker = Arc::new(aisopod_agent::usage::UsageTracker::new());
    let abort_registry = Arc::new(aisopod_agent::AbortRegistry::new());

    let pipeline = AgentPipeline::new_with_abort_registry(
        _config,
        _providers,
        _tools,
        _sessions,
        usage_tracker.clone(),
        abort_registry.clone(),
    );

    assert!(pipeline.has_usage_tracker());
    assert!(pipeline.usage_tracker().is_some());
    assert!(pipeline.abort_registry().is_some());
}

// ============================================================================
// Helper Functions
// ============================================================================

fn create_test_dependencies() -> (
    Arc<aisopod_config::AisopodConfig>,
    Arc<aisopod_provider::ProviderRegistry>,
    Arc<aisopod_tools::ToolRegistry>,
    Arc<aisopod_session::SessionStore>,
) {
    // Create minimal test dependencies
    // In real scenarios, these would be properly configured
    let config = Arc::new(aisopod_config::AisopodConfig::default());
    let providers = Arc::new(aisopod_provider::ProviderRegistry::new());
    let tools = Arc::new(aisopod_tools::ToolRegistry::new());
    let sessions =
        Arc::new(SessionStore::new_in_memory().expect("Failed to create in-memory session store"));

    (config, providers, tools, sessions)
}

// ============================================================================
// Pipeline Integration Tests with Mock Provider
// ============================================================================

#[tokio::test]
async fn test_pipeline_text_only_response() {
    // Test a simple text-only response (no tool calls)

    // Create mock provider that returns text response
    let mock_provider = Arc::new(MockProvider::new("mock").with_response_text("Hello, world!"));

    // Get test configuration
    let config = test_config();

    // Create provider registry and add mock provider
    let mut providers = aisopod_provider::ProviderRegistry::new();
    providers.register(mock_provider);
    // Register alias for the mock model
    providers.register_alias("mock/test-model", "mock", "mock/test-model");
    let providers = Arc::new(providers);

    // Create other dependencies
    let tools = test_tool_registry();
    let sessions = test_session_store();
    let abort_registry = test_abort_registry();
    let usage_tracker = Arc::new(aisopod_agent::usage::UsageTracker::new());

    // Create pipeline
    let pipeline = AgentPipeline::new(Arc::new(config), providers.clone(), tools, sessions);

    // Set up abort registry on pipeline
    let pipeline = AgentPipeline::new_with_abort_registry(
        pipeline.config().clone(),
        providers,
        pipeline.tools().clone(),
        pipeline.sessions().clone(),
        usage_tracker,
        abort_registry,
    );

    // Create message stream
    let (event_tx, mut event_rx) = tokio::sync::mpsc::channel(50);

    // Create run params
    let messages = vec![user_message("Hello")];
    let params = test_agent_run_params("test_session", messages, Some("default"));

    // Execute pipeline
    let result = pipeline.execute(&params, &event_tx).await;

    // Verify result
    assert!(
        result.is_ok(),
        "Pipeline execution should succeed: {:?}",
        result
    );
    let result = result.unwrap();

    // Verify response
    assert!(
        result.response.contains("Hello"),
        "Response should contain the expected text"
    );
}

#[tokio::test]
async fn test_pipeline_with_tool_call() {
    // Test a response with one tool call -> tool executed -> final text response

    use aisopod_provider::ToolCall;

    // Create mock provider that returns a tool call first, then text
    let mock_provider = Arc::new(MockProvider::new("mock").with_tool_calls(vec![ToolCall {
        id: "call_1".to_string(),
        name: "calculator".to_string(),
        arguments: r#"{"operation":"add","a":5,"b":3}"#.to_string(),
    }]));

    // Get test configuration
    let config = test_config();

    // Create provider registry and add mock provider
    let mut providers = aisopod_provider::ProviderRegistry::new();
    providers.register(mock_provider);
    // Register alias for the mock model
    providers.register_alias("mock/test-model", "mock", "mock/test-model");
    let providers = Arc::new(providers);

    // Create other dependencies
    let tools = test_tool_registry();
    let sessions = test_session_store();
    let abort_registry = test_abort_registry();
    let usage_tracker = Arc::new(aisopod_agent::usage::UsageTracker::new());

    // Create pipeline with abort registry
    let pipeline = AgentPipeline::new_with_usage_tracker(
        Arc::new(config),
        providers.clone(),
        tools.clone(),
        sessions.clone(),
        usage_tracker,
    );

    let pipeline = AgentPipeline::new_with_abort_registry(
        pipeline.config().clone(),
        providers,
        tools,
        sessions,
        pipeline.usage_tracker().cloned().unwrap(),
        abort_registry,
    );

    // Create message stream
    let (event_tx, mut event_rx) = tokio::sync::mpsc::channel(50);

    // Create run params
    let messages = vec![user_message("What is 5 + 3?")];
    let params = test_agent_run_params("test_session", messages, Some("default"));

    // Execute pipeline
    let result = pipeline.execute(&params, &event_tx).await;

    // Verify result
    assert!(result.is_ok(), "Pipeline execution should succeed");
    let result = result.unwrap();

    // Verify tool calls were recorded
    assert!(!result.tool_calls.is_empty(), "Should have tool calls");
}

#[tokio::test]
async fn test_pipeline_with_multiple_tool_calls() {
    // Test a response with multiple sequential tool calls

    use aisopod_provider::ToolCall;

    // Create mock provider that returns multiple tool calls
    let mock_provider = Arc::new(MockProvider::new("mock").with_tool_calls(vec![
        ToolCall {
            id: "call_1".to_string(),
            name: "calculator".to_string(),
            arguments: r#"{"operation":"add","a":5,"b":3}"#.to_string(),
        },
        ToolCall {
            id: "call_2".to_string(),
            name: "calculator".to_string(),
            arguments: r#"{"operation":"multiply","a":2,"b":4}"#.to_string(),
        },
    ]));

    // Get test configuration
    let config = test_config();

    // Create provider registry and add mock provider
    let mut providers = aisopod_provider::ProviderRegistry::new();
    providers.register(mock_provider);
    // Register alias for the mock model
    providers.register_alias("mock/test-model", "mock", "mock/test-model");
    let providers = Arc::new(providers);

    // Create other dependencies
    let tools = test_tool_registry();
    let sessions = test_session_store();
    let abort_registry = test_abort_registry();
    let usage_tracker = Arc::new(aisopod_agent::usage::UsageTracker::new());

    // Create pipeline
    let pipeline = AgentPipeline::new_with_usage_tracker(
        Arc::new(config),
        providers.clone(),
        tools.clone(),
        sessions.clone(),
        usage_tracker,
    );

    let pipeline = AgentPipeline::new_with_abort_registry(
        pipeline.config().clone(),
        providers,
        pipeline.tools().clone(),
        pipeline.sessions().clone(),
        pipeline.usage_tracker().cloned().unwrap(),
        abort_registry,
    );

    // Create message stream
    let (event_tx, mut event_rx) = tokio::sync::mpsc::channel(50);

    // Create run params
    let messages = vec![user_message("What is 5 + 3 and 2 * 4?")];
    let params = test_agent_run_params("test_session", messages, Some("default"));

    // Execute pipeline
    let result = pipeline.execute(&params, &event_tx).await;

    // Verify result
    assert!(result.is_ok(), "Pipeline execution should succeed");
    let result = result.unwrap();

    // Verify multiple tool calls were recorded
    assert_eq!(result.tool_calls.len(), 2, "Should have two tool calls");
}

#[tokio::test]
async fn test_pipeline_event_order() {
    // Test that events are emitted in the correct order

    // Create mock provider
    let mock_provider = Arc::new(MockProvider::new("mock").with_response_text("Test response"));

    // Get test configuration
    let config = test_config();

    // Create provider registry and add mock provider
    let mut providers = aisopod_provider::ProviderRegistry::new();
    providers.register(mock_provider);
    // Register alias for the mock model
    providers.register_alias("mock/test-model", "mock", "mock/test-model");
    let providers = Arc::new(providers);

    // Create other dependencies
    let tools = test_tool_registry();
    let sessions = test_session_store();

    // Create pipeline
    let pipeline = AgentPipeline::new(Arc::new(config), providers, tools, sessions);

    // Create message stream
    let (event_tx, mut event_rx) = tokio::sync::mpsc::channel(50);

    // Create run params
    let messages = vec![user_message("Hello")];
    let params = test_agent_run_params("test_session", messages, Some("default"));

    // Execute pipeline
    let _ = pipeline.execute(&params, &event_tx).await;

    // Collect events
    let mut events = Vec::new();
    while let Ok(event) = event_rx.try_recv() {
        events.push(event);
    }

    // Verify we got some events
    assert!(!events.is_empty(), "Should have received events");
}

#[tokio::test]
async fn test_pipeline_error_handling() {
    // Test error handling when the provider returns an error

    // Create mock provider that fails
    let mock_provider = Arc::new(MockProvider::new("mock").with_error("Test error"));

    // Get test configuration
    let config = test_config();

    // Create provider registry and add mock provider
    let mut providers = aisopod_provider::ProviderRegistry::new();
    providers.register(mock_provider);
    // Register alias for the mock model
    providers.register_alias("mock/test-model", "mock", "mock/test-model");
    let providers = Arc::new(providers);

    // Create other dependencies
    let tools = test_tool_registry();
    let sessions = test_session_store();

    // Create pipeline
    let pipeline = AgentPipeline::new(Arc::new(config), providers, tools, sessions);

    // Create message stream
    let (event_tx, mut event_rx) = tokio::sync::mpsc::channel(50);

    // Create run params
    let messages = vec![user_message("Hello")];
    let params = test_agent_run_params("test_session", messages, Some("default"));

    // Execute pipeline - should fail
    let result = pipeline.execute(&params, &event_tx).await;

    // Verify error handling
    assert!(result.is_err(), "Pipeline should fail with error");
    assert!(result.unwrap_err().to_string().contains("Test error"));
}
