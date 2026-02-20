//! Tool registry tests

use std::sync::Arc;

use aisopod_tools::{Tool, ToolContext, ToolResult, ToolRegistry};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;

struct TestTool {
    name: String,
    description: String,
}

impl TestTool {
    fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
        }
    }
}

#[async_trait]
impl Tool for TestTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {},
            "required": []
        })
    }

    async fn execute(
        &self,
        _params: serde_json::Value,
        _ctx: &ToolContext,
    ) -> Result<ToolResult> {
        Ok(ToolResult::success(format!("Executed {}", self.name)))
    }
}

#[tokio::test]
async fn test_new_registry_is_empty() {
    let registry = ToolRegistry::new();
    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);
}

#[tokio::test]
async fn test_register_tool() {
    let mut registry = ToolRegistry::new();
    let tool = Arc::new(TestTool::new("test_tool", "A test tool"));
    
    registry.register(tool);
    
    assert_eq!(registry.len(), 1);
    assert!(registry.get("test_tool").is_some());
}

#[tokio::test]
async fn test_get_tool() {
    let mut registry = ToolRegistry::new();
    let tool = Arc::new(TestTool::new("my_tool", "My tool"));
    registry.register(tool);
    
    let retrieved = registry.get("my_tool");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name(), "my_tool");
}

#[tokio::test]
async fn test_get_nonexistent_tool() {
    let registry = ToolRegistry::new();
    assert!(registry.get("nonexistent").is_none());
}

#[tokio::test]
async fn test_list_tools() {
    let mut registry = ToolRegistry::new();
    
    registry.register(Arc::new(TestTool::new("tool_a", "Tool A")));
    registry.register(Arc::new(TestTool::new("tool_b", "Tool B")));
    
    let tools = registry.list();
    assert_eq!(tools.len(), 2);
    assert!(tools.contains(&"tool_a".to_string()));
    assert!(tools.contains(&"tool_b".to_string()));
}

#[tokio::test]
async fn test_schemas() {
    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(TestTool::new("my_tool", "My description")));
    
    let schemas = registry.schemas();
    assert_eq!(schemas.len(), 1);
    
    let schema = &schemas[0];
    assert_eq!(schema["name"], "my_tool");
    assert_eq!(schema["description"], "My description");
    assert_eq!(schema["parameters"], json!({
        "type": "object",
        "properties": {},
        "required": []
    }));
}

#[tokio::test]
async fn test_remove_tool() {
    let mut registry = ToolRegistry::new();
    let tool = Arc::new(TestTool::new("removable", "A tool to remove"));
    registry.register(tool);
    
    assert!(registry.remove("removable"));
    assert!(registry.get("removable").is_none());
    assert!(registry.is_empty());
}

#[tokio::test]
async fn test_remove_nonexistent_tool() {
    let mut registry = ToolRegistry::new();
    assert!(!registry.remove("nonexistent"));
}

#[tokio::test]
async fn test_duplicate_registration_overwrites_with_warning() {
    let mut registry = ToolRegistry::new();
    
    let tool1 = Arc::new(TestTool::new("duplicate", "First tool"));
    let tool2 = Arc::new(TestTool::new("duplicate", "Second tool"));
    
    registry.register(tool1);
    registry.register(tool2); // Should overwrite with warning
    
    let retrieved = registry.get("duplicate");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().description(), "Second tool");
    assert_eq!(registry.len(), 1);
}

#[tokio::test]
async fn test_register_all_builtins() {
    let mut registry = ToolRegistry::new();
    aisopod_tools::register_all_tools(&mut registry);
    
    let tools = registry.list();
    assert!(!tools.is_empty());
    
    // Verify all expected tools are registered
    assert!(tools.contains(&"bash".to_string()));
    assert!(tools.contains(&"canvas".to_string()));
    assert!(tools.contains(&"cron".to_string()));
    assert!(tools.contains(&"file".to_string()));
    assert!(tools.contains(&"message".to_string()));
    assert!(tools.contains(&"subagent".to_string()));
    assert!(tools.contains(&"session".to_string()));
}

#[tokio::test]
async fn test_register_and_remove_all() {
    let mut registry = ToolRegistry::new();
    aisopod_tools::register_all_tools(&mut registry);
    
    let tools = registry.list();
    let original_len = tools.len();
    
    // Remove all tools
    for tool_name in tools {
        registry.remove(&tool_name);
    }
    
    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);
}
