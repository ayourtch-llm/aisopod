//! # aisopod-provider
//!
//! Provider abstractions and implementations for LLM backends and AI service providers.
//!
//! ## Overview
//!
//! This crate provides the [`ModelProvider`] trait, which is the primary
//! abstraction for communicating with AI model providers. Every concrete
//! provider (Anthropic, OpenAI, Gemini, Bedrock, Ollama) implements this trait.
//!
//! ## Core Types
//!
//! - [`ModelProvider`] - The main trait for AI providers
//! - [`ChatCompletionRequest`] - Request for chat completion
//! - [`ChatCompletionChunk`] - Streaming response chunk
//! - [`Message`] - A chat message
//! - [`ModelInfo`] - Information about a supported model
//! - [`ProviderHealth`] - Health status of a provider
//!
//! ## Normalization
//!
//! - [`normalize::ProviderError`] - Standard error type for provider operations
//! - [`normalize::map_http_error`] - Map HTTP errors to ProviderError
//! - [`normalize::enforce_alternating_turns`] - Merge consecutive same-role messages
//! - [`normalize::extract_system_prompt`] - Extract system prompt from messages
//! - [`normalize::aggregate_usage`] - Aggregate token usage from streaming chunks
//!
//! ## Model Discovery
//!
//! - [`discovery::ModelCatalog`] - Aggregated model catalog with caching and capability filtering
//!
//! ## Registry
//!
//! - [`ProviderRegistry`] - Central registry for managing provider instances
//! - [`ModelAlias`] - Mapping from alias to provider/model pair
//!
//! ## Example
//!
//! ```ignore
//! use aisopod_provider::{ModelProvider, ChatCompletionRequest, Message, Role, MessageContent, ProviderRegistry};
//!
//! async fn example(provider: &impl ModelProvider) -> anyhow::Result<()> {
//!     let request = ChatCompletionRequest {
//!         model: "gpt-4".to_string(),
//!         messages: vec![Message {
//!             role: Role::User,
//!             content: MessageContent::Text("Hello!".to_string()),
//!             tool_calls: None,
//!             tool_call_id: None,
//!         }],
//!         tools: None,
//!         temperature: None,
//!         max_tokens: None,
//!         stop: None,
//!         stream: true,
//!     };
//!
//!     let response = provider.chat_completion(request).await?;
//!     Ok(())
//! }
//! ```

#![deny(unused_must_use)]

pub mod auth;
pub mod discovery;
pub mod helpers;
pub mod normalize;
pub mod providers;
pub mod registry;
pub mod trait_module;
pub mod types;

// Re-export the main trait and all types for convenience
pub use crate::auth::{AuthProfile, AuthProfileManager, ProfileStatus};
pub use crate::discovery::ModelCatalog;
pub use crate::helpers::{
    create_test_model, create_test_request, create_test_tool, create_test_tool_call, MockProvider,
};
pub use crate::normalize::{
    aggregate_usage, enforce_alternating_turns, extract_system_prompt, map_http_error,
    ProviderError,
};
pub use crate::registry::{ModelAlias, ProviderRegistry};
pub use crate::trait_module::{ChatCompletionStream, ModelProvider};
pub use crate::types::{
    ChatCompletionChunk, ChatCompletionRequest, ContentPart, FinishReason, Message, MessageContent,
    MessageDelta, ModelInfo, ProviderHealth, Role, TokenUsage, ToolCall, ToolDefinition,
};
