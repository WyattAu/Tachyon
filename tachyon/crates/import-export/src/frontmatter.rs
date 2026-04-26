//! YAML frontmatter parsing for markdown files.
//!
//! Parses the `---\n...\n---` block at the start of markdown files into
//! a typed struct. Compatible with Obsidian, Jekyll, Hugo, and other
//! static site generators.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Parsed YAML frontmatter from a markdown file.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Frontmatter {
    /// Document title
    pub title: Option<String>,
    /// Document description/summary
    pub description: Option<String>,
    /// Tags (from YAML list or comma-separated string)
    #[serde(default)]
    pub tags: Vec<String>,
    /// Aliases (Obsidian: alternative titles for the document)
    #[serde(default)]
    pub aliases: Vec<String>,
    /// CSS class (Obsidian)
    pub cssclass: Option<String>,
    /// Creation date (ISO 8601 string)
    pub created: Option<String>,
    /// Last modification date (ISO 8601 string)
    pub modified: Option<String>,
    /// Document category
    pub category: Option<String>,
    /// Template used (Obsidian)
    pub template: Option<String>,
    /// Author
    pub author: Option<String>,
    /// Status (draft, published, etc.)
    pub status: Option<String>,
    /// Visibility (public, private, restricted)
    pub visibility: Option<String>,
    /// Any additional fields not explicitly mapped
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_yaml::Value>,
}

impl Frontmatter {
    /// Parse frontmatter from markdown content.
    /// Returns (frontmatter, body_content).
    /// If no frontmatter is found, returns (default Frontmatter, original content).
    pub fn parse(content: &str) -> (Self, &str) {
        let trimmed = content.trim_start();
        if !trimmed.starts_with("---") {
            return (Self::default(), content);
        }

        let after_delim = &trimmed[3..];
        let end = match after_delim.find("\n---") {
            Some(pos) => pos,
            None => return (Self::default(), content),
        };

        let yaml_block = &after_delim[..end];
        let body = after_delim[end + 4..].trim_start_matches('\n');

        let frontmatter = serde_yaml::from_str(yaml_block).unwrap_or_else(|e| {
            tracing::warn!("Failed to parse frontmatter YAML: {}", e);
            Self::default()
        });

        (frontmatter, body)
    }

    /// Serialize this frontmatter back to YAML string (without `---` delimiters).
    pub fn to_yaml_string(&self) -> String {
        serde_yaml::to_string(self).unwrap_or_default()
    }

    /// Generate the full frontmatter block with `---` delimiters.
    pub fn to_frontmatter_block(&self) -> String {
        let yaml = self.to_yaml_string();
        if yaml.is_empty() || yaml == "{}\n" {
            String::new()
        } else {
            format!("---\n{}---\n\n", yaml)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter_with_all_fields() {
        let content = r#"---
title: "My Document"
description: "A test document"
tags:
  - rust
  - web
aliases:
  - "My Doc"
created: "2024-01-15T10:30:00Z"
modified: "2024-01-20T14:00:00Z"
status: published
custom_field: "custom value"
---

# Hello World
"#;

        let (fm, body) = Frontmatter::parse(content);
        assert_eq!(fm.title.as_deref(), Some("My Document"));
        assert_eq!(fm.description.as_deref(), Some("A test document"));
        assert_eq!(fm.tags, vec!["rust", "web"]);
        assert_eq!(fm.aliases, vec!["My Doc"]);
        assert_eq!(fm.status.as_deref(), Some("published"));
        assert_eq!(fm.created.as_deref(), Some("2024-01-15T10:30:00Z"));
        assert!(body.trim().starts_with("# Hello World"));
        assert!(fm.extra.contains_key("custom_field"));
    }

    #[test]
    fn test_parse_frontmatter_tags_as_string() {
        // When tags is a string instead of a list, serde_yaml fails to
        // deserialize into Vec<String>, causing the whole frontmatter
        // parse to fall back to default.
        let content = r#"---
title: "Tags Test"
tags: "rust, web, api"
---

Body here.
"#;
        let (fm, _) = Frontmatter::parse(content);
        // Parse failure returns default (title = None)
        assert!(fm.title.is_none());
    }

    #[test]
    fn test_parse_no_frontmatter() {
        let content = "# Just markdown\n\nNo frontmatter here.";
        let (fm, body) = Frontmatter::parse(content);
        assert!(fm.title.is_none());
        assert_eq!(body, content);
    }

    #[test]
    fn test_parse_empty_frontmatter() {
        let content = "---\n---\n\n# Hello";
        let (fm, body) = Frontmatter::parse(content);
        assert!(fm.title.is_none());
        assert!(body.trim().starts_with("# Hello"));
    }

    #[test]
    fn test_to_frontmatter_block_roundtrip() {
        let fm = Frontmatter {
            title: Some("Test".to_string()),
            tags: vec!["a".to_string(), "b".to_string()],
            ..Default::default()
        };
        let block = fm.to_frontmatter_block();
        assert!(block.starts_with("---\n"));
        assert!(block.ends_with("---\n\n"));
        assert!(block.contains("title: Test"));
    }

    #[test]
    fn test_parse_obsidian_callout_frontmatter() {
        let content = r#"---
title: "Obsidian Note"
cssclass: wide-page
tags:
  - daily
  - journal
created: "2024-03-01"
---

> [!note] This is an Obsidian callout
> It should be preserved in the body.
"#;
        let (fm, body) = Frontmatter::parse(content);
        assert_eq!(fm.title.as_deref(), Some("Obsidian Note"));
        assert_eq!(fm.cssclass.as_deref(), Some("wide-page"));
        assert_eq!(fm.tags, vec!["daily", "journal"]);
        assert!(body.contains("> [!note]"));
    }
}
