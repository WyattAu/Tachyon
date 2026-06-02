//! WASM tree-sitter syntax highlighter for `wasm32-unknown-unknown`.
//!
//! Uses `tree-sitter-highlight-wasm` (by d-e-s-o), a fork of
//! `tree-sitter-highlight` that works in WebAssembly via `tree-sitter-c2rust`
//! (pure Rust tree-sitter via c2rust transpilation).
//!
//! ## Grammar loading
//!
//! Currently grammar loading is a stub (`preload_grammar` is a no-op).
//! Runtime WASM grammar loading needs `web-sys` fetch, which is a separate
//! integration step.

use std::collections::HashMap;

use tree_sitter_highlight_wasm::{HighlightConfiguration, HighlightEvent, Highlighter};

use crate::highlight::{HighlightProvider, HighlightSpan, HighlightToken};

const CAPTURE_NAMES: &[&str] = &[
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

fn capture_to_token(highlight: tree_sitter_highlight_wasm::Highlight) -> HighlightToken {
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

const DEFAULT_GRAMMAR_BASE_URL: &str = "/grammars/";

pub struct WasmTreeSitterHighlighter {
    base_url: String,
    #[allow(dead_code)] // placeholder for web-sys fetch integration
    loaded_grammars: HashMap<String, Vec<u8>>,
    configs: HashMap<String, HighlightConfiguration>,
}

impl WasmTreeSitterHighlighter {
    pub fn new(base_url: &str) -> Self {
        let base = if base_url.ends_with('/') {
            base_url.to_owned()
        } else {
            format!("{base_url}/")
        };
        Self {
            base_url: base,
            loaded_grammars: HashMap::new(),
            configs: HashMap::new(),
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(DEFAULT_GRAMMAR_BASE_URL)
    }

    pub fn preload_grammar(&mut self, _language: &str) {
        // TODO: use web_sys::fetch to download {base_url}tree-sitter-{language}.wasm
        //       and store bytes in self.loaded_grammars, then create HighlightConfiguration.
    }

    pub fn is_grammar_loaded(&self, language: &str) -> bool {
        self.configs.contains_key(language)
    }

    pub fn loaded_languages(&self) -> Vec<String> {
        self.configs.keys().cloned().collect()
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn grammar_wasm_filename(language: &str) -> String {
        format!("tree-sitter-{language}.wasm")
    }

    fn highlight_code(&self, code: &str, language: &str) -> Vec<Vec<HighlightSpan>> {
        let config = match self.configs.get(language) {
            Some(c) => c,
            None => {
                return vec![vec![HighlightSpan {
                    token: HighlightToken::Text,
                    start_col: 0,
                    end_col: code.len(),
                }]];
            }
        };

        let mut highlighter = Highlighter::new();
        let highlights = match highlighter.highlight(config, code.as_bytes(), None, |_| None) {
            Ok(h) => h,
            Err(e) => {
                tracing::warn!("wasm tree-sitter highlight error: {e}");
                return vec![vec![HighlightSpan {
                    token: HighlightToken::Text,
                    start_col: 0,
                    end_col: code.len(),
                }]];
            }
        };

        let lines: Vec<&str> = code.lines().collect();
        let line_count = lines.len();
        let mut result: Vec<Vec<HighlightSpan>> = vec![Vec::new(); line_count];

        let mut line_starts: Vec<usize> = Vec::with_capacity(line_count + 1);
        let mut offset = 0;
        for &line in &lines {
            line_starts.push(offset);
            offset += line.len() + 1;
        }
        line_starts.push(offset);

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

impl Default for WasmTreeSitterHighlighter {
    fn default() -> Self {
        Self::with_defaults()
    }
}

impl HighlightProvider for WasmTreeSitterHighlighter {
    fn highlight_line(&self, line: &str, _in_code_block: &mut bool) -> Vec<HighlightSpan> {
        if line.is_empty() {
            return Vec::new();
        }
        vec![HighlightSpan {
            token: HighlightToken::Text,
            start_col: 0,
            end_col: line.len(),
        }]
    }

    fn highlight_document(&self, text: &str) -> Vec<Vec<HighlightSpan>> {
        if self.configs.is_empty() {
            let mut in_code_block = false;
            return text
                .lines()
                .map(|line| self.highlight_line(line, &mut in_code_block))
                .collect();
        }

        let language = self
            .configs
            .keys()
            .next()
            .map(|s| s.as_str())
            .unwrap_or("text");
        self.highlight_code(text, language)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_with_default_url() {
        let h = WasmTreeSitterHighlighter::with_defaults();
        assert_eq!(h.base_url(), "/grammars/");
    }

    #[test]
    fn new_custom_url_adds_trailing_slash() {
        let h = WasmTreeSitterHighlighter::new("https://cdn.example.com/grammars");
        assert_eq!(h.base_url(), "https://cdn.example.com/grammars/");
    }

    #[test]
    fn new_custom_url_preserves_trailing_slash() {
        let h = WasmTreeSitterHighlighter::new("https://cdn.example.com/grammars/");
        assert_eq!(h.base_url(), "https://cdn.example.com/grammars/");
    }

    #[test]
    fn highlight_line_returns_text_token() {
        let h = WasmTreeSitterHighlighter::new("/grammars/");
        let mut in_cb = false;
        let spans = h.highlight_line("fn main() {", &mut in_cb);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].token, HighlightToken::Text);
        assert_eq!(spans[0].start_col, 0);
        assert_eq!(spans[0].end_col, "fn main() {".len());
    }

    #[test]
    fn highlight_empty_line() {
        let h = WasmTreeSitterHighlighter::new("/grammars/");
        let mut in_cb = false;
        let spans = h.highlight_line("", &mut in_cb);
        assert!(spans.is_empty());
    }

    #[test]
    fn highlight_document_returns_per_line_spans() {
        let h = WasmTreeSitterHighlighter::new("/grammars/");
        let text = "line one\nline two\nline three\n";
        let spans = h.highlight_document(text);
        assert_eq!(spans.len(), 3);
        for line_spans in &spans {
            assert_eq!(line_spans.len(), 1);
            assert_eq!(line_spans[0].token, HighlightToken::Text);
        }
        assert_eq!(spans[0][0].end_col, "line one".len());
        assert_eq!(spans[1][0].end_col, "line two".len());
        assert_eq!(spans[2][0].end_col, "line three".len());
    }

    #[test]
    fn grammar_wasm_filename() {
        assert_eq!(
            WasmTreeSitterHighlighter::grammar_wasm_filename("rust"),
            "tree-sitter-rust.wasm"
        );
        assert_eq!(
            WasmTreeSitterHighlighter::grammar_wasm_filename("python"),
            "tree-sitter-python.wasm"
        );
    }

    #[test]
    fn preload_grammar_is_noop() {
        let mut h = WasmTreeSitterHighlighter::new("/grammars/");
        h.preload_grammar("rust");
        assert!(!h.is_grammar_loaded("rust"));
    }

    #[test]
    fn loaded_languages_empty_initially() {
        let h = WasmTreeSitterHighlighter::new("/grammars/");
        assert!(h.loaded_languages().is_empty());
    }

    #[test]
    fn default_trait_impl() {
        let h = WasmTreeSitterHighlighter::default();
        assert_eq!(h.base_url(), "/grammars/");
    }
}
