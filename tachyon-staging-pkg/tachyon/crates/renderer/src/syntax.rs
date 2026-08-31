//! Syntax highlighting with tree-sitter
//!
//! This module provides syntax highlighting capabilities using tree-sitter
//! library, supporting multiple programming languages with configurable themes.

use crate::error::{RendererError, RendererResult};
use crate::types::{Language, SyntaxTheme};
use std::sync::OnceLock;
use tracing::debug;
use tree_sitter_highlight::{Highlight, HighlightConfiguration, Highlighter, HtmlRenderer};

/// Global highlight configurations cache
static HIGHLIGHT_CONFIGS: OnceLock<std::collections::HashMap<Language, HighlightConfiguration>> =
    OnceLock::new();

/// Syntax highlighter for highlighting code
pub struct SyntaxHighlighter {
    /// Current theme
    theme: SyntaxTheme,
    /// Cached highlight configurations
    configs: &'static std::collections::HashMap<Language, HighlightConfiguration>,
}

impl SyntaxHighlighter {
    /// Create a new syntax highlighter
    pub fn new() -> Self {
        Self::with_theme(SyntaxTheme::default())
    }

    /// Create a new syntax highlighter with a specific theme
    pub fn with_theme(theme: SyntaxTheme) -> Self {
        let configs = HIGHLIGHT_CONFIGS.get_or_init(Self::init_configs);
        Self { theme, configs }
    }

    /// Initialize all highlight configurations
    fn init_configs() -> std::collections::HashMap<Language, HighlightConfiguration> {
        let mut configs = std::collections::HashMap::new();

        // Initialize Rust
        if let Ok(mut config) = HighlightConfiguration::new(
            tree_sitter_rust::LANGUAGE.into(),
            "rust",
            tree_sitter_rust::HIGHLIGHTS_QUERY,
            tree_sitter_rust::INJECTIONS_QUERY,
            "",
        ) {
            config.configure(THEME_HIGHLIGHT_NAMES);
            configs.insert(Language::Rust, config);
        }

        // Initialize Python
        if let Ok(mut config) = HighlightConfiguration::new(
            tree_sitter_python::LANGUAGE.into(),
            "python",
            tree_sitter_python::HIGHLIGHTS_QUERY,
            "",
            "",
        ) {
            config.configure(THEME_HIGHLIGHT_NAMES);
            configs.insert(Language::Python, config);
        }

        // Initialize JavaScript
        if let Ok(mut config) = HighlightConfiguration::new(
            tree_sitter_javascript::LANGUAGE.into(),
            "javascript",
            tree_sitter_javascript::HIGHLIGHT_QUERY,
            tree_sitter_javascript::INJECTIONS_QUERY,
            "",
        ) {
            config.configure(THEME_HIGHLIGHT_NAMES);
            configs.insert(Language::JavaScript, config);
        }

        // Initialize TypeScript
        if let Ok(mut config) = HighlightConfiguration::new(
            tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            "typescript",
            tree_sitter_typescript::HIGHLIGHTS_QUERY,
            "",
            "",
        ) {
            config.configure(THEME_HIGHLIGHT_NAMES);
            configs.insert(Language::TypeScript, config);
        }

        // Initialize JSON
        if let Ok(mut config) = HighlightConfiguration::new(
            tree_sitter_json::language(),
            "json",
            tree_sitter_json::HIGHLIGHTS_QUERY,
            "",
            "",
        ) {
            config.configure(THEME_HIGHLIGHT_NAMES);
            configs.insert(Language::Json, config);
        }

        // Initialize TOML
        // Note: tree_sitter_toml uses an incompatible tree-sitter version
        // Skipping TOML highlighting until dependency is updated

        // Initialize YAML
        if let Ok(mut config) = HighlightConfiguration::new(
            tree_sitter_yaml::LANGUAGE.into(),
            "yaml",
            tree_sitter_yaml::HIGHLIGHTS_QUERY,
            "",
            "",
        ) {
            config.configure(THEME_HIGHLIGHT_NAMES);
            configs.insert(Language::Yaml, config);
        }

        // Initialize HTML
        if let Ok(mut config) = HighlightConfiguration::new(
            tree_sitter_html::LANGUAGE.into(),
            "html",
            tree_sitter_html::HIGHLIGHTS_QUERY,
            tree_sitter_html::INJECTIONS_QUERY,
            "",
        ) {
            config.configure(THEME_HIGHLIGHT_NAMES);
            configs.insert(Language::Html, config);
        }

        // Initialize CSS
        if let Ok(mut config) = HighlightConfiguration::new(
            tree_sitter_css::LANGUAGE.into(),
            "css",
            tree_sitter_css::HIGHLIGHTS_QUERY,
            "",
            "",
        ) {
            config.configure(THEME_HIGHLIGHT_NAMES);
            configs.insert(Language::Css, config);
        }

        // Initialize SQL
        // Note: tree_sitter_sql uses an incompatible tree-sitter version
        // Skipping SQL highlighting until dependency is updated

        // Initialize Bash
        if let Ok(mut config) = HighlightConfiguration::new(
            tree_sitter_bash::LANGUAGE.into(),
            "bash",
            tree_sitter_bash::HIGHLIGHT_QUERY,
            "",
            "",
        ) {
            config.configure(THEME_HIGHLIGHT_NAMES);
            configs.insert(Language::Bash, config);
        }

        // Initialize Markdown
        // Note: tree_sitter_markdown uses an incompatible tree-sitter version
        // Skipping Markdown highlighting until dependency is updated

        debug!(
            "Initialized {} syntax highlight configurations",
            configs.len()
        );
        configs
    }

    /// Highlight code and return HTML
    pub fn highlight(&self, code: &str, language: &str) -> RendererResult<String> {
        let lang = Language::from_name(language)
            .ok_or_else(|| RendererError::unsupported_language(language))?;

        self.highlight_with_lang(code, &lang)
    }

    /// Highlight code with a known language
    pub fn highlight_with_lang(&self, code: &str, language: &Language) -> RendererResult<String> {
        let config = self
            .configs
            .get(language)
            .ok_or_else(|| RendererError::unsupported_language(language.as_str()))?;

        let mut highlighter = Highlighter::new();
        let highlights = highlighter
            .highlight(config, code.as_bytes(), None, |lang_name| {
                self.configs
                    .get(&Language::from_name(lang_name).unwrap_or(Language::Rust))
            })
            .map_err(|e| RendererError::syntax_highlight(e.to_string()))?;

        let mut renderer = HtmlRenderer::new();
        renderer
            .render(highlights, code.as_bytes(), &|highlight, buf| {
                buf.extend(self.get_css_class(highlight).as_bytes());
            })
            .map_err(|e| RendererError::syntax_highlight(e.to_string()))?;

        let mut html = String::new();
        html.push_str("<pre class=\"syntax-highlight\"><code>");
        for line in renderer.lines() {
            html.push_str(&html_escape(line));
        }
        html.push_str("</code></pre>");

        Ok(html)
    }

    /// Get CSS class for a highlight
    fn get_css_class(&self, highlight: Highlight) -> &'static str {
        let idx = highlight.0;
        if idx < THEME_HIGHLIGHT_NAMES.len() {
            THEME_HIGHLIGHT_NAMES[idx]
        } else {
            ""
        }
    }

    /// Get the current theme
    pub fn theme(&self) -> SyntaxTheme {
        self.theme
    }

    /// Set the theme
    pub fn set_theme(&mut self, theme: SyntaxTheme) {
        self.theme = theme;
    }

    /// Check if a language is supported
    pub fn is_language_supported(&self, language: &str) -> bool {
        Language::from_name(language)
            .map(|lang| self.configs.contains_key(&lang))
            .unwrap_or(false)
    }

    /// Get list of supported languages
    pub fn supported_languages(&self) -> Vec<&'static str> {
        self.configs.keys().map(|lang| lang.as_str()).collect()
    }

    /// Generate CSS stylesheet for the current theme
    pub fn generate_stylesheet(&self) -> String {
        let theme_colors = match self.theme {
            SyntaxTheme::Light => &LIGHT_THEME_COLORS,
            SyntaxTheme::Dark => &DARK_THEME_COLORS,
            SyntaxTheme::HighContrast => &HIGH_CONTRAST_THEME_COLORS,
            SyntaxTheme::Custom => &DARK_THEME_COLORS, // Default to dark for custom
        };

        let mut css = String::from(".syntax-highlight {\n");
        css.push_str("  font-family: 'Fira Code', 'Consolas', monospace;\n");
        css.push_str("  line-height: 1.5;\n");
        css.push_str("  overflow-x: auto;\n");
        css.push_str("  padding: 1em;\n");
        css.push_str("  border-radius: 4px;\n");
        css.push_str(&format!(
            "  background-color: {};\n",
            theme_colors.background
        ));
        css.push_str(&format!("  color: {};\n", theme_colors.foreground));
        css.push_str("}\n\n");

        for (i, name) in THEME_HIGHLIGHT_NAMES.iter().enumerate() {
            if let Some(color) = theme_colors.highlights.get(i) {
                css.push_str(&format!(
                    ".syntax-highlight .{} {{ color: {}; }}\n",
                    name, color
                ));
            }
        }

        css
    }
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

/// HTML escape helper
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Theme highlight names (standard tree-sitter highlight names)
const THEME_HIGHLIGHT_NAMES: &[&str] = &[
    "attribute",
    "constant",
    "function.builtin",
    "function",
    "keyword",
    "operator",
    "property",
    "punctuation",
    "punctuation.bracket",
    "punctuation.delimiter",
    "string",
    "string.escape",
    "string.special",
    "tag",
    "type",
    "type.builtin",
    "variable",
    "variable.builtin",
    "variable.parameter",
    "comment",
    "constructor",
    "embedded",
    "label",
    "number",
    "repeat",
    "character",
    "conditional",
    "define",
    "include",
    "boolean",
];

/// Theme color definitions
struct ThemeColors {
    background: &'static str,
    foreground: &'static str,
    highlights: &'static [&'static str],
}

/// Dark theme colors (similar to One Dark)
const DARK_THEME_COLORS: ThemeColors = ThemeColors {
    background: "#282c34",
    foreground: "#abb2bf",
    highlights: &[
        "#e06c75", // attribute
        "#e5c07b", // constant
        "#e5c07b", // function.builtin
        "#61afef", // function
        "#c678dd", // keyword
        "#56b6c2", // operator
        "#e06c75", // property
        "#abb2bf", // punctuation
        "#abb2bf", // punctuation.bracket
        "#abb2bf", // punctuation.delimiter
        "#98c379", // string
        "#56b6c2", // string.escape
        "#56b6c2", // string.special
        "#e06c75", // tag
        "#e5c07b", // type
        "#e5c07b", // type.builtin
        "#e06c75", // variable
        "#e5c07b", // variable.builtin
        "#e06c75", // variable.parameter
        "#5c6370", // comment
        "#e5c07b", // constructor
        "#98c379", // embedded
        "#c678dd", // label
        "#d19a66", // number
        "#c678dd", // repeat
        "#98c379", // character
        "#c678dd", // conditional
        "#c678dd", // define
        "#c678dd", // include
        "#d19a66", // boolean
    ],
};

/// Light theme colors (similar to One Light)
const LIGHT_THEME_COLORS: ThemeColors = ThemeColors {
    background: "#fafafa",
    foreground: "#383a42",
    highlights: &[
        "#e45649", // attribute
        "#986801", // constant
        "#a626a4", // function.builtin
        "#4078f2", // function
        "#a626a4", // keyword
        "#0184bc", // operator
        "#e45649", // property
        "#383a42", // punctuation
        "#383a42", // punctuation.bracket
        "#383a42", // punctuation.delimiter
        "#50a14f", // string
        "#0184bc", // string.escape
        "#0184bc", // string.special
        "#e45649", // tag
        "#986801", // type
        "#c18401", // type.builtin
        "#e45649", // variable
        "#986801", // variable.builtin
        "#e45649", // variable.parameter
        "#a0a1a7", // comment
        "#986801", // constructor
        "#50a14f", // embedded
        "#a626a4", // label
        "#986801", // number
        "#a626a4", // repeat
        "#50a14f", // character
        "#a626a4", // conditional
        "#a626a4", // define
        "#a626a4", // include
        "#986801", // boolean
    ],
};

/// High contrast theme colors
const HIGH_CONTRAST_THEME_COLORS: ThemeColors = ThemeColors {
    background: "#000000",
    foreground: "#ffffff",
    highlights: &[
        "#ff6b6b", // attribute
        "#ffd93d", // constant
        "#ffd93d", // function.builtin
        "#6bcfff", // function
        "#ff79c6", // keyword
        "#8be9fd", // operator
        "#ff6b6b", // property
        "#ffffff", // punctuation
        "#ffffff", // punctuation.bracket
        "#ffffff", // punctuation.delimiter
        "#50fa7b", // string
        "#8be9fd", // string.escape
        "#8be9fd", // string.special
        "#ff6b6b", // tag
        "#ffd93d", // type
        "#ffd93d", // type.builtin
        "#ff6b6b", // variable
        "#ffd93d", // variable.builtin
        "#ff6b6b", // variable.parameter
        "#bfbfbf", // comment
        "#ffd93d", // constructor
        "#50fa7b", // embedded
        "#ff79c6", // label
        "#ffb86c", // number
        "#ff79c6", // repeat
        "#50fa7b", // character
        "#ff79c6", // conditional
        "#ff79c6", // define
        "#ff79c6", // include
        "#ffb86c", // boolean
    ],
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlighter_creation() {
        let highlighter = SyntaxHighlighter::new();
        assert_eq!(highlighter.theme(), SyntaxTheme::Dark);
    }

    #[test]
    fn test_rust_highlighting() {
        let highlighter = SyntaxHighlighter::new();
        let code = r#"fn main() { println!("Hello"); }"#;
        let result = highlighter.highlight(code, "rust");

        assert!(result.is_ok());
        let html = result.unwrap();
        assert!(html.contains("<pre"));
        assert!(html.contains("</pre>"));
        assert!(html.contains("syntax-highlight"));
    }

    #[test]
    fn test_python_highlighting() {
        let highlighter = SyntaxHighlighter::new();
        let code = "def hello():\n    print('Hello')";
        let result = highlighter.highlight(code, "python");

        assert!(result.is_ok());
    }

    #[test]
    fn test_unsupported_language() {
        let highlighter = SyntaxHighlighter::new();
        let code = "some code";
        let result = highlighter.highlight(code, "unknown_lang");

        assert!(result.is_err());
    }

    #[test]
    fn test_is_language_supported() {
        let highlighter = SyntaxHighlighter::new();

        assert!(highlighter.is_language_supported("rust"));
        assert!(highlighter.is_language_supported("python"));
        assert!(highlighter.is_language_supported("js"));
        assert!(!highlighter.is_language_supported("unknown"));
    }

    #[test]
    fn test_stylesheet_generation() {
        let highlighter = SyntaxHighlighter::new();
        let css = highlighter.generate_stylesheet();

        assert!(css.contains(".syntax-highlight"));
        assert!(css.contains("background-color"));
    }

    #[test]
    fn test_theme_switching() {
        let mut highlighter = SyntaxHighlighter::new();
        assert_eq!(highlighter.theme(), SyntaxTheme::Dark);

        highlighter.set_theme(SyntaxTheme::Light);
        assert_eq!(highlighter.theme(), SyntaxTheme::Light);

        let css = highlighter.generate_stylesheet();
        assert!(css.contains("#fafafa")); // Light theme background
    }

    #[test]
    fn test_html_escape() {
        let escaped = html_escape("<script>alert('xss')</script>");
        assert!(escaped.contains("&lt;"));
        assert!(escaped.contains("&gt;"));
        assert!(!escaped.contains("<script>"));
    }
}
