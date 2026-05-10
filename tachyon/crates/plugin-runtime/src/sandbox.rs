//! WASM Sandbox for plugin execution.

use crate::error::PluginRuntimeError;
use crate::PluginRuntimeResult;
use serde_json::Value;
use std::path::Path;
use wasmtime::*;
use wasmtime_wasi::p1::{self, WasiP1Ctx};
use wasmtime_wasi::p2::pipe::{MemoryInputPipe, MemoryOutputPipe};
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

        let input_bytes = ctx.input.to_string().into_bytes();
        let stdout_pipe = MemoryOutputPipe::new(1024 * 1024);
        let stderr_pipe = MemoryOutputPipe::new(1024 * 1024);

        let mut wasi_builder = WasiCtxBuilder::new();
        if self.config.enable_wasi {
            wasi_builder
                .stdin(MemoryInputPipe::new(input_bytes))
                .stdout(stdout_pipe.clone())
                .stderr(stderr_pipe.clone());
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

        let call_entry =
            |name: &str, store: &mut Store<WasiP1Ctx>, instance: &Instance| match instance
                .get_typed_func::<(), ()>(&mut *store, name)
            {
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
            };

        if instance.get_export(&mut store, "_start").is_some() {
            call_entry("_start", &mut store, &instance)?;
        } else if instance.get_export(&mut store, "main").is_some() {
            call_entry("main", &mut store, &instance)?;
        } else {
            return Ok(ctx.input.clone());
        }

        let stdout_bytes = stdout_pipe.contents();
        let stdout_str = String::from_utf8_lossy(&stdout_bytes).to_string();

        match serde_json::from_str::<Value>(&stdout_str) {
            Ok(data) => Ok(data),
            Err(_) => {
                let trimmed = stdout_str.trim();
                if trimmed.is_empty() {
                    Ok(ctx.input.clone())
                } else {
                    Ok(serde_json::json!({ "output": trimmed }))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_test_config() -> SandboxConfig {
        SandboxConfig {
            max_fuel: 1_000_000,
            memory_limit: 16 * 1024 * 1024,
            enable_wasi: false,
        }
    }

    fn make_plugin_context() -> PluginContext {
        PluginContext {
            hook: "test_hook".to_string(),
            input: serde_json::json!({"key": "value"}),
            timeout_ms: 5000,
        }
    }

    #[test]
    fn sandbox_new_with_nonexistent_path_returns_not_found() {
        let path = PathBuf::from("/tmp/__nonexistent_wasm_file_12345__.wasm");
        let config = make_test_config();
        let result = PluginSandbox::new(&path, &config);
        match result {
            Err(PluginRuntimeError::NotFound(msg)) => {
                assert!(msg.contains("__nonexistent_wasm_file_12345__"));
            }
            Err(other) => panic!("expected NotFound error, got: {:?}", other),
            Ok(_) => panic!("expected error, got Ok"),
        }
    }

    #[test]
    fn sandbox_new_with_existing_file_succeeds() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let config = make_test_config();
        let sandbox = PluginSandbox::new(tmp.path(), &config);
        assert!(sandbox.is_ok());
        let sb = sandbox.unwrap();
        assert_eq!(sb.wasm_path, tmp.path());
    }

    #[test]
    fn sandbox_execute_invalid_wasm_returns_compilation_error() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), b"not valid wasm").unwrap();
        let config = make_test_config();
        let sandbox = PluginSandbox::new(tmp.path(), &config).unwrap();
        let ctx = make_plugin_context();
        let result = sandbox.execute(&ctx);
        match result {
            Err(PluginRuntimeError::Compilation(msg)) => {
                assert!(msg.contains("Failed to compile"));
            }
            Err(other) => panic!("expected Compilation error, got: {:?}", other),
            Ok(_) => panic!("expected error, got Ok"),
        }
    }

    #[test]
    fn sandbox_execute_empty_wasm_returns_compilation_error() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), b"").unwrap();
        let config = make_test_config();
        let sandbox = PluginSandbox::new(tmp.path(), &config).unwrap();
        let ctx = make_plugin_context();
        let result = sandbox.execute(&ctx);
        assert!(result.is_err());
    }

    #[test]
    fn sandbox_config_default_values() {
        let config = SandboxConfig::default();
        assert_eq!(config.max_fuel, 10_000_000);
        assert_eq!(config.memory_limit, 64 * 1024 * 1024);
        assert!(config.enable_wasi);
    }

    #[test]
    fn plugin_output_fields() {
        let output = PluginOutput {
            data: serde_json::json!(42),
            stdout: "hello".to_string(),
            stderr: String::new(),
        };
        assert_eq!(output.data, 42);
        assert_eq!(output.stdout, "hello");
        assert!(output.stderr.is_empty());
    }

    #[test]
    fn plugin_context_fields() {
        let ctx = PluginContext {
            hook: "on_save".to_string(),
            input: serde_json::json!({"path": "/tmp/file.txt"}),
            timeout_ms: 1000,
        };
        assert_eq!(ctx.hook, "on_save");
        assert_eq!(ctx.timeout_ms, 1000);
        assert_eq!(ctx.input["path"], "/tmp/file.txt");
    }
}
