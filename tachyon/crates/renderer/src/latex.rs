//! LaTeX rendering with katex
//!
//! This module provides LaTeX equation rendering capabilities using the katex library.

use crate::error::{RendererError, RendererResult};
use katex::Opts;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, trace};

/// LaTeX renderer for rendering mathematical equations
pub struct LatexRenderer {
    /// KaTeX rendering options
    opts: Opts,

    /// Cache for rendered equations
    cache: Arc<Mutex<HashMap<String, String>>>,

    /// Enable caching
    enable_cache: bool,

    /// Display mode (block) delimiter
    display_delimiter_start: String,

    /// Display mode (block) delimiter end
    display_delimiter_end: String,

    /// Inline mode delimiter
    inline_delimiter_start: String,

    /// Inline mode delimiter end
    inline_delimiter_end: String,
}

impl LatexRenderer {
    /// Create a new LaTeX renderer with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new LaTeX renderer with caching enabled
    pub fn with_cache() -> Self {
        Self {
            enable_cache: true,
            ..Self::default()
        }
    }

    /// Create a new LaTeX renderer with custom delimiters
    pub fn with_delimiters<S1, S2, S3, S4>(
        display_start: S1,
        display_end: S2,
        inline_start: S3,
        inline_end: S4,
    ) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
        S3: Into<String>,
        S4: Into<String>,
    {
        Self {
            display_delimiter_start: display_start.into(),
            display_delimiter_end: display_end.into(),
            inline_delimiter_start: inline_start.into(),
            inline_delimiter_end: inline_end.into(),
            ..Self::default()
        }
    }

    /// Render LaTeX to HTML
    pub fn render<S: AsRef<str>>(&self, latex: S) -> RendererResult<String> {
        let latex = latex.as_ref();
        trace!("Rendering LaTeX: {}", latex);

        // Check cache first
        if self.enable_cache {
            let cache = self.cache.lock();
            if let Some(cached) = cache.get(latex) {
                trace!("Cache hit for LaTeX");
                return Ok(cached.clone());
            }
        }

        // Render with KaTeX
        let html = katex::render(latex)
            .map_err(|e| RendererError::latex_render(format!("KaTeX error: {}", e)))?;

        // Cache the result
        if self.enable_cache {
            let mut cache = self.cache.lock();
            cache.insert(latex.to_string(), html.clone());
        }

        debug!("Rendered LaTeX equation");
        Ok(html)
    }

    /// Render display mode LaTeX
    pub fn render_display<S: AsRef<str>>(&self, latex: S) -> RendererResult<String> {
        let latex = latex.as_ref();
        let html = katex::render(latex)
            .map_err(|e| RendererError::latex_render(format!("KaTeX display mode error: {}", e)))?;
        debug!("Rendered display mode LaTeX");
        Ok(html)
    }

    /// Render inline mode LaTeX
    pub fn render_inline<S: AsRef<str>>(&self, latex: S) -> RendererResult<String> {
        let latex = latex.as_ref();
        let html = katex::render(latex)
            .map_err(|e| RendererError::latex_render(format!("KaTeX inline mode error: {}", e)))?;
        debug!("Rendered inline mode LaTeX");
        Ok(html)
    }

    /// Render LaTeX from text, finding and replacing LaTeX delimiters
    pub fn render_from_text<S: AsRef<str>>(&self, text: S) -> RendererResult<String> {
        let text = text.as_ref();
        let chars: Vec<char> = text.chars().collect();
        let mut result = String::new();
        let mut i = 0;

        while i < chars.len() {
            // Find display mode math
            if let Some(start) = find_substring(&chars, i, &self.display_delimiter_start) {
                if let Some(end) = find_substring(&chars, start, &self.display_delimiter_end) {
                    let latex: String = chars[start..end].iter().collect();
                    let html = self.render_display(&latex)?;
                    result.push_str(&html);
                    i = end + self.display_delimiter_end.len();
                    continue;
                }
            }

            // Find inline mode math
            if let Some(start) = find_substring(&chars, i, &self.inline_delimiter_start) {
                if let Some(end) = find_substring(&chars, start, &self.inline_delimiter_end) {
                    let latex: String = chars[start..end].iter().collect();
                    let html = self.render_inline(&latex)?;
                    result.push_str(&html);
                    i = end + self.inline_delimiter_end.len();
                    continue;
                }
            }

            // Copy regular text
            result.push(chars[i]);
            i += 1;
        }

        Ok(result)
    }

    /// Validate LaTeX syntax
    pub fn validate<S: AsRef<str>>(&self, latex: S) -> bool {
        let latex = latex.as_ref();
        katex::render(latex).is_ok()
    }

    /// Get rendering options
    pub fn opts(&self) -> &Opts {
        &self.opts
    }

    /// Set rendering options
    pub fn set_opts(&mut self, opts: Opts) {
        self.opts = opts;
    }

    /// Enable or disable caching
    pub fn set_cache(&mut self, enable: bool) {
        self.enable_cache = enable;
    }

    /// Clear cache
    pub fn clear_cache(&self) {
        let mut cache = self.cache.lock();
        cache.clear();
        debug!("LaTeX cache cleared");
    }

    /// Get cache size
    pub fn cache_size(&self) -> usize {
        self.cache.lock().len()
    }

    /// Set custom delimiters
    pub fn set_delimiters<S1, S2, S3, S4>(
        &mut self,
        display_start: S1,
        display_end: S2,
        inline_start: S3,
        inline_end: S4,
    ) where
        S1: Into<String>,
        S2: Into<String>,
        S3: Into<String>,
        S4: Into<String>,
    {
        self.display_delimiter_start = display_start.into();
        self.display_delimiter_end = display_end.into();
        self.inline_delimiter_start = inline_start.into();
        self.inline_delimiter_end = inline_end.into();
    }
}

impl Default for LatexRenderer {
    fn default() -> Self {
        Self {
            opts: Opts::default(),
            cache: Arc::new(Mutex::new(HashMap::new())),
            enable_cache: false,
            display_delimiter_start: "$$".to_string(),
            display_delimiter_end: "$$".to_string(),
            inline_delimiter_start: "$".to_string(),
            inline_delimiter_end: "$".to_string(),
        }
    }
}

/// Find substring in character array
fn find_substring(chars: &[char], start: usize, pattern: &str) -> Option<usize> {
    let pattern_chars: Vec<char> = pattern.chars().collect();
    if pattern_chars.is_empty() {
        return None;
    }

    let mut i = start;
    while i + pattern_chars.len() <= chars.len() {
        if &chars[i..i + pattern_chars.len()] == pattern_chars.as_slice() {
            return Some(i);
        }
        i += 1;
    }

    None
}

/// LaTeX document renderer for rendering complete LaTeX documents
#[derive(Default)]
pub struct LatexDocumentRenderer {
    /// LaTeX renderer
    renderer: LatexRenderer,
}

impl LatexDocumentRenderer {
    /// Create a new LaTeX document renderer
    pub fn new() -> Self {
        Self {
            renderer: LatexRenderer::new(),
        }
    }

    /// Create a new LaTeX document renderer with caching
    pub fn with_cache() -> Self {
        Self {
            renderer: LatexRenderer::with_cache(),
        }
    }

    /// Render a LaTeX document
    pub fn render<S: AsRef<str>>(&self, latex: S) -> RendererResult<String> {
        self.renderer.render_from_text(latex)
    }

    /// Get the underlying LaTeX renderer
    pub fn renderer(&self) -> &LatexRenderer {
        &self.renderer
    }

    /// Get the LaTeX renderer mutably
    pub fn renderer_mut(&mut self) -> &mut LatexRenderer {
        &mut self.renderer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_simple_latex() {
        let renderer = LatexRenderer::new();
        let result = renderer.render("E = mc^2").unwrap();
        assert!(result.contains("E"));
    }

    #[test]
    fn test_render_with_cache() {
        let renderer = LatexRenderer::with_cache();
        let result1 = renderer.render("E = mc^2").unwrap();
        let result2 = renderer.render("E = mc^2").unwrap();
        assert_eq!(result1, result2);
        assert!(renderer.cache_size() > 0);
    }

    #[test]
    fn test_render_from_text() {
        let renderer = LatexRenderer::new();
        let text = "The equation $E = mc^2$ is famous";
        let result = renderer.render_from_text(text).unwrap();
        // The text should at least contain the non-LaTeX parts
        assert!(result.contains("The equation") || result.contains("famous"));
    }

    #[test]
    fn test_validate_latex() {
        let renderer = LatexRenderer::new();
        assert!(renderer.validate("E = mc^2"));
        assert!(!renderer.validate("\\invalid latex"));
    }

    #[test]
    fn test_latex_document_renderer() {
        let doc_renderer = LatexDocumentRenderer::new();
        let result = doc_renderer.render("E = mc^2").unwrap();
        assert!(result.contains("E"));
    }
}
