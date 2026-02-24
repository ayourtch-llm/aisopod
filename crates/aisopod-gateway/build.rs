// Build script for aisopod-gateway
// This script triggers the UI build before compiling the Rust code.

use std::process::Command;
use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=../ui");

    // Check if UI should be built (skip if AISOPOD_BUILD_UI is not set)
    // This allows development mode without rebuilding the UI on every cargo build
    if env::var("AISOPOD_BUILD_UI").is_err() {
        return;
    }

    // The build script runs from the target/debug/build/<crate>/ directory
    // We need to find the workspace root and then the ui directory
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let manifest_path = PathBuf::from(&manifest_dir);
    
    // Go up from crates/aisopod-gateway -> crates -> workspace root
    let workspace_root = manifest_path
        .parent()
        .and_then(|p| p.parent())
        .expect("Failed to find workspace root");
    
    let ui_dir = workspace_root.join("ui");

    // Check if pnpm is available
    let pnpm_path = env::var("PNPM_PATH")
        .ok()
        .filter(|p| std::path::Path::new(p).exists())
        .or_else(|| {
            // Try common locations
            let home = env::var("HOME").ok()?;
            let candidate = format!("{}/.nvm/versions/node/v22.17.0/bin/pnpm", home);
            std::path::Path::new(&candidate).exists().then_some(candidate)
        })
        .unwrap_or_else(|| "pnpm".to_string());

    // Build the UI using pnpm from the ui directory
    let status = Command::new(&pnpm_path)
        .args(&["build"])
        .current_dir(&ui_dir)
        .status()
        .expect("Failed to execute pnpm build");

    if !status.success() {
        panic!("UI build failed with exit code: {}", status);
    }
}
