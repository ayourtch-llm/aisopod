//! Built-in file operations tool for reading, writing, searching, listing, and inspecting files.

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use tokio::fs;
use tokio::fs::{create_dir_all, read_dir};
use walkdir::WalkDir;

use crate::{Tool, ToolContext, ToolResult};

/// A built-in tool for file operations including read, write, search, list, and metadata operations.
///
/// This tool provides a safe way to perform file system operations from within
/// the aisopod system. It enforces workspace path restrictions and uses
/// canonicalized paths for security.
///
/// # Configuration
///
/// The tool operates within a workspace directory configured via ToolContext.
/// All file operations are restricted to the workspace directory for security.
///
/// # Operations
///
/// The tool supports the following operations:
///
/// - `read`: Read a file's contents as text
/// - `write`: Write content to a file (creates parent directories if needed)
/// - `search`: Search for patterns in files using glob patterns
/// - `list`: List directory contents with details
/// - `metadata`: Get file/directory metadata including size, permissions, and modification time
///
/// # Parameters
///
/// All operations require:
/// - `operation`: One of "read", "write", "search", "list", "metadata"
/// - `path`: Path to the file or directory (relative to workspace)
///
/// Operation-specific parameters:
/// - `write`: Requires `content` field with the text to write
/// - `search`: Requires `pattern` field with the search pattern (supports glob syntax)
/// - `list`: Optional `glob` field to filter entries
///
/// # Example
///
/// ```json
/// {
///   "operation": "read",
///   "path": "src/main.rs"
/// }
/// ```
///
/// ```json
/// {
///   "operation": "write",
///   "path": "src/new_file.rs",
///   "content": "fn main() { println!(\"Hello\"); }"
/// }
/// ```
///
/// ```json
/// {
///   "operation": "search",
///   "path": ".",
///   "pattern": "*.rs"
/// }
/// ```
#[derive(Debug, Clone)]
pub struct FileTool;

impl Default for FileTool {
    fn default() -> Self {
        Self::new()
    }
}

impl FileTool {
    /// Creates a new FileTool instance.
    pub fn new() -> Self {
        FileTool
    }

    /// Resolves a path relative to the workspace and validates it stays within bounds.
    ///
    /// This helper ensures all file operations stay within the workspace
    /// directory for security. It returns an error if the resolved path
    /// would escape the workspace, even if the file doesn't exist.
    async fn resolve_path(&self, path: &Path, ctx: &ToolContext) -> Result<PathBuf> {
        let workspace = ctx
            .workspace_path
            .as_ref()
            .ok_or_else(|| anyhow!("Workspace path not configured in ToolContext"))?;

        let resolved = workspace.join(path);

        // Normalize the path to resolve any .. and . components
        let normalized = resolved
            .components()
            .fold(PathBuf::new(), |mut acc, component| {
                match component {
                    std::path::Component::CurDir => {}
                    std::path::Component::ParentDir => {
                        acc.pop();
                    }
                    _ => acc.push(component.as_os_str()),
                }
                acc
            });

        // Canonicalize the workspace path
        let workspace_canonical = fs::canonicalize(workspace)
            .await
            .map_err(|e| anyhow!("Failed to canonicalize workspace '{}': {}", workspace.display(), e))?;

        // Check if the normalized path starts with the workspace canonical path
        if !normalized.starts_with(&workspace_canonical) {
            return Err(anyhow!(
                "Path '{}' escapes the workspace directory '{}'",
                path.display(),
                workspace.display()
            ));
        }

        Ok(normalized)
    }

    /// Reads a file and returns its contents.
    async fn read_file(&self, path: &Path, ctx: &ToolContext) -> Result<ToolResult> {
        let resolved_path = self.resolve_path(path, ctx).await?;

        let content = fs::read_to_string(&resolved_path)
            .await
            .map_err(|e| anyhow!("Failed to read file '{}': {}", resolved_path.display(), e))?;

        Ok(ToolResult::success(content))
    }

    /// Writes content to a file, creating parent directories if needed.
    async fn write_file(&self, path: &Path, content: &str, ctx: &ToolContext) -> Result<ToolResult> {
        let resolved_path = self.resolve_path(path, ctx).await?;

        // Create parent directories if they don't exist
        if let Some(parent) = resolved_path.parent() {
            if !parent.exists() {
                create_dir_all(parent)
                    .await
                    .map_err(|e| anyhow!("Failed to create directories for '{}': {}", parent.display(), e))?;
            }
        }

        // Write the content
        fs::write(&resolved_path, content)
            .await
            .map_err(|e| anyhow!("Failed to write file '{}': {}", resolved_path.display(), e))?;

        Ok(ToolResult::success(format!(
            "Successfully wrote to '{}'",
            resolved_path.display()
        )))
    }

    /// Searches for files matching a glob pattern.
    async fn search_files(&self, path: &Path, pattern: &str, ctx: &ToolContext) -> Result<ToolResult> {
        let resolved_path = self.resolve_path(path, ctx).await?;

        // Build the glob pattern - if pattern contains no path separators, search recursively
        let search_path = if pattern.contains('/') || pattern.contains('\\') {
            resolved_path.join(pattern)
        } else {
            resolved_path.join("**").join(pattern)
        };

        let glob_pattern = search_path.to_string_lossy();

        let mut results: Vec<String> = Vec::new();

        for entry in WalkDir::new(&resolved_path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let entry_path = entry.path();

            // Check if the path matches the glob pattern
            let entry_path_str = entry_path.to_string_lossy();
            
            // Simple glob matching - check if the pattern matches
            if let Some(relative_path) = entry_path_str.strip_prefix(&*resolved_path.to_string_lossy()) {
                // Remove leading separator
                let relative_path = relative_path.trim_start_matches('/').trim_start_matches('\\');
                
                // Check if this relative path matches the pattern
                if glob_match(&pattern, relative_path) {
                    results.push(entry_path.display().to_string());
                }
            }
        }

        if results.is_empty() {
            Ok(ToolResult::success(format!(
                "No files found matching pattern '{}' in '{}'",
                pattern,
                resolved_path.display()
            )))
        } else {
            let output = format!(
                "Found {} file(s) matching '{}':\n{}",
                results.len(),
                pattern,
                results.join("\n")
            );
            Ok(ToolResult::success(output))
        }
    }

    /// Lists directory contents with details.
    async fn list_directory(&self, path: &Path, glob_filter: Option<&str>, ctx: &ToolContext) -> Result<ToolResult> {
        let resolved_path = self.resolve_path(path, ctx).await?;

        let mut entries = fs::read_dir(&resolved_path)
            .await
            .map_err(|e| anyhow!("Failed to read directory '{}': {}", resolved_path.display(), e))?;

        let mut results: Vec<Value> = Vec::new();

        loop {
            match entries.next_entry().await {
                Ok(Some(entry)) => {
                    let entry_path = entry.path();
                    
                    // Get metadata - handle the case where the entry was deleted between read_dir and metadata
                    let metadata = match entry.metadata().await {
                        Ok(m) => m,
                        Err(_) => continue, // Entry was deleted, skip it
                    };
                    let file_type = metadata.file_type();

                    // Apply glob filter if specified
                    if let Some(filter) = glob_filter {
                        let file_name = entry_path.file_name().map(|f| f.to_string_lossy()).unwrap_or_default();
                        if !glob_match(filter, &file_name) {
                            continue;
                        }
                    }

                    let entry_info = json!({
                        "name": entry_path.file_name().map(|f| f.to_string_lossy()).unwrap_or_default(),
                        "path": entry_path.display().to_string(),
                        "is_file": file_type.is_file(),
                        "is_dir": file_type.is_dir(),
                        "is_symlink": file_type.is_symlink(),
                        "size": metadata.len(),
                        "permissions": format!("{:o}", metadata.permissions().mode() & 0o777),
                        "modified": metadata.modified().ok().and_then(|t| {
                            t.duration_since(std::time::UNIX_EPOCH).ok().map(|d| {
                                let seconds = d.as_secs();
                                let nanos = d.subsec_nanos();
                                DateTime::<Utc>::from_timestamp(seconds as i64, nanos as u32)
                                    .map(|dt| dt.to_rfc3339())
                                    .unwrap_or_default()
                            })
                        }).unwrap_or_default(),
                    });

                    results.push(entry_info);
                }
                Ok(None) => break,
                Err(e) => {
                    return Err(anyhow!("Failed to read directory entry: {}", e));
                }
            }
        }

        if results.is_empty() {
            Ok(ToolResult::success(format!(
                "Directory '{}' is empty",
                resolved_path.display()
            )))
        } else {
            Ok(ToolResult::success(format!(
                "Contents of '{}':\n{}",
                resolved_path.display(),
                serde_json::to_string_pretty(&results)?
            )))
        }
    }

    /// Gets file/directory metadata.
    async fn get_metadata(&self, path: &Path, ctx: &ToolContext) -> Result<ToolResult> {
        let resolved_path = self.resolve_path(path, ctx).await?;

        let metadata = fs::metadata(&resolved_path)
            .await
            .map_err(|e| anyhow!("Failed to get metadata for '{}': {}", resolved_path.display(), e))?;

        let file_type = metadata.file_type();
        let permissions = metadata.permissions();

        let modified = metadata
            .modified()
            .ok()
            .and_then(|t| {
                t.duration_since(std::time::UNIX_EPOCH).ok().map(|d| {
                    let seconds = d.as_secs();
                    let nanos = d.subsec_nanos();
                    DateTime::<Utc>::from_timestamp(seconds as i64, nanos as u32)
                        .map(|dt| dt.to_rfc3339())
                        .unwrap_or_default()
                })
            })
            .unwrap_or_default();

        let metadata_info = json!({
            "path": resolved_path.display().to_string(),
            "is_file": file_type.is_file(),
            "is_dir": file_type.is_dir(),
            "is_symlink": file_type.is_symlink(),
            "size": metadata.len(),
            "permissions_octal": format!("{:o}", permissions.mode() & 0o777),
            "permissions_human": format_permissions(permissions.mode() & 0o777),
            "modified_iso8601": modified,
            "modified_timestamp": metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs()),
        });

        Ok(ToolResult::success(format!(
            "Metadata for '{}':\n{}",
            resolved_path.display(),
            serde_json::to_string_pretty(&metadata_info)?
        )))
    }
}

/// Simple glob pattern matching.
///
/// Supports:
/// - `*` - matches any sequence of characters except path separators
/// - `**` - matches any sequence of characters including path separators
/// - `?` - matches any single character
/// - `[abc]` - matches any character in the set
/// - `[a-z]` - matches any character in the range
fn glob_match(pattern: &str, text: &str) -> bool {
    let pattern = pattern.to_string();
    let text = text.to_string();
    
    // Handle ** for recursive matching
    if pattern == "**" {
        return true;
    }

    // Simple implementation using regex
    let regex_pattern = glob_to_regex(&pattern);
    regex::Regex::new(&regex_pattern)
        .map(|re| re.is_match(&text))
        .unwrap_or(false)
}

/// Converts a glob pattern to a regex pattern.
fn glob_to_regex(pattern: &str) -> String {
    let mut regex = String::new();
    let chars: Vec<char> = pattern.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        match chars[i] {
            '*' => {
                if i + 1 < chars.len() && chars[i + 1] == '*' {
                    // ** matches any sequence including path separators
                    regex.push_str(".*");
                    i += 2;
                } else {
                    // * matches any sequence except path separators
                    regex.push_str("[^/\\\\]*");
                    i += 1;
                }
            }
            '?' => {
                // ? matches any single character except path separators
                regex.push_str("[^/\\\\]");
                i += 1;
            }
            '[' => {
                // Character class - find matching ]
                let mut end = i + 1;
                while end < chars.len() && chars[end] != ']' {
                    end += 1;
                }
                if end < chars.len() {
                    // Extract the character class
                    let class: String = chars[i..=end].iter().collect();
                    regex.push_str(&glob_class_to_regex(&class));
                    i = end + 1;
                } else {
                    regex.push_str("\\[");
                    i += 1;
                }
            }
            c => {
                // Escape regex special characters
                match c {
                    '.' | '\\' | '+' | '^' | '$' | '(' | ')' | '|' | '{' | '}' | '=' | '!' | '<' | '>' | ':' | '-' => {
                        regex.push('\\');
                    }
                    _ => {}
                }
                regex.push(c);
                i += 1;
            }
        }
    }

    format!("^{}$", regex)
}

/// Converts a glob character class to regex.
fn glob_class_to_regex(class: &str) -> String {
    let inner: String = class[1..class.len() - 1].chars().collect();
    let mut result = String::from("[");
    
    let mut chars: Vec<char> = inner.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        match chars[i] {
            '^' if i == 0 => {
                result.push('^');
                i += 1;
            }
            ']' if i == 0 => {
                result.push_str("\\]");
                i += 1;
            }
            '-' if i > 0 && i + 1 < chars.len() && chars[i + 1] != ']' => {
                // Range like a-z
                let start = chars[i - 1];
                let end = chars[i + 1];
                result.push_str(&format!("{}-{}", start, end));
                i += 3;
            }
            '\\' | ']' | '[' => {
                result.push('\\');
                result.push(chars[i]);
                i += 1;
            }
            c => {
                result.push(c);
                i += 1;
            }
        }
    }
    
    result.push(']');
    result
}

/// Formats permissions in human-readable form (e.g., "rw-r--r--").
fn format_permissions(mode: u32) -> String {
    let mut fmt = String::with_capacity(9);
    
    // Owner permissions
    fmt.push(if mode & 0o400 > 0 { 'r' } else { '-' });
    fmt.push(if mode & 0o200 > 0 { 'w' } else { '-' });
    fmt.push(if mode & 0o100 > 0 {
        if mode & 0o4000 > 0 { 's' } else { 'x' }
    } else {
        '-'
    });
    
    // Group permissions
    fmt.push(if mode & 0o040 > 0 { 'r' } else { '-' });
    fmt.push(if mode & 0o020 > 0 { 'w' } else { '-' });
    fmt.push(if mode & 0o010 > 0 {
        if mode & 0o2000 > 0 { 's' } else { 'x' }
    } else {
        '-'
    });
    
    // Others permissions
    fmt.push(if mode & 0o004 > 0 { 'r' } else { '-' });
    fmt.push(if mode & 0o002 > 0 { 'w' } else { '-' });
    fmt.push(if mode & 0o001 > 0 {
        if mode & 0o1000 > 0 { 't' } else { 'x' }
    } else {
        '-'
    });
    
    fmt
}

#[async_trait]
impl Tool for FileTool {
    fn name(&self) -> &str {
        "file"
    }

    fn description(&self) -> &str {
        "Read, write, search, list, and inspect files"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["read", "write", "search", "list", "metadata"],
                    "description": "The file operation to perform"
                },
                "path": {
                    "type": "string",
                    "description": "Path to the file or directory (relative to workspace)"
                },
                "content": {
                    "type": "string",
                    "description": "Content to write (required for 'write' operation)"
                },
                "pattern": {
                    "type": "string",
                    "description": "Glob pattern for search (required for 'search' operation)"
                },
                "glob": {
                    "type": "string",
                    "description": "Glob pattern to filter list entries (optional for 'list' operation)"
                }
            },
            "required": ["operation", "path"]
        })
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        // Extract operation (required)
        let operation = params
            .get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required parameter 'operation'"))?;

        // Validate operation
        match operation {
            "read" | "search" | "list" | "metadata" => {
                // These operations require path only
            }
            "write" => {
                // Write operation requires content
                if params.get("content").is_none() {
                    return Err(anyhow!("Write operation requires 'content' parameter"));
                }
            }
            _ => {
                return Err(anyhow!(
                    "Invalid operation '{}'. Must be one of: read, write, search, list, metadata",
                    operation
                ));
            }
        }

        // Extract path (required for all operations)
        let path_str = params
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required parameter 'path'"))?;

        let path = Path::new(path_str);

        // Dispatch to operation-specific handler
        match operation {
            "read" => self.read_file(path, ctx).await,
            "write" => {
                let content = params
                    .get("content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Write operation requires 'content' parameter"))?;
                self.write_file(path, content, ctx).await
            }
            "search" => {
                let pattern = params
                    .get("pattern")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Search operation requires 'pattern' parameter"))?;
                self.search_files(path, pattern, ctx).await
            }
            "list" => {
                let glob_filter = params
                    .get("glob")
                    .and_then(|v| v.as_str());
                self.list_directory(path, glob_filter, ctx).await
            }
            "metadata" => self.get_metadata(path, ctx).await,
            _ => Err(anyhow!("Invalid operation: {}", operation)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

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
        assert_eq!(schema["properties"]["operation"]["type"], "string");
        assert_eq!(schema["properties"]["operation"]["enum"], json!(["read", "write", "search", "list", "metadata"]));
        assert!(schema["required"].as_array().unwrap().contains(&json!("operation")));
        assert!(schema["required"].as_array().unwrap().contains(&json!("path")));
    }

    #[tokio::test]
    async fn test_file_tool_read() {
        let tool = FileTool::new();
        
        // Create a temporary directory for testing
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "Hello, World!").unwrap();

        let ctx = ToolContext::new("test_agent", "test_session")
            .with_workspace_path(temp_dir.path());

        let result = tool
            .execute(
                json!({
                    "operation": "read",
                    "path": "test.txt"
                }),
                &ctx,
            )
            .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.is_error);
        assert!(output.content.contains("Hello, World!"));
    }

    #[tokio::test]
    async fn test_file_tool_write() {
        let tool = FileTool::new();
        
        let temp_dir = TempDir::new().unwrap();
        let ctx = ToolContext::new("test_agent", "test_session")
            .with_workspace_path(temp_dir.path());

        let result = tool
            .execute(
                json!({
                    "operation": "write",
                    "path": "new_file.txt",
                    "content": "New file content"
                }),
                &ctx,
            )
            .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.is_error);

        // Verify the file was created
        let file_path = temp_dir.path().join("new_file.txt");
        assert!(file_path.exists());
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "New file content");
    }

    #[tokio::test]
    async fn test_file_tool_write_creates_parent_dirs() {
        let tool = FileTool::new();
        
        let temp_dir = TempDir::new().unwrap();
        let ctx = ToolContext::new("test_agent", "test_session")
            .with_workspace_path(temp_dir.path());

        let result = tool
            .execute(
                json!({
                    "operation": "write",
                    "path": "nested/path/to/file.txt",
                    "content": "Content in nested dir"
                }),
                &ctx,
            )
            .await;

        assert!(result.is_ok());

        // Verify the file was created with parent directories
        let file_path = temp_dir.path().join("nested/path/to/file.txt");
        assert!(file_path.exists());
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "Content in nested dir");
    }

    #[tokio::test]
    async fn test_file_tool_metadata() {
        let tool = FileTool::new();
        
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "test").unwrap();

        let ctx = ToolContext::new("test_agent", "test_session")
            .with_workspace_path(temp_dir.path());

        let result = tool
            .execute(
                json!({
                    "operation": "metadata",
                    "path": "test.txt"
                }),
                &ctx,
            )
            .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.is_error);
        assert!(output.content.contains("test.txt"));
        assert!(output.content.contains("\"is_file\": true"));
    }

    #[tokio::test]
    async fn test_file_tool_list() {
        let tool = FileTool::new();
        
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("file1.txt"), "content1").unwrap();
        fs::write(temp_dir.path().join("file2.txt"), "content2").unwrap();
        fs::create_dir(temp_dir.path().join("subdir")).unwrap();

        let ctx = ToolContext::new("test_agent", "test_session")
            .with_workspace_path(temp_dir.path());

        let result = tool
            .execute(
                json!({
                    "operation": "list",
                    "path": "."
                }),
                &ctx,
            )
            .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.is_error);
        assert!(output.content.contains("file1.txt"));
        assert!(output.content.contains("file2.txt"));
        assert!(output.content.contains("subdir"));
    }

    #[tokio::test]
    async fn test_file_tool_list_with_glob_filter() {
        let tool = FileTool::new();
        
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("file1.txt"), "content1").unwrap();
        fs::write(temp_dir.path().join("file2.rs"), "content2").unwrap();
        fs::write(temp_dir.path().join("file3.txt"), "content3").unwrap();

        let ctx = ToolContext::new("test_agent", "test_session")
            .with_workspace_path(temp_dir.path());

        let result = tool
            .execute(
                json!({
                    "operation": "list",
                    "path": ".",
                    "glob": "*.txt"
                }),
                &ctx,
            )
            .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.is_error);
        assert!(output.content.contains("file1.txt"));
        assert!(output.content.contains("file3.txt"));
        assert!(!output.content.contains("file2.rs"));
    }

    #[tokio::test]
    async fn test_file_tool_search() {
        let tool = FileTool::new();
        
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("main.rs"), "fn main() {}").unwrap();
        fs::write(temp_dir.path().join("lib.rs"), "pub fn test() {}").unwrap();
        fs::write(temp_dir.path().join("readme.md"), "# Test").unwrap();

        let ctx = ToolContext::new("test_agent", "test_session")
            .with_workspace_path(temp_dir.path());

        let result = tool
            .execute(
                json!({
                    "operation": "search",
                    "path": ".",
                    "pattern": "*.rs"
                }),
                &ctx,
            )
            .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.is_error);
        assert!(output.content.contains("main.rs"));
        assert!(output.content.contains("lib.rs"));
    }

    #[tokio::test]
    async fn test_file_tool_escape_workspace() {
        let tool = FileTool::new();
        
        let temp_dir = TempDir::new().unwrap();
        let ctx = ToolContext::new("test_agent", "test_session")
            .with_workspace_path(temp_dir.path());

        // Try to access a file outside the workspace
        let result = tool
            .execute(
                json!({
                    "operation": "read",
                    "path": "../outside.txt"
                }),
                &ctx,
            )
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("escapes"));
    }

    #[tokio::test]
    async fn test_file_tool_missing_operation() {
        let tool = FileTool::new();
        
        let temp_dir = TempDir::new().unwrap();
        let ctx = ToolContext::new("test_agent", "test_session")
            .with_workspace_path(temp_dir.path());

        let result = tool
            .execute(
                json!({
                    "path": "test.txt"
                }),
                &ctx,
            )
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Missing required parameter 'operation'"));
    }

    #[tokio::test]
    async fn test_file_tool_invalid_operation() {
        let tool = FileTool::new();
        
        let temp_dir = TempDir::new().unwrap();
        let ctx = ToolContext::new("test_agent", "test_session")
            .with_workspace_path(temp_dir.path());

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
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Invalid operation"));
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
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Missing required parameter 'path'"));
    }

    #[tokio::test]
    async fn test_file_tool_write_missing_content() {
        let tool = FileTool::new();
        
        let temp_dir = TempDir::new().unwrap();
        let ctx = ToolContext::new("test_agent", "test_session")
            .with_workspace_path(temp_dir.path());

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
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Write operation requires 'content' parameter"));
    }

    #[tokio::test]
    async fn test_file_tool_search_missing_pattern() {
        let tool = FileTool::new();
        
        let temp_dir = TempDir::new().unwrap();
        let ctx = ToolContext::new("test_agent", "test_session")
            .with_workspace_path(temp_dir.path());

        let result = tool
            .execute(
                json!({
                    "operation": "search",
                    "path": "."
                }),
                &ctx,
            )
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Search operation requires 'pattern' parameter"));
    }

    #[tokio::test]
    async fn test_file_tool_no_workspace() {
        let tool = FileTool::new();
        let ctx = ToolContext::new("test_agent", "test_session");

        let result = tool
            .execute(
                json!({
                    "operation": "read",
                    "path": "test.txt"
                }),
                &ctx,
            )
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Workspace path not configured"));
    }
}
