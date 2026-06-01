//! Single-page HTML export — portable document bundle.

use crate::render::render_markdown;

pub fn export_single_page_html(
    title: &str,
    content: &str,
    _slug: &str,
    options: &SinglePageOptions,
) -> String {
    let rendered = render_markdown(content);
    let css = inline_css();
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{title}</title>
<meta name="generator" content="Tachyon Single-Page Export">
<style>{css}</style>
</head>
<body>
<article class="tachyon-doc">
<header class="doc-header">
<h1>{title}</h1>
{byline}
</header>
<main class="doc-content">
{rendered}
</main>
<footer class="doc-footer">
<p>Exported from Tachyon Knowledge Base</p>
</footer>
</article>
<script>
document.documentElement.dataset.theme = 'light';
document.querySelector('.theme-toggle')?.addEventListener('click', () => {{
    const current = document.documentElement.dataset.theme;
    document.documentElement.dataset.theme = current === 'dark' ? 'light' : 'dark';
}});
</script>
</body>
</html>"#,
        title = html_escape(title),
        rendered = rendered,
        css = css,
        byline = if let Some(author) = &options.author {
            format!("<p class=\"byline\">By {}</p>", html_escape(author))
        } else {
            String::new()
        },
    )
}

#[derive(Debug, Clone, Default)]
pub struct SinglePageOptions {
    pub author: Option<String>,
    pub include_toc: bool,
    pub dark_mode: bool,
}

fn inline_css() -> String {
    r#"
:root { --bg: #fff; --text: #1a1a1a; --link: #0066cc; --code-bg: #f4f4f4; --border: #ddd; }
@media (prefers-color-scheme: dark) {
    :root { --bg: #1a1a1a; --text: #e0e0e0; --link: #4da6ff; --code-bg: #2a2a2a; --border: #444; }
}
body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; max-width: 800px; margin: 0 auto; padding: 2rem; background: var(--bg); color: var(--text); line-height: 1.6; }
h1, h2, h3 { color: var(--text); }
a { color: var(--link); }
code { background: var(--code-bg); padding: 0.2em 0.4em; border-radius: 3px; font-size: 0.9em; }
pre { background: var(--code-bg); padding: 1rem; border-radius: 6px; overflow-x: auto; border: 1px solid var(--border); }
pre code { padding: 0; background: none; }
blockquote { border-left: 4px solid var(--border); padding-left: 1rem; margin-left: 0; color: #666; }
table { border-collapse: collapse; width: 100%; }
th, td { border: 1px solid var(--border); padding: 0.5rem 1rem; text-align: left; }
th { background: var(--code-bg); }
img { max-width: 100%; height: auto; }
.doc-header { border-bottom: 2px solid var(--border); padding-bottom: 1rem; margin-bottom: 2rem; }
.doc-footer { border-top: 1px solid var(--border); padding-top: 1rem; margin-top: 3rem; color: #999; font-size: 0.9em; }
.byline { color: #666; font-style: italic; }
"#.to_string()
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_page_export_basic() {
        let html = export_single_page_html(
            "Test Document",
            "# Hello\n\nThis is **bold** text.",
            "test",
            &SinglePageOptions::default(),
        );
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("Test Document"));
        assert!(html.contains("<strong>bold</strong>"));
        assert!(html.contains("tachyon-doc"));
    }

    #[test]
    fn test_single_page_with_author() {
        let html = export_single_page_html(
            "Doc",
            "Content",
            "doc",
            &SinglePageOptions {
                author: Some("Alice".to_string()),
                ..Default::default()
            },
        );
        assert!(html.contains("By Alice"));
    }

    #[test]
    fn test_html_escape_in_title() {
        let html = export_single_page_html(
            "Test <script>alert('xss')</script>",
            "Content",
            "test",
            &SinglePageOptions::default(),
        );
        assert!(html.contains("&lt;script&gt;"));
        assert!(!html.contains("<title>Test <script>"));
    }

    #[test]
    fn test_single_page_has_inline_css() {
        let html = export_single_page_html("T", "C", "t", &SinglePageOptions::default());
        assert!(html.contains(":root"));
        assert!(html.contains("--bg:"));
    }

    #[test]
    fn test_html_escape_function() {
        assert_eq!(html_escape("a&b"), "a&amp;b");
        assert_eq!(html_escape("<tag>"), "&lt;tag&gt;");
    }
}
