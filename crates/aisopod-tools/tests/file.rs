//! File tool tests

use std::fs;
use std::path::PathBuf;

use aisopod_tools::{FileTool, Tool, ToolContext, ToolResult};
use serde_json::json;

#[tokio::test]
async fn test_file_tool_name() {
    let tool = FileTool::new();
    assert_eq!(tool.name(), "file");
}

#[tokio::test]
async fn test_file_tool_description() {
    let tool = FileTool::new();
    assert_eq!(tool.description(), "Read, write, search, list, and inspect files");
}

#[tokio::test]
async fn test_file_tool_schema() {
    let tool = FileTool::new();
    let schema = tool.parameters_schema();

    assert_eq!(schema["type"], "object");
    assert!(schema["properties"]["operation"].is_object());
    assert!(schema["properties"]["path"].is_object());
}

#[tokio::test]
async fn test_file_tool_read_file() {
    let tool = FileTool::new();
    
    // Create a temp file
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("test_file_read.txt");
    fs::write(&test_file, "Hello, World!").unwrap();
    
    let ctx = ToolContext::new("test_agent", "test_session")
        .with_workspace_path(temp_dir.clone());

    let result = tool
        .execute(
            json!({
                "operation": "read",
                "path": test_file.file_name().unwrap().to_str().unwrap()
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.is_error);
    assert!(output.content.contains("Hello, World!"));
    
    // Cleanup
    let _ = fs::remove_file(&test_file);
}

#[tokio::test]
async fn test_file_tool_write_file() {
    let tool = FileTool::new();
    
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("test_file_write.txt");
    let _ = fs::remove_file(&test_file); // Clean up if exists
    
    let ctx = ToolContext::new("test_agent", "test_session")
        .with_workspace_path(temp_dir.clone());

    let write_result = tool
        .execute(
            json!({
                "operation": "write",
                "path": test_file.file_name().unwrap().to_str().unwrap(),
                "content": "Test content"
            }),
            &ctx,
        )
        .await;

    assert!(write_result.is_ok());
    let output = write_result.unwrap();
    assert!(!output.is_error);
    
    // Verify file was written
    let content = fs::read_to_string(&test_file).unwrap();
    assert_eq!(content, "Test content");
    
    // Cleanup
    let _ = fs::remove_file(&test_file);
}

#[tokio::test]
async fn test_file_tool_list_directory() {
    let tool = FileTool::new();
    
    let temp_dir = std::env::temp_dir();
    let ctx = ToolContext::new("test_agent", "test_session")
        .with_workspace_path(temp_dir.clone());

    let result = tool
        .execute(
            json!({
                "operation": "list",
                "path": "."
            }),
            &ctx,
        )
        .await;

    if result.is_err() {
        eprintln!("list_directory result is Err: {:?}", result);
    }
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.is_error);
    // Should contain directory listing
}

#[tokio::test]
async fn test_file_tool_metadata() {
    let tool = FileTool::new();
    
    // Create a temp file
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("test_file_metadata.txt");
    fs::write(&test_file, "Metadata test").unwrap();
    
    let ctx = ToolContext::new("test_agent", "test_session")
        .with_workspace_path(temp_dir.clone());

    let result = tool
        .execute(
            json!({
                "operation": "metadata",
                "path": test_file.file_name().unwrap().to_str().unwrap()
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.is_error);
    // Should contain file metadata
    assert!(output.content.contains("size") || output.content.contains("metadata"));
    
    // Cleanup
    let _ = fs::remove_file(&test_file);
}

#[tokio::test]
async fn test_file_tool_workspace_path_restriction() {
    let tool = FileTool::new();
    
    let temp_dir = std::env::temp_dir();
    let ctx = ToolContext::new("test_agent", "test_session")
        .with_workspace_path(temp_dir.clone());

    // Try to access file outside workspace (should fail)
    let result = tool
        .execute(
            json!({
                "operation": "read",
                "path": "../../../etc/passwd"
            }),
            &ctx,
        )
        .await;

    assert!(result.is_err() || {
        // Some implementations may allow this depending on canonicalization
        let output = result.unwrap();
        output.is_error
    });
}

#[tokio::test]
async fn test_file_tool_search_glob() {
    let tool = FileTool::new();
    
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("test_file_glob.txt");
    fs::write(&test_file, "Glob test").unwrap();
    
    let ctx = ToolContext::new("test_agent", "test_session")
        .with_workspace_path(temp_dir);

    let result = tool
        .execute(
            json!({
                "operation": "search",
                "path": ".",
                "pattern": "*.txt"
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.is_error);
    
    // Cleanup
    let _ = fs::remove_file(&test_file);
}

#[tokio::test]
async fn test_file_tool_missing_operation() {
    let tool = FileTool::new();
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "path": "test.txt"
            }),
            &ctx,
        )
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_file_tool_missing_path() {
    let tool = FileTool::new();
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "operation": "read"
            }),
            &ctx,
        )
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_file_tool_missing_content_for_write() {
    let tool = FileTool::new();
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "operation": "write",
                "path": "test.txt"
            }),
            &ctx,
        )
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_file_tool_invalid_operation() {
    let tool = FileTool::new();
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "operation": "invalid",
                "path": "test.txt"
            }),
            &ctx,
        )
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_file_tool_nested_directory() {
    let tool = FileTool::new();
    
    let temp_dir = std::env::temp_dir();
    let nested_dir = temp_dir.join("nested_test_dir");
    let _ = fs::remove_dir_all(&nested_dir); // Clean up
    fs::create_dir_all(&nested_dir).unwrap();
    
    let test_file = nested_dir.join("nested_file.txt");
    fs::write(&test_file, "Nested content").unwrap();
    
    let ctx = ToolContext::new("test_agent", "test_session")
        .with_workspace_path(temp_dir);

    let result = tool
        .execute(
            json!({
                "operation": "read",
                "path": "nested_test_dir/nested_file.txt"
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.is_error);
    assert!(output.content.contains("Nested content"));
    
    // Cleanup
    let _ = fs::remove_dir_all(&nested_dir);
}

#[tokio::test]
async fn test_file_tool_search_nonexistent_pattern() {
    let tool = FileTool::new();
    
    let temp_dir = std::env::temp_dir();
    let ctx = ToolContext::new("test_agent", "test_session")
        .with_workspace_path(temp_dir);

    let result = tool
        .execute(
            json!({
                "operation": "search",
                "path": ".",
                "pattern": "*.nonexistent12345"
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.is_error);
    // Should return empty or indicate no matches
}

#[tokio::test]
async fn test_file_tool_list_empty_directory() {
    let tool = FileTool::new();
    
    let temp_dir = std::env::temp_dir();
    let empty_dir = temp_dir.join("empty_test_dir");
    let _ = fs::remove_dir_all(&empty_dir); // Clean up
    fs::create_dir_all(&empty_dir).unwrap();
    
    let ctx = ToolContext::new("test_agent", "test_session")
        .with_workspace_path(temp_dir);

    let result = tool
        .execute(
            json!({
                "operation": "list",
                "path": "empty_test_dir"
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.is_error);
    
    // Cleanup
    let _ = fs::remove_dir_all(&empty_dir);
}
