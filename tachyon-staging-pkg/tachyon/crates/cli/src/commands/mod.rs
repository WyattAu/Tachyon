// Command modules for Tachyon CLI

pub mod build;
pub mod gui;
pub mod init;
pub mod plugin;
pub mod serve;

// Re-export command handlers
pub use build::BuildCommand;
pub use gui::GuiCommand;
pub use init::InitCommand;
pub use plugin::{PluginInfoCommand, PluginInstallCommand, PluginListCommand};
pub use serve::ServeCommand;

use crate::error::CliResult;

/// Trait for CLI commands
pub trait Command {
    /// Execute the command
    fn execute(&self) -> CliResult<()>;

    /// Get command name
    fn name(&self) -> &str;

    /// Get command description
    fn description(&self) -> &str;
}
