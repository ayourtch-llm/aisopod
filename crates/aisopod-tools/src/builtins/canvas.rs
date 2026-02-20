//! Built-in canvas tool for generating visual HTML/CSS/JS output.

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use dashmap::DashMap;
use serde_json::{json, Value};

use crate::{Tool, ToolContext, ToolResult};

/// Trait for canvas renderer implementations.
///
/// This trait defines the interface for canvas rendering backends.
/// Implementations can store canvas data in memory, database, or other storage.
#[async_trait]
pub trait CanvasRenderer: Send + Sync {
    /// Creates a new canvas with the given ID and content.
    ///
    /// # Arguments
    ///
    /// * `canvas_id` - The unique identifier for the canvas
    /// * `content` - The HTML/CSS/JS content for the canvas
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if the canvas was created successfully.
    async fn create(&self, canvas_id: &str, content: &str) -> Result<()>;

    /// Updates an existing canvas with new content.
    ///
    /// # Arguments
    ///
    /// * `canvas_id` - The unique identifier for the canvas
    /// * `content` - The new HTML/CSS/JS content for the canvas
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if the canvas was updated successfully.
    async fn update(&self, canvas_id: &str, content: &str) -> Result<()>;

    /// Gets the content of a canvas by ID.
    ///
    /// # Arguments
    ///
    /// * `canvas_id` - The unique identifier for the canvas
    ///
    /// # Returns
    ///
    /// Returns the canvas content as a String if found, or an error.
    async fn get(&self, canvas_id: &str) -> Result<String>;
}

/// In-memory canvas renderer implementation using DashMap.
///
/// This implementation stores canvas data in memory using a thread-safe
/// HashMap. It's useful for testing and scenarios where persistence
/// is not required.
#[derive(Clone, Default)]
pub struct InMemoryCanvasRenderer {
    canvases: Arc<DashMap<String, String>>,
}

impl InMemoryCanvasRenderer {
    /// Creates a new InMemoryCanvasRenderer.
    pub fn new() -> Self {
        Self {
            canvases: Arc::new(DashMap::new()),
        }
    }

    /// Creates a new InMemoryCanvasRenderer with pre-populated data.
    pub fn with_canvases(canvases: HashMap<String, String>) -> Self {
        Self {
            canvases: Arc::new(DashMap::from_iter(canvases)),
        }
    }

    /// Returns the number of canvases stored.
    pub fn len(&self) -> usize {
        self.canvases.len()
    }

    /// Returns true if no canvases are stored.
    pub fn is_empty(&self) -> bool {
        self.canvases.is_empty()
    }
}

#[async_trait]
impl CanvasRenderer for InMemoryCanvasRenderer {
    async fn create(&self, canvas_id: &str, content: &str) -> Result<()> {
        if self.canvases.contains_key(canvas_id) {
            return Err(anyhow::anyhow!(
                "Canvas with ID '{}' already exists",
                canvas_id
            ));
        }
        self.canvases
            .insert(canvas_id.to_string(), content.to_string());
        Ok(())
    }

    async fn update(&self, canvas_id: &str, content: &str) -> Result<()> {
        if !self.canvases.contains_key(canvas_id) {
            return Err(anyhow::anyhow!("Canvas with ID '{}' not found", canvas_id));
        }
        self.canvases
            .insert(canvas_id.to_string(), content.to_string());
        Ok(())
    }

    async fn get(&self, canvas_id: &str) -> Result<String> {
        self.canvases
            .get(canvas_id)
            .map(|entry| entry.value().clone())
            .ok_or_else(|| anyhow::anyhow!("Canvas with ID '{}' not found", canvas_id))
    }
}

/// A built-in tool for generating and managing visual HTML/CSS/JS output.
///
/// This tool allows agents to create, update, and retrieve canvas content
/// for visual output. The actual storage is handled by an implementation
/// of the `CanvasRenderer` trait.
///
/// # Parameters
///
/// The tool accepts the following parameters:
///
/// - `operation`: The operation to perform (create/update/get)
/// - `canvas_id`: The unique identifier for the canvas
/// - `content`: The HTML/CSS/JS content (required for create/update)
///
/// # Example
///
/// ```json
/// {
///   "operation": "create",
///   "canvas_id": "my_visualization",
///   "content": "<html><body><canvas id='canvas'></canvas></body></html>"
/// }
/// ```
#[derive(Clone)]
pub struct CanvasTool {
    /// The canvas renderer implementation.
    renderer: Arc<dyn CanvasRenderer>,
}

impl CanvasTool {
    /// Creates a new CanvasTool with the given renderer.
    pub fn new(renderer: Arc<dyn CanvasRenderer>) -> Self {
        Self { renderer }
    }

    /// Creates a new CanvasTool with an in-memory renderer.
    pub fn with_in_memory() -> Self {
        Self::new(Arc::new(InMemoryCanvasRenderer::new()))
    }
}

#[async_trait]
impl Tool for CanvasTool {
    fn name(&self) -> &str {
        "canvas"
    }

    fn description(&self) -> &str {
        "Generate and manage visual HTML/CSS/JS output"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["create", "update", "get"],
                    "description": "The operation to perform: create, update, or get"
                },
                "canvas_id": {
                    "type": "string",
                    "description": "The unique identifier for the canvas"
                },
                "content": {
                    "type": "string",
                    "description": "The HTML/CSS/JS content (required for create/update operations)"
                }
            },
            "required": ["operation", "canvas_id"]
        })
    }

    async fn execute(&self, params: Value, _ctx: &ToolContext) -> Result<ToolResult> {
        // Extract operation parameter (required)
        let operation = params
            .get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter 'operation'"))?;

        // Extract canvas_id parameter (required)
        let canvas_id = params
            .get("canvas_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter 'canvas_id'"))?;

        // Dispatch based on operation
        match operation {
            "create" => {
                // Extract content parameter (required for create)
                let content = params
                    .get("content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        anyhow::anyhow!("Missing required parameter 'content' for create operation")
                    })?;

                match self.renderer.create(canvas_id, content).await {
                    Ok(()) => Ok(ToolResult::success(format!(
                        "Canvas '{}' created successfully",
                        canvas_id
                    ))),
                    Err(e) => Ok(ToolResult::error(e.to_string())),
                }
            }
            "update" => {
                // Extract content parameter (required for update)
                let content = params
                    .get("content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        anyhow::anyhow!("Missing required parameter 'content' for update operation")
                    })?;

                match self.renderer.update(canvas_id, content).await {
                    Ok(()) => Ok(ToolResult::success(format!(
                        "Canvas '{}' updated successfully",
                        canvas_id
                    ))),
                    Err(e) => Ok(ToolResult::error(e.to_string())),
                }
            }
            "get" => match self.renderer.get(canvas_id).await {
                Ok(content) => Ok(ToolResult::success(content)),
                Err(e) => Ok(ToolResult::error(e.to_string())),
            },
            _ => Ok(ToolResult::error(format!(
                "Invalid operation '{}'. Must be one of: create, update, get",
                operation
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_tool_name() {
        let tool = CanvasTool::with_in_memory();
        assert_eq!(tool.name(), "canvas");
    }

    #[test]
    fn test_canvas_tool_description() {
        let tool = CanvasTool::with_in_memory();
        assert_eq!(
            tool.description(),
            "Generate and manage visual HTML/CSS/JS output"
        );
    }

    #[test]
    fn test_canvas_tool_schema() {
        let tool = CanvasTool::with_in_memory();
        let schema = tool.parameters_schema();

        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["operation"].is_object());
        assert!(schema["properties"]["canvas_id"].is_object());
        assert!(schema["properties"]["content"].is_object());

        let operation_enum = schema["properties"]["operation"]["enum"]
            .as_array()
            .unwrap();
        assert!(operation_enum.contains(&json!("create")));
        assert!(operation_enum.contains(&json!("update")));
        assert!(operation_enum.contains(&json!("get")));

        let required = schema["required"].as_array().unwrap();
        assert!(required.contains(&json!("operation")));
        assert!(required.contains(&json!("canvas_id")));
    }

    #[tokio::test]
    async fn test_canvas_tool_create() {
        let tool = CanvasTool::with_in_memory();
        let ctx = ToolContext::new("test_agent", "test_session");

        let result = tool
            .execute(
                json!({
                    "operation": "create",
                    "canvas_id": "my_visualization",
                    "content": "<html><body><canvas id='canvas'></canvas></body></html>"
                }),
                &ctx,
            )
            .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.is_error);
        assert!(output
            .content
            .contains("Canvas 'my_visualization' created successfully"));
    }

    #[tokio::test]
    async fn test_canvas_tool_update() {
        let tool = CanvasTool::with_in_memory();
        let ctx = ToolContext::new("test_agent", "test_session");

        // First create a canvas
        tool.execute(
            json!({
                "operation": "create",
                "canvas_id": "my_visualization",
                "content": "initial content"
            }),
            &ctx,
        )
        .await
        .unwrap();

        // Then update it
        let result = tool
            .execute(
                json!({
                    "operation": "update",
                    "canvas_id": "my_visualization",
                    "content": "updated content"
                }),
                &ctx,
            )
            .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.is_error);
        assert!(output
            .content
            .contains("Canvas 'my_visualization' updated successfully"));
    }

    #[tokio::test]
    async fn test_canvas_tool_get() {
        let tool = CanvasTool::with_in_memory();
        let ctx = ToolContext::new("test_agent", "test_session");

        // First create a canvas
        tool.execute(
            json!({
                "operation": "create",
                "canvas_id": "my_visualization",
                "content": "<html><body>Test</body></html>"
            }),
            &ctx,
        )
        .await
        .unwrap();

        // Then get it
        let result = tool
            .execute(
                json!({
                    "operation": "get",
                    "canvas_id": "my_visualization"
                }),
                &ctx,
            )
            .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.is_error);
        assert!(output.content.contains("<html><body>Test</body></html>"));
    }

    #[tokio::test]
    async fn test_canvas_tool_get_nonexistent() {
        let tool = CanvasTool::with_in_memory();
        let ctx = ToolContext::new("test_agent", "test_session");

        let result = tool
            .execute(
                json!({
                    "operation": "get",
                    "canvas_id": "nonexistent"
                }),
                &ctx,
            )
            .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.is_error);
        assert!(output.content.contains("not found"));
    }

    #[tokio::test]
    async fn test_canvas_tool_create_duplicate() {
        let tool = CanvasTool::with_in_memory();
        let ctx = ToolContext::new("test_agent", "test_session");

        // First create
        tool.execute(
            json!({
                "operation": "create",
                "canvas_id": "my_visualization",
                "content": "content"
            }),
            &ctx,
        )
        .await
        .unwrap();

        // Try to create again
        let result = tool
            .execute(
                json!({
                    "operation": "create",
                    "canvas_id": "my_visualization",
                    "content": "new content"
                }),
                &ctx,
            )
            .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.is_error);
        assert!(output.content.contains("already exists"));
    }

    #[tokio::test]
    async fn test_canvas_tool_update_nonexistent() {
        let tool = CanvasTool::with_in_memory();
        let ctx = ToolContext::new("test_agent", "test_session");

        let result = tool
            .execute(
                json!({
                    "operation": "update",
                    "canvas_id": "nonexistent",
                    "content": "content"
                }),
                &ctx,
            )
            .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.is_error);
        assert!(output.content.contains("not found"));
    }

    #[tokio::test]
    async fn test_canvas_tool_missing_operation() {
        let tool = CanvasTool::with_in_memory();
        let ctx = ToolContext::new("test_agent", "test_session");

        let result = tool
            .execute(
                json!({
                    "canvas_id": "my_visualization"
                }),
                &ctx,
            )
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Missing required parameter 'operation'"));
    }

    #[tokio::test]
    async fn test_canvas_tool_missing_canvas_id() {
        let tool = CanvasTool::with_in_memory();
        let ctx = ToolContext::new("test_agent", "test_session");

        let result = tool
            .execute(
                json!({
                    "operation": "create"
                }),
                &ctx,
            )
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Missing required parameter 'canvas_id'"));
    }

    #[tokio::test]
    async fn test_canvas_tool_missing_content_for_create() {
        let tool = CanvasTool::with_in_memory();
        let ctx = ToolContext::new("test_agent", "test_session");

        let result = tool
            .execute(
                json!({
                    "operation": "create",
                    "canvas_id": "my_visualization"
                }),
                &ctx,
            )
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Missing required parameter 'content' for create operation"));
    }

    #[tokio::test]
    async fn test_canvas_tool_missing_content_for_update() {
        let tool = CanvasTool::with_in_memory();
        let ctx = ToolContext::new("test_agent", "test_session");

        let result = tool
            .execute(
                json!({
                    "operation": "update",
                    "canvas_id": "my_visualization"
                }),
                &ctx,
            )
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Missing required parameter 'content' for update operation"));
    }

    #[tokio::test]
    async fn test_canvas_tool_invalid_operation() {
        let tool = CanvasTool::with_in_memory();
        let ctx = ToolContext::new("test_agent", "test_session");

        let result = tool
            .execute(
                json!({
                    "operation": "invalid",
                    "canvas_id": "my_visualization"
                }),
                &ctx,
            )
            .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.is_error);
        assert!(output.content.contains("Invalid operation"));
    }

    #[tokio::test]
    async fn test_in_memory_renderer_create_and_get() {
        let renderer = InMemoryCanvasRenderer::new();

        renderer
            .create("test_canvas", "<html>Test</html>")
            .await
            .unwrap();

        let content = renderer.get("test_canvas").await.unwrap();
        assert_eq!(content, "<html>Test</html>");
    }

    #[tokio::test]
    async fn test_in_memory_renderer_update() {
        let renderer = InMemoryCanvasRenderer::new();

        renderer.create("test_canvas", "initial").await.unwrap();

        renderer.update("test_canvas", "updated").await.unwrap();

        let content = renderer.get("test_canvas").await.unwrap();
        assert_eq!(content, "updated");
    }

    #[tokio::test]
    async fn test_in_memory_renderer_get_nonexistent() {
        let renderer = InMemoryCanvasRenderer::new();

        let result = renderer.get("nonexistent").await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_in_memory_renderer_len() {
        let renderer = InMemoryCanvasRenderer::new();

        assert_eq!(renderer.len(), 0);

        renderer.create("canvas1", "content1").await.unwrap();

        renderer.create("canvas2", "content2").await.unwrap();

        assert_eq!(renderer.len(), 2);
    }

    #[tokio::test]
    async fn test_in_memory_renderer_is_empty() {
        let renderer = InMemoryCanvasRenderer::new();

        assert!(renderer.is_empty());

        renderer.create("canvas1", "content1").await.unwrap();

        assert!(!renderer.is_empty());
    }

    #[tokio::test]
    async fn test_in_memory_renderer_with_canvases() {
        let mut canvases = std::collections::HashMap::new();
        canvases.insert("canvas1".to_string(), "content1".to_string());
        canvases.insert("canvas2".to_string(), "content2".to_string());

        let renderer = InMemoryCanvasRenderer::with_canvases(canvases);

        assert_eq!(renderer.len(), 2);

        let content = renderer.get("canvas1").await.unwrap();
        assert_eq!(content, "content1");
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let renderer = Arc::new(InMemoryCanvasRenderer::new());
        let mut handles = vec![];

        for i in 0..10 {
            let renderer_clone = Arc::clone(&renderer);
            let handle = tokio::spawn(async move {
                renderer_clone
                    .create(&format!("canvas_{}", i), &format!("content_{}", i))
                    .await
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap().unwrap();
        }

        assert_eq!(renderer.len(), 10);
    }
}
