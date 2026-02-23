//! Dynamic plugin loading from shared libraries.
//!
//! This module provides the [`DynamicPluginLoader`] struct that scans
//! plugin directories for shared libraries and loads them at runtime.
//!
//! # Plugin Directory Structure
//!
//! Plugins are expected to be in subdirectories of the plugin directories,
//! each containing:
//!
//! - `aisopod.plugin.toml` - Plugin manifest
//! - `lib{plugin_name}.so` (Linux), `lib{plugin_name}.dylib` (macOS), or
//!   `{plugin_name}.dll` (Windows) - The shared library
//!
//! # Example
//!
//! ```ignore
//! use aisopod_plugin::dynamic::{DynamicPluginLoader, LoadError};
//! use std::path::PathBuf;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let loader = DynamicPluginLoader::new(vec![
//!         PathBuf::from("~/.aisopod/plugins"),
//!     ]);
//!
//!     // Discover plugins in directories
//!     let discovered = loader.discover()?;
//!
//!     // Load each discovered plugin
//!     for plugin in discovered {
//!         match unsafe { loader.load_plugin(&plugin) } {
//!             Ok(plugin) => {
//!                 println!("Loaded plugin: {}", plugin.id());
//!                 // Use the plugin...
//!             }
//!             Err(e) => eprintln!("Failed to load {}: {}", plugin.manifest.plugin.id, e),
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::abi::{PluginAbiVersionFn, PluginCreateFn};
use crate::manifest::PluginManifest;
use crate::Plugin;
use thiserror::Error;

/// Error types for dynamic plugin loading.
///
/// This enum captures all possible errors that can occur when
/// scanning directories, loading shared libraries, and creating
/// plugin instances.
#[derive(Debug, Error)]
pub enum LoadError {
    /// Failed to scan a plugin directory.
    #[error("Failed to scan directory {0}: {1}")]
    DirectoryScan(PathBuf, String),

    /// Failed to load a shared library.
    #[error("Failed to load library {0}: {1}")]
    LibraryLoad(PathBuf, String),

    /// Missing required symbol in the library.
    #[error("Missing required symbol '{0}' in library {1}")]
    MissingSymbol(String, PathBuf),

    /// ABI version mismatch between host and plugin.
    #[error("ABI version mismatch for plugin '{plugin_id}': expected {expected}, found {found}")]
    AbiMismatch {
        /// Expected ABI version
        expected: u32,
        /// Found ABI version
        found: u32,
        /// Plugin identifier
        plugin_id: String,
    },

    /// Manifest parsing or validation error.
    #[error("Manifest error for {0}: {1}")]
    Manifest(String, #[source] crate::manifest::ManifestError),

    /// File I/O error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Version compatibility check failed.
    #[error("Version compatibility check failed for plugin '{plugin_id}': {error}")]
    VersionCompatibility {
        plugin_id: String,
        error: String,
    },
}

/// Information about a discovered plugin.
///
/// This struct contains the manifest and location information
/// for a plugin that has been discovered in a plugin directory.
#[derive(Debug, Clone)]
pub struct DiscoveredPlugin {
    /// The plugin's manifest information.
    pub manifest: PluginManifest,
    /// The directory containing the plugin files.
    pub dir: PathBuf,
}

/// Loader for dynamic plugins from shared libraries.
///
/// This struct provides methods to scan plugin directories for
/// shared libraries and load them at runtime. It handles:
///
/// - Directory scanning for plugin manifests
/// - Library loading with platform-specific naming
/// - ABI version checking
/// - Version compatibility validation
///
/// # Platform Support
///
/// On Unix-like systems (Linux, macOS), this uses `libloading` crate
/// for dynamic library loading. On Windows, the feature is conditionally
/// compiled and uses `libloading` as well, but with `.dll` extensions.
#[derive(Debug)]
pub struct DynamicPluginLoader {
    /// Directories to scan for plugins.
    plugin_dirs: Vec<PathBuf>,
}

impl DynamicPluginLoader {
    /// Creates a new `DynamicPluginLoader` with the given plugin directories.
    ///
    /// # Arguments
    ///
    /// * `plugin_dirs` - Vector of paths to scan for plugins. Each path
    ///   should contain subdirectories with plugin manifests and shared libraries.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use aisopod_plugin::dynamic::DynamicPluginLoader;
    /// use std::path::PathBuf;
    ///
    /// let loader = DynamicPluginLoader::new(vec![
    ///     PathBuf::from("/usr/lib/aisopod/plugins"),
    ///     PathBuf::from("~/.aisopod/plugins"),
    /// ]);
    /// ```
    pub fn new(plugin_dirs: Vec<PathBuf>) -> Self {
        Self { plugin_dirs }
    }

    /// Scans plugin directories for discovered plugins.
    ///
    /// This method scans each configured plugin directory for subdirectories
    /// containing `aisopod.plugin.toml` manifest files. For each manifest
    /// found, it creates a `DiscoveredPlugin` entry.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<DiscoveredPlugin>)` - List of discovered plugins
    /// * `Err(LoadError)` - Error if directory scanning fails
    ///
    /// # Example
    ///
    /// ```ignore
    /// use aisopod_plugin::dynamic::DynamicPluginLoader;
    /// use std::path::PathBuf;
    ///
    /// let loader = DynamicPluginLoader::new(vec![PathBuf::from("~/.aisopod/plugins")]);
    /// let discovered = loader.discover()?;
    /// for plugin in discovered {
    ///     println!("Found: {} v{}", plugin.manifest.plugin.name, plugin.manifest.plugin.version);
    /// }
    /// ```
    pub fn discover(&self) -> Result<Vec<DiscoveredPlugin>, LoadError> {
        let mut discovered = Vec::new();

        for dir in &self.plugin_dirs {
            // Skip non-existent directories gracefully
            if !dir.exists() {
                tracing::debug!("Plugin directory does not exist, skipping: {:?}", dir);
                continue;
            }

            // Try to read the directory
            let entries = match std::fs::read_dir(dir) {
                Ok(entries) => entries,
                Err(e) => {
                    return Err(LoadError::DirectoryScan(dir.clone(), e.to_string()));
                }
            };

            for entry in entries {
                let entry = entry.map_err(|e| {
                    LoadError::DirectoryScan(dir.clone(), format!("Failed to read entry: {}", e))
                })?;
                let path = entry.path();

                // Check if it's a directory
                if path.is_dir() {
                    // Look for manifest file
                    let manifest_path = path.join("aisopod.plugin.toml");
                    if manifest_path.exists() {
                        // Parse the manifest
                        let manifest = match PluginManifest::from_file(&manifest_path) {
                            Ok(m) => m,
                            Err(e) => {
                                return Err(LoadError::Manifest(
                                    path.to_string_lossy().to_string(),
                                    e,
                                ));
                            }
                        };

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

    /// Loads a discovered plugin from its shared library.
    ///
    /// This method loads the plugin's shared library, verifies ABI compatibility,
    /// and creates a plugin instance. The library is kept alive by storing it
    /// in a `std::mem::forget`.
    ///
    /// # Safety
    ///
    /// This method is unsafe because it loads and executes dynamic library code.
    /// The caller must ensure the library is from a trusted source.
    ///
    /// # Arguments
    ///
    /// * `discovered` - The `DiscoveredPlugin` containing manifest and location
    ///
    /// # Returns
    ///
    /// * `Ok(Arc<dyn Plugin>)` - Loaded plugin instance
    /// * `Err(LoadError)` - Error if loading fails
    ///
    /// # Errors
    ///
    /// This function can return:
    /// - `LoadError::LibraryLoad` - Failed to load the shared library
    /// - `LoadError::MissingSymbol` - Required symbol not found
    /// - `LoadError::AbiMismatch` - ABI version mismatch
    /// - `LoadError::Manifest` - Manifest error
    /// - `LoadError::Io` - I/O error
    /// - `LoadError::VersionCompatibility` - Version compatibility check failed
    ///
    /// # Example
    ///
    /// ```ignore
    /// use aisopod_plugin::dynamic::DynamicPluginLoader;
    /// use std::path::PathBuf;
    ///
    /// let loader = DynamicPluginLoader::new(vec![PathBuf::from("~/.aisopod/plugins")]);
    /// let discovered = loader.discover()?;
    ///
    /// if let Some(plugin_info) = discovered.first() {
    ///     let plugin = unsafe { loader.load_plugin(plugin_info) }?;
    ///     println!("Loaded: {}", plugin.id());
    /// }
    /// ```
    pub unsafe fn load_plugin(
        &self,
        discovered: &DiscoveredPlugin,
    ) -> Result<Arc<dyn Plugin>, LoadError> {
        let lib_path = self.library_path(discovered);

        // Load the shared library
        let lib = libloading::Library::new(&lib_path).map_err(|e| {
            LoadError::LibraryLoad(lib_path.clone(), e.to_string())
        })?;

        // Check ABI version
        let abi_version_fn = lib
            .get::<PluginAbiVersionFn>(b"aisopod_plugin_abi_version")
            .map_err(|_| LoadError::MissingSymbol("aisopod_plugin_abi_version".into(), lib_path.clone()))?;
        
        let plugin_abi = abi_version_fn();
        if plugin_abi != crate::abi::ABI_VERSION {
            return Err(LoadError::AbiMismatch {
                expected: crate::abi::ABI_VERSION,
                found: plugin_abi,
                plugin_id: discovered.manifest.plugin.id.clone(),
            });
        }

        // Check version compatibility
        self.check_version_compatibility(discovered)?;

        // Create plugin instance
        let create_fn = lib
            .get::<PluginCreateFn>(b"aisopod_plugin_create")
            .map_err(|_| LoadError::MissingSymbol("aisopod_plugin_create".into(), lib_path.clone()))?;
        
        let plugin = Arc::from_raw(create_fn());

        // Keep the library alive by storing it alongside the plugin
        std::mem::forget(lib);

        Ok(plugin)
    }

    /// Gets the path to the plugin's shared library.
    ///
    /// This method constructs the library path based on the plugin's
    /// entry point and the platform-specific naming convention.
    fn library_path(&self, discovered: &DiscoveredPlugin) -> PathBuf {
        let entry_point = &discovered.manifest.plugin.entry_point;
        let lib_name = library_filename(entry_point);
        discovered.dir.join(lib_name)
    }

    /// Checks version compatibility between host and plugin.
    ///
    /// This method validates that the plugin's version is compatible
    /// with the host version according to the manifest's compatibility
    /// constraints.
    ///
    /// # Arguments
    ///
    /// * `discovered` - The discovered plugin to check
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Versions are compatible
    /// * `Err(LoadError::VersionCompatibility)` - Versions are not compatible
    fn check_version_compatibility(&self, discovered: &DiscoveredPlugin) -> Result<(), LoadError> {
        let plugin_version = &discovered.manifest.plugin.version;
        let compatibility = &discovered.manifest.compatibility;
        let plugin_id = &discovered.manifest.plugin.id;

        if let Some(compat) = compatibility {
            let host_version = env!("CARGO_PKG_VERSION");
            
            let host_ver = semver::Version::parse(host_version).map_err(|_| {
                LoadError::VersionCompatibility {
                    plugin_id: plugin_id.clone(),
                    error: "Invalid host version".to_string(),
                }
            })?;

            let plugin_ver = semver::Version::parse(plugin_version).map_err(|_| {
                LoadError::VersionCompatibility {
                    plugin_id: plugin_id.clone(),
                    error: "Invalid plugin version".to_string(),
                }
            })?;

            // Check minimum version constraint
            if let Some(min_version) = &compat.min_host_version {
                let min_ver = semver::Version::parse(min_version).map_err(|_| {
                    LoadError::VersionCompatibility {
                        plugin_id: plugin_id.clone(),
                        error: "Invalid min_host_version".to_string(),
                    }
                })?;

                if host_ver < min_ver {
                    return Err(LoadError::VersionCompatibility {
                        plugin_id: plugin_id.clone(),
                        error: format!(
                            "Host version {} is below minimum required {}",
                            host_version, min_version
                        ),
                    });
                }
            }

            // Check maximum version constraint
            if let Some(max_version) = &compat.max_host_version {
                let max_ver = semver::Version::parse(max_version).map_err(|_| {
                    LoadError::VersionCompatibility {
                        plugin_id: plugin_id.clone(),
                        error: "Invalid max_host_version".to_string(),
                    }
                })?;

                if host_ver > max_ver {
                    return Err(LoadError::VersionCompatibility {
                        plugin_id: plugin_id.clone(),
                        error: format!(
                            "Host version {} is above maximum allowed {}",
                            host_version, max_version
                        ),
                    });
                }
            }

            // Check if plugin version is compatible with host
            if let Some(min_plugin) = &compat.min_host_version {
                let min_plugin_ver = semver::Version::parse(min_plugin).map_err(|_| {
                    LoadError::VersionCompatibility {
                        plugin_id: plugin_id.clone(),
                        error: "Invalid min_host_version".to_string(),
                    }
                })?;
                
                if plugin_ver < min_plugin_ver {
                    return Err(LoadError::VersionCompatibility {
                        plugin_id: plugin_id.clone(),
                        error: format!(
                            "Plugin version {} is below minimum required {}",
                            plugin_version, min_plugin
                        ),
                    });
                }
            }

            if let Some(max_plugin) = &compat.max_host_version {
                let max_plugin_ver = semver::Version::parse(max_plugin).map_err(|_| {
                    LoadError::VersionCompatibility {
                        plugin_id: plugin_id.clone(),
                        error: "Invalid max_host_version".to_string(),
                    }
                })?;
                
                if plugin_ver > max_plugin_ver {
                    return Err(LoadError::VersionCompatibility {
                        plugin_id: plugin_id.clone(),
                        error: format!(
                            "Plugin version {} is above maximum allowed {}",
                            plugin_version, max_plugin
                        ),
                    });
                }
            }
        }

        Ok(())
    }
}

/// Returns the platform-specific shared library filename.
///
/// This function constructs the library filename based on the platform:
/// - Linux: `lib{name}.so`
/// - macOS: `lib{name}.dylib`
/// - Windows: `{name}.dll`
///
/// # Arguments
///
/// * `name` - The base name of the library (without `lib` prefix or extension)
///
/// # Examples
///
/// ```ignore
/// assert_eq!(library_filename("my_plugin"), "libmy_plugin.so"); // Linux
/// assert_eq!(library_filename("my_plugin"), "libmy_plugin.dylib"); // macOS
/// assert_eq!(library_filename("my_plugin"), "my_plugin.dll"); // Windows
/// ```
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_library_filename_linux() {
        #[cfg(target_os = "linux")]
        {
            assert_eq!(library_filename("test"), "libtest.so");
            assert_eq!(library_filename("my_plugin"), "libmy_plugin.so");
        }
    }

    #[test]
    fn test_library_filename_macos() {
        #[cfg(target_os = "macos")]
        {
            assert_eq!(library_filename("test"), "libtest.dylib");
            assert_eq!(library_filename("my_plugin"), "libmy_plugin.dylib");
        }
    }

    #[test]
    fn test_library_filename_windows() {
        #[cfg(target_os = "windows")]
        {
            assert_eq!(library_filename("test"), "test.dll");
            assert_eq!(library_filename("my_plugin"), "my_plugin.dll");
        }
    }

    #[test]
    fn test_dynamic_plugin_loader_new() {
        let dirs = vec![PathBuf::from("/tmp/plugins")];
        let loader = DynamicPluginLoader::new(dirs);
        
        // Just verify it was created (no crash)
        let _ = loader;
    }

    #[test]
    fn test_discover_nonexistent_directory() {
        let loader = DynamicPluginLoader::new(vec![PathBuf::from("/tmp/nonexistent_aisopod_test_12345")]);
        let result = loader.discover();
        
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_discover_empty_directory() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let loader = DynamicPluginLoader::new(vec![temp_dir.path().to_path_buf()]);
        let result = loader.discover();
        
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_discover_with_valid_manifest() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let plugin_dir = temp_dir.path().join("test-plugin");
        fs::create_dir(&plugin_dir).expect("Failed to create plugin dir");

        let manifest_content = r#"
            [plugin]
            id = "test-plugin"
            name = "Test Plugin"
            version = "0.1.0"
            description = "A test plugin"
            author = "Test Author"
            entry_point = "libtest_plugin"

            [capabilities]
            channels = ["text"]
        "#;

        let manifest_path = plugin_dir.join("aisopod.plugin.toml");
        fs::write(&manifest_path, manifest_content)
            .expect("Failed to write manifest");

        let loader = DynamicPluginLoader::new(vec![temp_dir.path().to_path_buf()]);
        let result = loader.discover();

        assert!(result.is_ok(), "Discover should succeed");
        let discovered = result.unwrap();
        assert_eq!(discovered.len(), 1, "Should find one plugin");

        let plugin = &discovered[0];
        assert_eq!(plugin.manifest.plugin.id, "test-plugin");
        assert_eq!(plugin.manifest.plugin.name, "Test Plugin");
        assert_eq!(plugin.manifest.plugin.version, "0.1.0");
        assert_eq!(plugin.dir, plugin_dir);
    }

    #[test]
    fn test_discover_multiple_plugins() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create first plugin
        let plugin1_dir = temp_dir.path().join("plugin-1");
        fs::create_dir(&plugin1_dir).expect("Failed to create plugin1 dir");
        fs::write(plugin1_dir.join("aisopod.plugin.toml"), r#"
            [plugin]
            id = "plugin-1"
            name = "Plugin 1"
            version = "0.1.0"
            description = "Plugin 1"
            author = "Author"
            entry_point = "libplugin1"
        "#).expect("Failed to write manifest");

        // Create second plugin
        let plugin2_dir = temp_dir.path().join("plugin-2");
        fs::create_dir(&plugin2_dir).expect("Failed to create plugin2 dir");
        fs::write(plugin2_dir.join("aisopod.plugin.toml"), r#"
            [plugin]
            id = "plugin-2"
            name = "Plugin 2"
            version = "0.2.0"
            description = "Plugin 2"
            author = "Author"
            entry_point = "libplugin2"
        "#).expect("Failed to write manifest");

        let loader = DynamicPluginLoader::new(vec![temp_dir.path().to_path_buf()]);
        let result = loader.discover();

        assert!(result.is_ok());
        let discovered = result.unwrap();
        assert_eq!(discovered.len(), 2, "Should find two plugins");
    }

    #[test]
    fn test_discover_invalid_manifest() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let plugin_dir = temp_dir.path().join("invalid-plugin");
        fs::create_dir(&plugin_dir).expect("Failed to create plugin dir");

        let invalid_manifest = r#"
            [plugin]
            id = ""
            name = "Invalid"
            version = "not-a-version"
            entry_point = ""
        "#;

        fs::write(plugin_dir.join("aisopod.plugin.toml"), invalid_manifest)
            .expect("Failed to write manifest");

        let loader = DynamicPluginLoader::new(vec![temp_dir.path().to_path_buf()]);
        let result = loader.discover();

        assert!(result.is_err());
    }

    #[test]
    fn test_check_version_compatibility_no_constraints() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let plugin_dir = temp_dir.path().join("test-plugin");
        fs::create_dir(&plugin_dir).expect("Failed to create plugin dir");

        let manifest_content = r#"
            [plugin]
            id = "test-plugin"
            name = "Test Plugin"
            version = "0.1.0"
            description = "A test plugin"
            author = "Test Author"
            entry_point = "libtest_plugin"
        "#;

        let manifest_path = plugin_dir.join("aisopod.plugin.toml");
        fs::write(&manifest_path, manifest_content)
            .expect("Failed to write manifest");

        let discovered = DiscoveredPlugin {
            manifest: PluginManifest::from_file(&manifest_path).unwrap(),
            dir: plugin_dir,
        };

        let loader = DynamicPluginLoader::new(vec![]);
        let result = loader.check_version_compatibility(&discovered);
        
        assert!(result.is_ok(), "Should pass with no constraints");
    }

    #[test]
    fn test_check_version_compatibility_with_constraints() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let plugin_dir = temp_dir.path().join("test-plugin");
        fs::create_dir(&plugin_dir).expect("Failed to create plugin dir");

        let manifest_content = r#"
            [plugin]
            id = "test-plugin"
            name = "Test Plugin"
            version = "0.1.0"
            description = "A test plugin"
            author = "Test Author"
            entry_point = "libtest_plugin"

            [compatibility]
            min_host_version = "0.0.1"
            max_host_version = "99.99.99"
        "#;

        let manifest_path = plugin_dir.join("aisopod.plugin.toml");
        fs::write(&manifest_path, manifest_content)
            .expect("Failed to write manifest");

        let discovered = DiscoveredPlugin {
            manifest: PluginManifest::from_file(&manifest_path).unwrap(),
            dir: plugin_dir,
        };

        let loader = DynamicPluginLoader::new(vec![]);
        let result = loader.check_version_compatibility(&discovered);
        
        assert!(result.is_ok(), "Should pass with valid constraints");
    }

    #[test]
    fn test_check_version_compatibility_host_too_old() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let plugin_dir = temp_dir.path().join("test-plugin");
        fs::create_dir(&plugin_dir).expect("Failed to create plugin dir");

        // Use a very high minimum version that will definitely be higher than current
        let manifest_content = r#"
            [plugin]
            id = "test-plugin"
            name = "Test Plugin"
            version = "0.1.0"
            description = "A test plugin"
            author = "Test Author"
            entry_point = "libtest_plugin"

            [compatibility]
            min_host_version = "999.0.0"
        "#;

        let manifest_path = plugin_dir.join("aisopod.plugin.toml");
        fs::write(&manifest_path, manifest_content)
            .expect("Failed to write manifest");

        let discovered = DiscoveredPlugin {
            manifest: PluginManifest::from_file(&manifest_path).unwrap(),
            dir: plugin_dir,
        };

        let loader = DynamicPluginLoader::new(vec![]);
        let result = loader.check_version_compatibility(&discovered);
        
        assert!(result.is_err(), "Should fail when host is too old");
    }
}
