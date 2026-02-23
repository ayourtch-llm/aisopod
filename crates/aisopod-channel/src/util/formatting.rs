//! Message formatting normalization across platforms.
//!
//! This module provides a common representation for formatted text that can be
//! converted between different platform-specific markdown formats.
//!
//! # Supported Platforms
//!
//! - Telegram (MarkdownV2)
//! - Discord (Discord markdown)
//! - Slack (mrkdwn)
//! - WhatsApp (plain text with limited formatting)

use std::fmt;

/// Represents a piece of formatted text in an intermediate format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormatSegment {
    /// Plain text content
    Text(String),
    /// Bold text
    Bold(String),
    /// Italic text
    Italic(String),
    /// Strikethrough text
    Strikethrough(String),
    /// Inline code
    Code(String),
    /// Code block
    CodeBlock { language: Option<String>, content: String },
    /// Hyperlink
    Link { url: String, text: String },
    /// Quoted text
    Quote(String),
    /// Superscript text
    Superscript(String),
    /// Subscript text
    Subscript(String),
    /// Underline text
    Underline(String),
    /// Small text
    Small(String),
    /// Preformatted text
    Pre(String),
}

/// A normalized representation of markdown content.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NormalizedMarkdown {
    /// The segments that make up this markdown content
    pub segments: Vec<FormatSegment>,
}

impl NormalizedMarkdown {
    /// Create a new empty NormalizedMarkdown
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    /// Create a NormalizedMarkdown from plain text
    pub fn from_text(text: &str) -> Self {
        Self {
            segments: vec![FormatSegment::Text(text.to_string())],
        }
    }

    /// Convert to plain text by stripping all formatting
    pub fn to_plain_text(&self) -> String {
        self.segments
            .iter()
            .map(|segment| match segment {
                FormatSegment::Text(text) => text.clone(),
                FormatSegment::Bold(text) => text.clone(),
                FormatSegment::Italic(text) => text.clone(),
                FormatSegment::Strikethrough(text) => text.clone(),
                FormatSegment::Code(text) => text.clone(),
                FormatSegment::CodeBlock { content, .. } => content.clone(),
                FormatSegment::Link { text, .. } => text.clone(),
                FormatSegment::Quote(text) => text.clone(),
                FormatSegment::Superscript(text) => text.clone(),
                FormatSegment::Subscript(text) => text.clone(),
                FormatSegment::Underline(text) => text.clone(),
                FormatSegment::Small(text) => text.clone(),
                FormatSegment::Pre(text) => text.clone(),
            })
            .collect::<Vec<_>>()
            .join("")
    }

    /// Convert to Telegram MarkdownV2
    pub fn to_telegram_markdown(&self) -> String {
        self.segments
            .iter()
            .map(|segment| match segment {
                FormatSegment::Text(text) => escape_telegram(text),
                FormatSegment::Bold(text) => format!("*{}*", escape_telegram(text)),
                FormatSegment::Italic(text) => format!("_{}_", escape_telegram(text)),
                FormatSegment::Strikethrough(text) => format!("~{}~", escape_telegram(text)),
                FormatSegment::Code(text) => format!("`{}`", escape_telegram(text)),
                FormatSegment::CodeBlock { language, content } => {
                    let lang = language.as_deref().unwrap_or("");
                    format!("```{}\n{}\n```", lang, content)
                }
                FormatSegment::Link { url, text } => {
                    format!("[{}]({})", escape_telegram(text), url)
                }
                FormatSegment::Quote(text) => format!("> {}", escape_telegram(text)),
                FormatSegment::Superscript(text) => format!("^{}^", escape_telegram(text)),
                FormatSegment::Subscript(text) => format!(",{},", escape_telegram(text)),
                FormatSegment::Underline(text) => format!("<u>{}</u>", escape_telegram(text)),
                FormatSegment::Small(text) => format!("<small>{}</small>", escape_telegram(text)),
                FormatSegment::Pre(text) => format!("```{}\n```", text),
            })
            .collect::<Vec<_>>()
            .join("")
    }

    /// Convert to Discord markdown
    pub fn to_discord_markdown(&self) -> String {
        self.segments
            .iter()
            .map(|segment| match segment {
                FormatSegment::Text(text) => text.clone(),
                FormatSegment::Bold(text) => format!("**{}**", text),
                FormatSegment::Italic(text) => format!("_{}_", text),
                FormatSegment::Strikethrough(text) => format!("~~{}~~", text),
                FormatSegment::Code(text) => format!("`{}`", text),
                FormatSegment::CodeBlock { language, content } => {
                    let lang = language.as_deref().unwrap_or("");
                    format!("```{}\n{}\n```", lang, content)
                }
                FormatSegment::Link { url, text } => {
                    eprintln!("Link URL: {}", url);
                    format!("[{}]({})", escape_telegram(text), url)
                }
                FormatSegment::Quote(text) => format!("> {}", escape_telegram(text)),
                FormatSegment::Superscript(text) => format!("^^{}^^", text),
                FormatSegment::Subscript(text) => format!(",{},,", text),
                FormatSegment::Underline(text) => format!("__{}__", text),
                FormatSegment::Small(text) => format!("<small>{}</small>", text),
                FormatSegment::Pre(text) => format!("```\n{}\n```", text),
            })
            .collect::<Vec<_>>()
            .join("")
    }

    /// Convert to Slack mrkdwn
    pub fn to_slack_mrkdwn(&self) -> String {
        self.segments
            .iter()
            .map(|segment| match segment {
                FormatSegment::Text(text) => escape_slack(text),
                FormatSegment::Bold(text) => format!("*{}*", escape_slack(text)),
                FormatSegment::Italic(text) => format!("_{}_", escape_slack(text)),
                FormatSegment::Strikethrough(text) => format!("~{}~", escape_slack(text)),
                FormatSegment::Code(text) => format!("`{}`", escape_slack(text)),
                FormatSegment::CodeBlock { language, content } => {
                    let lang = language.as_deref().unwrap_or("");
                    format!("```{}\n{}\n```", lang, content)
                }
                FormatSegment::Link { url, text } => {
                    format!("<{}|{}>", url, escape_slack(text))
                }
                FormatSegment::Quote(text) => format!("> {}", escape_slack(text)),
                FormatSegment::Superscript(text) => format!("^{}^", escape_slack(text)),
                FormatSegment::Subscript(text) => format!(",{},,", text),
                FormatSegment::Underline(text) => format!("<u>{}</u>", escape_slack(text)),
                FormatSegment::Small(text) => format!("<small>{}</small>", escape_slack(text)),
                FormatSegment::Pre(text) => format!("```\n{}\n```", text),
            })
            .collect::<Vec<_>>()
            .join("")
    }

    /// Convert to WhatsApp text (plain text with limited formatting)
    pub fn to_whatsapp_text(&self) -> String {
        // WhatsApp has very limited formatting support
        // We preserve the structure but minimize formatting
        self.segments
            .iter()
            .map(|segment| match segment {
                FormatSegment::Text(text) => text.clone(),
                FormatSegment::Bold(text) => text.clone(),
                FormatSegment::Italic(text) => text.clone(),
                FormatSegment::Strikethrough(text) => text.clone(),
                FormatSegment::Code(text) => text.clone(),
                FormatSegment::CodeBlock { content, .. } => content.clone(),
                FormatSegment::Link { text, url } => format!("{} ({})", text, url),
                FormatSegment::Quote(text) => text.clone(),
                FormatSegment::Superscript(text) => text.clone(),
                FormatSegment::Subscript(text) => text.clone(),
                FormatSegment::Underline(text) => text.clone(),
                FormatSegment::Small(text) => text.clone(),
                FormatSegment::Pre(text) => text.clone(),
            })
            .collect::<Vec<_>>()
            .join("")
    }

    /// Add a text segment
    pub fn add_text(&mut self, text: &str) {
        self.segments.push(FormatSegment::Text(text.to_string()));
    }

    /// Add a bold segment
    pub fn add_bold(&mut self, text: &str) {
        self.segments.push(FormatSegment::Bold(text.to_string()));
    }

    /// Add an italic segment
    pub fn add_italic(&mut self, text: &str) {
        self.segments.push(FormatSegment::Italic(text.to_string()));
    }

    /// Add a strikethrough segment
    pub fn add_strikethrough(&mut self, text: &str) {
        self.segments.push(FormatSegment::Strikethrough(text.to_string()));
    }

    /// Add a code segment
    pub fn add_code(&mut self, text: &str) {
        self.segments.push(FormatSegment::Code(text.to_string()));
    }

    /// Add a link segment
    pub fn add_link(&mut self, url: &str, text: &str) {
        self.segments.push(FormatSegment::Link {
            url: url.to_string(),
            text: text.to_string(),
        });
    }

    /// Add a quote segment
    pub fn add_quote(&mut self, text: &str) {
        self.segments.push(FormatSegment::Quote(text.to_string()));
    }
}

impl fmt::Display for NormalizedMarkdown {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_plain_text())
    }
}

// Telegram MarkdownV2 escaping
fn escape_telegram(text: &str) -> String {
    let mut result = String::new();
    for c in text.chars() {
        match c {
            '_' | '*' | '[' | ']' | '(' | ')' | '~' | '`' | '>' | '#' | '+' | '-' | '=' | '|' | '{' | '}' | '.' | '!' => {
                result.push('\\');
                result.push(c);
            }
            c => result.push(c),
        }
    }
    result
}

// Slack mrkdwn escaping
fn escape_slack(text: &str) -> String {
    let mut result = String::new();
    for c in text.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            c => result.push(c),
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalized_markdown_creation() {
        let markdown = NormalizedMarkdown::new();
        assert!(markdown.segments.is_empty());
    }

    #[test]
    fn test_from_text() {
        let markdown = NormalizedMarkdown::from_text("Hello World");
        assert_eq!(markdown.segments.len(), 1);
        assert_eq!(markdown.to_plain_text(), "Hello World");
    }

    #[test]
    fn test_to_plain_text() {
        let mut markdown = NormalizedMarkdown::new();
        markdown.add_bold("Hello");
        markdown.add_text(" ");
        markdown.add_italic("World");

        assert_eq!(markdown.to_plain_text(), "Hello World");
    }

    #[test]
    fn test_telegram_markdown() {
        let mut markdown = NormalizedMarkdown::new();
        markdown.add_bold("Hello");
        markdown.add_text(" ");
        markdown.add_italic("World");

        let result = markdown.to_telegram_markdown();
        assert_eq!(result, "*Hello* _World_");
    }

    #[test]
    fn test_discord_markdown() {
        let mut markdown = NormalizedMarkdown::new();
        markdown.add_bold("Hello");
        markdown.add_text(" ");
        markdown.add_italic("World");

        let result = markdown.to_discord_markdown();
        assert_eq!(result, "**Hello** _World_");
    }

    #[test]
    fn test_slack_mrkdwn() {
        let mut markdown = NormalizedMarkdown::new();
        markdown.add_bold("Hello");
        markdown.add_text(" ");
        markdown.add_italic("World");

        let result = markdown.to_slack_mrkdwn();
        assert_eq!(result, "*Hello* _World_");
    }

    #[test]
    fn test_whatsapp_text() {
        let mut markdown = NormalizedMarkdown::new();
        markdown.add_bold("Hello");
        markdown.add_text(" ");
        markdown.add_italic("World");

        let result = markdown.to_whatsapp_text();
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_link_formatting() {
        let mut markdown = NormalizedMarkdown::new();
        markdown.add_link("https://example.com", "Example");

        let telegram = markdown.to_telegram_markdown();
        eprintln!("Telegram output: {}", telegram);
        assert!(telegram.contains("[Example](https://example.com)"));

        let discord = markdown.to_discord_markdown();
        assert!(discord.contains("[Example](https://example.com)"));

        let slack = markdown.to_slack_mrkdwn();
        assert!(slack.contains("<https://example.com|Example>"));
    }

    #[test]
    fn test_telegram_escaping() {
        let mut markdown = NormalizedMarkdown::new();
        markdown.add_text("Hello _world*!");

        let result = markdown.to_telegram_markdown();
        // Escaped characters should be preceded by backslash
        assert!(result.contains("\\_"));
        assert!(result.contains("\\*"));
    }

    #[test]
    fn test_slack_escaping() {
        let mut markdown = NormalizedMarkdown::new();
        markdown.add_text("Hello <world> & friends");

        let result = markdown.to_slack_mrkdwn();
        assert!(result.contains("&lt;"));
        assert!(result.contains("&gt;"));
        assert!(result.contains("&amp;"));
    }

    #[test]
    fn test_code_block_formatting() {
        let mut markdown = NormalizedMarkdown::new();
        markdown.add_text("Here is some code: ");
        markdown.segments.push(FormatSegment::CodeBlock {
            language: Some("rust".to_string()),
            content: "fn main() { println!(\"Hello\"); }".to_string(),
        });

        let telegram = markdown.to_telegram_markdown();
        assert!(telegram.contains("```rust"));
        assert!(telegram.contains("fn main()"));
    }

    #[test]
    fn test_quote_formatting() {
        let mut markdown = NormalizedMarkdown::new();
        markdown.add_text("Said: ");
        markdown.add_quote("Hello world");

        let telegram = markdown.to_telegram_markdown();
        assert!(telegram.contains("> Hello world"));
    }

    #[test]
    fn test_complex_formatting() {
        let mut markdown = NormalizedMarkdown::new();
        markdown.add_text("The ");
        markdown.add_bold("quick");
        markdown.add_text(" ");
        markdown.add_italic("brown");
        markdown.add_text(" ");
        markdown.add_bold("fox");
        markdown.add_text(" jumps over the ");
        markdown.add_strikethrough("lazy");
        markdown.add_text(" dog.");

        let plain = markdown.to_plain_text();
        assert!(plain.contains("The quick brown fox jumps over the lazy dog."));
    }
}

// ============================================================================
// Parsing functions
// ============================================================================

/// Parse Telegram MarkdownV2 syntax into NormalizedMarkdown
pub fn from_telegram_markdown(input: &str) -> NormalizedMarkdown {
    let mut result = NormalizedMarkdown::new();
    parse_markdown(input, &mut result, ParseFormat::Telegram);
    result
}

/// Parse Discord markdown syntax into NormalizedMarkdown
pub fn from_discord_markdown(input: &str) -> NormalizedMarkdown {
    let mut result = NormalizedMarkdown::new();
    parse_markdown(input, &mut result, ParseFormat::Discord);
    result
}

/// Parse Slack mrkdwn syntax into NormalizedMarkdown
pub fn from_slack_mrkdwn(input: &str) -> NormalizedMarkdown {
    let mut result = NormalizedMarkdown::new();
    parse_markdown(input, &mut result, ParseFormat::Slack);
    result
}

/// Parse plain text into NormalizedMarkdown
pub fn from_plain_text(input: &str) -> NormalizedMarkdown {
    NormalizedMarkdown::from_text(input)
}

#[derive(Clone, Copy)]
enum ParseFormat {
    Telegram,
    Discord,
    Slack,
}

/// Generic markdown parser
fn parse_markdown(input: &str, result: &mut NormalizedMarkdown, format: ParseFormat) {
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    let mut current_text = String::new();

    while i < chars.len() {
        // Check for code blocks first
        if chars[i] == '`' {
            // Check for code block (3 backticks)
            if i + 2 < chars.len() && chars[i + 1] == '`' && chars[i + 2] == '`' {
                // Save current text first
                if !current_text.is_empty() {
                    result.add_text(&current_text);
                    current_text.clear();
                }

                i += 3; // Skip ```
                let start = i;

                // Find closing ```
                while i < chars.len() && !(chars[i] == '`' && i + 1 < chars.len() && chars[i + 1] == '`' && i + 2 < chars.len() && chars[i + 2] == '`') {
                    i += 1;
                }

                let content = input[start..i].trim().to_string();
                result.segments.push(FormatSegment::CodeBlock {
                    language: None,
                    content,
                });
                i += 3; // Skip closing ```
                continue;
            } else {
                // Inline code
                if !current_text.is_empty() {
                    result.add_text(&current_text);
                    current_text.clear();
                }

                i += 1; // Skip opening `
                let start = i;

                while i < chars.len() && chars[i] != '`' {
                    i += 1;
                }

                let content = input[start..i].to_string();
                result.add_code(&content);
                i += 1; // Skip closing `
                continue;
            }
        }

        // Check for bold/italic based on format
        match format {
            ParseFormat::Telegram | ParseFormat::Discord => {
                if chars[i] == '*' {
                    if !current_text.is_empty() {
                        result.add_text(&current_text);
                        current_text.clear();
                    }
                    i += 1;
                    let start = i;
                    while i < chars.len() && chars[i] != '*' {
                        i += 1;
                    }
                    let content = input[start..i].to_string();
                    result.add_bold(&content);
                    if i < chars.len() {
                        i += 1; // Skip closing *
                    }
                    continue;
                } else if chars[i] == '_' {
                    if !current_text.is_empty() {
                        result.add_text(&current_text);
                        current_text.clear();
                    }
                    i += 1;
                    let start = i;
                    while i < chars.len() && chars[i] != '_' {
                        i += 1;
                    }
                    let content = input[start..i].to_string();
                    result.add_italic(&content);
                    if i < chars.len() {
                        i += 1; // Skip closing _
                    }
                    continue;
                }
            }
            ParseFormat::Slack => {
                // Slack uses * for bold and _ for italic (but only outside of links)
                // For simplicity, we'll parse the same way
                if chars[i] == '*' {
                    if !current_text.is_empty() {
                        result.add_text(&current_text);
                        current_text.clear();
                    }
                    i += 1;
                    let start = i;
                    while i < chars.len() && chars[i] != '*' {
                        i += 1;
                    }
                    let content = input[start..i].to_string();
                    result.add_bold(&content);
                    if i < chars.len() {
                        i += 1; // Skip closing *
                    }
                    continue;
                } else if chars[i] == '_' {
                    if !current_text.is_empty() {
                        result.add_text(&current_text);
                        current_text.clear();
                    }
                    i += 1;
                    let start = i;
                    while i < chars.len() && chars[i] != '_' {
                        i += 1;
                    }
                    let content = input[start..i].to_string();
                    result.add_italic(&content);
                    if i < chars.len() {
                        i += 1; // Skip closing _
                    }
                    continue;
                }
            }
        }

        current_text.push(chars[i]);
        i += 1;
    }

    // Add remaining text
    if !current_text.is_empty() {
        result.add_text(&current_text);
    }
}
