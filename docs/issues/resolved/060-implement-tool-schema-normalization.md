# Issue 060: Implement Tool Schema Normalization for Providers

## Summary
Implement conversion functions that transform internal tool definitions into the provider-specific formats required by Anthropic, OpenAI, and Google Gemini for function calling / tool use.

## Location
- Crate: `aisopod-tools`
- File: `crates/aisopod-tools/src/schema.rs`

## Current Behavior
No schema normalization exists. Tool definitions cannot be converted to the formats required by different AI providers.

## Expected Behavior
After this issue is completed:
- Internal tool definitions (name, description, parameters JSON Schema) can be converted to Anthropic's tool format.
- Internal tool definitions can be converted to OpenAI's function calling format.
- Internal tool definitions can be converted to Google Gemini's function declaration format.
- Each conversion function produces correctly structured JSON that the respective provider API accepts.

## Impact
Each AI provider has a slightly different format for tool/function definitions. Without normalization, the system would need to hard-code provider-specific formatting in every tool, which is unmaintainable. Centralized normalization keeps tools provider-agnostic.

## Suggested Implementation
1. **Create `schema.rs`** — Add `crates/aisopod-tools/src/schema.rs`.

2. **Define the internal tool definition type** (or reuse from the registry):
   ```rust
   pub struct ToolDefinition {
       pub name: String,
       pub description: String,
       pub parameters: serde_json::Value, // JSON Schema
   }
   ```

3. **Implement `to_anthropic_format()`**:
   ```rust
   pub fn to_anthropic_format(tool: &ToolDefinition) -> serde_json::Value {
       json!({
           "name": tool.name,
           "description": tool.description,
           "input_schema": tool.parameters,
       })
   }
   ```
   Reference: Anthropic uses `input_schema` for the parameters schema.

4. **Implement `to_openai_format()`**:
   ```rust
   pub fn to_openai_format(tool: &ToolDefinition) -> serde_json::Value {
       json!({
           "type": "function",
           "function": {
               "name": tool.name,
               "description": tool.description,
               "parameters": tool.parameters,
           }
       })
   }
   ```
   Reference: OpenAI wraps tools in a `{"type": "function", "function": {...}}` envelope.

5. **Implement `to_gemini_format()`**:
   ```rust
   pub fn to_gemini_format(tool: &ToolDefinition) -> serde_json::Value {
       json!({
           "name": tool.name,
           "description": tool.description,
           "parameters": tool.parameters,
       })
   }
   ```
   Reference: Gemini uses `FunctionDeclaration` with `name`, `description`, and `parameters`.

6. **Implement batch conversion** — Add functions that convert a `Vec<ToolDefinition>` to the appropriate array format for each provider.

7. **Re-export from `lib.rs`**.

8. **Verify** — Run `cargo check -p aisopod-tools`.

## Dependencies
- Issue 049 (Tool trait and core types)
- Issue 050 (Tool registry)
- Issue 038 (ModelProvider trait and core types — for understanding provider-specific formats)

## Acceptance Criteria
- [x] `to_anthropic_format()` produces valid Anthropic tool JSON with `input_schema`.
- [x] `to_openai_format()` produces valid OpenAI function calling JSON with `type: "function"` wrapper.
- [x] `to_gemini_format()` produces valid Gemini function declaration JSON.
- [x] Batch conversion functions handle multiple tools.
- [x] All conversion functions are well-documented.
- [x] `cargo check -p aisopod-tools` compiles without errors.

## Resolution

The `schema.rs` module was already implemented in the codebase with all required functionality:

### Implementation Summary

**File**: `crates/aisopod-tools/src/schema.rs`

**Struct**: `ToolDefinition`
- `name`: The unique tool name
- `description`: Human-readable description
- `parameters`: JSON Schema for tool parameters

**Conversion Functions**:
1. `to_anthropic_format()` - Converts to Anthropic's format with `input_schema`
2. `to_openai_format()` - Converts to OpenAI's format with `type: "function"` wrapper
3. `to_gemini_format()` - Converts to Gemini's `FunctionDeclaration` format

**Batch Conversion Functions**:
1. `to_anthropic_batch()` - Batch conversion for multiple tools
2. `to_openai_batch()` - Batch conversion for multiple tools
3. `to_gemini_batch()` - Batch conversion for multiple tools

**Re-exports**: All conversion functions and `ToolDefinition` are re-exported from `lib.rs`.

### Verification

- `cargo check -p aisopod-tools`: ✅ Passed
- `cargo test -p aisopod-tools`: ✅ 137 tests passed (including 8 schema-specific tests)
- `cargo doc --test -p aisopod-tools`: ✅ 1 passed, 20 ignored (example tests)

All acceptance criteria have been met.

---
*Created: 2026-02-15*
*Resolved: 2026-02-20*
