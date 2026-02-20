//! Agent execution pipeline for aisopod-agent.
//!
//! This module implements the core agent execution loop that ties together
//! agent resolution, model selection, tool preparation, system prompt construction,
//! transcript repair, model calling, tool call handling, and event streaming.

use std::sync::Arc;

use anyhow::Result;
use tokio::sync::mpsc;
use futures_util::StreamExt;

use crate::resolution::{resolve_agent_config, resolve_agent_model, resolve_session_agent_id, ModelChain};
use crate::types::{AgentEvent, AgentRunParams, AgentRunResult, ToolCallRecord, UsageReport};
use crate::{failover, prompt, transcript};
use aisopod_provider::ToolDefinition;

/// A stream of agent events from an agent run.
///
/// This is a wrapper around the `mpsc::Receiver<AgentEvent>` that provides
/// an ergonomic interface for consumers of agent event streams.
pub struct AgentRunStream {
    receiver: mpsc::Receiver<AgentEvent>,
}

impl AgentRunStream {
    /// Creates a new AgentRunStream from an mpsc receiver.
    pub fn new(receiver: mpsc::Receiver<AgentEvent>) -> Self {
        Self { receiver }
    }

    /// Returns the underlying receiver.
    pub fn receiver(&self) -> &mpsc::Receiver<AgentEvent> {
        &self.receiver
    }

    /// Consumes the stream and returns the receiver.
    pub fn into_receiver(self) -> mpsc::Receiver<AgentEvent> {
        self.receiver
    }
}

/// The execution pipeline for running an agent.
///
/// This struct encapsulates the full pipeline of agent execution:
/// - Agent ID resolution
/// - Agent configuration resolution
/// - Model chain resolution
/// - Tool preparation
/// - System prompt construction
/// - Transcript repair
/// - Model calling with tool call handling
/// - Event streaming
pub struct AgentPipeline {
    config: Arc<aisopod_config::AisopodConfig>,
    providers: Arc<aisopod_provider::ProviderRegistry>,
    tools: Arc<aisopod_tools::ToolRegistry>,
    sessions: Arc<aisopod_session::SessionStore>,
}

impl AgentPipeline {
    /// Creates a new AgentPipeline with the given dependencies.
    pub fn new(
        config: Arc<aisopod_config::AisopodConfig>,
        providers: Arc<aisopod_provider::ProviderRegistry>,
        tools: Arc<aisopod_tools::ToolRegistry>,
        sessions: Arc<aisopod_session::SessionStore>,
    ) -> Self {
        Self {
            config,
            providers,
            tools,
            sessions,
        }
    }

    /// Executes the full agent pipeline with the given parameters.
    ///
    /// This method:
    /// 1. Resolves the agent ID from the session
    /// 2. Resolves the agent configuration
    /// 3. Resolves the model chain (primary + fallbacks)
    /// 4. Prepares tool schemas for the agent
    /// 5. Builds the system prompt
    /// 6. Repairs the message transcript for the target provider
    /// 7. Calls the model in a loop, handling tool calls
    /// 8. Streams events to the provided sender
    ///
    /// # Arguments
    ///
    /// * `params` - The agent run parameters
    /// * `event_tx` - The channel sender for streaming events
    ///
    /// # Returns
    ///
    /// Returns the final result of the agent run, or an error if execution failed.
    pub async fn execute(
        &self,
        params: &AgentRunParams,
        event_tx: &mpsc::Sender<AgentEvent>,
    ) -> Result<AgentRunResult> {
        // 1. Resolve agent ID
        let agent_id = resolve_session_agent_id(&self.config, &params.session_key)?;

        // 2. Resolve agent config
        let agent_config = resolve_agent_config(&self.config, &agent_id)?;

        // 3. Resolve model chain (primary + fallbacks)
        let model_chain = resolve_agent_model(&self.config, &agent_id)?;

        // 4. Prepare tool schemas for the agent - convert to ToolDefinition
        let tool_definitions: Vec<ToolDefinition> = self.tools.schemas().iter()
            .filter_map(|s| {
                let name = s.get("name")?.as_str()?.to_string();
                let description = s.get("description")?.as_str()?.to_string();
                let parameters = s.get("parameters")?.clone();
                Some(ToolDefinition::new(name, description, parameters))
            })
            .collect();

        // 5. Build system prompt
        let system_prompt = self.build_system_prompt(&agent_config, &tool_definitions);

        // 6. Repair message transcript
        let provider_kind = self.determine_provider_kind(&model_chain);
        let messages = transcript::repair_transcript(&params.messages, provider_kind);

        // 7. Call model in a loop
        self.execute_model_loop(
            &agent_id,
            &model_chain,
            &system_prompt,
            messages,
            &tool_definitions,
            event_tx,
            params,
        )
        .await
    }

    /// Builds the system prompt from agent config and tool schemas.
    fn build_system_prompt(
        &self,
        agent_config: &aisopod_config::types::Agent,
        tool_definitions: &[ToolDefinition],
    ) -> String {
        let builder = prompt::SystemPromptBuilder::new()
            .with_base_prompt(&agent_config.system_prompt)
            .with_dynamic_context();

        // Convert ToolDefinition to ToolSchema type for the builder
        let schemas: Vec<crate::types::ToolSchema> = tool_definitions
            .iter()
            .map(|tool| {
                crate::types::ToolSchema::new(
                    &tool.name,
                    &tool.description,
                    tool.parameters.clone()
                )
            })
            .collect();

        builder.with_tool_descriptions(&schemas).build()
    }

    /// Determines the provider kind from the model chain.
    fn determine_provider_kind(&self, model_chain: &ModelChain) -> transcript::ProviderKind {
        // Extract provider from model ID (format: "provider/model")
        let primary = model_chain.primary();
        if primary.starts_with("anthropic/") {
            transcript::ProviderKind::Anthropic
        } else if primary.starts_with("openai/") {
            transcript::ProviderKind::OpenAI
        } else if primary.starts_with("google/") || primary.starts_with("gemini/") {
            transcript::ProviderKind::Google
        } else {
            transcript::ProviderKind::Other
        }
    }

    /// Executes the model call loop with tool call handling.
    #[allow(clippy::too_many_arguments)]
    async fn execute_model_loop(
        &self,
        agent_id: &str,
        model_chain: &ModelChain,
        system_prompt: &str,
        mut messages: Vec<aisopod_provider::Message>,
        tool_definitions: &[ToolDefinition],
        event_tx: &mpsc::Sender<AgentEvent>,
        params: &AgentRunParams,
    ) -> Result<AgentRunResult> {
        // Create failover state for tracking model attempts
        let mut failover_state = failover::FailoverState::new(model_chain);
        let mut usage = UsageReport::new(0, 0);
        let mut tool_calls: Vec<ToolCallRecord> = Vec::new();

        loop {
            // Get the current model ID for the request
            let current_model = failover_state.current_model().to_string();

            // Get the current provider and model
            let (provider, _) = self
                .providers
                .resolve_model(&current_model)
                .ok_or_else(|| anyhow::anyhow!("Model not found: {}", current_model))?;

            // Build the request
            let request = aisopod_provider::ChatCompletionRequest {
                model: current_model.clone(),
                messages: messages.clone(),
                tools: if tool_definitions.is_empty() {
                    None
                } else {
                    Some(tool_definitions.to_vec())
                },
                temperature: None,
                max_tokens: None,
                stop: None,
                stream: true,
            };
            let request_clone = request.clone();

            // Call model with failover support
            let response_stream = failover::execute_with_failover(
                &mut failover_state,
                event_tx.clone(),
                move |model_id: String| {
                    let provider = provider.clone();
                    let request = request_clone.clone();
                    async move {
                        let request = aisopod_provider::ChatCompletionRequest {
                            model: model_id,
                            ..request.clone()
                        };
                        provider.chat_completion(request).await.map_err(|e| e)
                    }
                },
            )
            .await?;

            // Process streaming response
            let mut response_text = String::new();
            let mut response_tool_calls: Vec<aisopod_provider::ToolCall> = Vec::new();

            let mut stream = response_stream;
            while let Some(chunk) = stream.next().await {
                let chunk = chunk?;

                // Emit text delta events
                if let Some(ref content) = chunk.delta.content {
                    let _ = event_tx.send(AgentEvent::TextDelta {
                        text: content.clone(),
                        index: None, // TODO: track message index
                    }).await;
                    response_text.push_str(content);
                }

                // Collect tool calls
                if let Some(ref tool_calls_chunk) = chunk.delta.tool_calls {
                    response_tool_calls.extend(tool_calls_chunk.clone());
                }

                // Aggregate usage
                if let Some(ref u) = chunk.usage {
                    usage = UsageReport::new(
                        u.prompt_tokens as u64,
                        u.completion_tokens as u64,
                    );
                }
            }

            // Check if there are tool calls
            if response_tool_calls.is_empty() {
                // No tool calls - we're done
                let result = AgentRunResult::new(
                    response_text.clone(),
                    tool_calls.clone(),
                    usage,
                );
                let _ = event_tx.send(AgentEvent::Complete { result: result.clone() }).await;
                return Ok(result);
            }

            // Process tool calls
            for tool_call in response_tool_calls {
                let _ = event_tx.send(AgentEvent::ToolCallStart {
                    tool_name: tool_call.name.clone(),
                    call_id: tool_call.id.clone(),
                }).await;

                // Execute the tool
                let tool_result = self.execute_tool(&tool_call, agent_id, &params.session_key).await?;

                let result_content = tool_result.content.clone();
                let _ = event_tx.send(AgentEvent::ToolCallResult {
                    call_id: tool_call.id.clone(),
                    result: tool_result.content,
                    is_error: tool_result.is_error,
                }).await;

                // Add tool result to messages
                messages.push(aisopod_provider::Message {
                    role: aisopod_provider::Role::Assistant,
                    content: aisopod_provider::MessageContent::Text(response_text.clone()),
                    tool_calls: Some(vec![aisopod_provider::ToolCall {
                        id: tool_call.id.clone(),
                        name: tool_call.name.clone(),
                        arguments: tool_call.arguments.clone(),
                    }]),
                    tool_call_id: None,
                });

                messages.push(aisopod_provider::Message {
                    role: aisopod_provider::Role::Tool,
                    content: aisopod_provider::MessageContent::Text(result_content),
                    tool_calls: None,
                    tool_call_id: Some(tool_call.id),
                });
            }

            // Continue the loop with updated messages
        }
    }

    /// Executes a tool and returns the result.
    async fn execute_tool(
        &self,
        tool_call: &aisopod_provider::ToolCall,
        agent_id: &str,
        session_key: &str,
    ) -> Result<aisopod_tools::ToolResult> {
        let tool_name = &tool_call.name;
        let params: serde_json::Value = serde_json::from_str(&tool_call.arguments)?;

        let tool = self.tools.get(tool_name).ok_or_else(|| {
            anyhow::anyhow!("Tool not found: {}", tool_name)
        })?;

        let ctx = aisopod_tools::ToolContext::new(agent_id, session_key);

        tool.execute(params, &ctx).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_run_stream_new() {
        let (tx, rx) = mpsc::channel(10);
        let stream = AgentRunStream::new(rx);
        // Capacity returns usize, not Option<usize>
        assert!(stream.receiver().capacity() > 0);
        drop(tx); // Clean up
    }

    #[test]
    fn test_agent_run_stream_into_receiver() {
        let (tx, rx) = mpsc::channel(10);
        let stream = AgentRunStream::new(rx);
        let mut receiver = stream.into_receiver();
        assert!(receiver.try_recv().is_err());
        drop(tx); // Clean up
    }
}
