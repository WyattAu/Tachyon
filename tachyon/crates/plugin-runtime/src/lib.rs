//! Tachyon Plugin Runtime — WASM Sandbox
//!
//! Provides a secure sandbox for executing Tachyon plugins compiled to WebAssembly.
//! Plugins run in an isolated WASM instance with limited capabilities via WASI.

#![allow(dead_code)]

mod ai;
mod error;
mod marketplace;
mod permissions;
mod sandbox;
mod signing;

#[cfg(feature = "registry-client")]
pub mod registry_client;

#[cfg(feature = "registry-client")]
pub use registry_client::{
    PluginDownloadResponse, PluginListResponse, RegistryClient, RegistryConfig, SearchQuery,
};

pub use ai::{AiCapability, AiCapabilityType, AiError, AiRequest, AiResponse, Embedding};
pub use error::{PluginRuntimeError, PluginRuntimeResult};
pub use marketplace::{
    MarketplaceError, MarketplaceResult, PluginCompatibility, PluginId, PluginInstallStatus,
    PluginManifest, PluginMarketplace, PluginVersion,
};
pub use permissions::{PermissionSet, PluginPermission};
pub use sandbox::{PluginContext, PluginOutput, PluginSandbox, SandboxConfig};
pub use signing::{PluginSignature, SigningKeyPair};

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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_runtime() -> PluginRuntime {
        let tmp = tempfile::TempDir::new().unwrap();
        PluginRuntime::new(tmp.path())
    }

    fn make_loaded_plugin(name: &str, version: &str) -> LoadedPlugin {
        LoadedPlugin {
            name: name.to_string(),
            version: version.to_string(),
            runtime_type: "wasm".to_string(),
            wasm_path: std::path::PathBuf::from("/nonexistent/plugin.wasm"),
            extension_points: vec!["on_save".to_string()],
            enabled: false,
        }
    }

    fn make_loaded_plugin_with_wasm(
        name: &str,
        version: &str,
        wasm_path: &std::path::Path,
    ) -> LoadedPlugin {
        LoadedPlugin {
            name: name.to_string(),
            version: version.to_string(),
            runtime_type: "wasm".to_string(),
            wasm_path: wasm_path.to_path_buf(),
            extension_points: vec!["on_save".to_string()],
            enabled: false,
        }
    }

    #[test]
    fn new_creates_plugins_dir() {
        let tmp = tempfile::TempDir::new().unwrap();
        let sub_dir = tmp.path().join("plugins");
        let _runtime = PluginRuntime::new(&sub_dir);
        assert!(sub_dir.exists());
    }

    #[test]
    fn list_plugins_empty() {
        let runtime = make_runtime();
        assert!(runtime.list_plugins().is_empty());
    }

    #[test]
    fn get_plugin_nonexistent_returns_none() {
        let runtime = make_runtime();
        assert!(runtime.get_plugin("missing", "1.0.0").is_none());
    }

    #[test]
    fn invoke_hook_no_plugins_returns_empty() {
        let runtime = make_runtime();
        let results = runtime.invoke_hook("on_save", serde_json::json!({}), 1000);
        assert!(results.is_empty());
    }

    #[test]
    fn load_plugin_nonexistent_wasm_returns_error() {
        let mut runtime = make_runtime();
        let plugin = make_loaded_plugin("test", "1.0.0");
        let result = runtime.load_plugin(plugin);
        assert!(result.is_err());
        match result.unwrap_err() {
            PluginRuntimeError::NotFound(msg) => {
                assert!(msg.contains("WASM file not found"));
            }
            other => panic!("expected NotFound, got: {:?}", other),
        }
    }

    #[test]
    fn load_plugin_success() {
        let mut runtime = make_runtime();
        let wasm_file = tempfile::NamedTempFile::new().unwrap();
        let plugin = make_loaded_plugin_with_wasm("test", "1.0.0", wasm_file.path());
        let result = runtime.load_plugin(plugin);
        assert!(result.is_ok());
        assert_eq!(runtime.list_plugins().len(), 1);
    }

    #[test]
    fn unload_plugin_nonexistent_returns_error() {
        let mut runtime = make_runtime();
        let result = runtime.unload_plugin("missing", "1.0.0");
        assert!(result.is_err());
        match result.unwrap_err() {
            PluginRuntimeError::NotFound(msg) => assert!(msg.contains("Plugin not found")),
            other => panic!("expected NotFound, got: {:?}", other),
        }
    }

    #[test]
    fn enable_plugin_nonexistent_returns_error() {
        let mut runtime = make_runtime();
        let result = runtime.enable_plugin("missing", "1.0.0");
        assert!(result.is_err());
        match result.unwrap_err() {
            PluginRuntimeError::NotFound(_) => {}
            other => panic!("expected NotFound, got: {:?}", other),
        }
    }

    #[test]
    fn disable_plugin_nonexistent_returns_error() {
        let mut runtime = make_runtime();
        let result = runtime.disable_plugin("missing", "1.0.0");
        assert!(result.is_err());
    }

    #[test]
    fn load_then_get_plugin() {
        let mut runtime = make_runtime();
        let wasm_file = tempfile::NamedTempFile::new().unwrap();
        let plugin = make_loaded_plugin_with_wasm("myplug", "2.0.0", wasm_file.path());
        runtime.load_plugin(plugin).unwrap();
        let retrieved = runtime.get_plugin("myplug", "2.0.0");
        assert!(retrieved.is_some());
        let p = retrieved.unwrap();
        assert_eq!(p.name, "myplug");
        assert_eq!(p.version, "2.0.0");
    }

    #[test]
    fn enable_plugin_toggles_state() {
        let mut runtime = make_runtime();
        let wasm_file = tempfile::NamedTempFile::new().unwrap();
        let plugin = make_loaded_plugin_with_wasm("toggle", "1.0.0", wasm_file.path());
        runtime.load_plugin(plugin).unwrap();
        assert!(!runtime.get_plugin("toggle", "1.0.0").unwrap().enabled);
        runtime.enable_plugin("toggle", "1.0.0").unwrap();
        assert!(runtime.get_plugin("toggle", "1.0.0").unwrap().enabled);
        runtime.disable_plugin("toggle", "1.0.0").unwrap();
        assert!(!runtime.get_plugin("toggle", "1.0.0").unwrap().enabled);
    }

    #[test]
    fn unload_plugin_removes_from_list() {
        let mut runtime = make_runtime();
        let wasm_file = tempfile::NamedTempFile::new().unwrap();
        let plugin = make_loaded_plugin_with_wasm("ephemeral", "1.0.0", wasm_file.path());
        runtime.load_plugin(plugin).unwrap();
        assert_eq!(runtime.list_plugins().len(), 1);
        runtime.unload_plugin("ephemeral", "1.0.0").unwrap();
        assert!(runtime.list_plugins().is_empty());
        assert!(runtime.get_plugin("ephemeral", "1.0.0").is_none());
    }

    #[test]
    fn invoke_hook_disabled_plugin_not_called() {
        let mut runtime = make_runtime();
        let wasm_file = tempfile::NamedTempFile::new().unwrap();
        let plugin = LoadedPlugin {
            name: "disabled".to_string(),
            version: "1.0.0".to_string(),
            runtime_type: "wasm".to_string(),
            wasm_path: wasm_file.path().to_path_buf(),
            extension_points: vec!["on_save".to_string()],
            enabled: false,
        };
        runtime.load_plugin(plugin).unwrap();
        let results = runtime.invoke_hook("on_save", serde_json::json!({}), 1000);
        assert!(results.is_empty());
    }

    #[test]
    fn invoke_hook_non_matching_extension_not_called() {
        let mut runtime = make_runtime();
        let wasm_file = tempfile::NamedTempFile::new().unwrap();
        let plugin = LoadedPlugin {
            name: "other".to_string(),
            version: "1.0.0".to_string(),
            runtime_type: "wasm".to_string(),
            wasm_path: wasm_file.path().to_path_buf(),
            extension_points: vec!["on_delete".to_string()],
            enabled: true,
        };
        runtime.load_plugin(plugin).unwrap();
        let results = runtime.invoke_hook("on_save", serde_json::json!({}), 1000);
        assert!(results.is_empty());
    }

    #[test]
    fn plugins_dir_returns_path() {
        let tmp = tempfile::TempDir::new().unwrap();
        let runtime = PluginRuntime::new(tmp.path());
        assert_eq!(runtime.plugins_dir(), tmp.path());
    }
}
