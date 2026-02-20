//! Agent execution types for the aisopod-agent crate.
//!
//! This module defines the core types used for agent execution,
//! including parameters, results, events, and usage reporting.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Session metadata for matching agents to sessions.
///
/// Contains information about a session that can be used to determine
/// which agent should handle it, such as channel, account, peer, and guild information.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionMetadata {
    /// The channel ID associated with the session.
    #[serde(default)]
    pub channel: Option<String>,
    /// The account ID associated with the session.
    #[serde(default)]
    pub account_id: Option<String>,
    /// The peer ID associated with the session.
    #[serde(default)]
    pub peer: Option<String>,
    /// The guild ID associated with the session.
    #[serde(default)]
    pub guild_id: Option<String>,
}

impl SessionMetadata {
    /// Creates a new SessionMetadata with all fields set to None.
    pub fn new() -> Self {
        Self {
            channel: None,
            account_id: None,
            peer: None,
            guild_id: None,
        }
    }

    /// Creates a new SessionMetadata with the given channel.
    pub fn with_channel(channel: impl Into<String>) -> Self {
        Self {
            channel: Some(channel.into()),
            ..Self::new()
        }
    }

    /// Creates a new SessionMetadata with the given account ID.
    pub fn with_account_id(account_id: impl Into<String>) -> Self {
        Self {
            account_id: Some(account_id.into()),
            ..Self::new()
        }
    }

    /// Creates a new SessionMetadata with the given peer.
    pub fn with_peer(peer: impl Into<String>) -> Self {
        Self {
            peer: Some(peer.into()),
            ..Self::new()
        }
    }

    /// Creates a new SessionMetadata with the given guild ID.
    pub fn with_guild_id(guild_id: impl Into<String>) -> Self {
        Self {
            guild_id: Some(guild_id.into()),
            ..Self::new()
        }
    }
}

/// Record of a tool call made during agent execution.
///
/// This type captures the essential information about a tool call
/// including its identifier, name, and arguments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRecord {
    /// The unique identifier for this tool call.
    pub id: String,
    /// The name of the tool being called.
    pub name: String,
    /// The JSON string arguments for the tool.
    pub arguments: String,
}

impl ToolCallRecord {
    /// Creates a new `ToolCallRecord` with the given id, name, and arguments.
    pub fn new(id: impl Into<String>, name: impl Into<String>, arguments: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            arguments: arguments.into(),
        }
    }
}

/// Parameters for running an agent.
///
/// Contains the session key, message history, and optional agent ID
/// needed to execute an agent run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRunParams {
    /// The session key identifying the current conversation session.
    pub session_key: String,
    /// The list of messages in the conversation.
    pub messages: Vec<aisopod_provider::Message>,
    /// The unique identifier of the agent to run, if known.
    pub agent_id: Option<String>,
    /// The current depth of agent spawning (for recursion limits).
    #[serde(default)]
    pub depth: usize,
}

impl AgentRunParams {
    /// Creates a new `AgentRunParams` with the given session key, messages, and optional agent ID.
    pub fn new(session_key: impl Into<String>, messages: Vec<aisopod_provider::Message>, agent_id: Option<impl Into<String>>) -> Self {
        Self {
            session_key: session_key.into(),
            messages,
            agent_id: agent_id.map(|id| id.into()),
            depth: 0,
        }
    }

    /// Creates a new `AgentRunParams` with the given session key, messages, agent ID, and depth.
    pub fn with_depth(
        session_key: impl Into<String>,
        messages: Vec<aisopod_provider::Message>,
        agent_id: Option<impl Into<String>>,
        depth: usize,
    ) -> Self {
        Self {
            session_key: session_key.into(),
            messages,
            agent_id: agent_id.map(|id| id.into()),
            depth,
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
    /// List of tool calls that were made during the run.
    pub tool_calls: Vec<ToolCallRecord>,
    /// Usage statistics for the run.
    pub usage: UsageReport,
}

impl AgentRunResult {
    /// Creates a new `AgentRunResult` with the given response, tool calls, and usage.
    pub fn new(
        response: impl Into<String>,
        tool_calls: Vec<ToolCallRecord>,
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
    pub input_tokens: u64,
    /// The number of output tokens (completion tokens).
    pub output_tokens: u64,
    /// The total number of tokens used.
    #[serde(default)]
    pub total_tokens: u64,
    /// The number of requests made.
    #[serde(default)]
    pub request_count: u64,
}

impl UsageReport {
    /// Creates a new `UsageReport` with the given token counts.
    pub fn new(input_tokens: u64, output_tokens: u64) -> Self {
        let total_tokens = input_tokens + output_tokens;
        Self {
            input_tokens,
            output_tokens,
            total_tokens,
            request_count: 0,
        }
    }

    /// Adds the given token counts to this report.
    pub fn add(&mut self, input: u64, output: u64) {
        self.input_tokens += input;
        self.output_tokens += output;
        self.total_tokens = self.input_tokens + self.output_tokens;
        self.request_count += 1;
    }
}

impl Default for UsageReport {
    fn default() -> Self {
        Self {
            input_tokens: 0,
            output_tokens: 0,
            total_tokens: 0,
            request_count: 0,
        }
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
        text: String,
        /// Optional index of the message in the stream.
        #[serde(default)]
        index: Option<usize>,
    },
    /// A tool call has started.
    ToolCallStart {
        /// The tool name being called.
        tool_name: String,
        /// The unique identifier for this tool call.
        call_id: String,
    },
    /// A tool call has completed with a result.
    ToolCallResult {
        /// The unique identifier of the tool call.
        call_id: String,
        /// The result of the tool execution.
        result: String,
        /// Whether the tool execution resulted in an error.
        is_error: bool,
    },
    /// The model is switching to a different model.
    ModelSwitch {
        /// The previous model ID.
        from: String,
        /// The new model ID.
        to: String,
        /// The reason for the model switch.
        reason: String,
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

/// Schema definition for a tool.
///
/// This type is used for documenting tool capabilities in system prompts.
/// It contains the essential information about a tool: its name, description,
/// and parameter schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    /// The unique name of the tool.
    pub name: String,
    /// A human-readable description of what the tool does.
    pub description: String,
    /// The JSON Schema defining the tool's expected parameters.
    pub parameters: Value,
}

impl ToolSchema {
    /// Creates a new ToolSchema with the given name, description, and parameters.
    pub fn new(name: impl Into<String>, description: impl Into<String>, parameters: Value) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parameters,
        }
    }
}
