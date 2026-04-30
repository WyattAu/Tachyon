//! WASM Sandbox for plugin execution.

use crate::error::PluginRuntimeError;
use crate::PluginRuntimeResult;
use serde_json::Value;
use std::path::Path;
use wasmtime::*;
use wasmtime_wasi::p1::{self, WasiP1Ctx};
use wasmtime_wasi::WasiCtxBuilder;

#[derive(Debug, Clone)]
pub struct SandboxConfig {
    pub max_fuel: u64,
    pub memory_limit: usize,
    pub enable_wasi: bool,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            max_fuel: 10_000_000,
            memory_limit: 64 * 1024 * 1024,
            enable_wasi: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PluginContext {
    pub hook: String,
    pub input: Value,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone)]
pub struct PluginOutput {
    pub data: Value,
    pub stdout: String,
    pub stderr: String,
}

pub struct PluginSandbox {
    wasm_path: std::path::PathBuf,
    config: SandboxConfig,
}

impl PluginSandbox {
    pub fn new(wasm_path: &Path, config: &SandboxConfig) -> PluginRuntimeResult<Self> {
        if !wasm_path.exists() {
            return Err(PluginRuntimeError::NotFound(format!(
                "WASM file not found: {}",
                wasm_path.display()
            )));
        }
        Ok(Self {
            wasm_path: wasm_path.to_path_buf(),
            config: config.clone(),
        })
    }

    pub fn execute(&self, ctx: &PluginContext) -> PluginRuntimeResult<Value> {
        let mut engine_config = Config::new();
        engine_config.consume_fuel(true);
        engine_config.wasm_component_model(true);

        let engine = Engine::new(&engine_config)
            .map_err(|e| PluginRuntimeError::Runtime(format!("Engine creation: {}", e)))?;

        let mut wasi_builder = WasiCtxBuilder::new();
        if self.config.enable_wasi {
            wasi_builder
                .inherit_stdin()
                .inherit_stdout()
                .inherit_stderr();
        }
        let wasi_ctx = wasi_builder.build_p1();

        let mut store = Store::new(&engine, wasi_ctx);
        store
            .set_fuel(self.config.max_fuel)
            .map_err(|e| PluginRuntimeError::Runtime(format!("Fuel setup: {}", e)))?;

        let module = match Module::from_file(&engine, &self.wasm_path) {
            Ok(m) => m,
            Err(e) => {
                return Err(PluginRuntimeError::Compilation(format!(
                    "Failed to compile {}: {}",
                    self.wasm_path.display(),
                    e
                )))
            }
        };

        let mut linker = Linker::new(&engine);
        p1::add_to_linker_sync(&mut linker, |t| t)
            .map_err(|e| PluginRuntimeError::Runtime(format!("WASI linker: {}", e)))?;

        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| PluginRuntimeError::Execution(format!("Instantiation failed: {}", e)))?;

        let call_entry = |name: &str, store: &mut Store<WasiP1Ctx>, instance: &Instance| {
            match instance.get_typed_func::<(), ()>(&mut *store, name) {
                Ok(func) => func.call(store, ()).map_err(|e| {
                    let err_str = e.to_string();
                    if err_str.contains("fuel") || err_str.contains("trap") {
                        PluginRuntimeError::Timeout(ctx.timeout_ms)
                    } else {
                        PluginRuntimeError::Execution(err_str)
                    }
                }),
                Err(_) => Err(PluginRuntimeError::Execution(format!(
                    "entry point '{}' not found",
                    name
                ))),
            }
        };

        if instance.get_export(&mut store, "_start").is_some() {
            call_entry("_start", &mut store, &instance)?;
        } else if instance.get_export(&mut store, "main").is_some() {
            call_entry("main", &mut store, &instance)?;
        } else {
            return Ok(ctx.input.clone());
        }

        Ok(ctx.input.clone())
    }
}
