//! Tachyon Plugin Runtime — WASM Sandbox
//!
//! Provides a secure sandbox for executing Tachyon plugins compiled to WebAssembly.
//! Plugins run in an isolated WASM instance with limited capabilities via WASI.

#![allow(dead_code)]

mod error;
mod sandbox;

pub use error::{PluginRuntimeError, PluginRuntimeResult};
pub use sandbox::{PluginContext, PluginOutput, PluginSandbox, SandboxConfig};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct LoadedPlugin {
    pub name: String,
    pub version: String,
    pub runtime_type: String,
    pub wasm_path: PathBuf,
    pub extension_points: Vec<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookResult {
    pub success: bool,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
    pub execution_time_us: u64,
}

#[derive(Debug, Clone)]
pub struct HookInvocation {
    pub hook: String,
    pub input: serde_json::Value,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone)]
pub struct PluginRuntime {
    plugins: HashMap<String, LoadedPlugin>,
    sandbox_config: SandboxConfig,
    plugins_dir: PathBuf,
}

impl PluginRuntime {
    pub fn new(plugins_dir: impl Into<PathBuf>) -> Self {
        let plugins_dir = plugins_dir.into();
        std::fs::create_dir_all(&plugins_dir).ok();
        Self {
            plugins: HashMap::new(),
            sandbox_config: SandboxConfig::default(),
            plugins_dir,
        }
    }

    pub fn load_plugin(&mut self, plugin: LoadedPlugin) -> PluginRuntimeResult<()> {
        if !plugin.wasm_path.exists() {
            return Err(PluginRuntimeError::NotFound(format!(
                "WASM file not found: {}",
                plugin.wasm_path.display()
            )));
        }

        let key = format!("{}:{}", plugin.name, plugin.version);
        tracing::info!("Loaded plugin: {}:{}", plugin.name, plugin.version);
        self.plugins.insert(key, plugin);
        Ok(())
    }

    pub fn unload_plugin(&mut self, name: &str, version: &str) -> PluginRuntimeResult<()> {
        let key = format!("{}:{}", name, version);
        self.plugins.remove(&key).map(|_| ()).ok_or_else(|| {
            PluginRuntimeError::NotFound(format!("Plugin not found: {}:{}", name, version))
        })
    }

    pub fn enable_plugin(&mut self, name: &str, version: &str) -> PluginRuntimeResult<()> {
        let key = format!("{}:{}", name, version);
        if let Some(plugin) = self.plugins.get_mut(&key) {
            plugin.enabled = true;
            Ok(())
        } else {
            Err(PluginRuntimeError::NotFound(format!(
                "Plugin not found: {}:{}",
                name, version
            )))
        }
    }

    pub fn disable_plugin(&mut self, name: &str, version: &str) -> PluginRuntimeResult<()> {
        let key = format!("{}:{}", name, version);
        if let Some(plugin) = self.plugins.get_mut(&key) {
            plugin.enabled = false;
            Ok(())
        } else {
            Err(PluginRuntimeError::NotFound(format!(
                "Plugin not found: {}:{}",
                name, version
            )))
        }
    }

    pub fn list_plugins(&self) -> Vec<&LoadedPlugin> {
        self.plugins.values().collect()
    }

    pub fn get_plugin(&self, name: &str, version: &str) -> Option<&LoadedPlugin> {
        self.plugins.get(&format!("{}:{}", name, version))
    }

    pub fn invoke_hook(
        &self,
        hook: &str,
        input: serde_json::Value,
        timeout_ms: u64,
    ) -> Vec<HookResult> {
        let matching: Vec<_> = self
            .plugins
            .values()
            .filter(|p| p.enabled && p.extension_points.iter().any(|ep| ep == hook))
            .collect();

        if matching.is_empty() {
            return vec![];
        }

        let ctx = PluginContext {
            hook: hook.to_string(),
            input: input.clone(),
            timeout_ms,
        };

        matching
            .iter()
            .map(|plugin| {
                let start = std::time::Instant::now();
                let result = self.execute_plugin(plugin, &ctx);
                let elapsed = start.elapsed().as_micros() as u64;

                match result {
                    Ok(output) => HookResult {
                        success: true,
                        output: Some(output),
                        error: None,
                        execution_time_us: elapsed,
                    },
                    Err(e) => {
                        tracing::warn!(
                            "Plugin {}:{} hook {} failed: {}",
                            plugin.name,
                            plugin.version,
                            hook,
                            e
                        );
                        HookResult {
                            success: false,
                            output: None,
                            error: Some(e.to_string()),
                            execution_time_us: elapsed,
                        }
                    }
                }
            })
            .collect()
    }

    fn execute_plugin(
        &self,
        plugin: &LoadedPlugin,
        ctx: &PluginContext,
    ) -> PluginRuntimeResult<serde_json::Value> {
        let sandbox = PluginSandbox::new(&plugin.wasm_path, &self.sandbox_config)?;
        let output = sandbox.execute(ctx)?;
        Ok(output)
    }

    pub fn plugins_dir(&self) -> &Path {
        &self.plugins_dir
    }
}
