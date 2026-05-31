//! Slash-command plugin extensibility.
//!
//! Allows WASM plugins to register custom slash commands (e.g., /mermaid, /plantuml).
//! The server aggregates registered commands and exposes them via API.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A registered slash command from a plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashCommand {
    /// Command trigger (e.g., "mermaid", "plantuml").
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Plugin that registered this command.
    pub plugin_name: String,
    /// Usage hint (e.g., "/mermaid <diagram-code>").
    pub usage: Option<String>,
    /// Whether this command requires document context.
    pub requires_document: bool,
    /// Command category for grouping in UI.
    pub category: CommandCategory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandCategory {
    Formatting,
    Visualization,
    Code,
    Ai,
    Data,
    Custom,
}

/// Registry of available slash commands.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommandRegistry {
    commands: HashMap<String, SlashCommand>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a slash command. Returns false if already registered.
    pub fn register(&mut self, command: SlashCommand) -> bool {
        let name = command.name.clone();
        self.commands.insert(name.clone(), command).is_none()
    }

    /// Unregister a command by name.
    pub fn unregister(&mut self, name: &str) -> bool {
        self.commands.remove(name).is_some()
    }

    /// Unregister all commands from a specific plugin.
    pub fn unregister_by_plugin(&mut self, plugin_name: &str) -> usize {
        let before = self.commands.len();
        self.commands
            .retain(|_, cmd| cmd.plugin_name != plugin_name);
        before - self.commands.len()
    }

    /// Look up a command by name.
    pub fn get(&self, name: &str) -> Option<&SlashCommand> {
        self.commands.get(name)
    }

    /// List all registered commands.
    pub fn list(&self) -> Vec<&SlashCommand> {
        self.commands.values().collect()
    }

    /// List commands grouped by category.
    pub fn list_by_category(&self) -> HashMap<CommandCategory, Vec<&SlashCommand>> {
        let mut grouped: HashMap<CommandCategory, Vec<&SlashCommand>> = HashMap::new();
        for cmd in self.commands.values() {
            grouped.entry(cmd.category).or_default().push(cmd);
        }
        grouped
    }

    /// Number of registered commands.
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_command() {
        let mut registry = CommandRegistry::new();
        assert!(registry.register(SlashCommand {
            name: "mermaid".to_string(),
            description: "Render Mermaid diagrams".to_string(),
            plugin_name: "diagrams".to_string(),
            usage: Some("/mermaid <diagram>".to_string()),
            requires_document: true,
            category: CommandCategory::Visualization,
        }));
        assert!(!registry.is_empty());
        assert_eq!(registry.len(), 1);
        assert!(registry.get("mermaid").is_some());
    }

    #[test]
    fn test_duplicate_registration() {
        let mut registry = CommandRegistry::new();
        assert!(registry.register(make_cmd("test", "plugin-a")));
        assert!(!registry.register(make_cmd("test", "plugin-b")));
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_unregister_by_plugin() {
        let mut registry = CommandRegistry::new();
        registry.register(make_cmd("cmd1", "plugin-a"));
        registry.register(make_cmd("cmd2", "plugin-a"));
        registry.register(make_cmd("cmd3", "plugin-b"));
        assert_eq!(registry.unregister_by_plugin("plugin-a"), 2);
        assert_eq!(registry.len(), 1);
        assert!(registry.get("cmd3").is_some());
    }

    #[test]
    fn test_list_by_category() {
        let mut registry = CommandRegistry::new();
        registry.register(SlashCommand {
            name: "mermaid".to_string(),
            description: "Diagrams".to_string(),
            plugin_name: "p1".to_string(),
            usage: None,
            requires_document: true,
            category: CommandCategory::Visualization,
        });
        registry.register(SlashCommand {
            name: "bold".to_string(),
            description: "Bold text".to_string(),
            plugin_name: "p2".to_string(),
            usage: None,
            requires_document: false,
            category: CommandCategory::Formatting,
        });

        let grouped = registry.list_by_category();
        assert_eq!(grouped.len(), 2);
        assert!(grouped.get(&CommandCategory::Visualization).is_some());
    }

    fn make_cmd(name: &str, plugin: &str) -> SlashCommand {
        SlashCommand {
            name: name.to_string(),
            description: format!("{} command", name),
            plugin_name: plugin.to_string(),
            usage: None,
            requires_document: false,
            category: CommandCategory::Custom,
        }
    }
}
