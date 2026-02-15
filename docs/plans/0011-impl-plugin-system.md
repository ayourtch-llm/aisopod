# 0011 — Plugin System

**Master Plan Reference:** Section 3.9 — Plugin System  
**Phase:** 5 (Extensibility)  
**Dependencies:** 0001 (Project Structure), 0002 (Configuration), 0003 (Gateway), 0005 (Tools)

---

## Objective

Implement the extensible plugin system that allows dynamic registration of
channels, tools, CLI commands, and lifecycle hooks — matching OpenClaw's
plugin architecture.

---

## Deliverables

### 1. Plugin Trait (`aisopod-plugin`)

Define the core plugin interface:

```rust
pub trait Plugin: Send + Sync {
    /// Plugin unique identifier
    fn id(&self) -> &str;

    /// Plugin metadata (name, version, description)
    fn meta(&self) -> &PluginMeta;

    /// Register plugin capabilities with the API
    fn register(&self, api: &mut PluginApi) -> Result<()>;

    /// Initialize plugin (called after all plugins registered)
    fn init(&self, ctx: &PluginContext) -> Result<()> { Ok(()) }

    /// Shutdown plugin gracefully
    fn shutdown(&self) -> Result<()> { Ok(()) }
}

pub struct PluginMeta {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: Option<String>,
    pub supported_channels: Vec<String>,
    pub supported_providers: Vec<String>,
}
```

### 2. Plugin API

The API provided to plugins during registration:

```rust
pub struct PluginApi {
    // Registration methods
    pub fn register_channel(&mut self, plugin: Arc<dyn ChannelPlugin>);
    pub fn register_tool(&mut self, tool: Arc<dyn Tool>);
    pub fn register_command(&mut self, command: PluginCommand);
    pub fn register_provider(&mut self, provider: Arc<dyn ModelProvider>);
    pub fn register_hook(&mut self, hook: Hook);
}
```

### 3. Plugin Manifest

Define the manifest format (`aisopod.plugin.toml` or `.json`):

```toml
[plugin]
id = "my-plugin"
name = "My Plugin"
version = "1.0.0"
description = "A custom plugin"
entry = "lib.so"  # or "plugin.wasm"

[config]
# Plugin-specific config schema (optional)

[capabilities]
channels = ["custom-channel"]
providers = ["custom-provider"]
tools = ["custom-tool"]
```

### 4. Plugin Discovery & Loading

**Phase 1 — Compiled-in plugins:**
- Feature-gated Cargo features for each built-in plugin
- `--features telegram,discord,slack` at compile time
- Zero runtime overhead for unused plugins

**Phase 2 — Dynamic shared libraries:**
- Load `.so`/`.dylib`/`.dll` plugins at runtime
- Use `libloading` crate for dynamic linking
- Plugin directory scanning (`~/.aisopod/plugins/`)
- Version compatibility checking

**Phase 3 — WASM plugins (future):**
- Sandboxed execution via `wasmtime`
- Capability-based security
- Platform-independent distribution

### 5. Plugin Registry

```rust
pub struct PluginRegistry {
    plugins: HashMap<String, Arc<dyn Plugin>>,
    load_order: Vec<String>,
}

impl PluginRegistry {
    pub fn register(&mut self, plugin: Arc<dyn Plugin>) -> Result<()>;
    pub fn get(&self, id: &str) -> Option<&Arc<dyn Plugin>>;
    pub fn list(&self) -> Vec<&Arc<dyn Plugin>>;
    pub fn init_all(&self, ctx: &PluginContext) -> Result<()>;
    pub fn shutdown_all(&self) -> Result<()>;
}
```

### 6. Hook System

Lifecycle hooks that plugins can register:

```rust
pub enum Hook {
    // Agent lifecycle
    BeforeAgentRun(Box<dyn Fn(&AgentRunParams) -> Result<()>>),
    AfterAgentRun(Box<dyn Fn(&AgentRunResult) -> Result<()>>),

    // Message lifecycle
    BeforeMessageSend(Box<dyn Fn(&OutgoingMessage) -> Result<Option<OutgoingMessage>>>),
    AfterMessageReceive(Box<dyn Fn(&IncomingMessage) -> Result<()>>),

    // Tool lifecycle
    BeforeToolExecute(Box<dyn Fn(&str, &Value) -> Result<()>>),
    AfterToolExecute(Box<dyn Fn(&str, &ToolResult) -> Result<()>>),

    // Session lifecycle
    OnSessionCreate(Box<dyn Fn(&SessionKey) -> Result<()>>),
    OnSessionEnd(Box<dyn Fn(&SessionKey) -> Result<()>>),

    // Gateway lifecycle
    OnGatewayStart(Box<dyn Fn() -> Result<()>>),
    OnGatewayShutdown(Box<dyn Fn() -> Result<()>>),
    OnClientConnect(Box<dyn Fn(&GatewayClient) -> Result<()>>),
    OnClientDisconnect(Box<dyn Fn(&str) -> Result<()>>),
}
```

### 7. Plugin CLI Commands

- Allow plugins to register custom CLI subcommands
- Security hardening:
  - Reserved command name protection (prevent conflicts with built-in commands)
  - Argument sanitization (max size, control char removal)
  - Authorization checks (requireAuth flag)
  - Registry locking during execution

### 8. Plugin Configuration

- Plugins can define their own config schema
- Plugin config stored within the main aisopod config
- Config validation at plugin load time
- Hot reload support for plugin config changes

---

## Acceptance Criteria

- [ ] Plugin trait is well-defined and implementable
- [ ] Plugin API allows registering channels, tools, commands, hooks
- [ ] Plugin manifest format is documented and validated
- [ ] Compiled-in plugins activate via feature flags
- [ ] Dynamic plugin loading works (Phase 2)
- [ ] Plugin registry manages lifecycle (register → init → shutdown)
- [ ] Hook system fires events at correct lifecycle points
- [ ] Plugin CLI commands integrate securely with main CLI
- [ ] Plugin configuration validates and hot-reloads
- [ ] Reserved command protection prevents conflicts
- [ ] Unit tests cover plugin registration and hook execution
- [ ] Integration tests verify end-to-end plugin loading
