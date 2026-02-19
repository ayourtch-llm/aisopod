//! System prompt construction for agent execution.
//!
//! This module provides a builder pattern for constructing system prompts
//! that include base instructions, dynamic context, tool descriptions,
//! skill instructions, and memory context.

use crate::types::ToolSchema;
use chrono::{DateTime, Utc};

/// A section of the system prompt with a label and content.
#[derive(Debug, Clone, Default)]
pub struct PromptSection {
    /// The label/header for this section (e.g., "Tools", "Skills").
    pub label: String,
    /// The content of this section.
    pub content: String,
}

impl PromptSection {
    /// Creates a new prompt section with the given label and content.
    pub fn new(label: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            content: content.into(),
        }
    }
}

/// Builder for constructing system prompts with multiple sections.
///
/// The builder allows incrementally adding sections to the system prompt
/// and then generating the complete prompt via the `build()` method.
///
/// # Example
///
/// ```ignore
/// let builder = SystemPromptBuilder::new();
/// let prompt = builder
///     .with_base_prompt("You are a helpful assistant.")
///     .with_dynamic_context()
///     .with_tool_descriptions(&tools)
///     .build();
/// ```
#[derive(Debug, Clone, Default)]
pub struct SystemPromptBuilder {
    /// The list of sections to include in the prompt.
    sections: Vec<PromptSection>,
}

impl SystemPromptBuilder {
    /// Creates a new empty system prompt builder.
    pub fn new() -> Self {
        Self {
            sections: Vec::new(),
        }
    }

    /// Adds a base prompt to the system prompt.
    ///
    /// This should contain the primary instructions for the agent.
    pub fn with_base_prompt(mut self, prompt: &str) -> Self {
        self.sections.push(PromptSection::new("Base Prompt", prompt.to_string()));
        self
    }

    /// Adds dynamic context including current date/time and workspace info.
    ///
    /// The dynamic context includes:
    /// - Current UTC timestamp
    /// - Workspace path if available
    pub fn with_dynamic_context(mut self) -> Self {
        let now: DateTime<Utc> = Utc::now();
        let timestamp = now.format("%Y-%m-%d %H:%M:%S UTC").to_string();
        
        let workspace_info = Self::get_workspace_info();
        
        let content = format!(
            "Current UTC timestamp: {}\n\n{}",
            timestamp,
            workspace_info
        );
        
        self.sections.push(PromptSection::new("Dynamic Context", content));
        self
    }

    /// Retrieves workspace information.
    fn get_workspace_info() -> String {
        // Try to get the current directory as workspace path
        if let Ok(cwd) = std::env::current_dir() {
            format!("Workspace path: {}", cwd.display())
        } else {
            "Workspace path: unavailable".to_string()
        }
    }

    /// Adds formatted tool descriptions to the system prompt.
    ///
    /// Each tool description includes the tool name, description, and parameter schema.
    pub fn with_tool_descriptions(mut self, tools: &[ToolSchema]) -> Self {
        if tools.is_empty() {
            self.sections.push(PromptSection::new("Tools", "No tools available.".to_string()));
            return self;
        }

        let mut content = String::new();
        content.push_str("Available tools:\n\n");

        for tool in tools {
            content.push_str(&format!(
                "## {}\n{}\n\nParameters:\n```json\n{}\n```\n\n",
                tool.name,
                tool.description,
                serde_json::to_string_pretty(&tool.parameters).unwrap_or_else(|_| "Invalid schema".to_string())
            ));
        }

        self.sections.push(PromptSection::new("Tools", content));
        self
    }

    /// Appends skill instruction blocks to the system prompt.
    ///
    /// Each skill is formatted as a distinct section with the skill name as header.
    pub fn with_skill_instructions(mut self, skills: &[String]) -> Self {
        let mut content = String::new();
        content.push_str("Skill instructions:\n\n");

        for skill in skills {
            content.push_str(&format!("## {}\n{}\n\n", skill, "Skill-specific instructions would go here."));
        }

        self.sections.push(PromptSection::new("Skills", content));
        self
    }

    /// Adds retrieved memory context to the system prompt.
    pub fn with_memory_context(mut self, memory: &str) -> Self {
        if memory.is_empty() {
            return self;
        }

        let content = format!(
            "Relevant memory context:\n\n{}",
            memory
        );

        self.sections.push(PromptSection::new("Memory Context", content));
        self
    }

    /// Builds the complete system prompt by concatenating all sections.
    ///
    /// Each section is separated by a blank line and includes its header.
    pub fn build(&self) -> String {
        let mut result = String::new();

        for (i, section) in self.sections.iter().enumerate() {
            if i > 0 {
                result.push('\n');
            }
            result.push_str(&format!("## {}\n{}", section.label, section.content));
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ToolSchema;

    #[test]
    fn test_new_builder_is_empty() {
        let builder = SystemPromptBuilder::new();
        assert!(builder.sections.is_empty());
    }

    #[test]
    fn test_with_base_prompt() {
        let builder = SystemPromptBuilder::new()
            .with_base_prompt("You are a helpful assistant.");
        
        assert_eq!(builder.sections.len(), 1);
        assert_eq!(builder.sections[0].label, "Base Prompt");
        assert_eq!(builder.sections[0].content, "You are a helpful assistant.");
    }

    #[test]
    fn test_build_with_only_base_prompt() {
        let builder = SystemPromptBuilder::new()
            .with_base_prompt("You are a helpful assistant.");
        
        let prompt = builder.build();
        assert!(prompt.contains("## Base Prompt"));
        assert!(prompt.contains("You are a helpful assistant."));
    }

    #[test]
    fn test_build_with_all_sections() {
        let tools = vec![
            ToolSchema {
                name: "calculator".to_string(),
                description: "A calculator tool".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "operation": {"type": "string"},
                        "value": {"type": "number"}
                    }
                }),
            },
            ToolSchema {
                name: "file_reader".to_string(),
                description: "Reads a file".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string"}
                    }
                }),
            },
        ];

        let skills = vec![
            "coding".to_string(),
            "analysis".to_string(),
        ];

        let memory = "User prefers Python over JavaScript.";

        let builder = SystemPromptBuilder::new()
            .with_base_prompt("You are a helpful assistant.")
            .with_dynamic_context()
            .with_tool_descriptions(&tools)
            .with_skill_instructions(&skills)
            .with_memory_context(memory);

        let prompt = builder.build();

        // Check all sections are present
        assert!(prompt.contains("## Base Prompt"));
        assert!(prompt.contains("You are a helpful assistant."));
        assert!(prompt.contains("## Dynamic Context"));
        assert!(prompt.contains("## Tools"));
        assert!(prompt.contains("## calculator"));
        assert!(prompt.contains("A calculator tool"));
        assert!(prompt.contains("## file_reader"));
        assert!(prompt.contains("Reads a file"));
        assert!(prompt.contains("## Skills"));
        assert!(prompt.contains("## coding"));
        assert!(prompt.contains("## analysis"));
        assert!(prompt.contains("## Memory Context"));
        assert!(prompt.contains("User prefers Python over JavaScript."));
    }

    #[test]
    fn test_tool_descriptions_formatting() {
        let tools = vec![
            ToolSchema {
                name: "test_tool".to_string(),
                description: "A test tool".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "param1": {"type": "string"},
                        "param2": {"type": "number"}
                    },
                    "required": ["param1"]
                }),
            },
        ];

        let builder = SystemPromptBuilder::new()
            .with_tool_descriptions(&tools);

        let prompt = builder.build();

        // Check tool name is present
        assert!(prompt.contains("## test_tool"));
        // Check description is present
        assert!(prompt.contains("A test tool"));
        // Check parameter schema is present in JSON format
        assert!(prompt.contains("param1"));
        assert!(prompt.contains("param2"));
        // Check JSON formatting
        assert!(prompt.contains("```json"));
    }

    #[test]
    fn test_empty_tool_descriptions() {
        let tools: Vec<ToolSchema> = vec![];

        let builder = SystemPromptBuilder::new()
            .with_tool_descriptions(&tools);

        let prompt = builder.build();

        assert!(prompt.contains("## Tools"));
        assert!(prompt.contains("No tools available."));
    }

    #[test]
    fn test_dynamic_context_timestamp() {
        let builder = SystemPromptBuilder::new()
            .with_dynamic_context();

        let prompt = builder.build();

        assert!(prompt.contains("## Dynamic Context"));
        assert!(prompt.contains("Current UTC timestamp:"));
        // Check timestamp format (YYYY-MM-DD HH:MM:SS UTC)
        let lines: Vec<&str> = prompt.lines().collect();
        let timestamp_line = lines.iter().find(|line| line.contains("Current UTC timestamp:"));
        assert!(timestamp_line.is_some());
        
        let timestamp = timestamp_line.unwrap();
        assert!(timestamp.contains("UTC"));
    }

    #[test]
    fn test_workspace_info_in_dynamic_context() {
        let builder = SystemPromptBuilder::new()
            .with_dynamic_context();

        let prompt = builder.build();

        assert!(prompt.contains("## Dynamic Context"));
        // Should contain workspace path info
        assert!(prompt.contains("Workspace path:"));
    }

    #[test]
    fn test_empty_skill_instructions() {
        let skills: Vec<String> = vec![];

        let builder = SystemPromptBuilder::new()
            .with_skill_instructions(&skills);

        let prompt = builder.build();

        // Skills section should still be present but empty or minimal
        assert!(prompt.contains("## Skills"));
    }

    #[test]
    fn test_empty_memory_context() {
        let memory = "";

        let builder = SystemPromptBuilder::new()
            .with_memory_context(memory);

        let prompt = builder.build();

        // Memory section should not be added if empty
        assert!(!prompt.contains("## Memory Context"));
    }

    #[test]
    fn test_multiple_sections_order() {
        let builder = SystemPromptBuilder::new()
            .with_base_prompt("Base")
            .with_dynamic_context()
            .with_tool_descriptions(&[])
            .with_skill_instructions(&[])
            .with_memory_context("Memory");

        let prompt = builder.build();

        // Check order of sections
        let base_pos = prompt.find("## Base Prompt").unwrap();
        let dynamic_pos = prompt.find("## Dynamic Context").unwrap();
        let tools_pos = prompt.find("## Tools").unwrap();
        let skills_pos = prompt.find("## Skills").unwrap();
        let memory_pos = prompt.find("## Memory Context").unwrap();

        assert!(base_pos < dynamic_pos);
        assert!(dynamic_pos < tools_pos);
        assert!(tools_pos < skills_pos);
        assert!(skills_pos < memory_pos);
    }
}
