# Issue 112: Implement Dynamic Shared Library Plugin Loading (Phase 2)

## Summary
Implement dynamic plugin loading from shared libraries (`.so`, `.dylib`, `.dll`) using the `libloading` crate. Support plugin directory scanning, version compatibility checking, and safe loading with error handling for ABI mismatches.

## Location
- Crate: `aisopod-plugin`
- File: `crates/aisopod-plugin/src/dynamic.rs`, `crates/aisopod-plugin/src/abi.rs`

## Current Behavior
The plugin system supports only compiled-in plugins (Issue 111). There is no mechanism to discover and load plugins from the filesystem at runtime.

## Expected Behavior
The system scans a configurable plugin directory (defaulting to `~/.aisopod/plugins/`) for shared libraries accompanied by `aisopod.plugin.toml` manifests. Each library is loaded, its ABI version is checked against the host, and the plugin's constructor function is called to obtain a `Plugin` trait object that is registered with the `PluginRegistry`.

## Impact
Dynamic loading enables third-party plugin development without recompiling the host binary. This is Phase 2 of the loading strategy and is essential for extensibility.

## Suggested Implementation
1. **Define ABI version constants in `abi.rs`:**
   ```rust
   /// ABI version for plugin compatibility checking.
   /// Bump this when the Plugin trait or PluginApi changes.
   pub const ABI_VERSION: u32 = 1;

   /// Function signature that every dynamic plugin must export.
   pub type PluginCreateFn = unsafe extern "C" fn() -> *mut dyn Plugin;

   /// Function to query the plugin's ABI version.
   pub type PluginAbiVersionFn = unsafe extern "C" fn() -> u32;
   ```
2. **Implement plugin directory scanning in `dynamic.rs`:**
   ```rust
   use std::path::{Path, PathBuf};
   use crate::manifest::PluginManifest;

   pub struct DynamicPluginLoader {
       plugin_dirs: Vec<PathBuf>,
   }

   impl DynamicPluginLoader {
       pub fn new(plugin_dirs: Vec<PathBuf>) -> Self {
           Self { plugin_dirs }
       }

       pub fn discover(&self) -> Result<Vec<DiscoveredPlugin>, LoadError> {
           let mut discovered = Vec::new();
           for dir in &self.plugin_dirs {
               if !dir.exists() {
                   continue;
               }
               for entry in std::fs::read_dir(dir)? {
                   let entry = entry?;
                   let path = entry.path();
                   if path.is_dir() {
                       let manifest_path = path.join("aisopod.plugin.toml");
                       if manifest_path.exists() {
                           let manifest = PluginManifest::from_file(&manifest_path)?;
                           discovered.push(DiscoveredPlugin {
                               manifest,
                               dir: path,
                           });
                       }
                   }
               }
           }
           Ok(discovered)
       }
   }
   ```
3. **Implement safe library loading:**
   ```rust
   use libloading::{Library, Symbol};
   use crate::abi::{ABI_VERSION, PluginCreateFn, PluginAbiVersionFn};

   impl DynamicPluginLoader {
       pub unsafe fn load_plugin(
           &self,
           discovered: &DiscoveredPlugin,
       ) -> Result<Arc<dyn Plugin>, LoadError> {
           let lib_name = discovered.manifest.plugin.entry_point.clone();
           let lib_path = discovered.dir.join(library_filename(&lib_name));

           let lib = Library::new(&lib_path)
               .map_err(|e| LoadError::LibraryLoad(lib_path.clone(), e.to_string()))?;

           // Check ABI version
           let abi_version_fn: Symbol<PluginAbiVersionFn> = lib
               .get(b"aisopod_plugin_abi_version")
               .map_err(|_| LoadError::MissingSymbol("aisopod_plugin_abi_version".into()))?;
           let plugin_abi = abi_version_fn();
           if plugin_abi != ABI_VERSION {
               return Err(LoadError::AbiMismatch {
                   expected: ABI_VERSION,
                   found: plugin_abi,
                   plugin_id: discovered.manifest.plugin.id.clone(),
               });
           }

           // Create plugin instance
           let create_fn: Symbol<PluginCreateFn> = lib
               .get(b"aisopod_plugin_create")
               .map_err(|_| LoadError::MissingSymbol("aisopod_plugin_create".into()))?;
           let plugin = Arc::from_raw(create_fn());

           // Keep the library alive by storing it alongside the plugin
           std::mem::forget(lib);

           Ok(plugin)
       }
   }
   ```
4. **Implement version compatibility checking** using the manifest's `compatibility` section and semver comparison.
5. **Define `LoadError`:**
   ```rust
   #[derive(Debug, thiserror::Error)]
   pub enum LoadError {
       #[error("Failed to load library {0}: {1}")]
       LibraryLoad(PathBuf, String),
       #[error("Missing required symbol: {0}")]
       MissingSymbol(String),
       #[error("ABI mismatch for plugin '{plugin_id}': expected {expected}, found {found}")]
       AbiMismatch { expected: u32, found: u32, plugin_id: String },
       #[error("Manifest error: {0}")]
       Manifest(#[from] crate::manifest::ManifestError),
       #[error("IO error: {0}")]
       Io(#[from] std::io::Error),
   }
   ```
6. **Platform-specific library file name helper:**
   ```rust
   fn library_filename(name: &str) -> String {
       #[cfg(target_os = "linux")]
       { format!("lib{}.so", name) }
       #[cfg(target_os = "macos")]
       { format!("lib{}.dylib", name) }
       #[cfg(target_os = "windows")]
       { format!("{}.dll", name) }
   }
   ```

## Dependencies
- Issue 110 (PluginRegistry lifecycle management)
- Issue 109 (plugin manifest format and parser)

## Acceptance Criteria
- [ ] `DynamicPluginLoader` scans plugin directories for manifest files
- [ ] Shared libraries are loaded using the `libloading` crate
- [ ] ABI version is checked before constructing the plugin instance
- [ ] ABI mismatches produce clear error messages with plugin ID
- [ ] Version compatibility is validated against manifest constraints
- [ ] Platform-specific library extensions are handled (`.so`, `.dylib`, `.dll`)
- [ ] Missing directories are handled gracefully (no crash)
- [ ] `LoadError` provides descriptive error variants
- [ ] `cargo build -p aisopod-plugin` compiles without errors

---
*Created: 2026-02-15*
