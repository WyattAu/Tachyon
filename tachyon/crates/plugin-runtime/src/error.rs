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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_display() {
        let err = PluginRuntimeError::NotFound("plugin X not found".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Plugin not found"));
        assert!(msg.contains("plugin X not found"));
    }

    #[test]
    fn compilation_display() {
        let err = PluginRuntimeError::Compilation("syntax error".to_string());
        let msg = err.to_string();
        assert!(msg.contains("compilation"));
        assert!(msg.contains("syntax error"));
    }

    #[test]
    fn execution_display() {
        let err = PluginRuntimeError::Execution("trap".to_string());
        assert!(err.to_string().contains("execution"));
    }

    #[test]
    fn timeout_display() {
        let err = PluginRuntimeError::Timeout(5000);
        let msg = err.to_string();
        assert!(msg.contains("5000ms"));
    }

    #[test]
    fn invalid_input_display() {
        let err = PluginRuntimeError::InvalidInput("bad json".to_string());
        assert!(err.to_string().contains("Invalid plugin input"));
    }

    #[test]
    fn io_error_roundtrip() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let err: PluginRuntimeError = io_err.into();
        let msg = err.to_string();
        assert!(msg.contains("I/O error"));
        assert!(msg.contains("file missing"));
    }

    #[test]
    fn runtime_display() {
        let err = PluginRuntimeError::Runtime("engine init failed".to_string());
        assert!(err.to_string().contains("engine init failed"));
    }
}
