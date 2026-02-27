//! LINE channel implementation for aisopod.
//!
//! This crate provides integration with the LINE Messaging API, enabling
//! aisopod to send and receive messages with LINE users and groups.

pub mod api;
pub mod auth;
pub mod channel;
pub mod config;
pub mod flex;
pub mod webhook;

pub use api::{LineApi, LineMessage, LineResponse, UserProfile, FlexBuilder, FlexComponent, FlexContainer, FlexContainerType, FlexStyles, FlexBlockStyle, TextComponentBuilder, BoxComponentBuilder};
pub use auth::{issue_stateless_token, validate_token, revoke_token, issue_stateful_token, refresh_token, TokenResponse, TokenExchangeResponse, TokenValidation, is_token_expired, TokenManager};
pub use channel::{LineChannel};
pub use config::{LineAccountConfig};
pub use webhook::{WebhookEventType, MessageEvent, FollowEvent, UnfollowEvent, JoinEvent, LeaveEvent, PostbackEvent, BeaconEvent, EventSource, MessageContent, PostbackContent, BeaconContent, WebhookRequestBody, LINE_SIGNATURE_HEADER, verify_signature, parse_webhook_body, extract_destination, extract_first_event, extract_reply_token, extract_source, get_user_id, get_group_id, get_room_id, get_destination, contains_message_event, contains_follow_event, contains_join_event};
