//! Compaction-related tests for agent engine.
//!
//! This module tests message compaction strategies for managing context window size.

use aisopod_agent::compaction::{compact_messages, estimate_token_count, select_strategy, CompactionSeverity, CompactionStrategy};
use aisopod_provider::{Message, MessageContent, Role};
use aisopod_agent::context_guard::ContextWindowGuard;

fn text_message(role: Role, content: &str) -> Message {
    Message {
        role,
        content: MessageContent::Text(content.to_string()),
        tool_calls: None,
        tool_call_id: None,
    }
}

fn message_to_text(msg: &Message) -> String {
    match &msg.content {
        MessageContent::Text(text) => text.clone(),
        MessageContent::Parts(parts) => parts
            .iter()
            .filter_map(|p| {
                if let aisopod_provider::ContentPart::Text { text } = p {
                    Some(text.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join(" "),
        _ => String::new(),
    }
}

#[test]
fn test_compaction_strategy_default() {
    let strategy = CompactionStrategy::default();
    assert!(matches!(strategy, CompactionStrategy::Summary { .. }));
}

#[test]
fn test_compaction_strategy_enum() {
    assert_eq!(CompactionStrategy::AdaptiveChunking, CompactionStrategy::AdaptiveChunking);
    assert_eq!(CompactionStrategy::Summary { keep_recent: 10 }, CompactionStrategy::Summary { keep_recent: 10 });
    assert_eq!(CompactionStrategy::HardClear { keep_recent: 5 }, CompactionStrategy::HardClear { keep_recent: 5 });
    assert_eq!(CompactionStrategy::ToolResultTruncation { max_chars: 1000 }, CompactionStrategy::ToolResultTruncation { max_chars: 1000 });
}

#[test]
fn test_hard_clear_truncates() {
    let messages = (0..20)
        .map(|i| text_message(Role::User, &format!("Message {}", i)))
        .collect::<Vec<_>>();

    let result = compact_messages(&messages, CompactionStrategy::HardClear { keep_recent: 5 });

    assert_eq!(result.len(), 5);
    assert!(result.iter().any(|m| message_to_text(m).contains("Message 19")));
    assert!(result.iter().any(|m| message_to_text(m).contains("Message 15")));
}

#[test]
fn test_hard_clear_no_op_when_under_limit() {
    let messages = (0..5)
        .map(|i| text_message(Role::User, &format!("Message {}", i)))
        .collect::<Vec<_>>();

    let result = compact_messages(&messages, CompactionStrategy::HardClear { keep_recent: 10 });

    assert_eq!(result.len(), 5);
}

#[test]
fn test_tool_result_truncation() {
    let long_content = "a".repeat(10000);
    let messages = vec![
        text_message(Role::User, "Short message"),
        text_message(Role::Tool, &long_content),
    ];

    let result = compact_messages(&messages, CompactionStrategy::ToolResultTruncation { max_chars: 1000 });

    assert_eq!(result.len(), 2);

    // First message should be unchanged
    assert_eq!(message_to_text(&result[0]), "Short message");

    // Second message should be truncated
    let truncated_text = message_to_text(&result[1]);
    assert!(truncated_text.starts_with("a".repeat(1000).as_str()));
    assert!(truncated_text.contains("[truncated]"));
    assert_eq!(truncated_text.len(), 1012); // 1000 + " [truncated]".len()
}

#[test]
fn test_summary_strategy() {
    let messages = (0..20)
        .map(|i| text_message(Role::User, &format!("Message {}", i)))
        .collect::<Vec<_>>();

    let result = compact_messages(&messages, CompactionStrategy::Summary { keep_recent: 5 });

    assert_eq!(result.len(), 6); // 1 summary + 5 recent
}

#[test]
fn test_summary_no_op_when_under_limit() {
    let messages = (0..5)
        .map(|i| text_message(Role::User, &format!("Message {}", i)))
        .collect::<Vec<_>>();

    let result = compact_messages(&messages, CompactionStrategy::Summary { keep_recent: 10 });

    assert_eq!(result.len(), 5);
}

#[test]
fn test_adaptive_chunking() {
    let messages = (0..20)
        .map(|i| text_message(Role::User, &format!("Message {}", i)))
        .collect::<Vec<_>>();

    let result = compact_messages(&messages, CompactionStrategy::AdaptiveChunking);

    // Should chunk old messages and keep recent
    assert!(result.len() < 20);
    assert!(result.iter().any(|m| message_to_text(m).contains("Previous")));
}

#[test]
fn test_adaptive_chunking_no_op_when_under_limit() {
    let messages = (0..5)
        .map(|i| text_message(Role::User, &format!("Message {}", i)))
        .collect::<Vec<_>>();

    let result = compact_messages(&messages, CompactionStrategy::AdaptiveChunking);

    // Not enough messages to chunk
    assert_eq!(result.len(), 5);
}

#[test]
fn test_estimate_token_count() {
    let messages = vec![
        text_message(Role::User, "Hello"),
        text_message(Role::Assistant, "Hi there"),
    ];

    // "HelloHi there" = 14 chars / 4 = ~3 tokens
    let tokens = estimate_token_count(&messages);
    assert!(tokens >= 2 && tokens <= 5);
}

#[test]
fn test_select_strategy_hard_clear() {
    let guard = ContextWindowGuard {
        warn_threshold: 0.8,
        hard_limit: 10000,
        min_available: 1000,
    };

    // At hard limit
    let strategy = select_strategy(&guard, 10000);
    assert!(matches!(strategy, CompactionStrategy::HardClear { .. }));

    // Over hard limit
    let strategy = select_strategy(&guard, 15000);
    assert!(matches!(strategy, CompactionStrategy::HardClear { .. }));
}

#[test]
fn test_select_strategy_summary() {
    let guard = ContextWindowGuard {
        warn_threshold: 0.8,
        hard_limit: 10000,
        min_available: 1000,
    };

    // At warn threshold (80% of 10000 = 8000)
    let strategy = select_strategy(&guard, 8000);
    assert!(matches!(strategy, CompactionStrategy::Summary { .. }));

    // Over warn threshold
    let strategy = select_strategy(&guard, 9000);
    assert!(matches!(strategy, CompactionStrategy::Summary { .. }));
}

#[test]
fn test_select_strategy_adaptive_chunking() {
    let guard = ContextWindowGuard {
        warn_threshold: 0.8,
        hard_limit: 10000,
        min_available: 1000,
    };

    // Under warn threshold
    let strategy = select_strategy(&guard, 5000);
    assert!(matches!(strategy, CompactionStrategy::AdaptiveChunking));
}

#[test]
fn test_compact_messages_empty_list() {
    let messages: Vec<Message> = vec![];

    let result = compact_messages(&messages, CompactionStrategy::HardClear { keep_recent: 5 });
    assert!(result.is_empty());
}

#[test]
fn test_compact_messages_preserves_role() {
    let messages = vec![
        text_message(Role::User, "User message"),
        text_message(Role::Assistant, "Assistant message"),
    ];

    let result = compact_messages(&messages, CompactionStrategy::HardClear { keep_recent: 5 });

    assert_eq!(result.len(), 2);
    assert_eq!(result[0].role, Role::User);
    assert_eq!(result[1].role, Role::Assistant);
}

#[test]
fn test_compact_messages_with_empty_strategy() {
    let messages = vec![
        text_message(Role::User, "Message 1"),
        text_message(Role::Assistant, "Message 2"),
    ];

    // Using Summary with keep_recent larger than message count
    let result = compact_messages(&messages, CompactionStrategy::Summary { keep_recent: 10 });

    assert_eq!(result.len(), 2);
    assert_eq!(message_to_text(&result[0]), "Message 1");
    assert_eq!(message_to_text(&result[1]), "Message 2");
}
