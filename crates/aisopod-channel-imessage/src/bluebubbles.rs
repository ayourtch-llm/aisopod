//! BlueBubbles backend implementation for iMessage channel.
//!
//! This module provides communication with BlueBubbles server via HTTP API.
//! BlueBubbles is a third-party server that provides iMessage access on any platform.

use crate::config::{BlueBubblesConfig, ImessageError, ImessageResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};
use url::Url;

/// BlueBubbles API response status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BlueBubblesResponse {
    /// Request was successful
    Success,
    /// Request failed
    Fail,
    /// Request pending
    Pending,
}

/// BlueBubbles API response wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueBubblesResponseWrapper<T> {
    /// Response status
    pub response: BlueBubblesResponse,
    /// Response data
    pub data: Option<T>,
    /// Error message if applicable
    pub error: Option<String>,
}

/// BlueBubbles API endpoints.
#[derive(Debug, Clone)]
pub struct BlueBubblesEndpoints {
    /// Base URL for the BlueBubbles API
    base_url: Url,
}

impl BlueBubblesEndpoints {
    /// Creates new endpoints from the base URL.
    pub fn new(base_url: &str) -> Result<Self, ImessageError> {
        let url = Url::parse(base_url).map_err(|_| ImessageError::InvalidUrl {
            url: base_url.to_string(),
            message: "Invalid BlueBubbles API URL".to_string(),
        })?;

        Ok(Self { base_url: url })
    }

    /// Returns the base URL without trailing slash.
    pub fn base_url(&self) -> &str {
        let url_str = self.base_url.as_str();
        // Remove trailing slash if present
        url_str.strip_suffix('/').unwrap_or(url_str)
    }

    /// Returns the URL for sending a message.
    pub fn send_message(&self) -> Url {
        self.base_url.join("api/v1/message/send").unwrap()
    }

    /// Returns the URL for sending a message to a group.
    pub fn send_message_to_group(&self) -> Url {
        self.base_url.join("api/v1/message/send/group").unwrap()
    }

    /// Returns the URL for sending media.
    pub fn send_media(&self) -> Url {
        self.base_url.join("api/v1/message/sendMedia").unwrap()
    }

    /// Returns the URL for sending media to a group.
    pub fn send_media_to_group(&self) -> Url {
        self.base_url
            .join("api/v1/message/sendMedia/group")
            .unwrap()
    }

    /// Returns the URL for retrieving chat history.
    pub fn chat_history(&self, guid: &str) -> Url {
        self.base_url
            .join(&format!("api/v1/message/getChatHistory/{}", guid))
            .unwrap()
    }

    /// Returns the URL for listing contacts.
    pub fn contacts(&self) -> Url {
        self.base_url.join("api/v1/contact").unwrap()
    }

    /// Returns the URL for listing chats.
    pub fn chats(&self) -> Url {
        self.base_url.join("api/v1/chat").unwrap()
    }

    /// Returns the URL for the WebSocket endpoint.
    pub fn websocket(&self) -> Url {
        let mut ws_url = self.base_url.clone();

        // Convert http/https to ws/wss
        match ws_url.scheme() {
            "http" => {
                ws_url.set_scheme("ws").ok();
                // Remove default port 80 for ws
                if ws_url.port() == Some(80) {
                    ws_url.set_port(None).ok();
                }
            }
            "https" => {
                ws_url.set_scheme("wss").ok();
                // Remove default port 443 for wss
                if ws_url.port() == Some(443) {
                    ws_url.set_port(None).ok();
                }
            }
            _ => {}
        }

        ws_url
    }

    /// Returns the URL for getting message info.
    pub fn message_info(&self, guid: &str) -> Url {
        self.base_url
            .join(&format!("api/v1/message/{}", guid))
            .unwrap()
    }
}

/// BlueBubbles HTTP client.
#[derive(Clone)]
pub struct BlueBubblesClient {
    /// HTTP client
    client: reqwest::Client,
    /// API endpoints
    endpoints: BlueBubblesEndpoints,
    /// API password (if required)
    api_password: Option<String>,
}

impl BlueBubblesClient {
    /// Creates a new BlueBubbles client.
    pub fn new(config: &BlueBubblesConfig) -> Result<Self, ImessageError> {
        let api_url = config
            .api_url
            .as_ref()
            .ok_or_else(|| ImessageError::MissingBlueBubblesUrl)?;

        let endpoints = BlueBubblesEndpoints::new(api_url)?;
        let client = reqwest::Client::new();

        Ok(Self {
            client,
            endpoints,
            api_password: config.api_password.clone(),
        })
    }

    /// Sets up request headers.
    fn setup_headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/json"),
        );

        if let Some(password) = &self.api_password {
            if let Ok(header_value) = reqwest::header::HeaderValue::from_str(password) {
                headers.insert("password", header_value);
            }
        }

        headers
    }

    /// Sends a POST request to the BlueBubbles API.
    async fn post<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        endpoint: &Url,
        body: &T,
    ) -> Result<R, ImessageError> {
        debug!("POST to {}", endpoint);

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/json"),
        );

        if let Some(password) = &self.api_password {
            if let Ok(header_value) = reqwest::header::HeaderValue::from_str(password) {
                headers.insert("password", header_value);
            }
        }

        let response = self
            .client
            .post(endpoint.clone())
            .headers(headers)
            .json(body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();

            warn!(
                "BlueBubbles API error: status={}, error={}",
                status, error_text
            );

            return Err(ImessageError::BlueBubblesApi(format!(
                "HTTP {}: {}",
                status, error_text
            )));
        }

        let result: BlueBubblesResponseWrapper<R> = response.json().await?;

        match result.response {
            BlueBubblesResponse::Success => Ok(result.data.ok_or_else(|| {
                ImessageError::BlueBubblesApi("Expected data in response".to_string())
            })?),
            BlueBubblesResponse::Fail => Err(ImessageError::BlueBubblesApi(
                result.error.unwrap_or_else(|| "Unknown error".to_string()),
            )),
            BlueBubblesResponse::Pending => {
                Err(ImessageError::BlueBubblesApi("Request pending".to_string()))
            }
        }
    }

    /// Sends a GET request to the BlueBubbles API.
    async fn get<R: for<'de> Deserialize<'de>>(&self, endpoint: &Url) -> Result<R, ImessageError> {
        debug!("GET {}", endpoint);

        let response = self
            .client
            .get(endpoint.clone())
            .headers(self.setup_headers())
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();

            warn!(
                "BlueBubbles API error: status={}, error={}",
                status, error_text
            );

            return Err(ImessageError::BlueBubblesApi(format!(
                "HTTP {}: {}",
                status, error_text
            )));
        }

        let result: BlueBubblesResponseWrapper<R> = response.json().await?;

        match result.response {
            BlueBubblesResponse::Success => Ok(result.data.ok_or_else(|| {
                ImessageError::BlueBubblesApi("Expected data in response".to_string())
            })?),
            BlueBubblesResponse::Fail => Err(ImessageError::BlueBubblesApi(
                result.error.unwrap_or_else(|| "Unknown error".to_string()),
            )),
            BlueBubblesResponse::Pending => {
                Err(ImessageError::BlueBubblesApi("Request pending".to_string()))
            }
        }
    }

    /// Sends a text message.
    ///
    /// # Arguments
    /// * `to` - Recipient identifier (phone number or email)
    /// * `text` - Message text
    /// * `group` - Optional group identifier (if sending to a group)
    ///
    /// # Returns
    /// Message GUID if successful
    pub async fn send_text(
        &self,
        to: &str,
        text: &str,
        group: Option<&str>,
    ) -> Result<String, ImessageError> {
        let body = serde_json::json!({
            "destination": to,
            "text": text,
            "group": group.unwrap_or_default()
        });

        let endpoint = if group.is_some() {
            self.endpoints.send_message_to_group()
        } else {
            self.endpoints.send_message()
        };

        let response: BlueBubblesSendMessageResponse = self.post(&endpoint, &body).await?;

        response.message_guid.ok_or_else(|| {
            ImessageError::BlueBubblesApi("Missing message GUID in response".to_string())
        })
    }

    /// Sends media.
    ///
    /// # Arguments
    /// * `to` - Recipient identifier
    /// * `media_path` - Path to the media file
    /// * `mime_type` - MIME type of the media
    /// * `group` - Optional group identifier
    ///
    /// # Returns
    /// Message GUID if successful
    pub async fn send_media(
        &self,
        to: &str,
        media_path: &str,
        mime_type: &str,
        group: Option<&str>,
    ) -> Result<String, ImessageError> {
        // Read the media file
        let data = tokio::fs::read(media_path)
            .await
            .map_err(|e| ImessageError::MediaError(format!("Failed to read media file: {}", e)))?;

        // Convert to base64
        let data_base64 = base64::encode(data);

        let body = serde_json::json!({
            "destination": to,
            "data": data_base64,
            "mimeType": mime_type,
            "group": group.unwrap_or_default()
        });

        let endpoint = if group.is_some() {
            self.endpoints.send_media_to_group()
        } else {
            self.endpoints.send_media()
        };

        let response: BlueBubblesSendMessageResponse = self.post(&endpoint, &body).await?;

        response.message_guid.ok_or_else(|| {
            ImessageError::BlueBubblesApi("Missing message GUID in response".to_string())
        })
    }

    /// Retrieves chat history.
    ///
    /// # Arguments
    /// * `guid` - Chat identifier
    /// * `limit` - Maximum number of messages to retrieve
    ///
    /// # Returns
    /// List of messages
    pub async fn get_chat_history(
        &self,
        guid: &str,
        limit: usize,
    ) -> Result<Vec<BlueBubblesMessage>, ImessageError> {
        let mut endpoint = self.endpoints.chat_history(guid);

        // Add query parameters
        {
            let mut query_pairs = endpoint.query_pairs_mut();
            query_pairs.append_pair("limit", &limit.to_string());
        }

        let response: BlueBubblesChatHistoryResponse = self.get(&endpoint).await?;

        Ok(response.messages.unwrap_or_default())
    }

    /// Lists all contacts.
    ///
    /// # Returns
    /// List of contacts
    pub async fn list_contacts(&self) -> Result<Vec<BlueBubblesContact>, ImessageError> {
        let endpoint = self.endpoints.contacts();
        let response: BlueBubblesContactListResponse = self.get(&endpoint).await?;

        Ok(response.contacts.unwrap_or_default())
    }

    /// Lists all chats.
    ///
    /// # Returns
    /// List of chats
    pub async fn list_chats(&self) -> Result<Vec<BlueBubblesChat>, ImessageError> {
        let endpoint = self.endpoints.chats();
        let response: BlueBubblesChatListResponse = self.get(&endpoint).await?;

        Ok(response.chats.unwrap_or_default())
    }
}

/// Request/response types for BlueBubbles API.

/// Response for sending a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueBubblesSendMessageResponse {
    /// Message GUID
    #[serde(rename = "messageGuid")]
    pub message_guid: Option<String>,
    /// Success flag
    pub success: Option<bool>,
}

/// Message data structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueBubblesMessage {
    /// Message GUID
    #[serde(rename = "guid")]
    pub guid: String,
    /// Chat GUID
    #[serde(rename = "chatGuid")]
    pub chat_guid: String,
    /// Sender identifier
    #[serde(rename = "sender")]
    pub sender: Option<String>,
    /// Text content
    pub text: Option<String>,
    /// Message date
    #[serde(rename = "date")]
    pub date: Option<String>,
    /// Whether the message was sent by us
    #[serde(rename = "isFromMe")]
    pub is_from_me: Option<bool>,
    /// Media attachments
    #[serde(rename = "attachments")]
    pub attachments: Option<Vec<BlueBubblesAttachment>>,
    /// Message type
    #[serde(rename = "messageType")]
    pub message_type: Option<String>,
}

/// Attachment data structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueBubblesAttachment {
    /// Attachment GUID
    #[serde(rename = "guid")]
    pub guid: String,
    /// File path
    pub path: Option<String>,
    /// MIME type
    #[serde(rename = "mimeType")]
    pub mime_type: Option<String>,
    /// File size
    #[serde(rename = "size")]
    pub size: Option<u64>,
}

/// Contact data structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueBubblesContact {
    /// Contact identifier
    #[serde(rename = "id")]
    pub id: String,
    /// Display name
    #[serde(rename = "displayName")]
    pub display_name: String,
    /// Phone numbers
    #[serde(rename = "phoneNumbers")]
    pub phone_numbers: Option<Vec<BlueBubblesPhoneNumber>>,
    /// Email addresses
    #[serde(rename = "emailAddresses")]
    pub email_addresses: Option<Vec<BlueBubblesEmailAddress>>,
}

/// Phone number structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueBubblesPhoneNumber {
    /// Phone number
    pub number: String,
    /// Label (e.g., "home", "mobile")
    pub label: Option<String>,
}

/// Email address structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueBubblesEmailAddress {
    /// Email address
    pub address: String,
    /// Label (e.g., "home", "work")
    pub label: Option<String>,
}

/// Chat data structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueBubblesChat {
    /// Chat GUID
    #[serde(rename = "guid")]
    pub guid: String,
    /// Chat name
    pub name: Option<String>,
    /// Participants
    #[serde(rename = "participants")]
    pub participants: Option<Vec<String>>,
    /// Last message date
    #[serde(rename = "lastMessageDate")]
    pub last_message_date: Option<String>,
    /// Whether it's a group chat
    #[serde(rename = "isGroupChat")]
    pub is_group_chat: Option<bool>,
}

/// Response for chat history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueBubblesChatHistoryResponse {
    /// Messages
    pub messages: Option<Vec<BlueBubblesMessage>>,
    /// Success flag
    pub success: Option<bool>,
}

/// Response for contact list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueBubblesContactListResponse {
    /// Contacts
    pub contacts: Option<Vec<BlueBubblesContact>>,
    /// Success flag
    pub success: Option<bool>,
}

/// Response for chat list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueBubblesChatListResponse {
    /// Chats
    pub chats: Option<Vec<BlueBubblesChat>>,
    /// Success flag
    pub success: Option<bool>,
}

/// BlueBubbles backend implementation.
#[derive(Clone)]
pub struct BlueBubblesBackend {
    /// HTTP client
    client: BlueBubblesClient,
    /// Configuration
    config: BlueBubblesConfig,
    /// Whether connected
    connected: bool,
}

impl BlueBubblesBackend {
    /// Creates a new BlueBubbles backend.
    pub fn new(config: BlueBubblesConfig) -> Result<Self, ImessageError> {
        let client = BlueBubblesClient::new(&config)?;

        Ok(Self {
            client,
            config,
            connected: false,
        })
    }

    /// Returns the HTTP client.
    pub fn client(&self) -> &BlueBubblesClient {
        &self.client
    }

    /// Returns the configuration.
    pub fn config(&self) -> &BlueBubblesConfig {
        &self.config
    }

    /// Checks if connected.
    pub fn is_connected(&self) -> bool {
        self.connected
    }
}

/// BlueBubbles backend trait.
#[async_trait]
pub trait BlueBubblesBackendImpl: Send + Sync {
    /// Connect to BlueBubbles server.
    async fn connect(&mut self) -> ImessageResult<()>;

    /// Disconnect from BlueBubbles server.
    async fn disconnect(&mut self) -> ImessageResult<()>;

    /// Send a text message.
    async fn send_text(&self, to: &str, text: &str) -> ImessageResult<String>;

    /// Send a text message to a group.
    async fn send_text_to_group(&self, group_id: &str, text: &str) -> ImessageResult<String>;

    /// Send media.
    async fn send_media(
        &self,
        to: &str,
        media_path: &str,
        mime_type: &str,
    ) -> ImessageResult<String>;

    /// Send media to a group.
    async fn send_media_to_group(
        &self,
        group_id: &str,
        media_path: &str,
        mime_type: &str,
    ) -> ImessageResult<String>;
}

#[async_trait]
impl BlueBubblesBackendImpl for BlueBubblesBackend {
    async fn connect(&mut self) -> ImessageResult<()> {
        info!(
            "Connecting to BlueBubbles server at {}",
            self.config.api_url.as_ref().unwrap()
        );

        // Test connection
        match self.client().list_chats().await {
            Ok(_) => {
                self.connected = true;
                info!("Connected to BlueBubbles server");
                Ok(())
            }
            Err(e) => {
                warn!("Failed to connect to BlueBubbles: {}", e);
                self.connected = false;
                Err(e)
            }
        }
    }

    async fn disconnect(&mut self) -> ImessageResult<()> {
        self.connected = false;
        info!("Disconnected from BlueBubbles server");
        Ok(())
    }

    async fn send_text(&self, to: &str, text: &str) -> ImessageResult<String> {
        if !self.connected {
            return Err(ImessageError::ConnectionError(
                "Not connected to BlueBubbles server".to_string(),
            ));
        }

        self.client.send_text(to, text, None).await
    }

    async fn send_text_to_group(&self, group_id: &str, text: &str) -> ImessageResult<String> {
        if !self.connected {
            return Err(ImessageError::ConnectionError(
                "Not connected to BlueBubbles server".to_string(),
            ));
        }

        self.client.send_text(group_id, text, Some(group_id)).await
    }

    async fn send_media(
        &self,
        to: &str,
        media_path: &str,
        mime_type: &str,
    ) -> ImessageResult<String> {
        if !self.connected {
            return Err(ImessageError::ConnectionError(
                "Not connected to BlueBubbles server".to_string(),
            ));
        }

        self.client
            .send_media(to, media_path, mime_type, None)
            .await
    }

    async fn send_media_to_group(
        &self,
        group_id: &str,
        media_path: &str,
        mime_type: &str,
    ) -> ImessageResult<String> {
        if !self.connected {
            return Err(ImessageError::ConnectionError(
                "Not connected to BlueBubbles server".to_string(),
            ));
        }

        self.client
            .send_media(group_id, media_path, mime_type, Some(group_id))
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bluebubbles_endpoints() {
        let endpoints = BlueBubblesEndpoints::new("http://localhost:12345").unwrap();

        assert_eq!(endpoints.base_url(), "http://localhost:12345");
        assert_eq!(
            endpoints.send_message().to_string(),
            "http://localhost:12345/api/v1/message/send"
        );
        assert_eq!(
            endpoints.send_message_to_group().to_string(),
            "http://localhost:12345/api/v1/message/send/group"
        );
    }

    #[test]
    fn test_bluebubbles_endpoints_invalid_url() {
        let result = BlueBubblesEndpoints::new("not-a-url");
        assert!(matches!(result, Err(ImessageError::InvalidUrl { .. })));
    }

    #[test]
    fn test_bluebubbles_endpoints_websocket() {
        let endpoints = BlueBubblesEndpoints::new("http://localhost:12345").unwrap();
        let ws_url = endpoints.websocket();

        assert_eq!(ws_url.scheme(), "ws");
        assert_eq!(ws_url.host_str(), Some("localhost"));
        assert_eq!(ws_url.port(), Some(12345));
    }

    #[test]
    fn test_bluebubbles_endpoints_https_websocket() {
        let endpoints = BlueBubblesEndpoints::new("https://example.com:443").unwrap();
        let ws_url = endpoints.websocket();

        assert_eq!(ws_url.scheme(), "wss");
        assert_eq!(ws_url.host_str(), Some("example.com"));
        assert_eq!(ws_url.port(), None);
    }

    #[test]
    fn test_bluebubbles_endpoints_chat_history() {
        let endpoints = BlueBubblesEndpoints::new("http://localhost:12345").unwrap();
        let url = endpoints.chat_history("chat123");

        assert_eq!(
            url.to_string(),
            "http://localhost:12345/api/v1/message/getChatHistory/chat123"
        );
    }

    #[test]
    fn test_bluebubbles_endpoints_contacts() {
        let endpoints = BlueBubblesEndpoints::new("http://localhost:12345").unwrap();
        let url = endpoints.contacts();

        assert_eq!(url.to_string(), "http://localhost:12345/api/v1/contact");
    }

    #[test]
    fn test_bluebubbles_endpoints_chats() {
        let endpoints = BlueBubblesEndpoints::new("http://localhost:12345").unwrap();
        let url = endpoints.chats();

        assert_eq!(url.to_string(), "http://localhost:12345/api/v1/chat");
    }

    #[test]
    fn test_bluebubbles_endpoints_message_info() {
        let endpoints = BlueBubblesEndpoints::new("http://localhost:12345").unwrap();
        let url = endpoints.message_info("msg123");

        assert_eq!(
            url.to_string(),
            "http://localhost:12345/api/v1/message/msg123"
        );
    }
}
