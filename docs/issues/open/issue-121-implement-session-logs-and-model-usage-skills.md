# Issue 121: Implement Session-Logs and Model-Usage Skills (Tier 1)

## Summary
Implement two built-in Tier 1 skills: `session-logs` for accessing session log history, and `model-usage` for tracking and reporting token consumption and model usage statistics. Both skills provide tools and system-prompt fragments to agents.

## Location
- Crate: `aisopod-plugin`
- File: `crates/aisopod-plugin/src/skills/builtin/session_logs.rs`, `crates/aisopod-plugin/src/skills/builtin/model_usage.rs`

## Current Behavior
Message storage exists (Issue 076) and usage tracking exists (Issue 070), but agents have no built-in way to query session logs or inspect their own token consumption. There are no skills wrapping these subsystems.

## Expected Behavior
After this issue is completed:
- A `SessionLogsSkill` provides a `get_session_logs` tool that retrieves message history for the current or specified session.
- A `ModelUsageSkill` provides `get_usage_summary` and `get_token_consumption` tools for reporting model usage statistics.
- Both skills include system-prompt fragments describing their capabilities.
- Both skills are feature-gated behind `skill-session-logs` and `skill-model-usage` respectively.

## Impact
These skills give agents self-awareness about their conversation history and resource consumption. This enables agents to make informed decisions about context management and provides users with transparency into system usage.

## Suggested Implementation
1. **Implement `SessionLogsSkill` in `session_logs.rs`:**
   ```rust
   pub struct SessionLogsSkill {
       meta: SkillMeta,
   }

   impl SessionLogsSkill {
       pub fn new() -> Self {
           Self {
               meta: SkillMeta {
                   name: "Session Logs".to_string(),
                   description: "Access session message history and logs".to_string(),
                   version: "0.1.0".to_string(),
                   category: SkillCategory::System,
                   required_env_vars: vec![],
                   required_binaries: vec![],
                   platform: None,
               },
           }
       }
   }

   #[async_trait]
   impl Skill for SessionLogsSkill {
       fn id(&self) -> &str { "session-logs" }
       fn meta(&self) -> &SkillMeta { &self.meta }

       fn system_prompt_fragment(&self) -> Option<String> {
           Some(
               "You have access to session log history. \
                Use `get_session_logs` to retrieve past messages from the current or a specified session.".to_string()
           )
       }

       fn tools(&self) -> Vec<Arc<dyn Tool>> {
           vec![Arc::new(GetSessionLogsTool)]
       }

       async fn init(&self, _ctx: &SkillContext) -> Result<(), Box<dyn std::error::Error>> {
           Ok(())
       }
   }
   ```

2. **Implement `GetSessionLogsTool`:**
   ```rust
   pub struct GetSessionLogsTool;

   #[async_trait]
   impl Tool for GetSessionLogsTool {
       fn name(&self) -> &str { "get_session_logs" }

       fn description(&self) -> &str {
           "Retrieve message history for the current or a specified session"
       }

       fn parameters_schema(&self) -> serde_json::Value {
           serde_json::json!({
               "type": "object",
               "properties": {
                   "session_key": {
                       "type": "string",
                       "description": "Session key to query. Defaults to the current session if omitted."
                   },
                   "limit": {
                       "type": "integer",
                       "description": "Maximum number of messages to return. Defaults to 50.",
                       "default": 50
                   }
               },
               "required": []
           })
       }

       async fn execute(
           &self,
           params: serde_json::Value,
           ctx: &ToolContext,
       ) -> Result<ToolResult> {
           let session_key = params.get("session_key")
               .and_then(|v| v.as_str())
               .unwrap_or(&ctx.session_key);
           let limit = params.get("limit")
               .and_then(|v| v.as_u64())
               .unwrap_or(50);

           // TODO: Query message storage (Issue 076) for session history
           let result = serde_json::json!({
               "session_key": session_key,
               "limit": limit,
               "messages": [],
           });
           Ok(ToolResult {
               content: serde_json::to_string_pretty(&result)?,
               is_error: false,
               metadata: Some(result),
           })
       }
   }
   ```

3. **Implement `ModelUsageSkill` in `model_usage.rs`:**
   ```rust
   pub struct ModelUsageSkill {
       meta: SkillMeta,
   }

   impl ModelUsageSkill {
       pub fn new() -> Self {
           Self {
               meta: SkillMeta {
                   name: "Model Usage".to_string(),
                   description: "Track and report model usage and token consumption".to_string(),
                   version: "0.1.0".to_string(),
                   category: SkillCategory::System,
                   required_env_vars: vec![],
                   required_binaries: vec![],
                   platform: None,
               },
           }
       }
   }

   #[async_trait]
   impl Skill for ModelUsageSkill {
       fn id(&self) -> &str { "model-usage" }
       fn meta(&self) -> &SkillMeta { &self.meta }

       fn system_prompt_fragment(&self) -> Option<String> {
           Some(
               "You have access to model usage tracking tools. \
                Use `get_usage_summary` for an overview of model usage across sessions. \
                Use `get_token_consumption` for detailed token consumption data.".to_string()
           )
       }

       fn tools(&self) -> Vec<Arc<dyn Tool>> {
           vec![
               Arc::new(GetUsageSummaryTool),
               Arc::new(GetTokenConsumptionTool),
           ]
       }

       async fn init(&self, _ctx: &SkillContext) -> Result<(), Box<dyn std::error::Error>> {
           Ok(())
       }
   }
   ```

4. **Implement `GetUsageSummaryTool`:**
   ```rust
   pub struct GetUsageSummaryTool;

   #[async_trait]
   impl Tool for GetUsageSummaryTool {
       fn name(&self) -> &str { "get_usage_summary" }

       fn description(&self) -> &str {
           "Get a summary of model usage including total requests and token counts"
       }

       fn parameters_schema(&self) -> serde_json::Value {
           serde_json::json!({
               "type": "object",
               "properties": {
                   "since": {
                       "type": "string",
                       "description": "ISO 8601 timestamp to filter usage from. Defaults to last 24 hours."
                   }
               },
               "required": []
           })
       }

       async fn execute(
           &self,
           _params: serde_json::Value,
           _ctx: &ToolContext,
       ) -> Result<ToolResult> {
           // TODO: Query usage tracking (Issue 070) for summary data
           let summary = serde_json::json!({
               "total_requests": 0,
               "total_input_tokens": 0,
               "total_output_tokens": 0,
               "models": {},
           });
           Ok(ToolResult {
               content: serde_json::to_string_pretty(&summary)?,
               is_error: false,
               metadata: Some(summary),
           })
       }
   }
   ```

5. **Implement `GetTokenConsumptionTool`:**
   ```rust
   pub struct GetTokenConsumptionTool;

   #[async_trait]
   impl Tool for GetTokenConsumptionTool {
       fn name(&self) -> &str { "get_token_consumption" }

       fn description(&self) -> &str {
           "Get detailed token consumption data broken down by model and session"
       }

       fn parameters_schema(&self) -> serde_json::Value {
           serde_json::json!({
               "type": "object",
               "properties": {
                   "model": {
                       "type": "string",
                       "description": "Filter by model name"
                   },
                   "session_key": {
                       "type": "string",
                       "description": "Filter by session key"
                   }
               },
               "required": []
           })
       }

       async fn execute(
           &self,
           _params: serde_json::Value,
           _ctx: &ToolContext,
       ) -> Result<ToolResult> {
           // TODO: Query usage tracking (Issue 070) for detailed data
           let data = serde_json::json!({
               "consumption": [],
           });
           Ok(ToolResult {
               content: serde_json::to_string_pretty(&data)?,
               is_error: false,
               metadata: Some(data),
           })
       }
   }
   ```

6. **Feature-gate the modules** — In `skills/builtin/mod.rs`:
   ```rust
   #[cfg(feature = "skill-session-logs")]
   pub mod session_logs;

   #[cfg(feature = "skill-model-usage")]
   pub mod model_usage;
   ```

7. **Verify** — Run `cargo check -p aisopod-plugin --features skill-session-logs,skill-model-usage`.

## Dependencies
- Issue 116 (Skill trait, SkillMeta, and SkillCategory types)
- Issue 117 (SkillRegistry for discovery and lifecycle)
- Issue 076 (Message storage and history retrieval)
- Issue 070 (Usage tracking)

## Acceptance Criteria
- [ ] `SessionLogsSkill` implements the `Skill` trait and provides `get_session_logs` tool
- [ ] `ModelUsageSkill` implements the `Skill` trait and provides `get_usage_summary` and `get_token_consumption` tools
- [ ] Both skills include descriptive system-prompt fragments
- [ ] `get_session_logs` accepts optional `session_key` and `limit` parameters
- [ ] `get_usage_summary` accepts optional `since` parameter
- [ ] `get_token_consumption` accepts optional `model` and `session_key` filters
- [ ] Both skills are feature-gated behind their respective feature flags
- [ ] `cargo check -p aisopod-plugin --features skill-session-logs,skill-model-usage` compiles without errors

---
*Created: 2026-02-15*
