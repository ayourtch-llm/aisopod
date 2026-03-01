//! Integration tests for the aisopod-channel crate.
//!
//! These tests verify the end-to-end message flow for each Tier 1 channel
//! (Telegram, Discord, WhatsApp, Slack) using mock API servers.

#![deny(unused_must_use)]

pub mod cross_channel;
pub mod discord;
pub mod mock_servers;
pub mod slack;
pub mod telegram;
pub mod whatsapp;
