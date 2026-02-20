//! Pipeline-related tests for agent engine.
//!
//! This module tests the agent execution pipeline components.

use aisopod_agent::pipeline::{AgentRunStream, AgentPipeline};
use std::sync::Arc;

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
    
    let pipeline = AgentPipeline::new(
        _config,
        _providers,
        _tools,
        _sessions,
    );
    
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
    let sessions = Arc::new(aisopod_session::SessionStore::new());
    
    (config, providers, tools, sessions)
}
