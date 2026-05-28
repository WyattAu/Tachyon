//! HTML templates for the SSG engine.
//!
//! Uses string interpolation instead of a template engine to avoid
//! additional dependencies. Templates are designed to be:
//! - Clean and modern (inspired by Docusaurus / VitePress)
//! - Responsive with Tailwind CSS CDN
//! - Dark mode support via `class` strategy on `<html>`
//! - Customizable via ColorTheme CSS custom properties
//! - Accessible (semantic HTML, ARIA labels)

use crate::manifest::{ColorTheme, SidebarItem, SiteConfig};
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

    let mut css = format!(
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
    );

    if t.dark_primary.is_some()
        || t.dark_secondary.is_some()
        || t.dark_accent.is_some()
        || t.dark_code_bg.is_some()
    {
        css.push_str("\n    .dark {");
        if let Some(ref dp) = t.dark_primary {
            css.push_str(&format!(" --tachyon-primary: {};", dp));
        }
        if let Some(ref ds) = t.dark_secondary {
            css.push_str(&format!(" --tachyon-secondary: {};", ds));
        }
        if let Some(ref da) = t.dark_accent {
            css.push_str(&format!(" --tachyon-accent: {};", da));
        }
        if let Some(ref dcb) = t.dark_code_bg {
            css.push_str(&format!(" --tachyon-code-bg: {};", dcb));
        }
        css.push_str(" }");
    }

    css
}

/// Render a document page.
pub fn render_doc_page(ctx: &PageContext) -> String {
    let nav_html = render_nav(
        ctx.site,
        ctx.current_slug,
        ctx.language,
        ctx.language_switcher,
        ctx.root_prefix,
    );
    let tags_html = render_tags(ctx.tags);
    let theme_class = match ctx.site.theme.as_str() {
        "dark" => "dark",
        "light" => "light",
        _ => "",
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
    let sidebar_html = render_sidebar(
        &ctx.site.menu_items,
        ctx.current_slug,
        ctx.root_prefix,
        ctx.sidebar_auto_items,
    );

    let pagefind_base = ctx
        .site
        .base_url
        .trim_end_matches('/')
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    let pagefind_prefix = match pagefind_base.find('/') {
        Some(idx) => {
            let path = &pagefind_base[idx..];
            path.trim_end_matches('/').to_string()
        }
        None => String::new(),
    };

    let pagefind_css = if ctx.site.pagefind_enabled {
        format!(
            r#"<link href="{}pagefind/pagefind-ui.css" rel="stylesheet">"#,
            ctx.root_prefix
        )
    } else {
        String::new()
    };

    let pagefind_js = if ctx.site.pagefind_enabled {
        format!(
            r##"<script src="{}pagefind/pagefind-ui.js"></script>
<script>
(function() {{
  document.addEventListener('DOMContentLoaded', function() {{
    new PagefindUI({{element:"#tachyon-search",showSubResults:true,baseUrl:"{base}"}});
  }});
}})();
</script>"##,
            ctx.root_prefix,
            base = pagefind_prefix
        )
    } else {
        String::new()
    };

    let mermaid_script = if ctx.site.mermaid_enabled {
        r##"<script src="https://cdn.jsdelivr.net/npm/mermaid@11/dist/mermaid.min.js"></script>
<script>mermaid.initialize({startOnLoad:true,theme:"default"});</script>"##
            .to_string()
    } else {
        String::new()
    };

    let highlight_script = if ctx.site.syntax_highlighting_enabled {
        let theme = &ctx.site.code_theme;
        format!(
            r##"<link rel="stylesheet" href="https://cdn.jsdelivr.net/gh/highlightjs/cdn-release@11.11.1/build/styles/{theme}.min.css">
<script src="https://cdn.jsdelivr.net/gh/highlightjs/cdn-release@11.11.1/build/highlight.min.js"></script>
<script>document.addEventListener('DOMContentLoaded',function(){{hljs.highlightAll({{cssSelector:'pre code:not(.language-mermaid)'}})}});</script>"##
        )
    } else {
        String::new()
    };

    let footer_html = render_footer(ctx.site, ctx.root_prefix);

    format!(
        r##"<!DOCTYPE html>
<html lang="{language}" dir="{dir}" class="{theme_class}">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{title} — {site_title}</title>
  <meta name="description" content="{description}">
  <link rel="canonical" href="{page_url}">
  {hreflang_tags}
  <meta property="og:title" content="{title}">
  <meta property="og:description" content="{description}">
  <meta property="og:url" content="{page_url}">
  <meta property="og:type" content="article">
  <meta property="og:site_name" content="{site_title}">
{og_image_tags}
  <meta name="twitter:card" content="summary">
  <meta name="twitter:title" content="{title}">
  <meta name="twitter:description" content="{description}">
{twitter_image_tags}
  {favicon}
  {json_ld}
  <script src="https://cdn.tailwindcss.com"></script>
  <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/katex.min.css">
  <script defer src="https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/katex.min.js"></script>
  <script defer src="https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/contrib/auto-render.min.js" onload="renderMathInElement(document.body);"></script>
  {pagefind_css}
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
    .admonition {{ border-left: 4px solid; padding: 0.75rem 1rem; margin: 1rem 0; border-radius: 0.25rem; }}
    .admonition-title {{ font-weight: 600; margin-bottom: 0.25rem; }}
    .admonition-note {{ border-color: #2563eb; background: #eff6ff; }}
    .admonition-note .admonition-title {{ color: #2563eb; }}
    .admonition-warning {{ border-color: #d97706; background: #fffbeb; }}
    .admonition-warning .admonition-title {{ color: #d97706; }}
    .admonition-tip {{ border-color: #059669; background: #ecfdf5; }}
    .admonition-tip .admonition-title {{ color: #059669; }}
    .admonition-danger {{ border-color: #dc2626; background: #fef2f2; }}
    .admonition-danger .admonition-title {{ color: #dc2626; }}
    .admonition-info {{ border-color: #0891b2; background: #ecfeff; }}
    .admonition-info .admonition-title {{ color: #0891b2; }}
    .admonition-success {{ border-color: #16a34a; background: #f0fdf4; }}
    .admonition-success .admonition-title {{ color: #16a34a; }}
    @media (prefers-color-scheme: dark) {{
      html:not(.light) .admonition-note {{ background: #1e3a5f; }}
      html:not(.light) .admonition-warning {{ background: #422006; }}
      html:not(.light) .admonition-tip {{ background: #064e3b; }}
      html:not(.light) .admonition-danger {{ background: #450a0a; }}
      html:not(.light) .admonition-info {{ background: #164e63; }}
      html:not(.light) .admonition-success {{ background: #052e16; }}
    }}
    .dark .admonition-note {{ background: #1e3a5f; }}
    .dark .admonition-warning {{ background: #422006; }}
    .dark .admonition-tip {{ background: #064e3b; }}
    .dark .admonition-danger {{ background: #450a0a; }}
    .dark .admonition-info {{ background: #164e63; }}
    .dark .admonition-success {{ background: #052e16; }}
    nav.toc {{ background: #f9fafb; border: 1px solid #e5e7eb; border-radius: 0.5rem; padding: 1rem 1.25rem; margin-bottom: 1.5rem; }}
    nav.toc ul {{ list-style: none; padding-left: 0; margin: 0; }}
    nav.toc li {{ margin-bottom: 0.25rem; }}
    nav.toc li a {{ color: #4b5563; text-decoration: none; font-size: 0.875rem; line-height: 1.5; display: block; padding: 0.125rem 0; border-left: 2px solid transparent; }}
    nav.toc li a:hover {{ color: #2563eb; }}
    nav.toc li a.toc-active {{ color: #2563eb; border-left-color: #2563eb; font-weight: 500; }}
    nav.toc li.toc-h3 {{ padding-left: 1rem; }}
    .code-block-wrapper {{ position: relative; }}
    .code-block-wrapper pre {{ margin-bottom: 0; }}
    .code-copy-btn {{ position: absolute; top: 0.5rem; right: 0.5rem; background: rgba(255,255,255,0.1); color: #9ca3af; border: 1px solid rgba(255,255,255,0.2); border-radius: 0.25rem; padding: 0.25rem 0.5rem; font-size: 0.75rem; cursor: pointer; opacity: 0; transition: opacity 0.2s; }}
    .code-block-wrapper:hover .code-copy-btn {{ opacity: 1; }}
    .code-copy-btn:hover {{ background: rgba(255,255,255,0.2); color: #f9fafb; }}
    .code-title {{ background: #f3f4f6; color: #374151; padding: 0.375rem 0.75rem; font-size: 0.75rem; font-weight: 500; border-bottom: 1px solid #e5e7eb; border-radius: 0.5rem 0.5rem 0 0; margin-bottom: 0; }}
    .dark .code-title {{ background: #374151; color: #d1d5db; border-color: #4b5563; }}
    .code-title + .code-block-wrapper {{ border-radius: 0 0 0.5rem 0.5rem; margin-top: 0; }}
    .content-group {{ margin: 1rem 0; border-radius: 0.5rem; overflow: hidden; border: 1px solid #e5e7eb; }}
    .content-tabs {{ display: flex; background: #f3f4f6; border-bottom: 1px solid #e5e7eb; }}
    .content-tab {{ padding: 0.5rem 1rem; font-size: 0.875rem; font-weight: 500; color: #6b7280; background: none; border: none; cursor: pointer; border-bottom: 2px solid transparent; }}
    .content-tab.active {{ color: #2563eb; border-bottom-color: #2563eb; background: #fff; }}
    .content-tab:hover {{ color: #374151; }}
    .content-tab-panel {{ display: none; padding: 1rem; }}
    .content-tab-panel.active {{ display: block; }}
    @media (prefers-color-scheme: dark) {{
      html:not(.light) .content-group {{ border-color: #374151; }}
      html:not(.light) .content-tabs {{ background: #1f2937; border-color: #374151; }}
      html:not(.light) .content-tab {{ color: #9ca3af; }}
      html:not(.light) .content-tab.active {{ color: #60a5fa; border-bottom-color: #60a5fa; background: #111827; }}
    }}
    .dark .content-group {{ border-color: #374151; }}
    .dark .content-tabs {{ background: #1f2937; border-color: #374151; }}
    .dark .content-tab {{ color: #9ca3af; }}
    .dark .content-tab.active {{ color: #60a5fa; border-bottom-color: #60a5fa; background: #111827; }}
    .code-group {{ margin-bottom: 1rem; border-radius: 0.5rem; overflow: hidden; border: 1px solid #e5e7eb; }}
    .code-tabs {{ display: flex; background: #f3f4f6; border-bottom: 1px solid #e5e7eb; }}
    .code-tabs .tab {{ padding: 0.375rem 0.75rem; font-size: 0.75rem; font-weight: 500; color: #6b7280; background: none; border: none; cursor: pointer; border-bottom: 2px solid transparent; }}
    .code-tabs .tab.active {{ color: #2563eb; border-bottom-color: #2563eb; background: #fff; }}
    .code-tabs .tab:hover {{ color: #374151; }}
    .code-group .tab-content {{ display: none; }}
    .code-group .tab-content.active {{ display: block; }}
    .code-group .tab-content pre {{ margin: 0; border-radius: 0; border: none; }}
    @media (prefers-color-scheme: dark) {{
      html:not(.light) .code-group {{ border-color: #374151; }}
      html:not(.light) .code-tabs {{ background: #1f2937; border-color: #374151; }}
      html:not(.light) .code-tabs .tab {{ color: #9ca3af; }}
      html:not(.light) .code-tabs .tab.active {{ color: #60a5fa; border-bottom-color: #60a5fa; background: #111827; }}
    }}
    .dark .code-group {{ border-color: #374151; }}
    .dark .code-tabs {{ background: #1f2937; border-color: #374151; }}
    .dark .code-tabs .tab {{ color: #9ca3af; }}
    .dark .code-tabs .tab.active {{ color: #60a5fa; border-bottom-color: #60a5fa; background: #111827; }}
    @media (prefers-color-scheme: dark) {{
      html:not(.light) nav.toc {{ background: #1f2937; border-color: #374151; }}
      html:not(.light) nav.toc li a {{ color: #9ca3af; }}
      html:not(.light) nav.toc li a.toc-active {{ color: #60a5fa; border-left-color: #60a5fa; }}
    }}
    .dark nav.toc {{ background: #1f2937; border-color: #374151; }}
    .dark nav.toc li a {{ color: #9ca3af; }}
    .dark nav.toc li a.toc-active {{ color: #60a5fa; border-left-color: #60a5fa; }}
    .breadcrumbs {{ list-style: none; display: flex; flex-wrap: wrap; align-items: center; gap: 0.25rem; padding: 0; margin: 0; }}
    .breadcrumbs-item + .breadcrumbs-item::before {{ content: "/"; color: #9ca3af; margin-right: 0.25rem; }}
    .mermaid {{ margin: 1rem 0; text-align: center; }}
    .dark .breadcrumbs-item + .breadcrumbs-item::before {{ color: #4b5563; }}
    /* Explicit light theme overrides (when user toggles to light mode) */
    .light {{ background: #ffffff; color: #111827; }}
    .light .doc-content h1 {{ color: #111827; border-color: #e5e7eb; }}
    .light .doc-content h2 {{ color: #1f2937; }}
    .light .doc-content h3 {{ color: #374151; }}
    .light .doc-content p, .light .doc-content li {{ color: #4b5563; }}
    .light .doc-content code {{ background: #f3f4f6; color: #dc2626; }}
    .light .doc-content blockquote {{ border-color: #2563eb; color: #6b7280; }}
    .light .doc-content a {{ color: #2563eb; }}
    .light .doc-content th {{ background: #f9fafb; }}
    .light .doc-content th, .light .doc-content td {{ border-color: #e5e7eb; }}
    .light .admonition-note {{ background: #eff6ff; }}
    .light .admonition-warning {{ background: #fffbeb; }}
    .light .admonition-tip {{ background: #ecfdf5; }}
    .light .admonition-danger {{ background: #fef2f2; }}
    .light .admonition-info {{ background: #ecfeff; }}
    .light .admonition-success {{ background: #f0fdf4; }}
    .light nav.toc {{ background: #f9fafb; border-color: #e5e7eb; }}
    .light nav.toc li a {{ color: #4b5563; }}
    .light nav.toc li a.toc-active {{ color: #2563eb; border-left-color: #2563eb; }}
    .light .code-title {{ background: #f3f4f6; color: #374151; border-color: #e5e7eb; }}
    .light .content-tabs {{ background: #f3f4f6; border-color: #e5e7eb; }}
    .light .content-tab {{ color: #4b5563; border-color: #e5e7eb; background: #f3f4f6; }}
    .light .content-tab.active {{ color: #2563eb; border-color: #2563eb; background: #fff; }}
    .light .content-tab-panel {{ border-color: #e5e7eb; }}
    .light .code-group .tab {{ background: #f3f4f6; color: #4b5563; border-color: #e5e7eb; }}
    .light .code-group .tab.active {{ background: #fff; color: #2563eb; border-color: #2563eb; }}
    .light .code-group .tab-content {{ border-color: #e5e7eb; }}
    .light .breadcrumbs-item + .breadcrumbs-item::before {{ color: #9ca3af; }}
    /* Mobile sidebar overlay panel */
    @media (max-width: 767px) {{
      .sidebar-open {{
        position: fixed !important;
        top: 0 !important;
        left: 0 !important;
        bottom: 0 !important;
        width: 16rem !important;
        max-height: 100vh !important;
        z-index: 45 !important;
        background: white !important;
        padding: 1rem !important;
        margin: 0 !important;
        border-right: 1px solid #e5e7eb !important;
        overflow-y: auto !important;
        box-shadow: 2px 0 8px rgba(0,0,0,0.15) !important;
        transition: transform 0.2s ease-out;
      }}
      .sidebar-closed {{ display: none !important; }}
      .dark .sidebar-open {{ background: #111827 !important; border-right-color: #374151 !important; }}
    }}
    @media (min-width: 768px) {{
      .sidebar-closed {{ display: block; }}
    }}
    /* Search overlay */
    .tachyon-search-overlay {{
      display: none; position: fixed; top: 0; left: 0; right: 0; bottom: 0;
      background: rgba(0,0,0,0.5); z-index: 100; justify-content: center; padding-top: 10vh;
    }}
    .tachyon-search-overlay.open {{ display: flex; }}
    .tachyon-search-box {{
      background: white; border-radius: 0.5rem; width: 90%; max-width: 40rem;
      box-shadow: 0 20px 60px rgba(0,0,0,0.3); overflow: hidden;
    }}
    .dark .tachyon-search-box {{ background: #1f2937; }}
    .tachyon-search-box #tachyon-search {{
      border: none; outline: none; width: 100%; padding: 1rem 1.5rem;
      font-size: 1rem; background: transparent; color: #111827;
    }}
    .dark .tachyon-search-box #tachyon-search {{ color: #f9fafb; }}
    .tachyon-search-kbd {{
      position: absolute; right: 1rem; top: 50%; transform: translateY(-50%);
      font-size: 0.75rem; padding: 0.125rem 0.375rem; border: 1px solid #d1d5db;
      border-radius: 0.25rem; color: #9ca3af; pointer-events: none;
    }}
    /* TOC sidebar close button */
    .toc-close {{ display: none; }}
    @media (max-width: 1023px) {{
      .toc-close {{ display: block; }}
    }}
    /* Theme dropdown */
    #tachyon-theme-dropdown button.tachyon-theme-mode-active {{ font-weight: 600; background: #eff6ff; }}
    .dark #tachyon-theme-dropdown button.tachyon-theme-mode-active {{ background: #1e3a5f; }}
    #tachyon-theme-dropdown button.tachyon-theme-color-active {{ font-weight: 600; background: #f0f9ff; }}
    .dark #tachyon-theme-dropdown button.tachyon-theme-color-active {{ background: #1e293b; }}
    /* Color theme presets */
    [data-active-color="ocean"] {{ --tachyon-primary: #0ea5e9; --tachyon-secondary: #0284c7; --tachyon-accent: #06b6d4; }}
    [data-active-color="ocean"] .dark, .dark[data-active-color="ocean"] {{ --tachyon-primary: #38bdf8; --tachyon-secondary: #0ea5e9; --tachyon-accent: #22d3ee; }}
    [data-active-color="forest"] {{ --tachyon-primary: #059669; --tachyon-secondary: #065f46; --tachyon-accent: #16a34a; }}
    [data-active-color="forest"] .dark, .dark[data-active-color="forest"] {{ --tachyon-primary: #34d399; --tachyon-secondary: #059669; --tachyon-accent: #4ade80; }}
    [data-active-color="sunset"] {{ --tachyon-primary: #ea580c; --tachyon-secondary: #b45309; --tachyon-accent: #f59e0b; }}
    [data-active-color="sunset"] .dark, .dark[data-active-color="sunset"] {{ --tachyon-primary: #fb923c; --tachyon-secondary: #ea580c; --tachyon-accent: #fbbf24; }}
    [data-active-color="rose"] {{ --tachyon-primary: #e11d48; --tachyon-secondary: #be123c; --tachyon-accent: #f472b6; }}
    [data-active-color="rose"] .dark, .dark[data-active-color="rose"] {{ --tachyon-primary: #fb7185; --tachyon-secondary: #e11d48; --tachyon-accent: #f9a8d4; }}
    [data-active-color="slate"] {{ --tachyon-primary: #475569; --tachyon-secondary: #1e293b; --tachyon-accent: #94a3b8; }}
    [data-active-color="slate"] .dark, .dark[data-active-color="slate"] {{ --tachyon-primary: #94a3b8; --tachyon-secondary: #64748b; --tachyon-accent: #cbd5e1; }}
    {custom_css}
  </style>
</head>
  <body class="bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100 min-h-screen flex flex-col">
  <a href="#doc-content" class="sr-only focus:not-sr-only focus:absolute focus:top-2 focus:left-2 focus:z-[200] focus:bg-blue-600 focus:text-white focus:px-4 focus:py-2 focus:rounded">Skip to content</a>
  {nav}
  <!-- Search overlay -->
  <div class="tachyon-search-overlay" id="tachyon-search-overlay" onclick="if(event.target===this)this.classList.remove('open')">
    <div class="tachyon-search-box" style="position:relative;">
      <div id="tachyon-search"></div>
      <span class="tachyon-search-kbd">ESC</span>
    </div>
  </div>
  <button id="tachyon-sidebar-toggle" class="md:hidden fixed bottom-4 right-4 z-50 bg-blue-600 text-white rounded-full w-12 h-12 flex items-center justify-center shadow-lg hover:bg-blue-700 transition-colors" aria-label="Toggle sidebar" aria-expanded="false">&#9776;</button>
  <main class="flex-1" id="doc-content">
    {breadcrumbs_html}
    <div class="flex">
      {sidebar_html}
      <div class="doc-content flex-1" data-pagefind-body>
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
   {footer_html}
    {pagefind_js}
    {highlight_script}
    {mermaid_script}
  <script>
  // Dark/light theme toggle + color themes
  (function() {{
    var stored = localStorage.getItem('tachyon-theme');
    var storedColor = localStorage.getItem('tachyon-color') || 'default';
    if (stored === 'dark' || stored === 'light') document.documentElement.classList.add(stored);
    else if (window.matchMedia('(prefers-color-scheme: dark)').matches) document.documentElement.classList.add('dark');
    if (storedColor !== 'default') document.documentElement.setAttribute('data-active-color', storedColor);

    // Init toggle icon
    var isDarkInit = document.documentElement.classList.contains('dark');
    var toggleBtn = document.getElementById('tachyon-theme-toggle');
    var dropdown = document.getElementById('tachyon-theme-dropdown');
    if (toggleBtn) toggleBtn.innerHTML = isDarkInit ? '&#9728;' : '&#9790;';

    // Highlight active mode
    var activeMode = stored === 'dark' ? 'dark' : (stored === 'light' ? 'light' : 'auto');
    document.querySelectorAll('[data-theme-mode]').forEach(function(btn) {{
      if (btn.dataset.themeMode === activeMode) btn.classList.add('tachyon-theme-mode-active');
    }});
    // Highlight active color
    document.querySelectorAll('[data-theme-color]').forEach(function(btn) {{
      if (btn.dataset.themeColor === storedColor) btn.classList.add('tachyon-theme-color-active');
    }});

    // Toggle button click -> show dropdown
    toggleBtn.addEventListener('click', function(e) {{
      e.stopPropagation();
      dropdown.classList.toggle('hidden');
    }});

    // Mode buttons
    document.querySelectorAll('[data-theme-mode]').forEach(function(btn) {{
      btn.addEventListener('click', function() {{
        var mode = this.dataset.themeMode;
        var html = document.documentElement;
        html.classList.remove('dark', 'light');
        if (mode === 'dark' || mode === 'light') html.classList.add(mode);
        localStorage.setItem('tachyon-theme', mode);
        var isDark = html.classList.contains('dark');
        toggleBtn.innerHTML = isDark ? '&#9728;' : '&#9790;';
        dropdown.classList.add('hidden');
        // Update active highlighting
        document.querySelectorAll('[data-theme-mode]').forEach(function(b) {{ b.classList.remove('tachyon-theme-mode-active'); }});
        this.classList.add('tachyon-theme-mode-active');
      }});
    }});

    // Color buttons
    document.querySelectorAll('[data-theme-color]').forEach(function(btn) {{
      btn.addEventListener('click', function() {{
        var color = this.dataset.themeColor;
        document.documentElement.setAttribute('data-active-color', color);
        localStorage.setItem('tachyon-color', color);
        dropdown.classList.add('hidden');
        // Update active highlighting
        document.querySelectorAll('[data-theme-color]').forEach(function(b) {{ b.classList.remove('tachyon-color-active'); }});
        this.classList.add('tachyon-color-active');
      }});
    }});

  // Close dropdown on outside click
  document.addEventListener('click', function(e) {{
    var sel = document.getElementById('tachyon-theme-selector');
    if (sel && !sel.contains(e.target)) {{
      var dd = document.getElementById('tachyon-theme-dropdown');
      if (dd) dd.classList.add('hidden');
    }}
  }});
  }})();

  // Search overlay (Ctrl+K / Cmd+K)
  (function() {{
    var overlay = document.getElementById('tachyon-search-overlay');
    document.addEventListener('keydown', function(e) {{
      if ((e.ctrlKey || e.metaKey) && e.key === 'k') {{
        e.preventDefault();
        overlay.classList.toggle('open');
        if (overlay.classList.contains('open')) {{
          var input = overlay.querySelector('input');
          if (input) setTimeout(function() {{ input.focus(); }}, 100);
        }}
      }}
      if (e.key === 'Escape') overlay.classList.remove('open');
    }});
  }})();

  // TOC scroll-spy
  (function() {{
    var headings = document.querySelectorAll('.doc-content h2[id], .doc-content h3[id]');
    var tocLinks = document.querySelectorAll('.toc-sidebar a[href^="#"]');
    if (!headings.length || !tocLinks.length) return;
    var observer = new IntersectionObserver(function(entries) {{
      entries.forEach(function(entry) {{
        if (entry.isIntersecting) {{
          tocLinks.forEach(function(l) {{ l.classList.remove('toc-active'); }});
          var link = document.querySelector('.toc-sidebar a[href="#' + entry.target.id + '"]');
          if (link) link.classList.add('toc-active');
        }}
      }});
    }}, {{ rootMargin: '-10% 0px -80% 0px', threshold: 0 }});
    headings.forEach(function(h) {{ observer.observe(h); }});
  }})();
  </script>
</body>
</html>"##,
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
        footer_html = footer_html,
        custom_css = ctx.site.custom_css.as_deref().unwrap_or_default(),
        theme_class = theme_class,
        language = ctx.language,
        dir = dir,
        theme_vars = theme_vars,
        breadcrumbs_html = breadcrumbs_html,
        toc_html = toc_html,
        prev_next_html = prev_next_html,
        sidebar_html = sidebar_html,
        json_ld = &ctx.json_ld,
        pagefind_css = pagefind_css,
        pagefind_js = pagefind_js,
        mermaid_script = mermaid_script,
        highlight_script = highlight_script,
        hreflang_tags = ctx.hreflang_tags,
        og_image_tags = ctx
            .og_image
            .map(|url| format!(
                r#"  <meta property="og:image" content="{}">"#,
                escape_html(url)
            ))
            .unwrap_or_default(),
        twitter_image_tags = ctx
            .og_image
            .map(|url| format!(
                r#"  <meta name="twitter:image" content="{}">"#,
                escape_html(url)
            ))
            .unwrap_or_default(),
    )
}

/// Render the site index page.
pub fn render_index_page(ctx: &IndexContext) -> String {
    let nav_html = render_nav(ctx.site, None, ctx.language, ctx.language_switcher, "");
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
    root_prefix: &str,
) -> String {
    let extra_links: String = site
        .nav_links
        .iter()
        .map(|link| {
            format!(
                r#"<a href="{rp}{href}" class="text-sm text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white">{label}</a>"#,
                rp = root_prefix,
                href = escape_html(&link.href),
                label = escape_html(&link.label),
            )
        })
        .collect::<Vec<_>>()
        .join("\n          ");

    let logo = site
        .logo_url
        .as_ref()
        .map(|url| format!(r#"<img src="{}" alt="" class="h-6 w-auto mr-2">"#, url))
        .unwrap_or_default();

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
        r#"<nav data-pagefind-ignore class="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 px-4 py-3 sticky top-0 z-50">
  <div class="max-w-6xl mx-auto flex items-center justify-between">
    <div class="flex items-center gap-4">
      <a href="{rp}index.html" class="flex items-center font-semibold text-gray-900 dark:text-white hover:text-blue-600 dark:hover:text-blue-400">
        {logo}{site_title}
      </a>
      <div class="hidden md:flex items-center gap-4" id="tachyon-nav-links">
        {extra_links}
      </div>
      {switcher_html}
    </div>
    <div class="flex items-center gap-2">
      <button id="tachyon-nav-search-btn" class="p-2 text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 rounded" aria-label="Search (Ctrl+K)" title="Search (Ctrl+K)">
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"></path></svg>
      </button>
      <div class="relative" id="tachyon-theme-selector">
        <button id="tachyon-theme-toggle" class="p-2 text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 rounded" aria-label="Toggle theme" title="Theme">
          &#9790;
        </button>
        <div id="tachyon-theme-dropdown" class="hidden absolute right-0 top-full mt-1 w-48 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-lg z-[200] overflow-hidden">
          <div class="p-2 border-b border-gray-200 dark:border-gray-600">
            <span class="text-xs font-medium text-gray-500 dark:text-gray-400 px-2">Mode</span>
          </div>
          <button data-theme-mode="auto" class="w-full text-left px-3 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-b transition-colors tachyon-theme-mode-active">&#9790; Auto</button>
          <button data-theme-mode="light" class="w-full text-left px-3 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-b transition-colors">&#9788; Light</button>
          <button data-theme-mode="dark" class="w-full text-left px-3 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-b transition-colors">&#9728; Dark</button>
          <div class="p-2 border-t border-gray-200 dark:border-gray-600 mt-1">
            <span class="text-xs font-medium text-gray-500 dark:text-gray-400 px-2">Color Theme</span>
          </div>
          <button data-theme-color="default" class="w-full text-left px-3 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-b transition-colors tachyon-theme-color-active flex items-center gap-2">
            <span class="w-3 h-3 rounded-full border border-gray-300" style="background:linear-gradient(135deg,#2563eb,#7c3aed)"></span> Default
          </button>
          <button data-theme-color="ocean" class="w-full text-left px-3 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-b transition-colors flex items-center gap-2">
            <span class="w-3 h-3 rounded-full border border-gray-300" style="background:linear-gradient(135deg,#0ea5e9,#0284c7)"></span> Ocean
          </button>
          <button data-theme-color="forest" class="w-full text-left px-3 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-b transition-colors flex items-center gap-2">
            <span class="w-3 h-3 rounded-full border border-gray-300" style="background:linear-gradient(135deg,#059669,#065f46)"></span> Forest
          </button>
          <button data-theme-color="sunset" class="w-full text-left px-3 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-b transition-colors flex items-center gap-2">
            <span class="w-3 h-3 rounded-full border border-gray-300" style="background:linear-gradient(135deg,#ea580c,#b45309)"></span> Sunset
          </button>
          <button data-theme-color="rose" class="w-full text-left px-3 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-b transition-colors flex items-center gap-2">
            <span class="w-3 h-3 rounded-full border border-gray-300" style="background:linear-gradient(135deg,#e11d48,#be123c)"></span> Rose
          </button>
          <button data-theme-color="slate" class="w-full text-left px-3 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-b transition-colors flex items-center gap-2">
            <span class="w-3 h-3 rounded-full border border-gray-300" style="background:linear-gradient(135deg,#475569,#1e293b)"></span> Slate
          </button>
        </div>
      </div>
      <button id="tachyon-nav-hamburger" class="md:hidden p-2 text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 rounded" aria-label="Menu">
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"></path></svg>
      </button>
    </div>
  </div>
  <!-- Mobile nav dropdown -->
  <div id="tachyon-mobile-nav" class="hidden md:hidden border-t border-gray-200 dark:border-gray-700 px-4 py-3 space-y-2">
    {extra_links}
  </div>
  <script>
  (function() {{
    var hamburger = document.getElementById('tachyon-nav-hamburger');
    var mobileNav = document.getElementById('tachyon-mobile-nav');
    if (hamburger && mobileNav) {{
      hamburger.addEventListener('click', function() {{
        mobileNav.classList.toggle('hidden');
      }});
    }}
    var searchBtn = document.getElementById('tachyon-nav-search-btn');
    var searchOverlay = document.getElementById('tachyon-search-overlay');
    if (searchBtn && searchOverlay) {{
      searchBtn.addEventListener('click', function() {{
        searchOverlay.classList.add('open');
        var input = searchOverlay.querySelector('input');
        if (input) setTimeout(function() {{ input.focus(); }}, 100);
      }});
    }}
  }})();
  </script>
</nav>"#,
        rp = root_prefix,
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
        .enumerate()
        .map(|(i, (label, href))| {
            let display = capitalize_first(label);
            let position = i + 1;
            if i == breadcrumbs.len() - 1 {
                format!(
                    r#"<li class="breadcrumbs-item" itemprop="itemListElement" itemscope itemtype="https://schema.org/ListItem" aria-current="page">
      <span itemprop="name" class="text-sm text-gray-900 dark:text-gray-100 font-medium">{display}</span>
      <meta itemprop="position" content="{position}" />
    </li>"#,
                    display = display,
                    position = position,
                )
            } else {
                format!(
                    r#"<li class="breadcrumbs-item" itemprop="itemListElement" itemscope itemtype="https://schema.org/ListItem">
      <a itemprop="item" href="{href}"><span itemprop="name" class="text-sm text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200">{display}</span></a>
      <meta itemprop="position" content="{position}" />
    </li>"#,
                    href = href,
                    display = display,
                    position = position,
                )
            }
        })
        .collect();
    let items_html = items.join("\n    ");
    format!(
        r#"<nav class="max-w-4xl mx-auto px-4 py-2" aria-label="Breadcrumb">
  <ol class="breadcrumbs" itemscope itemtype="https://schema.org/BreadcrumbList">
    {items_html}
  </ol>
</nav>"#,
        items_html = items_html,
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
        r#"<aside class="toc-sidebar hidden lg:block w-56 flex-shrink-0 border-l border-gray-200 dark:border-gray-700 pl-6 ml-6 sticky top-16 self-start max-h-[calc(100vh-4rem)] overflow-y-auto" data-pagefind-ignore>
  <div class="flex items-center justify-between mb-3">
    <h2 class="text-xs font-semibold uppercase tracking-wider text-gray-500 dark:text-gray-400">On this page</h2>
    <button class="toc-close text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 p-0.5" onclick="this.closest('aside').style.display='none'" aria-label="Close table of contents">
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path></svg>
    </button>
  </div>
  <nav class="toc" aria-label="Table of Contents">
    <ul class="space-y-1">
{items}
    </ul>
  </nav>
</aside>"#,
        items = items
    )
}

/// Render a collapsible sidebar from the site menu structure.
fn render_sidebar(
    menu_items: &[SidebarItem],
    current_slug: Option<&str>,
    root_prefix: &str,
    auto_items: &[(String, String)],
) -> String {
    render_sidebar_inner(menu_items, current_slug, root_prefix, auto_items)
}

pub(crate) fn render_sidebar_test(
    menu_items: &[SidebarItem],
    current_slug: Option<&str>,
) -> String {
    render_sidebar_inner(menu_items, current_slug, "", &[])
}

fn render_sidebar_inner(
    menu_items: &[SidebarItem],
    current_slug: Option<&str>,
    root_prefix: &str,
    auto_items: &[(String, String)],
) -> String {
    let items_html = if !menu_items.is_empty() {
        menu_items
            .iter()
            .map(|item| render_sidebar_item(item, current_slug, 0, root_prefix))
            .collect::<Vec<_>>()
            .join("\n")
    } else if !auto_items.is_empty() {
        auto_items
            .iter()
            .map(|(slug, title)| {
                let is_active = current_slug.map(|s| s == slug.as_str()).unwrap_or(false);
                let active_class = if is_active {
                    " bg-blue-50 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300 font-medium"
                } else {
                    " text-gray-700 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-gray-800"
                };
                format!(
                    r#"<li><a href="{root_prefix}{slug}.html" class="block px-3 py-1.5 rounded text-sm transition-colors{active_class}">{title}</a></li>"#
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        return String::new();
    };
    format!(
        r#"<aside id="tachyon-sidebar" data-pagefind-ignore class="sidebar-closed md:block w-60 flex-shrink-0 border-r border-gray-200 dark:border-gray-700 pr-4 mr-4 sticky top-16 self-start max-h-[calc(100vh-4rem)] overflow-y-auto">
  <nav class="sidebar" aria-label="Site navigation">
    <ul class="space-y-1">
{items_html}
    </ul>
  </nav>
</aside>
<div id="tachyon-sidebar-overlay" class="hidden fixed inset-0 bg-black/50 z-40 md:hidden"></div>
<script>
document.addEventListener('DOMContentLoaded', function() {{
  var toggle = document.getElementById('tachyon-sidebar-toggle');
  var sidebar = document.getElementById('tachyon-sidebar');
  var overlay = document.getElementById('tachyon-sidebar-overlay');
  function openSidebar() {{
    sidebar.classList.remove('sidebar-closed');
    sidebar.classList.add('sidebar-open');
    overlay.classList.remove('hidden');
    toggle.innerHTML = '&times;';
    toggle.setAttribute('aria-expanded', 'true');
  }}
  function closeSidebar() {{
    sidebar.classList.remove('sidebar-open');
    sidebar.classList.add('sidebar-closed');
    overlay.classList.add('hidden');
    toggle.innerHTML = '&#9776;';
    toggle.setAttribute('aria-expanded', 'false');
  }}
  if (toggle && sidebar && overlay) {{
    toggle.addEventListener('click', function() {{
      if (sidebar.classList.contains('sidebar-open')) {{ closeSidebar(); }} else {{ openSidebar(); }}
    }});
    overlay.addEventListener('click', closeSidebar);
    sidebar.querySelectorAll('a').forEach(function(link) {{
      link.addEventListener('click', closeSidebar);
    }});
  }}
}})();
</script>"#,
        items_html = items_html
    )
}

fn render_sidebar_item(
    item: &SidebarItem,
    current_slug: Option<&str>,
    depth: usize,
    root_prefix: &str,
) -> String {
    let is_active = current_slug
        .map(|slug| {
            let item_slug = item.href.trim_start_matches('/').trim_end_matches(".html");
            slug == item_slug
        })
        .unwrap_or(false);
    let active_class = if is_active {
        " bg-blue-50 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300 font-medium"
    } else {
        " text-gray-700 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-gray-800"
    };
    let indent_style = format!("padding-left: {}rem;", depth as f32 * 0.75);
    let href = if item.href.starts_with("http") || item.href.starts_with('#') {
        item.href.clone()
    } else {
        format!("{}{}", root_prefix, item.href.trim_start_matches('/'))
    };
    let link = format!(
        r#"<li><a href="{}" class="block text-sm rounded px-2 py-1.5{}" style="{}">{}</a></li>"#,
        escape_html(&href),
        active_class,
        indent_style,
        escape_html(&item.label),
    );
    if item.children.is_empty() {
        link
    } else {
        let children_html: String = item
            .children
            .iter()
            .map(|child| render_sidebar_item(child, current_slug, depth + 1, root_prefix))
            .collect::<Vec<_>>()
            .join("\n");
        format!(
            r#"{link}
 <details class="ml-2">
   <summary class="text-xs text-gray-500 dark:text-gray-400 cursor-pointer py-1 select-none">{label}</summary>
   <ul class="space-y-1 mt-1">
{children_html}
   </ul>
 </details>"#,
            link = link,
            label = escape_html(&item.label),
            children_html = children_html,
        )
    }
}

/// Render prev/next page navigation links.
fn render_prev_next(prev: Option<&(String, String)>, next: Option<&(String, String)>) -> String {
    let prev_html = match prev {
        Some((title, href)) => format!(
            r#"<a href="{}" class="flex-1 text-left px-4 py-3 rounded-lg border border-gray-200 dark:border-gray-700 hover:border-blue-300 dark:hover:border-blue-600 hover:shadow-sm transition-all">
  <span class="text-xs text-gray-500 dark:text-gray-400 block">Previous</span>
  <span class="text-sm font-medium text-gray-900 dark:text-gray-100">{}</span>
</a>"#,
            href,
            escape_html(title)
        ),
        None => String::new(),
    };
    let next_html = match next {
        Some((title, href)) => format!(
            r#"<a href="{}" class="flex-1 text-right px-4 py-3 rounded-lg border border-gray-200 dark:border-gray-700 hover:border-blue-300 dark:hover:border-blue-600 hover:shadow-sm transition-all">
  <span class="text-xs text-gray-500 dark:text-gray-400 block">Next</span>
  <span class="text-sm font-medium text-gray-900 dark:text-gray-100">{}</span>
</a>"#,
            href,
            escape_html(title)
        ),
        None => String::new(),
    };
    if prev_html.is_empty() && next_html.is_empty() {
        return String::new();
    }
    format!(r#"<div class="flex gap-4 mt-8 mb-4">{prev_html}{next_html}</div>"#)
}

fn render_footer(site: &SiteConfig, _root_prefix: &str) -> String {
    format!(
        r#"<footer data-pagefind-ignore class="border-t border-gray-200 dark:border-gray-700 py-6 mt-12">
  <div class="max-w-6xl mx-auto px-4 text-center text-sm text-gray-500 dark:text-gray-400">
    {footer}
  </div>
</footer>"#,
        footer = escape_html(&site.footer),
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
