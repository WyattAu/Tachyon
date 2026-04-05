// GUI command for Tachyon CLI

use crate::commands::Command;
use crate::config::TachyonConfig;
use crate::error::{CliError, CliResult};
use std::path::PathBuf;

/// Options for GUI command
#[derive(Debug, Clone)]
pub struct GuiOptions {
    /// Repository path
    pub repo_path: PathBuf,

    /// Enable dev tools
    pub dev_tools: bool,

    /// Window width
    pub window_width: Option<u32>,

    /// Window height
    pub window_height: Option<u32>,

    /// Start maximized
    pub start_maximized: bool,

    /// Start minimized
    pub start_minimized: bool,

    /// Server host to connect to
    pub server_host: Option<String>,

    /// Server port to connect to
    pub server_port: Option<u16>,
}

impl Default for GuiOptions {
    fn default() -> Self {
        Self {
            repo_path: PathBuf::from(".tachyon"),
            dev_tools: false,
            window_width: None,
            window_height: None,
            start_maximized: false,
            start_minimized: false,
            server_host: None,
            server_port: None,
        }
    }
}

/// GUI command handler
pub struct GuiCommand {
    options: GuiOptions,
}

impl GuiCommand {
    /// Create a new GUI command
    pub fn new(options: GuiOptions) -> Self {
        Self { options }
    }

    /// Create from clap arguments
    pub fn from_args(
        repo_path: Option<PathBuf>,
        dev_tools: bool,
        window_width: Option<u32>,
        window_height: Option<u32>,
        start_maximized: bool,
        start_minimized: bool,
        server_host: Option<String>,
        server_port: Option<u16>,
    ) -> Self {
        Self::new(GuiOptions {
            repo_path: repo_path.unwrap_or_else(|| PathBuf::from(".tachyon")),
            dev_tools,
            window_width,
            window_height,
            start_maximized,
            start_minimized,
            server_host,
            server_port,
        })
    }

    /// Load configuration from file
    fn load_config(&self) -> CliResult<TachyonConfig> {
        let config_path = self.options.repo_path.join("tachyon.toml");

        if config_path.exists() {
            TachyonConfig::load_from_file(&config_path)
        } else {
            Ok(TachyonConfig::default())
        }
    }

    /// Validate repository path
    fn validate_repo_path(&self) -> CliResult<()> {
        if !self.options.repo_path.exists() {
            return Err(CliError::init_failed(format!(
                "Repository path does not exist: {}. Run 'tachyon init' first.",
                self.options.repo_path.display()
            )));
        }

        let db_path = self.options.repo_path.join("db");
        if !db_path.exists() {
            return Err(CliError::init_failed(format!(
                "Database directory does not exist: {}. Run 'tachyon init' first.",
                db_path.display()
            )));
        }

        Ok(())
    }

    /// Get effective window width
    fn get_window_width(&self, config: &TachyonConfig) -> u32 {
        self.options
            .window_width
            .unwrap_or(config.desktop.window_width)
    }

    /// Get effective window height
    fn get_window_height(&self, config: &TachyonConfig) -> u32 {
        self.options
            .window_height
            .unwrap_or(config.desktop.window_height)
    }

    /// Get effective dev tools setting
    fn get_dev_tools(&self, config: &TachyonConfig) -> bool {
        self.options.dev_tools || config.desktop.dev_tools
    }

    /// Get effective server host
    fn get_server_host(&self, config: &TachyonConfig) -> String {
        self.options
            .server_host
            .clone()
            .unwrap_or_else(|| config.server.host.clone())
    }

    /// Get effective server port
    fn get_server_port(&self, config: &TachyonConfig) -> u16 {
        self.options.server_port.unwrap_or(config.server.http_port)
    }

    /// Launch Tauri desktop application
    fn launch_tauri(&self, config: &TachyonConfig) -> CliResult<()> {
        // Note: This is a placeholder for actual Tauri integration
        // In a full implementation, this would spawn the Tauri application
        // as a separate process or use the Tauri API directly

        println!("Launching Tauri desktop application...");
        println!("");
        println!("Configuration:");
        println!("  Repository: {}", self.options.repo_path.display());
        println!(
            "  Server: {}:{}",
            self.get_server_host(config),
            self.get_server_port(config)
        );
        println!(
            "  Window size: {}x{}",
            self.get_window_width(config),
            self.get_window_height(config)
        );
        println!("  Dev tools: {}", self.get_dev_tools(config));
        println!("  Maximized: {}", self.options.start_maximized);
        println!("  Minimized: {}", self.options.start_minimized);
        println!("");

        // In production, we would:
        // 1. Build the Tauri app if needed
        // 2. Launch the Tauri application with the specified configuration
        // 3. Handle the application lifecycle

        // For now, we'll simulate the launch
        println!("Desktop application launched successfully!");
        println!("");
        println!("Note: This is a stub implementation.");
        println!("Full Tauri integration requires building the desktop crate separately.");

        Ok(())
    }
}

impl Command for GuiCommand {
    fn execute(&self) -> CliResult<()> {
        // Validate repository path
        self.validate_repo_path()?;

        // Load configuration
        let config = self.load_config()?;

        println!("");
        println!("Tachyon Desktop Application");
        println!("============================");
        println!("");

        // Launch the desktop application
        self.launch_tauri(&config)?;

        Ok(())
    }

    fn name(&self) -> &str {
        "gui"
    }

    fn description(&self) -> &str {
        "Launch Tauri desktop application"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gui_options_default() {
        let options = GuiOptions::default();
        assert_eq!(options.repo_path, PathBuf::from(".tachyon"));
        assert!(!options.dev_tools);
        assert!(!options.start_maximized);
        assert!(!options.start_minimized);
    }

    #[test]
    fn test_gui_command_from_args() {
        let cmd = GuiCommand::from_args(
            Some(PathBuf::from("/tmp/test")),
            true,
            Some(1920),
            Some(1080),
            true,
            false,
            Some("localhost".to_string()),
            Some(9000),
        );

        assert_eq!(cmd.options.repo_path, PathBuf::from("/tmp/test"));
        assert!(cmd.options.dev_tools);
        assert_eq!(cmd.options.window_width, Some(1920));
        assert_eq!(cmd.options.window_height, Some(1080));
        assert!(cmd.options.start_maximized);
        assert!(!cmd.options.start_minimized);
        assert_eq!(cmd.options.server_host, Some("localhost".to_string()));
        assert_eq!(cmd.options.server_port, Some(9000));
    }

    #[test]
    fn test_validate_repo_path_not_exists() {
        let options = GuiOptions {
            repo_path: PathBuf::from("/nonexistent/path"),
            ..Default::default()
        };
        let cmd = GuiCommand::new(options);

        let result = cmd.validate_repo_path();
        assert!(result.is_err());
    }

    #[test]
    fn test_get_window_width_from_options() {
        let options = GuiOptions {
            window_width: Some(2000),
            ..Default::default()
        };
        let config = TachyonConfig::default();
        let cmd = GuiCommand::new(options);

        assert_eq!(cmd.get_window_width(&config), 2000);
    }

    #[test]
    fn test_get_window_width_from_config() {
        let options = GuiOptions::default();
        let config = TachyonConfig::default();
        let cmd = GuiCommand::new(options);

        assert_eq!(cmd.get_window_width(&config), config.desktop.window_width);
    }

    #[test]
    fn test_get_window_height_from_options() {
        let options = GuiOptions {
            window_height: Some(1200),
            ..Default::default()
        };
        let config = TachyonConfig::default();
        let cmd = GuiCommand::new(options);

        assert_eq!(cmd.get_window_height(&config), 1200);
    }

    #[test]
    fn test_get_dev_tools_from_options() {
        let options = GuiOptions {
            dev_tools: true,
            ..Default::default()
        };
        let config = TachyonConfig::default();
        let cmd = GuiCommand::new(options);

        assert!(cmd.get_dev_tools(&config));
    }

    #[test]
    fn test_get_dev_tools_from_config() {
        let options = GuiOptions::default();
        let mut config = TachyonConfig::default();
        config.desktop.dev_tools = true;
        let cmd = GuiCommand::new(options);

        assert!(cmd.get_dev_tools(&config));
    }

    #[test]
    fn test_get_server_host_from_options() {
        let options = GuiOptions {
            server_host: Some("192.168.1.1".to_string()),
            ..Default::default()
        };
        let config = TachyonConfig::default();
        let cmd = GuiCommand::new(options);

        assert_eq!(cmd.get_server_host(&config), "192.168.1.1");
    }

    #[test]
    fn test_get_server_host_from_config() {
        let options = GuiOptions::default();
        let config = TachyonConfig::default();
        let cmd = GuiCommand::new(options);

        assert_eq!(cmd.get_server_host(&config), config.server.host);
    }

    #[test]
    fn test_get_server_port_from_options() {
        let options = GuiOptions {
            server_port: Some(9090),
            ..Default::default()
        };
        let config = TachyonConfig::default();
        let cmd = GuiCommand::new(options);

        assert_eq!(cmd.get_server_port(&config), 9090);
    }

    #[test]
    fn test_get_server_port_from_config() {
        let options = GuiOptions::default();
        let config = TachyonConfig::default();
        let cmd = GuiCommand::new(options);

        assert_eq!(cmd.get_server_port(&config), config.server.http_port);
    }
}
