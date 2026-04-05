// Build command for Tachyon CLI

use crate::commands::Command;
use crate::config::TachyonConfig;
use crate::error::{CliError, CliResult};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Options for build command
#[derive(Debug, Clone)]
pub struct BuildOptions {
    /// Repository path
    pub repo_path: PathBuf,

    /// Output directory
    pub output_dir: Option<PathBuf>,

    /// Generate documentation
    pub gen_docs: bool,

    /// Minify assets
    pub minify: bool,

    /// Generate source maps
    pub source_maps: bool,

    /// Clean build (remove output directory first)
    pub clean: bool,

    /// Verbose output
    pub verbose: bool,
}

impl Default for BuildOptions {
    fn default() -> Self {
        Self {
            repo_path: PathBuf::from(".tachyon"),
            output_dir: None,
            gen_docs: true,
            minify: false,
            source_maps: true,
            clean: false,
            verbose: false,
        }
    }
}

/// Build statistics
#[derive(Debug, Default)]
pub struct BuildStats {
    /// Number of files copied
    pub files_copied: usize,

    /// Number of documents generated
    pub docs_generated: usize,

    /// Number of assets bundled
    pub assets_bundled: usize,

    /// Build duration in seconds
    pub duration_secs: f64,
}

/// Build command handler
pub struct BuildCommand {
    options: BuildOptions,
}

impl BuildCommand {
    /// Create a new build command
    pub fn new(options: BuildOptions) -> Self {
        Self { options }
    }

    /// Create from clap arguments
    pub fn from_args(
        repo_path: Option<PathBuf>,
        output_dir: Option<PathBuf>,
        gen_docs: bool,
        minify: bool,
        source_maps: bool,
        clean: bool,
        verbose: bool,
    ) -> Self {
        Self::new(BuildOptions {
            repo_path: repo_path.unwrap_or_else(|| PathBuf::from(".tachyon")),
            output_dir,
            gen_docs,
            minify,
            source_maps,
            clean,
            verbose,
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

    /// Get effective output directory
    fn get_output_dir(&self, config: &TachyonConfig) -> PathBuf {
        self.options
            .output_dir
            .clone()
            .unwrap_or_else(|| config.build.output_dir.clone())
    }

    /// Get effective gen_docs setting
    fn get_gen_docs(&self, config: &TachyonConfig) -> bool {
        self.options.gen_docs || config.build.gen_docs
    }

    /// Get effective minify setting
    fn get_minify(&self, config: &TachyonConfig) -> bool {
        self.options.minify || config.build.minify
    }

    /// Get effective source_maps setting
    fn get_source_maps(&self, config: &TachyonConfig) -> bool {
        self.options.source_maps || config.build.source_maps
    }

    /// Clean output directory
    fn clean_output_dir(&self, output_dir: &Path) -> CliResult<()> {
        if output_dir.exists() {
            println!("Cleaning output directory: {}", output_dir.display());

            fs::remove_dir_all(output_dir).map_err(|e| {
                CliError::io(
                    output_dir,
                    format!("Failed to clean output directory: {}", e),
                )
            })?;

            println!("Output directory cleaned.");
        }

        Ok(())
    }

    /// Create output directory structure
    fn create_output_structure(&self, output_dir: &Path) -> CliResult<()> {
        let directories = vec![
            output_dir.join("docs"),
            output_dir.join("assets"),
            output_dir.join("static"),
            output_dir.join("css"),
            output_dir.join("js"),
        ];

        for dir in &directories {
            if !dir.exists() {
                fs::create_dir_all(dir)
                    .map_err(|e| CliError::io(dir, format!("Failed to create directory: {}", e)))?;
            }
        }

        Ok(())
    }

    /// Copy static files
    fn copy_static_files(&self, repo_path: &Path, output_dir: &Path) -> CliResult<usize> {
        let static_dir = output_dir.join("static");
        let mut count = 0;

        // Copy README
        let readme_src = repo_path.join("README.md");
        if readme_src.exists() {
            let readme_dst = static_dir.join("README.md");
            fs::copy(&readme_src, &readme_dst)
                .map_err(|e| CliError::io(&readme_dst, format!("Failed to copy README: {}", e)))?;
            count += 1;
        }

        // Copy gitignore
        let gitignore_src = repo_path.join(".gitignore");
        if gitignore_src.exists() {
            let gitignore_dst = static_dir.join(".gitignore");
            fs::copy(&gitignore_src, &gitignore_dst).map_err(|e| {
                CliError::io(&gitignore_dst, format!("Failed to copy .gitignore: {}", e))
            })?;
            count += 1;
        }

        Ok(count)
    }

    /// Generate documentation
    fn generate_docs(&self, _repo_path: &Path, output_dir: &Path) -> CliResult<usize> {
        let docs_dir = output_dir.join("docs");
        let mut count = 0;

        // Generate index.html
        let index_content = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Tachyon Documentation</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            max-width: 800px;
            margin: 0 auto;
            padding: 2rem;
            line-height: 1.6;
        }
        h1 { color: #333; border-bottom: 2px solid #007bff; padding-bottom: 0.5rem; }
        h2 { color: #555; margin-top: 2rem; }
        code { background: #f4f4f4; padding: 0.2rem 0.4rem; border-radius: 4px; }
        pre { background: #2d2d2d; color: #f4f4f4; padding: 1rem; border-radius: 4px; overflow-x: auto; }
    </style>
</head>
<body>
    <h1>Tachyon Documentation</h1>
    <p>Welcome to the Tachyon knowledge base documentation.</p>
    <p>This documentation was automatically generated by the Tachyon build system.</p>
    <h2>Getting Started</h2>
    <p>Tachyon is a high-performance knowledge base system built for managing complex information networks.</p>
    <h2>Documentation Structure</h2>
    <ul>
        <li>Nodes - Individual knowledge units</li>
        <li>Edges - Relationships between nodes</li>
        <li>Documents - Rich content with formatting</li>
    </ul>
</body>
</html>
"#;

        let index_path = docs_dir.join("index.html");
        fs::write(&index_path, index_content)
            .map_err(|e| CliError::io(&index_path, format!("Failed to write index.html: {}", e)))?;
        count += 1;

        // Generate API documentation placeholder
        let api_content = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>API Documentation</title>
    <link rel="stylesheet" href="../css/style.css">
</head>
<body>
    <h1>API Documentation</h1>
    <p>API documentation will be generated here.</p>
</body>
</html>
"#;

        let api_path = docs_dir.join("api.html");
        fs::write(&api_path, api_content)
            .map_err(|e| CliError::io(&api_path, format!("Failed to write api.html: {}", e)))?;
        count += 1;

        Ok(count)
    }

    /// Bundle assets
    fn bundle_assets(&self, repo_path: &Path, output_dir: &Path) -> CliResult<usize> {
        let assets_dir = output_dir.join("assets");
        let nodes_dir = repo_path.join("nodes");
        let documents_dir = repo_path.join("documents");
        let mut count = 0;

        // Copy nodes
        if nodes_dir.exists() {
            for entry in WalkDir::new(&nodes_dir).min_depth(1) {
                let entry = entry
                    .map_err(|e| CliError::generic(format!("Failed to walk directory: {}", e)))?;

                if entry.path().is_file() {
                    let file_name = entry.file_name();
                    let dst_path = assets_dir.join("nodes").join(file_name);

                    // Create parent directory if needed
                    if let Some(parent) = dst_path.parent() {
                        fs::create_dir_all(parent).map_err(|e| {
                            CliError::io(parent, format!("Failed to create directory: {}", e))
                        })?;
                    }

                    fs::copy(entry.path(), &dst_path).map_err(|e| {
                        CliError::io(
                            &dst_path,
                            format!("Failed to copy {}: {}", entry.path().display(), e),
                        )
                    })?;
                    count += 1;
                }
            }
        }

        // Copy documents
        if documents_dir.exists() {
            for entry in WalkDir::new(&documents_dir).min_depth(1) {
                let entry = entry
                    .map_err(|e| CliError::generic(format!("Failed to walk directory: {}", e)))?;

                if entry.path().is_file() {
                    let file_name = entry.file_name();
                    let dst_path = assets_dir.join("documents").join(file_name);

                    // Create parent directory if needed
                    if let Some(parent) = dst_path.parent() {
                        fs::create_dir_all(parent).map_err(|e| {
                            CliError::io(parent, format!("Failed to create directory: {}", e))
                        })?;
                    }

                    fs::copy(entry.path(), &dst_path).map_err(|e| {
                        CliError::io(
                            &dst_path,
                            format!("Failed to copy {}: {}", entry.path().display(), e),
                        )
                    })?;
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    /// Generate CSS
    fn generate_css(&self, output_dir: &Path) -> CliResult<()> {
        let css_dir = output_dir.join("css");
        fs::create_dir_all(&css_dir)?;
        let style_content = r#"/* Tachyon Base Styles */
:root {
    --primary-color: #007bff;
    --secondary-color: #6c757d;
    --success-color: #28a745;
    --danger-color: #dc3545;
    --warning-color: #ffc107;
    --info-color: #17a2b8;
    --light-color: #f8f9fa;
    --dark-color: #343a40;
    --border-color: #dee2e6;
    --font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    --font-size-base: 16px;
    --line-height: 1.6;
}

* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: var(--font-family);
    font-size: var(--font-size-base);
    line-height: var(--line-height);
    color: var(--dark-color);
    background-color: #fff;
}

a {
    color: var(--primary-color);
    text-decoration: none;
}

a:hover {
    text-decoration: underline;
}

code {
    font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
    background-color: var(--light-color);
    padding: 0.2rem 0.4rem;
    border-radius: 4px;
    font-size: 0.9em;
}

pre {
    background-color: var(--dark-color);
    color: #f8f9fa;
    padding: 1rem;
    border-radius: 4px;
    overflow-x: auto;
}

pre code {
    background-color: transparent;
    padding: 0;
    color: inherit;
}
"#;

        let style_path = css_dir.join("style.css");
        fs::write(&style_path, style_content)
            .map_err(|e| CliError::io(&style_path, format!("Failed to write style.css: {}", e)))?;

        Ok(())
    }

    /// Generate JavaScript
    fn generate_js(&self, output_dir: &Path) -> CliResult<()> {
        let js_dir = output_dir.join("js");
        fs::create_dir_all(&js_dir)?;
        let app_content = r#"// Tachyon Base JavaScript
(function() {
    'use strict';

    const Tachyon = {
        version: '0.1.0',
        
        init: function() {
            console.log('Tachyon v' + this.version + ' initialized');
        },
        
        // Utility functions
        debounce: function(func, wait) {
            let timeout;
            return function executedFunction(...args) {
                const later = () => {
                    clearTimeout(timeout);
                    func(...args);
                };
                clearTimeout(timeout);
                timeout = setTimeout(later, wait);
            };
        },
        
        throttle: function(func, limit) {
            let inThrottle;
            return function executedFunction(...args) {
                if (!inThrottle) {
                    func(...args);
                    inThrottle = true;
                    setTimeout(() => inThrottle = false, limit);
                }
            };
        }
    };

    // Initialize when DOM is ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', Tachyon.init.bind(Tachyon));
    } else {
        Tachyon.init();
    }

    // Export to global scope
    window.Tachyon = Tachyon;
})();
"#;

        let app_path = js_dir.join("app.js");
        fs::write(&app_path, app_content)
            .map_err(|e| CliError::io(&app_path, format!("Failed to write app.js: {}", e)))?;

        Ok(())
    }
}

impl Command for BuildCommand {
    fn execute(&self) -> CliResult<()> {
        // Validate repository path
        self.validate_repo_path()?;

        // Load configuration
        let config = self.load_config()?;

        // Get effective settings
        let output_dir = self.get_output_dir(&config);
        let gen_docs = self.get_gen_docs(&config);
        let minify = self.get_minify(&config);
        let source_maps = self.get_source_maps(&config);

        println!("");
        println!("Tachyon Build");
        println!("===============");
        println!("Repository: {}", self.options.repo_path.display());
        println!("Output: {}", output_dir.display());
        println!("Generate docs: {}", gen_docs);
        println!("Minify: {}", minify);
        println!("Source maps: {}", source_maps);
        println!("");

        // Clean output directory if requested
        if self.options.clean {
            self.clean_output_dir(&output_dir)?;
            println!();
        }

        // Create output directory structure
        println!("Creating output structure...");
        self.create_output_structure(&output_dir)?;
        println!("Output structure created.");
        println!("");

        // Copy static files
        println!("Copying static files...");
        let files_copied = self.copy_static_files(&self.options.repo_path, &output_dir)?;
        println!("Copied {} static file(s).", files_copied);
        println!("");

        // Generate documentation
        if gen_docs {
            println!("Generating documentation...");
            let docs_generated = self.generate_docs(&self.options.repo_path, &output_dir)?;
            println!("Generated {} documentation file(s).", docs_generated);
            println!("");
        }

        // Bundle assets
        println!("Bundling assets...");
        let assets_bundled = self.bundle_assets(&self.options.repo_path, &output_dir)?;
        println!("Bundled {} asset(s).", assets_bundled);
        println!("");

        // Generate CSS
        println!("Generating CSS...");
        self.generate_css(&output_dir)?;
        println!("CSS generated.");
        println!("");

        // Generate JavaScript
        println!("Generating JavaScript...");
        self.generate_js(&output_dir)?;
        println!("JavaScript generated.");
        println!("");

        // Summary
        println!("Build completed successfully!");
        println!("");
        println!("Summary:");
        println!("  Files copied: {}", files_copied);
        println!("  Assets bundled: {}", assets_bundled);
        println!("  Output directory: {}", output_dir.display());
        println!("");

        Ok(())
    }

    fn name(&self) -> &str {
        "build"
    }

    fn description(&self) -> &str {
        "Build documentation and bundle assets"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_build_options_default() {
        let options = BuildOptions::default();
        assert_eq!(options.repo_path, PathBuf::from(".tachyon"));
        assert!(options.gen_docs);
        assert!(!options.minify);
        assert!(options.source_maps);
        assert!(!options.clean);
        assert!(!options.verbose);
    }

    #[test]
    fn test_build_command_from_args() {
        let cmd = BuildCommand::from_args(
            Some(PathBuf::from("/tmp/test")),
            Some(PathBuf::from("/tmp/dist")),
            false,
            true,
            false,
            true,
            true,
        );

        assert_eq!(cmd.options.repo_path, PathBuf::from("/tmp/test"));
        assert_eq!(cmd.options.output_dir, Some(PathBuf::from("/tmp/dist")));
        assert!(!cmd.options.gen_docs);
        assert!(cmd.options.minify);
        assert!(!cmd.options.source_maps);
        assert!(cmd.options.clean);
        assert!(cmd.options.verbose);
    }

    #[test]
    fn test_validate_repo_path_not_exists() {
        let options = BuildOptions {
            repo_path: PathBuf::from("/nonexistent/path"),
            ..Default::default()
        };
        let cmd = BuildCommand::new(options);

        let result = cmd.validate_repo_path();
        assert!(result.is_err());
    }

    #[test]
    fn test_create_output_structure() {
        let dir = tempdir().unwrap();
        let output_dir = dir.path().join("dist");
        let options = BuildOptions::default();
        let cmd = BuildCommand::new(options);

        cmd.create_output_structure(&output_dir).unwrap();

        assert!(output_dir.join("docs").exists());
        assert!(output_dir.join("assets").exists());
        assert!(output_dir.join("static").exists());
        assert!(output_dir.join("css").exists());
        assert!(output_dir.join("js").exists());
    }

    #[test]
    fn test_clean_output_dir() {
        let dir = tempdir().unwrap();
        let output_dir = dir.path().join("dist");

        // Create some files
        fs::create_dir_all(&output_dir).unwrap();
        fs::write(output_dir.join("test.txt"), "test").unwrap();

        let options = BuildOptions::default();
        let cmd = BuildCommand::new(options);

        cmd.clean_output_dir(&output_dir).unwrap();

        assert!(!output_dir.exists());
    }

    #[test]
    fn test_generate_css() {
        let dir = tempdir().unwrap();
        let output_dir = dir.path().join("dist");
        let options = BuildOptions::default();
        let cmd = BuildCommand::new(options);

        cmd.generate_css(&output_dir).unwrap();

        let css_path = output_dir.join("css").join("style.css");
        assert!(css_path.exists());
        let content = fs::read_to_string(&css_path).unwrap();
        assert!(content.contains("--primary-color:"));
        assert!(content.contains("/* Tachyon Base Styles */"));
    }

    #[test]
    fn test_generate_js() {
        let dir = tempdir().unwrap();
        let output_dir = dir.path().join("dist");
        let options = BuildOptions::default();
        let cmd = BuildCommand::new(options);

        cmd.generate_js(&output_dir).unwrap();

        let js_path = output_dir.join("js").join("app.js");
        assert!(js_path.exists());
        let content = fs::read_to_string(&js_path).unwrap();
        assert!(content.contains("Tachyon"));
        assert!(content.contains("init: function()"));
    }

    #[test]
    fn test_build_stats_default() {
        let stats = BuildStats::default();
        assert_eq!(stats.files_copied, 0);
        assert_eq!(stats.docs_generated, 0);
        assert_eq!(stats.assets_bundled, 0);
        assert_eq!(stats.duration_secs, 0.0);
    }
}
