//! Prompt-related tests for agent engine.
//!
//! This module tests system prompt construction with multiple sections.

use aisopod_agent::prompt::{PromptSection, SystemPromptBuilder};
use aisopod_agent::types::ToolSchema;

#[test]
fn test_prompt_section_new() {
    let section = PromptSection::new("Test", "Content");
    assert_eq!(section.label, "Test");
    assert_eq!(section.content, "Content");
}

#[test]
fn test_system_prompt_builder_new() {
    let builder = SystemPromptBuilder::new();
    // sections field is private, just test that we can create a builder
    let prompt = builder.build();
    assert!(prompt.is_empty());
}

#[test]
fn test_system_prompt_builder_with_base_prompt() {
    let builder = SystemPromptBuilder::new()
        .with_base_prompt("You are a helpful assistant.");
    
    let prompt = builder.build();
    assert!(prompt.contains("## Base Prompt"));
    assert!(prompt.contains("You are a helpful assistant."));
}

#[test]
fn test_system_prompt_builder_build() {
    let builder = SystemPromptBuilder::new()
        .with_base_prompt("Base content");
    
    let prompt = builder.build();
    assert!(prompt.contains("## Base Prompt"));
    assert!(prompt.contains("Base content"));
}

#[test]
fn test_system_prompt_builder_with_all_sections() {
    let builder = SystemPromptBuilder::new()
        .with_base_prompt("You are a helpful assistant.")
        .with_dynamic_context();

    let prompt = builder.build();
    
    assert!(prompt.contains("## Base Prompt"));
    assert!(prompt.contains("You are a helpful assistant."));
    assert!(prompt.contains("## Dynamic Context"));
    assert!(prompt.contains("Current UTC timestamp:"));
    assert!(prompt.contains("Workspace path:"));
}

#[test]
fn test_system_prompt_builder_empty_tool_descriptions() {
    let tools: Vec<ToolSchema> = vec![];
    
    let builder = SystemPromptBuilder::new()
        .with_base_prompt("Base")
        .with_tool_descriptions(&tools);
    
    let prompt = builder.build();
    
    assert!(prompt.contains("## Tools"));
    assert!(prompt.contains("No tools available."));
}

#[test]
fn test_system_prompt_builder_with_tool_descriptions() {
    let tools = vec![
        ToolSchema::new(
            "calculator",
            "A calculator tool",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "operation": {"type": "string"}
                }
            })
        ),
    ];
    
    let builder = SystemPromptBuilder::new()
        .with_base_prompt("Base")
        .with_tool_descriptions(&tools);
    
    let prompt = builder.build();
    
    assert!(prompt.contains("## Tools"));
    assert!(prompt.contains("## calculator"));
    assert!(prompt.contains("A calculator tool"));
    assert!(prompt.contains("operation"));
}

#[test]
fn test_system_prompt_builder_with_skill_instructions() {
    let skills = vec![
        "coding".to_string(),
        "analysis".to_string(),
    ];
    
    let builder = SystemPromptBuilder::new()
        .with_base_prompt("Base")
        .with_skill_instructions(&skills);
    
    let prompt = builder.build();
    
    assert!(prompt.contains("## Skills"));
    assert!(prompt.contains("## coding"));
    assert!(prompt.contains("## analysis"));
}

#[test]
fn test_system_prompt_builder_with_memory_context() {
    let memory = "User prefers Python";
    
    let builder = SystemPromptBuilder::new()
        .with_base_prompt("Base")
        .with_memory_context(memory);
    
    let prompt = builder.build();
    
    assert!(prompt.contains("## Memory Context"));
    assert!(prompt.contains("User prefers Python"));
}

#[test]
fn test_system_prompt_builder_empty_memory_context() {
    let memory = "";
    
    let builder = SystemPromptBuilder::new()
        .with_base_prompt("Base")
        .with_memory_context(memory);
    
    let prompt = builder.build();
    
    // Memory section should not be added if empty
    assert!(!prompt.contains("## Memory Context"));
}

#[test]
fn test_system_prompt_builder_section_order() {
    let builder = SystemPromptBuilder::new()
        .with_base_prompt("Base")
        .with_dynamic_context()
        .with_tool_descriptions(&[])
        .with_skill_instructions(&[])
        .with_memory_context("Memory");
    
    let prompt = builder.build();
    
    // Check order
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

#[test]
fn test_system_prompt_builder_multiple_sections_concatenated() {
    let builder = SystemPromptBuilder::new()
        .with_base_prompt("First")
        .with_base_prompt("Second");
    
    let prompt = builder.build();
    
    // Both sections should be present
    assert!(prompt.contains("First"));
    assert!(prompt.contains("Second"));
    assert!(prompt.matches("## Base Prompt").count() == 2);
}
