//! Message transcript repair for provider-specific ordering requirements.
//!
//! This module provides functions to repair message transcripts to satisfy
//! the turn-ordering requirements of different AI model providers.
//!
//! ## Overview
//!
//! Different providers have different rules about message ordering:
//! - **Anthropic**: Requires strictly alternating user/assistant turns
//! - **OpenAI**: More flexible, but system messages should be at the start
//! - **Gemini**: Requires alternating user/model turns
//! - **Other**: No specific requirements (pass-through)
//!
//! This module provides a `repair_transcript()` function that takes a message
//! list and a target provider, returning a corrected message list.

use aisopod_provider::{Message, MessageContent, Role};

/// Specifies which AI provider we're repairing for.
///
/// Different providers have different requirements for message ordering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderKind {
    /// Anthropic's Claude models
    Anthropic,
    /// OpenAI's GPT models
    OpenAI,
    /// Google's Gemini models
    Google,
    /// Any other provider (pass-through)
    Other,
}

/// Repair a message transcript for the specified provider.
///
/// This function analyzes the message sequence and inserts synthetic messages
/// where needed to satisfy the provider's turn-ordering requirements.
///
/// # Arguments
///
/// * `messages` - The message transcript to repair
/// * `provider` - The target provider kind
///
/// # Returns
///
/// A repaired message list that satisfies the provider's requirements.
///
/// # Examples
///
/// ```
/// use aisopod_provider::{Message, MessageContent, Role};
/// use aisopod_agent::transcript::{repair_transcript, ProviderKind};
///
/// let messages = vec![
///     Message { role: Role::User, content: MessageContent::Text("First".to_string()), tool_calls: None, tool_call_id: None },
///     Message { role: Role::User, content: MessageContent::Text("Second".to_string()), tool_calls: None, tool_call_id: None },
///     Message { role: Role::Assistant, content: MessageContent::Text("Response".to_string()), tool_calls: None, tool_call_id: None },
/// ];
///
/// let repaired = repair_transcript(&messages, ProviderKind::Anthropic);
/// // Should have inserted a synthetic assistant message between the two user messages
/// assert_eq!(repaired.len(), 4); // 2 users + synthetic assistant + 1 assistant
/// ```
pub fn repair_transcript(messages: &[Message], provider: ProviderKind) -> Vec<Message> {
    let mut result = messages.to_vec();
    
    match provider {
        ProviderKind::Anthropic => repair_for_anthropic(&mut result),
        ProviderKind::OpenAI => repair_for_openai(&mut result),
        ProviderKind::Google => repair_for_google(&mut result),
        ProviderKind::Other => {
            // No-op pass-through
        }
    }
    
    result
}

/// Repair a transcript for Anthropic's strict alternating turn requirements.
///
/// Anthropic requires that messages strictly alternate between user and assistant.
/// This function:
/// 1. Ensures the sequence starts with a user message
/// 2. Inserts synthetic messages between consecutive same-role messages
fn repair_for_anthropic(messages: &mut Vec<Message>) {
    if messages.is_empty() {
        return;
    }
    
    // First, ensure the sequence starts with a user message
    if messages[0].role != Role::User {
        messages.insert(0, synthetic_user_message());
    }
    
    // Now process the sequence to handle consecutive same-role messages
    let mut i = 0;
    while i < messages.len().saturating_sub(1) {
        let current_role = messages[i].role.clone();
        let next_role = messages[i + 1].role.clone();
        
        if current_role == next_role {
            // Insert a synthetic message of the opposite role between them
            let synthetic = if current_role == Role::User {
                synthetic_assistant_message()
            } else {
                synthetic_user_message()
            };
            messages.insert(i + 1, synthetic);
            i += 1; // Skip the inserted message
        } else {
            i += 1;
        }
    }
}

/// Repair a transcript for OpenAI's more flexible requirements.
///
/// OpenAI is more permissive with message ordering, but we still:
/// 1. Ensure system message is at the start
/// 2. Merge consecutive system messages (deduplicate)
fn repair_for_openai(messages: &mut Vec<Message>) {
    if messages.is_empty() {
        return;
    }
    
    // Count system messages
    let system_count = messages.iter().filter(|m| m.role == Role::System).count();
    
    if system_count > 0 {
        // Move all system messages to the beginning
        let mut system_messages: Vec<Message> = messages
            .iter()
            .filter(|m| m.role == Role::System)
            .cloned()
            .collect();
        
        // Keep only the first system message and merge any additional ones
        // into it for better context
        if system_messages.len() > 1 {
            let first_system = system_messages.remove(0);
            let merged_content = merge_system_messages(&system_messages);
            let mut result = vec![first_system];
            
            // Merge the content of additional system messages into the first one
            if let MessageContent::Text(ref text) = result[0].content {
                let new_content = format!("{} {}", text, merged_content);
                result[0].content = MessageContent::Text(new_content);
            }
            
            system_messages = result;
        }
        
        // Collect non-system messages
        let non_system: Vec<Message> = messages
            .iter()
            .filter(|m| m.role != Role::System)
            .cloned()
            .collect();
        
        // Reassemble: system messages first, then non-system
        messages.clear();
        messages.extend(system_messages);
        messages.extend(non_system);
    }
}

/// Merge multiple system messages into a single text content.
fn merge_system_messages(messages: &[Message]) -> String {
    messages
        .iter()
        .filter_map(|m| {
            if let MessageContent::Text(ref text) = m.content {
                Some(text.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Repair a transcript for Google's Gemini requirements.
///
/// Gemini requires alternating user/model turns similar to Anthropic.
/// This function is similar to the Anthropic repair but adapted for Gemini.
fn repair_for_google(messages: &mut Vec<Message>) {
    if messages.is_empty() {
        return;
    }
    
    // Gemini typically expects user to start
    if messages[0].role != Role::User {
        messages.insert(0, synthetic_user_message());
    }
    
    // Handle consecutive same-role messages
    let mut i = 0;
    while i < messages.len().saturating_sub(1) {
        let current_role = messages[i].role.clone();
        let next_role = messages[i + 1].role.clone();
        
        if current_role == next_role {
            let synthetic = if current_role == Role::User {
                synthetic_assistant_message()
            } else {
                synthetic_user_message()
            };
            messages.insert(i + 1, synthetic);
            i += 1;
        } else {
            i += 1;
        }
    }
}

/// Create a synthetic user message with a recognizable marker.
fn synthetic_user_message() -> Message {
    Message {
        role: Role::User,
        content: MessageContent::Text("[continued]".to_string()),
        tool_calls: None,
        tool_call_id: None,
    }
}

/// Create a synthetic assistant message with a recognizable marker.
fn synthetic_assistant_message() -> Message {
    Message {
        role: Role::Assistant,
        content: MessageContent::Text("[continued]".to_string()),
        tool_calls: None,
        tool_call_id: None,
    }
}

#[cfg(test)]
mod tests {
    use aisopod_provider::{ContentPart, Message, MessageContent, Role};
    
    use super::{
        repair_transcript, ProviderKind, synthetic_assistant_message, synthetic_user_message,
    };

    /// Helper to create a simple text message
    fn text_message(role: Role, content: &str) -> Message {
        Message {
            role,
            content: MessageContent::Text(content.to_string()),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    /// Helper to check if a message has the [continued] marker
    fn is_continued_message(msg: &Message) -> bool {
        use aisopod_provider::ContentPart;
        match &msg.content {
            MessageContent::Text(text) => text.contains("[continued]"),
            MessageContent::Parts(parts) => parts.iter().any(|p| {
                if let ContentPart::Text { text } = p {
                    text.contains("[continued]")
                } else {
                    false
                }
            }),
            _ => false,
        }
    }

    // Anthropic repair tests

    #[test]
    fn test_anthropic_consecutive_user_messages() {
        let messages = vec![
            text_message(Role::User, "First user message"),
            text_message(Role::User, "Second user message"),
            text_message(Role::Assistant, "Assistant response"),
        ];

        let repaired = repair_transcript(&messages, ProviderKind::Anthropic);

        // Should have inserted a synthetic assistant between the two user messages
        assert_eq!(repaired.len(), 4);
        assert_eq!(repaired[0].role, Role::User);
        assert_eq!(repaired[1].role, Role::Assistant); // synthetic
        assert!(is_continued_message(&repaired[1]));
        assert_eq!(repaired[2].role, Role::User);
        assert_eq!(repaired[3].role, Role::Assistant);
    }

    #[test]
    fn test_anthropic_consecutive_assistant_messages() {
        let messages = vec![
            text_message(Role::User, "User message"),
            text_message(Role::Assistant, "First assistant message"),
            text_message(Role::Assistant, "Second assistant message"),
        ];

        let repaired = repair_transcript(&messages, ProviderKind::Anthropic);

        // Should have inserted a synthetic user between the two assistant messages
        assert_eq!(repaired.len(), 4);
        assert_eq!(repaired[0].role, Role::User);
        assert_eq!(repaired[1].role, Role::Assistant);
        assert_eq!(repaired[2].role, Role::User); // synthetic
        assert!(is_continued_message(&repaired[2]));
        assert_eq!(repaired[3].role, Role::Assistant);
    }

    #[test]
    fn test_anthropic_starts_with_assistant() {
        let messages = vec![
            text_message(Role::Assistant, "Assistant first"),
            text_message(Role::User, "User response"),
        ];

        let repaired = repair_transcript(&messages, ProviderKind::Anthropic);

        // Should have inserted a synthetic user at the beginning
        assert_eq!(repaired.len(), 3);
        assert_eq!(repaired[0].role, Role::User); // synthetic
        assert!(is_continued_message(&repaired[0]));
        assert_eq!(repaired[1].role, Role::Assistant);
        assert_eq!(repaired[2].role, Role::User);
    }

    #[test]
    fn test_anthropic_valid_alternating_sequence() {
        let messages = vec![
            text_message(Role::User, "First"),
            text_message(Role::Assistant, "Response"),
            text_message(Role::User, "Follow-up"),
            text_message(Role::Assistant, "Final response"),
        ];

        let repaired = repair_transcript(&messages, ProviderKind::Anthropic);

        // Valid sequence should pass through unchanged
        assert_eq!(repaired.len(), 4);
        assert_eq!(repaired[0].role, Role::User);
        assert_eq!(repaired[1].role, Role::Assistant);
        assert_eq!(repaired[2].role, Role::User);
        assert_eq!(repaired[3].role, Role::Assistant);
    }

    #[test]
    fn test_anthropic_empty_sequence() {
        let messages: Vec<Message> = vec![];

        let repaired = repair_transcript(&messages, ProviderKind::Anthropic);

        assert!(repaired.is_empty());
    }

    #[test]
    fn test_anthropic_single_message_user() {
        let messages = vec![text_message(Role::User, "Single user message")];

        let repaired = repair_transcript(&messages, ProviderKind::Anthropic);

        // Single user message should pass through unchanged
        assert_eq!(repaired.len(), 1);
        assert_eq!(repaired[0].role, Role::User);
    }

    #[test]
    fn test_anthropic_single_message_assistant() {
        let messages = vec![text_message(Role::Assistant, "Single assistant message")];

        let repaired = repair_transcript(&messages, ProviderKind::Anthropic);

        // Single assistant message should have synthetic user prepended
        assert_eq!(repaired.len(), 2);
        assert_eq!(repaired[0].role, Role::User); // synthetic
        assert_eq!(repaired[1].role, Role::Assistant);
    }

    // OpenAI repair tests

    #[test]
    fn test_openai_system_message_at_start() {
        let messages = vec![
            text_message(Role::User, "User message"),
            text_message(Role::System, "System instruction"),
            text_message(Role::Assistant, "Assistant response"),
        ];

        let repaired = repair_transcript(&messages, ProviderKind::OpenAI);

        // System message should be moved to the start
        assert_eq!(repaired.len(), 3);
        assert_eq!(repaired[0].role, Role::System);
        assert_eq!(repaired[1].role, Role::User);
        assert_eq!(repaired[2].role, Role::Assistant);
    }

    #[test]
    fn test_openai_multiple_system_messages_deduplicated() {
        let messages = vec![
            text_message(Role::System, "First system"),
            text_message(Role::User, "User"),
            text_message(Role::System, "Second system"),
        ];

        let repaired = repair_transcript(&messages, ProviderKind::OpenAI);

        // Should merge system messages
        assert_eq!(repaired.len(), 2);
        assert_eq!(repaired[0].role, Role::System);
        // The content should be merged
        if let MessageContent::Text(ref content) = repaired[0].content {
            assert!(content.contains("First system"));
            assert!(content.contains("Second system"));
        } else {
            panic!("Expected text content for merged system message");
        }
        assert_eq!(repaired[1].role, Role::User);
    }

    #[test]
    fn test_openai_valid_sequence_unchanged() {
        let messages = vec![
            text_message(Role::System, "System"),
            text_message(Role::User, "User"),
            text_message(Role::Assistant, "Assistant"),
        ];

        let repaired = repair_transcript(&messages, ProviderKind::OpenAI);

        // Valid sequence should pass through (with system at start)
        assert_eq!(repaired.len(), 3);
        assert_eq!(repaired[0].role, Role::System);
        assert_eq!(repaired[1].role, Role::User);
        assert_eq!(repaired[2].role, Role::Assistant);
    }

    // Gemini repair tests

    #[test]
    fn test_gemini_consecutive_user_messages() {
        let messages = vec![
            text_message(Role::User, "First"),
            text_message(Role::User, "Second"),
            text_message(Role::Assistant, "Response"),
        ];

        let repaired = repair_transcript(&messages, ProviderKind::Google);

        // Should insert synthetic assistant between consecutive user messages
        assert_eq!(repaired.len(), 4);
        assert_eq!(repaired[0].role, Role::User);
        assert_eq!(repaired[1].role, Role::Assistant); // synthetic
        assert!(is_continued_message(&repaired[1]));
        assert_eq!(repaired[2].role, Role::User);
        assert_eq!(repaired[3].role, Role::Assistant);
    }

    #[test]
    fn test_gemini_starts_with_assistant() {
        let messages = vec![
            text_message(Role::Assistant, "Assistant first"),
            text_message(Role::User, "User"),
        ];

        let repaired = repair_transcript(&messages, ProviderKind::Google);

        // Should insert synthetic user at beginning
        assert_eq!(repaired.len(), 3);
        assert_eq!(repaired[0].role, Role::User); // synthetic
        assert!(is_continued_message(&repaired[0]));
        assert_eq!(repaired[1].role, Role::Assistant);
        assert_eq!(repaired[2].role, Role::User);
    }

    #[test]
    fn test_gemini_valid_sequence_unchanged() {
        let messages = vec![
            text_message(Role::User, "User"),
            text_message(Role::Assistant, "Assistant"),
        ];

        let repaired = repair_transcript(&messages, ProviderKind::Google);

        // Valid alternating sequence should pass through
        assert_eq!(repaired.len(), 2);
        assert_eq!(repaired[0].role, Role::User);
        assert_eq!(repaired[1].role, Role::Assistant);
    }

    // Other provider tests

    #[test]
    fn test_other_provider_pass_through() {
        let messages = vec![
            text_message(Role::Assistant, "Assistant first"),
            text_message(Role::Assistant, "Another assistant"),
            text_message(Role::System, "System anywhere"),
        ];

        let repaired = repair_transcript(&messages, ProviderKind::Other);

        // Other provider should pass through unchanged
        assert_eq!(repaired.len(), 3);
        assert_eq!(repaired[0].role, Role::Assistant);
        assert_eq!(repaired[1].role, Role::Assistant);
        assert_eq!(repaired[2].role, Role::System);
    }

    // ProviderKind enum tests

    #[test]
    fn test_provider_kind_enum() {
        assert_eq!(ProviderKind::Anthropic, ProviderKind::Anthropic);
        assert_eq!(ProviderKind::OpenAI, ProviderKind::OpenAI);
        assert_eq!(ProviderKind::Google, ProviderKind::Google);
        assert_eq!(ProviderKind::Other, ProviderKind::Other);
        assert_ne!(ProviderKind::Anthropic, ProviderKind::OpenAI);
    }

    #[test]
    fn test_synthetic_messages_have_markers() {
        let user_synthetic = synthetic_user_message();
        let assistant_synthetic = synthetic_assistant_message();

        assert!(is_continued_message(&user_synthetic));
        assert!(is_continued_message(&assistant_synthetic));
    }
}
