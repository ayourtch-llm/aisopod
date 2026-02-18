//! Request/Response normalization layer for AI provider communications.
//!
//! This module provides shared normalization logic for converting between
//! internal message formats and provider-specific formats, handling provider
//! quirks, mapping provider error codes to standard error types, and
//! aggregating token usage across providers.

use std::time::Duration;

use crate::types::*;

/// Standard error type for AI provider operations.
///
/// This enum provides a unified error interface that abstracts provider-specific
/// error formats, enabling consistent error handling across all providers.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ProviderError {
    /// Authentication failed with the provider.
    #[error("Authentication failed for provider '{provider}': {message}")]
    AuthenticationFailed {
        provider: String,
        message: String,
    },

    /// Rate limit exceeded with the provider.
    #[error("Rate limit exceeded for provider '{provider}'")]
    RateLimited {
        provider: String,
        retry_after: Option<Duration>,
    },

    /// Invalid request sent to the provider.
    #[error("Invalid request for provider '{provider}': {message}")]
    InvalidRequest {
        provider: String,
        message: String,
    },

    /// Requested model not found at the provider.
    #[error("Model '{model}' not found for provider '{provider}'")]
    ModelNotFound {
        provider: String,
        model: String,
    },

    /// Request exceeds the context length limit.
    #[error("Context length exceeded for provider '{provider}' (max tokens: {max_tokens})")]
    ContextLengthExceeded {
        provider: String,
        max_tokens: u32,
    },

    /// Server-side error from the provider.
    #[error("Server error from provider '{provider}' (status {status}): {message}")]
    ServerError {
        provider: String,
        status: u16,
        message: String,
    },

    /// Network error occurred during communication.
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    /// Stream was closed unexpectedly.
    #[error("Stream closed unexpectedly")]
    StreamClosed,

    /// Unknown or unexpected error.
    #[error("Unknown error from provider '{provider}': {message}")]
    Unknown {
        provider: String,
        message: String,
    },
}

impl PartialEq for ProviderError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::AuthenticationFailed { provider: p1, message: m1 }, Self::AuthenticationFailed { provider: p2, message: m2 }) => {
                p1 == p2 && m1 == m2
            }
            (Self::RateLimited { provider: p1, retry_after: r1 }, Self::RateLimited { provider: p2, retry_after: r2 }) => {
                p1 == p2 && r1 == r2
            }
            (Self::InvalidRequest { provider: p1, message: m1 }, Self::InvalidRequest { provider: p2, message: m2 }) => {
                p1 == p2 && m1 == m2
            }
            (Self::ModelNotFound { provider: p1, model: m1 }, Self::ModelNotFound { provider: p2, model: m2 }) => {
                p1 == p2 && m1 == m2
            }
            (Self::ContextLengthExceeded { provider: p1, max_tokens: m1 }, Self::ContextLengthExceeded { provider: p2, max_tokens: m2 }) => {
                p1 == p2 && m1 == m2
            }
            (Self::ServerError { provider: p1, status: s1, message: m1 }, Self::ServerError { provider: p2, status: s2, message: m2 }) => {
                p1 == p2 && s1 == s2 && m1 == m2
            }
            (Self::StreamClosed, Self::StreamClosed) => true,
            (Self::Unknown { provider: p1, message: m1 }, Self::Unknown { provider: p2, message: m2 }) => {
                p1 == p2 && m1 == m2
            }
            _ => false,
        }
    }
}

/// Maps HTTP status codes and response bodies to ProviderError variants.
///
/// This function handles common HTTP error patterns:
/// - 401 / 403 → AuthenticationFailed
/// - 429 → RateLimited
/// - 400 → InvalidRequest
/// - 404 → ModelNotFound
/// - 5xx → ServerError
/// - Other codes → Unknown
///
/// # Arguments
///
/// * `provider` - The provider name (e.g., "anthropic", "openai")
/// * `status` - The HTTP status code
/// * `body` - The response body for additional error context
///
/// # Returns
///
/// A ProviderError variant corresponding to the HTTP status code.
pub fn map_http_error(provider: &str, status: u16, body: &str) -> ProviderError {
    match status {
        401 | 403 => {
            let message = extract_error_message(body);
            ProviderError::AuthenticationFailed {
                provider: provider.to_string(),
                message,
            }
        }
        429 => {
            // Parse retry-after header if present in body (common format)
            let retry_after = parse_retry_after(body);
            ProviderError::RateLimited {
                provider: provider.to_string(),
                retry_after,
            }
        }
        400 => {
            let message = extract_error_message(body);
            ProviderError::InvalidRequest {
                provider: provider.to_string(),
                message,
            }
        }
        404 => {
            // Try to extract model name from body
            let model = extract_model_name(body);
            ProviderError::ModelNotFound {
                provider: provider.to_string(),
                model: model.unwrap_or_else(|| "unknown".to_string()),
            }
        }
        413 => {
            // Context length exceeded (413 Payload Too Large)
            let max_tokens = extract_max_tokens(body);
            ProviderError::ContextLengthExceeded {
                provider: provider.to_string(),
                max_tokens: max_tokens.unwrap_or(0),
            }
        }
        500..=599 => {
            let message = extract_error_message(body);
            ProviderError::ServerError {
                provider: provider.to_string(),
                status,
                message,
            }
        }
        _ => {
            ProviderError::Unknown {
                provider: provider.to_string(),
                message: body.to_string(),
            }
        }
    }
}

/// Enforces alternating turns between user and assistant messages.
///
/// Some providers (like Anthropic) require that messages alternate between
/// user and assistant roles. This function merges consecutive messages with
/// the same role into a single message.
///
/// # Arguments
///
/// * `messages` - Mutable reference to the message vector to normalize
///
/// # Example
///
/// ```
/// use aisopod_provider::{Message, MessageContent, Role};
/// use aisopod_provider::normalize::enforce_alternating_turns;
///
/// let mut messages = vec![
///     Message { role: Role::User, content: MessageContent::Text("First".to_string()), tool_calls: None, tool_call_id: None },
///     Message { role: Role::User, content: MessageContent::Text("Second".to_string()), tool_calls: None, tool_call_id: None },
///     Message { role: Role::Assistant, content: MessageContent::Text("Response".to_string()), tool_calls: None, tool_call_id: None },
///     Message { role: Role::Assistant, content: MessageContent::Text("More".to_string()), tool_calls: None, tool_call_id: None },
/// ];
///
/// enforce_alternating_turns(&mut messages);
/// // Now we have 2 messages: user merged content, assistant merged content
/// ```
pub fn enforce_alternating_turns(messages: &mut Vec<Message>) {
    if messages.len() <= 1 {
        return;
    }

    let mut i = 0;
    while i < messages.len().saturating_sub(1) {
        let current_role = messages[i].role.clone();
        let next_role = messages[i + 1].role.clone();

        if current_role == next_role {
            // Merge the next message into the current one
            let next_content = messages.remove(i + 1).content;
            
            // Merge content - if both are Text, concatenate; otherwise use Parts
            match (&mut messages[i].content, next_content) {
                (MessageContent::Text(current_text), MessageContent::Text(next_text)) => {
                    *current_text = format!("{} {}", current_text, next_text);
                }
                (MessageContent::Parts(parts), MessageContent::Text(next_text)) => {
                    parts.push(ContentPart::Text { text: next_text });
                }
                (MessageContent::Text(current_text), MessageContent::Parts(next_parts)) => {
                    // Extract the first text part if any, then add remaining parts
                    let mut next_iter = next_parts.into_iter();
                    if let Some(ContentPart::Text { text }) = next_iter.next() {
                        *current_text = format!("{} {}", current_text, text);
                        // Collect remaining parts into a Vec first to avoid borrow issues
                        let remaining_parts: Vec<ContentPart> = next_iter.collect();
                        if !remaining_parts.is_empty() {
                            // Replace content with a new Parts variant
                            let old_text = std::mem::take(current_text);
                            messages[i].content = MessageContent::Parts(vec![
                                ContentPart::Text { text: old_text },
                            ]);
                            // Now extend with remaining parts
                            if let MessageContent::Parts(ref mut parts) = messages[i].content {
                                parts.extend(remaining_parts);
                            }
                        }
                    }
                }
                (MessageContent::Parts(current_parts), MessageContent::Parts(next_parts)) => {
                    current_parts.extend(next_parts);
                }
            }
        } else {
            i += 1;
        }
    }
}

/// Extracts and removes the system prompt from messages.
///
/// Some providers handle system prompts separately from the message history.
/// This function finds and removes the first system message, returning it
/// if present.
///
/// # Arguments
///
/// * `messages` - Mutable reference to the message vector
///
/// # Returns
///
/// The extracted system prompt text, if present.
///
/// # Example
///
/// ```
/// use aisopod_provider::{Message, MessageContent, Role};
/// use aisopod_provider::normalize::extract_system_prompt;
///
/// let mut messages = vec![
///     Message { role: Role::System, content: MessageContent::Text("Be helpful".to_string()), tool_calls: None, tool_call_id: None },
///     Message { role: Role::User, content: MessageContent::Text("Hello".to_string()), tool_calls: None, tool_call_id: None },
/// ];
///
/// let system = extract_system_prompt(&mut messages);
/// assert_eq!(system, Some("Be helpful".to_string()));
/// assert_eq!(messages.len(), 1);
/// ```
pub fn extract_system_prompt(messages: &mut Vec<Message>) -> Option<String> {
    if messages.is_empty() {
        return None;
    }

    // Find the first system message
    for i in 0..messages.len() {
        if messages[i].role == Role::System {
            let message = messages.remove(i);
            match message.content {
                MessageContent::Text(text) => return Some(text),
                MessageContent::Parts(parts) => {
                    // Convert parts to text
                    let text = parts
                        .into_iter()
                        .filter_map(|p| match p {
                            ContentPart::Text { text } => Some(text),
                            ContentPart::Image { .. } => None,
                        })
                        .collect::<Vec<_>>()
                        .join(" ");
                    return Some(text);
                }
            }
        }
    }

    None
}

/// Aggregates token usage from a sequence of streaming chunks.
///
/// When streaming responses from providers, each chunk may contain partial
/// usage information. This function sums up all the usage statistics,
/// using the last chunk's usage if only one reports it (common pattern).
///
/// # Arguments
///
/// * `chunks` - Slice of chat completion chunks
///
/// # Returns
///
/// A TokenUsage struct with aggregated totals.
///
/// # Example
///
/// ```
/// use aisopod_provider::{ChatCompletionChunk, MessageDelta, Role, TokenUsage, FinishReason};
/// use aisopod_provider::normalize::aggregate_usage;
///
/// let chunks = vec![
///     ChatCompletionChunk {
///         id: "chunk1".to_string(),
///         delta: MessageDelta { role: Some(Role::Assistant), content: Some("Hello".to_string()), tool_calls: None },
///         finish_reason: None,
///         usage: None,
///     },
///     ChatCompletionChunk {
///         id: "chunk2".to_string(),
///         delta: MessageDelta { role: None, content: Some(" world".to_string()), tool_calls: None },
///         finish_reason: Some(FinishReason::Stop),
///         usage: Some(TokenUsage {
///             prompt_tokens: 5,
///             completion_tokens: 2,
///             total_tokens: 7,
///         }),
///     },
/// ];
///
/// let usage = aggregate_usage(&chunks);
/// assert_eq!(usage.prompt_tokens, 5);
/// assert_eq!(usage.completion_tokens, 2);
/// assert_eq!(usage.total_tokens, 7);
/// ```
pub fn aggregate_usage(chunks: &[ChatCompletionChunk]) -> TokenUsage {
    let mut total_prompt_tokens: u32 = 0;
    let mut total_completion_tokens: u32 = 0;
    let mut total_total_tokens: u32 = 0;

    for chunk in chunks {
        if let Some(usage) = &chunk.usage {
            total_prompt_tokens += usage.prompt_tokens;
            total_completion_tokens += usage.completion_tokens;
            total_total_tokens += usage.total_tokens;
        }
    }

    TokenUsage {
        prompt_tokens: total_prompt_tokens,
        completion_tokens: total_completion_tokens,
        total_tokens: total_total_tokens,
    }
}

// Helper functions for parsing error responses

fn extract_error_message(body: &str) -> String {
    // Try to parse as JSON first
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(body) {
        // Try common error message fields
        if let Some(message) = json.get("error").and_then(|e| e.get("message")) {
            return message.as_str().unwrap_or(body).to_string();
        }
        if let Some(message) = json.get("message") {
            return message.as_str().unwrap_or(body).to_string();
        }
        if let Some(error) = json.get("error") {
            if let Some(msg) = error.get("message") {
                return msg.as_str().unwrap_or(body).to_string();
            }
        }
    }

    // Fallback: use the body directly
    body.trim().to_string()
}

fn extract_model_name(body: &str) -> Option<String> {
    // Try to extract model name from common error formats
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(body) {
        // Try common error fields for model info
        if let Some(error) = json.get("error") {
            if let Some(model) = error.get("model") {
                return model.as_str().map(|s| s.to_string());
            }
        }
    }

    // Try to extract from body text
    let body_lower = body.to_lowercase();
    if let Some(pos) = body_lower.find("model") {
        let rest = &body[pos..];
        if let Some(end) = rest.find(|c: char| c == '"' || c == '\'' || c == ' ' || c == '\n') {
            let model = rest[..end].trim_matches('"').trim_matches('\'').to_string();
            if !model.is_empty() {
                return Some(model);
            }
        }
    }

    None
}

fn extract_max_tokens(body: &str) -> Option<u32> {
    // Try to parse max_tokens from JSON
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(body) {
        // Try common fields for max tokens
        if let Some(max) = json.get("error").and_then(|e| e.get("max_tokens")) {
            return max.as_u64().map(|v| v as u32);
        }
        if let Some(max) = json.get("max_tokens") {
            return max.as_u64().map(|v| v as u32);
        }
    }

    None
}

fn parse_retry_after(body: &str) -> Option<Duration> {
    // Try to parse retry-after as JSON field
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(body) {
        if let Some(retry) = json.get("retry_after") {
            if let Some(seconds) = retry.as_f64() {
                return Some(Duration::from_secs_f64(seconds));
            }
        }
        if let Some(retry) = json.get("error") {
            if let Some(msg) = retry.get("message") {
                let msg_str = msg.as_str().unwrap_or("");
                // Try to extract seconds from message like "retry after 60 seconds"
                if let Some(pos) = msg_str.find("retry") {
                    let after = &msg_str[pos..];
                    if let Some(num_start) = after.find(|c: char| c.is_ascii_digit()) {
                        let rest = &after[num_start..];
                        if let Some(num_end) = rest.find(|c: char| !c.is_ascii_digit()) {
                            if let Ok(seconds) = rest[..num_end].parse::<u64>() {
                                return Some(Duration::from_secs(seconds));
                            }
                        }
                    }
                }
            }
        }
    }

    // Try to parse as raw seconds in body
    let body = body.trim();
    if let Ok(seconds) = body.parse::<u64>() {
        return Some(Duration::from_secs(seconds));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create a message
    fn msg(role: Role, content: &str) -> Message {
        Message {
            role,
            content: MessageContent::Text(content.to_string()),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    // Helper to create a chunk
    fn chunk(usage: Option<TokenUsage>) -> ChatCompletionChunk {
        ChatCompletionChunk {
            id: "chunk_123".to_string(),
            delta: MessageDelta {
                role: Some(Role::Assistant),
                content: Some("test".to_string()),
                tool_calls: None,
            },
            finish_reason: None,
            usage,
        }
    }

    #[test]
    fn test_map_http_error_authentication_401() {
        let error = map_http_error("openai", 401, "{\"error\": {\"message\": \"Invalid API key\"}}");
        match error {
            ProviderError::AuthenticationFailed { provider, message } => {
                assert_eq!(provider, "openai");
                assert_eq!(message, "Invalid API key");
            }
            _ => panic!("Expected AuthenticationFailed"),
        }
    }

    #[test]
    fn test_map_http_error_authentication_403() {
        let error = map_http_error("anthropic", 403, "Forbidden");
        match error {
            ProviderError::AuthenticationFailed { provider, message } => {
                assert_eq!(provider, "anthropic");
                assert!(message.contains("Forbidden"));
            }
            _ => panic!("Expected AuthenticationFailed"),
        }
    }

    #[test]
    fn test_map_http_error_rate_limited_429() {
        let error = map_http_error("openai", 429, "{\"error\": {\"type\": \"rate_limit_error\"}}");
        match error {
            ProviderError::RateLimited { provider, retry_after } => {
                assert_eq!(provider, "openai");
                assert!(retry_after.is_none());
            }
            _ => panic!("Expected RateLimited"),
        }
    }

    #[test]
    fn test_map_http_error_invalid_request_400() {
        let error = map_http_error("openai", 400, "{\"error\": {\"message\": \"Invalid model\"}}");
        match error {
            ProviderError::InvalidRequest { provider, message } => {
                assert_eq!(provider, "openai");
                assert_eq!(message, "Invalid model");
            }
            _ => panic!("Expected InvalidRequest"),
        }
    }

    #[test]
    fn test_map_http_error_model_not_found_404() {
        let error = map_http_error("openai", 404, "{\"error\": {\"type\": \"not_found_error\", \"model\": \"gpt-5\"}}");
        match error {
            ProviderError::ModelNotFound { provider, model } => {
                assert_eq!(provider, "openai");
                assert_eq!(model, "gpt-5");
            }
            _ => panic!("Expected ModelNotFound"),
        }
    }

    #[test]
    fn test_map_http_error_context_length_exceeded_413() {
        let error = map_http_error("openai", 413, "{\"error\": {\"max_tokens\": 8192}}");
        match error {
            ProviderError::ContextLengthExceeded { provider, max_tokens } => {
                assert_eq!(provider, "openai");
                assert_eq!(max_tokens, 8192);
            }
            _ => panic!("Expected ContextLengthExceeded"),
        }
    }

    #[test]
    fn test_map_http_error_server_error_500() {
        let error = map_http_error("openai", 500, "Internal Server Error");
        match error {
            ProviderError::ServerError { provider, status, message } => {
                assert_eq!(provider, "openai");
                assert_eq!(status, 500);
                assert_eq!(message, "Internal Server Error");
            }
            _ => panic!("Expected ServerError"),
        }
    }

    #[test]
    fn test_map_http_error_unknown() {
        let error = map_http_error("openai", 418, "I'm a teapot");
        match error {
            ProviderError::Unknown { provider, message } => {
                assert_eq!(provider, "openai");
                assert_eq!(message, "I'm a teapot");
            }
            _ => panic!("Expected Unknown"),
        }
    }

    #[test]
    fn test_enforce_alternating_turns_consecutive_same_role() {
        let mut messages = vec![
            msg(Role::User, "First"),
            msg(Role::User, "Second"),
            msg(Role::Assistant, "Response"),
            msg(Role::Assistant, "More"),
        ];

        enforce_alternating_turns(&mut messages);

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, Role::User);
        assert_eq!(messages[0].content, MessageContent::Text("First Second".to_string()));
        assert_eq!(messages[1].role, Role::Assistant);
        assert_eq!(messages[1].content, MessageContent::Text("Response More".to_string()));
    }

    #[test]
    fn test_enforce_alternating_turns_already_alternating() {
        let mut messages = vec![
            msg(Role::User, "First"),
            msg(Role::Assistant, "Response"),
            msg(Role::User, "Second"),
        ];

        enforce_alternating_turns(&mut messages);

        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0].content, MessageContent::Text("First".to_string()));
        assert_eq!(messages[1].content, MessageContent::Text("Response".to_string()));
        assert_eq!(messages[2].content, MessageContent::Text("Second".to_string()));
    }

    #[test]
    fn test_enforce_alternating_turns_single_message() {
        let mut messages = vec![msg(Role::User, "Only one")];

        enforce_alternating_turns(&mut messages);

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, MessageContent::Text("Only one".to_string()));
    }

    #[test]
    fn test_enforce_alternating_turns_empty() {
        let mut messages: Vec<Message> = vec![];

        enforce_alternating_turns(&mut messages);

        assert!(messages.is_empty());
    }

    #[test]
    fn test_enforce_alternating_turns_three_consecutive() {
        let mut messages = vec![
            msg(Role::User, "First"),
            msg(Role::User, "Second"),
            msg(Role::User, "Third"),
        ];

        enforce_alternating_turns(&mut messages);

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, MessageContent::Text("First Second Third".to_string()));
    }

    #[test]
    fn test_extract_system_prompt_basic() {
        let mut messages = vec![
            msg(Role::System, "You are a helpful assistant"),
            msg(Role::User, "Hello"),
        ];

        let system = extract_system_prompt(&mut messages);

        assert_eq!(system, Some("You are a helpful assistant".to_string()));
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].role, Role::User);
    }

    #[test]
    fn test_extract_system_prompt_no_system() {
        let mut messages = vec![
            msg(Role::User, "Hello"),
            msg(Role::Assistant, "Hi there"),
        ];

        let system = extract_system_prompt(&mut messages);

        assert!(system.is_none());
        assert_eq!(messages.len(), 2);
    }

    #[test]
    fn test_extract_system_prompt_empty() {
        let mut messages: Vec<Message> = vec![];

        let system = extract_system_prompt(&mut messages);

        assert!(system.is_none());
    }

    #[test]
    fn test_extract_system_prompt_system_not_first() {
        let mut messages = vec![
            msg(Role::User, "Hello"),
            msg(Role::System, "Ignore me"),
            msg(Role::Assistant, "Hi there"),
        ];

        let system = extract_system_prompt(&mut messages);

        // Should extract the system message even if not first
        assert_eq!(system, Some("Ignore me".to_string()));
        assert_eq!(messages.len(), 2);
    }

    #[test]
    fn test_extract_system_prompt_with_parts() {
        let mut messages = vec![
            Message {
                role: Role::System,
                content: MessageContent::Parts(vec![
                    ContentPart::Text { text: "System".to_string() },
                    ContentPart::Text { text: "prompt".to_string() },
                ]),
                tool_calls: None,
                tool_call_id: None,
            },
            msg(Role::User, "Hello"),
        ];

        let system = extract_system_prompt(&mut messages);

        // Parts are joined with a space
        assert_eq!(system, Some("System prompt".to_string()));
        assert_eq!(messages.len(), 1);
    }

    #[test]
    fn test_aggregate_usage_last_chunk_has_usage() {
        let chunks = vec![
            chunk(None),
            chunk(None),
            chunk(Some(TokenUsage {
                prompt_tokens: 5,
                completion_tokens: 3,
                total_tokens: 8,
            })),
        ];

        let usage = aggregate_usage(&chunks);

        assert_eq!(usage.prompt_tokens, 5);
        assert_eq!(usage.completion_tokens, 3);
        assert_eq!(usage.total_tokens, 8);
    }

    #[test]
    fn test_aggregate_usage_first_chunk_has_usage() {
        let chunks = vec![
            chunk(Some(TokenUsage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            })),
            chunk(None),
            chunk(None),
        ];

        let usage = aggregate_usage(&chunks);

        assert_eq!(usage.prompt_tokens, 10);
        assert_eq!(usage.completion_tokens, 5);
        assert_eq!(usage.total_tokens, 15);
    }

    #[test]
    fn test_aggregate_usage_empty() {
        let chunks: Vec<ChatCompletionChunk> = vec![];

        let usage = aggregate_usage(&chunks);

        assert_eq!(usage.prompt_tokens, 0);
        assert_eq!(usage.completion_tokens, 0);
        assert_eq!(usage.total_tokens, 0);
    }

    #[test]
    fn test_aggregate_usage_multiple_chunks_with_usage() {
        let chunks = vec![
            chunk(Some(TokenUsage {
                prompt_tokens: 5,
                completion_tokens: 2,
                total_tokens: 7,
            })),
            chunk(Some(TokenUsage {
                prompt_tokens: 10,
                completion_tokens: 3,
                total_tokens: 13,
            })),
            chunk(Some(TokenUsage {
                prompt_tokens: 15,
                completion_tokens: 5,
                total_tokens: 20,
            })),
        ];

        let usage = aggregate_usage(&chunks);

        // Should sum all chunk usages
        assert_eq!(usage.prompt_tokens, 30);  // 5 + 10 + 15
        assert_eq!(usage.completion_tokens, 10);  // 2 + 3 + 5
        assert_eq!(usage.total_tokens, 40);  // 7 + 13 + 20
    }

    #[test]
    fn test_provider_error_display() {
        let error = ProviderError::AuthenticationFailed {
            provider: "openai".to_string(),
            message: "Invalid API key".to_string(),
        };

        let display = format!("{}", error);
        assert!(display.contains("Authentication failed"));
        assert!(display.contains("openai"));
        assert!(display.contains("Invalid API key"));
    }

    #[test]
    fn test_provider_error_is_error() {
        let error: ProviderError = map_http_error("openai", 401, "Unauthorized");
        
        // Should implement std::error::Error
        let _: &dyn std::error::Error = &error;
    }
}
