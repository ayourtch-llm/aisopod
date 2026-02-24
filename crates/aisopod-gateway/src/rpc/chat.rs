//! Chat RPC methods for aisopod-gateway.
//!
//! This module provides the `chat.send` RPC handler implementation
//! that integrates with the AgentRunner to execute agents and stream responses.

use anyhow::Result;
use serde_json::json;
use std::sync::Arc;

/// Handler for the chat.send RPC method.
///
/// This handler receives messages from WebSocket clients, executes them through
/// an agent, and streams the responses back via the WebSocket connection.
///
/// # Parameters
/// - `text`: The message text to send to the agent
/// - `channel`: Optional channel ID for routing messages
/// - `agent`: Optional agent ID to execute (uses default if not specified)
pub struct ChatSendHandler;

impl ChatSendHandler {
    /// Parse parameters from the JSON-RPC request
    fn parse_params(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<SendMessageParams, serde_json::Error> {
        match params {
            Some(p) => serde_json::from_value::<SendMessageParams>(p),
            None => Ok(SendMessageParams {
                text: String::new(),
                channel: None,
                agent: None,
            }),
        }
    }
}

/// Parameters for the chat.send RPC method
#[derive(Debug, Clone, serde::Deserialize)]
pub struct SendMessageParams {
    /// The message text to send
    #[serde(default)]
    pub text: String,
    /// Optional channel ID for routing
    #[serde(default)]
    pub channel: Option<String>,
    /// Optional agent ID to execute
    #[serde(default)]
    pub agent: Option<String>,
}

impl ChatSendHandler {
    /// Handle the chat.send RPC request with full dependencies
    ///
    /// This is called by the WebSocket handler with the full context
    pub fn handle_with_deps(
        &self,
        conn_id: String,
        params: Option<serde_json::Value>,
        agent_runner: std::sync::Arc<aisopod_agent::AgentRunner>,
        ws_sender: std::sync::Arc<tokio::sync::mpsc::Sender<axum::extract::ws::Message>>,
    ) -> serde_json::Value {
        // Parse parameters
        let params = match self.parse_params(params) {
            Ok(p) => p,
            Err(e) => {
                return serde_json::json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32602,
                        "message": format!("Invalid parameters: {}", e)
                    },
                    "id": conn_id
                });
            }
        };

        let channel = params.channel;
        let agent_id = params.agent;
        let conn_id_clone = conn_id.clone();
        let conn_id_for_response = conn_id_clone.clone();

        // Spawn a task to run the agent and stream results
        tokio::spawn(async move {
            if let Err(e) = run_agent_and_stream(
                agent_runner,
                ws_sender,
                conn_id_clone,
                params.text,
                channel,
                agent_id,
            )
            .await
            {
                eprintln!("Error running agent: {}", e);
            }
        });

        // Return immediate acknowledgment (use conn_id_for_response since conn_id_clone is moved)
        serde_json::json!({
            "jsonrpc": "2.0",
            "result": {
                "status": "accepted",
                "message": "Agent execution started"
            },
            "id": conn_id_for_response
        })
    }
}

/// Run an agent and stream results via WebSocket
async fn run_agent_and_stream(
    agent_runner: Arc<aisopod_agent::AgentRunner>,
    ws_sender: std::sync::Arc<tokio::sync::mpsc::Sender<axum::extract::ws::Message>>,
    conn_id: String,
    text: String,
    channel: Option<String>,
    agent_id: Option<String>,
) -> Result<()> {
    // Create a session key based on channel or conn_id
    let session_key = channel.unwrap_or_else(|| conn_id.clone());

    // Build the initial message
    let message = aisopod_provider::Message {
        role: aisopod_provider::Role::User,
        content: aisopod_provider::MessageContent::Text(text),
        tool_calls: None,
        tool_call_id: None,
    };

    // Prepare agent run parameters
    let params = aisopod_agent::AgentRunParams::new(
        session_key.clone(),
        vec![message],
        agent_id,
    );

    // Run the agent and get the event stream
    let stream = agent_runner.run(params).await?;

    // Stream events to WebSocket
    let mut receiver = stream.into_receiver();
    let mut has_sent_done = false;

    while let Some(event) = receiver.recv().await {
        match event {
            aisopod_agent::AgentEvent::TextDelta { text: delta, .. } => {
                // Stream text deltas as they arrive
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": "chat.response",
                    "params": {
                        "text": delta,
                        "done": false
                    }
                });

                if let Err(e) = ws_sender.send(axum::extract::ws::Message::Text(
                    serde_json::to_string(&response)?
                )).await {
                    eprintln!("Failed to send text delta: {}", e);
                    break;
                }
            }
            aisopod_agent::AgentEvent::ToolCallStart { tool_name, call_id } => {
                // Stream tool call start
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": "chat.response",
                    "params": {
                        "tool_call_start": {
                            "tool_name": tool_name,
                            "call_id": call_id
                        },
                        "done": false
                    }
                });

                if let Err(e) = ws_sender.send(axum::extract::ws::Message::Text(
                    serde_json::to_string(&response)?
                )).await {
                    eprintln!("Failed to send tool call start: {}", e);
                    break;
                }
            }
            aisopod_agent::AgentEvent::ToolCallResult { call_id, result, is_error } => {
                // Stream tool call result
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": "chat.response",
                    "params": {
                        "tool_call_result": {
                            "call_id": call_id,
                            "result": result,
                            "is_error": is_error
                        },
                        "done": false
                    }
                });

                if let Err(e) = ws_sender.send(axum::extract::ws::Message::Text(
                    serde_json::to_string(&response)?
                )).await {
                    eprintln!("Failed to send tool call result: {}", e);
                    break;
                }
            }
            aisopod_agent::AgentEvent::Error { message } => {
                // Stream error
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": "chat.response",
                    "params": {
                        "error": message,
                        "done": true
                    }
                });

                if let Err(e) = ws_sender.send(axum::extract::ws::Message::Text(
                    serde_json::to_string(&response)?
                )).await {
                    eprintln!("Failed to send error: {}", e);
                }
                has_sent_done = true;
                break;
            }
            aisopod_agent::AgentEvent::Complete { result } => {
                // Stream final result with done marker
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": "chat.response",
                    "params": {
                        "text": result.response,
                        "tool_calls": result.tool_calls,
                        "usage": result.usage,
                        "done": true
                    }
                });

                if let Err(e) = ws_sender.send(axum::extract::ws::Message::Text(
                    serde_json::to_string(&response)?
                )).await {
                    eprintln!("Failed to send complete: {}", e);
                }
                has_sent_done = true;
            }
            _ => {
                // Ignore other event types for now
            }
        }
    }

    // Ensure we send a done marker if we haven't already
    if !has_sent_done {
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "chat.response",
            "params": {
                "done": true
            }
        });

        let _ = ws_sender.send(axum::extract::ws::Message::Text(
            serde_json::to_string(&response)?
        )).await;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_message_params_deserialization() {
        let json = r#"{"text":"Hello","channel":"test-channel","agent":"my-agent"}"#;
        let params: SendMessageParams = serde_json::from_str(json).unwrap();

        assert_eq!(params.text, "Hello");
        assert_eq!(params.channel, Some("test-channel".to_string()));
        assert_eq!(params.agent, Some("my-agent".to_string()));
    }

    #[test]
    fn test_send_message_params_minimal() {
        let json = r#"{"text":"Hello"}"#;
        let params: SendMessageParams = serde_json::from_str(json).unwrap();

        assert_eq!(params.text, "Hello");
        assert_eq!(params.channel, None);
        assert_eq!(params.agent, None);
    }

    #[test]
    fn test_send_message_params_empty_text() {
        let json = r#"{}"#;
        let params: SendMessageParams = serde_json::from_str(json).unwrap();

        assert_eq!(params.text, "");
        assert_eq!(params.channel, None);
        assert_eq!(params.agent, None);
    }
}
