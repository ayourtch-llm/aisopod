//! Output formatting utilities for the aisopod application.
//!
//! This module provides colored terminal output, table formatting for list commands,
//! JSON output mode, and progress indicators.

use colored::Colorize;
use comfy_table::{presets::UTF8_FULL, Table};
use indicatif::{ProgressBar, ProgressStyle};

/// Output formatting options
pub struct Output {
    json_mode: bool,
}

impl Output {
    /// Create a new Output formatter
    pub fn new(json_mode: bool) -> Self {
        Self { json_mode }
    }

    /// Check if output is a TTY (for color support)
    pub fn is_tty() -> bool {
        atty::is(atty::Stream::Stdout)
    }

    /// Print a table with headers and rows
    pub fn print_table(&self, headers: &[&str], rows: Vec<Vec<String>>) {
        if self.json_mode {
            self.print_table_json(headers, rows);
            return;
        }

        let mut table = Table::new();
        table.load_preset(UTF8_FULL);
        table.set_header(headers.iter().map(|h| *h).collect::<Vec<&str>>());
        for row in rows {
            table.add_row(row);
        }
        println!("{table}");
    }

    /// Print table in JSON format
    fn print_table_json(&self, headers: &[&str], rows: Vec<Vec<String>>) {
        let json_rows: Vec<serde_json::Value> = rows
            .iter()
            .map(|row| {
                serde_json::json!({
                    "headers": headers,
                    "values": row
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&json_rows).unwrap());
    }

    /// Print a success message
    pub fn success(&self, msg: &str) {
        if self.json_mode {
            println!(
                r#"{{"status":"success","message":"{}"}}"#,
                escape_json_string(msg)
            );
        } else if Output::is_tty() {
            println!("{} {}", "✓".green(), msg);
        } else {
            println!("[SUCCESS] {}", msg);
        }
    }

    /// Print an error message
    pub fn error(&self, msg: &str) {
        if self.json_mode {
            eprintln!(
                r#"{{"status":"error","message":"{}"}}"#,
                escape_json_string(msg)
            );
        } else if Output::is_tty() {
            eprintln!("{} {}", "✗".red(), msg);
        } else {
            eprintln!("[ERROR] {}", msg);
        }
    }

    /// Print an info message
    pub fn info(&self, msg: &str) {
        if self.json_mode {
            println!(
                r#"{{"status":"info","message":"{}"}}"#,
                escape_json_string(msg)
            );
        } else if Output::is_tty() {
            println!("{} {}", "ℹ".blue(), msg);
        } else {
            println!("[INFO] {}", msg);
        }
    }

    /// Print a warning message
    pub fn warning(&self, msg: &str) {
        if self.json_mode {
            println!(
                r#"{{"status":"warning","message":"{}"}}"#,
                escape_json_string(msg)
            );
        } else if Output::is_tty() {
            println!("{} {}", "⚠".yellow(), msg);
        } else {
            println!("[WARNING] {}", msg);
        }
    }
}

/// Escape a string for JSON output
fn escape_json_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Create a spinner progress bar
pub fn create_spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    pb
}

/// Create a progress bar with a known total
pub fn create_progress_bar(total: u64, message: &str) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("█▉▊▋▌▍▎▏ "),
    );
    pb.set_message(message.to_string());
    pb
}

/// Format a string with the specified color if output is a TTY
pub fn color_str(msg: &str, color: colored::Color) -> String {
    if Output::is_tty() {
        msg.color(color).to_string()
    } else {
        msg.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_new() {
        let output = Output::new(false);
        assert!(!output.json_mode);
    }

    #[test]
    fn test_output_json_mode() {
        let output = Output::new(true);
        assert!(output.json_mode);
    }

    #[test]
    fn test_escape_json_string() {
        assert_eq!(escape_json_string("hello"), "hello");
        assert_eq!(escape_json_string("hello\nworld"), "hello\\nworld");
        assert_eq!(escape_json_string("hello\"world"), "hello\\\"world");
        assert_eq!(escape_json_string("hello\\world"), "hello\\\\world");
    }
}
