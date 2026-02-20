use serde::{Deserialize, Serialize};

/// Channels configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChannelsConfig {
    /// Channel definitions
    #[serde(default)]
    pub channels: Vec<Channel>,
    /// Default channel settings
    #[serde(default)]
    pub default: ChannelDefaults,
}

/// Channel definition
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Channel {
    /// Channel ID
    pub id: String,
    /// Channel name
    #[serde(default)]
    pub name: String,
    /// Channel type
    #[serde(default)]
    pub channel_type: String,
    /// Connection settings
    #[serde(default)]
    pub connection: ChannelConnection,
}

/// Channel connection settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChannelConnection {
    /// Connection string or endpoint
    #[serde(default)]
    pub endpoint: String,
    /// Authentication token
    #[serde(default)]
    pub token: String,
}

/// Default channel settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChannelDefaults {
    /// Default channel type
    #[serde(default)]
    pub channel_type: String,
}
