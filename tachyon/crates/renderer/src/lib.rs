//! Tachyon Renderer Library
//!
//! This library provides document rendering capabilities for Tachyon
//! knowledge management system, including:
//!
//! - Markdown parsing with CommonMark and GFM support
//! - Template engine with Jinja2 compatibility
//! - LRU cache for rendered documents
//! - Syntax highlighting for multiple languages
//! - LaTeX equation rendering

// Public modules
pub mod cache;
pub mod context;
pub mod error;
pub mod latex;
pub mod markdown;
pub mod page;
pub mod sanitize;
pub mod syntax;
pub mod template;
pub mod types;

// Re-export commonly used types
pub use cache::RenderCache;
pub use error::{RendererError, RendererResult};
pub use latex::{LatexDocumentRenderer, LatexRenderer};
pub use markdown::MarkdownParser;
pub use page::SiteConfig;
pub use sanitize::sanitize_html;
pub use syntax::SyntaxHighlighter;
pub use template::TemplateEngine;
pub use types::{
    CacheConfig, CacheEntry, CacheKey, CacheStats, Language, MarkdownOptions, OutputFormat,
    RenderConfig, RenderMetadata, RenderResult, RenderStats,
};

use std::time::Instant;
use tracing::info;

/// Main renderer for documents
pub struct Renderer {
    /// Render configuration
    config: RenderConfig,
    /// Markdown parser
    markdown: MarkdownParser,
    /// Template engine
    template: TemplateEngine,
    /// Syntax highlighter
    syntax: SyntaxHighlighter,
    /// LaTeX renderer
    latex: LatexRenderer,
    /// Cache for rendered documents
    cache: RenderCache,
}

impl Renderer {
    /// Create a new renderer with specified configuration
    pub fn new(config: RenderConfig) -> Self {
        let template = TemplateEngine::new();
        Self {
            markdown: MarkdownParser::with_options(config.markdown.clone()),
            template,
            syntax: SyntaxHighlighter::new(),
            latex: LatexRenderer::with_cache(),
            cache: RenderCache::with_config(config.cache.clone()),
            config,
        }
    }

    /// Render a document
    pub fn render<S: AsRef<str>>(
        &self,
        content: S,
        _template_name: Option<&str>,
    ) -> RendererResult<RenderResult> {
        let content = content.as_ref();
        let start_time = Instant::now();
        info!("Rendering document with format: {:?}", self.config.format);

        // Parse markdown
        let render_result = self
            .markdown
            .parse(content, self.config.format)
            .map_err(|e| RendererError::markdown_parse(e.to_string()))?;

        let sanitized_content = sanitize_html(&render_result.content);
        let render_result = render_result.with_content(sanitized_content);

        // Update statistics
        let render_time = start_time.elapsed();
        let mut stats = render_result.stats.clone();
        stats.render_time_ms = render_time.as_millis() as u64;
        stats.output_size_bytes = render_result.content.len();

        let result = render_result.with_stats(stats);
        Ok(result)
    }

    /// Get render configuration
    pub fn config(&self) -> &RenderConfig {
        &self.config
    }

    /// Get markdown parser
    pub fn markdown(&self) -> &MarkdownParser {
        &self.markdown
    }

    /// Get template engine
    pub fn template(&self) -> &TemplateEngine {
        &self.template
    }

    /// Get syntax highlighter
    pub fn syntax(&self) -> &SyntaxHighlighter {
        &self.syntax
    }

    /// Get LaTeX renderer
    pub fn latex(&self) -> &LatexRenderer {
        &self.latex
    }

    /// Get cache
    pub fn cache(&self) -> &RenderCache {
        &self.cache
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new(RenderConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_simple_markdown() {
        let renderer = Renderer::new(RenderConfig::default());
        let result = renderer.render("# Hello World", None).unwrap();
        assert!(result.content.contains("Hello"));
    }
}
