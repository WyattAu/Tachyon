//! Composite highlighter: markdown structure via regex + code blocks via tree-sitter.
//!
//! Only compiled behind `native-tree-sitter` feature. Uses [`RegexHighlighter`]
//! for markdown tokens and [`TreeSitterHighlighter`] for fenced code block
//! interiors, auto-detecting language from `` ```lang `` markers.

use std::sync::LazyLock;

use regex::Regex;

use super::tree_sitter::{TreeSitterHighlighter, TsLanguage};
use super::{HighlightProvider, HighlightSpan, RegexHighlighter};

macro_rules! static_re {
    ($name:ident, $pattern:expr) => {
        static $name: LazyLock<Regex> = LazyLock::new(|| Regex::new($pattern).unwrap());
    };
}

static_re!(FENCE_LANG_RE, r"^(`{3,})(\w*)");
static_re!(FENCE_CLOSE_RE, r"^(`{3,})\s*$");

/// A markdown-aware highlighter that uses regex for structure tokens
/// (headings, bold, links, etc.) and tree-sitter for syntax highlighting
/// inside fenced code blocks.
///
/// Language is auto-detected from `` ```lang `` markers. Unsupported or
/// missing language tags fall back to plain `CodeBlock` token (no syntax highlighting).
///
/// # Example
///
/// ```ignore
/// let composite = CompositeHighlighter::new();
/// let spans = composite.highlight_document("```rust\nfn main() {}\n```");
/// // Line 0: CodeBlock
/// // Line 1: Keyword, Function, Punctuation, etc.
/// // Line 2: CodeBlock
/// ```
pub struct CompositeHighlighter {
    regex: RegexHighlighter,
}

impl CompositeHighlighter {
    pub fn new() -> Self {
        Self {
            regex: RegexHighlighter::new(),
        }
    }

    /// Parse a document into code block regions.
    ///
    /// Each block is `(start_line_idx, end_line_idx_exclusive, Option<TsLanguage>)`.
    fn parse_code_blocks(lines: &[&str]) -> Vec<(usize, usize, Option<TsLanguage>)> {
        let mut blocks = Vec::new();
        let mut i = 0;
        while i < lines.len() {
            if let Some(caps) = FENCE_LANG_RE.captures(lines[i]) {
                let lang_tag = caps.get(2).map(|m| m.as_str()).unwrap_or("");
                let start = i;
                i += 1;
                while i < lines.len() && !FENCE_CLOSE_RE.is_match(lines[i]) {
                    i += 1;
                }
                // i now points to closing fence (or past end).
                // Include the closing fence line in the block so regex marks it CodeBlock.
                if i < lines.len() {
                    i += 1;
                }
                let end = i;
                let lang = TsLanguage::from_name(lang_tag);
                blocks.push((start, end, lang));
            } else {
                i += 1;
            }
        }
        blocks
    }
}

impl Default for CompositeHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

impl HighlightProvider for CompositeHighlighter {
    /// Line-by-line fallback: regex only, no tree-sitter context.
    fn highlight_line(&self, line: &str, in_code_block: &mut bool) -> Vec<HighlightSpan> {
        self.regex.highlight_line(line, in_code_block)
    }

    /// Full-document highlight: regex for markdown, tree-sitter for code blocks.
    fn highlight_document(&self, text: &str) -> Vec<Vec<HighlightSpan>> {
        let lines: Vec<&str> = text.lines().collect();
        let line_count = lines.len();
        let blocks = Self::parse_code_blocks(&lines);

        // First pass: regex spans for all lines (preserving code-block state).
        let mut result: Vec<Vec<HighlightSpan>> = Vec::with_capacity(line_count);
        let mut cb = false;
        for line in text.lines() {
            result.push(self.regex.highlight_line(line, &mut cb));
        }

        // Second pass: override code block interiors with tree-sitter spans.
        for (start, end, lang) in &blocks {
            let start = *start;
            let end = (*end).min(line_count);

            // Skip fence opener line (index 0 in block).
            // Check if last line is a closing fence; skip it too.
            let interior_start = start + 1;
            let has_close =
                end > start + 1 && end <= line_count && FENCE_CLOSE_RE.is_match(lines[end - 1]);
            let interior_end = if has_close { end - 1 } else { end };

            if interior_start >= interior_end {
                continue; // Empty code block (opener only, or opener+closer).
            }

            let interior_text: String = lines[interior_start..interior_end].join("\n");

            if let Some(lang) = lang {
                let ts = TreeSitterHighlighter::new(*lang);
                let ts_spans = ts.highlight_document(&interior_text);

                // Map tree-sitter spans back to document line indices.
                for (local_idx, spans) in ts_spans.iter().enumerate() {
                    let doc_idx = interior_start + local_idx;
                    if doc_idx < line_count {
                        result[doc_idx] = spans.clone();
                    }
                }
            }
            // None: keep regex CodeBlock spans (already set in first pass).
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::super::HighlightToken;
    use super::*;

    #[test]
    fn parse_rust_code_block() {
        let lines = vec![
            "```rust",
            "fn main() {",
            "    println!(\"hello\");",
            "}",
            "```",
        ];
        let blocks = CompositeHighlighter::parse_code_blocks(&lines);
        assert_eq!(blocks.len(), 1);
        let (start, end, lang) = &blocks[0];
        assert_eq!(*start, 0);
        assert_eq!(*end, 5); // includes closing fence
        assert_eq!(*lang, Some(TsLanguage::Rust));
    }

    #[test]
    fn parse_python_code_block() {
        let lines = vec!["```python", "def foo():", "    pass", "```"];
        let blocks = CompositeHighlighter::parse_code_blocks(&lines);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].2, Some(TsLanguage::Python));
    }

    #[test]
    fn parse_unknown_language() {
        let lines = vec!["```brainfuck", "+[-->+<]", "```"];
        let blocks = CompositeHighlighter::parse_code_blocks(&lines);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].2, None);
    }

    #[test]
    fn parse_empty_language_tag() {
        let lines = vec!["```", "some code", "```"];
        let blocks = CompositeHighlighter::parse_code_blocks(&lines);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].2, None);
    }

    #[test]
    fn parse_multiple_code_blocks() {
        let lines = vec![
            "# Title",
            "```rust",
            "let x = 1;",
            "```",
            "some text",
            "```python",
            "print('hi')",
            "```",
        ];
        let blocks = CompositeHighlighter::parse_code_blocks(&lines);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].2, Some(TsLanguage::Rust));
        assert_eq!(blocks[1].2, Some(TsLanguage::Python));
    }

    #[test]
    fn composite_highlights_rust_block() {
        let c = CompositeHighlighter::new();
        let doc = "```rust\nfn main() {\n    println!(\"hello\");\n}\n```";
        let spans = c.highlight_document(doc);

        // 5 lines total
        assert_eq!(spans.len(), 5);

        // Line 0 (fence): CodeBlock token
        assert!(
            spans[0]
                .iter()
                .any(|s| s.token == HighlightToken::CodeBlock)
        );

        // Line 1 (fn main): should have Keyword and Function from tree-sitter
        let line1_tokens: Vec<&HighlightToken> = spans[1].iter().map(|s| &s.token).collect();
        assert!(line1_tokens.contains(&&HighlightToken::Keyword));
        assert!(line1_tokens.contains(&&HighlightToken::Function));

        // Line 4 (closing fence): CodeBlock token
        assert!(
            spans[4]
                .iter()
                .any(|s| s.token == HighlightToken::CodeBlock)
        );
    }

    #[test]
    fn composite_keeps_markdown_structure() {
        let c = CompositeHighlighter::new();
        let doc = "# Heading\n\n```rust\nlet x = 1;\n```\n\n**bold text**";
        let spans = c.highlight_document(doc);

        // 7 lines
        assert_eq!(spans.len(), 7);

        // Line 0: Heading1
        assert!(spans[0].iter().any(|s| s.token == HighlightToken::Heading1));

        // Line 6: Bold
        assert!(spans[6].iter().any(|s| s.token == HighlightToken::Bold));
    }

    #[test]
    fn composite_unknown_lang_fallback() {
        let c = CompositeHighlighter::new();
        let doc = "```brainfuck\n++--<>[]\n```";
        let spans = c.highlight_document(doc);

        assert_eq!(spans.len(), 3);
        // All lines should have CodeBlock (regex fallback)
        for line_spans in &spans {
            assert!(
                line_spans
                    .iter()
                    .any(|s| s.token == HighlightToken::CodeBlock)
            );
        }
    }

    #[test]
    fn composite_empty_document() {
        let c = CompositeHighlighter::new();
        let spans = c.highlight_document("");
        assert!(spans.is_empty());
    }

    #[test]
    fn highlight_line_fallback() {
        let c = CompositeHighlighter::new();
        let mut cb = false;
        let spans = c.highlight_line("# Hello", &mut cb);
        assert!(spans.iter().any(|s| s.token == HighlightToken::Heading1));
    }
}
