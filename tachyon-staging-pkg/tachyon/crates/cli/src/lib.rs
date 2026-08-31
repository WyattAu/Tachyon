// Tachyon CLI Library
// Provides command-line interface functionality for Tachyon knowledge base system

pub mod commands;
pub mod config;
pub mod error;
pub mod plugin_commands;

// Re-export commonly used types
pub use error::{CliError, CliResult};

/// Tachyon CLI version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Tachyon CLI name
pub const NAME: &str = env!("CARGO_PKG_NAME");
