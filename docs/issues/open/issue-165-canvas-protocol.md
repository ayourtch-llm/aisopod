# Issue 165: Implement Canvas Protocol for Interactive UI

## Summary
Implement the `canvas.*` RPC methods that allow the server to push rich interactive UI content (HTML/CSS/JS) to connected clients and receive user interaction events back from those clients.

## Location
- Crate: `aisopod-gateway`
- File: `crates/aisopod-gateway/src/rpc/canvas.rs`

## Current Behavior
No mechanism exists for the server to render interactive UI on client devices. Agent responses are limited to text-based chat messages.

## Expected Behavior
1. **`canvas.update`** (server → client) — Sends HTML/CSS/JS content to a client for rendering in a canvas container identified by `canvas_id`.
2. **`canvas.interact`** (client → server) — The client reports user interactions (clicks, input changes, form submissions) that occurred within a canvas.
3. **Canvas lifecycle** — Canvases can be created, updated, and destroyed. The server tracks active canvases per connection.

## Impact
The canvas protocol enables agents to present rich, interactive experiences beyond plain text — forms, visualizations, maps, approval workflows, and more. This is a key differentiator for the app protocol.

## Suggested Implementation
1. Create `crates/aisopod-gateway/src/rpc/canvas.rs`.
2. Define the canvas types:
   ```rust
   use serde::{Deserialize, Serialize};
   use serde_json::Value;

   #[derive(Debug, Serialize)]
   pub struct CanvasUpdateParams {
       pub canvas_id: String,
       pub action: CanvasAction,
       pub content: Option<CanvasContent>,
   }

   #[derive(Debug, Serialize)]
   pub enum CanvasAction {
       Create,
       Update,
       Destroy,
   }

   #[derive(Debug, Serialize)]
   pub struct CanvasContent {
       pub html: String,
       pub css: Option<String>,
       pub js: Option<String>,
       pub title: Option<String>,
   }

   #[derive(Debug, Deserialize)]
   pub struct CanvasInteractParams {
       pub canvas_id: String,
       pub event_type: String,        // "click", "input", "submit", "custom"
       pub element_id: Option<String>,
       pub data: Option<Value>,       // Event-specific payload
   }

   #[derive(Debug, Serialize)]
   pub struct CanvasInteractResult {
       pub received: bool,
   }
   ```
3. Implement canvas state tracking per connection:
   ```rust
   use std::collections::HashMap;

   pub struct CanvasState {
       active_canvases: HashMap<String, CanvasContent>,
   }

   impl CanvasState {
       pub fn new() -> Self {
           Self {
               active_canvases: HashMap::new(),
           }
       }

       pub fn create_or_update(&mut self, canvas_id: String, content: CanvasContent) {
           self.active_canvases.insert(canvas_id, content);
       }

       pub fn destroy(&mut self, canvas_id: &str) -> bool {
           self.active_canvases.remove(canvas_id).is_some()
       }

       pub fn get(&self, canvas_id: &str) -> Option<&CanvasContent> {
           self.active_canvases.get(canvas_id)
       }
   }
   ```
4. Implement `canvas.update` as a server-initiated message:
   - The server (via an agent action) constructs a `CanvasUpdateParams` and sends it as a JSON-RPC notification to the target client's WebSocket.
   - For `Create` and `Update` actions, `content` is required.
   - For `Destroy`, only `canvas_id` is needed.
5. Implement `canvas.interact` handler:
   ```rust
   pub async fn handle_canvas_interact(
       connection: &mut ConnectionState,
       params: CanvasInteractParams,
   ) -> Result<CanvasInteractResult, RpcError> {
       // Verify the canvas_id exists in this connection's active canvases
       if !connection.canvas_state.get(&params.canvas_id).is_some() {
           return Err(RpcError::invalid_params("Unknown canvas_id"));
       }

       // Forward the interaction event to the agent/handler that owns this canvas
       connection
           .event_sender
           .send(Event::CanvasInteraction(params))
           .await
           .map_err(|_| RpcError::internal("Failed to forward interaction"))?;

       Ok(CanvasInteractResult { received: true })
   }
   ```
6. Register `canvas.interact` with the RPC router. `canvas.update` is dispatched directly by server logic, not by client RPC calls.

## Dependencies
- Issue 030 (RPC router for method registration)
- Issue 034 (event broadcasting for forwarding interactions)

## Acceptance Criteria
- [ ] Server can send `canvas.update` with Create/Update/Destroy actions to clients
- [ ] Clients can send `canvas.interact` events back to the server
- [ ] Canvas state is tracked per connection (active canvases map)
- [ ] Destroying a canvas removes it from state
- [ ] Interactions for unknown canvas IDs return an error
- [ ] Unit tests cover canvas lifecycle and interaction forwarding

---
*Created: 2026-02-15*
