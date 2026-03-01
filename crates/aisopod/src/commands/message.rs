//! Message sending command implementation module
//!
//! This module provides the `aisopod message` command for sending messages to agents
//! via the gateway's WebSocket interface.

use anyhow::{anyhow, Result};
use clap::Args;
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::io::Write;
use tokio_tungstenite::connect_async;

/// Message command arguments
#[derive(Args)]
pub struct MessageArgs {
    /// Message text to send
    pub text: String,

    /// Target channel
    #[arg(long)]
    pub channel: Option<String>,

    /// Target agent ID (uses default if not specified)
    #[arg(long)]
    pub agent: Option<String>,
}

/// Get the WebSocket URL from the configuration
fn get_ws_url(config: &aisopod_config::AisopodConfig) -> String {
    let scheme = if config.gateway.tls.enabled {
        "wss"
    } else {
        "ws"
    };
    let host = &config.gateway.bind.address;
    let port = config.gateway.server.port;
    format!("{}://{}:{}/ws", scheme, host, port)
}

/// Send a message to an agent via WebSocket
pub async fn run(args: MessageArgs, config_path: Option<String>) -> Result<()> {
    // Load configuration
    let config = match config_path {
        Some(path) => {
            let config_path = std::path::Path::new(&path);
            aisopod_config::load_config(config_path)
                .map_err(|e| anyhow!("Failed to load configuration from '{}': {}", path, e))?
        }
        None => {
            // Use default configuration
            aisopod_config::AisopodConfig::default()
        }
    };

    // Get WebSocket URL
    let ws_url = get_ws_url(&config);

    // Connect to the gateway via WebSocket
    let (mut ws_stream, _) = connect_async(&ws_url)
        .await
        .map_err(|e| anyhow!("Failed to connect to gateway at {}: {}", ws_url, e))?;

    // Build the JSON-RPC request
    let request = json!({
        "jsonrpc": "2.0",
        "method": "chat.send",
        "params": {
            "text": args.text,
            "channel": args.channel,
            "agent": args.agent,
        },
        "id": 1
    });

    // Send the request
    ws_stream
        .send(request.to_string().into())
        .await
        .map_err(|e| anyhow!("Failed to send message: {}", e))?;

    // Stream response chunks to terminal
    while let Some(msg) = ws_stream.next().await {
        let msg = msg.map_err(|e| anyhow!("Failed to receive response: {}", e))?;
        if msg.is_text() {
            let response: Value = serde_json::from_str(msg.to_text()?)
                .map_err(|e| anyhow!("Failed to parse response: {}", e))?;

            // Check if this is an error response
            if let Some(error) = response.get("error") {
                return Err(anyhow!(
                    "Gateway error: {}",
                    error.get("message").unwrap_or(&json!("Unknown error"))
                ));
            }

            // Extract result content
            if let Some(result) = response.get("result") {
                // Check if streaming is done
                if result.get("done").and_then(|d| d.as_bool()) == Some(true) {
                    break;
                }

                // Print text content if available
                if let Some(text) = result.get("text").and_then(|t| t.as_str()) {
                    print!("{}", text);
                    std::io::stdout().flush()?;
                }
            }
        }
    }

    // Close the connection
    ws_stream.close(None).await?;

    // Print newline after response
    println!();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_args_default() {
        let args = MessageArgs {
            text: "Hello".to_string(),
            channel: None,
            agent: None,
        };

        assert_eq!(args.text, "Hello");
        assert!(args.channel.is_none());
        assert!(args.agent.is_none());
    }

    #[test]
    fn test_message_args_with_options() {
        let args = MessageArgs {
            text: "Test message".to_string(),
            channel: Some("telegram".to_string()),
            agent: Some("myagent".to_string()),
        };

        assert_eq!(args.text, "Test message");
        assert_eq!(args.channel, Some("telegram".to_string()));
        assert_eq!(args.agent, Some("myagent".to_string()));
    }
}
