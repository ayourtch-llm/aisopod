//! Integration tests for iMessage channel.
//!
//! These tests verify the iMessage channel works correctly with both
//! backends (AppleScript and BlueBubbles).

use aisopod_channel::ChannelPlugin;
use aisopod_channel::ChannelRegistry;
use aisopod_channel_imessage::config::BlueBubblesConfig;
use aisopod_channel_imessage::{
    register, BackendType, ChatType, ImessageAccountConfig, ImessageChannel, PeerKind,
};

/// Mock AppleScript backend for testing on non-macOS platforms.
#[cfg(not(target_os = "macos"))]
mod mock_applescript {
    use crate::BackendType;

    pub struct MockAppleScriptBackend {
        connected: bool,
    }

    impl MockAppleScriptBackend {
        pub fn new() -> Self {
            Self { connected: false }
        }

        pub fn connect(&mut self) {
            self.connected = true;
        }

        pub fn disconnect(&mut self) {
            self.connected = false;
        }

        pub fn is_connected(&self) -> bool {
            self.connected
        }
    }
}

#[tokio::test]
async fn test_imessage_channel_creation() {
    let config = ImessageAccountConfig::new("test-channel");

    // Ensure we have a valid backend for testing
    #[cfg(target_os = "macos")]
    let config = config;

    #[cfg(not(target_os = "macos"))]
    let config = {
        let mut config = config;
        // On non-macOS, bluebubbles requires api_url
        config.bluebubbles = BlueBubblesConfig {
            api_url: Some("http://localhost:12345".to_string()),
            ..Default::default()
        };
        config
    };

    let result = ImessageChannel::new(config).await;

    // Should succeed on any platform (with appropriate backend)
    assert!(result.is_ok(), "Channel creation should succeed");
}

#[tokio::test]
async fn test_imessage_channel_default_backend() {
    let mut config = ImessageAccountConfig::default();

    // On non-macOS, bluebubbles requires api_url
    #[cfg(not(target_os = "macos"))]
    {
        config.bluebubbles = BlueBubblesConfig {
            api_url: Some("http://localhost:12345".to_string()),
            ..Default::default()
        };
    }

    let channel = ImessageChannel::new(config).await.unwrap();

    // Check that the backend type is correct for the platform
    let backend = channel.backend_type();

    #[cfg(target_os = "macos")]
    assert_eq!(backend, BackendType::AppleScript);

    #[cfg(not(target_os = "macos"))]
    assert_eq!(backend, BackendType::BlueBubbles);
}

#[tokio::test]
async fn test_imessage_channel_capabilities() {
    let config = ImessageAccountConfig::new("test-capabilities");

    // Ensure we have a valid backend for testing
    #[cfg(target_os = "macos")]
    let config = config;

    #[cfg(not(target_os = "macos"))]
    let config = {
        let mut config = config;
        // On non-macOS, bluebubbles requires api_url
        config.bluebubbles = BlueBubblesConfig {
            api_url: Some("http://localhost:12345".to_string()),
            ..Default::default()
        };
        config
    };

    let channel = ImessageChannel::new(config).await.unwrap();

    let caps = channel.capabilities();

    // Verify capabilities
    assert!(caps.chat_types.contains(&ChatType::Dm));
    assert!(caps.chat_types.contains(&ChatType::Group));
    assert!(caps.supports_media);
    assert!(caps.supports_reactions);
    assert!(caps.supports_voice);
}

#[tokio::test]
async fn test_imessage_channel_registration() {
    let mut registry = ChannelRegistry::new();

    let config = ImessageAccountConfig::new("test-register");

    // Ensure we have a valid backend for testing
    #[cfg(target_os = "macos")]
    let config = config;

    #[cfg(not(target_os = "macos"))]
    let config = {
        let mut config = config;
        // On non-macOS, bluebubbles requires api_url
        config.bluebubbles = BlueBubblesConfig {
            api_url: Some("http://localhost:12345".to_string()),
            ..Default::default()
        };
        config
    };

    let result = register(&mut registry, config, "test-register").await;

    assert!(result.is_ok(), "Channel registration should succeed");
    assert!(registry.contains("imessage-test-register"));
}

#[tokio::test]
async fn test_imessage_channel_id() {
    let config = ImessageAccountConfig::new("my-account");

    // Ensure we have a valid backend for testing
    #[cfg(target_os = "macos")]
    let config = config;

    #[cfg(not(target_os = "macos"))]
    let config = {
        let mut config = config;
        // On non-macOS, bluebubbles requires api_url
        config.bluebubbles = BlueBubblesConfig {
            api_url: Some("http://localhost:12345".to_string()),
            ..Default::default()
        };
        config
    };

    let channel = ImessageChannel::new(config).await.unwrap();

    assert_eq!(channel.id(), "imessage-my-account");
}

#[tokio::test]
async fn test_imessage_channel_meta() {
    let config = ImessageAccountConfig::new("test-meta");

    // Ensure we have a valid backend for testing
    #[cfg(target_os = "macos")]
    let config = config;

    #[cfg(not(target_os = "macos"))]
    let config = {
        let mut config = config;
        // On non-macOS, bluebubbles requires api_url
        config.bluebubbles = BlueBubblesConfig {
            api_url: Some("http://localhost:12345".to_string()),
            ..Default::default()
        };
        config
    };

    let channel = ImessageChannel::new(config).await.unwrap();

    let meta = channel.meta();

    assert_eq!(meta.label, "iMessage");
    assert!(meta.docs_url.is_some());
}

#[tokio::test]
async fn test_imessage_channel_config_adapter() {
    let config = ImessageAccountConfig::new("test-config");

    // Ensure we have a valid backend for testing
    #[cfg(target_os = "macos")]
    let config = config;

    #[cfg(not(target_os = "macos"))]
    let config = {
        let mut config = config;
        // On non-macOS, bluebubbles requires api_url
        config.bluebubbles = BlueBubblesConfig {
            api_url: Some("http://localhost:12345".to_string()),
            ..Default::default()
        };
        config
    };

    let channel = ImessageChannel::new(config).await.unwrap();

    let config_adapter = channel.config();

    let accounts = config_adapter.list_accounts().unwrap();
    assert!(accounts.contains(&"test-config".to_string()));
}

#[tokio::test]
async fn test_imessage_channel_security_adapter() {
    let config = ImessageAccountConfig::new("test-security");

    // Ensure we have a valid backend for testing
    #[cfg(target_os = "macos")]
    let config = config;

    #[cfg(not(target_os = "macos"))]
    let config = {
        let mut config = config;
        // On non-macOS, bluebubbles requires api_url
        config.bluebubbles = BlueBubblesConfig {
            api_url: Some("http://localhost:12345".to_string()),
            ..Default::default()
        };
        config
    };

    let channel = ImessageChannel::new(config).await.unwrap();

    let security = channel.security();

    // Security adapter should be available
    assert!(security.is_some());
}

#[tokio::test]
async fn test_imessage_channel_disconnected_state() {
    let config = ImessageAccountConfig::new("test-disconnected");

    // Ensure we have a valid backend for testing
    #[cfg(target_os = "macos")]
    let config = config;

    #[cfg(not(target_os = "macos"))]
    let config = {
        let mut config = config;
        // On non-macOS, bluebubbles requires api_url
        config.bluebubbles = BlueBubblesConfig {
            api_url: Some("http://localhost:12345".to_string()),
            ..Default::default()
        };
        config
    };

    let channel = ImessageChannel::new(config).await.unwrap();

    // Channel should be disconnected initially
    assert!(!channel.is_connected());
}

#[tokio::test]
async fn test_imessage_account_config_validation() {
    // Valid AppleScript config
    let config = ImessageAccountConfig {
        backend: "applescript".to_string(),
        ..Default::default()
    };

    #[cfg(target_os = "macos")]
    {
        // On macOS, this should succeed (or fail only if osascript is missing)
        let result = config.validate();
        // Don't assert success as osascript might not exist in test environment
        let _ = result;
    }

    #[cfg(not(target_os = "macos"))]
    {
        // On non-macOS, AppleScript should fail
        assert!(config.validate().is_err());
    }

    // Valid BlueBubbles config
    let config = ImessageAccountConfig {
        backend: "bluebubbles".to_string(),
        bluebubbles: crate::BlueBubblesConfig {
            api_url: Some("http://localhost:12345".to_string()),
            ..Default::default()
        },
        ..Default::default()
    };

    assert!(config.validate().is_ok());
}

#[tokio::test]
async fn test_imessage_parse_message_dm() {
    let json = serde_json::json!({
        "guid": "msg123",
        "address": "+1234567890",
        "text": "Hello!",
        "is_from_me": false,
        "date": 1234567890
    });

    let result = aisopod_channel_imessage::parse_imessage_message(json, "test", "imessage");

    assert!(result.is_ok());
    let message = result.unwrap();

    assert_eq!(message.id, "msg123");
    assert_eq!(message.sender.id, "+1234567890");
}

#[tokio::test]
async fn test_imessage_parse_message_group() {
    let json = serde_json::json!({
        "guid": "msg456",
        "address": "+1234567890",
        "text": "Hello group!",
        "chat_guid": "group789",
        "date": 1234567890
    });

    let result = aisopod_channel_imessage::parse_imessage_message(json, "test", "imessage");

    assert!(result.is_ok());
    let message = result.unwrap();

    assert_eq!(message.id, "msg456");
    assert_eq!(message.peer.id, "group789");
    assert_eq!(message.peer.kind, PeerKind::Group);
}
