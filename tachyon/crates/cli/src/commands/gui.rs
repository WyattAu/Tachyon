// GUI command for Tachyon CLI
//
// Launches the Tauri desktop application by spawning `cargo tauri dev`
// (development) or the built binary (release).

use crate::commands::Command;
use crate::config::TachyonConfig;
use crate::error::{CliError, CliResult};
use std::path::PathBuf;
use std::process::Command as ProcessCommand;

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

    /// Release mode (use built binary instead of cargo tauri dev)
    pub release: bool,
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
            release: false,
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
        dev_tools: bool,
        window_width: Option<u32>,
        window_height: Option<u32>,
        start_maximized: bool,
        start_minimized: bool,
        server_host: Option<String>,
        server_port: Option<u16>,
        release: bool,
    ) -> Self {
        Self::new(GuiOptions {
            repo_path: PathBuf::from(".tachyon"),
            dev_tools,
            window_width,
            window_height,
            start_maximized,
            start_minimized,
            server_host,
            server_port,
            release,
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

    /// Locate the Tauri project directory.
    ///
    /// The desktop crate lives at `tachyon/crates/desktop/src-tauri/`.
    /// We resolve it relative to the current executable or the CARGO_MANIFEST_DIR.
    fn find_tauri_dir() -> CliResult<PathBuf> {
        // Try CARGO_MANIFEST_DIR first (set during `cargo run`)
        if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
            let tauri_dir = PathBuf::from(&manifest_dir)
                .join("..")
                .join("desktop")
                .join("src-tauri");
            if tauri_dir.exists() {
                return Ok(tauri_dir);
            }
        }

        // Try relative to current directory (for local development)
        let local = PathBuf::from("crates/desktop/src-tauri");
        if local.exists() {
            return Ok(local);
        }

        Err(CliError::server(
            "Cannot locate Tauri project directory. Run from the tachyon/ workspace root.",
        ))
    }

    /// Launch Tauri desktop application
    fn launch_tauri(&self, config: &TachyonConfig) -> CliResult<()> {
        let tauri_dir = Self::find_tauri_dir()?;

        let server_host = self.get_server_host(config);
        let server_port = self.get_server_port(config);

        // Set environment variables so the Tauri app can configure itself
        let api_url = format!("http://{}:{}/api/v1", server_host, server_port);

        println!("Launching Tachyon Desktop...");
        println!("");
        println!("Configuration:");
        println!("  Tauri project: {}", tauri_dir.display());
        println!("  API URL: {}", api_url);
        println!(
            "  Dev tools: {}",
            self.options.dev_tools || config.desktop.dev_tools
        );
        println!(
            "  Mode: {}",
            if self.options.release {
                "release"
            } else {
                "development"
            }
        );
        println!("");

        let mut cmd = ProcessCommand::new("cargo");

        if self.options.release {
            // In release mode, run the built binary directly
            // The binary is at target/release/tachyon-desktop-app
            let bin_path = tauri_dir.join("../../target/release/tachyon-desktop-app");
            if !bin_path.exists() {
                println!(
                    "Release binary not found at {}. Building...",
                    bin_path.display()
                );
                let build_status = ProcessCommand::new("cargo")
                    .args(["tauri", "build"])
                    .current_dir(&tauri_dir)
                    .status()
                    .map_err(|e| {
                        CliError::server(format!("Failed to spawn cargo tauri build: {}", e))
                    })?;

                if !build_status.success() {
                    return Err(CliError::server("cargo tauri build failed".to_string()));
                }
            }

            cmd = ProcessCommand::new(&bin_path);
        } else {
            // Development mode: `cargo tauri dev`
            cmd.args(["tauri", "dev"]);
            if self.options.dev_tools || config.desktop.dev_tools {
                cmd.args(["--", "--devtools"]);
            }
        }

        cmd.current_dir(&tauri_dir)
            .env("TACHYON_API_URL", &api_url)
            .env("TACHYON_SERVER_HOST", &server_host)
            .env("TACHYON_SERVER_PORT", server_port.to_string());

        println!("Starting Tauri application...");
        println!("(Press Ctrl+C in the Tauri window to exit)");
        println!("");

        let status = cmd
            .status()
            .map_err(|e| CliError::server(format!("Failed to launch Tauri: {}", e)))?;

        if status.success() {
            println!("Desktop application exited normally.");
        } else {
            eprintln!("Desktop application exited with code: {:?}", status.code());
        }

        Ok(())
    }
}

impl Command for GuiCommand {
    fn execute(&self) -> CliResult<()> {
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
        assert!(!options.release);
    }

    #[test]
    fn test_gui_command_from_args() {
        let cmd = GuiCommand::from_args(
            true,
            Some(1920),
            Some(1080),
            true,
            false,
            Some("localhost".to_string()),
            Some(9000),
            true,
        );

        assert_eq!(cmd.options.repo_path, PathBuf::from(".tachyon"));
        assert!(cmd.options.dev_tools);
        assert_eq!(cmd.options.window_width, Some(1920));
        assert_eq!(cmd.options.window_height, Some(1080));
        assert!(cmd.options.start_maximized);
        assert!(!cmd.options.start_minimized);
        assert_eq!(cmd.options.server_host, Some("localhost".to_string()));
        assert_eq!(cmd.options.server_port, Some(9000));
        assert!(cmd.options.release);
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
