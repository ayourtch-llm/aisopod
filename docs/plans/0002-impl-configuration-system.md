# 0002 — Configuration System

**Master Plan Reference:** Section 3.2 — Configuration System  
**Phase:** 1 (Foundation)  
**Dependencies:** 0001 (Project Structure)

---

## Objective

Port the OpenClaw Zod-schema-based configuration system to Rust using serde,
providing JSON5/TOML config file support, environment variable substitution,
`@include` directives, validation, and sensitive field handling.

---

## Deliverables

### 1. Configuration Types (`aisopod-config`)

Port all configuration structs from OpenClaw's `src/config/types.*.ts` files:

**Root config struct:**
```rust
pub struct AisopodConfig {
    pub meta: MetaConfig,
    pub auth: AuthConfig,
    pub env: EnvConfig,
    pub agents: AgentsConfig,
    pub models: ModelsConfig,
    pub channels: ChannelsConfig,
    pub tools: ToolsConfig,
    pub skills: SkillsConfig,
    pub plugins: PluginsConfig,
    pub session: SessionConfig,
    pub bindings: Vec<AgentBinding>,
    pub memory: MemoryConfig,
    pub gateway: GatewayConfig,
}
```

**Sub-config types** (each as a dedicated module):
- `MetaConfig` — Version metadata
- `AuthConfig` — API keys, auth profiles, gateway auth settings
- `EnvConfig` — Environment variables, shell injection
- `AgentsConfig` — Agent list + defaults (model, workspace, sandbox, subagents)
- `ModelsConfig` — Model definitions, providers, fallbacks
- `ChannelsConfig` — Per-channel configuration
- `ToolsConfig` — Bash, exec, file system tool settings
- `SkillsConfig` — Skill modules
- `PluginsConfig` — Plugin registry
- `SessionConfig` — Message handling, compaction
- `AgentBinding` — Agent-to-channel routing rules
- `MemoryConfig` — QMD memory system
- `GatewayConfig` — HTTP server, bind address, port, TLS

### 2. Config File Parsing

- Support JSON5 format (using `json5` crate) as primary
- Support TOML format (using `toml` crate) as alternative
- Auto-detect format from file extension

### 3. Environment Variable Substitution

Implement template expansion in config values:
- `${ENV_VAR}` — Required, error if not set
- `${ENV_VAR:-default}` — With default value
- Nested resolution support

### 4. Include Directive Support

Implement `@include` directive scanning:
- Parse `"@include": "path/to/fragment.json"` in config
- Merge included fragments into parent config
- Detect circular includes
- Relative path resolution from config file location

### 5. Validation

- Derive `serde::Deserialize` with `#[serde(deny_unknown_fields)]` where appropriate
- Custom validation functions for semantic rules (e.g., port ranges, valid model formats)
- Detailed error messages with path to invalid field
- Implement `validate()` method on `AisopodConfig`

### 6. Sensitive Field Handling

- Define `#[sensitive]` attribute or wrapper type for fields containing secrets
- Implement `Display` that redacts sensitive values
- JSON serialization option to mask sensitive fields for UI

### 7. Config Loading Pipeline

```
load_config(path) ->
  read file
  → detect format (JSON5 / TOML)
  → parse into serde_json::Value
  → expand environment variables
  → process @include directives
  → deserialize into AisopodConfig
  → validate semantic rules
  → return Result<AisopodConfig>
```

### 8. Default Config Generation

- Implement `Default` for all config structs with sensible defaults
- CLI command to generate a default config file
- Migration utility to convert OpenClaw config format

### 9. Config Hot Reload

- File watcher (using `notify` crate) for config changes
- Diff detection to identify changed sections
- Notify gateway of config changes via internal channel

---

## Acceptance Criteria

- [ ] All OpenClaw config types have Rust equivalents with serde derive
- [ ] JSON5 config files parse correctly
- [ ] TOML config files parse correctly
- [ ] Environment variable substitution works for `${VAR}` and `${VAR:-default}`
- [ ] `@include` directives resolve and merge correctly
- [ ] Circular include detection works
- [ ] Validation produces clear error messages
- [ ] Sensitive fields are redacted in Display/Debug output
- [ ] Default config generation produces a valid, commented config file
- [ ] Unit tests cover parsing, substitution, includes, and validation
- [ ] Config hot reload detects changes and notifies consumers
