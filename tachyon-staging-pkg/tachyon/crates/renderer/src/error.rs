//! Renderer error types
//!
//! This module defines error types for the rendering engine, including errors
//! from markdown parsing, template rendering, caching, syntax highlighting,
//! and LaTeX rendering.

use thiserror::Error;

/// Main error type for the renderer crate
#[derive(Error, Debug)]
pub enum RendererError {
    /// Error during markdown parsing
    #[error("Markdown parsing error: {0}")]
    MarkdownParse(String),

    /// Error during markdown to HTML conversion
    #[error("Markdown to HTML conversion error: {0}")]
    MarkdownToHtml(String),

    /// Error during template rendering
    #[error("Template rendering error: {message}")]
    TemplateRender {
        /// Name of the template
        template_name: String,
        /// Error message
        message: String,
    },

    /// Template not found
    #[error("Template not found: {0}")]
    TemplateNotFound(String),

    /// Template compilation error
    #[error("Template compilation error: {0}")]
    TemplateCompile(String),

    /// Template syntax error
    #[error("Template syntax error at line {line}, column {column}: {message}")]
    TemplateSyntax {
        /// Line number
        line: usize,
        /// Column number
        column: usize,
        /// Error message
        message: String,
    },

    /// Cache error
    #[error("Cache error: {0}")]
    Cache(String),

    /// Cache capacity exceeded
    #[error("Cache capacity exceeded: {0} items limit reached")]
    CacheCapacityExceeded(usize),

    /// Cache entry expired
    #[error("Cache entry expired")]
    CacheExpired,

    /// Syntax highlighting error
    #[error("Syntax highlighting error: {0}")]
    SyntaxHighlight(String),

    /// Unsupported language for syntax highlighting
    #[error("Unsupported language for syntax highlighting: {0}")]
    UnsupportedLanguage(String),

    /// Language parser not available
    #[error("Language parser not available for: {0}")]
    LanguageParserUnavailable(String),

    /// LaTeX rendering error
    #[error("LaTeX rendering error: {0}")]
    LatexRender(String),

    /// Invalid LaTeX syntax
    #[error("Invalid LaTeX syntax: {0}")]
    InvalidLatex(String),

    /// Unknown LaTeX command
    #[error("Unknown LaTeX command: {0}")]
    UnknownLatexCommand(String),

    /// Input/output error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Deserialization error
    #[error("Deserialization error: {0}")]
    Deserialization(String),

    /// Invalid input
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Timeout
    #[error("Operation timed out after {0}ms")]
    Timeout(u64),

    /// Internal error
    #[error("Internal renderer error: {0}")]
    Internal(String),
}

impl RendererError {
    /// Create a new markdown parsing error
    pub fn markdown_parse<S: Into<String>>(message: S) -> Self {
        RendererError::MarkdownParse(message.into())
    }

    /// Create a new markdown to HTML conversion error
    pub fn markdown_to_html<S: Into<String>>(message: S) -> Self {
        RendererError::MarkdownToHtml(message.into())
    }

    /// Create a new template rendering error
    pub fn template_render<S: Into<String>>(template_name: S, message: S) -> Self {
        RendererError::TemplateRender {
            template_name: template_name.into(),
            message: message.into(),
        }
    }

    /// Create a new template not found error
    pub fn template_not_found<S: Into<String>>(template_name: S) -> Self {
        RendererError::TemplateNotFound(template_name.into())
    }

    /// Create a new template compilation error
    pub fn template_compile<S: Into<String>>(message: S) -> Self {
        RendererError::TemplateCompile(message.into())
    }

    /// Create a new template syntax error
    pub fn template_syntax<S: Into<String>>(line: usize, column: usize, message: S) -> Self {
        RendererError::TemplateSyntax {
            line,
            column,
            message: message.into(),
        }
    }

    /// Create a new cache error
    pub fn cache<S: Into<String>>(message: S) -> Self {
        RendererError::Cache(message.into())
    }

    /// Create a new cache capacity exceeded error
    pub fn cache_capacity_exceeded(capacity: usize) -> Self {
        RendererError::CacheCapacityExceeded(capacity)
    }

    /// Create a new syntax highlighting error
    pub fn syntax_highlight<S: Into<String>>(message: S) -> Self {
        RendererError::SyntaxHighlight(message.into())
    }

    /// Create a new unsupported language error
    pub fn unsupported_language<S: Into<String>>(language: S) -> Self {
        RendererError::UnsupportedLanguage(language.into())
    }

    /// Create a new language parser unavailable error
    pub fn language_parser_unavailable<S: Into<String>>(language: S) -> Self {
        RendererError::LanguageParserUnavailable(language.into())
    }

    /// Create a new LaTeX rendering error
    pub fn latex_render<S: Into<String>>(message: S) -> Self {
        RendererError::LatexRender(message.into())
    }

    /// Create a new invalid LaTeX error
    pub fn invalid_latex<S: Into<String>>(message: S) -> Self {
        RendererError::InvalidLatex(message.into())
    }

    /// Create a new unknown LaTeX command error
    pub fn unknown_latex_command<S: Into<String>>(command: S) -> Self {
        RendererError::UnknownLatexCommand(command.into())
    }

    /// Create a new serialization error
    pub fn serialization<S: Into<String>>(message: S) -> Self {
        RendererError::Serialization(message.into())
    }

    /// Create a new deserialization error
    pub fn deserialization<S: Into<String>>(message: S) -> Self {
        RendererError::Deserialization(message.into())
    }

    /// Create a new invalid input error
    pub fn invalid_input<S: Into<String>>(message: S) -> Self {
        RendererError::InvalidInput(message.into())
    }

    /// Create a new timeout error
    pub fn timeout(duration_ms: u64) -> Self {
        RendererError::Timeout(duration_ms)
    }

    /// Create a new internal error
    pub fn internal<S: Into<String>>(message: S) -> Self {
        RendererError::Internal(message.into())
    }
}

impl From<serde_json::Error> for RendererError {
    fn from(err: serde_json::Error) -> Self {
        RendererError::Serialization(err.to_string())
    }
}

impl From<minijinja::Error> for RendererError {
    fn from(err: minijinja::Error) -> Self {
        match err.kind() {
            minijinja::ErrorKind::SyntaxError => RendererError::TemplateSyntax {
                line: 0,
                column: 0,
                message: err.to_string(),
            },
            minijinja::ErrorKind::TemplateNotFound => {
                RendererError::TemplateNotFound(err.to_string())
            }
            _ => RendererError::TemplateRender {
                template_name: "unknown".to_string(),
                message: err.to_string(),
            },
        }
    }
}

/// Result type alias for renderer operations
pub type RendererResult<T> = Result<T, RendererError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = RendererError::markdown_parse("Invalid markdown");
        assert_eq!(err.to_string(), "Markdown parsing error: Invalid markdown");
    }

    #[test]
    fn test_template_syntax_error() {
        let err = RendererError::template_syntax(10, 5, "Unexpected token");
        assert!(err.to_string().contains("line 10"));
        assert!(err.to_string().contains("column 5"));
    }

    #[test]
    fn test_from_minijinja_error() {
        let err: RendererError =
            minijinja::Error::new(minijinja::ErrorKind::TemplateNotFound, "template.html").into();
        assert!(matches!(err, RendererError::TemplateNotFound(_)));
    }

    #[test]
    fn test_from_pulldown_error() {
        // pulldown-cmark 0.9 doesn't have a public Error type
        // Test the markdown parse error variant directly
        let err = RendererError::MarkdownParse("Test parse error".to_string());
        assert!(matches!(err, RendererError::MarkdownParse(_)));
    }
}
