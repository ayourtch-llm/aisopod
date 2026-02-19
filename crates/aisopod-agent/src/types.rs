//! Agent execution types for the aisopod-agent crate.
//!
//! This module defines the core types used for agent execution,
//! including parameters, results, events, and usage reporting.

use serde::{Deserialize, Serialize};

/// Parameters for running an agent.
///
/// Contains the session key, message history, and agent ID
/// needed to execute an agent run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRunParams {
    /// The session key identifying the current conversation session.
    pub session_key: String,
    /// The list of messages in the conversation.
    pub messages: Vec<aisopod_provider::Message>,
    /// The unique identifier of the agent to run.
    pub agent_id: String,
}

impl AgentRunParams {
    /// Creates a new `AgentRunParams` with the given session key, messages, and agent ID.
    pub fn new(session_key: impl Into<String>, messages: Vec<aisopod_provider::Message>, agent_id: impl Into<String>) -> Self {
        Self {
            session_key: session_key.into(),
            messages,
            agent_id: agent_id.into(),
        }
    }
}

/// The result of an agent run.
///
/// Contains the response content, any tool calls made, and usage statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRunResult {
    /// The response content from the agent.
    pub response: String,
    /// Optional list of tool calls that were made during the run.
    pub tool_calls: Option<Vec<aisopod_provider::ToolCall>>,
    /// Usage statistics for the run.
    pub usage: UsageReport,
}

impl AgentRunResult {
    /// Creates a new `AgentRunResult` with the given response, tool calls, and usage.
    pub fn new(
        response: impl Into<String>,
        tool_calls: Option<Vec<aisopod_provider::ToolCall>>,
        usage: UsageReport,
    ) -> Self {
        Self {
            response: response.into(),
            tool_calls,
            usage,
        }
    }
}

/// Usage statistics for an agent run.
///
/// Contains the number of input and output tokens consumed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageReport {
    /// The number of input tokens (prompt tokens).
    pub input_tokens: u32,
    /// The number of output tokens (completion tokens).
    pub output_tokens: u32,
}

impl UsageReport {
    /// Creates a new `UsageReport` with the given token counts.
    pub fn new(input_tokens: u32, output_tokens: u32) -> Self {
        Self {
            input_tokens,
            output_tokens,
        }
    }

    /// Returns the total number of tokens used.
    pub fn total_tokens(&self) -> u32 {
        self.input_tokens + self.output_tokens
    }
}

/// Events emitted during agent execution.
///
/// This enum represents the various events that can occur during
/// an agent's execution, from streaming text deltas to tool calls
/// and final completion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentEvent {
    /// A delta of text in the streaming response.
    TextDelta {
        /// The text delta content.
        delta: String,
        /// Optional index of the message in the stream.
        #[serde(default)]
        index: Option<usize>,
    },
    /// A tool call has started.
    ToolCallStart {
        /// The tool call that started.
        tool_call: aisopod_provider::ToolCall,
    },
    /// A tool call has completed with a result.
    ToolCallResult {
        /// The tool call that completed.
        tool_call: aisopod_provider::ToolCall,
        /// The result of the tool execution.
        result: aisopod_tools::ToolResult,
    },
    /// The model is switching to a different model.
    ModelSwitch {
        /// The previous model ID.
        from: String,
        /// The new model ID.
        to: String,
    },
    /// An error occurred during agent execution.
    Error {
        /// The error message.
        message: String,
    },
    /// The agent run has completed successfully.
    Complete {
        /// The final result of the run.
        result: AgentRunResult,
    },
    /// Usage statistics for the run.
    Usage {
        /// The usage report.
        usage: UsageReport,
    },
}
