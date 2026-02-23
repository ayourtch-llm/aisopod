//! Connection handling for Slack API.
//!
//! This module provides the HTTP client and connection management
//! for interacting with the Slack Web API.

use anyhow::Result;
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// Slack Web API client handle.
///
/// This struct wraps the HTTP client with the bot token for making
/// authenticated requests to the Slack Web API.
#[derive(Clone)]
pub struct SlackClientHandle {
    /// The HTTP client
    client: HttpClient,
    /// The bot token
    bot_token: String,
}

impl SlackClientHandle {
    /// Create a new Slack client handle.
    pub fn new(bot_token: String) -> Self {
        let client = HttpClient::new();
        Self { client, bot_token }
    }

    /// Get the bot token.
    pub fn bot_token(&self) -> &str {
        &self.bot_token
    }

    /// Call the `auth.test` endpoint to verify the bot token.
    ///
    /// This method returns the bot's user ID and other information.
    /// It's used to validate the bot token on startup.
    ///
    /// # Returns
    ///
    /// * `Ok(AuthTestResponse)` - The authentication test response
    /// * `Err(anyhow::Error)` - An error if the request fails
    pub async fn auth_test(&self) -> Result<AuthTestResponse> {
        let url = "https://slack.com/api/auth.test";
        let response = self.client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.bot_token))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            return Err(anyhow::anyhow!("auth.test failed with status {}: {}", status, body));
        }

        let json: AuthTestResponse = response.json().await?;
        Ok(json)
    }

    /// Call the `apps.connections.open` endpoint to initiate Socket Mode.
    ///
    /// This method opens a WebSocket connection URL that can be used
    /// to connect to Slack's Socket Mode protocol.
    ///
    /// # Returns
    ///
    /// * `Ok(AppsConnectionsOpenResponse)` - The connection open response with WebSocket URL
    /// * `Err(anyhow::Error)` - An error if the request fails
    pub async fn apps_connections_open(&self) -> Result<AppsConnectionsOpenResponse> {
        let url = "https://slack.com/api/apps.connections.open";
        let response = self.client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.bot_token))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            return Err(anyhow::anyhow!("apps.connections.open failed with status {}: {}", status, body));
        }

        let json: AppsConnectionsOpenResponse = response.json().await?;
        Ok(json)
    }

    /// Make a generic POST request to the Slack Web API.
    ///
    /// This is a convenience method for making arbitrary POST requests
    /// to the Slack API.
    ///
    /// # Arguments
    ///
    /// * `url` - The API endpoint URL
    /// * `body` - The JSON body to send
    ///
    /// # Returns
    ///
    /// * `Ok(reqwest::Response)` - The HTTP response
    /// * `Err(anyhow::Error)` - An error if the request fails
    pub async fn post(&self, url: &str, body: &serde_json::Value) -> Result<reqwest::Response> {
        let response = self.client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.bot_token))
            .json(body)
            .send()
            .await?;

        Ok(response)
    }

    /// Make a generic GET request to the Slack Web API.
    ///
    /// This is a convenience method for making arbitrary GET requests
    /// to the Slack API.
    ///
    /// # Arguments
    ///
    /// * `url` - The API endpoint URL
    ///
    /// # Returns
    ///
    /// * `Ok(reqwest::Response)` - The HTTP response
    /// * `Err(anyhow::Error)` - An error if the request fails
    pub async fn get(&self, url: &str) -> Result<reqwest::Response> {
        let response = self.client
            .get(url)
            .header("Authorization", format!("Bearer {}", self.bot_token))
            .send()
            .await?;

        Ok(response)
    }
}

/// Response from the `auth.test` endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthTestResponse {
    /// Whether the request was successful
    pub ok: bool,
    /// The bot's user ID
    pub user_id: Option<String>,
    /// The bot's user name
    pub user: Option<String>,
    /// The team ID
    pub team_id: Option<String>,
    /// The team name
    pub team: Option<String>,
    /// The enterprise ID
    pub enterprise_id: Option<String>,
    /// The enterprise name
    pub enterprise_name: Option<String>,
    /// The URL for the Slack workspace
    pub url: Option<String>,
}

impl AuthTestResponse {
    /// Check if the authentication was successful.
    pub fn is_ok(&self) -> bool {
        self.ok
    }
}

/// Response from the `apps.connections.open` endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppsConnectionsOpenResponse {
    /// Whether the request was successful
    pub ok: bool,
    /// The WebSocket URL for Socket Mode
    pub url: Option<String>,
    /// The app ID
    pub app_id: Option<String>,
}

impl AppsConnectionsOpenResponse {
    /// Check if the connection open was successful.
    pub fn is_ok(&self) -> bool {
        self.ok
    }

    /// Get the WebSocket URL, or return an error if missing.
    pub fn get_url(&self) -> Result<String> {
        self.url.clone()
            .ok_or_else(|| anyhow::anyhow!("WebSocket URL not provided in response"))
    }
}

/// Create a new Slack client handle.
///
/// This is a convenience function for creating a client handle from a config.
pub fn create_client(bot_token: &str) -> SlackClientHandle {
    SlackClientHandle::new(bot_token.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_test_response_ok() {
        let response = AuthTestResponse {
            ok: true,
            user_id: Some("U123456".to_string()),
            user: Some("testbot".to_string()),
            team_id: Some("T123456".to_string()),
            team: Some("testteam".to_string()),
            enterprise_id: None,
            enterprise_name: None,
            url: None,
        };
        assert!(response.is_ok());
        assert_eq!(response.user_id, Some("U123456".to_string()));
    }

    #[test]
    fn test_auth_test_response_not_ok() {
        let response = AuthTestResponse {
            ok: false,
            user_id: None,
            user: None,
            team_id: None,
            team: None,
            enterprise_id: None,
            enterprise_name: None,
            url: None,
        };
        assert!(!response.is_ok());
    }

    #[test]
    fn test_apps_connections_open_response_ok() {
        let response = AppsConnectionsOpenResponse {
            ok: true,
            url: Some("wss://socket.slack.com/".to_string()),
            app_id: Some("A123456".to_string()),
        };
        assert!(response.is_ok());
        assert_eq!(response.get_url().unwrap(), "wss://socket.slack.com/");
    }

    #[test]
    fn test_apps_connections_open_response_missing_url() {
        let response = AppsConnectionsOpenResponse {
            ok: true,
            url: None,
            app_id: Some("A123456".to_string()),
        };
        assert!(response.is_ok());
        assert!(response.get_url().is_err());
    }
}
