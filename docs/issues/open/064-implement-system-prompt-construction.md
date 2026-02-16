# Issue 064: Implement System Prompt Construction

## Summary
Implement the system prompt builder that assembles a complete system prompt from multiple sources: base agent config prompt, dynamic context (date/time, workspace info), tool descriptions, skill instructions, and memory context.

## Location
- Crate: `aisopod-agent`
- File: `crates/aisopod-agent/src/prompt.rs`

## Current Behavior
No system prompt construction logic exists. There is no mechanism to combine the various prompt components into a coherent system prompt for the model.

## Expected Behavior
After this issue is completed:
- A `SystemPromptBuilder` or `build_system_prompt()` function assembles the full system prompt.
- The base prompt from the agent configuration is included as the foundation.
- Dynamic context (current date/time, workspace path, channel info) is injected.
- Tool descriptions for all available tools are appended.
- Skill instructions from the agent configuration are merged in.
- Memory context (retrieved from memory store, if available) is injected.
- The final prompt is a single string ready to be sent to the model provider.

## Impact
The system prompt defines the agent's personality, capabilities, and context. A well-constructed prompt is critical for correct agent behavior. Every agent execution depends on this component to inform the model about its tools, skills, and environment.

## Suggested Implementation
1. **Create `crates/aisopod-agent/src/prompt.rs`:**
   - Define a `SystemPromptBuilder` struct:
     ```rust
     pub struct SystemPromptBuilder {
         sections: Vec<PromptSection>,
     }

     struct PromptSection {
         label: String,
         content: String,
     }
     ```
   - Implement `SystemPromptBuilder::new()` → creates an empty builder.
   - Implement `with_base_prompt(prompt: &str)` → adds the agent's base system prompt.
   - Implement `with_dynamic_context()` → adds current date/time, workspace info, etc.
   - Implement `with_tool_descriptions(tools: &[ToolSchema])` → adds formatted tool descriptions.
   - Implement `with_skill_instructions(skills: &[String])` → appends skill instruction blocks.
   - Implement `with_memory_context(memory: &str)` → adds retrieved memory/context.
   - Implement `build() -> String` → concatenates all sections with appropriate separators.

2. **Formatting details:**
   - Use clear section headers (e.g., `## Tools`, `## Skills`) so the model can parse the prompt structure.
   - Tool descriptions should include the tool name, description, and parameter schema.
   - Dynamic context should include the current UTC timestamp and any workspace metadata.

3. **Update `crates/aisopod-agent/src/lib.rs`:**
   - Add `pub mod prompt;`.

4. **Add unit tests:**
   - Test building a prompt with only a base prompt.
   - Test building a prompt with all sections populated.
   - Test that tool descriptions are correctly formatted.
   - Test that dynamic context includes a valid timestamp.
   - Test that the build output contains all expected sections.

5. **Verify** — Run `cargo test -p aisopod-agent`.

## Dependencies
- Issue 062 (Agent types and AgentRunner skeleton)
- Issue 063 (Agent resolution — provides agent config with base prompt and skills)

## Acceptance Criteria
- [ ] `SystemPromptBuilder` or equivalent function exists and compiles.
- [ ] Base agent prompt, dynamic context, tool descriptions, skill instructions, and memory context are all included.
- [ ] Sections are clearly delineated in the output string.
- [ ] Unit tests verify correct assembly of all prompt components.
- [ ] `cargo check -p aisopod-agent` succeeds without errors.

---
*Created: 2026-02-15*
