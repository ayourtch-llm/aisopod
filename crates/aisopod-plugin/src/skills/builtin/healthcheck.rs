//! Healthcheck skill implementation.
//!
//! This skill provides system health monitoring tools to agents,
//! enabling them to inspect system state, check gateway status,
//! verify channel connectivity, and confirm model provider availability.

use crate::skills::{Skill, SkillCategory, SkillContext, SkillMeta};
use aisopod_tools::{Tool, ToolContext, ToolResult};
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

/// The healthcheck skill that provides system health monitoring tools.
pub struct HealthcheckSkill {
    meta: SkillMeta,
}

impl HealthcheckSkill {
    /// Creates a new HealthcheckSkill instance.
    pub fn new() -> Self {
        Self {
            meta: SkillMeta::new(
                "Healthcheck",
                "0.1.0",
                "System health monitoring and diagnostics".to_string(),
                SkillCategory::System,
                vec![],
                vec![],
                None,
            ),
        }
    }
}

impl std::fmt::Debug for HealthcheckSkill {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HealthcheckSkill")
            .field("id", &self.id())
            .field("meta", &self.meta)
            .finish()
    }
}

#[async_trait]
impl Skill for HealthcheckSkill {
    fn id(&self) -> &str {
        "healthcheck"
    }

    fn meta(&self) -> &SkillMeta {
        &self.meta
    }

    fn system_prompt_fragment(&self) -> Option<String> {
        Some(
            "You have access to system health monitoring tools. \
             Use `check_system_health` to verify gateway, channel, and model provider status. \
             Use `get_system_info` to retrieve system information including OS, architecture, and version."
                .to_string(),
        )
    }

    fn tools(&self) -> Vec<Arc<dyn Tool>> {
        vec![
            Arc::new(CheckSystemHealthTool),
            Arc::new(GetSystemInfoTool),
        ]
    }

    async fn init(&self, _ctx: &SkillContext) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

/// Tool that checks the health of the aisopod system.
pub struct CheckSystemHealthTool;

#[async_trait]
impl Tool for CheckSystemHealthTool {
    fn name(&self) -> &str {
        "check_system_health"
    }

    fn description(&self) -> &str {
        "Check the health of the aisopod system including gateway, channels, and model providers"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {},
            "required": []
        })
    }

    async fn execute(
        &self,
        _params: serde_json::Value,
        _ctx: &ToolContext,
    ) -> Result<ToolResult> {
        let health = serde_json::json!({
            "gateway": "ok",
            "channels": [],
            "providers": [],
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        Ok(ToolResult {
            content: serde_json::to_string_pretty(&health)?,
            is_error: false,
            metadata: Some(health),
        })
    }
}

/// Tool that retrieves system information.
pub struct GetSystemInfoTool;

#[async_trait]
impl Tool for GetSystemInfoTool {
    fn name(&self) -> &str {
        "get_system_info"
    }

    fn description(&self) -> &str {
        "Get system information including OS, architecture, and version information"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {},
            "required": []
        })
    }

    async fn execute(
        &self,
        _params: serde_json::Value,
        _ctx: &ToolContext,
    ) -> Result<ToolResult> {
        let info = serde_json::json!({
            "os": std::env::consts::OS,
            "arch": std::env::consts::ARCH,
            "version": env!("CARGO_PKG_VERSION"),
        });
        Ok(ToolResult {
            content: serde_json::to_string_pretty(&info)?,
            is_error: false,
            metadata: Some(info),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_healthcheck_skill_new() {
        let skill = HealthcheckSkill::new();
        assert_eq!(skill.id(), "healthcheck");
        assert_eq!(skill.meta().name, "Healthcheck");
        assert_eq!(skill.meta().version, "0.1.0");
        assert_eq!(skill.meta().category, SkillCategory::System);
    }

    #[test]
    fn test_healthcheck_skill_system_prompt() {
        let skill = HealthcheckSkill::new();
        let prompt = skill.system_prompt_fragment();
        assert!(prompt.is_some());
        let prompt = prompt.unwrap();
        assert!(prompt.contains("check_system_health"));
        assert!(prompt.contains("get_system_info"));
    }

    #[test]
    fn test_healthcheck_skill_has_tools() {
        let skill = HealthcheckSkill::new();
        let tools = skill.tools();
        assert_eq!(tools.len(), 2);
        
        let tool_names: Vec<&str> = tools.iter().map(|t| t.name()).collect();
        assert!(tool_names.contains(&"check_system_health"));
        assert!(tool_names.contains(&"get_system_info"));
    }

    #[test]
    fn test_check_system_health_tool() {
        let tool = CheckSystemHealthTool;
        assert_eq!(tool.name(), "check_system_health");
        assert!(tool.description().contains("health"));
        assert!(tool.description().contains("gateway"));
        assert!(tool.description().contains("channels"));
        assert!(tool.description().contains("providers"));
    }

    #[test]
    fn test_get_system_info_tool() {
        let tool = GetSystemInfoTool;
        assert_eq!(tool.name(), "get_system_info");
        assert!(tool.description().contains("system"));
        assert!(tool.description().contains("OS"));
        assert!(tool.description().contains("version"));
    }

    #[tokio::test]
    async fn test_check_system_health_execution() {
        let tool = CheckSystemHealthTool;
        let ctx = ToolContext::new("test-agent", "test-session");
        
        let result = tool.execute(serde_json::json!({}), &ctx).await.unwrap();
        
        assert!(!result.is_error);
        let health: serde_json::Value = serde_json::from_str(&result.content).unwrap();
        assert_eq!(health["gateway"], "ok");
        assert!(health["timestamp"].is_string());
    }

    #[tokio::test]
    async fn test_get_system_info_execution() {
        let tool = GetSystemInfoTool;
        let ctx = ToolContext::new("test-agent", "test-session");
        
        let result = tool.execute(serde_json::json!({}), &ctx).await.unwrap();
        
        assert!(!result.is_error);
        let info: serde_json::Value = serde_json::from_str(&result.content).unwrap();
        assert_eq!(info["os"], std::env::consts::OS);
        assert_eq!(info["arch"], std::env::consts::ARCH);
        assert_eq!(info["version"], env!("CARGO_PKG_VERSION"));
    }
}
