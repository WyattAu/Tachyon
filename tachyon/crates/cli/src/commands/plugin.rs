use crate::commands::Command;
use crate::error::{CliError, CliResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const REGISTRY_URL: &str = "https://registry.tachyon.dev";
const DEFAULT_PLUGINS_DIR: &str = ".tachyon/plugins";

#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: String,
    pub extension_points: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RegistryIndex {
    version: String,
    updated_at: String,
    plugins: Vec<RegistryPluginEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RegistryPluginEntry {
    id: String,
    name: String,
    description: String,
    author: String,
    version: String,
    license: String,
    download_url: String,
    checksum: String,
    size_bytes: u64,
    min_tachyon_version: String,
    extension_points: Vec<String>,
    tags: Vec<String>,
    featured: bool,
}

/// Plugin list command
pub struct PluginListCommand;

impl PluginListCommand {
    pub fn new() -> Self {
        Self
    }

    fn find_installed_plugins(&self) -> CliResult<Vec<PluginInfo>> {
        let plugins_dir = self.resolve_plugins_dir()?;
        if !plugins_dir.exists() {
            return Ok(vec![]);
        }

        let mut plugins = Vec::new();
        for entry in fs::read_dir(&plugins_dir).map_err(|e| {
            CliError::filesystem(format!(
                "Failed to read plugins dir {}: {}",
                plugins_dir.display(),
                e
            ))
        })? {
            let entry = entry.map_err(|e| CliError::filesystem(e.to_string()))?;
            let path = entry.path();
            if path.is_dir() {
                let manifest_path = path.join("plugin.json");
                if manifest_path.exists() {
                    if let Ok(content) = fs::read_to_string(&manifest_path) {
                        if let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&content) {
                            plugins.push(PluginInfo {
                                name: manifest["name"].as_str().unwrap_or("unknown").to_string(),
                                version: manifest["version"]
                                    .as_str()
                                    .unwrap_or("0.0.0")
                                    .to_string(),
                                description: manifest["description"]
                                    .as_str()
                                    .unwrap_or("")
                                    .to_string(),
                                author: manifest["author"]
                                    .as_str()
                                    .unwrap_or("unknown")
                                    .to_string(),
                                license: manifest["license"]
                                    .as_str()
                                    .unwrap_or("unknown")
                                    .to_string(),
                                extension_points: manifest["extension_points"]
                                    .as_array()
                                    .map(|a| {
                                        a.iter()
                                            .filter_map(|v| v.as_str().map(String::from))
                                            .collect()
                                    })
                                    .unwrap_or_default(),
                                tags: manifest["tags"]
                                    .as_array()
                                    .map(|a| {
                                        a.iter()
                                            .filter_map(|v| v.as_str().map(String::from))
                                            .collect()
                                    })
                                    .unwrap_or_default(),
                            });
                        }
                    }
                }
            }
        }

        plugins.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(plugins)
    }

    fn resolve_plugins_dir(&self) -> CliResult<PathBuf> {
        let cwd = std::env::current_dir().map_err(|e| CliError::filesystem(e.to_string()))?;
        Ok(cwd.join(DEFAULT_PLUGINS_DIR))
    }
}

impl Command for PluginListCommand {
    fn execute(&self) -> CliResult<()> {
        let plugins = self.find_installed_plugins()?;

        if plugins.is_empty() {
            println!("No plugins installed.");
            println!();
            println!("Install a plugin with: tachyon plugin install <url>");
            println!("Browse available plugins at: {}", REGISTRY_URL);
            return Ok(());
        }

        println!("Installed plugins ({}):", plugins.len());
        println!();
        for plugin in &plugins {
            println!(
                "  {} v{} - {}",
                plugin.name, plugin.version, plugin.description
            );
            println!(
                "    Author: {} | License: {}",
                plugin.author, plugin.license
            );
            if !plugin.tags.is_empty() {
                println!("    Tags: {}", plugin.tags.join(", "));
            }
            println!();
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "plugin list"
    }

    fn description(&self) -> &str {
        "List installed plugins"
    }
}

/// Plugin info command
pub struct PluginInfoCommand {
    pub name: String,
}

impl PluginInfoCommand {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    fn find_plugin(&self) -> CliResult<Option<PluginInfo>> {
        let plugins = self.find_installed_plugins()?;
        Ok(plugins.into_iter().find(|p| {
            p.name.eq_ignore_ascii_case(&self.name)
                || p.name
                    .to_lowercase()
                    .replace(' ', "-")
                    .eq_ignore_ascii_case(&self.name)
        }))
    }

    fn find_installed_plugins(&self) -> CliResult<Vec<PluginInfo>> {
        PluginListCommand::new().find_installed_plugins()
    }
}

impl Command for PluginInfoCommand {
    fn execute(&self) -> CliResult<()> {
        match self.find_plugin()? {
            Some(plugin) => {
                println!("Plugin: {}", plugin.name);
                println!("Version: {}", plugin.version);
                println!("Description: {}", plugin.description);
                println!("Author: {}", plugin.author);
                println!("License: {}", plugin.license);
                if !plugin.extension_points.is_empty() {
                    println!("Extension points:");
                    for ep in &plugin.extension_points {
                        println!("  - {}", ep);
                    }
                }
                if !plugin.tags.is_empty() {
                    println!("Tags: {}", plugin.tags.join(", "));
                }
            }
            None => {
                return Err(CliError::command(format!(
                    "Plugin '{}' not found. Use 'tachyon plugin list' to see installed plugins.",
                    self.name
                )));
            }
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "plugin info"
    }

    fn description(&self) -> &str {
        "Show detailed information about a plugin"
    }
}

/// Plugin install command
pub struct PluginInstallCommand {
    pub url: String,
}

impl PluginInstallCommand {
    pub fn new(url: String) -> Self {
        Self { url }
    }

    fn resolve_plugins_dir(&self) -> CliResult<PathBuf> {
        let cwd = std::env::current_dir().map_err(|e| CliError::filesystem(e.to_string()))?;
        let dir = cwd.join(DEFAULT_PLUGINS_DIR);
        fs::create_dir_all(&dir)
            .map_err(|e| CliError::filesystem(format!("Failed to create plugins dir: {}", e)))?;
        Ok(dir)
    }

    fn extract_plugin_name_from_url(&self) -> String {
        let path = self.url.trim_end_matches('/');
        let filename = path.rsplit('/').next().unwrap_or("plugin");
        let name = filename
            .trim_end_matches(".wasm")
            .trim_end_matches(".tar.gz");
        name.to_string()
    }
}

impl Command for PluginInstallCommand {
    fn execute(&self) -> CliResult<()> {
        println!("Installing plugin from: {}", self.url);

        // Extract a sensible name from the URL
        let plugin_name = self.extract_plugin_name_from_url();
        let plugins_dir = self.resolve_plugins_dir()?;
        let plugin_dir = plugins_dir.join(&plugin_name);

        fs::create_dir_all(&plugin_dir).map_err(|e| {
            CliError::filesystem(format!("Failed to create plugin directory: {}", e))
        })?;

        println!("Plugin directory: {}", plugin_dir.display());
        println!();
        println!("Note: Plugin download from remote registries is not yet implemented.");
        println!("To install a plugin manually:");
        println!("  1. Download the .wasm file from the URL");
        println!("  2. Place it in: {}", plugin_dir.display());
        println!("  3. Create a plugin.json manifest in the same directory");
        println!();
        println!("For local development, use:");
        println!("  tachyon plugin new <name> && cd <name> && tachyon plugin build");

        Ok(())
    }

    fn name(&self) -> &str {
        "plugin install"
    }

    fn description(&self) -> &str {
        "Install a plugin from a URL"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_plugin_name_from_url() {
        let cmd = PluginInstallCommand::new(
            "https://registry.tachyon.dev/plugins/my-plugin/1.0.0/my-plugin.wasm".to_string(),
        );
        assert_eq!(cmd.extract_plugin_name_from_url(), "my-plugin");
    }

    #[test]
    fn test_extract_plugin_name_trailing_slash() {
        let cmd = PluginInstallCommand::new(
            "https://registry.tachyon.dev/plugins/test-plugin.wasm/".to_string(),
        );
        assert_eq!(cmd.extract_plugin_name_from_url(), "test-plugin");
    }

    #[test]
    fn test_plugin_list_empty() {
        let cmd = PluginListCommand::new();
        let result = cmd.execute();
        // Should succeed even with no plugins (just prints message)
        assert!(result.is_ok());
    }
}
