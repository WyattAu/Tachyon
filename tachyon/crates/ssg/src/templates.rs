//! HTML templates for the SSG engine.
//!
//! Uses string interpolation instead of a template engine to avoid
//! additional dependencies. Templates are designed to be:
//! - Clean and modern (inspired by Docusaurus / VitePress)
//! - Responsive with Tailwind CSS CDN
//! - Dark mode support via `class` strategy on `<html>`
//! - Customizable via ColorTheme CSS custom properties
//! - Accessible (semantic HTML, ARIA labels)

use crate::manifest::{ColorTheme, SiteConfig};
use crate::render::{CategoryContext, DocCard, IndexContext, PageContext, TocEntry};

/// Generate CSS custom properties from a ColorTheme.
///
/// When a custom theme is provided, these override the default hardcoded values.
/// When no theme is set, returns an empty string (defaults in CSS take effect).
fn color_theme_css(theme: Option<&ColorTheme>) -> String {
    let Some(t) = theme else {
        return String::new();
    };

    let font_body = t
        .font_family
        .as_deref()
        .unwrap_or("ui-sans-serif, system-ui, -apple-system, sans-serif");
    let font_heading = t.heading_font_family.as_deref().unwrap_or(font_body);

    format!(
        r#"
    :root {{
      --tachyon-primary: {primary};
      --tachyon-secondary: {secondary};
      --tachyon-accent: {accent};
      --tachyon-code-bg: {code_bg};
      --tachyon-font-body: {font_body};
      --tachyon-font-heading: {font_heading};
    }}"#,
        primary = t.primary,
        secondary = t.secondary,
        accent = t.accent,
        code_bg = t.code_bg,
        font_body = font_body,
        font_heading = font_heading,
    )
}

/// Render a document page.
pub fn render_doc_page(ctx: &PageContext) -> String {
    let nav_html = render_nav(
        ctx.site,
        ctx.current_slug,
        ctx.language,
        ctx.language_switcher,
    );
    let tags_html = render_tags(ctx.tags);
    let theme_class = match ctx.site.theme.as_str() {
        "dark" => "dark",
        "light" => "",
        _ => "", // auto handled by CSS media query
    };
    let dir = crate::i18n::text_direction(ctx.language);

    let author_html = match ctx.author {
        Some(author) => format!(
            r#"<span class="text-sm text-gray-500 dark:text-gray-400">by {}</span>"#,
            author
        ),
        None => String::new(),
    };

    let updated_html = if ctx.site.show_updated_at {
        format!(
            r#"<span class="text-sm text-gray-500 dark:text-gray-400">Updated {}</span>"#,
            ctx.updated_at
        )
    } else {
        String::new()
    };

    let theme_vars = color_theme_css(ctx.site.color_theme.as_ref());

    let breadcrumbs_html = render_breadcrumbs(ctx.breadcrumbs);
    let toc_html = render_toc_sidebar(ctx.toc);
    let prev_next_html = render_prev_next(ctx.prev_link, ctx.next_link);

    format!(
        r#"<!DOCTYPE html>
<html lang="{language}" dir="{dir}" class="{theme_class}">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{title} — {site_title}</title>
  <meta name="description" content="{description}">
  <link rel="canonical" href="{page_url}">
  <meta property="og:title" content="{title}">
  <meta property="og:description" content="{description}">
  <meta property="og:url" content="{page_url}">
  <meta property="og:type" content="article">
  <meta name="twitter:card" content="summary">
  <meta name="twitter:title" content="{title}">
  <meta name="twitter:description" content="{description}">
  {favicon}
  {json_ld}
  <script src="https://cdn.tailwindcss.com"></script>
  <style>
    :root {{
      --tachyon-primary: #2563eb;
      --tachyon-secondary: #7c3aed;
      --tachyon-accent: #06b6d4;
      --tachyon-code-bg: #1f2937;
      --tachyon-font-body: ui-sans-serif, system-ui, -apple-system, sans-serif;
      --tachyon-font-heading: ui-sans-serif, system-ui, -apple-system, sans-serif;
    }}
{theme_vars}
    body {{ font-family: var(--tachyon-font-body); }}
    h1, h2, h3, h4, h5, h6 {{ font-family: var(--tachyon-font-heading); }}
    {{ prosemirror-styles }}
    .doc-content {{ max-width: 48rem; margin: 0 auto; padding: 2rem 1rem; }}
    .doc-content h1 {{ font-size: 2.25rem; font-weight: 700; margin-bottom: 1rem; border-bottom: 1px solid #e5e7eb; padding-bottom: 0.5rem; color: #111827; }}
    .doc-content h2 {{ font-size: 1.5rem; font-weight: 600; margin-top: 2rem; margin-bottom: 0.75rem; color: #1f2937; }}
    .doc-content h3 {{ font-size: 1.25rem; font-weight: 600; margin-top: 1.5rem; margin-bottom: 0.5rem; color: #374151; }}
    .doc-content p {{ margin-bottom: 1rem; line-height: 1.75; color: #4b5563; }}
    .doc-content ul, .doc-content ol {{ margin-bottom: 1rem; padding-left: 1.5rem; color: #4b5563; }}
    .doc-content li {{ margin-bottom: 0.25rem; line-height: 1.75; }}
    .doc-content code {{ background: #f3f4f6; padding: 0.125rem 0.375rem; border-radius: 0.25rem; font-size: 0.875rem; color: #dc2626; }}
    .doc-content pre {{ background: var(--tachyon-code-bg); color: #f9fafb; padding: 1rem; border-radius: 0.5rem; overflow-x: auto; margin-bottom: 1rem; }}
    .doc-content pre code {{ background: none; color: inherit; padding: 0; }}
    .doc-content blockquote {{ border-left: 4px solid var(--tachyon-primary); padding-left: 1rem; margin: 1rem 0; color: #6b7280; font-style: italic; }}
    .doc-content a {{ color: var(--tachyon-primary); text-decoration: underline; }}
    .doc-content a:hover {{ color: var(--tachyon-secondary); }}
    .doc-content img {{ max-width: 100%; border-radius: 0.5rem; margin: 1rem 0; }}
    .doc-content table {{ width: 100%; border-collapse: collapse; margin-bottom: 1rem; }}
    .doc-content th, .doc-content td {{ border: 1px solid #e5e7eb; padding: 0.5rem 0.75rem; text-align: left; }}
    .doc-content th {{ background: #f9fafb; font-weight: 600; }}
    @media (prefers-color-scheme: dark) {{
      html:not(.light) .doc-content h1 {{ color: #f9fafb; border-color: #374151; }}
      html:not(.light) .doc-content h2 {{ color: #e5e7eb; }}
      html:not(.light) .doc-content h3 {{ color: #d1d5db; }}
      html:not(.light) .doc-content p, html:not(.light) .doc-content li {{ color: #9ca3af; }}
      html:not(.light) .doc-content code {{ background: #374151; color: #f87171; }}
      html:not(.light) .doc-content blockquote {{ border-color: var(--tachyon-primary); color: #6b7280; }}
      html:not(.light) .doc-content a {{ color: var(--tachyon-accent); }}
      html:not(.light) .doc-content th {{ background: #1f2937; }}
      html:not(.light) .doc-content th, html:not(.light) .doc-content td {{ border-color: #374151; }}
    }}
    .dark .doc-content h1 {{ color: #f9fafb; border-color: #374151; }}
    .dark .doc-content h2 {{ color: #e5e7eb; }}
    .dark .doc-content h3 {{ color: #d1d5db; }}
    .dark .doc-content p, .dark .doc-content li {{ color: #9ca3af; }}
    .dark .doc-content code {{ background: #374151; color: #f87171; }}
    .dark .doc-content blockquote {{ border-color: var(--tachyon-primary); color: #6b7280; }}
    .dark .doc-content a {{ color: var(--tachyon-accent); }}
    .dark .doc-content th {{ background: #1f2937; }}
    .dark .doc-content th, .dark .doc-content td {{ border-color: #374151; }}
    {custom_css}
  </style>
</head>
<body class="bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100 min-h-screen flex flex-col">
  {nav}
  <main class="flex-1">
    {breadcrumbs_html}
    <div class="flex">
      <div class="doc-content flex-1">
        <article>
          <header class="mb-8">
            {tags_html}
            <h1>{title}</h1>
            <div class="flex items-center gap-3 mt-2">
              {author_html}
              {updated_html}
            </div>
          </header>
          {body}
        </article>
        {prev_next_html}
      </div>
      {toc_html}
    </div>
  </main>
  <footer class="border-t border-gray-200 dark:border-gray-700 py-6 mt-12">
    <div class="max-w-4xl mx-auto px-4 text-center text-sm text-gray-500 dark:text-gray-400">
      {footer}
    </div>
  </footer>
</body>
</html>"#,
        title = ctx.title,
        site_title = ctx.site.title,
        description = escape_html(ctx.description),
        page_url = ctx.page_url,
        favicon = ctx
            .site
            .favicon_url
            .as_ref()
            .map(|url| format!(r#"<link rel="icon" href="{}">"#, url))
            .unwrap_or_default(),
        nav = nav_html,
        tags_html = tags_html,
        author_html = author_html,
        updated_html = updated_html,
        body = ctx.body,
        footer = escape_html(&ctx.site.footer),
        custom_css = ctx.site.custom_css.as_deref().unwrap_or_default(),
        theme_class = theme_class,
        language = ctx.language,
        dir = dir,
        theme_vars = theme_vars,
        breadcrumbs_html = breadcrumbs_html,
        toc_html = toc_html,
        prev_next_html = prev_next_html,
        json_ld = &ctx.json_ld,
    )
}

/// Render the site index page.
pub fn render_index_page(ctx: &IndexContext) -> String {
    let nav_html = render_nav(ctx.site, None, ctx.language, ctx.language_switcher);
    let cards_html: String = ctx
        .documents
        .iter()
        .map(render_doc_card)
        .collect::<Vec<_>>()
        .join("\n");

    let theme_vars = color_theme_css(ctx.site.color_theme.as_ref());
    let dir = crate::i18n::text_direction(ctx.language);

    format!(
        r#"<!DOCTYPE html>
<html lang="{language}" dir="{dir}" class="{theme_class}">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{site_title}</title>
  <meta name="description" content="{site_description}">
  <link rel="canonical" href="{base_url}/index.html">
  <script src="https://cdn.tailwindcss.com"></script>
  <style>
    :root {{
      --tachyon-primary: #2563eb;
      --tachyon-secondary: #7c3aed;
      --tachyon-accent: #06b6d4;
      --tachyon-code-bg: #1f2937;
      --tachyon-font-body: ui-sans-serif, system-ui, -apple-system, sans-serif;
      --tachyon-font-heading: ui-sans-serif, system-ui, -apple-system, sans-serif;
    }}
{theme_vars}
    body {{ font-family: var(--tachyon-font-body); }}
    h1, h2, h3 {{ font-family: var(--tachyon-font-heading); }}
    @media (prefers-color-scheme: dark) {{ html {{ background: #111827; color: #f9fafb; }} html h1 {{ color: #f9fafb; }} html p {{ color: #9ca3af; }} }}
    {custom_css}
  </style>
</head>
<body class="bg-white dark:bg-gray-900 min-h-screen">
  {nav}
  <main class="max-w-4xl mx-auto px-4 py-12">
    <div class="mb-12">
      <h1 class="text-4xl font-bold mb-4">{site_title}</h1>
      <p class="text-lg text-gray-600 dark:text-gray-400">{site_description}</p>
    </div>
    <div class="grid gap-6">
      {cards_html}
    </div>
  </main>
  <footer class="border-t border-gray-200 dark:border-gray-700 py-6 mt-12">
    <div class="max-w-4xl mx-auto px-4 text-center text-sm text-gray-500 dark:text-gray-400">
      {footer}
    </div>
  </footer>
</body>
</html>"#,
        site_title = escape_html(&ctx.site.title),
        site_description = escape_html(&ctx.site.description),
        base_url = ctx.site.base_url.trim_end_matches('/'),
        nav = nav_html,
        cards_html = cards_html,
        footer = escape_html(&ctx.site.footer),
        custom_css = ctx.site.custom_css.as_deref().unwrap_or_default(),
        language = ctx.language,
        dir = dir,
        theme_class = match ctx.site.theme.as_str() {
            "dark" => "dark",
            "light" => "",
            _ => "",
        },
        theme_vars = theme_vars,
    )
}

/// Render a category index page.
pub fn render_category_page(ctx: &CategoryContext) -> String {
    let cards_html: String = ctx
        .documents
        .iter()
        .map(render_doc_card)
        .collect::<Vec<_>>()
        .join("\n");

    let theme_vars = color_theme_css(ctx.site.color_theme.as_ref());
    let dir = super::text_direction(ctx.language);

    format!(
        r#"<!DOCTYPE html>
<html lang="{language}" dir="{dir}" class="{theme_class}">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{category} — {site_title}</title>
  <meta name="description" content="Documents tagged: {category}">
  <script src="https://cdn.tailwindcss.com"></script>
  <style>
    :root {{
      --tachyon-primary: #2563eb;
      --tachyon-secondary: #7c3aed;
      --tachyon-accent: #06b6d4;
      --tachyon-code-bg: #1f2937;
      --tachyon-font-body: ui-sans-serif, system-ui, -apple-system, sans-serif;
      --tachyon-font-heading: ui-sans-serif, system-ui, -apple-system, sans-serif;
    }}
{theme_vars}
    body {{ font-family: var(--tachyon-font-body); }}
  </style>
</head>
<body class="bg-white dark:bg-gray-900 min-h-screen">
  <nav class="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 px-4 py-3">
    <div class="max-w-4xl mx-auto flex items-center gap-6">
      <a href="index.html" class="font-semibold text-gray-900 dark:text-white">{site_title}</a>
      <a href="index.html" class="text-sm text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200">← Back to all docs</a>
    </div>
  </nav>
  <main class="max-w-4xl mx-auto px-4 py-12">
    <h1 class="text-3xl font-bold mb-2">Tag: {category}</h1>
    <p class="text-gray-600 dark:text-gray-400 mb-8">{count} document{plural}</p>
    <div class="grid gap-6">
      {cards_html}
    </div>
  </main>
  <footer class="border-t border-gray-200 dark:border-gray-700 py-6 mt-12">
    <div class="max-w-4xl mx-auto px-4 text-center text-sm text-gray-500 dark:text-gray-400">
      {footer}
    </div>
  </footer>
</body>
</html>"#,
        category = escape_html(ctx.category_name),
        site_title = escape_html(&ctx.site.title),
        count = ctx.documents.len(),
        plural = if ctx.documents.len() == 1 { "" } else { "s" },
        cards_html = cards_html,
        footer = escape_html(&ctx.site.footer),
        language = ctx.language,
        dir = dir,
        theme_class = match ctx.site.theme.as_str() {
            "dark" => "dark",
            "light" => "",
            _ => "",
        },
        theme_vars = theme_vars,
    )
}

/// Render the navigation bar.
fn render_nav(
    site: &SiteConfig,
    _current: Option<&str>,
    _lang: &str,
    language_switcher: &str,
) -> String {
    let extra_links: String = site
        .nav_links
        .iter()
        .map(|link| {
            format!(
                r#"<a href="{}" class="text-sm text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white">{}</a>"#,
                escape_html(&link.href),
                escape_html(&link.label),
            )
        })
        .collect::<Vec<_>>()
        .join("\n          ");

    let logo = site
        .logo_url
        .as_ref()
        .map(|url| format!(r#"<img src="{}" alt="" class="h-6 w-auto mr-2">"#, url))
        .unwrap_or_default();

    // Build language switcher if present
    let switcher_html = if language_switcher.is_empty() {
        String::new()
    } else {
        format!(
            r#"
      <div class="hidden md:flex items-center gap-3 border-l border-gray-200 dark:border-gray-600 pl-4 ml-4">
        {switcher}
      </div>"#,
            switcher = language_switcher,
        )
    };

    format!(
        r#"<nav class="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 px-4 py-3 sticky top-0 z-50">
  <div class="max-w-6xl mx-auto flex items-center justify-between">
    <div class="flex items-center gap-4">
      <a href="index.html" class="flex items-center font-semibold text-gray-900 dark:text-white hover:text-blue-600 dark:hover:text-blue-400">
        {logo}{site_title}
      </a>
      <div class="hidden md:flex items-center gap-4">
        {extra_links}
      </div>
      {switcher_html}
    </div>
  </div>
</nav>"#,
        site_title = escape_html(&site.title),
        logo = logo,
        extra_links = extra_links,
        switcher_html = switcher_html,
    )
}

/// Render a document card for the index page.
fn render_doc_card(doc: &DocCard) -> String {
    let tags_html = if doc.tags.is_empty() {
        String::new()
    } else {
        let tags: Vec<String> = doc.tags.iter().map(|t| {
            format!(
                r#"<span class="inline-block bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 text-xs px-2 py-0.5 rounded-full">{}</span>"#,
                escape_html(t)
            )
        }).collect();
        format!(
            "<div class=\"flex gap-1 flex-wrap mt-2\">{}</div>",
            tags.join(" ")
        )
    };

    format!(
        r#"<a href="{slug}.html" class="block p-6 bg-gray-50 dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 hover:border-blue-300 dark:hover:border-blue-600 hover:shadow-md transition-all">
  <h3 class="text-lg font-semibold mb-2">{title}</h3>
  <p class="text-sm text-gray-600 dark:text-gray-400 mb-2">{description}</p>
  {tags_html}
  <div class="text-xs text-gray-400 mt-3">{updated_at}</div>
</a>"#,
        slug = doc.slug,
        title = escape_html(&doc.title),
        description = escape_html(&doc.description),
        tags_html = tags_html,
        updated_at = doc.updated_at,
    )
}

/// Render tags as colored badges.
fn render_tags(tags: &[String]) -> String {
    if tags.is_empty() {
        return String::new();
    }
    let badges: Vec<String> = tags.iter().map(|t| {
        format!(
            r#"<span class="inline-block bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 text-xs px-2 py-0.5 rounded-full mr-1">{}</span>"#,
            escape_html(t)
        )
    }).collect();
    format!("<div class=\"mb-4\">{}</div>", badges.join(""))
}

/// Render breadcrumb navigation from slug segments.
fn render_breadcrumbs(breadcrumbs: &[(String, String)]) -> String {
    if breadcrumbs.len() <= 1 {
        return String::new();
    }
    let items: Vec<String> = breadcrumbs
        .iter()
        .map(|(label, href)| {
            let display = capitalize_first(label);
            format!(
                r#"<a href="{}" class="text-sm text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200">{}</a>"#,
                href, display
            )
        })
        .collect();
    let last = items.len() - 1;
    let joined = items
        .iter()
        .take(last)
        .cloned()
        .collect::<Vec<_>>()
        .join(r#"<span class="text-gray-400 dark:text-gray-600 mx-1">/</span>"#);
    format!(
        r#"<nav class="max-w-4xl mx-auto px-4 py-2" aria-label="Breadcrumb">{joined}</nav>"#
    )
}

/// Render a TOC sidebar from extracted heading entries.
fn render_toc_sidebar(toc: &[TocEntry]) -> String {
    if toc.is_empty() {
        return String::new();
    }
    let items: String = toc
        .iter()
        .map(|entry| {
            let indent = match entry.level {
                1 => "ml-0",
                2 => "ml-2",
                3 => "ml-4",
                4 => "ml-6",
                _ => "ml-4",
            };
            format!(
                r##"<li class="{indent}"><a href="#{id}" class="text-sm text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white block py-0.5">{title}</a></li>"##,
                indent = indent,
                id = entry.id,
                title = escape_html(&entry.title),
            )
        })
        .collect::<Vec<_>>()
        .join("\n        ");
    format!(
        r#"<aside class="toc-sidebar hidden lg:block w-56 flex-shrink-0 border-l border-gray-200 dark:border-gray-700 pl-6 ml-6 sticky top-16 self-start max-h-[calc(100vh-4rem)] overflow-y-auto">
  <nav class="toc" aria-label="Table of Contents">
    <h2 class="text-xs font-semibold uppercase tracking-wider text-gray-500 dark:text-gray-400 mb-3">On this page</h2>
    <ul class="space-y-1">
        {items}
    </ul>
  </nav>
</aside>"#
    )
}

/// Render prev/next page navigation links.
fn render_prev_next(
    prev: Option<&(String, String)>,
    next: Option<&(String, String)>,
) -> String {
    let prev_html = match prev {
        Some((title, href)) => format!(
            r#"<a href="{}" class="flex-1 text-left px-4 py-3 rounded-lg border border-gray-200 dark:border-gray-700 hover:border-blue-300 dark:hover:border-blue-600 hover:shadow-sm transition-all">
  <span class="text-xs text-gray-500 dark:text-gray-400 block">Previous</span>
  <span class="text-sm font-medium text-gray-900 dark:text-gray-100">{}</span>
</a>"#,
            href, escape_html(title)
        ),
        None => String::new(),
    };
    let next_html = match next {
        Some((title, href)) => format!(
            r#"<a href="{}" class="flex-1 text-right px-4 py-3 rounded-lg border border-gray-200 dark:border-gray-700 hover:border-blue-300 dark:hover:border-blue-600 hover:shadow-sm transition-all">
  <span class="text-xs text-gray-500 dark:text-gray-400 block">Next</span>
  <span class="text-sm font-medium text-gray-900 dark:text-gray-100">{}</span>
</a>"#,
            href, escape_html(title)
        ),
        None => String::new(),
    };
    if prev_html.is_empty() && next_html.is_empty() {
        return String::new();
    }
    format!(
        r#"<div class="flex gap-4 mt-8 mb-4">{prev_html}{next_html}</div>"#
    )
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

/// Escape HTML special characters.
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// Default templates are defined above — these constants are for external reference.
pub const DEFAULT_BASE_TEMPLATE: &str = "";
pub const DEFAULT_INDEX_TEMPLATE: &str = "";
pub const DEFAULT_DOC_TEMPLATE: &str = "";
