//! Canvas Protocol RPC types and handlers for aisopod-gateway.
//!
//! This module implements the canvas protocol that allows the server to push
//! interactive UI content (HTML/CSS/JS) to connected clients and receive
//! user interaction events back from those clients.
//!
//! ## Canvas Protocol
//!
//! - `canvas.update` (server → client): Server sends HTML/CSS/JS content to clients
//! - `canvas.interact` (client → server): Client reports user interactions back to server

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Parameters for canvas.update RPC method (server-initiated notification)
#[derive(Debug, Serialize)]
pub struct CanvasUpdateParams {
    /// Unique identifier for the canvas
    pub canvas_id: String,
    /// The action to perform on the canvas
    pub action: CanvasAction,
    /// The content to create or update (None for destroy)
    pub content: Option<CanvasContent>,
}

/// Actions that can be performed on a canvas
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum CanvasAction {
    /// Create a new canvas
    Create,
    /// Update an existing canvas
    Update,
    /// Destroy an existing canvas
    Destroy,
}

/// HTML/CSS/JS content for a canvas
#[derive(Debug, Serialize, Clone)]
pub struct CanvasContent {
    /// HTML content
    pub html: String,
    /// Optional CSS styling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub css: Option<String>,
    /// Optional JavaScript behavior
    #[serde(skip_serializing_if = "Option::is_none")]
    pub js: Option<String>,
    /// Optional canvas title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// Parameters for canvas.interact RPC method (client-initiated call)
#[derive(Debug, Deserialize, Serialize)]
pub struct CanvasInteractParams {
    /// The canvas_id that the interaction belongs to
    pub canvas_id: String,
    /// Type of event: "click", "input", "submit", "custom", etc.
    pub event_type: String,
    /// Optional element ID within the canvas
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element_id: Option<String>,
    /// Optional event-specific payload data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Result returned from canvas.interact handler
#[derive(Debug, Serialize)]
pub struct CanvasInteractResult {
    /// Whether the interaction was received and will be forwarded
    pub received: bool,
}

/// State tracking for canvases per connection
#[derive(Debug, Default)]
pub struct CanvasState {
    /// Map of canvas_id to CanvasContent for active canvases
    active_canvases: HashMap<String, CanvasContent>,
}

impl CanvasState {
    /// Create a new empty CanvasState
    pub fn new() -> Self {
        Self {
            active_canvases: HashMap::new(),
        }
    }

    /// Create a new canvas or update an existing one
    pub fn create_or_update(&mut self, canvas_id: String, content: CanvasContent) {
        self.active_canvases.insert(canvas_id, content);
    }

    /// Destroy a canvas, removing it from active canvases
    /// Returns true if the canvas was found and removed, false otherwise
    pub fn destroy(&mut self, canvas_id: &str) -> bool {
        self.active_canvases.remove(canvas_id).is_some()
    }

    /// Get the content for a canvas by ID
    pub fn get(&self, canvas_id: &str) -> Option<&CanvasContent> {
        self.active_canvases.get(canvas_id)
    }

    /// Check if a canvas exists
    pub fn exists(&self, canvas_id: &str) -> bool {
        self.active_canvases.contains_key(canvas_id)
    }

    /// Get the count of active canvases
    pub fn len(&self) -> usize {
        self.active_canvases.len()
    }

    /// Check if there are no active canvases
    pub fn is_empty(&self) -> bool {
        self.active_canvases.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_state_new_is_empty() {
        let state = CanvasState::new();
        assert!(state.is_empty());
        assert_eq!(state.len(), 0);
    }

    #[test]
    fn test_canvas_state_create_or_update() {
        let mut state = CanvasState::new();
        let content = CanvasContent {
            html: "<div>Hello</div>".to_string(),
            css: Some("div { color: red; }".to_string()),
            js: None,
            title: None,
        };

        state.create_or_update("canvas-1".to_string(), content.clone());

        assert!(!state.is_empty());
        assert_eq!(state.len(), 1);
        assert!(state.get("canvas-1").is_some());
    }

    #[test]
    fn test_canvas_state_destroy() {
        let mut state = CanvasState::new();
        let content = CanvasContent {
            html: "<div>Hello</div>".to_string(),
            css: None,
            js: None,
            title: None,
        };

        state.create_or_update("canvas-1".to_string(), content);
        assert_eq!(state.len(), 1);

        // Destroy non-existent canvas returns false
        assert!(!state.destroy("nonexistent"));

        // Destroy existing canvas returns true
        assert!(state.destroy("canvas-1"));
        assert!(state.is_empty());
        assert!(state.get("canvas-1").is_none());
    }

    #[test]
    fn test_canvas_state_get() {
        let mut state = CanvasState::new();
        let content = CanvasContent {
            html: "<div>Hello</div>".to_string(),
            css: None,
            js: None,
            title: None,
        };

        state.create_or_update("canvas-1".to_string(), content);

        assert!(state.get("canvas-1").is_some());
        assert!(state.get("nonexistent").is_none());
    }

    #[test]
    fn test_canvas_state_exists() {
        let mut state = CanvasState::new();
        let content = CanvasContent {
            html: "<div>Hello</div>".to_string(),
            css: None,
            js: None,
            title: None,
        };

        state.create_or_update("canvas-1".to_string(), content);

        assert!(state.exists("canvas-1"));
        assert!(!state.exists("nonexistent"));
    }

    #[test]
    fn test_canvas_state_update_replaces() {
        let mut state = CanvasState::new();
        let content1 = CanvasContent {
            html: "<div>Original</div>".to_string(),
            css: None,
            js: None,
            title: None,
        };
        let content2 = CanvasContent {
            html: "<div>Updated</div>".to_string(),
            css: Some("div { color: blue; }".to_string()),
            js: None,
            title: None,
        };

        state.create_or_update("canvas-1".to_string(), content1);
        state.create_or_update("canvas-1".to_string(), content2.clone());

        // Update should replace, not add
        assert_eq!(state.len(), 1);
        let retrieved = state.get("canvas-1").unwrap();
        assert_eq!(retrieved.html, "<div>Updated</div>");
    }

    #[test]
    fn test_canvas_content_serialization() {
        let content = CanvasContent {
            html: "<div>Hello</div>".to_string(),
            css: Some("div { color: red; }".to_string()),
            js: Some("console.log('hi');".to_string()),
            title: Some("Test Canvas".to_string()),
        };

        let json = serde_json::to_string(&content).unwrap();
        assert!(json.contains("\"html\""));
        assert!(json.contains("\"css\""));
        assert!(json.contains("\"js\""));
        assert!(json.contains("\"title\""));
    }

    #[test]
    fn test_canvas_content_minimal_serialization() {
        let content = CanvasContent {
            html: "<div>Hello</div>".to_string(),
            css: None,
            js: None,
            title: None,
        };

        let json = serde_json::to_string(&content).unwrap();
        assert!(json.contains("\"html\""));
        // Optional fields should be omitted
        assert!(!json.contains("\"css\""));
        assert!(!json.contains("\"js\""));
        assert!(!json.contains("\"title\""));
    }

    #[test]
    fn test_canvas_action_serialization() {
        let create: CanvasAction = CanvasAction::Create;
        let update: CanvasAction = CanvasAction::Update;
        let destroy: CanvasAction = CanvasAction::Destroy;

        assert_eq!(serde_json::to_string(&create).unwrap(), "\"create\"");
        assert_eq!(serde_json::to_string(&update).unwrap(), "\"update\"");
        assert_eq!(serde_json::to_string(&destroy).unwrap(), "\"destroy\"");
    }

    #[test]
    fn test_canvas_interact_params_deserialization() {
        let json = r#"{"canvas_id":"canvas-1","event_type":"click","element_id":"btn-submit","data":{"foo":"bar"}}"#;
        let params: CanvasInteractParams = serde_json::from_str(json).unwrap();

        assert_eq!(params.canvas_id, "canvas-1");
        assert_eq!(params.event_type, "click");
        assert_eq!(params.element_id, Some("btn-submit".to_string()));
        assert!(params.data.is_some());
        assert_eq!(params.data.unwrap()["foo"], "bar");
    }

    #[test]
    fn test_canvas_interact_params_minimal() {
        let json = r#"{"canvas_id":"canvas-1","event_type":"click"}"#;
        let params: CanvasInteractParams = serde_json::from_str(json).unwrap();

        assert_eq!(params.canvas_id, "canvas-1");
        assert_eq!(params.event_type, "click");
        assert!(params.element_id.is_none());
        assert!(params.data.is_none());
    }

    #[test]
    fn test_canvas_update_params_serialization() {
        let content = CanvasContent {
            html: "<div>Hello</div>".to_string(),
            css: None,
            js: None,
            title: None,
        };
        let params = CanvasUpdateParams {
            canvas_id: "canvas-1".to_string(),
            action: CanvasAction::Create,
            content: Some(content),
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"canvas_id\":\"canvas-1\""));
        assert!(json.contains("\"action\":\"create\""));
        assert!(json.contains("\"content\""));
    }

    #[test]
    fn test_canvas_interact_result_serialization() {
        let result = CanvasInteractResult { received: true };
        let json = serde_json::to_string(&result).unwrap();
        assert_eq!(json, "{\"received\":true}");
    }
}
