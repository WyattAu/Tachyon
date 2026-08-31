// Configuration handling for Tachyon CLI

use crate::error::{CliError, CliResult};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Default configuration file name
pub const DEFAULT_CONFIG_FILE: &str = "tachyon.toml";

/// Default repository directory name
pub const DEFAULT_REPO_DIR: &str = ".tachyon";

/// Default database file name
pub const DEFAULT_DB_FILE: &str = "tachyon.db";

/// Default host for server
pub const DEFAULT_HOST: &str = "127.0.0.1";

/// Default port for HTTP server
pub const DEFAULT_HTTP_PORT: u16 = 8080;

/// Default port for WebSocket server
pub const DEFAULT_WS_PORT: u16 = 8081;

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Host address to bind to
    #[serde(default = "default_host")]
    pub host: String,

    /// HTTP port
    #[serde(default = "default_http_port")]
    pub http_port: u16,

    /// WebSocket port
    #[serde(default = "default_ws_port")]
    pub ws_port: u16,

    /// Enable TLS
    #[serde(default)]
    pub tls_enabled: bool,

    /// TLS certificate path
    #[serde(default)]
    pub tls_cert: Option<String>,

    /// TLS key path
    #[serde(default)]
    pub tls_key: Option<String>,

    /// Maximum request body size in bytes
    #[serde(default = "default_max_body_size")]
    pub max_body_size: usize,

    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,
}

/// Desktop configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopConfig {
    /// Enable dev tools
    #[serde(default)]
    pub dev_tools: bool,

    /// Window width
    #[serde(default = "default_window_width")]
    pub window_width: u32,

    /// Window height
    #[serde(default = "default_window_height")]
    pub window_height: u32,

    /// Start maximized
    #[serde(default)]
    pub start_maximized: bool,

    /// Start minimized
    #[serde(default)]
    pub start_minimized: bool,
}

/// Build configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    /// Output directory
    #[serde(default = "default_output_dir")]
    pub output_dir: PathBuf,

    /// Generate documentation
    #[serde(default = "default_gen_docs")]
    pub gen_docs: bool,

    /// Minify assets
    #[serde(default = "default_minify")]
    pub minify: bool,

    /// Source map generation
    #[serde(default = "default_source_maps")]
    pub source_maps: bool,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level: trace, debug, info, warn, error
    #[serde(default = "default_log_level")]
    pub level: String,

    /// Log to file
    #[serde(default)]
    pub log_to_file: bool,

    /// Log file path
    #[serde(default)]
    pub log_file: Option<PathBuf>,

    /// Log format: json, pretty, compact
    #[serde(default = "default_log_format")]
    pub format: String,
}

/// Main Tachyon configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TachyonConfig {
    /// Server configuration
    #[serde(default)]
    pub server: ServerConfig,

    /// Desktop configuration
    #[serde(default)]
    pub desktop: DesktopConfig,

    /// Build configuration
    #[serde(default)]
    pub build: BuildConfig,

    /// Logging configuration
    #[serde(default)]
    pub logging: LoggingConfig,

    /// Repository directory
    #[serde(default = "default_repo_dir")]
    pub repo_dir: PathBuf,

    /// Database directory
    #[serde(default)]
    pub db_dir: Option<PathBuf>,
}

impl Default for TachyonConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            desktop: DesktopConfig::default(),
            build: BuildConfig::default(),
            logging: LoggingConfig::default(),
            repo_dir: default_repo_dir(),
            db_dir: None,
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            http_port: default_http_port(),
            ws_port: default_ws_port(),
            tls_enabled: false,
            tls_cert: None,
            tls_key: None,
            max_body_size: default_max_body_size(),
            timeout: default_timeout(),
        }
    }
}

impl Default for DesktopConfig {
    fn default() -> Self {
        Self {
            dev_tools: false,
            window_width: default_window_width(),
            window_height: default_window_height(),
            start_maximized: false,
            start_minimized: false,
        }
    }
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            output_dir: default_output_dir(),
            gen_docs: default_gen_docs(),
            minify: default_minify(),
            source_maps: default_source_maps(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            log_to_file: false,
            log_file: None,
            format: default_log_format(),
        }
    }
}

// Default value functions
fn default_host() -> String {
    DEFAULT_HOST.to_string()
}

fn default_http_port() -> u16 {
    DEFAULT_HTTP_PORT
}

fn default_ws_port() -> u16 {
    DEFAULT_WS_PORT
}

fn default_max_body_size() -> usize {
    10 * 1024 * 1024 // 10MB
}

fn default_timeout() -> u64 {
    30
}

fn default_window_width() -> u32 {
    1280
}

fn default_window_height() -> u32 {
    720
}

fn default_output_dir() -> PathBuf {
    PathBuf::from("dist")
}

fn default_gen_docs() -> bool {
    true
}

fn default_minify() -> bool {
    false
}

fn default_source_maps() -> bool {
    true
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_format() -> String {
    "pretty".to_string()
}

fn default_repo_dir() -> PathBuf {
    PathBuf::from(DEFAULT_REPO_DIR)
}

impl TachyonConfig {
    /// Load configuration from file
    pub fn load_from_file(path: &Path) -> CliResult<Self> {
        if !path.exists() {
            return Err(CliError::config(format!(
                "Configuration file not found: {}",
                path.display()
            )));
        }

        let content = fs::read_to_string(path)
            .map_err(|e| CliError::io(path, format!("Failed to read configuration file: {}", e)))?;

        let config: TachyonConfig = toml::from_str(&content)
            .map_err(|e| CliError::config(format!("Failed to parse configuration file: {}", e)))?;

        Ok(config)
    }

    /// Save configuration to file
    pub fn save_to_file(&self, path: &Path) -> CliResult<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| CliError::config(format!("Failed to serialize configuration: {}", e)))?;

        fs::write(path, content).map_err(|e| {
            CliError::io(path, format!("Failed to write configuration file: {}", e))
        })?;

        Ok(())
    }

    /// Find configuration file in current directory or parent directories
    pub fn find_config() -> CliResult<Option<PathBuf>> {
        let mut current = env::current_dir()
            .map_err(|e| CliError::generic(format!("Failed to get current directory: {}", e)))?;

        loop {
            let config_path = current.join(DEFAULT_CONFIG_FILE);
            if config_path.exists() {
                return Ok(Some(config_path));
            }

            if let Some(parent) = current.parent() {
                if parent == current {
                    break;
                }
                current = parent.to_path_buf();
            } else {
                break;
            }
        }

        Ok(None)
    }

    /// Load configuration, searching for config file or using defaults
    pub fn load() -> CliResult<Self> {
        if let Some(config_path) = Self::find_config()? {
            Self::load_from_file(&config_path)
        } else {
            Ok(Self::default())
        }
    }

    /// Get database directory
    pub fn db_dir(&self) -> PathBuf {
        self.db_dir
            .clone()
            .unwrap_or_else(|| self.repo_dir.join("db"))
    }

    /// Get database file path
    pub fn db_file(&self) -> PathBuf {
        self.db_dir().join(DEFAULT_DB_FILE)
    }
}

/// Initialize default configuration
pub fn init_config() -> CliResult<TachyonConfig> {
    let config = TachyonConfig::default();
    let config_path = PathBuf::from(DEFAULT_CONFIG_FILE);

    config.save_to_file(&config_path)?;

    Ok(config)
}
