//! Template engine with minijinja
//!
//! This module provides Jinja2-compatible templating capabilities using minijinja.

use crate::error::{RendererError, RendererResult};
use crate::types::TemplateContext;
use minijinja::Environment;
use parking_lot::Mutex;
use std::sync::Arc;
use tracing::{debug, trace};

/// Template engine for rendering templates
pub struct TemplateEngine {
    /// Minijinja environment
    env: Arc<Mutex<Environment<'static>>>,

    /// Auto-escape HTML
    auto_escape: bool,
}

impl TemplateEngine {
    /// Create a new template engine with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new template engine with auto-escape enabled
    pub fn with_auto_escape() -> Self {
        let env = Environment::new();
        Self {
            env: Arc::new(Mutex::new(env)),
            auto_escape: true,
        }
    }

    /// Add a template string
    pub fn add_template(&self, name: &str, template_str: &str) -> RendererResult<()> {
        let mut env = self.env.lock();
        let name_owned = name.to_owned();
        let template_str_owned = template_str.to_owned();
        env.add_template_owned(name_owned, template_str_owned)
            .map_err(|e| {
                RendererError::template_compile(format!("Failed to add template: {}", e))
            })?;
        debug!("Added template: {}", name);
        Ok(())
    }

    /// Render a template with context
    pub fn render(&self, name: &str, context: &TemplateContext) -> RendererResult<String> {
        let env = self.env.lock();
        let template = env
            .get_template(name)
            .map_err(|e| RendererError::template_compile(format!("Template not found: {}", e)))?;

        let name_owned = name.to_string();
        let result = template.render(context.as_map()).map_err(|e| {
            RendererError::template_render(name_owned, format!("Failed to render template: {}", e))
        })?;
        trace!("Rendered template: {}", name);
        Ok(result)
    }

    /// Set auto-escape
    pub fn set_auto_escape(&mut self, enabled: bool) {
        self.auto_escape = enabled;
    }

    /// Check if auto-escape is enabled
    pub fn auto_escape(&self) -> bool {
        self.auto_escape
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        let env = Environment::new();
        Self {
            env: Arc::new(Mutex::new(env)),
            auto_escape: false,
        }
    }
}

/// Template builder for creating template engines
pub struct TemplateBuilder {
    auto_escape: bool,
}

impl TemplateBuilder {
    /// Create a new template builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set auto-escape
    pub fn auto_escape(mut self, enabled: bool) -> Self {
        self.auto_escape = enabled;
        self
    }

    /// Build the template engine
    pub fn build(self) -> TemplateEngine {
        if self.auto_escape {
            TemplateEngine::with_auto_escape()
        } else {
            TemplateEngine::new()
        }
    }
}

impl Default for TemplateBuilder {
    fn default() -> Self {
        Self { auto_escape: false }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_template() {
        let engine = TemplateEngine::new();
        let result = engine.add_template("test", "Hello {{ name }}!");
        assert!(result.is_ok());
    }

    #[test]
    fn test_render_template() {
        let engine = TemplateEngine::new();
        let _ = engine.add_template("test", "Hello {{ name }}!");

        let mut context = TemplateContext::new();
        context.set("name".to_string(), serde_json::json!("World"));

        let result = engine.render("test", &context);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("World"));
    }

    #[test]
    fn test_template_builder() {
        let builder = TemplateBuilder::new().auto_escape(true);
        let engine = builder.build();
        assert!(engine.auto_escape());
    }
}
