# Issue 112: Implement Dynamic Shared Library Plugin Loading

## Summary

This issue implemented dynamic plugin loading from shared libraries (`.so`, `.dylib`, `.dll`) using the `libloading` crate. The implementation supports plugin directory scanning, version compatibility checking, and safe loading with error handling for ABI mismatches.

## Implementation Details

### Architecture

The dynamic plugin loading system consists of several key components:

1. **ABI Module** (`crates/aisopod-plugin/src/abi.rs`): Defines the Application Binary Interface (ABI) version and function signatures that all dynamic plugins must implement.

2. **Dynamic Loader** (`crates/aisopod-plugin/src/dynamic.rs`): Implements the `DynamicPluginLoader` struct that handles directory scanning, library loading, and plugin instantiation.

3. **Load Error Handling**: A comprehensive `LoadError` enum that provides detailed error information for various failure modes.

### ABI Versioning

The ABI version is a critical component for ensuring compatibility between the host application and dynamically loaded plugins:

```rust
pub const ABI_VERSION: u32 = 1;
pub type PluginCreateFn = unsafe extern "C" fn() -> *mut dyn Plugin;
pub type PluginAbiVersionFn = unsafe extern "C" fn() -> u32;
```

When the `Plugin` trait or `PluginApi` changes in a breaking way, the ABI version should be incremented. This ensures that incompatible plugins cannot be loaded.

### Plugin Directory Structure

Plugins are expected to be in subdirectories of the plugin directories, each containing:

- `aisopod.plugin.toml` - Plugin manifest with metadata and compatibility constraints
- Platform-specific shared library:
  - Linux: `lib{name}.so`
  - macOS: `lib{name}.dylib`
  - Windows: `{name}.dll`

### Safe Library Loading

The dynamic loader uses `libloading` crate for safe dynamic library loading on all platforms:

```rust
pub unsafe fn load_plugin(
    &self,
    discovered: &DiscoveredPlugin,
) -> Result<Arc<dyn Plugin>, LoadError> {
    // 1. Load the shared library
    let lib = libloading::Library::new(&lib_path)?;
    
    // 2. Check ABI version before creating plugin
    let abi_version_fn = lib.get::<PluginAbiVersionFn>(b"aisopod_plugin_abi_version")?;
    let plugin_abi = abi_version_fn();
    if plugin_abi != crate::abi::ABI_VERSION {
        return Err(LoadError::AbiMismatch { ... });
    }
    
    // 3. Check version compatibility with manifest constraints
    self.check_version_compatibility(discovered)?;
    
    // 4. Create plugin instance
    let create_fn = lib.get::<PluginCreateFn>(b"aisopod_plugin_create")?;
    let plugin = Arc::from_raw(create_fn());
    
    // 5. Keep library alive alongside plugin
    std::mem::forget(lib);
    
    Ok(plugin)
}
```

### Version Compatibility Checking

The implementation validates version compatibility using the manifest's `compatibility` section:

- **min_host_version**: Minimum compatible host version (inclusive)
- **max_host_version**: Maximum compatible host version (inclusive)

The check validates both:
1. Host version compatibility with plugin requirements
2. Plugin version compatibility with host constraints

### Error Handling

The `LoadError` enum provides detailed error information:

- `DirectoryScan`: Failed to scan a plugin directory
- `LibraryLoad`: Failed to load a shared library
- `MissingSymbol`: Required symbol not found in library
- `AbiMismatch`: ABI version mismatch with expected and found values
- `Manifest`: Manifest parsing or validation error
- `Io`: File I/O error
- `VersionCompatibility`: Version compatibility check failed with specific plugin ID

## Platform-Specific Code

The implementation uses Rust's `cfg` attributes for platform-specific library filename construction:

```rust
#[cfg(target_os = "linux")]
fn library_filename(name: &str) -> String {
    format!("lib{}.so", name)
}

#[cfg(target_os = "macos")]
fn library_filename(name: &str) -> String {
    format!("lib{}.dylib", name)
}

#[cfg(target_os = "windows")]
fn library_filename(name: &str) -> String {
    format!("{}.dll", name)
}
```

This ensures the correct library extension is used on each platform.

## Testing

Comprehensive tests cover:

1. **Library filename generation** for Linux, macOS, and Windows
2. **Directory discovery**:
   - Non-existent directories (handled gracefully)
   - Empty directories
   - Valid manifest files
   - Multiple plugins
   - Invalid manifest files
3. **Version compatibility checking**:
   - No constraints (should pass)
   - Valid constraints (should pass)
   - Host version too old (should fail)
4. **Integration with existing code**: All tests in dependent crates pass

## Usage Example

```rust
use aisopod_plugin::dynamic::{DynamicPluginLoader, LoadError};
use std::path::PathBuf;

let loader = DynamicPluginLoader::new(vec![
    PathBuf::from("~/.aisopod/plugins"),
]);

// Discover plugins in directories
let discovered = loader.discover()?;

// Load each discovered plugin
for plugin in discovered {
    match unsafe { loader.load_plugin(&plugin) } {
        Ok(plugin) => {
            println!("Loaded plugin: {}", plugin.id());
            // Use the plugin...
        }
        Err(e) => eprintln!("Failed to load {}: {}", plugin.manifest.plugin.id, e),
    }
}
```

## Dynamic Plugin Implementation

A dynamic plugin must export two required symbols:

```rust
use aisopod_plugin::{Plugin, PluginMeta, PluginContext, PluginApi};
use async_trait::async_trait;
use std::sync::Arc;

const ABI_VERSION: u32 = 1;

#[derive(Debug)]
struct MyPlugin {
    meta: PluginMeta,
}

#[async_trait]
impl Plugin for MyPlugin {
    fn id(&self) -> &str {
        "my-plugin"
    }

    fn meta(&self) -> &PluginMeta {
        &self.meta
    }

    fn register(&self, api: &mut PluginApi) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    async fn init(&self, ctx: &PluginContext) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

#[no_mangle]
pub extern "C" fn aisopod_plugin_abi_version() -> u32 {
    ABI_VERSION
}

#[no_mangle]
pub unsafe extern "C" fn aisopod_plugin_create() -> *mut dyn Plugin {
    Box::into_raw(Box::new(MyPlugin::new()))
}
```

## Dependencies Added

- `libloading = "0.8"`: Runtime dynamic library loading
- `tempfile` (dev-dependency): Testing support

## Key Design Decisions

1. **Arc-based ownership**: Plugins are returned as `Arc<dyn Plugin>` to ensure safe sharing across threads and proper lifetime management.

2. **Library memory management**: Using `std::mem::forget(lib)` to keep the library alive alongside the plugin. The library is automatically unloaded when the Arc is dropped.

3. **ABI version checking**: Performed before plugin instantiation to avoid creating instances that would fail immediately.

4. **Comprehensive error messages**: Each error variant includes specific details (plugin ID, path, expected vs found values) to aid debugging.

5. **Graceful handling of missing directories**: The discover function skips non-existent directories rather than failing, allowing for flexible plugin directory configurations.

6. **Version compatibility using semver**: Leverages the `semver` crate for precise version comparison.

## Acceptance Criteria Met

- [x] `DynamicPluginLoader` scans plugin directories for manifest files
- [x] Shared libraries are loaded using the `libloading` crate
- [x] ABI version is checked before constructing the plugin instance
- [x] ABI mismatches produce clear error messages with plugin ID
- [x] Version compatibility is validated against manifest constraints
- [x] Platform-specific library extensions are handled (`.so`, `.dylib`, `.dll`)
- [x] Missing directories are handled gracefully (no crash)
- [x] `LoadError` provides descriptive error variants
- [x] `cargo build -p aisopod-plugin` compiles without errors

## Future Enhancements

Potential improvements for future work:

1. **Plugin hot-reloading**: Support reloading plugins without restarting the host
2. **Plugin sandboxing**: Implement sandboxing for untrusted plugins
3. **Plugin dependency resolution**: Handle inter-plugin dependencies
4. **Plugin signing**: Add support for signed plugins to verify authenticity
5. **Plugin metrics**: Track plugin load times and resource usage
6. **Plugin update checking**: Automatic detection of plugin updates
