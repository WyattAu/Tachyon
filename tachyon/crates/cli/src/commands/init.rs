// Init command for Tachyon CLI

use crate::commands::Command;
use crate::config::{TachyonConfig, DEFAULT_REPO_DIR};
use crate::error::{CliError, CliResult};
use git2::{Repository, RepositoryInitOptions};
use std::fs;
use std::path::{Path, PathBuf};
use tachyon_database::init_with_migrations;

/// Options for init command
#[derive(Debug, Clone)]
pub struct InitOptions {
    /// Path to initialize repository at
    pub path: PathBuf,

    /// Repository name
    pub name: Option<String>,

    /// Skip git initialization
    pub skip_git: bool,

    /// Skip database initialization
    pub skip_database: bool,

    /// Force initialization even if directory exists
    pub force: bool,

    /// Interactive mode
    pub interactive: bool,
}

impl Default for InitOptions {
    fn default() -> Self {
        Self {
            path: PathBuf::from(DEFAULT_REPO_DIR),
            name: None,
            skip_git: false,
            skip_database: false,
            force: false,
            interactive: false,
        }
    }
}

/// Init command handler
pub struct InitCommand {
    options: InitOptions,
}

impl InitCommand {
    /// Create a new init command
    pub fn new(options: InitOptions) -> Self {
        Self { options }
    }

    /// Create from clap arguments
    pub fn from_args(
        path: Option<PathBuf>,
        name: Option<String>,
        skip_git: bool,
        skip_database: bool,
        force: bool,
        interactive: bool,
    ) -> Self {
        Self::new(InitOptions {
            path: path.unwrap_or_else(|| PathBuf::from(DEFAULT_REPO_DIR)),
            name,
            skip_git,
            skip_database,
            force,
            interactive,
        })
    }

    /// Create default directory structure
    fn create_directory_structure(&self, base_path: &Path) -> CliResult<()> {
        let directories = vec![
            base_path.join("nodes"),
            base_path.join("edges"),
            base_path.join("documents"),
            base_path.join("db"),
            base_path.join("cache"),
            base_path.join("logs"),
            base_path.join("backup"),
        ];

        for dir in directories {
            if !dir.exists() {
                fs::create_dir_all(&dir).map_err(|e| {
                    CliError::io(&dir, format!("Failed to create directory: {}", e))
                })?;
            }
        }

        Ok(())
    }

    /// Create .gitignore file
    fn create_gitignore(&self, repo_path: &Path) -> CliResult<()> {
        let gitignore_path = repo_path.join(".gitignore");
        let content = r#"# Tachyon repository files
*.db
*.db-shm
*.db-wal
*.log
cache/
backup/
logs/

# OS files
.DS_Store
Thumbs.db

# Editor files
.vscode/
.idea/
*.swp
*.swo
*~

# Build artifacts
dist/
target/
"#;

        fs::write(&gitignore_path, content).map_err(|e| {
            CliError::io(
                &gitignore_path,
                format!("Failed to create .gitignore: {}", e),
            )
        })?;

        Ok(())
    }

    /// Create README file
    fn create_readme(&self, repo_path: &Path, name: &str) -> CliResult<()> {
        let readme_path = repo_path.join("README.md");
        let content = format!(
            r#"# {name}

This is a Tachyon knowledge base repository.

## Getting Started

Run `tachyon serve` to start the server, or `tachyon gui` to launch the desktop application.

## Documentation

For more information, visit the Tachyon documentation.

## Repository Structure

- `nodes/` - Node data files
- `edges/` - Edge relationship files
- `documents/` - Document content files
- `db/` - Database files
- `cache/` - Cached data
- `logs/` - Log files
- `backup/` - Database backups
"#,
            name = name
        );

        fs::write(&readme_path, content)
            .map_err(|e| CliError::io(&readme_path, format!("Failed to create README: {}", e)))?;

        Ok(())
    }

    /// Initialize git repository
    fn init_git(&self, repo_path: &Path) -> CliResult<()> {
        let mut opts = RepositoryInitOptions::new();
        opts.no_reinit(true);

        Repository::init_opts(repo_path, &opts)
            .map_err(|e| CliError::git(format!("Failed to initialize git repository: {}", e)))?;

        Ok(())
    }

    /// Initialize database
    fn init_database(&self, repo_path: &Path) -> CliResult<()> {
        let db_path = repo_path.join("db").join("tachyon.db");
        let db_dir = db_path
            .parent()
            .ok_or_else(|| CliError::database("Invalid database path".to_string()))?;

        if !db_dir.exists() {
            fs::create_dir_all(db_dir).map_err(|e| {
                CliError::io(
                    db_dir,
                    format!("Failed to create database directory: {}", e),
                )
            })?;
        }

        // Try to initialize the database
        let db_url = format!("sqlite:{}", db_path.to_string_lossy());
        match tokio::runtime::Runtime::new()
            .map_err(|e| CliError::database(format!("Failed to create tokio runtime: {}", e)))?
            .block_on(init_with_migrations(&db_url))
        {
            Ok(_) => Ok(()),
            Err(e) => {
                // Check if this is a "database already exists" type error
                let error_str = e.to_string();
                if error_str.contains("already exists") || error_str.contains("UNIQUE constraint") {
                    // Database already exists, that's okay
                    Ok(())
                } else {
                    Err(CliError::database(format!(
                        "Failed to initialize database: {}",
                        e
                    )))
                }
            }
        }
    }

    /// Interactive setup wizard
    #[allow(dead_code)]
    fn interactive_setup(&mut self) -> CliResult<String> {
        let mut name = self.options.name.clone();

        if name.is_none() {
            // Prompt for repository name
            let default_name = "My Knowledge Base";
            println!("Repository name [{}]: ", default_name);
            name = Some(default_name.to_string());
        }

        // For now, return the name
        // In a full implementation, we would use a proper prompt library like dialoguer
        Ok(name.unwrap_or_else(|| "My Knowledge Base".to_string()))
    }

    /// Check if path is empty or can be initialized
    fn check_path(&self, path: &Path) -> CliResult<bool> {
        if !path.exists() {
            return Ok(true);
        }

        if self.options.force {
            return Ok(true);
        }

        let entries: Vec<_> = fs::read_dir(path)
            .map_err(|e| CliError::io(path, format!("Failed to read directory: {}", e)))?
            .filter_map(|e| e.ok())
            .collect();

        if entries.is_empty() {
            return Ok(true);
        }

        Err(CliError::init_failed(format!(
            "Directory '{}' is not empty. Use --force to initialize anyway.",
            path.display()
        )))
    }
}

impl Command for InitCommand {
    fn execute(&self) -> CliResult<()> {
        let repo_path = &self.options.path;

        println!(
            "Initializing Tachyon repository at: {}",
            repo_path.display()
        );

        // Check if path can be initialized
        self.check_path(repo_path)?;

        // Create base directory if it doesn't exist
        if !repo_path.exists() {
            fs::create_dir_all(repo_path).map_err(|e| {
                CliError::io(
                    repo_path,
                    format!("Failed to create repository directory: {}", e),
                )
            })?;
        }

        // Get repository name
        let name = self
            .options
            .name
            .clone()
            .unwrap_or_else(|| "My Knowledge Base".to_string());

        // Create directory structure
        println!("Creating directory structure...");
        self.create_directory_structure(repo_path)?;

        // Initialize git repository
        if !self.options.skip_git {
            println!("Initializing git repository...");
            self.init_git(repo_path)?;
        }

        // Initialize database
        if !self.options.skip_database {
            println!("Initializing database...");
            self.init_database(repo_path)?;
        }

        // Create configuration file
        println!("Creating configuration file...");
        let config = TachyonConfig::default();
        config.save_to_file(&repo_path.join("tachyon.toml"))?;

        // Create .gitignore
        println!("Creating .gitignore...");
        self.create_gitignore(repo_path)?;

        // Create README
        println!("Creating README...");
        self.create_readme(repo_path, &name)?;

        println!("");
        println!("Repository initialized successfully!");
        println!("");
        println!("Next steps:");
        println!("  Run 'tachyon serve' to start the server");
        println!("  Run 'tachyon gui' to launch the desktop application");
        println!("  Run 'tachyon build' to build documentation");

        Ok(())
    }

    fn name(&self) -> &str {
        "init"
    }

    fn description(&self) -> &str {
        "Initialize a new Tachyon repository"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_init_command_default_options() {
        let options = InitOptions::default();
        assert_eq!(options.path, PathBuf::from(DEFAULT_REPO_DIR));
        assert!(options.name.is_none());
        assert!(!options.skip_git);
        assert!(!options.skip_database);
        assert!(!options.force);
    }

    #[test]
    fn test_init_command_from_args() {
        let cmd = InitCommand::from_args(
            Some(PathBuf::from("/tmp/test")),
            Some("Test Repo".to_string()),
            true,
            false,
            false,
            false,
        );

        assert_eq!(cmd.options.path, PathBuf::from("/tmp/test"));
        assert_eq!(cmd.options.name, Some("Test Repo".to_string()));
        assert!(cmd.options.skip_git);
    }

    #[test]
    fn test_create_directory_structure() {
        let dir = tempdir().unwrap();
        let repo_path = dir.path();
        let options = InitOptions {
            path: repo_path.to_path_buf(),
            name: None,
            skip_git: true,
            skip_database: true,
            force: true,
            interactive: false,
        };
        let cmd = InitCommand::new(options);

        cmd.create_directory_structure(repo_path).unwrap();

        assert!(repo_path.join("nodes").exists());
        assert!(repo_path.join("edges").exists());
        assert!(repo_path.join("documents").exists());
        assert!(repo_path.join("db").exists());
    }

    #[test]
    fn test_create_gitignore() {
        let dir = tempdir().unwrap();
        let repo_path = dir.path();
        let options = InitOptions {
            path: repo_path.to_path_buf(),
            name: None,
            skip_git: true,
            skip_database: true,
            force: true,
            interactive: false,
        };
        let cmd = InitCommand::new(options);

        cmd.create_gitignore(repo_path).unwrap();

        let gitignore_path = repo_path.join(".gitignore");
        assert!(gitignore_path.exists());
        let content = fs::read_to_string(&gitignore_path).unwrap();
        assert!(content.contains("*.db"));
        assert!(content.contains("cache/"));
    }

    #[test]
    fn test_create_readme() {
        let dir = tempdir().unwrap();
        let repo_path = dir.path();
        let options = InitOptions {
            path: repo_path.to_path_buf(),
            name: None,
            skip_git: true,
            skip_database: true,
            force: true,
            interactive: false,
        };
        let cmd = InitCommand::new(options);

        cmd.create_readme(repo_path, "Test Repository").unwrap();

        let readme_path = repo_path.join("README.md");
        assert!(readme_path.exists());
        let content = fs::read_to_string(&readme_path).unwrap();
        assert!(content.contains("Test Repository"));
    }

    #[test]
    fn test_check_path_empty() {
        let dir = tempdir().unwrap();
        let options = InitOptions::default();
        let cmd = InitCommand::new(options);

        assert!(cmd.check_path(dir.path()).is_ok());
    }

    #[test]
    fn test_check_path_not_empty_without_force() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("test.txt"), "test").unwrap();

        let options = InitOptions {
            path: dir.path().to_path_buf(),
            name: None,
            skip_git: true,
            skip_database: true,
            force: false,
            interactive: false,
        };
        let cmd = InitCommand::new(options);

        assert!(cmd.check_path(dir.path()).is_err());
    }

    #[test]
    fn test_check_path_not_empty_with_force() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("test.txt"), "test").unwrap();

        let options = InitOptions {
            path: dir.path().to_path_buf(),
            name: None,
            skip_git: true,
            skip_database: true,
            force: true,
            interactive: false,
        };
        let cmd = InitCommand::new(options);

        assert!(cmd.check_path(dir.path()).is_ok());
    }
}
