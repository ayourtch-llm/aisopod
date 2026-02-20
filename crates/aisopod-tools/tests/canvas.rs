//! Canvas tool tests

use std::collections::HashMap;
use std::sync::Arc;

use aisopod_tools::{
    CanvasRenderer, CanvasTool, InMemoryCanvasRenderer, Tool, ToolContext, ToolResult,
};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;

// Mock canvas renderer for testing
#[derive(Clone)]
struct MockCanvasRenderer {
    canvases: Arc<std::sync::Mutex<HashMap<String, String>>>,
}

impl MockCanvasRenderer {
    fn new() -> Self {
        Self {
            canvases: Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }

    fn canvas_exists(&self, canvas_id: &str) -> bool {
        self.canvases.lock().unwrap().contains_key(canvas_id)
    }

    fn get_canvas_content(&self, canvas_id: &str) -> Option<String> {
        self.canvases.lock().unwrap().get(canvas_id).cloned()
    }
}

#[async_trait]
impl CanvasRenderer for MockCanvasRenderer {
    async fn create(&self, canvas_id: &str, content: &str) -> Result<()> {
        if self.canvases.lock().unwrap().contains_key(canvas_id) {
            return Err(anyhow::anyhow!(
                "Canvas with ID '{}' already exists",
                canvas_id
            ));
        }
        self.canvases
            .lock()
            .unwrap()
            .insert(canvas_id.to_string(), content.to_string());
        Ok(())
    }

    async fn update(&self, canvas_id: &str, content: &str) -> Result<()> {
        if !self.canvases.lock().unwrap().contains_key(canvas_id) {
            return Err(anyhow::anyhow!("Canvas with ID '{}' not found", canvas_id));
        }
        self.canvases
            .lock()
            .unwrap()
            .insert(canvas_id.to_string(), content.to_string());
        Ok(())
    }

    async fn get(&self, canvas_id: &str) -> Result<String> {
        self.canvases
            .lock()
            .unwrap()
            .get(canvas_id)
            .map(|s| s.clone())
            .ok_or_else(|| anyhow::anyhow!("Canvas with ID '{}' not found", canvas_id))
    }
}

#[tokio::test]
async fn test_canvas_tool_name() {
    let tool = CanvasTool::with_in_memory();
    assert_eq!(tool.name(), "canvas");
}

#[tokio::test]
async fn test_canvas_tool_description() {
    let tool = CanvasTool::with_in_memory();
    assert_eq!(
        tool.description(),
        "Generate and manage visual HTML/CSS/JS output"
    );
}

#[tokio::test]
async fn test_canvas_tool_schema() {
    let tool = CanvasTool::with_in_memory();
    let schema = tool.parameters_schema();

    assert_eq!(schema["type"], "object");
    assert_eq!(schema["properties"]["operation"]["type"], "string");
    assert_eq!(
        schema["properties"]["operation"]["enum"],
        json!(["create", "update", "get"])
    );
    assert!(schema["properties"]["canvas_id"].is_object());
    assert!(schema["properties"]["content"].is_object());

    let required = schema["required"].as_array().unwrap();
    assert!(required.contains(&json!("operation")));
    assert!(required.contains(&json!("canvas_id")));
}

#[tokio::test]
async fn test_create_canvas() {
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
    assert!(output.content.contains("created"));
}

#[tokio::test]
async fn test_create_canvas_with_complex_content() {
    let tool = CanvasTool::with_in_memory();
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "operation": "create",
                "canvas_id": "complex_canvas",
                "content": "<!DOCTYPE html>\n<html>\n<head>\n  <style>canvas { background: #f0f0f0; }</style>\n</head>\n<body>\n  <canvas id='myCanvas' width='500' height='400'></canvas>\n  <script>\n    const canvas = document.getElementById('myCanvas');\n    const ctx = canvas.getContext('2d');\n    ctx.fillStyle = '#FF0000';\n    ctx.fillRect(0, 0, 80, 80);\n  </script>\n</body>\n</html>"
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.is_error);
}

#[tokio::test]
async fn test_create_canvas_duplicate_id() {
    let tool = CanvasTool::with_in_memory();
    let ctx = ToolContext::new("test_agent", "test_session");

    // Create canvas first time
    let result = tool
        .execute(
            json!({
                "operation": "create",
                "canvas_id": "unique-canvas",
                "content": "<html><body>Canvas 1</body></html>"
            }),
            &ctx,
        )
        .await;
    assert!(result.is_ok());

    // Try to create with same ID
    let result = tool
        .execute(
            json!({
                "operation": "create",
                "canvas_id": "unique-canvas",
                "content": "<html><body>Canvas 2</body></html>"
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
async fn test_retrieve_canvas() {
    let tool = CanvasTool::with_in_memory();
    let ctx = ToolContext::new("test_agent", "test_session");

    // First create a canvas
    let create_result = tool
        .execute(
            json!({
                "operation": "create",
                "canvas_id": "retrieve-test",
                "content": "<html><body>Test Content</body></html>"
            }),
            &ctx,
        )
        .await;
    assert!(create_result.is_ok());

    // Now retrieve it
    let result = tool
        .execute(
            json!({
                "operation": "get",
                "canvas_id": "retrieve-test"
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.is_error);
    assert!(output.content.contains("Test Content"));
}

#[tokio::test]
async fn test_retrieve_nonexistent_canvas() {
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
async fn test_update_canvas() {
    let tool = CanvasTool::with_in_memory();
    let ctx = ToolContext::new("test_agent", "test_session");

    // First create a canvas
    let create_result = tool
        .execute(
            json!({
                "operation": "create",
                "canvas_id": "update-test",
                "content": "<html><body>Original Content</body></html>"
            }),
            &ctx,
        )
        .await;
    assert!(create_result.is_ok());

    // Now update it
    let result = tool
        .execute(
            json!({
                "operation": "update",
                "canvas_id": "update-test",
                "content": "<html><body>Updated Content</body></html>"
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.is_error);
    assert!(output.content.contains("updated successfully"));

    // Verify the update by getting the canvas
    let get_result = tool
        .execute(
            json!({
                "operation": "get",
                "canvas_id": "update-test"
            }),
            &ctx,
        )
        .await;
    assert!(get_result.is_ok());
    let get_output = get_result.unwrap();
    assert!(!get_output.is_error);
    assert!(get_output.content.contains("Updated Content"));
}

#[tokio::test]
async fn test_update_nonexistent_canvas() {
    let tool = CanvasTool::with_in_memory();
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "operation": "update",
                "canvas_id": "nonexistent",
                "content": "<html><body>Content</body></html>"
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
async fn test_missing_operation() {
    let tool = CanvasTool::with_in_memory();
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "canvas_id": "test",
                "content": "<html></html>"
            }),
            &ctx,
        )
        .await;

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Missing required parameter 'operation'"));
}

#[tokio::test]
async fn test_missing_canvas_id() {
    let tool = CanvasTool::with_in_memory();
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "operation": "create",
                "content": "<html></html>"
            }),
            &ctx,
        )
        .await;

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Missing required parameter 'canvas_id'"));
}

#[tokio::test]
async fn test_create_missing_content() {
    let tool = CanvasTool::with_in_memory();
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "operation": "create",
                "canvas_id": "test"
            }),
            &ctx,
        )
        .await;

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Missing required parameter 'content'"));
}

#[tokio::test]
async fn test_update_missing_content() {
    let tool = CanvasTool::with_in_memory();
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "operation": "update",
                "canvas_id": "test"
            }),
            &ctx,
        )
        .await;

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Missing required parameter 'content'"));
}

#[tokio::test]
async fn test_invalid_operation() {
    let tool = CanvasTool::with_in_memory();
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "operation": "invalid",
                "canvas_id": "test"
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
async fn test_with_mock_renderer() {
    let renderer = MockCanvasRenderer::new();
    let tool = CanvasTool::new(Arc::new(renderer.clone()));
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "operation": "create",
                "canvas_id": "mock-canvas",
                "content": "<html>Mock Content</html>"
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());
    assert!(renderer.canvas_exists("mock-canvas"));
}

#[tokio::test]
async fn test_mock_renderer_get() {
    let renderer = MockCanvasRenderer::new();
    let tool = CanvasTool::new(Arc::new(renderer.clone()));
    let ctx = ToolContext::new("test_agent", "test_session");

    // Create via mock
    let create_result = tool
        .execute(
            json!({
                "operation": "create",
                "canvas_id": "get-test",
                "content": "<html>Get Test</html>"
            }),
            &ctx,
        )
        .await;
    assert!(create_result.is_ok());

    // Get via mock
    let result = tool
        .execute(
            json!({
                "operation": "get",
                "canvas_id": "get-test"
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.is_error);
    assert!(output.content.contains("Get Test"));
}

#[tokio::test]
async fn test_mock_renderer_update() {
    let renderer = MockCanvasRenderer::new();
    let tool = CanvasTool::new(Arc::new(renderer.clone()));
    let ctx = ToolContext::new("test_agent", "test_session");

    // Create via mock
    let create_result = tool
        .execute(
            json!({
                "operation": "create",
                "canvas_id": "update-mock",
                "content": "<html>Original</html>"
            }),
            &ctx,
        )
        .await;
    assert!(create_result.is_ok());

    // Update via mock
    let result = tool
        .execute(
            json!({
                "operation": "update",
                "canvas_id": "update-mock",
                "content": "<html>Updated</html>"
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());

    // Verify content was updated
    let content = renderer.get_canvas_content("update-mock");
    assert!(content.is_some());
    assert!(content.unwrap().contains("Updated"));
}

#[tokio::test]
async fn test_multiple_canvases() {
    let tool = CanvasTool::with_in_memory();
    let ctx = ToolContext::new("test_agent", "test_session");

    // Create multiple canvases
    let canvas_ids = [
        concat!("canvas-", "1"),
        concat!("canvas-", "2"),
        concat!("canvas-", "3"),
    ];

    for canvas_id in canvas_ids.iter() {
        let result = tool
            .execute(
                json!({
                    "operation": "create",
                    "canvas_id": canvas_id,
                    "content": format!("<html><body>Canvas {}</body></html>", canvas_id)
                }),
                &ctx,
            )
            .await;
        assert!(result.is_ok());
    }

    // Retrieve each one
    for canvas_id in canvas_ids.iter() {
        let result = tool
            .execute(
                json!({
                    "operation": "get",
                    "canvas_id": canvas_id
                }),
                &ctx,
            )
            .await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.is_error);
        assert!(output.content.contains(canvas_id));
    }
}

#[tokio::test]
async fn test_in_memory_renderer_create() {
    let renderer = InMemoryCanvasRenderer::new();

    let canvas_id = String::from("test") + "-" + "canvas";
    let result = renderer.create(&canvas_id, "<html>Test</html>").await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_in_memory_renderer_create_duplicate() {
    let renderer = InMemoryCanvasRenderer::new();

    // First create
    let canvas_id = concat!("unique-", "canvas");
    renderer
        .create(canvas_id, "<html>Test</html>")
        .await
        .unwrap();

    // Second create with same ID should fail
    let result = renderer.create(canvas_id, "<html>Test2</html>").await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_in_memory_renderer_update() {
    let renderer = InMemoryCanvasRenderer::new();

    // First create
    let canvas_id = concat!("update-", "canvas");
    renderer
        .create(canvas_id, "<html>Original</html>")
        .await
        .unwrap();

    // Then update
    let result = renderer.update(canvas_id, "<html>Updated</html>").await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_in_memory_renderer_update_nonexistent() {
    let renderer = InMemoryCanvasRenderer::new();

    let result = renderer.update("nonexistent", "<html>Content</html>").await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_in_memory_renderer_get() {
    let renderer = InMemoryCanvasRenderer::new();

    let canvas_id = concat!("get-", "canvas");
    let result = renderer
        .create(canvas_id, "<html>Get Me</html>")
        .await
        .unwrap();

    // Then get
    let result = renderer.get(canvas_id).await;

    assert!(result.is_ok());
    let content = result.unwrap();
    let get_part = "Get";
    let me_part = "Me";
    assert!(content.contains(get_part));
    assert!(content.contains(me_part));
}

#[tokio::test]
async fn test_in_memory_renderer_get_nonexistent() {
    let renderer = InMemoryCanvasRenderer::new();

    let result = renderer.get("nonexistent").await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_canvas_tool_with_special_characters() {
    let tool = CanvasTool::with_in_memory();
    let ctx = ToolContext::new("test_agent", "test_session");

    // Store content in a variable - using JSON-compatible content
    // Avoid single quote which causes parsing issues
    let content = "<html><body>Special: !@#$%^&*()_+-=[]{}|;</body></html>";

    let result = tool
        .execute(
            json!({
                "operation": "create",
                "canvas_id": concat!("special-", "chars"),
                "content": content
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_canvas_tool_multiline_content() {
    let tool = CanvasTool::with_in_memory();
    let ctx = ToolContext::new("test_agent", "test_session");

    // Build content using format! with newline character
    let content = format!("<!DOCTYPE html>{}<html>{}<head>{}  <title>Test</title>{} </head>{}<body>{}  <h1>Hello</h1>{} </body>{} </html>",
        '\n', '\n', '\n', '\n', '\n', '\n', '\n', '\n');

    let result = tool
        .execute(
            json!({
                "operation": "create",
                "canvas_id": concat!("multi-", "line"),
                "content": content
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_canvas_tool_with_unicode() {
    let tool = CanvasTool::with_in_memory();
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "operation": "create",
                "canvas_id": concat!("unicode-", "canvas"),
                "content": "<html><body>こんにちは世界 🌍</body></html>"
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_canvas_tool_empty_content() {
    let tool = CanvasTool::with_in_memory();
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "operation": "create",
                "canvas_id": concat!("empty-", "content"),
                "content": ""
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.is_error);
}

#[tokio::test]
async fn test_canvas_tool_empty_canvas_id() {
    let tool = CanvasTool::with_in_memory();
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "operation": "create",
                "canvas_id": "",
                "content": "<html></html>"
            }),
            &ctx,
        )
        .await;

    // Empty string is still a valid parameter value
    assert!(result.is_ok());
}
