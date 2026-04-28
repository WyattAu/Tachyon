//! Full-page HTML rendering for SEO and ISR (Incremental Static Regeneration)
//!
//! Wraps rendered document HTML into a complete HTML page with meta tags,
//! Open Graph, JSON-LD structured data, and navigation.

use crate::context::RenderContext;
use crate::template::TemplateEngine;
use crate::types::TemplateContext;

/// Site-level configuration for page rendering
#[derive(Debug, Clone)]
pub struct SiteConfig {
    /// Site title (e.g., "Tachyon")
    pub site_title: String,
    /// Site description
    pub site_description: String,
    /// Base URL for canonical URLs and OG tags (e.g., "https://tachyon.dev")
    pub base_url: String,
    /// Theme color for mobile browsers
    pub theme_color: String,
    /// OG image URL (default site-wide image)
    pub og_image: Option<String>,
    /// Custom template directory path (overrides defaults)
    pub template_dir: Option<String>,
}

impl Default for SiteConfig {
    fn default() -> Self {
        Self {
            site_title: "Tachyon".to_string(),
            site_description: "A deterministic, high-performance knowledge management system."
                .to_string(),
            base_url: "http://localhost:8080".to_string(),
            theme_color: "#2563eb".to_string(),
            og_image: None,
            template_dir: None,
        }
    }
}

/// Render a full HTML page from a RenderContext with SEO metadata.
///
/// Reserved for future use by the SSG / ISR pipeline.
///
/// Produces a complete `<!DOCTYPE html>` page with:
/// - Responsive viewport meta tag
/// - Open Graph tags for social sharing
/// - JSON-LD structured data (Article schema)
/// - Canonical URL
/// - Inline Tailwind CDN for styling
/// - Dark mode support via system preference
pub fn render_full_page(ctx: &RenderContext, site: &SiteConfig) -> String {
    let title = escape_html(&ctx.title);
    let description = escape_html(&extract_description(&ctx.content));
    let canonical_url = format!(
        "{}/docs/{}",
        site.base_url.trim_end_matches('/'),
        slugify(&ctx.title)
    );
    let og_image = site.og_image.as_deref().unwrap_or("").to_string();

    // Build breadcrumbs JSON-LD
    let breadcrumbs_json = ctx
        .navigation
        .as_ref()
        .map(|nav| {
            nav.breadcrumbs
                .iter()
                .enumerate()
                .map(|(i, b)| {
                    format!(
                        r#"{{"@type": "ListItem", "position": {}, "name": "{}", "item": "{}{}"}}"#,
                        i + 2, // offset by 2 for Home and Documents
                        escape_html(&b.title),
                        site.base_url.trim_end_matches('/'),
                        b.url,
                    )
                })
                .collect::<Vec<_>>()
                .join(",\n")
        })
        .unwrap_or_default();

    // Build tags JSON-LD
    let tags_json = ctx
        .metadata
        .as_ref()
        .map(|m| {
            m.tags
                .iter()
                .map(|t| format!(r#""{}""#, escape_html(t)))
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default();

    // Build author JSON-LD
    let author_json = ctx
        .author
        .as_ref()
        .map(|a| format!(r#""@type": "Person", "name": "{}""#, escape_html(&a.name),))
        .unwrap_or_else(|| r#""@type": "Organization", "name": "Tachyon""#.to_string());

    // Date for published time
    // Date for published time — pass through as-is if it looks like ISO 8601
    let published_time = ctx
        .metadata
        .as_ref()
        .map(|m| m.updated_at.clone())
        .unwrap_or_default();

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title} | {site_title}</title>
    <meta name="description" content="{description}">
    <meta name="theme-color" content="{theme_color}">
    <link rel="canonical" href="{canonical_url}">
    <meta property="og:type" content="article">
    <meta property="og:title" content="{title}">
    <meta property="og:description" content="{description}">
    <meta property="og:url" content="{canonical_url}">
    <meta property="og:site_name" content="{site_title}">
    <meta property="og:image" content="{og_image}">
    <meta property="og:image:width" content="1200">
    <meta property="og:image:height" content="630">
    <meta name="twitter:card" content="summary_large_image">
    <meta name="twitter:title" content="{title}">
    <meta name="twitter:description" content="{description}">
    <meta name="twitter:image" content="{og_image}">
    <link rel="icon" href="/favicon.svg" type="image/svg+xml">
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&family=JetBrains+Mono:wght@400;500&display=swap" rel="stylesheet">
    <script src="https://cdn.tailwindcss.com"><\/script>
    <script>
        tailwind.config = {{
            darkMode: 'class',
            theme: {{
                extend: {{
                    fontFamily: {{
                        sans: ['Inter', 'system-ui', '-apple-system', 'sans-serif'],
                        mono: ['JetBrains Mono', 'Consolas', 'monospace'],
                    }},
                }},
            }},
        }}
    <\/script>
    <style type="text/tailwindcss">
        @layer base {{
            html {{ -webkit-font-smoothing: antialiased; -moz-osx-font-smoothing: grayscale; }}
            body {{ @apply bg-gray-50 text-gray-900 dark:bg-gray-900 dark:text-gray-100; }}
            ::selection {{ @apply bg-blue-100 dark:bg-blue-900 text-blue-900 dark:text-blue-100; }}
        }}
    <\/style>
    <script type="application/ld+json">
    {{
        "@context": "https://schema.org",
        "@type": "Article",
        "headline": "{title}",
        "description": "{description}",
        "image": "{og_image}",
        "author": {author_json},
        "publisher": {{"@type": "Organization", "name": "{site_title}"}},
        "datePublished": "{published_time}",
        "mainEntityOfPage": {{
            "@type": "WebPage",
            "@id": "{canonical_url}"
        }}
    }}
    <\/script>
    <script type="application/ld+json">
    {{
        "@context": "https://schema.org",
        "@type": "BreadcrumbList",
        "itemListElement": [
            {{"@type": "ListItem", "position": 1, "name": "Home", "item": "{base_url}/"}},
            {{"@type": "ListItem", "position": 2, "name": "Documents", "item": "{base_url}/docs"}},
            {breadcrumbs_json}
        ]
    }}
    <\/script>
</head>
<body>
    <nav class="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700">
        <div class="max-w-6xl mx-auto px-6 py-3 flex items-center justify-between">
            <a href="/" class="text-lg font-semibold text-gray-900 dark:text-white hover:text-blue-600 dark:hover:text-blue-400">
                "{site_title}
            </a>
            <div class="flex items-center gap-4 text-sm text-gray-600 dark:text-gray-300">
                <a href="/docs" class="hover:text-gray-900 dark:hover:text-white">Docs</a>
                <a href="/search" class="hover:text-gray-900 dark:hover:text-white">Search</a>
            </div>
        </div>
    </nav>
    <main class="max-w-4xl mx-auto px-6 py-8">
        <article>
            <header class="mb-8">
                <h1 class="text-3xl font-bold text-gray-900 dark:text-white">{title}</h1>
                {tags_html}
            </header>
            <div class="prose prose-lg dark:prose-invert max-w-none">
                {content}
            </div>
        </article>
    </main>
    <footer class="border-t border-gray-200 dark:border-gray-700 mt-16">
        <div class="max-w-6xl mx-auto px-6 py-8 text-sm text-gray-500 dark:text-gray-400 text-center">
            Powered by <a href="/" class="hover:text-blue-600">{site_title}</a>
        </div>
    </footer>
    <script>
        // Apply dark mode from system preference
        if (window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) {{
            document.documentElement.classList.add('dark');
        }}
    <\/script>
</body>
</html>"#,
        title = title,
        description = description,
        canonical_url = canonical_url,
        og_image = og_image,
        theme_color = site.theme_color,
        site_title = site.site_title,
        base_url = site.base_url.trim_end_matches('/'),
        author_json = author_json,
        published_time = published_time,
        breadcrumbs_json = breadcrumbs_json,
        tags_html = if tags_json.is_empty() {
            String::new()
        } else {
            format!(
                r#"<div class="flex flex-wrap gap-2 mt-3">{}</div>"#,
                ctx.metadata
                    .as_ref()
                    .map(|m| {
                        m.tags
                            .iter()
                            .map(|t| {
                                format!(
                                    r#"<span class="inline-block px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200">{}</span>"#,
                                    escape_html(t),
                                )
                            })
                            .collect::<Vec<_>>()
                            .join("\n")
                    })
                    .unwrap_or_default()
            )
        },
        content = ctx.content.clone(),
    )
}

/// Render a full HTML page using a named template from the template engine.
///
/// Reserved for future use by the SSG pipeline.
pub fn render_full_page_with_template(
    ctx: &RenderContext,
    site: &SiteConfig,
    engine: &TemplateEngine,
    template_name: &str,
) -> crate::error::RendererResult<String> {
    let description = extract_description(&ctx.content);
    let canonical_url = format!(
        "{}/docs/{}",
        site.base_url.trim_end_matches('/'),
        slugify(&ctx.title)
    );
    let base_url = site.base_url.trim_end_matches('/').to_string();
    let og_image = site.og_image.as_deref().unwrap_or("").to_string();

    let breadcrumbs_json = ctx
        .navigation
        .as_ref()
        .map(|nav| {
            nav.breadcrumbs
                .iter()
                .enumerate()
                .map(|(i, b)| {
                    format!(
                        r#"{{"@type": "ListItem", "position": {}, "name": "{}", "item": "{}{}"}}"#,
                        i + 2,
                        escape_html(&b.title),
                        base_url,
                        b.url,
                    )
                })
                .collect::<Vec<_>>()
                .join(",\n")
        })
        .unwrap_or_default();

    let author_json = ctx
        .author
        .as_ref()
        .map(|a| format!(r#""@type": "Person", "name": "{}""#, escape_html(&a.name)))
        .unwrap_or_else(|| r#""@type": "Organization", "name": "Tachyon""#.to_string());

    let published_time = ctx
        .metadata
        .as_ref()
        .map(|m| m.updated_at.clone())
        .unwrap_or_default();

    let article_json_ld = format!(
        r#"{{
    "@context": "https://schema.org",
    "@type": "Article",
    "headline": "{}",
    "description": "{}",
    "image": "{}",
    "author": {},
    "publisher": {{"@type": "Organization", "name": "{}"}},
    "datePublished": "{}",
    "mainEntityOfPage": {{
        "@type": "WebPage",
        "@id": "{}"
    }}
}}"#,
        escape_html(&ctx.title),
        escape_html(&description),
        og_image,
        author_json,
        escape_html(&site.site_title),
        published_time,
        canonical_url,
    );

    let breadcrumb_json_ld = format!(
        r#"{{
    "@context": "https://schema.org",
    "@type": "BreadcrumbList",
    "itemListElement": [
        {{"@type": "ListItem", "position": 1, "name": "Home", "item": "{}/"}},
        {{"@type": "ListItem", "position": 2, "name": "Documents", "item": "{}/docs"}},
        {}
    ]
}}"#,
        base_url, base_url, breadcrumbs_json,
    );

    let tags_html = if ctx
        .metadata
        .as_ref()
        .map(|m| m.tags.is_empty())
        .unwrap_or(true)
    {
        String::new()
    } else {
        format!(
            r#"<div class="flex flex-wrap gap-2 mt-3">{}</div>"#,
            ctx.metadata
                .as_ref()
                .map(|m| {
                    m.tags
                        .iter()
                        .map(|t| {
                            format!(
                                r#"<span class="inline-block px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200">{}</span>"#,
                                escape_html(t),
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                })
                .unwrap_or_default()
        )
    };

    let mut template_ctx = TemplateContext::new();
    template_ctx.set("title".to_string(), ctx.title.clone());
    template_ctx.set("description".to_string(), description);
    template_ctx.set("canonical_url".to_string(), canonical_url);
    template_ctx.set("og_image".to_string(), og_image);
    template_ctx.set("theme_color".to_string(), site.theme_color.clone());
    template_ctx.set("site_title".to_string(), site.site_title.clone());
    template_ctx.set("base_url".to_string(), base_url);
    template_ctx.set("published_time".to_string(), published_time);
    template_ctx.set("tags_html".to_string(), tags_html);
    template_ctx.set("content".to_string(), ctx.content.clone());
    template_ctx.set("article_json_ld".to_string(), article_json_ld);
    template_ctx.set("breadcrumb_json_ld".to_string(), breadcrumb_json_ld);

    engine.render(template_name, &template_ctx)
}

/// Escape HTML special characters for safe embedding in attributes.
///
/// Reserved for future use by the rendering pipeline.
fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            _ => out.push(c),
        }
    }
    out
}

/// Extract a plain-text description from HTML content (first 160 chars).
///
/// Reserved for future use by the rendering pipeline.
fn extract_description(html: &str) -> String {
    // Strip all HTML tags
    let plain = strip_html_tags(html);
    let trimmed = plain.trim();
    if trimmed.len() <= 160 {
        trimmed.to_string()
    } else {
        // Find last space before 160 to avoid word break
        if let Some(pos) = trimmed[..160].rfind(' ') {
            format!("{}...", &trimmed[..pos])
        } else {
            format!("{}...", &trimmed[..157])
        }
    }
}

/// Naive HTML tag stripper for description extraction.
///
/// Reserved for future use by the rendering pipeline.
fn strip_html_tags(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ => {
                if !in_tag {
                    result.push(c);
                }
            }
        }
    }
    // Collapse whitespace
    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Convert a title to a URL-safe slug.
///
/// Reserved for future use by the rendering pipeline.
fn slugify(title: &str) -> String {
    let slug: String = title
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c.to_string()
            } else {
                "-".to_string()
            }
        })
        .collect();
    slug.split('-')
        .filter(|s: &&str| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_full_page_basic() {
        let ctx = RenderContext::new(
            "Test Document".to_string(),
            "<p>Hello world</p>".to_string(),
        );
        let site = SiteConfig::default();
        let html = render_full_page(&ctx, &site);

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<meta name=\"description\""));
        assert!(html.contains("<meta property=\"og:type\" content=\"article\""));
        assert!(html.contains("\"@type\": \"Article\""));
        assert!(html.contains("\"@type\": \"BreadcrumbList\""));
        assert!(html.contains("tailwindcss"));
    }

    #[test]
    fn test_render_full_page_with_author() {
        let mut ctx = RenderContext::new("Authored Doc".to_string(), "<p>Content</p>".to_string());
        ctx.author = Some(crate::context::AuthorInfo {
            name: "Test Author".to_string(),
            email: Some("test@example.com".to_string()),
            avatar_url: None,
        });
        ctx.metadata = Some(crate::context::RenderMetadata {
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-15T00:00:00Z".to_string(),
            tags: vec!["rust".to_string(), "testing".to_string()],
            read_time: Some(5),
        });

        let html = render_full_page(&ctx, &SiteConfig::default());
        assert!(html.contains("\"name\": \"Test Author\""));
        assert!(html.contains("rust"));
        assert!(html.contains("testing"));
    }

    #[test]
    fn test_escape_html() {
        assert_eq!(
            escape_html("<script>alert('xss')</script>"),
            "&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;"
        );
        assert_eq!(escape_html("Hello & <World>"), "Hello &amp; &lt;World&gt;");
    }

    #[test]
    fn test_extract_description() {
        assert_eq!(extract_description("<p>Hello world</p>"), "Hello world");
        assert_eq!(extract_description("<p>This is a longer description that exceeds the 160 character limit for meta descriptions.</p>"), "This is a longer description that exceeds the 160 character limit for meta descriptions.");
    }

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("  Multiple   Spaces  "), "multiple-spaces");
        assert_eq!(slugify("Rust & WASM"), "rust-wasm");
    }

    #[test]
    fn test_render_full_page_with_template() {
        use crate::template::TemplateEngine;

        let engine = TemplateEngine::with_defaults().unwrap();
        let ctx = RenderContext::new(
            "Test Document".to_string(),
            "<p>Hello world</p>".to_string(),
        );
        let site = SiteConfig::default();

        let result = render_full_page_with_template(&ctx, &site, &engine, "document.html");
        assert!(result.is_ok());
        let html = result.unwrap();
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<meta name=\"description\""));
        assert!(html.contains("Test Document"));
        assert!(html.contains("<p>Hello world</p>"));
        assert!(html.contains("\"@type\": \"Article\""));
        assert!(html.contains("\"@type\": \"BreadcrumbList\""));
        assert!(html.contains("tailwindcss"));
    }
}
