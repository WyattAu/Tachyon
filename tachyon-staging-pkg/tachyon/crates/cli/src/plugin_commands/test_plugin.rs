//! Plugin test subcommand.
//!
//! Compiles and runs a plugin's tests inside the Wasmtime sandbox
//! against a mock server state.

use std::path::{Path, PathBuf};
use tracing::{error, info, warn};

pub fn test_plugin(plugin_dir: &Path) -> Result<(), String> {
    let wasm_path = find_wasm_file(plugin_dir)
        .ok_or_else(|| format!("No .wasm file found in {}", plugin_dir.display()))?;

    info!(wasm_path = %wasm_path.display(), "Testing plugin");

    if !wasm_path.exists() {
        info!("WASM file not found, building...");
        super::build_plugin::build_plugin(false).map_err(|e| format!("Build failed: {}", e))?;
    }

    let wasm_bytes =
        std::fs::read(&wasm_path).map_err(|e| format!("Failed to read WASM: {}", e))?;

    info!(size = wasm_bytes.len(), "WASM binary loaded");

    let config = tachyon_plugin_runtime::SandboxConfig::default();
    let sandbox = tachyon_plugin_runtime::PluginSandbox::new(&wasm_path, &config)
        .map_err(|e| format!("Failed to create sandbox: {}", e))?;

    let test_hooks = vec![
        (
            "on_document_save",
            r#"{"title":"Test","content":"Hello world"}"#,
        ),
        ("on_document_delete", r#"{"id":"test-id"}"#),
    ];

    let mut passed = 0;
    let mut failed = 0;

    for (hook, input) in &test_hooks {
        let ctx = tachyon_plugin_runtime::PluginContext {
            hook: hook.to_string(),
            input: serde_json::from_str(input).unwrap_or(serde_json::json!({})),
            timeout_ms: 5000,
        };

        match sandbox.execute(&ctx) {
            Ok(output) => {
                if let Some(s) = output.as_str() {
                    let trimmed = s.trim();
                    if !trimmed.is_empty() {
                        info!(hook, stdout = %trimmed, "Test output");
                    }
                }
                passed += 1;
            }
            Err(e) => {
                error!(hook, error = %e, "Test FAILED");
                failed += 1;
            }
        }
    }

    println!();
    println!("Plugin test results:");
    println!("  Passed: {}", passed);
    println!("  Failed: {}", failed);
    println!("  Total:  {}", passed + failed);

    if failed > 0 {
        Err(format!("{} test(s) failed", failed))
    } else {
        Ok(())
    }
}

fn find_wasm_file(dir: &Path) -> Option<PathBuf> {
    if dir.is_dir() {
        let debug_path = dir.join("target/wasm32-unknown-unknown/debug");
        let release_path = dir.join("target/wasm32-unknown-unknown/release");

        for target_dir in [&debug_path, &release_path, dir] {
            if let Ok(entries) = std::fs::read_dir(target_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map_or(false, |ext| ext == "wasm") {
                        return Some(path);
                    }
                }
            }
        }
    }
    None
}
