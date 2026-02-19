//! Agent execution pipeline for orchestrating agent runs.
//!
//! This module provides the `AgentPipeline` struct which implements
//! the full agent execution loop: resolving agents, selecting models,
//! preparing tools, building system prompts, repairing transcripts,
//! calling models, handling tool calls, and streaming events.

use std::sync::Arc;

use anyhow::Result;
use futures_util::stream::StreamExt;
use tokio::sync::mpsc;

use crate::failover::{classify_error, execute_with_failover, FailoverAction, FailoverState};
use crate::types::{AgentEvent, AgentRunParams, AgentRunResult, ToolSchema, UsageReport};
use crate::{resolution, transcript, SystemPromptBuilder};

/// The core pipeline for agent execution.
///
/// `AgentPipeline` implements the full agent execution loop:
/// - Resolve agent configuration
/// - Select model from model chain
/// - Prepare tools for the agent
/// - Build system prompt
/// - Repair transcript for provider requirements
/// - Call model with messages and tools
/// - Handle tool call responses
/// - Stream events to subscribers
/// - Return final result
pub struct AgentPipeline {
    /// The agent configuration.
    config: Arc<aisopod_config::AisopodConfig>,
    /// The provider registry for model access.
    providers: Arc<aisopod_provider::ProviderRegistry>,
    /// The tool registry for tool execution.
    tools: Arc<aisopod_tools::ToolRegistry>,
    /// The session store for conversation state.
    sessions: Arc<aisopod_session::SessionStore>,
}

impl AgentPipeline {
    /// Creates a new `AgentPipeline` with the given dependencies.
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

    /// Executes the full agent pipeline with the given parameters and event sender.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters for the agent run.
    /// * `event_tx` - The sender for streaming events to subscribers.
    ///
    /// # Returns
    ///
    /// Returns the result of the agent run on success, or an error if
    /// the run failed.
    pub async fn execute(
        &self,
        params: &AgentRunParams,
        event_tx: &mpsc::UnboundedSender<AgentEvent>,
    ) -> Result<AgentRunResult> {
        // Step 1: Resolve agent configuration
        let agent_config = resolution::resolve_agent_config(&self.config, &params.agent_id)?;

        // Step 2: Resolve model chain
        let model_chain = resolution::resolve_agent_model(&self.config, &params.agent_id)?;

        // Step 3: Resolve the primary model provider
        let (provider, model_id) = self
            .providers
            .resolve_model(model_chain.primary())
            .ok_or_else(|| anyhow::anyhow!("Failed to resolve model: {}", model_chain.primary()))?;

        // Step 4: Get tools for the agent
        let agent_tools = self.get_agent_tools()?;

        // Step 5: Build system prompt
        let system_prompt = self.build_system_prompt(&agent_config, &agent_tools)?;

        // Step 6: Prepare message history with system prompt
        let mut messages = self.prepare_messages(&params.messages, &system_prompt);

        // Step 7: Repair transcript for provider requirements
        let provider_kind = self.get_provider_kind(&provider);
        messages = transcript::repair_transcript(&messages, provider_kind);

        // Step 8: Initialize failover state
        let mut failover_state = FailoverState::new(&model_chain);

        // Step 9: Main execution loop with failover
        loop {
            // Call model with failover - we pass an async closure that calls call_model_async
            let result = execute_with_failover(
                &mut failover_state,
                &mut |event| {
                    let _ = event_tx.send(event);
                },
                |model_id| {
                    let provider_clone = provider.clone();
                    let tools = agent_tools.clone();
                    let messages_clone = messages.clone();
                    
                    // Return an async block that will be awaited
                    async move {
                        Self::call_model_async(&provider_clone, model_id, &messages_clone, &tools).await
                    }
                },
            )
            .await;

            match result {
                Ok((response, tool_calls)) => {
                    // Check if model returned text response (no tool calls)
                    if tool_calls.is_empty() {
                        // Send final response and complete event
                        for delta in response.split(' ') {
                            let _ = event_tx.send(AgentEvent::TextDelta {
                                delta: format!("{} ", delta),
                                index: None,
                            });
                        }
                        let result = AgentRunResult::new(
                            response,
                            None,
                            UsageReport::new(0, 0), // TODO: Extract actual usage
                        );
                        let _ = event_tx.send(AgentEvent::Complete { result: result.clone() });
                        return Ok(result);
                    }

                    // Handle tool calls
                    let mut all_tool_calls = Vec::new();
                    for tool_call in &tool_calls {
                        // Send tool call start event
                        let _ = event_tx.send(AgentEvent::ToolCallStart {
                            tool_call: tool_call.clone(),
                        });

                        // Execute tool
                        let tool_result = self.execute_tool(tool_call, &params.session_key).await?;

                        // Send tool call result event
                        let _ = event_tx.send(AgentEvent::ToolCallResult {
                            tool_call: tool_call.clone(),
                            result: tool_result,
                        });

                        all_tool_calls.push(tool_call.clone());
                    }

                    // Append tool results to message history
                    messages.push(aisopod_provider::Message {
                        role: aisopod_provider::Role::Assistant,
                        content: aisopod_provider::MessageContent::Text(String::new()),
                        tool_calls: Some(all_tool_calls.clone()),
                        tool_call_id: None,
                    });

                    // Add tool results to messages
                    for tool_call in &all_tool_calls {
                        let tool_result = self.get_tool_result(tool_call, &params.session_key).await?;
                        messages.push(aisopod_provider::Message {
                            role: aisopod_provider::Role::Tool,
                            content: aisopod_provider::MessageContent::Text(tool_result.content.clone()),
                            tool_calls: None,
                            tool_call_id: Some(tool_call.id.clone()),
                        });
                    }
                }
                Err(msg) => {
                    // Handle failover errors
                    if msg.contains("Context length exceeded") {
                        // Compact messages and retry - for now, we'll just log and continue
                        // In a full implementation, we'd implement message compaction
                        let _ = event_tx.send(AgentEvent::Error {
                            message: format!("Context length exceeded. Could not compact messages enough to retry. Last error: {}", msg),
                        });
                        return Err(anyhow::anyhow!("All models exhausted or failed"));
                    } else {
                        // Other error - return it
                        let _ = event_tx.send(AgentEvent::Error {
                            message: msg.clone(),
                        });
                        return Err(anyhow::anyhow!("{}", msg));
                    }
                }
            }
        }
    }

    /// Calls the model with messages and tools - async version for failover.
    async fn call_model_async(
        provider: &Arc<dyn aisopod_provider::trait_module::ModelProvider>,
        model_id: &str,
        messages: &[aisopod_provider::Message],
        tools: &[ToolSchema],
    ) -> Result<(String, Vec<aisopod_provider::ToolCall>), aisopod_provider::normalize::ProviderError> {
        // Convert tool schemas to tool definitions
        let tool_defs: Vec<aisopod_provider::ToolDefinition> = tools
            .iter()
            .map(|t| aisopod_provider::ToolDefinition {
                name: t.name.clone(),
                description: t.description.clone(),
                parameters: t.parameters.clone(),
            })
            .collect();

        // Create completion request
        let request = aisopod_provider::ChatCompletionRequest {
            model: model_id.to_string(),
            messages: messages.to_vec(),
            tools: Some(tool_defs),
            temperature: None,
            max_tokens: None,
            stop: None,
            stream: true, // We need streaming for the full pipeline
        };

        // Call the model
        let mut stream = provider.chat_completion(request).await?;

        let mut full_response = String::new();
        let mut tool_calls: Vec<aisopod_provider::ToolCall> = Vec::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;

            // Handle text deltas
            if let Some(content) = &chunk.delta.content {
                full_response.push_str(content);
            }

            // Handle tool calls
            if let Some(ref chunk_tool_calls) = chunk.delta.tool_calls {
                for tool_call in chunk_tool_calls {
                    // Merge tool calls - check if we already have this tool call ID
                    if let Some(existing) = tool_calls.iter_mut().find(|t| t.id == tool_call.id) {
                        // Append arguments
                        existing.arguments.push_str(&tool_call.arguments);
                    } else {
                        tool_calls.push(tool_call.clone());
                    }
                }
            }

            // Handle finish reason
            if let Some(reason) = &chunk.finish_reason {
                match reason {
                    aisopod_provider::FinishReason::Stop => break,
                    aisopod_provider::FinishReason::ToolCall => {
                        // Model wants to make tool calls
                        break;
                    }
                    _ => {}
                }
            }
        }

        Ok((full_response, tool_calls))
    }

    /// Gets the tools configured for an agent.
    fn get_agent_tools(&self) -> Result<Vec<ToolSchema>> {
        let mut tools = Vec::new();

        // Get all registered tools from the registry
        for tool_name in self.tools.list() {
            if let Some(tool) = self.tools.get(&tool_name) {
                tools.push(ToolSchema::new(
                    tool.name(),
                    tool.description(),
                    tool.parameters_schema(),
                ));
            }
        }

        Ok(tools)
    }

    /// Builds the system prompt for the agent.
    fn build_system_prompt(
        &self,
        agent_config: &aisopod_config::types::Agent,
        tools: &[ToolSchema],
    ) -> Result<String> {
        let mut builder = SystemPromptBuilder::new();

        // Add dynamic context
        builder = builder.with_dynamic_context();

        // Add tool descriptions
        builder = builder.with_tool_descriptions(tools);

        // Build the final prompt
        Ok(builder.build())
    }

    /// Prepares the message history with the system prompt.
    fn prepare_messages(
        &self,
        user_messages: &[aisopod_provider::Message],
        system_prompt: &str,
    ) -> Vec<aisopod_provider::Message> {
        let mut messages = Vec::new();

        // Add system message at the start
        messages.push(aisopod_provider::Message {
            role: aisopod_provider::Role::System,
            content: aisopod_provider::MessageContent::Text(system_prompt.to_string()),
            tool_calls: None,
            tool_call_id: None,
        });

        // Add user messages
        messages.extend(user_messages.to_vec());

        messages
    }

    /// Gets the provider kind for transcript repair.
    fn get_provider_kind(
        &self,
        provider: &Arc<dyn aisopod_provider::trait_module::ModelProvider>,
    ) -> transcript::ProviderKind {
        let provider_id = provider.id().to_lowercase();

        match provider_id.as_str() {
            "anthropic" | "claude" => transcript::ProviderKind::Anthropic,
            "openai" | "gpt" => transcript::ProviderKind::OpenAI,
            "google" | "gemini" => transcript::ProviderKind::Google,
            _ => transcript::ProviderKind::Other,
        }
    }

    /// Calls the model with messages and tools.
    async fn call_model(
        &self,
        provider: &Arc<dyn aisopod_provider::trait_module::ModelProvider>,
        model_id: &str,
        messages: &[aisopod_provider::Message],
        tools: &[ToolSchema],
    ) -> Result<(String, Vec<aisopod_provider::ToolCall>)> {
        // Convert tool schemas to tool definitions
        let tool_defs: Vec<aisopod_provider::ToolDefinition> = tools
            .iter()
            .map(|t| aisopod_provider::ToolDefinition {
                name: t.name.clone(),
                description: t.description.clone(),
                parameters: t.parameters.clone(),
            })
            .collect();

        // Create completion request
        let request = aisopod_provider::ChatCompletionRequest {
            model: model_id.to_string(),
            messages: messages.to_vec(),
            tools: Some(tool_defs),
            temperature: None,
            max_tokens: None,
            stop: None,
            stream: true, // We need streaming for the full pipeline
        };

        // Call the model
        let mut stream = provider.chat_completion(request).await?;

        let mut full_response = String::new();
        let mut tool_calls: Vec<aisopod_provider::ToolCall> = Vec::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;

            // Handle text deltas
            if let Some(content) = &chunk.delta.content {
                full_response.push_str(content);
            }

            // Handle tool calls
            if let Some(ref chunk_tool_calls) = chunk.delta.tool_calls {
                for tool_call in chunk_tool_calls {
                    // Merge tool calls - check if we already have this tool call ID
                    if let Some(existing) = tool_calls.iter_mut().find(|t| t.id == tool_call.id) {
                        // Append arguments
                        existing.arguments.push_str(&tool_call.arguments);
                    } else {
                        tool_calls.push(tool_call.clone());
                    }
                }
            }

            // Handle finish reason
            if let Some(reason) = &chunk.finish_reason {
                match reason {
                    aisopod_provider::FinishReason::Stop => break,
                    aisopod_provider::FinishReason::ToolCall => {
                        // Model wants to make tool calls
                        break;
                    }
                    _ => {}
                }
            }
        }

        Ok((full_response, tool_calls))
    }

    /// Executes a tool and returns the result.
    async fn execute_tool(
        &self,
        tool_call: &aisopod_provider::ToolCall,
        session_key: &str,
    ) -> Result<aisopod_tools::ToolResult> {
        let tool_name = &tool_call.name;

        // Parse arguments
        let args: serde_json::Value = serde_json::from_str(&tool_call.arguments)
            .map_err(|e| anyhow::anyhow!("Failed to parse tool arguments: {}", e))?;

        // Get tool from registry
        let tool = self.tools.get(tool_name).ok_or_else(|| {
            anyhow::anyhow!("Tool not found: {}", tool_name)
        })?;

        // Create tool context
        let ctx = aisopod_tools::ToolContext::new("agent", session_key);

        // Execute the tool
        tool.execute(args, &ctx).await
    }

    /// Gets the tool result for a tool call.
    async fn get_tool_result(
        &self,
        _tool_call: &aisopod_provider::ToolCall,
        _session_key: &str,
    ) -> Result<aisopod_tools::ToolResult> {
        // This is a placeholder - in a real implementation, we'd retrieve
        // the result from a previous tool execution
        Ok(aisopod_tools::ToolResult::success("Tool executed"))
    }
}

/// A stream of agent events.
///
/// This type wraps the receiver end of the event channel and provides
/// an async stream interface for consuming agent events.
pub struct AgentRunStream {
    rx: mpsc::UnboundedReceiver<AgentEvent>,
}

impl AgentRunStream {
    /// Creates a new `AgentRunStream` from a receiver.
    pub fn new(rx: mpsc::UnboundedReceiver<AgentEvent>) -> Self {
        Self { rx }
    }

    /// Receives the next event from the stream.
    ///
    /// Returns `Some(AgentEvent)` if an event is available, or `None`
    /// if the stream has been closed.
    pub async fn next(&mut self) -> Option<AgentEvent> {
        self.rx.recv().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_pipeline_new() {
        let config = Arc::new(aisopod_config::AisopodConfig::default());
        let providers = Arc::new(aisopod_provider::ProviderRegistry::new());
        let tools = Arc::new(aisopod_tools::ToolRegistry::new());
        let sessions = Arc::new(aisopod_session::SessionStore::new());

        let pipeline = AgentPipeline::new(config, providers, tools, sessions);

        // Just verify it compiles - full tests will be added in subsequent issues
        assert_eq!(pipeline.config.meta.version, "1.0");
    }
}
