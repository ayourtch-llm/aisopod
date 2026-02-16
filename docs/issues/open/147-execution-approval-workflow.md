# Issue 147: Enhance Execution Approval Workflow

## Summary
Extend the approval workflow system from Issue 059 with risk-level auto-classification, auto-approve rules for known-safe commands, configurable approval timeouts, and WebSocket notifications to connected operators.

## Location
- Crate: `aisopod-tools`
- Files:
  - `crates/aisopod-tools/src/approval.rs` (extend)
  - `crates/aisopod-gateway/src/rpc/handlers/approval.rs` (extend)

## Current Behavior
Issue 059 introduced a basic `ApprovalRequest` type and `ApprovalHandler` trait with async approval waiting. However, there is no automatic risk classification, no auto-approve mechanism for safe commands, and no WebSocket push notification to operators when approval is needed.

## Expected Behavior
After this issue is completed:
- A `RiskLevel` enum (`Low`, `Medium`, `High`, `Critical`) classifies each operation automatically based on its content.
- Known-safe commands (e.g., `ls`, `cat`, `echo`, `pwd`, `whoami`, `git status`, `git log`, `git diff`) are auto-approved at the `Low` risk level.
- Approval timeout is configurable per risk level (e.g., 30s for Low, 120s for Critical).
- When an approval request is created, a WebSocket notification is broadcast to all connected operators.
- Operators can approve or deny via the RPC API or Web UI.

## Impact
Without risk-based approval, every dangerous operation either blocks waiting for human input (slowing agents) or is silently allowed (creating security risk). This feature balances safety with productivity by auto-approving safe operations and escalating risky ones.

## Suggested Implementation

1. **Define the `RiskLevel` enum** in `crates/aisopod-tools/src/approval.rs`:
   ```rust
   use serde::{Deserialize, Serialize};

   #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
   #[serde(rename_all = "lowercase")]
   pub enum RiskLevel {
       Low,
       Medium,
       High,
       Critical,
   }
   ```

2. **Implement auto-classification:**
   ```rust
   impl RiskLevel {
       pub fn classify(command: &str) -> Self {
           let cmd = command.trim();
           let base_cmd = cmd.split_whitespace().next().unwrap_or("");

           const SAFE_COMMANDS: &[&str] = &[
               "ls", "cat", "echo", "pwd", "whoami", "date", "uname",
               "wc", "head", "tail", "grep", "find", "which", "env",
               "printenv", "id", "hostname",
           ];

           const SAFE_PREFIXES: &[&str] = &[
               "git status", "git log", "git diff", "git branch",
               "git show", "git remote",
           ];

           // Check safe prefixes first (multi-word commands)
           for prefix in SAFE_PREFIXES {
               if cmd.starts_with(prefix) {
                   return Self::Low;
               }
           }

           if SAFE_COMMANDS.contains(&base_cmd) {
               return Self::Low;
           }

           // Dangerous patterns
           if cmd.contains("rm -rf")
               || cmd.contains("mkfs")
               || cmd.contains("dd if=")
               || cmd.contains("> /dev/")
               || cmd.contains("chmod 777")
           {
               return Self::Critical;
           }

           if base_cmd == "rm"
               || base_cmd == "mv"
               || base_cmd == "chmod"
               || base_cmd == "chown"
               || cmd.contains("sudo")
               || cmd.contains("curl") && cmd.contains("|")
           {
               return Self::High;
           }

           Self::Medium
       }
   }
   ```

3. **Add auto-approve logic:**
   ```rust
   pub struct AutoApproveRules {
       pub max_auto_approve_level: RiskLevel,
   }

   impl Default for AutoApproveRules {
       fn default() -> Self {
           Self {
               max_auto_approve_level: RiskLevel::Low,
           }
       }
   }

   impl AutoApproveRules {
       pub fn should_auto_approve(&self, risk_level: &RiskLevel) -> bool {
           risk_level <= &self.max_auto_approve_level
       }
   }
   ```

4. **Add configurable timeouts per risk level:**
   ```rust
   use std::time::Duration;

   impl RiskLevel {
       pub fn default_timeout(&self) -> Duration {
           match self {
               Self::Low => Duration::from_secs(30),
               Self::Medium => Duration::from_secs(60),
               Self::High => Duration::from_secs(120),
               Self::Critical => Duration::from_secs(300),
           }
       }
   }
   ```

5. **Broadcast approval requests via WebSocket** (integrate with Issue 034's event system):
   ```rust
   // In the approval handler, when a new request is created:
   use aisopod_gateway::broadcast::EventBroadcaster;

   pub async fn request_approval(
       &self,
       request: ApprovalRequest,
       broadcaster: &EventBroadcaster,
   ) -> ApprovalDecision {
       let risk_level = RiskLevel::classify(&request.operation);

       if self.auto_approve_rules.should_auto_approve(&risk_level) {
           tracing::info!(
               operation = %request.operation,
               risk_level = ?risk_level,
               "Auto-approved low-risk operation"
           );
           return ApprovalDecision::Approved;
       }

       // Broadcast to connected operators
       broadcaster.send(Event::ApprovalRequired {
           id: request.id.clone(),
           agent_id: request.agent_id.clone(),
           operation: request.operation.clone(),
           risk_level: risk_level.clone(),
       });

       // Wait for operator decision with timeout
       let timeout = risk_level.default_timeout();
       match tokio::time::timeout(timeout, self.wait_for_decision(&request.id)).await {
           Ok(decision) => decision,
           Err(_) => ApprovalDecision::Timeout,
       }
   }
   ```

6. **Add unit tests:**
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_safe_commands_classify_as_low() {
           assert_eq!(RiskLevel::classify("ls -la"), RiskLevel::Low);
           assert_eq!(RiskLevel::classify("cat foo.txt"), RiskLevel::Low);
           assert_eq!(RiskLevel::classify("git status"), RiskLevel::Low);
       }

       #[test]
       fn test_dangerous_commands_classify_as_critical() {
           assert_eq!(RiskLevel::classify("rm -rf /"), RiskLevel::Critical);
           assert_eq!(RiskLevel::classify("dd if=/dev/zero of=/dev/sda"), RiskLevel::Critical);
       }

       #[test]
       fn test_auto_approve_low_risk() {
           let rules = AutoApproveRules::default();
           assert!(rules.should_auto_approve(&RiskLevel::Low));
           assert!(!rules.should_auto_approve(&RiskLevel::Medium));
       }
   }
   ```

## Dependencies
- Issue 059 (approval workflow — `ApprovalRequest`, `ApprovalHandler` trait)
- Issue 034 (event broadcasting — `EventBroadcaster` for WebSocket notifications)

## Acceptance Criteria
- [ ] `RiskLevel` enum with `Low`, `Medium`, `High`, `Critical` variants
- [ ] Auto-classification correctly identifies safe commands as `Low` and dangerous commands as `High`/`Critical`
- [ ] Auto-approve rules skip the approval flow for operations at or below the configured risk level
- [ ] Approval timeout is configurable per risk level
- [ ] WebSocket notification is sent to operators when approval is required
- [ ] Operators can approve/deny via RPC API
- [ ] Unit tests cover classification, auto-approve, and timeout logic

---
*Created: 2026-02-15*
