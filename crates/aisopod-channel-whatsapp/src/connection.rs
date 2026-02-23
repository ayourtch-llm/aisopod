//! WhatsApp Business API connection types and configuration.

use serde::{Deserialize, Serialize};

/// WhatsApp connection mode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WhatsAppMode {
    /// WhatsApp Business API mode (Cloud API).
    #[serde(rename = "business_api")]
    BusinessApi,
    /// Baileys bridge mode (for future support).
    #[serde(rename = "baileys-bridge")]
    BaileysBridge,
}

/// Configuration for a WhatsApp Business API account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppAccountConfig {
    /// The connection mode (e.g., "business_api" or "baileys_bridge").
    pub mode: WhatsAppMode,
    /// The WhatsApp Business API access token.
    pub api_token: Option<String>,
    /// The WhatsApp Business Account ID.
    pub business_account_id: Option<String>,
    /// The phone number ID to use for sending/receiving messages.
    pub phone_number_id: Option<String>,
    /// The webhook verify token for challenge-response validation.
    pub webhook_verify_token: Option<String>,
    /// Optional list of allowed phone numbers (if empty, all numbers are allowed).
    pub allowed_numbers: Option<Vec<String>>,
}

impl Default for WhatsAppAccountConfig {
    fn default() -> Self {
        Self {
            mode: WhatsAppMode::BusinessApi,
            api_token: None,
            business_account_id: None,
            phone_number_id: None,
            webhook_verify_token: None,
            allowed_numbers: None,
        }
    }
}

impl WhatsAppAccountConfig {
    /// Creates a new WhatsAppAccountConfig with the given API token.
    ///
    /// # Arguments
    ///
    /// * `api_token` - The WhatsApp Business API access token
    /// * `phone_number_id` - The phone number ID for this account
    /// * `webhook_verify_token` - The token used for webhook verification
    pub fn new(api_token: String, phone_number_id: String, webhook_verify_token: String) -> Self {
        Self {
            mode: WhatsAppMode::BusinessApi,
            api_token: Some(api_token),
            business_account_id: None,
            phone_number_id: Some(phone_number_id),
            webhook_verify_token: Some(webhook_verify_token),
            allowed_numbers: None,
        }
    }
}

/// API response for phone number details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppPhoneNumberDetails {
    /// The phone number in E.164 format.
    pub id: String,
    /// The phone number in display format.
    pub display_phone_number: String,
    /// The verified name of the phone number.
    pub verified_name: Option<String>,
}

/// API response for business account details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppBusinessAccountDetails {
    /// The business account ID.
    pub id: String,
    /// The display name of the business account.
    pub display_name: String,
    /// The verified name of the business account.
    pub verified_name: Option<String>,
    /// The country code where the business is registered.
    pub country: Option<String>,
    /// The business message types setting.
    pub message_types: Option<Vec<String>>,
}

/// Error types for WhatsApp API operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhatsAppError {
    /// Invalid API token.
    InvalidToken,
    /// Network error.
    NetworkError(String),
    /// HTTP error with status code and message.
    HttpError(u16, String),
    /// Invalid configuration.
    InvalidConfig(String),
    /// Missing required configuration.
    MissingConfig(String),
}

impl std::fmt::Display for WhatsAppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WhatsAppError::InvalidToken => write!(f, "Invalid API token"),
            WhatsAppError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            WhatsAppError::HttpError(code, msg) => {
                write!(f, "HTTP error {}: {}", code, msg)
            }
            WhatsAppError::InvalidConfig(msg) => write!(f, "Invalid configuration: {}", msg),
            WhatsAppError::MissingConfig(field) => {
                write!(f, "Missing required configuration: {}", field)
            }
        }
    }
}

impl std::error::Error for WhatsAppError {}

impl From<reqwest::Error> for WhatsAppError {
    fn from(err: reqwest::Error) -> Self {
        WhatsAppError::NetworkError(err.to_string())
    }
}
