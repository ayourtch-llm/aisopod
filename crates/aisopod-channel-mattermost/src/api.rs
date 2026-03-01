//! Mattermost REST API client.
//!
//! This module provides a client for interacting with the Mattermost REST API.
//! It handles authentication and provides methods for common operations.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error, instrument};

/// Error types for Mattermost API operations.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    /// HTTP request failed
    #[error("HTTP error: {0}")]
    Http(String),
    /// Response parsing failed
    #[error("Parse error: {0}")]
    Parse(String),
    /// API returned an error response
    #[error("API error: {0}")]
    ApiError(String),
    /// Authentication failed
    #[error("Authentication error: {0}")]
    Auth(String),
}

impl From<reqwest::Error> for ApiError {
    fn from(err: reqwest::Error) -> Self {
        ApiError::Http(err.to_string())
    }
}

/// API response wrapper for Mattermost API.
#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    /// The response data if successful
    pub data: Option<T>,
    /// Error information if the request failed
    pub error: Option<ApiErrorResponse>,
}

/// Error response from the Mattermost API.
#[derive(Debug, Deserialize)]
pub struct ApiErrorResponse {
    /// Error message
    pub message: String,
    /// Error id (e.g., "api.channel.create_channel.permissions.app_error")
    pub id: Option<String>,
    /// Error detail
    pub detail: Option<String>,
}

/// Channel information from Mattermost API.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Channel {
    /// Unique identifier for the channel
    pub id: String,
    /// The name of the channel (unique per team)
    pub name: String,
    /// The display name of the channel
    pub display_name: String,
    /// The team ID this channel belongs to
    pub team_id: String,
    /// Whether this is a direct message channel
    #[serde(default)]
    pub type_: ChannelType,
    /// Whether this channel is grouped
    #[serde(default)]
    pub group: bool,
    /// The header of the channel
    #[serde(default)]
    pub header: String,
    /// The purpose of the channel
    #[serde(default)]
    pub purpose: String,
    /// The last post time
    #[serde(default)]
    pub last_post_at: i64,
    /// The total number of members
    #[serde(default)]
    pub total_msg_count: i64,
    /// Whether the channel is archived
    #[serde(default)]
    pub deleted: bool,
    /// Optional user IDs for direct message channels
    #[serde(default)]
    pub userIds: Option<Vec<String>>,
}

/// Channel type enumeration.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ChannelType {
    /// Private channel
    #[default]
    Private,
    /// Public channel
    Public,
    /// Direct message channel
    Direct,
    /// Group message channel
    Group,
}

/// Post information from Mattermost API.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Post {
    /// Unique identifier for the post
    pub id: String,
    /// The channel ID this post belongs to
    pub channel_id: String,
    /// The user ID who created the post
    pub user_id: String,
    /// The message content
    pub message: String,
    /// The creation timestamp
    pub create_at: i64,
    /// The update timestamp
    pub update_at: i64,
    /// Optional edit timestamp
    #[serde(default)]
    pub edit_at: i64,
    /// Whether this post has been deleted
    #[serde(default)]
    pub delete_at: i64,
    /// Whether this is a system message
    #[serde(default)]
    pub is_pinned: bool,
    /// Optional parent post ID for thread replies
    #[serde(default)]
    pub root_id: String,
    /// Optional user IDs mentioned in the post
    #[serde(default)]
    pub has_reactions: bool,
    /// Attachments
    #[serde(default)]
    pub props: serde_json::Value,
}

/// User information from Mattermost API.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct User {
    /// Unique identifier for the user
    pub id: String,
    /// The username (unique per team)
    pub username: String,
    /// The display name
    #[serde(default)]
    pub display_name: String,
    /// The email address
    #[serde(default)]
    pub email: String,
    /// Whether this is a bot account
    #[serde(default)]
    pub is_bot: bool,
    /// Whether this user is deleted
    #[serde(default)]
    pub delete_at: i64,
}

/// Mattermost API client.
#[derive(Clone)]
pub struct MattermostApi {
    base_url: String,
    token: String,
    http: reqwest::Client,
}

impl MattermostApi {
    /// Create a new Mattermost API client.
    ///
    /// # Arguments
    ///
    /// * `base_url` - The base URL of the Mattermost server (e.g., "https://mattermost.example.com")
    /// * `token` - The authentication token
    ///
    /// # Returns
    ///
    /// * `Ok(MattermostApi)` - The API client
    /// * `Err(anyhow::Error)` - An error if the URL is invalid
    pub fn new(base_url: String, token: String) -> Result<Self> {
        // Clean up the base URL - ensure no trailing slash
        let base_url = base_url.trim_end_matches('/').to_string();

        Ok(Self {
            base_url,
            token,
            http: reqwest::Client::new(),
        })
    }

    /// Get the base URL.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Create a new post in a channel.
    ///
    /// # Arguments
    ///
    /// * `channel_id` - The ID of the channel to post in
    /// * `message` - The message content
    ///
    /// # Returns
    ///
    /// * `Ok(Post)` - The created post
    /// * `Err(ApiError)` - An error if the request fails
    #[instrument(skip(self, message))]
    pub async fn create_post(&self, channel_id: &str, message: &str) -> Result<Post, ApiError> {
        let url = format!("{}/api/v4/posts", self.base_url);
        debug!(
            "Creating post in channel {} with message: {}",
            channel_id, message
        );

        let resp = self
            .http
            .post(&url)
            .bearer_auth(&self.token)
            .json(&serde_json::json!({
                "channel_id": channel_id,
                "message": message
            }))
            .send()
            .await
            .map_err(ApiError::from)?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            error!("create_post failed with status {}: {}", status, body);
            return Err(ApiError::Http(format!("HTTP {} {}", status, body)));
        }

        let post: Post = resp.json().await.map_err(ApiError::from)?;
        Ok(post)
    }

    /// Get a channel by its name within a team.
    ///
    /// # Arguments
    ///
    /// * `team_id` - The ID of the team
    /// * `name` - The name of the channel
    ///
    /// # Returns
    ///
    /// * `Ok(Channel)` - The channel information
    /// * `Err(ApiError)` - An error if the channel is not found
    #[instrument(skip(self))]
    pub async fn get_channel_by_name(
        &self,
        team_id: &str,
        name: &str,
    ) -> Result<Channel, ApiError> {
        let url = format!(
            "{}/api/v4/teams/{}/channels/name/{}",
            self.base_url, team_id, name
        );
        debug!("Getting channel {}/{}", team_id, name);

        let resp = self
            .http
            .get(&url)
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(ApiError::from)?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            error!(
                "get_channel_by_name failed with status {}: {}",
                status, body
            );
            return Err(ApiError::Http(format!("HTTP {} {}", status, body)));
        }

        let channel: Channel = resp.json().await.map_err(ApiError::from)?;
        Ok(channel)
    }

    /// Create a direct message channel with a user.
    ///
    /// # Arguments
    ///
    /// * `user_ids` - The IDs of the two users (the current user and the target user)
    ///
    /// # Returns
    ///
    /// * `Ok(Channel)` - The direct message channel
    /// * `Err(ApiError)` - An error if the request fails
    #[instrument(skip(self))]
    pub async fn create_direct_channel(&self, user_ids: [&str; 2]) -> Result<Channel, ApiError> {
        let url = format!("{}/api/v4/channels/direct", self.base_url);
        debug!("Creating direct channel with users: {:?}", user_ids);

        let resp = self
            .http
            .post(&url)
            .bearer_auth(&self.token)
            .json(&user_ids)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            error!(
                "create_direct_channel failed with status {}: {}",
                status, body
            );
            return Err(ApiError::Http(format!("HTTP {} {}", status, body)));
        }

        let channel: Channel = resp.json().await.map_err(ApiError::from)?;
        Ok(channel)
    }

    /// Get a channel by its ID.
    ///
    /// # Arguments
    ///
    /// * `channel_id` - The ID of the channel
    ///
    /// # Returns
    ///
    /// * `Ok(Channel)` - The channel information
    /// * `Err(ApiError)` - An error if the channel is not found
    #[instrument(skip(self))]
    pub async fn get_channel(&self, channel_id: &str) -> Result<Channel, ApiError> {
        let url = format!("{}/api/v4/channels/{}", self.base_url, channel_id);
        debug!("Getting channel {}", channel_id);

        let resp = self
            .http
            .get(&url)
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(ApiError::from)?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            error!("get_channel failed with status {}: {}", status, body);
            return Err(ApiError::Http(format!("HTTP {} {}", status, body)));
        }

        let channel: Channel = resp.json().await.map_err(ApiError::from)?;
        Ok(channel)
    }

    /// Get the current user's information.
    ///
    /// # Returns
    ///
    /// * `Ok(User)` - The current user's information
    /// * `Err(ApiError)` - An error if the request fails
    #[instrument(skip(self))]
    pub async fn get_current_user(&self) -> Result<User, ApiError> {
        let url = format!("{}/api/v4/users/me", self.base_url);
        debug!("Getting current user");

        let resp = self
            .http
            .get(&url)
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(ApiError::from)?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            error!("get_current_user failed with status {}: {}", status, body);
            return Err(ApiError::Http(format!("HTTP {} {}", status, body)));
        }

        let user: User = resp.json().await.map_err(ApiError::from)?;
        Ok(user)
    }

    /// List channels in a team.
    ///
    /// # Arguments
    ///
    /// * `team_id` - The ID of the team
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Channel>)` - List of channels in the team
    /// * `Err(ApiError)` - An error if the request fails
    #[instrument(skip(self))]
    pub async fn list_channels(&self, team_id: &str) -> Result<Vec<Channel>, ApiError> {
        let url = format!("{}/api/v4/teams/{}/channels", self.base_url, team_id);
        debug!("Listing channels for team {}", team_id);

        let resp = self
            .http
            .get(&url)
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(ApiError::from)?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            error!("list_channels failed with status {}: {}", status, body);
            return Err(ApiError::Http(format!("HTTP {} {}", status, body)));
        }

        let channels: Vec<Channel> = resp.json().await.map_err(ApiError::from)?;
        Ok(channels)
    }

    /// Get the team by its ID.
    ///
    /// # Arguments
    ///
    /// * `team_id` - The ID of the team
    ///
    /// # Returns
    ///
    /// * `Ok(Team)` - The team information
    /// * `Err(ApiError)` - An error if the team is not found
    #[instrument(skip(self))]
    pub async fn get_team(&self, team_id: &str) -> Result<Team, ApiError> {
        let url = format!("{}/api/v4/teams/{}", self.base_url, team_id);
        debug!("Getting team {}", team_id);

        let resp = self
            .http
            .get(&url)
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(ApiError::from)?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            error!("get_team failed with status {}: {}", status, body);
            return Err(ApiError::Http(format!("HTTP {} {}", status, body)));
        }

        let team: Team = resp.json().await.map_err(ApiError::from)?;
        Ok(team)
    }

    /// Get team by name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the team
    ///
    /// # Returns
    ///
    /// * `Ok(Team)` - The team information
    /// * `Err(ApiError)` - An error if the team is not found
    #[instrument(skip(self))]
    pub async fn get_team_by_name(&self, name: &str) -> Result<Team, ApiError> {
        let url = format!("{}/api/v4/teams/name/{}", self.base_url, name);
        debug!("Getting team by name: {}", name);

        let resp = self
            .http
            .get(&url)
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(ApiError::from)?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            error!("get_team_by_name failed with status {}: {}", status, body);
            return Err(ApiError::Http(format!("HTTP {} {}", status, body)));
        }

        let team: Team = resp.json().await.map_err(ApiError::from)?;
        Ok(team)
    }

    /// Get posts from a channel.
    ///
    /// # Arguments
    ///
    /// * `channel_id` - The ID of the channel
    /// * `page` - The page number (0-indexed)
    /// * `per_page` - Number of posts per page
    ///
    /// # Returns
    ///
    /// * `Ok(PostsResponse)` - The posts and pagination info
    /// * `Err(ApiError)` - An error if the request fails
    #[instrument(skip(self))]
    pub async fn get_posts(
        &self,
        channel_id: &str,
        page: u32,
        per_page: u32,
    ) -> Result<PostsResponse, ApiError> {
        let url = format!(
            "{}/api/v4/channels/{}/posts?page={}&per_page={}",
            self.base_url, channel_id, page, per_page
        );
        debug!(
            "Getting posts from channel {} (page {}, per_page {})",
            channel_id, page, per_page
        );

        let resp = self
            .http
            .get(&url)
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(ApiError::from)?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            error!("get_posts failed with status {}: {}", status, body);
            return Err(ApiError::Http(format!("HTTP {} {}", status, body)));
        }

        let posts: PostsResponse = resp.json().await.map_err(ApiError::from)?;
        Ok(posts)
    }
}

/// Response for getting posts from a channel.
#[derive(Debug, Deserialize)]
pub struct PostsResponse {
    /// Ordered list of post IDs
    pub order: Vec<String>,
    /// Posts by ID
    pub posts: HashMap<String, Post>,
}

/// Team information from Mattermost API.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Team {
    /// Unique identifier for the team
    pub id: String,
    /// The name of the team (unique)
    pub name: String,
    /// The display name of the team
    #[serde(default)]
    pub display_name: String,
    /// The description of the team
    #[serde(default)]
    pub description: String,
    /// The creation timestamp
    #[serde(default)]
    pub create_at: i64,
    /// The last update timestamp
    #[serde(default)]
    pub update_at: i64,
    /// The last delete timestamp
    #[serde(default)]
    pub delete_at: i64,
    /// Whether this is a closed team
    #[serde(default)]
    pub allowed_domains: Vec<String>,
    /// Whether this is a closed team
    #[serde(default)]
    pub closed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_client_new() {
        let api = MattermostApi::new(
            "https://mattermost.example.com".to_string(),
            "test-token".to_string(),
        )
        .unwrap();
        assert_eq!(api.base_url(), "https://mattermost.example.com");
    }

    #[test]
    fn test_api_client_clean_url() {
        let api = MattermostApi::new(
            "https://mattermost.example.com/".to_string(),
            "test-token".to_string(),
        )
        .unwrap();
        assert_eq!(api.base_url(), "https://mattermost.example.com");
    }

    #[test]
    fn test_api_client_clean_double_slash() {
        let api = MattermostApi::new(
            "https://mattermost.example.com//".to_string(),
            "test-token".to_string(),
        )
        .unwrap();
        assert_eq!(api.base_url(), "https://mattermost.example.com");
    }
}
