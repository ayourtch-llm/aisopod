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

pub use api::{
    BoxComponentBuilder, FlexBlockStyle, FlexBuilder, FlexComponent, FlexContainer,
    FlexContainerType, FlexStyles, LineApi, LineMessage, LineResponse, TextComponentBuilder,
    UserProfile,
};
pub use auth::{
    is_token_expired, issue_stateful_token, issue_stateless_token, refresh_token, revoke_token,
    validate_token, TokenExchangeResponse, TokenManager, TokenResponse, TokenValidation,
};
pub use channel::LineChannel;
pub use config::LineAccountConfig;
pub use webhook::{
    contains_follow_event, contains_join_event, contains_message_event, extract_destination,
    extract_first_event, extract_reply_token, extract_source, get_destination, get_group_id,
    get_room_id, get_user_id, parse_webhook_body, verify_signature, BeaconContent, BeaconEvent,
    EventSource, FollowEvent, JoinEvent, LeaveEvent, MessageContent, MessageEvent, PostbackContent,
    PostbackEvent, UnfollowEvent, WebhookEventType, WebhookRequestBody, LINE_SIGNATURE_HEADER,
};
