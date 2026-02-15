# Configuration Subsystem

**Crate:** `aisopod-config`

## Overview

The configuration subsystem provides loading, parsing, validation, and hot-reload of
aisopod's TOML/JSON5 configuration files. It replaces OpenClaw's Zod-schema-based
system with serde-derived Rust types, maintaining the same config-driven architecture.

## Key Types

- **`AisopodConfig`** — Root configuration struct containing all subsystem configs
  (`MetaConfig`, `AuthConfig`, `AgentsConfig`, `ModelsConfig`, `ChannelsConfig`,
  `ToolsConfig`, `PluginsConfig`, `SessionConfig`, `MemoryConfig`, `GatewayConfig`,
  `Vec<AgentBinding>`).
- **`Sensitive<T>`** — Wrapper type for secrets that redacts values in `Display`/`Debug`
  output and JSON serialization for UI.
- **`ConfigLoader`** — Orchestrates the full loading pipeline: read → detect format →
  parse → env substitution → `@include` processing → deserialize → validate.

## Loading Pipeline

1. Read file from disk (default: `~/.aisopod/aisopod.toml` or `.json`)
2. Detect format from file extension (TOML via `toml`, JSON5 via `json5`)
3. Expand environment variables (`${VAR}`, `${VAR:-default}`)
4. Process `@include` directives with circular-include detection
5. Deserialize into `AisopodConfig` via serde
6. Run semantic validation (`validate()` method)

## Hot Reload

File watching via the `notify` crate detects config changes, diffs changed sections,
and notifies the gateway through an internal `tokio::watch` channel.

## Dependencies

- **aisopod-shared** — Path resolution, binary detection utilities used during
  config evaluation.

## Design Decisions

- **serde over Zod:** Rust's serde ecosystem replaces Zod schemas, giving compile-time
  type safety plus runtime deserialization in one derive macro.
- **TOML as primary format:** TOML is the idiomatic Rust config format; JSON5 is
  retained for migration from OpenClaw configs.
- **`#[serde(deny_unknown_fields)]`** on key structs catches typos early with clear
  error messages including the path to the invalid field.
- **`Default` impls** on all config structs allow generating a valid starter config
  and simplify partial config merging.
