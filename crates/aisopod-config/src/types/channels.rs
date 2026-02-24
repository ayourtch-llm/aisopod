use crate::sensitive::Sensitive;
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
    /// Telegram-specific configuration
    #[serde(default)]
    pub telegram: TelegramConfig,
    /// Discord-specific configuration
    #[serde(default)]
    pub discord: DiscordConfig,
    /// WhatsApp-specific configuration
    #[serde(default)]
    pub whatsapp: WhatsappConfig,
    /// Slack-specific configuration
    #[serde(default)]
    pub slack: SlackConfig,
    /// GitHub-specific configuration
    #[serde(default)]
    pub github: GenericChannelConfig,
    /// GitLab-specific configuration
    #[serde(default)]
    pub gitlab: GenericChannelConfig,
    /// Bitbucket-specific configuration
    #[serde(default)]
    pub bitbucket: GenericChannelConfig,
    /// Mattermost-specific configuration
    #[serde(default)]
    pub mattermost: GenericChannelConfig,
    /// Matrix-specific configuration
    #[serde(default)]
    pub matrix: MatrixConfig,
}

/// Generic channel configuration for platforms with simple token auth
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GenericChannelConfig {
    /// API token
    pub token: Option<Sensitive<String>>,
}

/// Telegram bot configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TelegramConfig {
    /// Telegram bot token
    pub token: Option<Sensitive<String>>,
}

/// Discord bot configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiscordConfig {
    /// Discord bot token
    pub token: Option<Sensitive<String>>,
    /// Discord client secret (for OAuth)
    #[serde(default)]
    pub client_secret: Option<Sensitive<String>>,
}

/// WhatsApp Business API configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WhatsappConfig {
    /// Access token
    pub access_token: Option<Sensitive<String>>,
    /// Phone number ID
    #[serde(default)]
    pub phone_number_id: Option<String>,
}

/// Slack workspace configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SlackConfig {
    /// Bot token (starts with xoxb-)
    pub token: Option<Sensitive<String>>,
    /// Signing secret (for request verification)
    #[serde(default)]
    pub signing_secret: Option<Sensitive<String>>,
    /// Verification token (deprecated but still used in some cases)
    #[serde(default)]
    pub verification_token: Option<Sensitive<String>>,
    /// App-level token (for server-side operations)
    #[serde(default)]
    pub app_token: Option<Sensitive<String>>,
}

/// Matrix room configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MatrixConfig {
    /// Access token
    pub access_token: Option<Sensitive<String>>,
    /// Home server URL
    #[serde(default)]
    pub home_server: Option<String>,
    /// User ID
    #[serde(default)]
    pub user_id: Option<String>,
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
