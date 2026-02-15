# 0015 — Sandbox & Security

**Master Plan Reference:** Section 3.15 — Sandbox & Security  
**Phase:** 7 (Production)  
**Dependencies:** 0003 (Gateway), 0005 (Tools), 0006 (Agent Engine)

---

## Objective

Implement per-agent sandbox isolation and the security framework including
container execution, workspace access controls, tool policies, execution
approval workflows, and authentication scopes.

---

## Deliverables

### 1. Sandbox Configuration

```rust
pub struct SandboxConfig {
    pub enabled: bool,
    pub runtime: SandboxRuntime,    // Docker or Podman
    pub image: String,               // Container image
    pub workspace_access: WorkspaceAccess,
    pub network_access: bool,
    pub memory_limit: Option<String>, // e.g., "512m"
    pub cpu_limit: Option<f64>,      // e.g., 1.0
    pub timeout: Duration,
}

pub enum SandboxRuntime {
    Docker,
    Podman,
}

pub enum WorkspaceAccess {
    None,
    ReadOnly,
    ReadWrite,
}
```

### 2. Container Execution

- Create and manage containers per-agent
- Mount workspace directory with appropriate permissions
- Execute tool commands inside container
- Capture stdout/stderr
- Enforce timeout and resource limits
- Container cleanup after agent run

**Implementation:**
```rust
pub struct SandboxExecutor {
    runtime: SandboxRuntime,
}

impl SandboxExecutor {
    pub async fn execute(
        &self,
        config: &SandboxConfig,
        command: &str,
        working_dir: &Path,
    ) -> Result<ExecutionResult>;

    pub async fn create_container(&self, config: &SandboxConfig) -> Result<ContainerId>;
    pub async fn destroy_container(&self, id: &ContainerId) -> Result<()>;
}
```

### 3. Execution Approval Workflow

For dangerous operations requiring human confirmation:

```rust
pub struct ApprovalRequest {
    pub id: String,
    pub agent_id: String,
    pub session_key: String,
    pub operation: String,         // e.g., "bash: rm -rf /tmp"
    pub risk_level: RiskLevel,
    pub timestamp: DateTime<Utc>,
    pub timeout: Duration,
}

pub enum ApprovalDecision {
    Approved,
    Denied { reason: String },
    Timeout,
}
```

**Flow:**
1. Tool execution requests approval
2. Approval request sent to connected operators via WebSocket
3. Operator approves/denies via UI or CLI
4. Tool proceeds or reports denial
5. Auto-approve rules for known-safe commands

### 4. Tool Policy Enforcement

```rust
pub struct ToolPolicyEngine {
    global_policy: ToolPolicy,
    agent_policies: HashMap<String, ToolPolicy>,
}

impl ToolPolicyEngine {
    pub fn is_allowed(&self, agent_id: &str, tool_name: &str) -> bool;
    pub fn get_effective_policy(&self, agent_id: &str) -> ToolPolicy;
}
```

### 5. Authentication & Authorization

Enhance the gateway auth system:

- **API token generation** and rotation
- **Password hashing** (argon2 or bcrypt)
- **Scope-based authorization** for all RPC methods
- **Device token management** (issue, revoke, refresh)
- **Session token management** for web UI

### 6. Security Hardening

- Non-root execution in containers
- Loopback-only binding by default
- HTTPS enforcement option
- Request size limits
- Input sanitization for all user inputs
- Secrets masking in logs
- Config file permission checks

### 7. Audit Logging

- Log all security-relevant events
- Authentication attempts (success/failure)
- Authorization decisions
- Tool executions (especially in sandbox)
- Approval workflow events
- Config changes

---

## Acceptance Criteria

- [ ] Sandbox containers create/execute/destroy correctly
- [ ] Workspace access controls enforce read/write/none
- [ ] Resource limits (memory, CPU, timeout) are enforced
- [ ] Approval workflow sends requests and processes decisions
- [ ] Auto-approve rules work for safe commands
- [ ] Tool policies block denied tools
- [ ] Auth tokens generate and validate correctly
- [ ] Scope-based authorization enforces access control
- [ ] Non-root execution works in containers
- [ ] Audit logging captures security events
- [ ] Integration tests verify sandbox isolation
- [ ] Security tests verify auth and authorization
