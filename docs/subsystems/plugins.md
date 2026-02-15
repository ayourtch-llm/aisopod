# Plugin System

**Crate:** `aisopod-plugin`

## Overview

The plugin system provides extensibility for aisopod — allowing dynamic registration
of channels, tools, CLI commands, model providers, and lifecycle hooks. Plugins
progress through three loading phases: compiled-in (feature flags), dynamic shared
libraries, and (future) WASM sandboxed modules.

## Key Types

- **`Plugin` trait** — Core interface: `id()`, `meta()`, `register(api)`, `init(ctx)`,
  `shutdown()`.
- **`PluginMeta`** — Name, version, description, author, supported channels/providers.
- **`PluginApi`** — Registration surface provided during `register()`:
  `register_channel()`, `register_tool()`, `register_command()`,
  `register_provider()`, `register_hook()`.
- **`PluginRegistry`** — Manages plugin lifecycle (registration order, init, shutdown)
  with `HashMap<String, Arc<dyn Plugin>>`.
- **`PluginContext`** — Runtime context passed during `init()` (config, shared state).
- **`Hook` enum** — Lifecycle events: `BeforeAgentRun`, `AfterAgentRun`,
  `BeforeMessageSend`, `AfterMessageReceive`, `BeforeToolExecute`,
  `AfterToolExecute`, `OnSessionCreate`, `OnSessionEnd`, `OnGatewayStart`,
  `OnGatewayShutdown`, `OnClientConnect`, `OnClientDisconnect`.

## Manifest Format

Plugins declare metadata in `aisopod.plugin.toml`:

```toml
[plugin]
id = "my-plugin"
name = "My Plugin"
version = "1.0.0"
entry = "lib.so"

[capabilities]
channels = ["custom-channel"]
tools = ["custom-tool"]
```

## Discovery & Loading Phases

1. **Compiled-in** — Feature-gated (`--features telegram,discord`). Zero runtime
   overhead for unused plugins.
2. **Dynamic shared libraries** — `.so`/`.dylib`/`.dll` loaded via `libloading`
   from `~/.aisopod/plugins/`. Version compatibility is checked against the manifest.
3. **WASM (future)** — Sandboxed via `wasmtime` with capability-based security.

## CLI Command Registration

Plugins can add custom subcommands with security hardening:
- 72 reserved command names are protected from conflicts.
- Arguments are sanitized (max 4 KB, control character removal).
- Commands may require authentication (`requireAuth` flag).
- The registry is locked during command execution to prevent races.

## Plugin Configuration

Plugins define their own config schema within the manifest. Plugin config is stored
inside the main `aisopod` config file, validated at load time, and participates in
hot reload.

## Dependencies

- **aisopod-config** — `PluginsConfig`, plugin config validation.
- **aisopod-tools** — `Tool` trait for tool registration.
- **aisopod-channel** — `ChannelPlugin` trait for channel registration.
- **aisopod-provider** — `ModelProvider` trait for provider registration.
- **aisopod-gateway** — Hook integration for gateway lifecycle events.

## Design Decisions

- **Phased loading strategy:** Compiled-in plugins (Phase 1) ship first for
  reliability; dynamic loading (Phase 2) adds flexibility without blocking the
  initial release.
- **`PluginApi` as a builder:** Passing a mutable registration API into `register()`
  keeps plugin initialization declarative and testable.
- **Hook enum over trait objects:** A single `Hook` enum is simpler to dispatch and
  serialize for diagnostics than a separate trait per lifecycle event.
