//! Transcript-related tests for agent engine.
//!
//! This module tests message transcript repair for different provider requirements.

use aisopod_agent::transcript::{repair_transcript, ProviderKind};
use aisopod_provider::{Message, MessageContent, Role};

fn text_message(role: Role, content: &str) -> Message {
    Message {
        role,
        content: MessageContent::Text(content.to_string()),
        tool_calls: None,
        tool_call_id: None,
    }
}

fn is_continued_message(msg: &Message) -> bool {
    match &msg.content {
        MessageContent::Text(text) => text.contains("[continued]"),
        _ => false,
    }
}

#[test]
fn test_provider_kind_enum() {
    assert_eq!(ProviderKind::Anthropic, ProviderKind::Anthropic);
    assert_eq!(ProviderKind::OpenAI, ProviderKind::OpenAI);
    assert_eq!(ProviderKind::Google, ProviderKind::Google);
    assert_eq!(ProviderKind::Other, ProviderKind::Other);
    assert_ne!(ProviderKind::Anthropic, ProviderKind::OpenAI);
}

#[test]
fn test_anthropic_consecutive_user_messages() {
    let messages = vec![
        text_message(Role::User, "First"),
        text_message(Role::User, "Second"),
        text_message(Role::Assistant, "Response"),
    ];

    let repaired = repair_transcript(&messages, ProviderKind::Anthropic);

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
        text_message(Role::User, "User"),
        text_message(Role::Assistant, "First"),
        text_message(Role::Assistant, "Second"),
    ];

    let repaired = repair_transcript(&messages, ProviderKind::Anthropic);

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
        text_message(Role::Assistant, "First"),
        text_message(Role::User, "Second"),
    ];

    let repaired = repair_transcript(&messages, ProviderKind::Anthropic);

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
        text_message(Role::Assistant, "Final"),
    ];

    let repaired = repair_transcript(&messages, ProviderKind::Anthropic);

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
    let messages = vec![text_message(Role::User, "Single")];

    let repaired = repair_transcript(&messages, ProviderKind::Anthropic);

    assert_eq!(repaired.len(), 1);
    assert_eq!(repaired[0].role, Role::User);
}

#[test]
fn test_anthropic_single_message_assistant() {
    let messages = vec![text_message(Role::Assistant, "Single")];

    let repaired = repair_transcript(&messages, ProviderKind::Anthropic);

    assert_eq!(repaired.len(), 2);
    assert_eq!(repaired[0].role, Role::User); // synthetic
    assert_eq!(repaired[1].role, Role::Assistant);
}

#[test]
fn test_openai_system_message_at_start() {
    let messages = vec![
        text_message(Role::User, "User"),
        text_message(Role::System, "System"),
        text_message(Role::Assistant, "Assistant"),
    ];

    let repaired = repair_transcript(&messages, ProviderKind::OpenAI);

    assert_eq!(repaired.len(), 3);
    assert_eq!(repaired[0].role, Role::System);
    assert_eq!(repaired[1].role, Role::User);
    assert_eq!(repaired[2].role, Role::Assistant);
}

#[test]
fn test_openai_multiple_system_messages_deduplicated() {
    let messages = vec![
        text_message(Role::System, "First"),
        text_message(Role::User, "User"),
        text_message(Role::System, "Second"),
    ];

    let repaired = repair_transcript(&messages, ProviderKind::OpenAI);

    assert_eq!(repaired.len(), 2);
    assert_eq!(repaired[0].role, Role::System);
    
    if let MessageContent::Text(ref content) = repaired[0].content {
        assert!(content.contains("First"));
        assert!(content.contains("Second"));
    } else {
        panic!("Expected text content");
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

    assert_eq!(repaired.len(), 3);
    assert_eq!(repaired[0].role, Role::System);
    assert_eq!(repaired[1].role, Role::User);
    assert_eq!(repaired[2].role, Role::Assistant);
}

#[test]
fn test_gemini_consecutive_user_messages() {
    let messages = vec![
        text_message(Role::User, "First"),
        text_message(Role::User, "Second"),
        text_message(Role::Assistant, "Response"),
    ];

    let repaired = repair_transcript(&messages, ProviderKind::Google);

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
        text_message(Role::Assistant, "First"),
        text_message(Role::User, "Second"),
    ];

    let repaired = repair_transcript(&messages, ProviderKind::Google);

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

    assert_eq!(repaired.len(), 2);
    assert_eq!(repaired[0].role, Role::User);
    assert_eq!(repaired[1].role, Role::Assistant);
}

#[test]
fn test_other_provider_pass_through() {
    let messages = vec![
        text_message(Role::Assistant, "First"),
        text_message(Role::Assistant, "Second"),
        text_message(Role::System, "System"),
    ];

    let repaired = repair_transcript(&messages, ProviderKind::Other);

    assert_eq!(repaired.len(), 3);
    assert_eq!(repaired[0].role, Role::Assistant);
    assert_eq!(repaired[1].role, Role::Assistant);
    assert_eq!(repaired[2].role, Role::System);
}

#[test]
fn test_synthetic_messages_have_markers() {
    let user_synthetic = Message {
        role: Role::User,
        content: MessageContent::Text("[continued]".to_string()),
        tool_calls: None,
        tool_call_id: None,
    };
    let assistant_synthetic = Message {
        role: Role::Assistant,
        content: MessageContent::Text("[continued]".to_string()),
        tool_calls: None,
        tool_call_id: None,
    };

    assert!(is_continued_message(&user_synthetic));
    assert!(is_continued_message(&assistant_synthetic));
}
