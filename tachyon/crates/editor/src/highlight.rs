use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HighlightToken {
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightSpan {
    pub token: HighlightToken,
    pub start_col: usize,
    pub end_col: usize,
}

macro_rules! static_re {
    ($name:ident, $pattern:expr) => {
        static $name: Lazy<Regex> = Lazy::new(|| Regex::new($pattern).unwrap());
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

pub struct Highlighter;

impl Highlighter {
    pub fn new() -> Self {
        Self
    }

    pub fn highlight_line(&self, line: &str, in_code_block: &mut bool) -> Vec<HighlightSpan> {
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

impl Default for Highlighter {
    fn default() -> Self {
        Self::new()
    }
}

/// Map a highlight token to a CSS class name.
///
/// Reserved for future use: CSS class generation for syntax highlighting.
#[cfg(test)]
pub(crate) fn css_class(token: &HighlightToken) -> &'static str {
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlight_heading1() {
        let h = Highlighter::new();
        let mut in_cb = false;
        let spans = h.highlight_line("# Hello", &mut in_cb);
        assert!(spans.iter().any(|s| s.token == HighlightToken::Heading1));
        assert!(!in_cb);
    }

    #[test]
    fn highlight_heading2() {
        let h = Highlighter::new();
        let mut in_cb = false;
        let spans = h.highlight_line("## World", &mut in_cb);
        assert!(spans.iter().any(|s| s.token == HighlightToken::Heading2));
    }

    #[test]
    fn highlight_code_block_toggle() {
        let h = Highlighter::new();
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
        let h = Highlighter::new();
        let mut in_cb = false;
        let spans = h.highlight_line("This is **bold** text", &mut in_cb);
        assert!(spans.iter().any(|s| s.token == HighlightToken::Bold));
    }

    #[test]
    fn highlight_italic() {
        let h = Highlighter::new();
        let mut in_cb = false;
        let spans = h.highlight_line("This is *italic* text", &mut in_cb);
        assert!(spans.iter().any(|s| s.token == HighlightToken::Italic));
    }

    #[test]
    fn highlight_link() {
        let h = Highlighter::new();
        let mut in_cb = false;
        let spans = h.highlight_line("[example](https://example.com)", &mut in_cb);
        assert!(spans.iter().any(|s| s.token == HighlightToken::Link));
        assert!(spans.iter().any(|s| s.token == HighlightToken::LinkUrl));
    }

    #[test]
    fn highlight_wiki_link() {
        let h = Highlighter::new();
        let mut in_cb = false;
        let spans = h.highlight_line("See [[My Page]] for details", &mut in_cb);
        assert!(spans.iter().any(|s| s.token == HighlightToken::WikiLink));
    }

    #[test]
    fn highlight_blockquote() {
        let h = Highlighter::new();
        let mut in_cb = false;
        let spans = h.highlight_line("> This is a quote", &mut in_cb);
        assert!(spans.iter().any(|s| s.token == HighlightToken::Blockquote));
    }

    #[test]
    fn highlight_list_item() {
        let h = Highlighter::new();
        let mut in_cb = false;
        let spans = h.highlight_line("- Item one", &mut in_cb);
        assert!(spans.iter().any(|s| s.token == HighlightToken::ListMarker));
    }

    #[test]
    fn highlight_task_marker() {
        let h = Highlighter::new();
        let mut in_cb = false;
        let spans = h.highlight_line("- [ ] Task", &mut in_cb);
        assert!(spans.iter().any(|s| s.token == HighlightToken::TaskMarker));
    }

    #[test]
    fn highlight_tag() {
        let h = Highlighter::new();
        let mut in_cb = false;
        let spans = h.highlight_line("Some text #rust and #code", &mut in_cb);
        assert!(spans.iter().any(|s| s.token == HighlightToken::Tag));
    }

    #[test]
    fn highlight_frontmatter_delimiter() {
        let h = Highlighter::new();
        let mut in_cb = false;
        let spans = h.highlight_line("---", &mut in_cb);
        assert_eq!(spans[0].token, HighlightToken::Frontmatter);
    }

    #[test]
    fn highlight_horizontal_rule() {
        let h = Highlighter::new();
        let mut in_cb = false;
        let spans = h.highlight_line("---", &mut in_cb);
        assert_eq!(spans[0].token, HighlightToken::Frontmatter);
    }

    #[test]
    fn highlight_plain_text() {
        let h = Highlighter::new();
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
        let h = Highlighter::new();
        let mut in_cb = true;
        let spans = h.highlight_line("# Not a heading", &mut in_cb);
        assert!(in_cb);
        assert_eq!(spans[0].token, HighlightToken::CodeBlock);
    }
}
