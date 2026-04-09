// Tachyon CLI - Main Entry Point

use clap::{Parser, Subcommand};
use tachyon_cli::commands::{BuildCommand, Command, GuiCommand, InitCommand, ServeCommand};
use tachyon_cli::{CliError, CliResult, VERSION};

/// Tachyon Knowledge Base CLI
#[derive(Parser)]
#[command(name = "tachyon")]
#[command(about = "Command-line interface for Tachyon knowledge base system", long_about = None)]
#[command(version = VERSION)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Available CLI commands
#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Tachyon repository
    Init {
        /// Path to initialize repository at
        #[arg(short, long, value_name = "PATH")]
        path: Option<std::path::PathBuf>,

        /// Repository name
        #[arg(short, long, value_name = "NAME")]
        name: Option<String>,

        /// Skip git initialization
        #[arg(long)]
        skip_git: bool,

        /// Skip database initialization
        #[arg(long)]
        skip_database: bool,

        /// Force initialization even if directory exists
        #[arg(long)]
        force: bool,

        /// Interactive setup wizard
        #[arg(long)]
        interactive: bool,
    },

    /// Start HTTP/2 and WebSocket servers
    Serve {
        /// Host address to bind to
        #[arg(short, long, value_name = "HOST", default_value = "127.0.0.1")]
        host: String,

        /// HTTP port
        #[arg(short = 'p', long, value_name = "PORT", default_value = "8080")]
        http_port: u16,

        /// WebSocket port
        #[arg(long, value_name = "PORT", default_value = "8081")]
        ws_port: u16,

        /// Enable TLS
        #[arg(long)]
        tls_enabled: bool,

        /// TLS certificate path
        #[arg(long, value_name = "PATH")]
        tls_cert: Option<String>,

        /// TLS key path
        #[arg(long, value_name = "PATH")]
        tls_key: Option<String>,

        /// Repository path
        #[arg(short, long, value_name = "PATH", default_value = ".tachyon")]
        repo_path: std::path::PathBuf,

        /// Maximum request body size in bytes
        #[arg(long, value_name = "SIZE")]
        max_body_size: Option<usize>,

        /// Request timeout in seconds
        #[arg(long, value_name = "SECONDS")]
        timeout: Option<u64>,

        /// Enable file watching and auto-sync
        #[arg(long)]
        watch: bool,

        /// Path to watch for changes (defaults to --repo-path or current directory)
        #[arg(long, value_name = "PATH")]
        watch_path: Option<std::path::PathBuf>,
    },

    /// Launch Tauri desktop application
    Gui {
        /// Repository path
        #[arg(short, long, value_name = "PATH", default_value = ".tachyon")]
        repo_path: std::path::PathBuf,

        /// Enable dev tools
        #[arg(long)]
        dev_tools: bool,

        /// Window width
        #[arg(long, value_name = "WIDTH")]
        window_width: Option<u32>,

        /// Window height
        #[arg(long, value_name = "HEIGHT")]
        window_height: Option<u32>,

        /// Start maximized
        #[arg(long)]
        start_maximized: bool,

        /// Start minimized
        #[arg(long)]
        start_minimized: bool,

        /// Server host to connect to
        #[arg(long, value_name = "HOST")]
        server_host: Option<String>,

        /// Server port to connect to
        #[arg(long, value_name = "PORT")]
        server_port: Option<u16>,
    },

    /// Build static site from database documents
    Build {
        /// Path to the content repository
        #[arg(long, default_value = ".")]
        repo_path: std::path::PathBuf,

        /// Output directory for generated site
        #[arg(short, long, default_value = "dist")]
        output_dir: std::path::PathBuf,

        /// Database URL (PostgreSQL connection string)
        #[arg(long)]
        database_url: Option<String>,

        /// Site title
        #[arg(long, default_value = "Tachyon Docs")]
        site_title: String,

        /// Site description
        #[arg(long, default_value = "Knowledge Management System")]
        site_description: String,

        /// Base URL for canonical links and sitemap
        #[arg(long, default_value = "/")]
        base_url: String,

        /// Only build published documents
        #[arg(long)]
        published_only: bool,

        /// Clean output directory before building
        #[arg(long)]
        clean: bool,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,

        /// Custom template directory
        #[arg(long, value_name = "PATH")]
        template: Option<std::path::PathBuf>,
    },
}

/// Initialize logging
fn init_logging() -> CliResult<()> {
    use tracing_subscriber::{fmt, EnvFilter};

    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("tachyon_cli=info"));

    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .try_init()
        .map_err(|e| CliError::generic(format!("Failed to initialize logging: {}", e)))?;

    Ok(())
}

/// Main entry point
fn main() {
    // Parse command-line arguments
    let cli = Cli::parse();

    // Initialize logging
    if let Err(e) = init_logging() {
        eprintln!("Failed to initialize logging: {}", e);
        std::process::exit(1);
    }

    // Execute command
    let result = match cli.command {
        Commands::Init {
            path,
            name,
            skip_git,
            skip_database,
            force,
            interactive,
        } => {
            let cmd =
                InitCommand::from_args(path, name, skip_git, skip_database, force, interactive);
            cmd.execute()
        }

        Commands::Serve {
            host,
            http_port,
            ws_port,
            tls_enabled,
            tls_cert,
            tls_key,
            repo_path,
            max_body_size,
            timeout,
            watch,
            watch_path,
        } => {
            let cmd = ServeCommand::from_args(
                Some(host),
                Some(http_port),
                Some(ws_port),
                tls_enabled,
                tls_cert,
                tls_key,
                Some(repo_path),
                max_body_size,
                timeout,
                watch,
                watch_path,
            );
            cmd.execute()
        }

        Commands::Gui {
            repo_path,
            dev_tools,
            window_width,
            window_height,
            start_maximized,
            start_minimized,
            server_host,
            server_port,
        } => {
            let cmd = GuiCommand::from_args(
                Some(repo_path),
                dev_tools,
                window_width,
                window_height,
                start_maximized,
                start_minimized,
                server_host,
                server_port,
            );
            cmd.execute()
        }

        Commands::Build {
            repo_path,
            output_dir,
            database_url,
            site_title,
            site_description,
            base_url,
            published_only,
            clean,
            verbose,
            template,
        } => {
            let cmd = BuildCommand::from_args(
                Some(repo_path),
                Some(output_dir),
                database_url,
                Some(site_title),
                Some(site_description),
                Some(base_url),
                published_only,
                clean,
                verbose,
                template,
            );
            cmd.execute()
        }
    };

    // Handle result
    match result {
        Ok(()) => {
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(e.exit_code());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_version() {
        assert!(!VERSION.is_empty());
    }
}
