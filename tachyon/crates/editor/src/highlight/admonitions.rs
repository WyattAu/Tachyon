use std::sync::LazyLock;

use regex::Regex;

use super::{HighlightSpan, HighlightToken};

macro_rules! static_re {
    ($name:ident, $pattern:expr) => {
        static $name: LazyLock<Regex> = LazyLock::new(|| Regex::new($pattern).unwrap());
    };
}

static_re!(
    ADMONITION_HEADER_RE,
    r"^>\s*\[!((?:note|tip|info|warning|danger|caution))\]\s*$"
);
static_re!(
    ADMONITION_CONTINUATION_RE,
    r"^>\s?(.*)"
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmonitionType {
    Note,
    Tip,
    Info,
    Warning,
    Danger,
    Caution,
}

impl AdmonitionType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "note" => Some(Self::Note),
            "tip" => Some(Self::Tip),
            "info" => Some(Self::Info),
            "warning" => Some(Self::Warning),
            "danger" => Some(Self::Danger),
            "caution" => Some(Self::Caution),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Note => "note",
            Self::Tip => "tip",
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Danger => "danger",
            Self::Caution => "caution",
        }
    }
}

pub fn highlight_admonition_line(line: &str, in_admonition: &mut bool) -> Option<Vec<HighlightSpan>> {
    if let Some(m) = ADMONITION_HEADER_RE.captures(line) {
        let admonition_type = m.get(1).unwrap().as_str();
        *in_admonition = true;
        return Some(vec![HighlightSpan {
            token: HighlightToken::AdmonitionHeader,
            start_col: 0,
            end_col: line.len(),
        }]);
    }

    if *in_admonition {
        if let Some(m) = ADMONITION_CONTINUATION_RE.find(line) {
            let content_start = m.start();
            let content_end = m.end();
            let mut spans = Vec::new();

            spans.push(HighlightSpan {
                token: HighlightToken::Blockquote,
                start_col: 0,
                end_col: content_start,
            });

            if content_end > content_start {
                let content = &line[content_start..content_end];
                if content.trim().is_empty() {
                    spans.push(HighlightSpan {
                        token: HighlightToken::Whitespace,
                        start_col: content_start,
                        end_col: content_end,
                    });
                } else {
                    spans.push(HighlightSpan {
                        token: HighlightToken::AdmonitionContent,
                        start_col: content_start,
                        end_col: content_end,
                    });
                }
            }

            *in_admonition = true;
            return Some(spans);
        } else {
            *in_admonition = false;
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_header() {
        let mut in_adv = false;
        let spans = highlight_admonition_line("> [!note]", &mut in_adv).unwrap();
        assert!(in_adv);
        assert!(spans
            .iter()
            .any(|s| s.token == HighlightToken::AdmonitionHeader));
    }

    #[test]
    fn test_tip_header() {
        let mut in_adv = false;
        let spans = highlight_admonition_line("> [!tip]", &mut in_adv).unwrap();
        assert!(in_adv);
        assert!(spans
            .iter()
            .any(|s| s.token == HighlightToken::AdmonitionHeader));
    }

    #[test]
    fn test_info_header() {
        let mut in_adv = false;
        let spans = highlight_admonition_line("> [!info]", &mut in_adv).unwrap();
        assert!(in_adv);
    }

    #[test]
    fn test_warning_header() {
        let mut in_adv = false;
        let spans = highlight_admonition_line("> [!warning]", &mut in_adv).unwrap();
        assert!(in_adv);
    }

    #[test]
    fn test_danger_header() {
        let mut in_adv = false;
        let spans = highlight_admonition_line("> [!danger]", &mut in_adv).unwrap();
        assert!(in_adv);
    }

    #[test]
    fn test_caution_header() {
        let mut in_adv = false;
        let spans = highlight_admonition_line("> [!caution]", &mut in_adv).unwrap();
        assert!(in_adv);
    }

    #[test]
    fn test_continuation_line() {
        let mut in_adv = true;
        let spans = highlight_admonition_line("> Some content here", &mut in_adv).unwrap();
        assert!(in_adv);
        assert!(spans
            .iter()
            .any(|s| s.token == HighlightToken::AdmonitionContent));
    }

    #[test]
    fn test_non_admonition_blockquote() {
        let mut in_adv = false;
        let result = highlight_admonition_line("> Just a blockquote", &mut in_adv);
        assert!(result.is_none());
        assert!(!in_adv);
    }

    #[test]
    fn test_exits_admonition_on_non_quote() {
        let mut in_adv = true;
        let result = highlight_admonition_line("Regular text", &mut in_adv);
        assert!(result.is_none());
        assert!(!in_adv);
    }

    #[test]
    fn test_admonition_type_from_str() {
        assert_eq!(AdmonitionType::from_str("note"), Some(AdmonitionType::Note));
        assert_eq!(AdmonitionType::from_str("TIP"), Some(AdmonitionType::Tip));
        assert_eq!(AdmonitionType::from_str("info"), Some(AdmonitionType::Info));
        assert_eq!(AdmonitionType::from_str("warning"), Some(AdmonitionType::Warning));
        assert_eq!(AdmonitionType::from_str("danger"), Some(AdmonitionType::Danger));
        assert_eq!(AdmonitionType::from_str("caution"), Some(AdmonitionType::Caution));
        assert_eq!(AdmonitionType::from_str("unknown"), None);
    }
}
