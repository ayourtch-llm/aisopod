//! Tool Registry for managing registered tools.
//!
//! The `ToolRegistry` provides a central mechanism for registering, looking up,
//! and enumerating tools. It stores tools as `Arc<dyn Tool>` keyed by name,
//! supporting both built-in tools and plugin-provided tools.
//!
//! # Example
//!
//! ```ignore
//! use std::sync::Arc;
//! use aisopod_tools::{Tool, ToolRegistry};
//!
//! let mut registry = ToolRegistry::new();
//!
//! // Register a tool
//! let my_tool: Arc<dyn Tool> = Arc::new(MyTool::new());
//! registry.register(my_tool);
//!
//! // Look up a tool
//! if let Some(tool) = registry.get("my_tool") {
//!     println!("Found tool: {}", tool.name());
//! }
//!
//! // List all registered tools
//! for name in registry.list() {
//!     println!("Registered tool: {}", name);
//! }
//! ```

use std::collections::HashMap;
use std::sync::Arc;

use crate::Tool;
use tracing::warn;

/// A registry that stores and manages tools by name.
///
/// The registry holds tools as `Arc<dyn Tool>` and provides methods for
/// registering, looking up, and listing registered tools. It also supports
/// generating JSON Schema definitions for AI model function calling.
///
/// # Tool Registration
///
/// Tools are registered by name, and duplicate registrations are handled by
/// logging a warning and overwriting the existing tool. This allows runtime
/// reconfiguration while providing visibility into overwrites.
///
/// # Thread Safety
///
/// The `ToolRegistry` is designed to be `Send` and `Sync` when all contained
/// tools are `Send` and `Sync`, as it uses `Arc<dyn Tool>` internally.
///
/// # Example
///
/// ```ignore
/// use std::sync::Arc;
/// use aisopod_tools::{Tool, ToolRegistry};
///
/// let mut registry = ToolRegistry::new();
/// let tool: Arc<dyn Tool> = Arc::new(MyTool);
/// registry.register(tool);
///
/// // Get a tool by name
/// if let Some(retrieved) = registry.get("my_tool") {
///     let result = retrieved.execute(json!({}), &ctx).await?;
/// }
///
/// // Generate schemas for AI function calling
/// let schemas = registry.schemas();
/// ```
#[derive(Default)]
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    /// Creates a new empty `ToolRegistry`.
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Registers a tool with the registry.
    ///
    /// Extracts the tool's name using `Tool::name()` and stores the tool
    /// in the registry. If a tool with the same name already exists, logs
    /// a warning and overwrites the existing tool.
    ///
    /// # Arguments
    ///
    /// * `tool` - An `Arc` wrapped tool that implements the `Tool` trait.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut registry = ToolRegistry::new();
    /// let tool: Arc<dyn Tool> = Arc::new(MyTool);
    /// registry.register(tool);
    /// ```
    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        let name = tool.name().to_string();
        
        if self.tools.contains_key(&name) {
            warn!(
                "Tool '{}' is already registered. Overwriting existing registration.",
                name
            );
        }
        
        self.tools.insert(name, tool);
    }

    /// Retrieves a tool by name.
    ///
    /// Returns an `Arc` to the tool if it is registered, or `None` if
    /// no tool with the given name exists.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the tool to retrieve.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// if let Some(tool) = registry.get("my_tool") {
    ///     // Use the tool
    /// }
    /// ```
    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }

    /// Returns a list of all registered tool names.
    ///
    /// The returned vector contains the names of all tools currently
    /// registered in this registry.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// for name in registry.list() {
    ///     println!("Registered: {}", name);
    /// }
    /// ```
    pub fn list(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }

    /// Generates JSON Schema definitions for all registered tools.
    ///
    /// Returns a vector of `serde_json::Value` where each entry represents
    /// a tool in the format suitable for AI model function calling:
    ///
    /// ```json
    /// {
    ///   "name": "<tool name>",
    ///   "description": "<tool description>",
    ///   "parameters": <parameters_schema>
    /// }
    /// ```
    ///
    /// This provides the generic internal format. Provider-specific
    /// conversion (e.g., to OpenAI or Anthropic formats) happens in
    /// later issues.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let schemas = registry.schemas();
    /// for schema in schemas {
    ///     println!("Tool schema: {}", schema);
    /// }
    /// ```
    pub fn schemas(&self) -> Vec<serde_json::Value> {
        self.tools
            .values()
            .map(|tool| {
                serde_json::json!({
                    "name": tool.name(),
                    "description": tool.description(),
                    "parameters": tool.parameters_schema()
                })
            })
            .collect()
    }

    /// Removes a tool from the registry by name.
    ///
    /// Returns `true` if a tool with the given name was present and removed,
    /// or `false` if no such tool exists.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the tool to remove.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// if registry.remove("my_tool") {
    ///     println!("Tool removed successfully");
    /// }
    /// ```
    pub fn remove(&mut self, name: &str) -> bool {
        self.tools.remove(name).is_some()
    }

    /// Returns the number of tools currently registered.
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// Returns `true` if the registry contains no tools.
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Tool, ToolContext, ToolResult};
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

    #[test]
    fn test_new_registry_is_empty() {
        let registry = ToolRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_register_tool() {
        let mut registry = ToolRegistry::new();
        let tool = Arc::new(TestTool::new("test_tool", "A test tool"));
        
        registry.register(tool);
        
        assert_eq!(registry.len(), 1);
        assert!(registry.get("test_tool").is_some());
    }

    #[test]
    fn test_get_tool() {
        let mut registry = ToolRegistry::new();
        let tool = Arc::new(TestTool::new("my_tool", "My tool"));
        registry.register(tool);
        
        let retrieved = registry.get("my_tool");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name(), "my_tool");
    }

    #[test]
    fn test_get_nonexistent_tool() {
        let registry = ToolRegistry::new();
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_list_tools() {
        let mut registry = ToolRegistry::new();
        
        registry.register(Arc::new(TestTool::new("tool_a", "Tool A")));
        registry.register(Arc::new(TestTool::new("tool_b", "Tool B")));
        
        let tools = registry.list();
        assert_eq!(tools.len(), 2);
        assert!(tools.contains(&"tool_a".to_string()));
        assert!(tools.contains(&"tool_b".to_string()));
    }

    #[test]
    fn test_schemas() {
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

    #[test]
    fn test_remove_tool() {
        let mut registry = ToolRegistry::new();
        let tool = Arc::new(TestTool::new("removable", "A tool to remove"));
        registry.register(tool);
        
        assert!(registry.remove("removable"));
        assert!(registry.get("removable").is_none());
        assert!(registry.is_empty());
    }

    #[test]
    fn test_remove_nonexistent_tool() {
        let mut registry = ToolRegistry::new();
        assert!(!registry.remove("nonexistent"));
    }

    #[test]
    fn test_duplicate_registration_overwrites_with_warning() {
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
}
