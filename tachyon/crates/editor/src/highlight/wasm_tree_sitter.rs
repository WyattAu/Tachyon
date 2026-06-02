//! WASM tree-sitter syntax highlighter for `wasm32-unknown-unknown`.
//!
//! # Architecture
//!
//! This module loads tree-sitter grammar **WASM files at runtime** from a
//! configurable URL base (CDN, `/grammars/`, etc.), following the
//! CodeMirror / Shiki model where grammar binaries are downloaded on demand
//! rather than compiled into the editor binary.
//!
//! ## Grammar files
//!
//! Expected filenames follow the pattern `tree-sitter-<lang>.wasm`:
//!
//! - `tree-sitter-rust.wasm`
//! - `tree-sitter-python.wasm`
//! - `tree-sitter-javascript.wasm`
//! - `tree-sitter-typescript.wasm`
//! - etc.
//!
//! ## Future integration
//!
//! Currently this stub returns plaintext [`HighlightToken::Text`] tokens as
//! a fallback. The intended integration path is:
//!
//! 1. Use `web-sys` `fetch` API to download grammar `.wasm` bytes into a
//!    [`Vec<u8>`].
//! 2. Feed those bytes into `tree-sitter-highlight-wasm` (or similar WASM-
//!    oriented tree-sitter crate) to obtain parsed syntax highlights.
//! 3. Map highlight captures to [`HighlightToken`] variants using the same
//!    capture names as the native highlighter in [`super::tree_sitter`].
//!
//! ## Why not `wasm-bindgen` in this file?
//!
//! This module compiles as plain Rust (no `wasm-bindgen` dependency) so it
//! can be type-checked with `cargo check --target wasm32-unknown-unknown`
//! without a full JS toolchain. When `web-sys` fetch integration lands, the
//! async runtime will live behind a feature gate.

use std::collections::HashMap;

use crate::highlight::{HighlightProvider, HighlightSpan, HighlightToken};

/// Default base URL for grammar WASM files.
const DEFAULT_GRAMMAR_BASE_URL: &str = "/grammars/";

/// WASM tree-sitter highlighter for browser environments.
///
/// Grammar `.wasm` binaries are fetched from [`base_url`] at runtime and
/// cached in [`loaded_grammars`]. Currently a plaintext stub pending
/// `tree-sitter-highlight-wasm` integration.
pub struct WasmTreeSitterHighlighter {
    /// Base URL (or path) where grammar `.wasm` files are served.
    /// Example: `"https://cdn.example.com/grammars/"`.
    base_url: String,

    /// Cache of language name → grammar WASM bytes.
    loaded_grammars: HashMap<String, Vec<u8>>,
}

impl WasmTreeSitterHighlighter {
    /// Create a new WASM tree-sitter highlighter.
    ///
    /// `base_url` is the prefix for grammar files. A trailing slash is
    /// appended if missing. Defaults to `"/grammars/"`.
    pub fn new(base_url: &str) -> Self {
        let base = if base_url.ends_with('/') {
            base_url.to_owned()
        } else {
            format!("{base_url}/")
        };
        Self {
            base_url: base,
            loaded_grammars: HashMap::new(),
        }
    }

    /// Create with the default grammar base URL (`"/grammars/"`).
    pub fn with_defaults() -> Self {
        Self::new(DEFAULT_GRAMMAR_BASE_URL)
    }

    /// Fetch and cache a grammar `.wasm` file from the configured base URL.
    ///
    /// On `wasm32`, the intended implementation uses `web_sys::fetch` to
    /// download `{base_url}tree-sitter-{language}.wasm` and stores the raw
    /// bytes in [`loaded_grammars`].
    ///
    /// For now this is a no-op stub. It will be implemented once
    /// `web-sys` fetch integration is added.
    pub fn preload_grammar(&mut self, _language: &str) {
        // TODO: use web_sys::fetch to download {base_url}tree-sitter-{language}.wasm
        //       and store bytes in self.loaded_grammars.
    }

    /// Check whether a grammar is already cached in memory.
    pub fn is_grammar_loaded(&self, language: &str) -> bool {
        self.loaded_grammars.contains_key(language)
    }

    /// List the names of all currently loaded grammars.
    pub fn loaded_languages(&self) -> Vec<String> {
        self.loaded_grammars.keys().cloned().collect()
    }

    /// Return the base URL this highlighter is configured with.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Build the expected filename for a grammar WASM file.
    ///
    /// Example: `grammar_wasm_filename("rust")` → `"tree-sitter-rust.wasm"`.
    pub fn grammar_wasm_filename(language: &str) -> String {
        format!("tree-sitter-{language}.wasm")
    }
}

impl Default for WasmTreeSitterHighlighter {
    fn default() -> Self {
        Self::with_defaults()
    }
}

impl HighlightProvider for WasmTreeSitterHighlighter {
    /// Return a plaintext span for the entire line.
    ///
    /// TODO: once `tree-sitter-highlight-wasm` is integrated, this should
    /// delegate to a full-document WASM tree-sitter parse and return the
    /// spans for just this line.
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

    // TODO: override with a single WASM tree-sitter parse of the full
    // document, then split by line boundaries (same strategy as the
    // native `super::tree_sitter::TreeSitterHighlighter`).
    // For now the default HighlightProvider::highlight_document is used
    // (calls highlight_line per line).
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
