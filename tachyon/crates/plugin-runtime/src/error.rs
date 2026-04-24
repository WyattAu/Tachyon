//! Error types for the plugin runtime.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PluginRuntimeError {
    #[error("Plugin not found: {0}")]
    NotFound(String),

    #[error("WASM compilation error: {0}")]
    Compilation(String),

    #[error("WASM execution error: {0}")]
    Execution(String),

    #[error("Plugin timed out after {0}ms")]
    Timeout(u64),

    #[error("Invalid plugin input: {0}")]
    InvalidInput(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("WASM runtime error: {0}")]
    Runtime(String),
}

pub type PluginRuntimeResult<T> = Result<T, PluginRuntimeError>;
