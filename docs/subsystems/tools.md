# Tool System

**Crate:** `aisopod-tools`

## Overview

The tool system gives AI agents the ability to interact with the external world
through a well-defined, policy-enforced interface. It provides a `Tool` trait, a
dynamic registry, built-in tools for common operations, and an approval workflow
for dangerous actions.

## Key Types

- **`Tool` trait** — `name()`, `description()`, `parameters_schema() -> Value`,
  `execute(params, ctx) -> Result<ToolResult>`.
- **`ToolContext`** — Execution context: agent ID, session key, workspace path,
  sandbox config, approval handler.
- **`ToolResult`** — Output: content string, is_error flag, optional metadata.
- **`ToolRegistry`** — Central lookup of all tools (built-in + plugin-contributed);
  supports dynamic registration and schema generation for AI model function defs.
- **`ToolPolicy`** — Per-agent allow/deny lists controlling which tools are available.

## Built-in Tools

| Tool       | Description                                         |
|------------|-----------------------------------------------------|
| `bash`     | Shell execution with timeout, output capture, sandbox support |
| `file`     | Read, write, search, list files within workspace    |
| `message`  | Send text/media messages to channels                |
| `subagent` | Spawn child agents with depth limits                |
| `session`  | List, send to, and patch conversation sessions      |
| `cron`     | Schedule, list, run, and remove recurring tasks     |
| `canvas`   | Generate interactive visual output (HTML/CSS/JS)    |

## Tool Execution Pipeline

1. Validate tool exists in registry
2. Evaluate tool policy (allow/deny lists)
3. Check if approval is required (e.g., bash commands)
4. Request and await approval if needed (with timeout)
5. Execute tool with `ToolContext`
6. Return `ToolResult` to the model

## Approval Workflow

Dangerous operations (primarily bash) go through an async approval flow:
- Auto-approve rules match safe command patterns
- Otherwise, an approval request is broadcast via WebSocket RPC
- The agent blocks until an operator resolves the request or the timeout expires

## Schema Normalization

Tool definitions are converted to provider-specific formats (Anthropic tool format,
OpenAI function calling format, Gemini function declarations) at the boundary between
the agent engine and the provider.

## Dependencies

- **aisopod-config** — `ToolsConfig`, per-agent tool policy definitions.
- **aisopod-channel** — `OutboundAdapter` used by the message tool.
- **aisopod-session** — Session access for the session tool.

## Design Decisions

- **Trait object registry:** `Arc<dyn Tool>` in the registry allows plugins to
  contribute tools at runtime without recompilation.
- **`serde_json::Value` for parameters:** Keeps the trait generic; JSON Schema
  validation happens before `execute()` is called.
- **Approval as an async handler:** The `ApprovalHandler` trait is injected via
  `ToolContext`, decoupling tool execution from WebSocket transport details.
