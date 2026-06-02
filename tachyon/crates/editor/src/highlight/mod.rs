#[cfg(feature = "native-tree-sitter")]
pub mod composite;
#[cfg(feature = "native-tree-sitter")]
pub mod tree_sitter;

use std::sync::LazyLock;

use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HighlightToken {
    // ─── Markdown tokens ───────────────────────────────────────────
    Heading1,
    Heading2,
    Heading3,
    Heading4,
    Heading5,
    Heading6,
    Bold,
    Italic,
    BoldItalic,
    Strikethrough,
    CodeInline,
    CodeBlock,
    Link,
    LinkUrl,
    LinkText,
    Image,
    ImageUrl,
    ImageAlt,
    Blockquote,
    ListItem,
    ListMarker,
    HorizontalRule,
    TableHeader,
    TableCell,
    TableBorder,
    WikiLink,
    Frontmatter,
    Tag,
    TaskMarker,
    Text,
    Whitespace,
    // ─── Code tokens (tree-sitter) ──────────────────────────────────
    Keyword,
    String,
    Number,
    Comment,
    Function,
    Type,
    Variable,
    Operator,
    Property,
    Punctuation,
    Constant,
    Attribute,
    CodeTag,
    Label,
    Embedded,
    Constructor,
    Character,
    Boolean,
    Conditional,
    Repeat,
    Define,
    Include,
    FunctionBuiltin,
    TypeBuiltin,
    VariableBuiltin,
    VariableParameter,
    StringEscape,
    StringSpecial,
    PunctuationBracket,
    PunctuationDelimiter,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightSpan {
    pub token: HighlightToken,
    pub start_col: usize,
    pub end_col: usize,
}

macro_rules! static_re {
    ($name:ident, $pattern:expr) => {
        static $name: LazyLock<Regex> = LazyLock::new(|| Regex::new($pattern).unwrap());
    };
}

static_re!(HEADING_RE, r"^(#{1,6})\s");
static_re!(BOLD_ITALIC_RE, r"\*\*\*[^*]+\*\*\*|___[^_]+___");
static_re!(BOLD_RE, r"\*\*[^*]+\*\*|__[^_]+__");
static_re!(
    ITALIC_RE,
    r"(?:^|\s)\*[^*\n]+\*|\*[^*\n]+\*(?:\s|$)|^_[^_\n]+_|_[^_\n]+_$"
);
static_re!(CODE_INLINE_RE, r"`([^`]+)`");
static_re!(CODE_BLOCK_START_RE, r"^(`{3,}).*");
static_re!(CODE_BLOCK_END_RE, r"^(`{3,})\s*$");
static_re!(LINK_RE, r"\[([^\]]+)\]\(([^)]+)\)");
static_re!(WIKI_LINK_RE, r"\[\[([^\]]+)\]\]");
static_re!(BLOCKQUOTE_RE, r"^(\s*>\s?)");
static_re!(LIST_ITEM_RE, r"^(\s*[-*+])\s");
static_re!(HORIZONTAL_RULE_RE, r"^(\s*([-*_])\s*){3,}\s*$");
static_re!(TASK_MARKER_RE, r"^(\s*[-*+])\s\[[ xX]\]");
static_re!(TAG_RE, r"(?:^|\s)#[a-zA-Z0-9_\-/]+");
static_re!(TABLE_RE, r"^\|?[\s\-:|]+\|?$");
static_re!(STRIKETHROUGH_RE, r"(~~)(.+?)~~");
static_re!(FRONTMATTER_DELIMITER_RE, r"^---\s*$");

/// Pluggable syntax highlighting backend.
///
/// Implementors produce [`HighlightSpan`] vectors for each line of text.
/// The editor holds `Box<dyn HighlightProvider>` and does not care whether
/// the backend is regex-based (markdown), tree-sitter (code), or WASM-loaded.
pub trait HighlightProvider: Send + Sync {
    /// Highlight a single line, returning ordered non-overlapping spans.
    ///
    /// `in_code_block` tracks fenced-code-block state across calls so the
    /// provider can suppress markdown tokens inside code fences.
    fn highlight_line(&self, line: &str, in_code_block: &mut bool) -> Vec<HighlightSpan>;

    /// Highlight an entire document, returning spans per line.
    ///
    /// Default implementation iterates [`highlight_line`](Self::highlight_line)
    /// for each line. Tree-sitter-backed providers override this to parse the
    /// full document once and split highlights by line boundaries.
    fn highlight_document(&self, text: &str) -> Vec<Vec<HighlightSpan>> {
        let mut in_code_block = false;
        text.lines()
            .map(|line| self.highlight_line(line, &mut in_code_block))
            .collect()
    }
}

/// Markdown highlighter powered by compile-time regex patterns.
///
/// Suitable for markdown/markdown-like content. For programming-language
/// syntax, use a tree-sitter-backed provider (Phase A2+).
pub struct RegexHighlighter;

impl HighlightProvider for RegexHighlighter {
    fn highlight_line(&self, line: &str, in_code_block: &mut bool) -> Vec<HighlightSpan> {
        if CODE_BLOCK_START_RE.is_match(line) && !*in_code_block {
            *in_code_block = true;
            return vec![HighlightSpan {
                token: HighlightToken::CodeBlock,
                start_col: 0,
                end_col: line.len(),
            }];
        }

        if CODE_BLOCK_END_RE.is_match(line) && *in_code_block {
            *in_code_block = false;
            return vec![HighlightSpan {
                token: HighlightToken::CodeBlock,
                start_col: 0,
                end_col: line.len(),
            }];
        }

        if *in_code_block {
            return vec![HighlightSpan {
                token: HighlightToken::CodeBlock,
                start_col: 0,
                end_col: line.len(),
            }];
        }

        if FRONTMATTER_DELIMITER_RE.is_match(line) {
            return vec![HighlightSpan {
                token: HighlightToken::Frontmatter,
                start_col: 0,
                end_col: line.len(),
            }];
        }

        let mut spans: Vec<(usize, usize, HighlightToken)> = Vec::new();

        if let Some(m) = HEADING_RE.find(line) {
            let level = m.as_str().chars().take_while(|&c| c == '#').count();
            let token = match level {
                1 => HighlightToken::Heading1,
                2 => HighlightToken::Heading2,
                3 => HighlightToken::Heading3,
                4 => HighlightToken::Heading4,
                5 => HighlightToken::Heading5,
                _ => HighlightToken::Heading6,
            };
            spans.push((m.start(), m.end(), token));
        }

        if let Some(m) = BLOCKQUOTE_RE.find(line) {
            spans.push((m.start(), m.end(), HighlightToken::Blockquote));
        }

        if let Some(m) = TASK_MARKER_RE.find(line) {
            spans.push((m.start(), m.end(), HighlightToken::TaskMarker));
        } else if let Some(m) = LIST_ITEM_RE.find(line) {
            spans.push((m.start(), m.end(), HighlightToken::ListMarker));
        }

        if HORIZONTAL_RULE_RE.is_match(line) {
            return vec![HighlightSpan {
                token: HighlightToken::HorizontalRule,
                start_col: 0,
                end_col: line.len(),
            }];
        }

        if TABLE_RE.is_match(line) {
            return vec![HighlightSpan {
                token: HighlightToken::TableBorder,
                start_col: 0,
                end_col: line.len(),
            }];
        }

        for m in WIKI_LINK_RE.find_iter(line) {
            spans.push((m.start(), m.end(), HighlightToken::WikiLink));
        }

        for m in LINK_RE.find_iter(line) {
            spans.push((m.start(), m.end(), HighlightToken::Link));
            for caps in LINK_RE.captures_iter(m.as_str()) {
                if let Some(text_m) = caps.get(1) {
                    let abs_start = m.start() + text_m.start();
                    let abs_end = m.start() + text_m.end();
                    spans.push((abs_start, abs_end, HighlightToken::LinkText));
                }
                if let Some(url_m) = caps.get(2) {
                    let abs_start = m.start() + url_m.start();
                    let abs_end = m.start() + url_m.end();
                    spans.push((abs_start, abs_end, HighlightToken::LinkUrl));
                }
            }
        }

        for m in STRIKETHROUGH_RE.find_iter(line) {
            spans.push((m.start(), m.end(), HighlightToken::Strikethrough));
        }

        for m in BOLD_ITALIC_RE.find_iter(line) {
            spans.push((m.start(), m.end(), HighlightToken::BoldItalic));
        }

        for m in BOLD_RE.find_iter(line) {
            spans.push((m.start(), m.end(), HighlightToken::Bold));
        }

        for m in ITALIC_RE.find_iter(line) {
            spans.push((m.start(), m.end(), HighlightToken::Italic));
        }

        for m in CODE_INLINE_RE.find_iter(line) {
            spans.push((m.start(), m.end(), HighlightToken::CodeInline));
        }

        for m in TAG_RE.find_iter(line) {
            spans.push((m.start(), m.end(), HighlightToken::Tag));
        }

        spans.sort_by_key(|(start, _, _)| *start);

        let mut result = Vec::new();
        let mut pos = 0;

        for (start, end, token) in spans {
            if start > pos {
                let gap = &line[pos..start];
                if gap.is_empty() {
                    // skip
                } else if gap.chars().all(|c| c.is_whitespace()) {
                    result.push(HighlightSpan {
                        token: HighlightToken::Whitespace,
                        start_col: pos,
                        end_col: start,
                    });
                } else {
                    result.push(HighlightSpan {
                        token: HighlightToken::Text,
                        start_col: pos,
                        end_col: start,
                    });
                }
            }
            result.push(HighlightSpan {
                token,
                start_col: start,
                end_col: end,
            });
            pos = end;
        }

        if pos < line.len() {
            let rest = &line[pos..];
            if rest.is_empty() {
                // skip
            } else if rest.chars().all(|c| c.is_whitespace()) {
                result.push(HighlightSpan {
                    token: HighlightToken::Whitespace,
                    start_col: pos,
                    end_col: line.len(),
                });
            } else {
                result.push(HighlightSpan {
                    token: HighlightToken::Text,
                    start_col: pos,
                    end_col: line.len(),
                });
            }
        }

        result
    }
}

impl RegexHighlighter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RegexHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

/// Map a highlight token to a CSS class name.
///
/// Used by frontend rendering to apply syntax-highlight colors.
pub fn css_class(token: &HighlightToken) -> &'static str {
    match token {
        HighlightToken::Heading1 => "ed-h1",
        HighlightToken::Heading2 => "ed-h2",
        HighlightToken::Heading3 => "ed-h3",
        HighlightToken::Heading4 => "ed-h4",
        HighlightToken::Heading5 => "ed-h5",
        HighlightToken::Heading6 => "ed-h6",
        HighlightToken::Bold => "ed-bold",
        HighlightToken::Italic => "ed-italic",
        HighlightToken::BoldItalic => "ed-bold-italic",
        HighlightToken::Strikethrough => "ed-strikethrough",
        HighlightToken::CodeInline => "ed-code-inline",
        HighlightToken::Link => "ed-link",
        HighlightToken::LinkUrl => "ed-link-url",
        HighlightToken::LinkText => "ed-link-text",
        HighlightToken::Image => "ed-image",
        HighlightToken::ImageUrl => "ed-image-url",
        HighlightToken::ImageAlt => "ed-image-alt",
        HighlightToken::WikiLink => "ed-wiki-link",
        HighlightToken::Blockquote => "ed-blockquote",
        HighlightToken::ListItem => "ed-list-item",
        HighlightToken::ListMarker => "ed-list-marker",
        HighlightToken::HorizontalRule => "ed-hr",
        HighlightToken::CodeBlock => "ed-code-block",
        HighlightToken::Frontmatter => "ed-frontmatter",
        HighlightToken::Tag => "ed-tag",
        HighlightToken::TaskMarker => "ed-task-marker",
        HighlightToken::TableHeader => "ed-table-header",
        HighlightToken::TableCell => "ed-table-cell",
        HighlightToken::TableBorder => "ed-table-border",
        HighlightToken::Text => "ed-text",
        HighlightToken::Whitespace => "ed-whitespace",
        // Code tokens
        HighlightToken::Keyword => "ed-keyword",
        HighlightToken::String => "ed-string",
        HighlightToken::Number => "ed-number",
        HighlightToken::Comment => "ed-comment",
        HighlightToken::Function => "ed-function",
        HighlightToken::Type => "ed-type",
        HighlightToken::Variable => "ed-variable",
        HighlightToken::Operator => "ed-operator",
        HighlightToken::Property => "ed-property",
        HighlightToken::Punctuation => "ed-punctuation",
        HighlightToken::Constant => "ed-constant",
        HighlightToken::Attribute => "ed-attribute",
        HighlightToken::CodeTag => "ed-tag-code",
        HighlightToken::Label => "ed-label",
        HighlightToken::Embedded => "ed-embedded",
        HighlightToken::Constructor => "ed-constructor",
        HighlightToken::Character => "ed-character",
        HighlightToken::Boolean => "ed-boolean",
        HighlightToken::Conditional => "ed-conditional",
        HighlightToken::Repeat => "ed-repeat",
        HighlightToken::Define => "ed-define",
        HighlightToken::Include => "ed-include",
        HighlightToken::FunctionBuiltin => "ed-function-builtin",
        HighlightToken::TypeBuiltin => "ed-type-builtin",
        HighlightToken::VariableBuiltin => "ed-variable-builtin",
        HighlightToken::VariableParameter => "ed-variable-parameter",
        HighlightToken::StringEscape => "ed-string-escape",
        HighlightToken::StringSpecial => "ed-string-special",
        HighlightToken::PunctuationBracket => "ed-punctuation-bracket",
        HighlightToken::PunctuationDelimiter => "ed-punctuation-delimiter",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlight_heading1() {
        let h = RegexHighlighter::new();
        let mut in_cb = false;
        let spans = h.highlight_line("# Hello", &mut in_cb);
        assert!(spans.iter().any(|s| s.token == HighlightToken::Heading1));
        assert!(!in_cb);
    }

    #[test]
    fn highlight_heading2() {
        let h = RegexHighlighter::new();
        let mut in_cb = false;
        let spans = h.highlight_line("## World", &mut in_cb);
        assert!(spans.iter().any(|s| s.token == HighlightToken::Heading2));
    }

    #[test]
    fn highlight_code_block_toggle() {
        let h = RegexHighlighter::new();
        let mut in_cb = false;
        let spans1 = h.highlight_line("```rust", &mut in_cb);
        assert!(in_cb);
        assert_eq!(spans1[0].token, HighlightToken::CodeBlock);

        let spans2 = h.highlight_line("let x = 1;", &mut in_cb);
        assert_eq!(spans2[0].token, HighlightToken::CodeBlock);
        assert!(in_cb);

        let spans3 = h.highlight_line("```", &mut in_cb);
        assert!(!in_cb);
        assert_eq!(spans3[0].token, HighlightToken::CodeBlock);
    }

    #[test]
    fn highlight_bold() {
        let h = RegexHighlighter::new();
        let mut in_cb = false;
        let spans = h.highlight_line("This is **bold** text", &mut in_cb);
        assert!(spans.iter().any(|s| s.token == HighlightToken::Bold));
    }

    #[test]
    fn highlight_italic() {
        let h = RegexHighlighter::new();
        let mut in_cb = false;
        let spans = h.highlight_line("This is *italic* text", &mut in_cb);
        assert!(spans.iter().any(|s| s.token == HighlightToken::Italic));
    }

    #[test]
    fn highlight_link() {
        let h = RegexHighlighter::new();
        let mut in_cb = false;
        let spans = h.highlight_line("[example](https://example.com)", &mut in_cb);
        assert!(spans.iter().any(|s| s.token == HighlightToken::Link));
        assert!(spans.iter().any(|s| s.token == HighlightToken::LinkUrl));
    }

    #[test]
    fn highlight_wiki_link() {
        let h = RegexHighlighter::new();
        let mut in_cb = false;
        let spans = h.highlight_line("See [[My Page]] for details", &mut in_cb);
        assert!(spans.iter().any(|s| s.token == HighlightToken::WikiLink));
    }

    #[test]
    fn highlight_blockquote() {
        let h = RegexHighlighter::new();
        let mut in_cb = false;
        let spans = h.highlight_line("> This is a quote", &mut in_cb);
        assert!(spans.iter().any(|s| s.token == HighlightToken::Blockquote));
    }

    #[test]
    fn highlight_list_item() {
        let h = RegexHighlighter::new();
        let mut in_cb = false;
        let spans = h.highlight_line("- Item one", &mut in_cb);
        assert!(spans.iter().any(|s| s.token == HighlightToken::ListMarker));
    }

    #[test]
    fn highlight_task_marker() {
        let h = RegexHighlighter::new();
        let mut in_cb = false;
        let spans = h.highlight_line("- [ ] Task", &mut in_cb);
        assert!(spans.iter().any(|s| s.token == HighlightToken::TaskMarker));
    }

    #[test]
    fn highlight_tag() {
        let h = RegexHighlighter::new();
        let mut in_cb = false;
        let spans = h.highlight_line("Some text #rust and #code", &mut in_cb);
        assert!(spans.iter().any(|s| s.token == HighlightToken::Tag));
    }

    #[test]
    fn highlight_frontmatter_delimiter() {
        let h = RegexHighlighter::new();
        let mut in_cb = false;
        let spans = h.highlight_line("---", &mut in_cb);
        assert_eq!(spans[0].token, HighlightToken::Frontmatter);
    }

    #[test]
    fn highlight_horizontal_rule() {
        let h = RegexHighlighter::new();
        let mut in_cb = false;
        let spans = h.highlight_line("---", &mut in_cb);
        assert_eq!(spans[0].token, HighlightToken::Frontmatter);
    }

    #[test]
    fn highlight_plain_text() {
        let h = RegexHighlighter::new();
        let mut in_cb = false;
        let spans = h.highlight_line("Just plain text", &mut in_cb);
        assert!(spans.iter().any(|s| s.token == HighlightToken::Text));
    }

    #[test]
    fn css_class_mapping() {
        assert_eq!(css_class(&HighlightToken::Heading1), "ed-h1");
        assert_eq!(css_class(&HighlightToken::Bold), "ed-bold");
        assert_eq!(css_class(&HighlightToken::CodeBlock), "ed-code-block");
        assert_eq!(css_class(&HighlightToken::Text), "ed-text");
    }

    #[test]
    fn highlight_inside_code_block() {
        let h = RegexHighlighter::new();
        let mut in_cb = true;
        let spans = h.highlight_line("# Not a heading", &mut in_cb);
        assert!(in_cb);
        assert_eq!(spans[0].token, HighlightToken::CodeBlock);
    }
}
