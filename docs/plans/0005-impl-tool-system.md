# 0005 — Tool System

**Master Plan Reference:** Section 3.6 — Tool System  
**Phase:** 2 (Core Runtime)  
**Dependencies:** 0001 (Project Structure), 0002 (Configuration System)

---

## Objective

Implement the agent tool system that allows AI agents to interact with the external
world through a well-defined, extensible tool interface with policy enforcement
and approval workflows.

---

## Deliverables

### 1. Tool Trait (`aisopod-tools`)

Define the core tool abstraction:

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    /// Tool name (used in function calling)
    fn name(&self) -> &str;

    /// Tool description for the AI model
    fn description(&self) -> &str;

    /// JSON Schema for tool parameters
    fn parameters_schema(&self) -> serde_json::Value;

    /// Execute the tool with given parameters
    async fn execute(
        &self,
        params: serde_json::Value,
        ctx: &ToolContext,
    ) -> Result<ToolResult>;
}

pub struct ToolContext {
    pub agent_id: String,
    pub session_key: String,
    pub workspace_path: Option<PathBuf>,
    pub sandbox_config: Option<SandboxConfig>,
    pub approval_handler: Option<Arc<dyn ApprovalHandler>>,
}

pub struct ToolResult {
    pub content: String,
    pub is_error: bool,
    pub metadata: Option<serde_json::Value>,
}
```

### 2. Tool Registry

- Central registry for all available tools
- Dynamic registration (built-in + plugin-contributed)
- Tool lookup by name
- Schema generation for AI model function definitions

### 3. Built-in Tools

Port all tools from OpenClaw's `src/agents/tools/`:

**Bash/Shell Tool:**
- Execute shell commands in configurable environment
- Working directory control
- Timeout enforcement
- Output capture (stdout + stderr)
- Approval workflow for dangerous commands
- Sandbox execution support (Docker/Podman)

**File Operations Tool:**
- Read file contents
- Write/create files
- Search files (glob, grep)
- Directory listing
- File metadata (size, permissions, modification time)
- Workspace path restrictions

**Message Tool:**
- Send messages to channels
- Target specific channel/account/peer
- Support text, media, and structured messages

**Subagent Tool:**
- Spawn child agent within current session
- Pass context and constraints
- Depth limit enforcement
- Allowlist control

**Session Tool:**
- List active sessions
- Send messages to specific sessions
- Patch session metadata
- Session history access

**Cron Tool:**
- Schedule recurring tasks
- List scheduled jobs
- Run job immediately
- Remove scheduled jobs
- Cron expression parsing

**Canvas Tool:**
- Generate visual output (HTML/CSS/JS)
- Interactive canvas rendering
- Live canvas updates

**Channel-Specific Tools:**
- Channel-dependent operations (reactions, thread management, etc.)

### 4. Tool Policy Enforcement

Port the tool allow/deny system from OpenClaw:

```rust
pub struct ToolPolicy {
    pub allow: Option<Vec<String>>,  // Whitelist (if set, only these allowed)
    pub deny: Option<Vec<String>>,   // Blacklist (these are blocked)
}
```

- Per-agent tool policies
- Global tool policies
- Tool filtering before execution
- Policy evaluation with clear deny messages

### 5. Approval Workflow

For dangerous operations (e.g., bash commands):

- Approval request generation
- Async approval waiting (with timeout)
- Auto-approve rules (safe commands)
- Manual approval via WebSocket RPC
- Approval state tracking

### 6. Tool Execution Pipeline

```
Tool call from AI model
  → Validate tool exists in registry
  → Check tool policy (allow/deny)
  → Check approval requirement
  → Request approval if needed (wait/timeout)
  → Execute tool with context
  → Capture result/error
  → Return to AI model
```

### 7. Schema Normalization

- Convert tool definitions to provider-specific format:
  - Anthropic tool format
  - OpenAI function calling format
  - Gemini function declaration format
- Normalize tool results back to internal format

---

## Acceptance Criteria

- [ ] Tool trait is well-defined and extensible
- [ ] Tool registry supports dynamic registration and lookup
- [ ] Bash tool executes commands with timeout and output capture
- [ ] File operations tool handles read/write/search within workspace
- [ ] Message tool delivers to channels
- [ ] Tool policy enforcement blocks denied tools
- [ ] Approval workflow handles request/wait/resolve cycle
- [ ] Tool schemas convert correctly for each provider format
- [ ] Sandbox execution isolates tool operations
- [ ] Unit tests cover all built-in tools
- [ ] Integration tests verify the complete tool execution pipeline
