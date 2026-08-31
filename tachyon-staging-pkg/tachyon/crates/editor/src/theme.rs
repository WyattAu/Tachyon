//! Syntax highlighting themes.
//!
//! Maps [`HighlightToken`] variants to CSS color values.
//! Ships with Dark, Light, and High Contrast themes. Custom themes via builder.

use crate::highlight::HighlightToken;

/// A complete syntax theme mapping highlight tokens to CSS colors.
#[derive(Debug, Clone)]
pub struct SyntaxTheme {
    name: String,
    colors: Vec<(HighlightToken, String)>,
}

impl SyntaxTheme {
    pub fn name(&self) -> &str {
        &self.name
    }

    /// One Dark inspired theme.
    pub fn dark() -> Self {
        Self {
            name: "dark".into(),
            colors: vec![
                // Markdown tokens
                (HighlightToken::Heading1, "#e06c75".into()),
                (HighlightToken::Heading2, "#e06c75".into()),
                (HighlightToken::Heading3, "#e5c07b".into()),
                (HighlightToken::Heading4, "#61afef".into()),
                (HighlightToken::Heading5, "#56b6c2".into()),
                (HighlightToken::Heading6, "#abb2bf".into()),
                (HighlightToken::Bold, "#e06c75".into()),
                (HighlightToken::Italic, "#c678dd".into()),
                (HighlightToken::BoldItalic, "#e06c75".into()),
                (HighlightToken::Strikethrough, "#e06c75".into()),
                (HighlightToken::CodeInline, "#98c379".into()),
                (HighlightToken::CodeBlock, "#abb2bf".into()),
                (HighlightToken::Link, "#61afef".into()),
                (HighlightToken::LinkUrl, "#98c379".into()),
                (HighlightToken::LinkText, "#61afef".into()),
                (HighlightToken::Image, "#61afef".into()),
                (HighlightToken::ImageUrl, "#98c379".into()),
                (HighlightToken::ImageAlt, "#e06c75".into()),
                (HighlightToken::Blockquote, "#5c6370".into()),
                (HighlightToken::ListItem, "#abb2bf".into()),
                (HighlightToken::ListMarker, "#e06c75".into()),
                (HighlightToken::HorizontalRule, "#5c6370".into()),
                (HighlightToken::TableHeader, "#e5c07b".into()),
                (HighlightToken::TableCell, "#abb2bf".into()),
                (HighlightToken::TableBorder, "#5c6370".into()),
                (HighlightToken::WikiLink, "#61afef".into()),
                (HighlightToken::Frontmatter, "#5c6370".into()),
                (HighlightToken::Tag, "#56b6c2".into()),
                (HighlightToken::TaskMarker, "#e06c75".into()),
                (HighlightToken::Text, "#abb2bf".into()),
                (HighlightToken::Whitespace, "#abb2bf".into()),
                // Code tokens
                (HighlightToken::Keyword, "#c678dd".into()),
                (HighlightToken::String, "#98c379".into()),
                (HighlightToken::Number, "#d19a66".into()),
                (HighlightToken::Comment, "#5c6370".into()),
                (HighlightToken::Function, "#61afef".into()),
                (HighlightToken::Type, "#e5c07b".into()),
                (HighlightToken::Variable, "#e06c75".into()),
                (HighlightToken::Operator, "#56b6c2".into()),
                (HighlightToken::Property, "#e06c75".into()),
                (HighlightToken::Punctuation, "#abb2bf".into()),
                (HighlightToken::Constant, "#e5c07b".into()),
                (HighlightToken::Attribute, "#e06c75".into()),
                (HighlightToken::CodeTag, "#e06c75".into()),
                (HighlightToken::Label, "#c678dd".into()),
                (HighlightToken::Embedded, "#98c379".into()),
                (HighlightToken::Constructor, "#e5c07b".into()),
                (HighlightToken::Character, "#98c379".into()),
                (HighlightToken::Boolean, "#d19a66".into()),
                (HighlightToken::Conditional, "#c678dd".into()),
                (HighlightToken::Repeat, "#c678dd".into()),
                (HighlightToken::Define, "#c678dd".into()),
                (HighlightToken::Include, "#c678dd".into()),
                (HighlightToken::FunctionBuiltin, "#e5c07b".into()),
                (HighlightToken::TypeBuiltin, "#e5c07b".into()),
                (HighlightToken::VariableBuiltin, "#e5c07b".into()),
                (HighlightToken::VariableParameter, "#e06c75".into()),
                (HighlightToken::StringEscape, "#56b6c2".into()),
                (HighlightToken::StringSpecial, "#56b6c2".into()),
                (HighlightToken::PunctuationBracket, "#abb2bf".into()),
                (HighlightToken::PunctuationDelimiter, "#abb2bf".into()),
            ],
        }
    }

    /// One Light inspired theme.
    pub fn light() -> Self {
        Self {
            name: "light".into(),
            colors: vec![
                // Markdown tokens
                (HighlightToken::Heading1, "#e45649".into()),
                (HighlightToken::Heading2, "#e45649".into()),
                (HighlightToken::Heading3, "#986801".into()),
                (HighlightToken::Heading4, "#4078f2".into()),
                (HighlightToken::Heading5, "#0184bc".into()),
                (HighlightToken::Heading6, "#383a42".into()),
                (HighlightToken::Bold, "#e45649".into()),
                (HighlightToken::Italic, "#a626a4".into()),
                (HighlightToken::BoldItalic, "#e45649".into()),
                (HighlightToken::Strikethrough, "#e45649".into()),
                (HighlightToken::CodeInline, "#50a14f".into()),
                (HighlightToken::CodeBlock, "#383a42".into()),
                (HighlightToken::Link, "#4078f2".into()),
                (HighlightToken::LinkUrl, "#50a14f".into()),
                (HighlightToken::LinkText, "#4078f2".into()),
                (HighlightToken::Image, "#4078f2".into()),
                (HighlightToken::ImageUrl, "#50a14f".into()),
                (HighlightToken::ImageAlt, "#e45649".into()),
                (HighlightToken::Blockquote, "#a0a1a7".into()),
                (HighlightToken::ListItem, "#383a42".into()),
                (HighlightToken::ListMarker, "#e45649".into()),
                (HighlightToken::HorizontalRule, "#a0a1a7".into()),
                (HighlightToken::TableHeader, "#986801".into()),
                (HighlightToken::TableCell, "#383a42".into()),
                (HighlightToken::TableBorder, "#a0a1a7".into()),
                (HighlightToken::WikiLink, "#4078f2".into()),
                (HighlightToken::Frontmatter, "#a0a1a7".into()),
                (HighlightToken::Tag, "#0184bc".into()),
                (HighlightToken::TaskMarker, "#e45649".into()),
                (HighlightToken::Text, "#383a42".into()),
                (HighlightToken::Whitespace, "#383a42".into()),
                // Code tokens
                (HighlightToken::Keyword, "#a626a4".into()),
                (HighlightToken::String, "#50a14f".into()),
                (HighlightToken::Number, "#986801".into()),
                (HighlightToken::Comment, "#a0a1a7".into()),
                (HighlightToken::Function, "#4078f2".into()),
                (HighlightToken::Type, "#986801".into()),
                (HighlightToken::Variable, "#e45649".into()),
                (HighlightToken::Operator, "#0184bc".into()),
                (HighlightToken::Property, "#e45649".into()),
                (HighlightToken::Punctuation, "#383a42".into()),
                (HighlightToken::Constant, "#986801".into()),
                (HighlightToken::Attribute, "#e45649".into()),
                (HighlightToken::CodeTag, "#e45649".into()),
                (HighlightToken::Label, "#a626a4".into()),
                (HighlightToken::Embedded, "#50a14f".into()),
                (HighlightToken::Constructor, "#986801".into()),
                (HighlightToken::Character, "#50a14f".into()),
                (HighlightToken::Boolean, "#986801".into()),
                (HighlightToken::Conditional, "#a626a4".into()),
                (HighlightToken::Repeat, "#a626a4".into()),
                (HighlightToken::Define, "#a626a4".into()),
                (HighlightToken::Include, "#a626a4".into()),
                (HighlightToken::FunctionBuiltin, "#986801".into()),
                (HighlightToken::TypeBuiltin, "#c18401".into()),
                (HighlightToken::VariableBuiltin, "#986801".into()),
                (HighlightToken::VariableParameter, "#e45649".into()),
                (HighlightToken::StringEscape, "#0184bc".into()),
                (HighlightToken::StringSpecial, "#0184bc".into()),
                (HighlightToken::PunctuationBracket, "#383a42".into()),
                (HighlightToken::PunctuationDelimiter, "#383a42".into()),
            ],
        }
    }

    /// High contrast accessibility-focused theme (dark background, vivid colors).
    pub fn high_contrast() -> Self {
        Self {
            name: "high_contrast".into(),
            colors: vec![
                // Markdown tokens
                (HighlightToken::Heading1, "#ff6b6b".into()),
                (HighlightToken::Heading2, "#ff6b6b".into()),
                (HighlightToken::Heading3, "#ffd93d".into()),
                (HighlightToken::Heading4, "#6bcfff".into()),
                (HighlightToken::Heading5, "#8be9fd".into()),
                (HighlightToken::Heading6, "#ffffff".into()),
                (HighlightToken::Bold, "#ff6b6b".into()),
                (HighlightToken::Italic, "#ff79c6".into()),
                (HighlightToken::BoldItalic, "#ff6b6b".into()),
                (HighlightToken::Strikethrough, "#ff6b6b".into()),
                (HighlightToken::CodeInline, "#50fa7b".into()),
                (HighlightToken::CodeBlock, "#ffffff".into()),
                (HighlightToken::Link, "#6bcfff".into()),
                (HighlightToken::LinkUrl, "#50fa7b".into()),
                (HighlightToken::LinkText, "#6bcfff".into()),
                (HighlightToken::Image, "#6bcfff".into()),
                (HighlightToken::ImageUrl, "#50fa7b".into()),
                (HighlightToken::ImageAlt, "#ff6b6b".into()),
                (HighlightToken::Blockquote, "#bfbfbf".into()),
                (HighlightToken::ListItem, "#ffffff".into()),
                (HighlightToken::ListMarker, "#ff6b6b".into()),
                (HighlightToken::HorizontalRule, "#bfbfbf".into()),
                (HighlightToken::TableHeader, "#ffd93d".into()),
                (HighlightToken::TableCell, "#ffffff".into()),
                (HighlightToken::TableBorder, "#bfbfbf".into()),
                (HighlightToken::WikiLink, "#6bcfff".into()),
                (HighlightToken::Frontmatter, "#bfbfbf".into()),
                (HighlightToken::Tag, "#8be9fd".into()),
                (HighlightToken::TaskMarker, "#ff6b6b".into()),
                (HighlightToken::Text, "#ffffff".into()),
                (HighlightToken::Whitespace, "#ffffff".into()),
                // Code tokens
                (HighlightToken::Keyword, "#ff79c6".into()),
                (HighlightToken::String, "#50fa7b".into()),
                (HighlightToken::Number, "#ffb86c".into()),
                (HighlightToken::Comment, "#bfbfbf".into()),
                (HighlightToken::Function, "#6bcfff".into()),
                (HighlightToken::Type, "#ffd93d".into()),
                (HighlightToken::Variable, "#ff6b6b".into()),
                (HighlightToken::Operator, "#8be9fd".into()),
                (HighlightToken::Property, "#ff6b6b".into()),
                (HighlightToken::Punctuation, "#ffffff".into()),
                (HighlightToken::Constant, "#ffd93d".into()),
                (HighlightToken::Attribute, "#ff6b6b".into()),
                (HighlightToken::CodeTag, "#ff6b6b".into()),
                (HighlightToken::Label, "#ff79c6".into()),
                (HighlightToken::Embedded, "#50fa7b".into()),
                (HighlightToken::Constructor, "#ffd93d".into()),
                (HighlightToken::Character, "#50fa7b".into()),
                (HighlightToken::Boolean, "#ffb86c".into()),
                (HighlightToken::Conditional, "#ff79c6".into()),
                (HighlightToken::Repeat, "#ff79c6".into()),
                (HighlightToken::Define, "#ff79c6".into()),
                (HighlightToken::Include, "#ff79c6".into()),
                (HighlightToken::FunctionBuiltin, "#ffd93d".into()),
                (HighlightToken::TypeBuiltin, "#ffd93d".into()),
                (HighlightToken::VariableBuiltin, "#ffd93d".into()),
                (HighlightToken::VariableParameter, "#ff6b6b".into()),
                (HighlightToken::StringEscape, "#8be9fd".into()),
                (HighlightToken::StringSpecial, "#8be9fd".into()),
                (HighlightToken::PunctuationBracket, "#ffffff".into()),
                (HighlightToken::PunctuationDelimiter, "#ffffff".into()),
            ],
        }
    }

    /// Get CSS color for a token. Returns `"inherit"` if not mapped.
    pub fn color(&self, token: &HighlightToken) -> &str {
        for (t, c) in &self.colors {
            if t == token {
                return c;
            }
        }
        "inherit"
    }

    /// Create a builder for custom themes.
    pub fn builder(name: &str) -> SyntaxThemeBuilder {
        SyntaxThemeBuilder {
            theme: Self {
                name: name.into(),
                colors: Vec::new(),
            },
        }
    }
}

/// Builder for constructing custom [`SyntaxTheme`]s.
#[derive(Debug, Clone)]
pub struct SyntaxThemeBuilder {
    theme: SyntaxTheme,
}

impl SyntaxThemeBuilder {
    /// Set the CSS color for a specific highlight token.
    pub fn color(mut self, token: HighlightToken, color: impl Into<String>) -> Self {
        self.theme.colors.push((token, color.into()));
        self
    }

    /// Build the theme.
    pub fn build(self) -> SyntaxTheme {
        self.theme
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dark_theme_returns_colors() {
        let t = SyntaxTheme::dark();
        assert_eq!(t.name(), "dark");
        assert_eq!(t.color(&HighlightToken::Keyword), "#c678dd");
        assert_eq!(t.color(&HighlightToken::String), "#98c379");
        assert_eq!(t.color(&HighlightToken::Comment), "#5c6370");
    }

    #[test]
    fn light_theme_returns_colors() {
        let t = SyntaxTheme::light();
        assert_eq!(t.name(), "light");
        assert_eq!(t.color(&HighlightToken::Keyword), "#a626a4");
        assert_eq!(t.color(&HighlightToken::String), "#50a14f");
    }

    #[test]
    fn high_contrast_theme_returns_colors() {
        let t = SyntaxTheme::high_contrast();
        assert_eq!(t.name(), "high_contrast");
        assert_eq!(t.color(&HighlightToken::Keyword), "#ff79c6");
        assert_eq!(t.color(&HighlightToken::String), "#50fa7b");
    }

    #[test]
    fn unmapped_token_returns_inherit() {
        let t = SyntaxTheme::builder("minimal").build();
        assert_eq!(t.color(&HighlightToken::Keyword), "inherit");
    }

    #[test]
    fn custom_builder_works() {
        let t = SyntaxTheme::builder("my theme")
            .color(HighlightToken::Keyword, "#ff0000")
            .color(HighlightToken::String, "#00ff00")
            .build();
        assert_eq!(t.name(), "my theme");
        assert_eq!(t.color(&HighlightToken::Keyword), "#ff0000");
        assert_eq!(t.color(&HighlightToken::String), "#00ff00");
        assert_eq!(t.color(&HighlightToken::Comment), "inherit");
    }

    #[test]
    fn markdown_tokens_have_colors_in_dark() {
        let t = SyntaxTheme::dark();
        assert_ne!(t.color(&HighlightToken::Heading1), "inherit");
        assert_ne!(t.color(&HighlightToken::Bold), "inherit");
        assert_ne!(t.color(&HighlightToken::CodeBlock), "inherit");
        assert_ne!(t.color(&HighlightToken::WikiLink), "inherit");
    }

    #[test]
    fn all_code_tokens_have_colors_in_dark() {
        let t = SyntaxTheme::dark();
        let code_tokens = [
            HighlightToken::Keyword,
            HighlightToken::String,
            HighlightToken::Number,
            HighlightToken::Comment,
            HighlightToken::Function,
            HighlightToken::Type,
            HighlightToken::Variable,
            HighlightToken::Operator,
            HighlightToken::Property,
            HighlightToken::Punctuation,
            HighlightToken::Constant,
            HighlightToken::Attribute,
            HighlightToken::CodeTag,
            HighlightToken::Label,
            HighlightToken::Embedded,
            HighlightToken::Constructor,
            HighlightToken::Character,
            HighlightToken::Boolean,
            HighlightToken::Conditional,
            HighlightToken::Repeat,
            HighlightToken::Define,
            HighlightToken::Include,
            HighlightToken::FunctionBuiltin,
            HighlightToken::TypeBuiltin,
            HighlightToken::VariableBuiltin,
            HighlightToken::VariableParameter,
            HighlightToken::StringEscape,
            HighlightToken::StringSpecial,
            HighlightToken::PunctuationBracket,
            HighlightToken::PunctuationDelimiter,
        ];
        for token in &code_tokens {
            assert_ne!(
                t.color(token),
                "inherit",
                "Dark theme missing color for {:?}",
                token
            );
        }
    }

    #[test]
    fn clone_and_debug_traits() {
        let t = SyntaxTheme::dark();
        let _cloned = t.clone();
        let _ = format!("{:?}", t);
    }
}
