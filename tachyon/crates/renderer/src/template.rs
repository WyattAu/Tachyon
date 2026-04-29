//! Template engine with minijinja
//!
//! This module provides Jinja2-compatible templating capabilities using minijinja.

use crate::error::{RendererError, RendererResult};
use crate::types::TemplateContext;
use minijinja::Environment;
use parking_lot::Mutex;
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, trace};

pub(crate) const DEFAULT_BASE_TEMPLATE: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{ title }} | {{ site_title }}</title>
    <meta name="description" content="{{ description }}">
    <meta name="theme-color" content="{{ theme_color }}">
    <link rel="canonical" href="{{ canonical_url }}">
    <meta property="og:type" content="article">
    <meta property="og:title" content="{{ title }}">
    <meta property="og:description" content="{{ description }}">
    <meta property="og:url" content="{{ canonical_url }}">
    <meta property="og:site_name" content="{{ site_title }}">
    <meta property="og:image" content="{{ og_image }}">
    <meta property="og:image:width" content="1200">
    <meta property="og:image:height" content="630">
    <meta name="twitter:card" content="summary_large_image">
    <meta name="twitter:title" content="{{ title }}">
    <meta name="twitter:description" content="{{ description }}">
    <meta name="twitter:image" content="{{ og_image }}">
    <link rel="icon" href="/favicon.svg" type="image/svg+xml">
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&family=JetBrains+Mono:wght@400;500&display=swap" rel="stylesheet">
    <script src="https://cdn.tailwindcss.com"></script>
    <script>
        tailwind.config = {
            darkMode: 'class',
            theme: {
                extend: {
                    fontFamily: {
                        sans: ['Inter', 'system-ui', '-apple-system', 'sans-serif'],
                        mono: ['JetBrains Mono', 'Consolas', 'monospace'],
                    },
                },
            },
        }
    </script>
    <style type="text/tailwindcss">
        @layer base {
            html { -webkit-font-smoothing: antialiased; -moz-osx-font-smoothing: grayscale; }
            body { @apply bg-gray-50 text-gray-900 dark:bg-gray-900 dark:text-gray-100; }
            ::selection { @apply bg-blue-100 dark:bg-blue-900 text-blue-900 dark:text-blue-100; }
        }
    </style>
    <script type="application/ld+json">{{ article_json_ld | safe }}</script>
    <script type="application/ld+json">{{ breadcrumb_json_ld | safe }}</script>
    {% block extra_head %}{% endblock %}
</head>
<body>
    <nav class="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700">
        <div class="max-w-6xl mx-auto px-6 py-3 flex items-center justify-between">
            <a href="/" class="text-lg font-semibold text-gray-900 dark:text-white hover:text-blue-600 dark:hover:text-blue-400">
                {{ site_title }}
            </a>
            <div class="flex items-center gap-4 text-sm text-gray-600 dark:text-gray-300">
                <a href="/docs" class="hover:text-gray-900 dark:hover:text-white">Docs</a>
                <a href="/search" class="hover:text-gray-900 dark:hover:text-white">Search</a>
            </div>
        </div>
    </nav>
    {% block body %}
    <main class="max-w-4xl mx-auto px-6 py-8">
        <article>
            <header class="mb-8">
                <h1 class="text-3xl font-bold text-gray-900 dark:text-white">{{ title }}</h1>
                {{ tags_html | safe }}
            </header>
            <div class="prose prose-lg dark:prose-invert max-w-none">
                {{ content | safe }}
            </div>
        </article>
    </main>
    {% endblock %}
    <footer class="border-t border-gray-200 dark:border-gray-700 mt-16">
        <div class="max-w-6xl mx-auto px-6 py-8 text-sm text-gray-500 dark:text-gray-400 text-center">
            Powered by <a href="/" class="hover:text-blue-600">{{ site_title }}</a>
        </div>
    </footer>
    <script>
        if (window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) {
            document.documentElement.classList.add('dark');
        }
    </script>
</body>
</html>"#;

pub(crate) const DEFAULT_DOCUMENT_TEMPLATE: &str = r#"{% extends "base.html" %}
{% block body %}
<main class="max-w-4xl mx-auto px-6 py-8">
    <article>
        <header class="mb-8">
            <h1 class="text-3xl font-bold text-gray-900 dark:text-white">{{ title }}</h1>
            {{ tags_html | safe }}
        </header>
        <div class="prose prose-lg dark:prose-invert max-w-none">
            {{ content | safe }}
        </div>
    </article>
</main>
{% endblock %}"#;

pub(crate) const DEFAULT_INDEX_TEMPLATE: &str = r#"{% extends "base.html" %}
{% block body %}
<main class="max-w-4xl mx-auto px-6 py-8">
    <h1 class="text-3xl font-bold text-gray-900 dark:text-white mb-8">Documents</h1>
    <div id="search-container" class="mb-6">
        <input type="text" id="search" placeholder="Search documents..."
            class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100">
    </div>
    <div id="document-list">
        {{ content | safe }}
    </div>
</main>
{% endblock %}"#;

pub struct TemplateEngine {
    env: Arc<Mutex<Environment<'static>>>,
    auto_escape: bool,
}

impl TemplateEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_auto_escape() -> Self {
        let env = Environment::new();
        Self {
            env: Arc::new(Mutex::new(env)),
            auto_escape: true,
        }
    }

    pub fn from_directory(path: &Path) -> RendererResult<Self> {
        let mut env = Environment::new();
        env.set_loader(minijinja::path_loader(path));
        env.set_auto_escape_callback(|name| {
            if name.ends_with(".html") {
                minijinja::AutoEscape::Html
            } else {
                minijinja::AutoEscape::None
            }
        });
        Ok(Self {
            env: Arc::new(Mutex::new(env)),
            auto_escape: true,
        })
    }

    pub fn with_defaults() -> RendererResult<Self> {
        let mut env = Environment::new();
        env.set_auto_escape_callback(|name| {
            if name.ends_with(".html") {
                minijinja::AutoEscape::Html
            } else {
                minijinja::AutoEscape::None
            }
        });
        env.add_template_owned("base.html".to_owned(), DEFAULT_BASE_TEMPLATE.to_owned())
            .map_err(|e| {
                RendererError::template_compile(format!("Failed to add default template: {}", e))
            })?;
        env.add_template_owned(
            "document.html".to_owned(),
            DEFAULT_DOCUMENT_TEMPLATE.to_owned(),
        )
        .map_err(|e| {
            RendererError::template_compile(format!("Failed to add default template: {}", e))
        })?;
        env.add_template_owned("index.html".to_owned(), DEFAULT_INDEX_TEMPLATE.to_owned())
            .map_err(|e| {
                RendererError::template_compile(format!("Failed to add default template: {}", e))
            })?;
        debug!("Loaded default templates: base.html, document.html, index.html");
        Ok(Self {
            env: Arc::new(Mutex::new(env)),
            auto_escape: true,
        })
    }

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

    pub fn has_template(&self, name: &str) -> bool {
        let env = self.env.lock();
        env.get_template(name).is_ok()
    }

    pub fn set_auto_escape(&mut self, enabled: bool) {
        self.auto_escape = enabled;
    }

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

#[derive(Default)]
pub struct TemplateBuilder {
    auto_escape: bool,
}

impl TemplateBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn auto_escape(mut self, enabled: bool) -> Self {
        self.auto_escape = enabled;
        self
    }

    pub fn build(self) -> TemplateEngine {
        if self.auto_escape {
            TemplateEngine::with_auto_escape()
        } else {
            TemplateEngine::new()
        }
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

    #[test]
    fn test_from_directory() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("base.html"),
            "<!DOCTYPE html><html><body>{{ content | safe }}</body></html>",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("custom.html"),
            "<h1>{{ title }}</h1><p>{{ content | safe }}</p>",
        )
        .unwrap();

        let engine = TemplateEngine::from_directory(dir.path()).unwrap();
        assert!(engine.has_template("base.html"));
        assert!(engine.has_template("custom.html"));
        assert!(!engine.has_template("nonexistent.html"));

        let mut ctx = TemplateContext::new();
        ctx.set("title".to_string(), "Hello");
        ctx.set("content".to_string(), "<p>World</p>");
        let result = engine.render("custom.html", &ctx).unwrap();
        assert!(result.contains("<h1>Hello</h1>"));
        assert!(result.contains("<p>World</p>"));
    }

    #[test]
    fn test_with_defaults() {
        let engine = TemplateEngine::with_defaults().unwrap();
        assert!(engine.has_template("base.html"));
        assert!(engine.has_template("document.html"));
        assert!(engine.has_template("index.html"));
        assert!(engine.auto_escape());
    }

    #[test]
    fn test_has_template() {
        let engine = TemplateEngine::new();
        assert!(!engine.has_template("missing.html"));
        engine.add_template("exists.html", "content").unwrap();
        assert!(engine.has_template("exists.html"));
    }

    #[test]
    fn test_render_default_document_template() {
        let engine = TemplateEngine::with_defaults().unwrap();

        let mut ctx = TemplateContext::new();
        ctx.set("title".to_string(), "Test Doc");
        ctx.set("description".to_string(), "A test document");
        ctx.set(
            "canonical_url".to_string(),
            "https://example.com/docs/test-doc",
        );
        ctx.set("og_image".to_string(), "");
        ctx.set("theme_color".to_string(), "#2563eb");
        ctx.set("site_title".to_string(), "Test Site");
        ctx.set("base_url".to_string(), "https://example.com");
        ctx.set("published_time".to_string(), "2026-04-09T00:00:00Z");
        ctx.set("tags_html".to_string(), "");
        ctx.set("content".to_string(), "<p>Hello world</p>");
        ctx.set(
            "article_json_ld".to_string(),
            r#"{"@type": "Article", "headline": "Test Doc"}"#.to_string(),
        );
        ctx.set(
            "breadcrumb_json_ld".to_string(),
            r#"{"@type": "BreadcrumbList"}"#.to_string(),
        );

        let result = engine.render("document.html", &ctx).unwrap();
        assert!(result.contains("<!DOCTYPE html>"));
        assert!(result.contains("Test Doc"));
        assert!(result.contains("<p>Hello world</p>"));
        assert!(result.contains("Test Site"));
        assert!(result.contains("article"));
    }
}
