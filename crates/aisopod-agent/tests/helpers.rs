//! Test helpers for agent engine testing.
//!
//! This module provides mock infrastructure for testing agent engine
//! implementations without making real HTTP calls or using actual external services.

use anyhow::Result;
use std::sync::{Arc, Mutex};

use aisopod_agent::types::{
    AgentEvent, AgentRunParams, AgentRunResult, SessionMetadata, ToolCallRecord, UsageReport,
};
use aisopod_agent::{AbortHandle, AbortRegistry};
use aisopod_config::AisopodConfig;
use aisopod_provider::trait_module::ModelProvider;
use aisopod_provider::{
    ChatCompletionRequest, ChatCompletionStream, Message, MessageContent, Role, ToolDefinition,
};
use aisopod_session::SessionStore;
use aisopod_tools::{Tool, ToolContext, ToolRegistry, ToolResult};

/// Alias for test results
pub type TestResult<T = ()> = Result<T>;

// ============================================================================
// Mock Provider Implementation
// ============================================================================

/// A mock provider implementation for testing without real HTTP calls.
///
/// This provider simulates various behaviors through configuration:
/// - Success responses with configurable text or tool calls
/// - Error responses
/// - Custom model lists
pub struct MockProvider {
    id: String,
    response_text: Option<String>,
    tool_calls: Vec<aisopod_provider::ToolCall>,
    should_fail: bool,
    error_message: Option<String>,
    /// Track if tool calls have been returned (for stateful behavior)
    tool_calls_returned: std::sync::Arc<std::sync::Mutex<bool>>,
}

impl MockProvider {
    /// Creates a new mock provider with default success response.
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            response_text: Some("Test response".to_string()),
            tool_calls: Vec::new(),
            should_fail: false,
            error_message: None,
            tool_calls_returned: std::sync::Arc::new(std::sync::Mutex::new(false)),
        }
    }

    /// Sets the response text to return.
    pub fn with_response_text(mut self, text: impl Into<String>) -> Self {
        self.response_text = Some(text.into());
        self
    }

    /// Sets tool calls to return in the response.
    pub fn with_tool_calls(mut self, tool_calls: Vec<aisopod_provider::ToolCall>) -> Self {
        self.tool_calls = tool_calls;
        // Reset the flag for new test
        *self.tool_calls_returned.lock().unwrap() = false;
        self
    }

    /// Configures the mock to fail with an error.
    pub fn with_error(mut self, error_message: impl Into<String>) -> Self {
        self.should_fail = true;
        self.error_message = Some(error_message.into());
        self
    }

    /// Creates a tool call with the given id, name, and arguments.
    pub fn create_tool_call(
        id: impl Into<String>,
        name: impl Into<String>,
        arguments: impl Into<String>,
    ) -> aisopod_provider::ToolCall {
        aisopod_provider::ToolCall {
            id: id.into(),
            name: name.into(),
            arguments: arguments.into(),
        }
    }
}

#[async_trait::async_trait]
impl ModelProvider for MockProvider {
    fn id(&self) -> &str {
        &self.id
    }

    async fn list_models(&self) -> Result<Vec<aisopod_provider::types::ModelInfo>> {
        Ok(vec![aisopod_provider::types::ModelInfo {
            id: format!("{}/test-model", self.id),
            name: "Test Model".to_string(),
            provider: self.id.clone(),
            context_window: 128000,
            supports_vision: false,
            supports_tools: !self.tool_calls.is_empty(),
        }])
    }

    async fn chat_completion(
        &self,
        _request: ChatCompletionRequest,
    ) -> Result<ChatCompletionStream> {
        if self.should_fail {
            return Err(anyhow::anyhow!(self
                .error_message
                .clone()
                .unwrap_or_else(|| "Mock error".to_string())));
        }

        // Create a simple stream with the response
        let response_text = self.response_text.clone().unwrap_or_default();

        // Check if we should return tool calls or text response
        // If tool_calls is not empty and we haven't returned them yet, return tool calls
        // Otherwise, return text response (this handles the case after tool calls are processed)
        let should_return_tool_calls = {
            let mut returned = self.tool_calls_returned.lock().unwrap();
            if !self.tool_calls.is_empty() && !*returned {
                *returned = true;
                true
            } else {
                false
            }
        };

        let tool_calls = if should_return_tool_calls {
            self.tool_calls.clone()
        } else {
            Vec::new()
        };

        let chunks = MockProvider::create_stream_chunks(&response_text, &tool_calls);

        let stream = async_stream::stream! {
            for chunk in chunks {
                yield Ok(chunk);
            }
        };

        Ok(Box::pin(stream))
    }

    async fn health_check(&self) -> Result<aisopod_provider::types::ProviderHealth> {
        Ok(aisopod_provider::types::ProviderHealth {
            available: !self.should_fail,
            latency_ms: Some(10),
        })
    }
}

impl MockProvider {
    fn create_stream_chunks(
        response_text: &str,
        tool_calls: &[aisopod_provider::ToolCall],
    ) -> Vec<aisopod_provider::types::ChatCompletionChunk> {
        let mut chunks = Vec::new();

        // Add content chunks for text response
        if !response_text.is_empty() {
            chunks.push(aisopod_provider::types::ChatCompletionChunk {
                id: "chunk_1".to_string(),
                delta: aisopod_provider::types::MessageDelta {
                    role: Some(Role::Assistant),
                    content: Some(response_text.to_string()),
                    tool_calls: None,
                },
                finish_reason: if tool_calls.is_empty() {
                    Some(aisopod_provider::types::FinishReason::Stop)
                } else {
                    None
                },
                usage: if tool_calls.is_empty() {
                    Some(aisopod_provider::types::TokenUsage {
                        prompt_tokens: 10,
                        completion_tokens: (response_text.len() / 4) as u32,
                        total_tokens: (10 + response_text.len() / 4) as u32,
                    })
                } else {
                    None
                },
            });
        }

        // Add tool call chunks
        for (i, tool_call) in tool_calls.iter().enumerate() {
            chunks.push(aisopod_provider::types::ChatCompletionChunk {
                id: format!("chunk_{}", i + 2),
                delta: aisopod_provider::types::MessageDelta {
                    role: None,
                    content: None,
                    tool_calls: Some(vec![tool_call.clone()]),
                },
                finish_reason: None,
                usage: None,
            });
        }

        // Add final chunk with usage if there were tool calls
        if !tool_calls.is_empty() {
            chunks.push(aisopod_provider::types::ChatCompletionChunk {
                id: "chunk_final".to_string(),
                delta: aisopod_provider::types::MessageDelta {
                    role: None,
                    content: None,
                    tool_calls: None,
                },
                finish_reason: Some(aisopod_provider::types::FinishReason::Stop),
                usage: Some(aisopod_provider::types::TokenUsage {
                    prompt_tokens: 10,
                    completion_tokens: tool_calls.len() as u32 * 5,
                    total_tokens: (10 + tool_calls.len() * 5) as u32,
                }),
            });
        }

        chunks
    }
}

// ============================================================================
// Mock Tool Implementation
// ============================================================================

/// A mock tool implementation for testing.
pub struct MockTool {
    name: String,
    result: Mutex<Option<String>>,
    error: Mutex<Option<String>>,
}

impl MockTool {
    /// Creates a new mock tool with the given name and result.
    pub fn new(name: impl Into<String>, result: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            result: Mutex::new(Some(result.into())),
            error: Mutex::new(None),
        }
    }

    /// Configures the tool to return an error on execution.
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        *self.error.lock().unwrap() = Some(error.into());
        self
    }
}

#[async_trait::async_trait]
impl Tool for MockTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "A mock tool for testing"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {},
            "required": []
        })
    }

    async fn execute(&self, _params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult> {
        if let Some(ref error) = *self.error.lock().unwrap() {
            return Ok(ToolResult {
                content: error.clone(),
                is_error: true,
                metadata: None,
            });
        }

        Ok(ToolResult {
            content: self.result.lock().unwrap().clone().unwrap_or_default(),
            is_error: false,
            metadata: None,
        })
    }
}

// ============================================================================
// Configuration Helpers
// ============================================================================

/// Creates a test configuration with a single agent and default settings.
pub fn test_config() -> AisopodConfig {
    AisopodConfig {
        agents: aisopod_config::types::AgentsConfig {
            default: aisopod_config::types::AgentDefaults {
                model: "mock/test-model".to_string(),
                workspace: String::new(),
                sandbox: false,
            },
            agents: vec![
                aisopod_config::types::Agent {
                    id: "default".to_string(),
                    name: String::new(),
                    model: "mock/test-model".to_string(),
                    workspace: String::new(),
                    sandbox: false,
                    subagents: Vec::new(),
                    system_prompt: "You are a helpful assistant.".to_string(),
                    max_subagent_depth: 3,
                    subagent_allowed_models: None,
                    skills: Vec::new(),
                },
                aisopod_config::types::Agent {
                    id: "test-agent".to_string(),
                    name: String::new(),
                    model: "mock/test-model".to_string(),
                    workspace: String::new(),
                    sandbox: false,
                    subagents: Vec::new(),
                    system_prompt: "You are a test agent.".to_string(),
                    max_subagent_depth: 3,
                    subagent_allowed_models: None,
                    skills: Vec::new(),
                },
                aisopod_config::types::Agent {
                    id: "fallback-agent".to_string(),
                    name: String::new(),
                    model: "mock/fallback-model".to_string(),
                    workspace: String::new(),
                    sandbox: false,
                    subagents: Vec::new(),
                    system_prompt: "You are a fallback agent.".to_string(),
                    max_subagent_depth: 3,
                    subagent_allowed_models: None,
                    skills: Vec::new(),
                },
            ],
        },
        models: aisopod_config::types::ModelsConfig {
            models: vec![],
            providers: vec![],
            fallbacks: vec![aisopod_config::types::ModelFallback {
                primary: "mock/test-model".to_string(),
                fallbacks: vec!["mock/fallback-model".to_string()],
            }],
            default_provider: String::new(),
        },
        bindings: vec![aisopod_config::types::AgentBinding {
            agent_id: "test-agent".to_string(),
            channels: vec![],
            priority: 100,
        }],
        ..Default::default()
    }
}

/// Creates a test configuration with multiple models for failover testing.
pub fn test_config_with_fallbacks() -> AisopodConfig {
    let mut config = test_config();

    config.models.fallbacks = vec![aisopod_config::types::ModelFallback {
        primary: "mock/test-model".to_string(),
        fallbacks: vec![
            "mock/fallback-model".to_string(),
            "mock/another-model".to_string(),
        ],
    }];

    config.agents.agents.push(aisopod_config::types::Agent {
        id: "fallback-agent".to_string(),
        name: String::new(),
        model: "mock/fallback-model".to_string(),
        workspace: String::new(),
        sandbox: false,
        subagents: Vec::new(),
        system_prompt: "You are a fallback agent.".to_string(),
        max_subagent_depth: 3,
        subagent_allowed_models: None,
        skills: Vec::new(),
    });

    config
}

// ============================================================================
// Session Store Helpers
// ============================================================================

/// Creates a test session store using an in-memory database.
pub fn test_session_store() -> Arc<SessionStore> {
    Arc::new(SessionStore::new_in_memory().expect("Failed to create in-memory session store"))
}

// ============================================================================
// Tool Registry Helpers
// ============================================================================

/// Creates a test tool registry with a mock calculator tool.
pub fn test_tool_registry() -> Arc<ToolRegistry> {
    let mut registry = ToolRegistry::new();

    let calculator = Arc::new(MockTool::new("calculator", "100"));

    registry.register(calculator);

    Arc::new(registry)
}

// ============================================================================
// Agent Event Stream Helpers
// ============================================================================

/// Collects all agent events from a channel into a vector.
pub async fn collect_events(mut rx: tokio::sync::mpsc::Receiver<AgentEvent>) -> Vec<AgentEvent> {
    let mut events = Vec::new();
    while let Ok(event) = rx.try_recv() {
        events.push(event);
    }
    events
}

// ============================================================================
// Agent Run Result Helpers
// ============================================================================

/// Creates a test agent run result with the given response and tool calls.
pub fn test_agent_run_result(
    response: impl Into<String>,
    tool_calls: Vec<ToolCallRecord>,
    input_tokens: u64,
    output_tokens: u64,
) -> AgentRunResult {
    AgentRunResult {
        response: response.into(),
        tool_calls,
        usage: UsageReport::new(input_tokens, output_tokens),
    }
}

// ============================================================================
// Tool Call Helpers
// ============================================================================

/// Creates a tool call record with the given id, name, and arguments.
pub fn test_tool_call_record(
    id: impl Into<String>,
    name: impl Into<String>,
    arguments: impl Into<String>,
) -> ToolCallRecord {
    ToolCallRecord::new(id, name, arguments)
}

// ============================================================================
// Tool Definition Helpers
// ============================================================================

/// Creates a test tool definition for a calculator tool.
pub fn test_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: "calculator".to_string(),
        description: "A calculator tool for arithmetic operations".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "description": "The operation to perform"
                },
                "a": {
                    "type": "number",
                    "description": "First operand"
                },
                "b": {
                    "type": "number",
                    "description": "Second operand"
                }
            },
            "required": ["operation", "a", "b"]
        }),
    }
}

// ============================================================================
// Message Helpers
// ============================================================================

/// Creates a user message with the given content.
pub fn user_message(content: impl Into<String>) -> Message {
    Message {
        role: Role::User,
        content: MessageContent::Text(content.into()),
        tool_calls: None,
        tool_call_id: None,
    }
}

/// Creates an assistant message with the given content.
pub fn assistant_message(content: impl Into<String>) -> Message {
    Message {
        role: Role::Assistant,
        content: MessageContent::Text(content.into()),
        tool_calls: None,
        tool_call_id: None,
    }
}

/// Creates a system message with the given content.
pub fn system_message(content: impl Into<String>) -> Message {
    Message {
        role: Role::System,
        content: MessageContent::Text(content.into()),
        tool_calls: None,
        tool_call_id: None,
    }
}

/// Creates a tool message with the given id and content.
pub fn tool_message(tool_call_id: impl Into<String>, content: impl Into<String>) -> Message {
    Message {
        role: Role::Tool,
        content: MessageContent::Text(content.into()),
        tool_calls: None,
        tool_call_id: Some(tool_call_id.into()),
    }
}

// ============================================================================
// Agent Run Params Helpers
// ============================================================================

/// Creates agent run params with the given session key, messages, and optional agent ID.
pub fn test_agent_run_params(
    session_key: impl Into<String>,
    messages: Vec<Message>,
    agent_id: Option<impl Into<String>>,
) -> AgentRunParams {
    AgentRunParams::new(session_key, messages, agent_id)
}

/// Creates agent run params with the given session key, messages, and agent ID.
pub fn test_agent_run_params_with_id(
    session_key: impl Into<String>,
    messages: Vec<Message>,
    agent_id: impl Into<String>,
) -> AgentRunParams {
    AgentRunParams::new(session_key, messages, Some(agent_id))
}

// ============================================================================
// Abort Registry Helpers
// ============================================================================

/// Creates a test abort registry.
pub fn test_abort_registry() -> Arc<AbortRegistry> {
    Arc::new(AbortRegistry::new())
}

/// Creates an abort handle for the given session key.
pub fn test_abort_handle(session_key: impl Into<String>) -> AbortHandle {
    AbortHandle::new(session_key.into())
}
