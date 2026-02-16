# Issue 058: Implement Canvas Tool

## Summary
Implement a built-in canvas tool that allows agents to generate visual HTML/CSS/JS output, support interactive canvas rendering, and provide a live update mechanism for dynamic content.

## Location
- Crate: `aisopod-tools`
- File: `crates/aisopod-tools/src/builtins/canvas.rs`

## Current Behavior
No canvas tool exists. Agents have no way to generate visual or interactive output.

## Expected Behavior
After this issue is completed:
- The `CanvasTool` struct implements the `Tool` trait.
- It supports operations via an `operation` parameter: `create`, `update`, and `get`.
- `create` — Generates a new canvas with HTML/CSS/JS content and returns a canvas ID.
- `update` — Updates an existing canvas by ID with new content.
- `get` — Retrieves the current content of a canvas by ID.
- Canvas content is stored and deliverable to the client (via the gateway or a pluggable renderer).

## Impact
The canvas tool enables agents to produce rich visual output — charts, diagrams, interactive widgets, and more — rather than being limited to plain text. This significantly expands the range of agent capabilities.

## Suggested Implementation
1. **Create `canvas.rs`** — Add `crates/aisopod-tools/src/builtins/canvas.rs`.

2. **Define a `CanvasRenderer` trait**:
   ```rust
   #[async_trait]
   pub trait CanvasRenderer: Send + Sync {
       async fn create(&self, content: &str) -> Result<String>; // returns canvas ID
       async fn update(&self, canvas_id: &str, content: &str) -> Result<()>;
       async fn get(&self, canvas_id: &str) -> Result<String>;
   }
   ```

3. **Define `CanvasTool`**:
   ```rust
   pub struct CanvasTool {
       renderer: Arc<dyn CanvasRenderer>,
   }
   ```

4. **Implement `Tool` for `CanvasTool`**:
   - `name()` → `"canvas"`
   - `description()` → `"Generate and manage visual HTML/CSS/JS output"`
   - `parameters_schema()` → JSON Schema with `operation` (enum: `create`, `update`, `get`), `content` (string), and `canvas_id` (string) properties.
   - `execute()`:
     1. Parse the `operation` parameter.
     2. For `create`: require `content`, call `renderer.create()`, return the canvas ID.
     3. For `update`: require `canvas_id` and `content`, call `renderer.update()`.
     4. For `get`: require `canvas_id`, call `renderer.get()`, return the content.

5. **Create an in-memory `CanvasRenderer` implementation** — Store canvases in a `DashMap<String, String>` (or `RwLock<HashMap>`) for testing and initial development.

6. **Register the tool** — Ensure the tool can be registered with the `ToolRegistry`.

7. **Verify** — Run `cargo check -p aisopod-tools`.

## Dependencies
- Issue 049 (Tool trait and core types)
- Issue 050 (Tool registry)

## Acceptance Criteria
- [ ] `CanvasTool` implements the `Tool` trait.
- [ ] `create` operation generates a canvas and returns an ID.
- [ ] `update` operation modifies existing canvas content.
- [ ] `get` operation retrieves canvas content by ID.
- [ ] Canvas content (HTML/CSS/JS) is stored and retrievable.
- [ ] `parameters_schema()` returns a valid JSON Schema.
- [ ] `cargo check -p aisopod-tools` compiles without errors.

---
*Created: 2026-02-15*
