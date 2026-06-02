//! Tree-sitter syntax highlighter (native only).
//!
//! Compiles tree-sitter grammars at build time via the `cc` crate.
//! Only available behind the `native-tree-sitter` feature flag.
//! WASM targets use a different provider (Phase A2+).

use std::sync::OnceLock;

use tree_sitter_highlight::{Highlight, HighlightConfiguration, Highlighter as TsHighlighter};

use crate::highlight::{HighlightProvider, HighlightSpan, HighlightToken};

// ─── Highlight capture names ─────────────────────────────────────────
// Must match THEME_HIGHLIGHT_NAMES in renderer/src/syntax.rs.

const CAPTURE_NAMES: &[&str] = &[
    "attribute",             // 0
    "constant",              // 1
    "function.builtin",      // 2
    "function",              // 3
    "keyword",               // 4
    "operator",              // 5
    "property",              // 6
    "punctuation",           // 7
    "punctuation.bracket",   // 8
    "punctuation.delimiter", // 9
    "string",                // 10
    "string.escape",         // 11
    "string.special",        // 12
    "tag",                   // 13
    "type",                  // 14
    "type.builtin",          // 15
    "variable",              // 16
    "variable.builtin",      // 17
    "variable.parameter",    // 18
    "comment",               // 19
    "constructor",           // 20
    "embedded",              // 21
    "label",                 // 22
    "number",                // 23
    "repeat",                // 24
    "character",             // 25
    "conditional",           // 26
    "define",                // 27
    "include",               // 28
    "boolean",               // 29
];

/// Map a tree-sitter [`Highlight`] index to a [`HighlightToken`].
fn capture_to_token(highlight: Highlight) -> HighlightToken {
    match CAPTURE_NAMES.get(highlight.0) {
        Some(&"keyword") => HighlightToken::Keyword,
        Some(&"string") => HighlightToken::String,
        Some(&"number") => HighlightToken::Number,
        Some(&"comment") => HighlightToken::Comment,
        Some(&"function") => HighlightToken::Function,
        Some(&"type") => HighlightToken::Type,
        Some(&"variable") => HighlightToken::Variable,
        Some(&"operator") => HighlightToken::Operator,
        Some(&"property") => HighlightToken::Property,
        Some(&"punctuation") => HighlightToken::Punctuation,
        Some(&"constant") => HighlightToken::Constant,
        Some(&"attribute") => HighlightToken::Attribute,
        Some(&"tag") => HighlightToken::CodeTag,
        Some(&"label") => HighlightToken::Label,
        Some(&"embedded") => HighlightToken::Embedded,
        Some(&"constructor") => HighlightToken::Constructor,
        Some(&"character") => HighlightToken::Character,
        Some(&"boolean") => HighlightToken::Boolean,
        Some(&"conditional") => HighlightToken::Conditional,
        Some(&"repeat") => HighlightToken::Repeat,
        Some(&"define") => HighlightToken::Define,
        Some(&"include") => HighlightToken::Include,
        Some(&"function.builtin") => HighlightToken::FunctionBuiltin,
        Some(&"type.builtin") => HighlightToken::TypeBuiltin,
        Some(&"variable.builtin") => HighlightToken::VariableBuiltin,
        Some(&"variable.parameter") => HighlightToken::VariableParameter,
        Some(&"string.escape") => HighlightToken::StringEscape,
        Some(&"string.special") => HighlightToken::StringSpecial,
        Some(&"punctuation.bracket") => HighlightToken::PunctuationBracket,
        Some(&"punctuation.delimiter") => HighlightToken::PunctuationDelimiter,
        _ => HighlightToken::Text,
    }
}

// ─── Language enum ───────────────────────────────────────────────────

/// Supported languages for tree-sitter highlighting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TsLanguage {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Json,
    Yaml,
    Css,
    Bash,
    Html,
}

impl TsLanguage {
    /// Parse a language name (case-insensitive, common aliases accepted).
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "rust" | "rs" => Some(Self::Rust),
            "python" | "py" => Some(Self::Python),
            "javascript" | "js" | "jsx" => Some(Self::JavaScript),
            "typescript" | "ts" | "tsx" => Some(Self::TypeScript),
            "json" => Some(Self::Json),
            "yaml" | "yml" => Some(Self::Yaml),
            "css" => Some(Self::Css),
            "bash" | "sh" | "shell" | "zsh" => Some(Self::Bash),
            "html" | "htm" => Some(Self::Html),
            _ => None,
        }
    }

    /// Canonical name string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::Python => "python",
            Self::JavaScript => "javascript",
            Self::TypeScript => "typescript",
            Self::Json => "json",
            Self::Yaml => "yaml",
            Self::Css => "css",
            Self::Bash => "bash",
            Self::Html => "html",
        }
    }
}

// ─── Configuration cache ──────────────────────────────────────────────

static CONFIGS: OnceLock<std::collections::HashMap<TsLanguage, HighlightConfiguration>> =
    OnceLock::new();

fn get_configs() -> &'static std::collections::HashMap<TsLanguage, HighlightConfiguration> {
    CONFIGS.get_or_init(init_configs)
}

fn init_configs() -> std::collections::HashMap<TsLanguage, HighlightConfiguration> {
    let mut configs = std::collections::HashMap::new();

    // Rust
    if let Ok(mut cfg) = HighlightConfiguration::new(
        tree_sitter_rust::LANGUAGE.into(),
        "rust",
        tree_sitter_rust::HIGHLIGHTS_QUERY,
        tree_sitter_rust::INJECTIONS_QUERY,
        "",
    ) {
        cfg.configure(CAPTURE_NAMES);
        configs.insert(TsLanguage::Rust, cfg);
    }

    // Python
    if let Ok(mut cfg) = HighlightConfiguration::new(
        tree_sitter_python::LANGUAGE.into(),
        "python",
        tree_sitter_python::HIGHLIGHTS_QUERY,
        "",
        "",
    ) {
        cfg.configure(CAPTURE_NAMES);
        configs.insert(TsLanguage::Python, cfg);
    }

    // JavaScript
    if let Ok(mut cfg) = HighlightConfiguration::new(
        tree_sitter_javascript::LANGUAGE.into(),
        "javascript",
        tree_sitter_javascript::HIGHLIGHT_QUERY,
        tree_sitter_javascript::INJECTIONS_QUERY,
        "",
    ) {
        cfg.configure(CAPTURE_NAMES);
        configs.insert(TsLanguage::JavaScript, cfg);
    }

    // TypeScript
    if let Ok(mut cfg) = HighlightConfiguration::new(
        tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        "typescript",
        tree_sitter_typescript::HIGHLIGHTS_QUERY,
        "",
        "",
    ) {
        cfg.configure(CAPTURE_NAMES);
        configs.insert(TsLanguage::TypeScript, cfg);
    }

    // JSON (uses language() not LANGUAGE)
    if let Ok(mut cfg) = HighlightConfiguration::new(
        tree_sitter_json::language(),
        "json",
        tree_sitter_json::HIGHLIGHTS_QUERY,
        "",
        "",
    ) {
        cfg.configure(CAPTURE_NAMES);
        configs.insert(TsLanguage::Json, cfg);
    }

    // YAML
    if let Ok(mut cfg) = HighlightConfiguration::new(
        tree_sitter_yaml::LANGUAGE.into(),
        "yaml",
        tree_sitter_yaml::HIGHLIGHTS_QUERY,
        "",
        "",
    ) {
        cfg.configure(CAPTURE_NAMES);
        configs.insert(TsLanguage::Yaml, cfg);
    }

    // CSS
    if let Ok(mut cfg) = HighlightConfiguration::new(
        tree_sitter_css::LANGUAGE.into(),
        "css",
        tree_sitter_css::HIGHLIGHTS_QUERY,
        "",
        "",
    ) {
        cfg.configure(CAPTURE_NAMES);
        configs.insert(TsLanguage::Css, cfg);
    }

    // Bash
    if let Ok(mut cfg) = HighlightConfiguration::new(
        tree_sitter_bash::LANGUAGE.into(),
        "bash",
        tree_sitter_bash::HIGHLIGHT_QUERY,
        "",
        "",
    ) {
        cfg.configure(CAPTURE_NAMES);
        configs.insert(TsLanguage::Bash, cfg);
    }

    // HTML
    if let Ok(mut cfg) = HighlightConfiguration::new(
        tree_sitter_html::LANGUAGE.into(),
        "html",
        tree_sitter_html::HIGHLIGHTS_QUERY,
        tree_sitter_html::INJECTIONS_QUERY,
        "",
    ) {
        cfg.configure(CAPTURE_NAMES);
        configs.insert(TsLanguage::Html, cfg);
    }

    tracing::debug!(
        "tree-sitter: {} language configs initialized",
        configs.len()
    );
    configs
}

// ─── TreeSitterHighlighter ──────────────────────────────────────────

/// Tree-sitter highlighter for code content.
///
/// Designed for standalone code files or code blocks extracted from
/// markdown. For full markdown documents, use [`RegexHighlighter`](super::RegexHighlighter).
pub struct TreeSitterHighlighter {
    language: TsLanguage,
}

impl TreeSitterHighlighter {
    /// Create a highlighter for a specific language.
    pub fn new(language: TsLanguage) -> Self {
        Self { language }
    }

    /// Create from a language name string (returns None if unsupported).
    pub fn from_language_name(name: &str) -> Option<Self> {
        TsLanguage::from_name(name).map(Self::new)
    }

    /// Check if a language name is supported.
    pub fn is_language_supported(name: &str) -> bool {
        TsLanguage::from_name(name).is_some()
    }

    /// List supported language names.
    pub fn supported_languages() -> Vec<&'static str> {
        get_configs().keys().map(|lang| lang.as_str()).collect()
    }

    /// Highlight a full code string, returning per-line spans.
    ///
    /// This is the primary interface for tree-sitter: parse once, split by line.
    fn highlight_code(&self, code: &str) -> Vec<Vec<HighlightSpan>> {
        let configs = get_configs();
        let config = match configs.get(&self.language) {
            Some(c) => c,
            None => {
                tracing::warn!("tree-sitter: no config for {:?}", self.language);
                return vec![vec![HighlightSpan {
                    token: HighlightToken::Text,
                    start_col: 0,
                    end_col: code.len(),
                }]];
            }
        };

        let mut ts_highlighter = TsHighlighter::new();
        let highlights = match ts_highlighter.highlight(config, code.as_bytes(), None, |_| None) {
            Ok(h) => h,
            Err(e) => {
                tracing::warn!("tree-sitter highlight error: {e}");
                return vec![vec![HighlightSpan {
                    token: HighlightToken::Text,
                    start_col: 0,
                    end_col: code.len(),
                }]];
            }
        };

        use tree_sitter_highlight::HighlightEvent;

        // Build per-line result.
        let lines: Vec<&str> = code.lines().collect();
        let line_count = lines.len();
        let mut result: Vec<Vec<HighlightSpan>> = vec![Vec::new(); line_count];

        // Map byte offsets to line index.
        let mut line_starts: Vec<usize> = Vec::with_capacity(line_count + 1);
        let mut offset = 0;
        for &line in &lines {
            line_starts.push(offset);
            offset += line.len() + 1; // +1 for '\n'
        }
        line_starts.push(offset);

        // Process highlight events: track active highlights, apply to source spans.
        let mut active_highlights: Vec<(usize, usize, HighlightToken)> = Vec::new();

        for event in highlights {
            let event = match event {
                Ok(e) => e,
                Err(_) => continue,
            };
            match event {
                HighlightEvent::HighlightStart(h) => {
                    active_highlights.push((0, 0, capture_to_token(h)));
                }
                HighlightEvent::HighlightEnd => {
                    active_highlights.pop();
                }
                HighlightEvent::Source { start, end } => {
                    for (_, _, token) in &mut active_highlights {
                        // Find which lines this source span covers.
                        let start_line = match line_starts.binary_search(&start) {
                            Ok(i) => i,
                            Err(i) => i.saturating_sub(1),
                        };
                        let end_line = match line_starts.binary_search(&end) {
                            Ok(i) => i,
                            Err(i) => i.saturating_sub(1),
                        };

                        for line_idx in start_line..=end_line.min(line_count - 1) {
                            let line_start = line_starts[line_idx];
                            let line_end = line_starts.get(line_idx + 1).copied().unwrap_or(offset);
                            let span_start = start.saturating_sub(line_start);
                            let span_end =
                                (end.saturating_sub(line_start)).min(line_end - line_start);
                            if span_start < span_end {
                                result[line_idx].push(HighlightSpan {
                                    token: token.clone(),
                                    start_col: span_start,
                                    end_col: span_end,
                                });
                            }
                        }
                    }
                }
            }
        }

        // Fill gaps with Text tokens for lines with no spans.
        for (line_idx, line_spans) in result.iter_mut().enumerate() {
            if line_spans.is_empty() {
                line_spans.push(HighlightSpan {
                    token: HighlightToken::Text,
                    start_col: 0,
                    end_col: lines[line_idx].len(),
                });
            } else {
                line_spans.sort_by_key(|s| s.start_col);
            }
        }

        result
    }
}

impl HighlightProvider for TreeSitterHighlighter {
    /// Tree-sitter requires full document context. This delegates to
    /// [`highlight_document`](Self::highlight_document) and returns
    /// the first line's spans (for single-line callers).
    fn highlight_line(&self, _line: &str, _in_code_block: &mut bool) -> Vec<HighlightSpan> {
        // Tree-sitter is document-level; use highlight_document instead.
        // This fallback treats the line as a plain document.
        let spans = self.highlight_document(_line);
        spans.into_iter().next().unwrap_or_default()
    }

    /// Full-document highlight using tree-sitter parse.
    fn highlight_document(&self, text: &str) -> Vec<Vec<HighlightSpan>> {
        self.highlight_code(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_name_parsing() {
        assert_eq!(TsLanguage::from_name("rust"), Some(TsLanguage::Rust));
        assert_eq!(TsLanguage::from_name("RS"), Some(TsLanguage::Rust));
        assert_eq!(TsLanguage::from_name("python"), Some(TsLanguage::Python));
        assert_eq!(TsLanguage::from_name("py"), Some(TsLanguage::Python));
        assert_eq!(TsLanguage::from_name("js"), Some(TsLanguage::JavaScript));
        assert_eq!(
            TsLanguage::from_name("typescript"),
            Some(TsLanguage::TypeScript)
        );
        assert_eq!(TsLanguage::from_name("json"), Some(TsLanguage::Json));
        assert_eq!(TsLanguage::from_name("yaml"), Some(TsLanguage::Yaml));
        assert_eq!(TsLanguage::from_name("yml"), Some(TsLanguage::Yaml));
        assert_eq!(TsLanguage::from_name("css"), Some(TsLanguage::Css));
        assert_eq!(TsLanguage::from_name("bash"), Some(TsLanguage::Bash));
        assert_eq!(TsLanguage::from_name("shell"), Some(TsLanguage::Bash));
        assert_eq!(TsLanguage::from_name("html"), Some(TsLanguage::Html));
        assert_eq!(TsLanguage::from_name("unknown"), None);
    }

    #[test]
    fn supported_languages_list() {
        let langs = TreeSitterHighlighter::supported_languages();
        assert!(!langs.is_empty());
        assert!(langs.contains(&"rust"));
        assert!(langs.contains(&"python"));
    }

    #[test]
    fn highlight_rust_code() {
        let h = TreeSitterHighlighter::new(TsLanguage::Rust);
        let code = "fn main() {\n    println!(\"hello\");\n}\n";
        let spans = h.highlight_document(code);
        assert_eq!(spans.len(), 3); // 3 lines

        // First line should have at least keyword "fn" and function "main".
        let first_line_tokens: Vec<&HighlightToken> = spans[0].iter().map(|s| &s.token).collect();
        assert!(first_line_tokens.contains(&&HighlightToken::Keyword));
        assert!(first_line_tokens.contains(&&HighlightToken::Function));
    }

    #[test]
    fn highlight_python_code() {
        let h = TreeSitterHighlighter::new(TsLanguage::Python);
        let code = "def hello():\n    print('world')\n";
        let spans = h.highlight_document(code);
        assert_eq!(spans.len(), 2);

        let first_tokens: Vec<&HighlightToken> = spans[0].iter().map(|s| &s.token).collect();
        assert!(first_tokens.contains(&&HighlightToken::Keyword));
        assert!(first_tokens.contains(&&HighlightToken::Function));
    }

    #[test]
    fn highlight_empty_document() {
        let h = TreeSitterHighlighter::new(TsLanguage::Rust);
        let spans = h.highlight_document("");
        assert!(spans.is_empty());
    }

    #[test]
    fn highlight_single_line() {
        let h = TreeSitterHighlighter::new(TsLanguage::Rust);
        let spans = h.highlight_line("let x = 5;", &mut false);
        assert!(!spans.is_empty());
    }

    #[test]
    fn from_language_name_factory() {
        assert!(TreeSitterHighlighter::from_language_name("rust").is_some());
        assert!(TreeSitterHighlighter::from_language_name("unknown").is_none());
    }

    #[test]
    fn css_class_mapping_for_code_tokens() {
        // Spot-check that code tokens don't panic in css_class.
        assert_eq!(
            crate::highlight::css_class(&HighlightToken::Keyword),
            "ed-keyword"
        );
        assert_eq!(
            crate::highlight::css_class(&HighlightToken::String),
            "ed-string"
        );
        assert_eq!(
            crate::highlight::css_class(&HighlightToken::Comment),
            "ed-comment"
        );
    }
}
