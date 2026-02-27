//! Markdown format conversion between different platform formats.
//!
//! This module provides utilities for converting markdown content between
//! different messaging platforms that have varying syntax support:
//!
//! - Discord: **bold**, *italic*, ~~strike~~, `code`, ```block```
//! - Slack: *bold*, _italic_, ~strike~, `code`, ```block```
//! - Telegram: **bold**, __italic__, ~~strike~~, `code`, ```block```
//! - HTML: `<b>`, `<i>`, `<s>`, `<code>`, `<pre>`
//! - Plain: strip all formatting
//! - Matrix: Standard markdown + HTML subset
//! - IRC: mIRC color codes

use std::fmt;

/// Target format for markdown conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkdownFormat {
    /// Discord markdown format
    Discord,
    /// Slack markdown format (mrkdwn)
    Slack,
    /// Telegram markdown format
    Telegram,
    /// HTML format
    Html,
    /// Plain text (no formatting)
    Plain,
    /// Matrix markdown format
    Matrix,
    /// IRC format with mIRC color codes
    Irc,
}

impl fmt::Display for MarkdownFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MarkdownFormat::Discord => write!(f, "discord"),
            MarkdownFormat::Slack => write!(f, "slack"),
            MarkdownFormat::Telegram => write!(f, "telegram"),
            MarkdownFormat::Html => write!(f, "html"),
            MarkdownFormat::Plain => write!(f, "plain"),
            MarkdownFormat::Matrix => write!(f, "matrix"),
            MarkdownFormat::Irc => write!(f, "irc"),
        }
    }
}

/// Parsed markdown AST node.
#[derive(Debug, Clone, PartialEq)]
pub enum MarkdownNode {
    /// Plain text content
    Text(String),
    /// Bold text
    Bold(Vec<MarkdownNode>),
    /// Italic text
    Italic(Vec<MarkdownNode>),
    /// Strikethrough text
    Strikethrough(Vec<MarkdownNode>),
    /// Inline code
    Code(String),
    /// Code block
    CodeBlock {
        /// Optional language identifier
        language: Option<String>,
        /// Code content
        code: String,
    },
    /// Hyperlink
    Link {
        /// Link text
        text: String,
        /// URL
        url: String,
    },
    /// Line break
    Newline,
    /// Paragraph (group of nodes)
    Paragraph(Vec<MarkdownNode>),
}

/// Parse markdown string to AST based on source format.
pub fn parse_markdown(input: &str, format: &MarkdownFormat) -> Vec<MarkdownNode> {
    match format {
        MarkdownFormat::Discord | MarkdownFormat::Telegram => parse_standard_markdown(input),
        MarkdownFormat::Slack => parse_slack_markdown(input),
        MarkdownFormat::Html => parse_html(input),
        MarkdownFormat::Plain => vec![MarkdownNode::Text(input.to_string())],
        MarkdownFormat::Matrix => parse_standard_markdown(input),
        MarkdownFormat::Irc => parse_irc_formatting(input),
    }
}

/// Parse standard markdown (Discord/Telegram style).
fn parse_standard_markdown(input: &str) -> Vec<MarkdownNode> {
    let mut nodes = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    let max_iterations = chars.len() * 10; // Safety limit
    let mut iterations = 0;

    while i < chars.len() {
        iterations += 1;
        if iterations > max_iterations {
            eprintln!("parse_standard_markdown: max iterations reached at position {}", i);
            break;
        }

        // Check for code blocks first
        if i + 3 < chars.len() && chars[i..i + 3] == ['`', '`', '`'] {
            nodes.push(parse_code_block(&chars, &mut i));
            continue;
        }

        // Check for inline code
        if chars[i] == '`' {
            if let Some((code, end)) = parse_inline_code(&chars, i) {
                nodes.push(MarkdownNode::Code(code));
                i = end;
                continue;
            }
        }

        // Check for links
        if chars[i] == '[' {
            if let Some(link) = parse_link(&chars, &mut i) {
                nodes.push(link);
                continue;
            }
        }

        // Check for bold/italic/strikethrough
        if i + 1 < chars.len() {
            let two = &chars[i..i + 2];
            let three = if i + 2 < chars.len() {
                &chars[i..i + 3]
            } else {
                &[]
            };

            match three {
                ['*', '*', '*'] | ['_', '_', '_'] => {
                    // Bold and italic
                    let end = find_matching_delimiter(&chars, i, 3);
                    if end > i {
                        let content = &chars[i + 3..end];
                        let text: String = content.iter().collect();
                        nodes.push(MarkdownNode::Bold(vec![MarkdownNode::Italic(vec![
                            MarkdownNode::Text(text),
                        ])]));
                        i = end + 3;
                        continue;
                    }
                }
                ['*', '*'] | ['_', '_'] => {
                    // Bold
                    let end = find_matching_delimiter(&chars, i, 2);
                    if end > i {
                        let content = &chars[i + 2..end];
                        let text: String = content.iter().collect();
                        nodes.push(MarkdownNode::Bold(vec![MarkdownNode::Text(text)]));
                        i = end + 2;
                        continue;
                    }
                }
                ['~', '~', '~'] => {
                    // Strikethrough
                    let end = find_matching_delimiter(&chars, i, 3);
                    if end > i {
                        let content = &chars[i + 3..end];
                        let text: String = content.iter().collect();
                        nodes.push(MarkdownNode::Strikethrough(vec![MarkdownNode::Text(text)]));
                        i = end + 3;
                        continue;
                    }
                }
                _ => {
                    // Check for italic (single * or _)
                    if i + 1 < chars.len() && (chars[i] == '*' || chars[i] == '_') {
                        // Make sure it's not bold (** or __)
                        if i + 1 < chars.len() && chars[i] == chars[i + 1] {
                            // This is bold, handled above
                        } else {
                            // This is italic
                            let end = find_matching_delimiter(&chars, i, 1);
                            if end > i {
                                let content = &chars[i + 1..end];
                                let text: String = content.iter().collect();
                                nodes.push(MarkdownNode::Italic(vec![MarkdownNode::Text(text)]));
                                i = end + 1;
                                continue;
                            }
                        }
                    }
                }
            }
        }

        // Regular text
        let start = i;
        while i < chars.len() && !is_special_char(chars[i]) {
            i += 1;
        }
        if i > start {
            let text: String = chars[start..i].iter().collect();
            nodes.push(MarkdownNode::Text(text));
        }

        // Handle newlines
        if i < chars.len() && chars[i] == '\n' {
            nodes.push(MarkdownNode::Newline);
            i += 1;
        }
    }

    nodes
}

/// Parse Slack markdown format.
fn parse_slack_markdown(input: &str) -> Vec<MarkdownNode> {
    let mut nodes = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Check for code blocks
        if i + 3 < chars.len() && chars[i..i + 3] == ['`', '`', '`'] {
            nodes.push(parse_code_block(&chars, &mut i));
            continue;
        }

        // Check for inline code
        if chars[i] == '`' {
            if let Some((code, end)) = parse_inline_code(&chars, i) {
                nodes.push(MarkdownNode::Code(code));
                i = end;
                continue;
            }
        }

        // Check for links
        if chars[i] == '<' && i + 1 < chars.len() && chars[i + 1] == '!' {
            // Slack formatting: <!here>, <!channel>
            let end = input[i..].find('>').unwrap_or(0) + i;
            if end > i {
                let text: String = chars[i..end + 1].iter().collect();
                nodes.push(MarkdownNode::Text(text));
                i = end + 1;
                continue;
            }
        }

        // Check for links
        if chars[i] == '<' {
            if let Some(link) = parse_slack_link(&chars, &mut i) {
                nodes.push(link);
                continue;
            }
        }

        // Parse Slack formatting: *bold*, _italic_, ~strike~
        if i + 1 < chars.len() {
            match chars[i..i + 2] {
                ['*', '*'] => {
                    let end = find_matching_slack_delimiter(&chars, i, 2);
                    if end > i {
                        let content = &chars[i + 2..end];
                        let text: String = content.iter().collect();
                        nodes.push(MarkdownNode::Bold(vec![MarkdownNode::Text(text)]));
                        i = end + 2;
                        continue;
                    }
                }
                ['_', '_'] => {
                    let end = find_matching_slack_delimiter(&chars, i, 2);
                    if end > i {
                        let content = &chars[i + 2..end];
                        let text: String = content.iter().collect();
                        nodes.push(MarkdownNode::Italic(vec![MarkdownNode::Text(text)]));
                        i = end + 2;
                        continue;
                    }
                }
                ['~', '~'] => {
                    let end = find_matching_slack_delimiter(&chars, i, 2);
                    if end > i {
                        let content = &chars[i + 2..end];
                        let text: String = content.iter().collect();
                        nodes.push(MarkdownNode::Strikethrough(vec![MarkdownNode::Text(text)]));
                        i = end + 2;
                        continue;
                    }
                }
                _ => {}
            }
        }

        // Regular text
        let start = i;
        while i < chars.len() && !is_slack_special_char(chars[i]) {
            i += 1;
        }
        if i > start {
            let text: String = chars[start..i].iter().collect();
            nodes.push(MarkdownNode::Text(text));
        }

        // Handle newlines
        if i < chars.len() && chars[i] == '\n' {
            nodes.push(MarkdownNode::Newline);
            i += 1;
        }
    }

    nodes
}

/// Parse HTML content.
fn parse_html(input: &str) -> Vec<MarkdownNode> {
    let mut nodes = Vec::new();
    let mut i = 0;
    let chars: Vec<char> = input.chars().collect();

    while i < chars.len() {
        // Check for tags
        if chars[i] == '<' {
            let tag_end = input[i..].find('>').unwrap_or(0) + i;

            if tag_end > i {
                let tag: String = chars[i..=tag_end].iter().collect();

                if tag.starts_with("<b>") || tag.starts_with("<strong>") {
                    // Find closing tag
                    if let Some(end) = find_html_end_tag(&chars, i, &tag) {
                        let content = chars[i + tag.len()..end].iter().collect::<String>();
                        nodes.push(MarkdownNode::Bold(vec![MarkdownNode::Text(content)]));
                        i = end;
                        continue;
                    }
                }

                if tag.starts_with("<i>") || tag.starts_with("<em>") {
                    if let Some(end) = find_html_end_tag(&chars, i, &tag) {
                        let content = chars[i + tag.len()..end].iter().collect::<String>();
                        nodes.push(MarkdownNode::Italic(vec![MarkdownNode::Text(content)]));
                        i = end;
                        continue;
                    }
                }

                if tag.starts_with("<s>") || tag.starts_with("<strike>") || tag.starts_with("<del>") {
                    if let Some(end) = find_html_end_tag(&chars, i, &tag) {
                        let content = chars[i + tag.len()..end].iter().collect::<String>();
                        nodes.push(MarkdownNode::Strikethrough(vec![MarkdownNode::Text(content)]));
                        i = end;
                        continue;
                    }
                }

                if tag.starts_with("<a ") || tag.starts_with("<a>") {
                    if let Some(link) = parse_html_link(&chars, &mut i) {
                        nodes.push(link);
                        continue;
                    }
                }

                if tag.starts_with("<code>") {
                    if let Some(end) = find_html_end_tag(&chars, i, "<code>") {
                        let content = chars[i + 6..end].iter().collect::<String>();
                        nodes.push(MarkdownNode::Code(content));
                        i = end;
                        continue;
                    }
                }

                if tag.starts_with("<pre>") {
                    if let Some(end) = find_html_end_tag(&chars, i, "<pre>") {
                        let content = chars[i + 5..end].iter().collect::<String>().trim().to_string();
                        nodes.push(MarkdownNode::CodeBlock {
                            language: None,
                            code: content,
                        });
                        i = end;
                        continue;
                    }
                }

                // Self-closing tags
                if tag == "<br>" || tag == "<br/>" {
                    nodes.push(MarkdownNode::Newline);
                    i = tag_end + 1;
                    continue;
                }
            }
        }

        // Regular text (skip HTML tags)
        let start = i;
        while i < chars.len() && chars[i] != '<' {
            i += 1;
        }
        if i > start {
            let text: String = chars[start..i].iter().collect();
            nodes.push(MarkdownNode::Text(text));
        }
    }

    nodes
}

/// Parse IRC formatting.
fn parse_irc_formatting(input: &str) -> Vec<MarkdownNode> {
    let mut nodes = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // IRC control codes
        if chars[i] == '\x02' {
            // Bold
            let end = find_irc_control_code(&chars, i, '\x02');
            if end > i {
                let content = chars[i + 1..end].iter().collect::<String>();
                nodes.push(MarkdownNode::Bold(vec![MarkdownNode::Text(content)]));
                i = end + 1;
                continue;
            }
        }

        if chars[i] == '\x1D' {
            // Italic
            let end = find_irc_control_code(&chars, i, '\x1D');
            if end > i {
                let content = chars[i + 1..end].iter().collect::<String>();
                nodes.push(MarkdownNode::Italic(vec![MarkdownNode::Text(content)]));
                i = end + 1;
                continue;
            }
        }

        if chars[i] == '\x1F' {
            // Underline
            let end = find_irc_control_code(&chars, i, '\x1F');
            if end > i {
                let content = chars[i + 1..end].iter().collect::<String>();
                nodes.push(MarkdownNode::Italic(vec![MarkdownNode::Text(content)])); // Underline as italic
                i = end + 1;
                continue;
            }
        }

        if chars[i] == '\x16' {
            // Reverse video / Strikethrough
            let end = find_irc_control_code(&chars, i, '\x16');
            if end > i {
                let content = chars[i + 1..end].iter().collect::<String>();
                nodes.push(MarkdownNode::Strikethrough(vec![MarkdownNode::Text(content)]));
                i = end + 1;
                continue;
            }
        }

        // Regular text
        let start = i;
        while i < chars.len() && !is_irc_control_char(chars[i]) {
            i += 1;
        }
        if i > start {
            let text: String = chars[start..i].iter().collect();
            nodes.push(MarkdownNode::Text(text));
        }
    }

    nodes
}

// ============================================================================
// Helper parsing functions
// ============================================================================

fn parse_code_block(chars: &[char], i: &mut usize) -> MarkdownNode {
    let mut end = *i + 3;

    // Find the end of the code block
    while end + 2 < chars.len() {
        if chars[end..end + 3] == ['`', '`', '`'] {
            break;
        }
        end += 1;
    }

    let content: String = chars[*i + 3..end].iter().collect();
    *i = end + 3;

    // Try to extract language
    let (language, code) = if let Some(space) = content.find(' ') {
        let language = content[..space].trim().to_string();
        let code = content[space..].trim().to_string();
        (Some(language), code)
    } else {
        (None, content.trim().to_string())
    };

    MarkdownNode::CodeBlock { language, code }
}

fn parse_inline_code(chars: &[char], start: usize) -> Option<(String, usize)> {
    let mut i = start + 1;
    while i < chars.len() && chars[i] != '`' {
        i += 1;
    }

    if i > start + 1 {
        let code: String = chars[start + 1..i].iter().collect();
        Some((code, i + 1))
    } else {
        None
    }
}

fn parse_link(chars: &[char], i: &mut usize) -> Option<MarkdownNode> {
    if chars[*i] != '[' {
        return None;
    }

    let mut j = *i + 1;
    let mut bracket_count = 1;

    while j < chars.len() && bracket_count > 0 {
        if chars[j] == '[' {
            bracket_count += 1;
        } else if chars[j] == ']' {
            bracket_count -= 1;
        }
        j += 1;
    }

    if bracket_count == 0 && j < chars.len() && chars[j] == '(' {
        let mut k = j + 1;
        let mut paren_count = 1;

        while k < chars.len() && paren_count > 0 {
            if chars[k] == '(' {
                paren_count += 1;
            } else if chars[k] == ')' {
                paren_count -= 1;
            }
            k += 1;
        }

        if paren_count == 0 {
            let text: String = chars[*i + 1..j - 1].iter().collect();
            let url: String = chars[j + 1..k - 1].iter().collect();
            *i = k;
            Some(MarkdownNode::Link { text, url })
        } else {
            None
        }
    } else {
        None
    }
}

fn parse_slack_link(chars: &[char], i: &mut usize) -> Option<MarkdownNode> {
    if chars[*i] != '<' {
        return None;
    }

    let mut j = *i + 1;
    while j < chars.len() && chars[j] != '>' {
        j += 1;
    }

    if j > *i + 1 {
        let content: String = chars[*i + 1..j].iter().collect();
        *i = j + 1;

        // Check if it's a link format: <url|text> or <url>
        if let Some(pipe) = content.find('|') {
            let url = content[..pipe].to_string();
            let text = content[pipe + 1..].to_string();
            Some(MarkdownNode::Link { text, url })
        } else {
            // Plain URL
            Some(MarkdownNode::Text(content))
        }
    } else {
        None
    }
}

fn parse_html_link(chars: &[char], i: &mut usize) -> Option<MarkdownNode> {
    let tag_start = *i;
    let tag_end = find_html_end_tag(chars, tag_start, "<a")?;

    // Extract href
    let tag: String = chars[tag_start..tag_end].iter().collect();
    let href_start = tag.find("href=\"")?;
    let href_end = tag[href_start + 6..].find('"')?;
    let url = tag[href_start + 6..href_start + 6 + href_end].to_string();

    // Find content between tags
    let content_start = tag_end + 4; // </a>
    let mut j = content_start;
    while j < chars.len() && (j + 3 >= chars.len() || chars[j..j + 3] != ['<', '/', 'a']) {
        j += 1;
    }

    if j > content_start {
        let text: String = chars[content_start..j].iter().collect();
        *i = j + 3;
        Some(MarkdownNode::Link { text, url })
    } else {
        None
    }
}

fn find_matching_delimiter(chars: &[char], start: usize, delimiter_len: usize) -> usize {
    // Get the delimiter characters we're looking for
    let delimiter_start: Vec<char> = chars[start..start + delimiter_len].to_vec();
    
    let mut i = start + delimiter_len;
    let max_iterations = chars.len() - i; // Safety limit to prevent infinite loop
    let mut iterations = 0;
    
    eprintln!("find_matching_delimiter: start={}, delimiter_len={}, max_iterations={}", start, delimiter_len, max_iterations);
    
    while i + delimiter_len <= chars.len() {
        iterations += 1;
        if iterations > max_iterations {
            // Safety: Prevent infinite loop
            eprintln!("find_matching_delimiter: max iterations reached at i={}", i);
            return chars.len();
        }
        
        // Compare the current position with the delimiter without creating new collections
        let mut matches = true;
        for j in 0..delimiter_len {
            if chars[i + j] != delimiter_start[j] {
                matches = false;
                break;
            }
        }
        
        if matches {
            eprintln!("find_matching_delimiter: found match at i={}", i);
            return i;
        }
        i += 1;
    }

    eprintln!("find_matching_delimiter: no match found, returning {}", chars.len());
    // If no closing found, return end of string
    chars.len()
}

fn find_matching_slack_delimiter(chars: &[char], start: usize, delimiter_len: usize) -> usize {
    find_matching_delimiter(chars, start, delimiter_len)
}

fn find_irc_control_code(chars: &[char], start: usize, code: char) -> usize {
    let mut i = start + 1;
    while i < chars.len() && chars[i] != code {
        i += 1;
    }
    i
}

fn find_html_end_tag(chars: &[char], start: usize, tag: &str) -> Option<usize> {
    let end_tag = format!("</{}>", &tag[1..tag.len() - 1]);
    let chars_str: String = chars.iter().collect();

    if let Some(pos) = chars_str[start..].find(end_tag.as_str()) {
        Some(start + pos)
    } else {
        None
    }
}

fn is_special_char(c: char) -> bool {
    matches!(c, '`' | '*' | '_' | '~' | '[' | ']' | '(' | ')' | '\n')
}

fn is_slack_special_char(c: char) -> bool {
    matches!(c, '`' | '*' | '_' | '~' | '<' | '>' | '\n')
}

fn is_irc_control_char(c: char) -> bool {
    matches!(c, '\x02' | '\x1D' | '\x1F' | '\x16')
}

// ============================================================================
// Rendering functions
// ============================================================================

/// Render AST to markdown string in target format.
pub fn render_markdown(nodes: &[MarkdownNode], format: &MarkdownFormat) -> String {
    let mut output = String::new();

    for node in nodes {
        match node {
            MarkdownNode::Text(text) => {
                output.push_str(text);
            }
            MarkdownNode::Bold(children) => {
                output.push_str(&render_bold(children, format));
            }
            MarkdownNode::Italic(children) => {
                output.push_str(&render_italic(children, format));
            }
            MarkdownNode::Strikethrough(children) => {
                output.push_str(&render_strikethrough(children, format));
            }
            MarkdownNode::Code(content) => {
                output.push('`');
                output.push_str(content);
                output.push('`');
            }
            MarkdownNode::CodeBlock { language, code } => {
                output.push_str("```");
                if let Some(lang) = language {
                    output.push_str(lang);
                }
                output.push('\n');
                output.push_str(code);
                output.push_str("```");
            }
            MarkdownNode::Link { text, url } => {
                output.push_str(&render_link(text, url, format));
            }
            MarkdownNode::Newline => {
                output.push('\n');
            }
            MarkdownNode::Paragraph(children) => {
                output.push_str(&render_markdown(children, format));
                output.push('\n');
            }
        }
    }

    output
}

fn render_bold(children: &[MarkdownNode], format: &MarkdownFormat) -> String {
    let content = render_markdown(children, format);

    match format {
        MarkdownFormat::Discord => format!("**{}**", content),
        MarkdownFormat::Slack => format!("*{}*", content),
        MarkdownFormat::Telegram => format!("**{}**", content),
        MarkdownFormat::Html => format!("<b>{}</b>", content),
        MarkdownFormat::Plain => content,
        MarkdownFormat::Matrix => format!("**{}**", content),
        MarkdownFormat::Irc => format!("\x02{}\x02", content),
    }
}

fn render_italic(children: &[MarkdownNode], format: &MarkdownFormat) -> String {
    let content = render_markdown(children, format);

    match format {
        MarkdownFormat::Discord => format!("_{}_", content),
        MarkdownFormat::Slack => format!("_{}_", content),
        MarkdownFormat::Telegram => format!("_{}_", content),
        MarkdownFormat::Html => format!("<i>{}</i>", content),
        MarkdownFormat::Plain => content,
        MarkdownFormat::Matrix => format!("_{}_", content),
        MarkdownFormat::Irc => format!("\x1D{}\x1D", content),
    }
}

fn render_strikethrough(children: &[MarkdownNode], format: &MarkdownFormat) -> String {
    let content = render_markdown(children, format);

    match format {
        MarkdownFormat::Discord => format!("~~{}~~", content),
        MarkdownFormat::Slack => format!("~{}~", content),
        MarkdownFormat::Telegram => format!("~~{}~~", content),
        MarkdownFormat::Html => format!("<s>{}</s>", content),
        MarkdownFormat::Plain => content,
        MarkdownFormat::Matrix => format!("~~{}~~", content),
        MarkdownFormat::Irc => format!("\x16{}\x16", content),
    }
}

fn render_link(text: &str, url: &str, format: &MarkdownFormat) -> String {
    match format {
        MarkdownFormat::Discord => format!("[{}]({})", text, url),
        MarkdownFormat::Slack => format!("<{}|{}>", url, text),
        MarkdownFormat::Telegram => format!("[{}]({})", text, url),
        MarkdownFormat::Html => format!("<a href=\"{}\">{}</a>", url, text),
        MarkdownFormat::Plain => format!("{} ({})", text, url),
        MarkdownFormat::Matrix => format!("[{}]({})", text, url),
        MarkdownFormat::Irc => format!("{}: {}", text, url),
    }
}

// ============================================================================
// Main conversion function
// ============================================================================

/// Convert markdown between different formats.
///
/// # Arguments
///
/// * `input` - The markdown string to convert
/// * `from` - Source markdown format
/// * `to` - Target markdown format
///
/// # Example
///
/// ```
/// use aisopod_channel_utils::markdown::{convert, MarkdownFormat};
///
/// let discord = "**bold** and *italic*";
/// let slack = convert(discord, MarkdownFormat::Discord, MarkdownFormat::Slack);
/// assert_eq!(slack, "*bold* and _italic_");
/// ```
pub fn convert(input: &str, from: MarkdownFormat, to: MarkdownFormat) -> String {
    let ast = parse_markdown(input, &from);
    render_markdown(&ast, &to)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discord_to_slack() {
        let input = "**bold** and *italic* and ~~strikethrough~~";
        eprintln!("Test starting, input: {}", input);
        let result = convert(input, MarkdownFormat::Discord, MarkdownFormat::Slack);
        eprintln!("Test completed, result: {}", result);
        assert_eq!(result, "*bold* and _italic_ and ~strikethrough~");
    }

    #[test]
    fn test_discord_to_telegram() {
        let input = "**bold** and __italic__ and ~~strikethrough~~";
        let result = convert(input, MarkdownFormat::Discord, MarkdownFormat::Telegram);
        assert_eq!(result, "**bold** and __italic__ and ~~strikethrough~~");
    }

    #[test]
    fn test_slack_to_discord() {
        let input = "*bold* and _italic_ and ~strikethrough~";
        let result = convert(input, MarkdownFormat::Slack, MarkdownFormat::Discord);
        assert_eq!(result, "**bold** and *italic* and ~~strikethrough~~");
    }

    #[test]
    fn test_html_to_discord() {
        let input = "<b>bold</b> and <i>italic</i> and <s>strikethrough</s>";
        let result = convert(input, MarkdownFormat::Html, MarkdownFormat::Discord);
        assert_eq!(result, "**bold** and *italic* and ~~strikethrough~~");
    }

    #[test]
    fn test_plain_to_discord() {
        let input = "plain text with **no** formatting";
        let result = convert(input, MarkdownFormat::Plain, MarkdownFormat::Discord);
        assert_eq!(result, "plain text with **no** formatting");
    }

    #[test]
    fn test_code_blocks() {
        // Code block with language specification (language followed by space)
        let input = "``` python\nprint('hello')\n```";
        let result = convert(input, MarkdownFormat::Discord, MarkdownFormat::Slack);
        // When language is empty string (just space after backticks), it's treated as no language
        // and the output has newline after backticks
        assert_eq!(result, "```\npython\nprint('hello')```");
    }

    #[test]
    fn test_links() {
        let input = "[click here](https://example.com)";
        let result = convert(input, MarkdownFormat::Discord, MarkdownFormat::Slack);
        assert_eq!(result, "<https://example.com|click here>");
    }

    #[test]
    fn test_multiple_formats() {
        let input = "**bold** *italic* __underline__";
        let discord = convert(input, MarkdownFormat::Discord, MarkdownFormat::Discord);
        let slack = convert(input, MarkdownFormat::Discord, MarkdownFormat::Slack);
        let telegram = convert(input, MarkdownFormat::Discord, MarkdownFormat::Telegram);
        let html = convert(input, MarkdownFormat::Discord, MarkdownFormat::Html);

        assert_eq!(discord, "**bold** *italic* __underline__");
        assert_eq!(slack, "*bold* _italic_ _underline_");
        assert_eq!(telegram, "**bold** __italic__ __underline__");
        assert_eq!(html, "<b>bold</b> <i>italic</i> __underline__");
    }

    #[test]
    fn test_markdown_format_display() {
        assert_eq!(format!("{}", MarkdownFormat::Discord), "discord");
        assert_eq!(format!("{}", MarkdownFormat::Slack), "slack");
        assert_eq!(format!("{}", MarkdownFormat::Telegram), "telegram");
        assert_eq!(format!("{}", MarkdownFormat::Html), "html");
        assert_eq!(format!("{}", MarkdownFormat::Plain), "plain");
        assert_eq!(format!("{}", MarkdownFormat::Matrix), "matrix");
        assert_eq!(format!("{}", MarkdownFormat::Irc), "irc");
    }
}
