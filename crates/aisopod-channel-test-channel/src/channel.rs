use crate::config::TestChannelConfig;
use aisopod_channel::adapters::{ChannelConfigAdapter, SecurityAdapter};
use aisopod_channel::{
    ChannelCapabilities, ChannelMeta, ChannelPlugin, IncomingMessage, OutgoingMessage,
};
use async_trait::async_trait;

/// Channel implementation for Test Channel.
pub struct TestChannelChannel {
    /// Channel identifier
    id: String,
    /// Channel metadata
    meta: ChannelMeta,
    /// Channel capabilities
    capabilities: ChannelCapabilities,
    // TODO: Add channel-specific fields
}

impl Default for TestChannelChannel {
    fn default() -> Self {
        Self {
            id: "test-channel".to_string(),
            meta: ChannelMeta {
                label: "Test Channel".to_string(),
                docs_url: Some("https://example.com/docs/test-channel".to_string()),
                ui_hints: serde_json::Value::Null,
            },
            capabilities: ChannelCapabilities::default(),
            // TODO: Initialize fields
        }
    }
}

#[async_trait]
impl ChannelPlugin for TestChannelChannel {
    fn id(&self) -> &str {
        &self.id
    }

    fn meta(&self) -> &ChannelMeta {
        &self.meta
    }

    fn capabilities(&self) -> &ChannelCapabilities {
        &self.capabilities
    }

    fn config(&self) -> &dyn ChannelConfigAdapter {
        unimplemented!("Test Channel channel config adapter not yet implemented")
    }

    fn security(&self) -> Option<&dyn SecurityAdapter> {
        None
    }

    async fn connect(&mut self) -> Result<(), anyhow::Error> {
        tracing::info!("Test Channel channel connecting...");
        todo!("Implement connect for Test Channel")
    }

    async fn send(&self, _msg: OutgoingMessage) -> Result<(), anyhow::Error> {
        todo!("Implement send for Test Channel")
    }

    async fn receive(&mut self) -> Result<IncomingMessage, anyhow::Error> {
        todo!("Implement receive for Test Channel")
    }

    async fn disconnect(&mut self) -> Result<(), anyhow::Error> {
        tracing::info!("Test Channel channel disconnecting...");
        Ok(())
    }
}
