# Issue 050: Implement Tool Registry

## Summary
Implement a central `ToolRegistry` struct that stores registered tools as `Arc<dyn Tool>` keyed by name, supports dynamic registration from built-in tools and plugins, provides tool lookup by name, and can generate JSON Schema definitions for AI model function calling.

## Location
- Crate: `aisopod-tools`
- File: `crates/aisopod-tools/src/registry.rs`

## Current Behavior
No tool registry exists. There is no mechanism to register, look up, or enumerate tools.

## Expected Behavior
After this issue is completed:
- A `ToolRegistry` struct holds a `HashMap<String, Arc<dyn Tool>>`.
- Tools can be registered at startup (built-in) and at runtime (plugin-contributed).
- Tools can be looked up by name, returning `Option<Arc<dyn Tool>>`.
- The registry can generate a list of tool schemas suitable for AI model function definitions (JSON Schema format).
- Duplicate registration of the same tool name returns an error or overwrites with a warning.

## Impact
The registry is the central access point for all tool operations. The tool execution pipeline, policy enforcement, and schema normalization all depend on it.

## Suggested Implementation
1. **Create `registry.rs`** — Add a new file `crates/aisopod-tools/src/registry.rs`.

2. **Define `ToolRegistry`**:
   ```rust
   pub struct ToolRegistry {
       tools: HashMap<String, Arc<dyn Tool>>,
   }
   ```

3. **Implement `register()`** — Accept an `Arc<dyn Tool>`, extract its `name()`, and insert it into the map. If a tool with the same name already exists, log a warning and overwrite it, or return an error — pick one approach and document it.

4. **Implement `get()`** — Accept a tool name (`&str`) and return `Option<Arc<dyn Tool>>`.

5. **Implement `list()`** — Return an iterator or `Vec` of all registered tool names.

6. **Implement `schemas()`** — Iterate all registered tools and build a `Vec<serde_json::Value>` where each entry is:
   ```json
   {
     "name": "<tool name>",
     "description": "<tool description>",
     "parameters": <parameters_schema()>
   }
   ```
   This is the generic internal format; provider-specific conversion happens in Issue 060.

7. **Implement `remove()`** — Accept a tool name and remove it from the registry. Return `bool` indicating whether it was present.

8. **Re-export `ToolRegistry` from `lib.rs`**.

9. **Verify** — Run `cargo check -p aisopod-tools`.

## Dependencies
- Issue 049 (Tool trait and core types)

## Acceptance Criteria
- [ ] `ToolRegistry` struct is defined and publicly accessible.
- [ ] Tools can be registered with `register()`.
- [ ] Tools can be looked up by name with `get()`.
- [ ] All registered tool names can be listed with `list()`.
- [ ] `schemas()` generates a JSON array of tool definitions.
- [ ] `cargo check -p aisopod-tools` compiles without errors.

---
*Created: 2026-02-15*
