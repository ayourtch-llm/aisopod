//! Session compaction strategies for managing context window size.
//!
//! This module provides adaptive history compaction strategies to manage
//! context window size when conversations grow too long. It includes
//! strategies for adaptive chunking, summary-based compaction, hard clearing,
//! and oversized tool result truncation.

use crate::context_guard::ContextWindowGuard;
use aisopod_provider::{Message, MessageContent};

/// Severity level for compaction decisions.
///
/// This enum indicates how urgent compaction is based on token usage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompactionSeverity {
    /// No compaction needed
    None,
    /// Warn threshold exceeded, gentle compaction recommended
    Warn,
    /// Hard limit exceeded, aggressive compaction required
    Critical,
}

/// Compaction strategy for reducing context window size.
///
/// Different strategies are appropriate for different levels of context pressure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompactionStrategy {
    /// Adaptive chunking: Group older messages into chunks for summarization
    AdaptiveChunking,
    /// Summary strategy: Keep recent messages, summarize older ones
    Summary { keep_recent: usize },
    /// Hard clear: Keep only the most recent messages
    HardClear { keep_recent: usize },
    /// Tool result truncation: Truncate oversized tool outputs
    ToolResultTruncation { max_chars: usize },
}

impl Default for CompactionStrategy {
    fn default() -> Self {
        CompactionStrategy::Summary { keep_recent: 10 }
    }
}

/// Compact messages according to the specified strategy.
///
/// This function applies the compaction strategy to the message list
/// and returns a new list with compaction applied.
///
/// # Arguments
///
/// * `messages` - The messages to compact
/// * `strategy` - The compaction strategy to apply
///
/// # Returns
///
/// A new list of messages with compaction applied
pub fn compact_messages(messages: &[Message], strategy: CompactionStrategy) -> Vec<Message> {
    match strategy {
        CompactionStrategy::AdaptiveChunking => adaptive_chunking(messages),
        CompactionStrategy::Summary { keep_recent } => summary(messages, keep_recent),
        CompactionStrategy::HardClear { keep_recent } => hard_clear(messages, keep_recent),
        CompactionStrategy::ToolResultTruncation { max_chars } => {
            truncate_tool_results(messages, max_chars)
        }
    }
}

/// Adaptive chunking: Group older messages into chunks for summarization.
///
/// This strategy identifies topic boundaries or groups messages by count,
/// then replaces each chunk with a placeholder that can be expanded later
/// with actual summarization.
fn adaptive_chunking(messages: &[Message]) -> Vec<Message> {
    if messages.is_empty() {
        return Vec::new();
    }

    // Keep the last few messages intact (typically user/assistant turns)
    // Group older messages into a single "chunk" placeholder
    const CHUNK_SIZE: usize = 5;
    const KEEP_RECENT: usize = 2;

    if messages.len() <= KEEP_RECENT + CHUNK_SIZE {
        // Not enough messages to chunk
        return messages.to_vec();
    }

    let (old_messages, recent_messages) = messages.split_at(messages.len() - KEEP_RECENT);

    if old_messages.is_empty() {
        return messages.to_vec();
    }

    // Create a chunked representation of old messages
    let mut result = Vec::new();

    // Group old messages into chunks
    let chunks: Vec<Vec<&Message>> = old_messages
        .chunks(CHUNK_SIZE)
        .map(|chunk| chunk.iter().collect())
        .collect();

    for chunk in chunks {
        // Replace each chunk with a summary placeholder
        let summary = format!(
            "[Previous {} messages summarized: {}...]",
            chunk.len(),
            chunk_first_content(&chunk)
        );
        result.push(Message {
            role: aisopod_provider::Role::Assistant,
            content: MessageContent::Text(summary),
            tool_calls: None,
            tool_call_id: None,
        });
    }

    // Append recent messages
    result.extend(recent_messages.to_vec());

    result
}

/// Helper to get content from first message in chunk
fn chunk_first_content(chunk: &[&Message]) -> String {
    for msg in chunk.iter() {
        if let MessageContent::Text(text) = &msg.content {
            return text.chars().take(50).collect::<String>();
        }
    }
    String::new()
}

/// Extract text content from a message (helper for tests)
fn message_to_text(msg: &Message) -> String {
    match &msg.content {
        MessageContent::Text(text) => text.clone(),
        MessageContent::Parts(parts) => parts
            .iter()
            .filter_map(|p| match p {
                aisopod_provider::ContentPart::Text { text } => Some(text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join(" "),
        _ => String::new(),
    }
}

/// Summary strategy: Keep recent messages, replace older ones with a summary.
///
/// This strategy preserves the most recent conversation history while
/// replacing older messages with a single summary placeholder.
fn summary(messages: &[Message], keep_recent: usize) -> Vec<Message> {
    if messages.is_empty() {
        return Vec::new();
    }

    if messages.len() <= keep_recent {
        return messages.to_vec();
    }

    let (old_messages, recent_messages) = messages.split_at(messages.len() - keep_recent);

    if old_messages.is_empty() {
        return messages.to_vec();
    }

    let mut result = Vec::new();

    // Add a summary of old messages
    let summary = format!(
        "[Previous {} messages summarized. Conversation continues with recent messages.]",
        old_messages.len()
    );
    result.push(Message {
        role: aisopod_provider::Role::Assistant,
        content: MessageContent::Text(summary),
        tool_calls: None,
        tool_call_id: None,
    });

    // Append recent messages
    result.extend(recent_messages.to_vec());

    result
}

/// Hard clear: Keep only the most recent messages.
///
/// This is a aggressive strategy that drops older messages completely.
fn hard_clear(messages: &[Message], keep_recent: usize) -> Vec<Message> {
    if messages.len() <= keep_recent {
        return messages.to_vec();
    }

    // Keep only the most recent messages
    messages[messages.len() - keep_recent..].to_vec()
}

/// Truncate tool results that exceed a character limit.
///
/// This strategy walks through messages and truncates any tool result
/// content that exceeds the maximum allowed characters, appending a
/// [truncated] marker.
fn truncate_tool_results(messages: &[Message], max_chars: usize) -> Vec<Message> {
    if messages.is_empty() {
        return Vec::new();
    }

    messages
        .iter()
        .map(|msg| {
            match &msg.content {
                MessageContent::Text(ref content) => {
                    if content.len() > max_chars {
                        let truncated = &content[..max_chars];
                        let new_content = format!("{} [truncated]", truncated);
                        Message {
                            role: msg.role.clone(),
                            content: MessageContent::Text(new_content),
                            tool_calls: msg.tool_calls.clone(),
                            tool_call_id: msg.tool_call_id.clone(),
                        }
                    } else {
                        msg.clone()
                    }
                }
                MessageContent::Parts(ref parts) => {
                    // For multi-modal content, we need to process text parts
                    // For simplicity, keep as-is (could be enhanced for image data)
                    msg.clone()
                }
                _ => msg.clone(),
            }
        })
        .collect()
}

/// Select an appropriate compaction strategy based on token usage.
///
/// This function determines the best strategy based on the context window
/// guard's thresholds and the current token count.
///
/// # Arguments
///
/// * `guard` - The context window guard with thresholds
/// * `token_count` - The current token count
///
/// # Returns
///
/// The recommended compaction strategy
pub fn select_strategy(guard: &ContextWindowGuard, token_count: usize) -> CompactionStrategy {
    if token_count >= guard.hard_limit {
        // Hard limit exceeded - use hard clear
        CompactionStrategy::HardClear {
            keep_recent: guard.min_available / 100, // Estimate 100 tokens per message
        }
    } else if token_count >= (guard.warn_threshold * guard.hard_limit as f64) as usize {
        // Warn threshold exceeded - use summary
        CompactionStrategy::Summary { keep_recent: 10 }
    } else {
        // Gentle compaction - adaptive chunking
        CompactionStrategy::AdaptiveChunking
    }
}

/// Calculate token count from messages (approximate).
///
/// This is a simple heuristic that estimates tokens based on character count.
/// A more accurate implementation would use an actual tokenizer.
pub fn estimate_token_count(messages: &[Message]) -> usize {
    // Rough estimate: ~4 characters per token
    messages
        .iter()
        .map(|msg| match &msg.content {
            MessageContent::Text(text) => text.len(),
            MessageContent::Parts(parts) => parts
                .iter()
                .map(|part| match part {
                    aisopod_provider::ContentPart::Text { text } => text.len(),
                    aisopod_provider::ContentPart::Image { data, .. } => data.len(),
                    _ => 0,
                })
                .sum::<usize>(),
            _ => 0,
        })
        .sum::<usize>()
        / 4
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text_message(role: aisopod_provider::Role, content: &str) -> Message {
        Message {
            role,
            content: MessageContent::Text(content.to_string()),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    #[test]
    fn test_compaction_strategy_default() {
        let strategy = CompactionStrategy::default();
        assert!(matches!(strategy, CompactionStrategy::Summary { .. }));
    }

    #[test]
    fn test_hard_clear_truncates() {
        let messages = (0..20)
            .map(|i| text_message(aisopod_provider::Role::User, &format!("Message {}", i)))
            .collect::<Vec<_>>();

        let result = compact_messages(&messages, CompactionStrategy::HardClear { keep_recent: 5 });

        assert_eq!(result.len(), 5);
        // Should keep the most recent messages
        assert!(result
            .iter()
            .any(|m| message_to_text(m).contains("Message 19")));
        assert!(result
            .iter()
            .any(|m| message_to_text(m).contains("Message 15")));
    }

    #[test]
    fn test_hard_clear_no_op_when_under_limit() {
        let messages = (0..5)
            .map(|i| text_message(aisopod_provider::Role::User, &format!("Message {}", i)))
            .collect::<Vec<_>>();

        let result = compact_messages(&messages, CompactionStrategy::HardClear { keep_recent: 10 });

        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_tool_result_truncation() {
        let long_content = "a".repeat(10000);
        let messages = vec![
            text_message(aisopod_provider::Role::User, "Short message"),
            text_message(aisopod_provider::Role::Tool, &long_content),
        ];

        let result = compact_messages(
            &messages,
            CompactionStrategy::ToolResultTruncation { max_chars: 1000 },
        );

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
            .map(|i| text_message(aisopod_provider::Role::User, &format!("Message {}", i)))
            .collect::<Vec<_>>();

        let result = compact_messages(&messages, CompactionStrategy::Summary { keep_recent: 5 });

        assert_eq!(result.len(), 6); // 1 summary + 5 recent
    }

    #[test]
    fn test_summary_no_op_when_under_limit() {
        let messages = (0..5)
            .map(|i| text_message(aisopod_provider::Role::User, &format!("Message {}", i)))
            .collect::<Vec<_>>();

        let result = compact_messages(&messages, CompactionStrategy::Summary { keep_recent: 10 });

        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_adaptive_chunking() {
        let messages = (0..20)
            .map(|i| text_message(aisopod_provider::Role::User, &format!("Message {}", i)))
            .collect::<Vec<_>>();

        let result = compact_messages(&messages, CompactionStrategy::AdaptiveChunking);

        // Should chunk old messages and keep recent
        assert!(result.len() < 20);
        assert!(result
            .iter()
            .any(|m| message_to_text(m).contains("Previous")));
    }

    #[test]
    fn test_adaptive_chunking_no_op_when_under_limit() {
        let messages = (0..5)
            .map(|i| text_message(aisopod_provider::Role::User, &format!("Message {}", i)))
            .collect::<Vec<_>>();

        let result = compact_messages(&messages, CompactionStrategy::AdaptiveChunking);

        // Not enough messages to chunk
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_estimate_token_count() {
        let messages = vec![
            text_message(aisopod_provider::Role::User, "Hello"),
            text_message(aisopod_provider::Role::Assistant, "Hi there"),
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
}
