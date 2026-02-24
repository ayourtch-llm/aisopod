# Sandbox Configuration Types Implementation

## Summary

This issue implemented the foundational sandbox configuration types for the aisopod system, including `SandboxConfig`, `SandboxRuntime`, and `WorkspaceAccess` enums/structs.

## Key Decisions

### 1. Module Structure

**Decision**: Created a dedicated `sandbox.rs` module within `aisopod-config/src/types/` rather than adding to existing modules like `tools.rs`.

**Rationale**: The sandbox configuration is a cross-cutting concern that affects tool execution but is distinct from tool-specific settings. This maintains separation of concerns and makes it easier to find and modify sandbox-related code.

### 2. Type Location

**Decision**: Defined all sandbox types in `aisopod-config` and re-exported them from `aisopod-tools`.

**Rationale**: The configuration types are conceptually part of the configuration system, not the tool execution system. By defining them in `aisopod-config`:
- The types are available to any crate that needs to reference sandbox configuration
- The dependency flow is clear (aisopod-tools depends on aisopod-config, not vice versa)
- The configuration crate owns the schema definition

### 3. Duration Serialization

**Decision**: Used `humantime-serde` crate for Duration serialization/deserialization with the `with = "humantime_serde"` attribute.

**Rationale**: 
- Human-readable duration strings (e.g., "60s", "5m") are more user-friendly than raw seconds
- The crate is already in the Cargo.lock, suggesting it's a common dependency
- Provides automatic parsing and serialization of duration strings

### 4. Sandbox Module in aisopod-tools

**Decision**: Created a `sandbox/` submodule in `aisopod-tools` that re-exports from `aisopod-config`.

**Rationale**: This provides a convenient import path for the tools crate while maintaining the proper separation of concerns:
- `aisopod_tools::sandbox::config::SandboxConfig` for tools that need direct access
- `aisopod_tools::{SandboxConfig, SandboxRuntime, WorkspaceAccess}` for quick imports

## Implementation Details

### Dependencies Added

```toml
# crates/aisopod-config/Cargo.toml
humantime = "2.3"
humantime-serde = "1.1"
```

### Type Definitions

**SandboxRuntime** - Container runtime selection:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SandboxRuntime {
    Docker,
    Podman,
}
```

**WorkspaceAccess** - File system access permissions:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WorkspaceAccess {
    None,
    ReadOnly,
    ReadWrite,
}
```

**SandboxConfig** - Full configuration with sensible defaults:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub enabled: bool,
    pub runtime: SandboxRuntime,
    pub image: String,
    pub workspace_access: WorkspaceAccess,
    pub network_access: bool,
    pub memory_limit: Option<String>,
    pub cpu_limit: Option<f64>,
    #[serde(with = "humantime_serde")]
    pub timeout: Duration,
}
```

### Default Values

The `Default` implementation for `SandboxConfig` provides:
- `enabled: false` - Sandbox is opt-in
- `runtime: Docker` - Docker as the default container runtime
- `image: "ubuntu:latest"` - A widely available base image
- `workspace_access: ReadOnly` - Conservative access by default
- `network_access: true` - Network access allowed by default
- `timeout: 5 minutes` - Reasonable default timeout

### Integration Points

**AgentBinding Extension**:
```rust
pub struct AgentBinding {
    // ... existing fields ...
    #[serde(default)]
    pub sandbox: Option<SandboxConfig>,
}
```

This allows each agent binding to specify its own sandbox configuration, or none at all.

## Testing Strategy

Added unit tests covering:
1. **Default construction**: Verifies sensible defaults
2. **Deserialization**: Parses TOML configuration
3. **Serialization**: Converts back to TOML correctly

Tests are located in `crates/aisopod-config/src/types/sandbox.rs`.

## Migration Notes

### Existing Code Updates

The following existing test files needed updates due to the new `sandbox` field in `AgentBinding`:

1. `crates/aisopod-agent/tests/helpers.rs`
2. `crates/aisopod-agent/tests/resolution.rs`

Both files now include `sandbox: None` in `AgentBinding` instantiations to maintain backward compatibility with code that doesn't use sandbox configuration.

### Breaking Changes

This change is **not backward compatible** with existing configuration files that use `AgentBinding` without the `sandbox` field. However:
- The field has `#[serde(default)]` so missing fields default to `None`
- Existing TOML configs without `sandbox` will continue to work with the default `None` value

## Future Considerations

### Potential Enhancements

1. **Resource Limits**: The `memory_limit` and `cpu_limit` fields accept strings and floats but could be strengthened with custom types for type safety.

2. **Additional Runtime Support**: Consider adding more container runtimes (e.g., `Containerd`, `Kubernetes`) or platform-specific options.

3. **Security Policies**: Add fields for security-related configurations like:
   - `read_only_root_filesystem`
   - `run_as_non_root`
   - `allowed_devices`

4. **Multi-Tenancy**: Consider adding fields for tenant-specific sandbox configurations.

## Lessons Learned

### What Worked Well

1. **Step-by-step Implementation**: Building the types incrementally (modules first, then serialization, then integration) made the implementation manageable.

2. **Unit Tests First**: Writing tests before finalizing the implementation helped ensure all traits (`Debug`, `Serialize`, `Deserialize`) were properly derived.

3. **Clear Separation**: Keeping configuration types separate from execution types made the codebase easier to understand and maintain.

### Challenges Encountered

1. **Import Paths**: The types live in `aisopod_config::types` rather than directly in `aisopod_config`, requiring explicit import paths in dependent crates.

2. **Test Updates**: Updating existing tests required finding and modifying all places where `AgentBinding` was constructed, as Rust requires all fields to be specified.

### Best Practices Applied

1. **Derive All Required Traits**: All public types derive `Debug`, `Clone`, `Serialize`, `Deserialize`, and `PartialEq` (for enums).

2. **Serde Defaults**: Using `#[serde(default)]` for optional fields ensures backward compatibility.

3. **Documentation**: Each type and field has clear doc comments explaining its purpose.

4. **Test Coverage**: Basic tests verify both default construction and round-trip serialization/deserialization.
