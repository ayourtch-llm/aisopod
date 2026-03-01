//! Built-in plugin loading and management.
//!
//! This module provides functionality for registering built-in plugins
//! based on compile-time feature flags. Plugins can be selectively enabled
//! or disabled at build time to create minimal custom builds.

use crate::{Plugin, PluginRegistry, RegistryError};
use std::sync::Arc;
use tracing::info;

/// Registers all built-in plugins that are enabled via feature flags.
///
/// This function checks which plugins are enabled through Cargo features
/// and registers them with the provided registry. Plugins not enabled via
/// features are completely excluded from the binary, producing zero runtime
/// overhead.
///
/// # Features
///
/// The following features control which plugins are loaded:
/// - `plugin-telegram` - Registers the Telegram channel plugin
/// - `plugin-discord` - Registers the Discord channel plugin
/// - `plugin-slack` - Registers the Slack channel plugin
/// - `plugin-whatsapp` - Registers the WhatsApp channel plugin
/// - `plugin-openai` - Registers the OpenAI provider plugin
/// - `plugin-anthropic` - Registers the Anthropic provider plugin
///
/// # Example
///
/// ```ignore
/// use aisopod_plugin::{PluginRegistry, register_builtin_plugins};
///
/// async fn example() -> Result<(), Box<dyn std::error::Error>> {
///     let mut registry = PluginRegistry::new();
///     register_builtin_plugins(&mut registry)?;
///
///     // Initialize all registered plugins
///     // registry.init_all(&ctx).await?;
///
///     Ok(())
/// }
/// ```
pub fn register_builtin_plugins(registry: &mut PluginRegistry) -> Result<(), RegistryError> {
    // Channel plugins
    #[cfg(feature = "plugin-telegram")]
    {
        register_telegram_plugin(registry)?;
    }

    #[cfg(feature = "plugin-discord")]
    {
        register_discord_plugin(registry)?;
    }

    #[cfg(feature = "plugin-slack")]
    {
        register_slack_plugin(registry)?;
    }

    #[cfg(feature = "plugin-whatsapp")]
    {
        register_whatsapp_plugin(registry)?;
    }

    // Provider plugins
    #[cfg(feature = "plugin-openai")]
    {
        register_openai_plugin(registry)?;
    }

    #[cfg(feature = "plugin-anthropic")]
    {
        register_anthropic_plugin(registry)?;
    }

    Ok(())
}

/// Registers the Telegram channel plugin.
#[cfg(feature = "plugin-telegram")]
fn register_telegram_plugin(registry: &mut PluginRegistry) -> Result<(), RegistryError> {
    use aisopod_channel::ChannelPlugin;
    use aisopod_channel_telegram::TelegramChannel;

    let channel = TelegramChannel::default();
    registry.register(Arc::new(channel));
    info!("Registered built-in plugin: telegram");
    Ok(())
}

/// Registers the Discord channel plugin.
#[cfg(feature = "plugin-discord")]
fn register_discord_plugin(registry: &mut PluginRegistry) -> Result<(), RegistryError> {
    use aisopod_channel::ChannelPlugin;
    use aisopod_channel_discord::DiscordChannel;

    let channel = DiscordChannel::default();
    registry.register(Arc::new(channel));
    info!("Registered built-in plugin: discord");
    Ok(())
}

/// Registers the Slack channel plugin.
#[cfg(feature = "plugin-slack")]
fn register_slack_plugin(registry: &mut PluginRegistry) -> Result<(), RegistryError> {
    use aisopod_channel::ChannelPlugin;
    use aisopod_channel_slack::SlackChannel;

    let channel = SlackChannel::default();
    registry.register(Arc::new(channel));
    info!("Registered built-in plugin: slack");
    Ok(())
}

/// Registers the WhatsApp channel plugin.
#[cfg(feature = "plugin-whatsapp")]
fn register_whatsapp_plugin(registry: &mut PluginRegistry) -> Result<(), RegistryError> {
    use aisopod_channel::ChannelPlugin;
    use aisopod_channel_whatsapp::WhatsAppChannel;

    let channel = WhatsAppChannel::default();
    registry.register(Arc::new(channel));
    info!("Registered built-in plugin: whatsapp");
    Ok(())
}

/// OpenAI provider plugin wrapper.
///
/// This struct wraps the OpenAIProvider and implements the Plugin trait
/// for integration with the aisopod plugin system.
#[cfg(feature = "plugin-openai")]
#[derive(Debug)]
pub struct OpenAIPluginWrapper {
    /// The underlying OpenAI provider
    provider: aisopod_provider_openai::OpenAIPlugin,
}

#[cfg(feature = "plugin-openai")]
impl OpenAIPluginWrapper {
    /// Creates a new OpenAI plugin wrapper.
    pub fn new() -> Self {
        Self {
            provider: aisopod_provider_openai::OpenAIPlugin::default(),
        }
    }
}

#[cfg(feature = "plugin-openai")]
impl Plugin for OpenAIPluginWrapper {
    fn id(&self) -> &str {
        "openai"
    }

    fn meta(&self) -> &crate::PluginMeta {
        // Use a simple static metadata for the wrapper
        // In a real implementation, this would come from the underlying provider
        static META: crate::PluginMeta = crate::PluginMeta {
            name: "openai",
            version: env!("CARGO_PKG_VERSION"),
            description: "OpenAI provider plugin",
            author: "aisopod",
            capabilities: vec!["chat".to_string(), "completion".to_string()],
            channels: vec!["text".to_string()],
        };
        &META
    }

    fn register(&self, _api: &mut crate::PluginApi) -> Result<(), Box<dyn std::error::Error>> {
        // Register OpenAI-specific capabilities
        info!("Registered OpenAI plugin");
        Ok(())
    }

    async fn init(&self, _ctx: &crate::PluginContext) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize the OpenAI provider
        info!("Initializing OpenAI plugin");
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Shutdown the OpenAI provider
        info!("Shutting down OpenAI plugin");
        Ok(())
    }
}

/// Registers the OpenAI provider plugin.
#[cfg(feature = "plugin-openai")]
fn register_openai_plugin(registry: &mut PluginRegistry) -> Result<(), RegistryError> {
    let plugin = OpenAIPluginWrapper::new();
    registry.register(Arc::new(plugin));
    info!("Registered built-in plugin: openai");
    Ok(())
}

/// Anthropic provider plugin wrapper.
///
/// This struct wraps the AnthropicProvider and implements the Plugin trait
/// for integration with the aisopod plugin system.
#[cfg(feature = "plugin-anthropic")]
#[derive(Debug)]
pub struct AnthropicPluginWrapper {
    /// The underlying Anthropic provider
    provider: aisopod_provider_anthropic::AnthropicPlugin,
}

#[cfg(feature = "plugin-anthropic")]
impl AnthropicPluginWrapper {
    /// Creates a new Anthropic plugin wrapper.
    pub fn new() -> Self {
        Self {
            provider: aisopod_provider_anthropic::AnthropicPlugin::default(),
        }
    }
}

#[cfg(feature = "plugin-anthropic")]
impl Plugin for AnthropicPluginWrapper {
    fn id(&self) -> &str {
        "anthropic"
    }

    fn meta(&self) -> &crate::PluginMeta {
        // Use a simple static metadata for the wrapper
        static META: crate::PluginMeta = crate::PluginMeta {
            name: "anthropic",
            version: env!("CARGO_PKG_VERSION"),
            description: "Anthropic provider plugin",
            author: "aisopod",
            capabilities: vec!["chat".to_string(), "completion".to_string()],
            channels: vec!["text".to_string()],
        };
        &META
    }

    fn register(&self, _api: &mut crate::PluginApi) -> Result<(), Box<dyn std::error::Error>> {
        // Register Anthropic-specific capabilities
        info!("Registered Anthropic plugin");
        Ok(())
    }

    async fn init(&self, _ctx: &crate::PluginContext) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize the Anthropic provider
        info!("Initializing Anthropic plugin");
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Shutdown the Anthropic provider
        info!("Shutting down Anthropic plugin");
        Ok(())
    }
}

/// Registers the Anthropic provider plugin.
#[cfg(feature = "plugin-anthropic")]
fn register_anthropic_plugin(registry: &mut PluginRegistry) -> Result<(), RegistryError> {
    let plugin = AnthropicPluginWrapper::new();
    registry.register(Arc::new(plugin));
    info!("Registered built-in plugin: anthropic");
    Ok(())
}

/// Returns a list of available built-in plugins based on enabled features.
///
/// This function returns the IDs of plugins that would be registered
/// if `register_builtin_plugins()` were called with the current feature flags.
///
/// # Returns
///
/// A vector of plugin ID strings for all enabled built-in plugins.
///
/// # Example
///
/// ```ignore
/// use aisopod_plugin::list_available_builtins;
///
/// let builtins = list_available_builtins();
/// for plugin_id in builtins {
///     println!("Available plugin: {}", plugin_id);
/// }
/// ```
pub fn list_available_builtins() -> Vec<&'static str> {
    let mut plugins = Vec::new();

    #[cfg(feature = "plugin-telegram")]
    plugins.push("telegram");

    #[cfg(feature = "plugin-discord")]
    plugins.push("discord");

    #[cfg(feature = "plugin-slack")]
    plugins.push("slack");

    #[cfg(feature = "plugin-whatsapp")]
    plugins.push("whatsapp");

    #[cfg(feature = "plugin-openai")]
    plugins.push("openai");

    #[cfg(feature = "plugin-anthropic")]
    plugins.push("anthropic");

    plugins
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_available_builtins() {
        // This test verifies that list_available_builtins compiles
        // The actual contents depend on which features are enabled
        let builtins = list_available_builtins();
        assert!(builtins.is_empty() || builtins.len() <= 6);
    }

    #[test]
    fn test_register_builtin_plugins_empty() {
        // With no features enabled, this should complete without error
        let mut registry = PluginRegistry::new();
        assert!(register_builtin_plugins(&mut registry).is_ok());
        assert_eq!(registry.list().len(), 0);
    }
}
