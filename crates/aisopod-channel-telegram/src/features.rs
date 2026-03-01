//! Telegram channel features: typing indicators, group mentions, message editing/deletion.
//!
//! This module provides advanced Telegram features including:
//! - Typing indicators for long-running operations
//! - Group mention detection (@botname)
//! - Message editing and deletion
//! - Bot username caching via get_me()

use crate::TelegramAccount;
use aisopod_channel::message::{MessageTarget, PeerKind};
use anyhow::Result;
use std::sync::Arc;
use teloxide::{
    prelude::*,
    types::{ChatAction, ChatId, MessageId, UserId},
};
use tokio::sync::RwLock;

/// Bot username cache entry
#[derive(Debug, Clone)]
pub struct BotUsername {
    /// The cached username (without @ prefix)
    pub username: String,
    /// The full name as returned by get_me
    pub full_name: String,
    /// When the cache was created (for expiration)
    pub cached_at: chrono::DateTime<chrono::Utc>,
}

impl BotUsername {
    /// Create a new cache entry
    pub fn new(username: String, full_name: String) -> Self {
        Self {
            username,
            full_name,
            cached_at: chrono::Utc::now(),
        }
    }

    /// Check if the cache is still valid (within 1 hour)
    pub fn is_valid(&self) -> bool {
        let now = chrono::Utc::now();
        let hour = chrono::Duration::hours(1);
        (now - self.cached_at) < hour
    }
}

/// Feature handler for Telegram channel
#[derive(Clone)]
pub struct TelegramFeatures {
    /// Cache of bot usernames per account
    username_cache: Arc<RwLock<std::collections::HashMap<String, BotUsername>>>,
}

impl TelegramFeatures {
    /// Create a new TelegramFeatures instance
    pub fn new() -> Self {
        Self {
            username_cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Get or fetch the bot username for an account
    ///
    /// This method caches the username to avoid repeated API calls.
    /// The cache expires after 1 hour.
    pub async fn get_bot_username(&self, account: &TelegramAccount) -> Result<String> {
        let account_id = &account.id;

        // Try to get from cache first
        {
            let cache = self.username_cache.read().await;
            if let Some(entry) = cache.get(account_id) {
                if entry.is_valid() {
                    return Ok(entry.username.clone());
                }
            }
        }

        // Cache miss or expired, fetch fresh
        let bot = account.bot.clone();
        let me = bot
            .get_me()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get bot info: {}", e))?;

        let username = me.username.clone().unwrap_or_default();
        let full_name = format!(
            "{} {}",
            me.first_name,
            me.last_name.clone().unwrap_or_default()
        );

        // Update cache
        {
            let mut cache = self.username_cache.write().await;
            cache.insert(
                account_id.clone(),
                BotUsername::new(username.clone(), full_name),
            );
        }

        Ok(username)
    }

    /// Check if a message in a group requires mentioning the bot
    ///
    /// This checks if the message contains @botname or if it's a reply
    /// to the bot's own message.
    pub async fn needs_mention(
        &self,
        account: &TelegramAccount,
        message: &teloxide::types::Message,
        target: &MessageTarget,
    ) -> Result<bool> {
        // For DMs, mention is never required
        if target.peer.kind == PeerKind::User {
            return Ok(false);
        }

        // Check if bot was mentioned
        let bot_username = self.get_bot_username(account).await?;

        if let Some(text) = message.text() {
            // Check for @botname mention
            let mention = format!("@{}", bot_username);
            if text.contains(&mention) {
                return Ok(false); // Mentioned, no need to skip
            }

            // Also check for mention without @ (just botname in some cases)
            if text.contains(&bot_username) {
                return Ok(false);
            }
        }

        // Check if this is a reply to the bot's message
        if let Some(reply_to) = message.reply_to_message() {
            if let Some(from) = reply_to.from() {
                let me = account.bot.get_me().await?;
                if from.id == me.id {
                    return Ok(false); // Replying to bot's message
                }
            }
        }

        // If we get here, the message doesn't mention the bot
        Ok(true)
    }

    /// Send a typing indicator to show the bot is working
    ///
    /// This sends a "typing" chat action that expires after 5 seconds.
    /// The caller is responsible for renewing it if needed.
    pub async fn send_typing(&self, account: &TelegramAccount, chat_id: i64) -> Result<()> {
        account
            .bot
            .send_chat_action(ChatId(chat_id), ChatAction::Typing)
            .await?;
        Ok(())
    }

    /// Continuously send typing indicators until a future completes
    ///
    /// This starts a background task that sends typing indicators every 4 seconds
    /// until the provided future completes.
    pub async fn send_typing_until<F, T>(
        &self,
        account: &TelegramAccount,
        chat_id: i64,
        future: F,
    ) -> T
    where
        F: std::future::Future<Output = T>,
    {
        let account = account.clone();

        // Spawn a task to send typing indicators
        let typing_handle = tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(4)).await;
                let _ = account
                    .bot
                    .send_chat_action(ChatId(chat_id), ChatAction::Typing)
                    .await;
            }
        });

        // Wait for the main future to complete
        let result = future.await;

        // Abort the typing task
        typing_handle.abort();

        result
    }

    /// Edit an existing message
    ///
    /// # Arguments
    /// * `account` - The Telegram account to use
    /// * `chat_id` - The chat ID where the message exists
    /// * `message_id` - The ID of the message to edit
    /// * `new_text` - The new text content
    ///
    /// # Returns
    /// * `Ok(())` - Message was edited successfully
    /// * `Err(anyhow::Error)` - An error if editing fails
    pub async fn edit_message(
        &self,
        account: &TelegramAccount,
        chat_id: i64,
        message_id: i64,
        new_text: &str,
    ) -> Result<()> {
        account
            .bot
            .edit_message_text(ChatId(chat_id), MessageId(message_id as i32), new_text)
            .parse_mode(account.config.parse_mode.clone())
            .await?;
        Ok(())
    }

    /// Delete a message
    ///
    /// # Arguments
    /// * `account` - The Telegram account to use
    /// * `chat_id` - The chat ID where the message exists
    /// * `message_id` - The ID of the message to delete
    ///
    /// # Returns
    /// * `Ok(())` - Message was deleted successfully
    /// * `Err(anyhow::Error)` - An error if deletion fails
    pub async fn delete_message(
        &self,
        account: &TelegramAccount,
        chat_id: i64,
        message_id: i64,
    ) -> Result<()> {
        account
            .bot
            .delete_message(ChatId(chat_id), MessageId(message_id as i32))
            .await?;
        Ok(())
    }

    /// Clear the username cache for a specific account
    pub async fn clear_username_cache(&self, account_id: &str) {
        let mut cache = self.username_cache.write().await;
        cache.remove(account_id);
    }

    /// Clear the entire username cache
    pub async fn clear_all_username_cache(&self) {
        let mut cache = self.username_cache.write().await;
        cache.clear();
    }
}

impl Default for TelegramFeatures {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bot_username_validity() {
        let now = chrono::Utc::now();
        let username = BotUsername::new("testbot".to_string(), "Test Bot".to_string());

        // Immediately after creation, cache should be valid
        assert!(username.is_valid());

        // Test with a recent timestamp
        let recent = BotUsername {
            username: "testbot".to_string(),
            full_name: "Test Bot".to_string(),
            cached_at: now - chrono::Duration::minutes(30),
        };
        assert!(recent.is_valid());

        // Test with an old timestamp
        let old = BotUsername {
            username: "testbot".to_string(),
            full_name: "Test Bot".to_string(),
            cached_at: now - chrono::Duration::hours(2),
        };
        assert!(!old.is_valid());
    }

    #[tokio::test]
    async fn test_username_cache() {
        let features = TelegramFeatures::new();

        // The cache should be empty initially
        let cache = features.username_cache.read().await;
        assert!(cache.is_empty());
        drop(cache);

        // We can't test full functionality without a real bot,
        // but we can verify the cache structure
        let mut cache = features.username_cache.write().await;
        cache.insert(
            "test-account".to_string(),
            BotUsername::new("testbot".to_string(), "Test Bot".to_string()),
        );
        assert_eq!(cache.len(), 1);
    }
}
