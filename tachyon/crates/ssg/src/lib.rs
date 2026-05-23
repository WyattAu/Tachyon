//! Tachyon SSG — Static Site Generator
//!
//! Generates a complete static site from Tachyon documents.

#![allow(dead_code)]
//! Produces HTML pages with navigation, index, sitemap, and RSS feed.
//! Designed to compete with Docusaurus as the best SSG for markdown content.

pub mod assets;
mod build;
mod build_cache;
mod error;
pub mod i18n;
mod manifest;
mod render;
mod rss;
mod sitemap;
mod templates;

pub use build::SiteGenerator;
pub use error::{SsgError, SsgResult};
pub use i18n::{language_display_name, text_direction};
pub use manifest::{
    BuildResult, ColorTheme, NavLink, SidebarItem, SiteConfig, SsgDocument, TranslationConfig,
};
pub use templates::{DEFAULT_BASE_TEMPLATE, DEFAULT_DOC_TEMPLATE, DEFAULT_INDEX_TEMPLATE};

pub mod slug {
    pub fn slugify(s: &str) -> String {
        s.to_lowercase()
            .chars()
            .map(|ch| {
                if ch.is_alphanumeric() || ch == '-' || ch == '_' {
                    ch
                } else if ch.is_whitespace() {
                    '-'
                } else {
                    '\0'
                }
            })
            .filter(|ch| *ch != '\0')
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::{escape_xml, html_to_plain, truncate_text};
    use chrono::Utc;

    fn sample_documents() -> Vec<SsgDocument> {
        vec![
            SsgDocument {
                slug: "getting-started".to_string(),
                title: "Getting Started".to_string(),
                content: "# Getting Started\n\nWelcome to Tachyon!".to_string(),
                description: Some("Learn how to use Tachyon".to_string()),
                author: Some("Tachyon Team".to_string()),
                tags: vec!["guide".to_string(), "beginner".to_string()],
                created_at: Utc::now(),
                updated_at: Utc::now(),
                order: 0,
                language: "en".to_string(),
            },
            SsgDocument {
                slug: "configuration".to_string(),
                title: "Configuration".to_string(),
                content: "# Configuration\n\nSet up your Tachyon instance.".to_string(),
                description: None,
                author: None,
                tags: vec!["guide".to_string()],
                created_at: Utc::now(),
                updated_at: Utc::now(),
                order: 1,
                language: "en".to_string(),
            },
        ]
    }

    #[test]
    fn test_site_config_default() {
        let config = SiteConfig::default();
        assert_eq!(config.title, "Tachyon Docs");
        assert_eq!(config.theme, "auto");
        assert!(!config.group_by_tag);
    }

    #[test]
    fn test_ssg_build_to_dir() {
        let config = SiteConfig::default();
        let generator = SiteGenerator::new(config);
        let docs = sample_documents();

        let tmp = std::env::temp_dir().join("tachyon-ssg-test");
        let _ = std::fs::remove_dir_all(&tmp);

        let result = generator.build_to_dir(&docs, &tmp).unwrap();

        assert_eq!(result.pages, 2);
        assert_eq!(result.total_files, 6);
        assert!(result.build_time_ms > 0);
        assert!(result.output_size_bytes > 0);
        assert_eq!(result.languages, 1);

        assert!(tmp.join("index.html").exists());
        assert!(tmp.join("getting-started.html").exists());
        assert!(tmp.join("configuration.html").exists());
        assert!(tmp.join("sitemap.xml").exists());
        assert!(tmp.join("feed.xml").exists());
        assert!(tmp.join("robots.txt").exists());

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_ssg_build_to_zip() {
        let config = SiteConfig::default();
        let generator = SiteGenerator::new(config);
        let docs = sample_documents();

        let (zip_bytes, result) = generator.build_to_zip(&docs).unwrap();

        assert!(!zip_bytes.is_empty());
        assert_eq!(result.pages, 2);
        assert_eq!(result.languages, 1);
    }

    #[test]
    fn test_ssg_with_tag_grouping() {
        let config = SiteConfig {
            group_by_tag: true,
            ..Default::default()
        };

        let generator = SiteGenerator::new(config);
        let docs = sample_documents();

        let tmp = std::env::temp_dir().join("tachyon-ssg-tag-test");
        let _ = std::fs::remove_dir_all(&tmp);

        let result = generator.build_to_dir(&docs, &tmp).unwrap();

        assert_eq!(result.category_pages, 2);
        assert!(tmp.join("category-guide.html").exists());
        assert!(tmp.join("category-beginner.html").exists());

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_sitemap_contains_all_docs() {
        let config = SiteConfig {
            base_url: "https://docs.example.com".to_string(),
            ..Default::default()
        };
        let generator = SiteGenerator::new(config);
        let docs = sample_documents();

        let tmp = std::env::temp_dir().join("tachyon-ssg-sitemap-test");
        let _ = std::fs::remove_dir_all(&tmp);
        generator.build_to_dir(&docs, &tmp).unwrap();

        let sitemap = std::fs::read_to_string(tmp.join("sitemap.xml")).unwrap();
        assert!(sitemap.contains("getting-started.html"));
        assert!(sitemap.contains("configuration.html"));
        assert!(sitemap.contains("index.html"));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_rss_feed() {
        let config = SiteConfig::default();
        let generator = SiteGenerator::new(config);
        let docs = sample_documents();

        let tmp = std::env::temp_dir().join("tachyon-ssg-rss-test");
        let _ = std::fs::remove_dir_all(&tmp);
        generator.build_to_dir(&docs, &tmp).unwrap();

        let rss = std::fs::read_to_string(tmp.join("feed.xml")).unwrap();
        assert!(rss.contains("<rss"));
        assert!(rss.contains("Getting Started"));
        assert!(rss.contains("Configuration"));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_slugify() {
        assert_eq!(slug::slugify("Hello World"), "hello-world");
        assert_eq!(slug::slugify("Rust & C++"), "rust-c");
        assert_eq!(slug::slugify("  multiple   spaces  "), "multiple-spaces");
    }

    #[test]
    fn test_html_to_plain() {
        assert_eq!(html_to_plain("<h1>Hello</h1><p>World</p>"), "Hello World");
        assert_eq!(
            html_to_plain("<b>bold</b> and <i>italic</i>"),
            "bold and italic"
        );
    }

    #[test]
    fn test_truncate_text() {
        assert_eq!(truncate_text("Hello World", 20), "Hello World");
        assert!(truncate_text("A very long string that goes on and on", 15).ends_with("..."));
    }

    #[test]
    fn test_escape_xml() {
        assert_eq!(
            escape_xml("a&b<c>d\"e'f"),
            "a&amp;b&lt;c&gt;d&quot;e&apos;f"
        );
    }

    #[test]
    fn test_ssg_multi_language_build() {
        let config = SiteConfig {
            language: "en".to_string(),
            translations: vec![TranslationConfig {
                language: "zh".to_string(),
                name: "中文".to_string(),
                base_url: "https://docs.example.com/zh".to_string(),
            }],
            ..Default::default()
        };
        let generator = SiteGenerator::new(config);

        let docs = vec![
            SsgDocument {
                slug: "getting-started".to_string(),
                title: "Getting Started".to_string(),
                content: "# Getting Started\n\nWelcome to Tachyon!".to_string(),
                description: Some("Learn how to use Tachyon".to_string()),
                author: Some("Tachyon Team".to_string()),
                tags: vec!["guide".to_string()],
                created_at: Utc::now(),
                updated_at: Utc::now(),
                order: 0,
                language: "en".to_string(),
            },
            SsgDocument {
                slug: "getting-started".to_string(),
                title: "快速入门".to_string(),
                content: "# 快速入门\n\n欢迎使用 Tachyon！".to_string(),
                description: Some("了解如何使用 Tachyon".to_string()),
                author: Some("Tachyon 团队".to_string()),
                tags: vec!["指南".to_string()],
                created_at: Utc::now(),
                updated_at: Utc::now(),
                order: 0,
                language: "zh".to_string(),
            },
        ];

        let tmp = std::env::temp_dir().join("tachyon-ssg-i18n-test");
        let _ = std::fs::remove_dir_all(&tmp);

        let result = generator.build_to_dir(&docs, &tmp).unwrap();

        assert_eq!(result.languages, 2);
        assert_eq!(result.pages, 2);

        let root_index = std::fs::read_to_string(tmp.join("index.html")).unwrap();
        assert!(root_index.contains("en/"));

        assert!(tmp.join("en/index.html").exists());
        assert!(tmp.join("en/getting-started.html").exists());
        let en_html = std::fs::read_to_string(tmp.join("en/getting-started.html")).unwrap();
        assert!(en_html.contains("Getting Started"));
        assert!(en_html.contains(r#"lang="en""#));
        assert!(en_html.contains(r#"dir="ltr""#));
        assert!(en_html.contains("English"));
        assert!(en_html.contains("中文"));

        assert!(tmp.join("zh/index.html").exists());
        assert!(tmp.join("zh/getting-started.html").exists());
        let zh_html = std::fs::read_to_string(tmp.join("zh/getting-started.html")).unwrap();
        assert!(zh_html.contains("快速入门"));
        assert!(zh_html.contains(r#"lang="zh""#));

        assert!(tmp.join("en/sitemap.xml").exists());
        assert!(tmp.join("en/feed.xml").exists());
        assert!(tmp.join("zh/sitemap.xml").exists());
        assert!(tmp.join("zh/feed.xml").exists());

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_admonition_note() {
        use crate::render::render_admonitions;
        let input = r#"<blockquote>
<p>[!NOTE]</p>
<p>This is a note</p>
</blockquote>"#;
        let output = render_admonitions(input);
        assert!(output.contains(r#"<div class="admonition admonition-note">"#));
        assert!(output.contains(r#"<p class="admonition-title">Note</p>"#));
        assert!(output.contains("This is a note"));
        assert!(!output.contains("<blockquote>"));
    }

    #[test]
    fn test_admonition_warning() {
        use crate::render::render_admonitions;
        let input = r#"<blockquote>
<p>[!WARNING]</p>
<p>Be careful!</p>
</blockquote>"#;
        let output = render_admonitions(input);
        assert!(output.contains(r#"admonition-warning"#));
        assert!(output.contains(r#"admonition-title">Warning"#));
        assert!(output.contains("Be careful!"));
    }

    #[test]
    fn test_admonition_tip() {
        use crate::render::render_admonitions;
        let input = r#"<blockquote>
<p>[!TIP]</p>
<p>Try this approach</p>
</blockquote>"#;
        let output = render_admonitions(input);
        assert!(output.contains(r#"admonition-tip"#));
        assert!(output.contains(r#"admonition-title">Tip"#));
    }

    #[test]
    fn test_admonition_danger() {
        use crate::render::render_admonitions;
        let input = r#"<blockquote>
<p>[!DANGER]</p>
<p>This will delete everything</p>
</blockquote>"#;
        let output = render_admonitions(input);
        assert!(output.contains(r#"admonition-danger"#));
        assert!(output.contains(r#"admonition-title">Danger"#));
    }

    #[test]
    fn test_admonition_info_and_success() {
        use crate::render::render_admonitions;
        let info_input = r#"<blockquote>
<p>[!INFO]</p>
<p>Some info</p>
</blockquote>"#;
        let output = render_admonitions(info_input);
        assert!(output.contains(r#"admonition-info"#));
        assert!(output.contains(r#"admonition-title">Info"#));

        let success_input = r#"<blockquote>
<p>[!SUCCESS]</p>
<p>It worked</p>
</blockquote>"#;
        let output = render_admonitions(success_input);
        assert!(output.contains(r#"admonition-success"#));
        assert!(output.contains(r#"admonition-title">Success"#));
    }

    #[test]
    fn test_admonition_no_match_plain_blockquote() {
        use crate::render::render_admonitions;
        let input = r#"<blockquote>
<p>Just a regular quote</p>
</blockquote>"#;
        let output = render_admonitions(input);
        assert!(output.contains("<blockquote>"));
        assert!(!output.contains("admonition"));
    }

    #[test]
    fn test_sidebar_generation() {
        use crate::templates::render_sidebar_test;
        let items = vec![
            SidebarItem {
                label: "Getting Started".to_string(),
                href: "getting-started.html".to_string(),
                children: vec![],
            },
            SidebarItem {
                label: "Guides".to_string(),
                href: "#".to_string(),
                children: vec![SidebarItem {
                    label: "Configuration".to_string(),
                    href: "configuration.html".to_string(),
                    children: vec![],
                }],
            },
        ];
        let html = render_sidebar_test(&items, Some("getting-started"));
        assert!(html.contains("Getting Started"));
        assert!(html.contains("getting-started.html"));
        assert!(html.contains("bg-blue-50"));
        assert!(html.contains("Guides"));
        assert!(html.contains("Configuration"));
        assert!(html.contains(r#"id="tachyon-sidebar""#));
    }

    #[test]
    fn test_sidebar_empty() {
        use crate::templates::render_sidebar_test;
        let html = render_sidebar_test(&[], Some("test"));
        assert!(html.is_empty());
    }

    #[test]
    fn test_toc_integration() {
        use crate::render::extract_toc;
        let html = r#"<h1 id="intro">Introduction</h1><p>Some text</p><h2 id="details">Details</h2><p>More text</p>"#;
        let toc = extract_toc(html);
        assert_eq!(toc.len(), 2);
        assert_eq!(toc[0].id, "intro");
        assert_eq!(toc[0].title, "Introduction");
        assert_eq!(toc[0].level, 1);
        assert_eq!(toc[1].id, "details");
        assert_eq!(toc[1].level, 2);
    }

    #[test]
    fn test_latex_cdn_in_output() {
        let config = SiteConfig::default();
        let generator = SiteGenerator::new(config);
        let docs = vec![SsgDocument {
            slug: "test-latex".to_string(),
            title: "LaTeX Test".to_string(),
            content: "# LaTeX Test\n\nSome content".to_string(),
            description: None,
            author: None,
            tags: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            order: 0,
            language: "en".to_string(),
        }];

        let tmp = std::env::temp_dir().join("tachyon-ssg-katex-test");
        let _ = std::fs::remove_dir_all(&tmp);
        generator.build_to_dir(&docs, &tmp).unwrap();

        let html = std::fs::read_to_string(tmp.join("test-latex.html")).unwrap();
        assert!(html.contains("katex.min.css"));
        assert!(html.contains("katex.min.js"));
        assert!(html.contains("auto-render.min.js"));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_render_markdown_with_code_blocks() {
        use crate::render::render_markdown;
        let deployment_content = r#"# Deployment

## Docker

```bash
docker build -t tachyon .
docker run -d -p 8080:8080 tachyon
```

### Docker Compose

```yaml
services:
  db:
    image: postgres:16
  app:
    build: .
```

## Static Site

```bash
cargo build --release -p tachyon-ssg
./target/release/tachyon-ssg-cli build --input ./docs --output ./site
```

## Nginx

```nginx
server {
    listen 80;
    server_name docs.example.com;
    location / {
        proxy_pass http://127.0.0.1:8080;
    }
}
```

## TLS

```bash
sudo certbot --nginx -d docs.example.com
```
"#;
        let html = render_markdown(deployment_content);
        eprintln!("=== RENDERED MARKDOWN ({} bytes) ===\n{}", html.len(), html);
        assert!(html.contains("Docker"), "Docker section missing");
        assert!(
            html.contains("Docker Compose"),
            "Docker Compose section missing"
        );
        assert!(html.contains("Static Site"), "Static Site section missing");
        assert!(html.contains("Nginx"), "Nginx section missing");
        assert!(html.contains("TLS"), "TLS section missing");
        assert!(
            html.contains("docker build"),
            "docker build command missing"
        );
        assert!(html.contains("listen 80"), "nginx content missing");
        assert!(html.contains("certbot"), "certbot command missing");
    }

    #[test]
    fn test_admonition_css_in_output() {
        let config = SiteConfig::default();
        let generator = SiteGenerator::new(config);
        let docs = vec![SsgDocument {
            slug: "test-admonition-css".to_string(),
            title: "Admonition CSS Test".to_string(),
            content: "# Test\n\nContent".to_string(),
            description: None,
            author: None,
            tags: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            order: 0,
            language: "en".to_string(),
        }];

        let tmp = std::env::temp_dir().join("tachyon-ssg-adm-css-test");
        let _ = std::fs::remove_dir_all(&tmp);
        generator.build_to_dir(&docs, &tmp).unwrap();

        let html = std::fs::read_to_string(tmp.join("test-admonition-css.html")).unwrap();
        assert!(html.contains("admonition-note"));
        assert!(html.contains("admonition-warning"));
        assert!(html.contains("admonition-tip"));
        assert!(html.contains("admonition-danger"));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_add_heading_ids() {
        use crate::render::add_heading_ids;
        let html = r#"<h2>First Section</h2><p>text</p><h3>Sub Section</h3>"#;
        let result = add_heading_ids(html);
        assert!(result.contains(r#"<h2 id="first-section">"#));
        assert!(result.contains(r#"<h3 id="sub-section">"#));
    }

    #[test]
    fn test_add_heading_ids_preserves_existing() {
        use crate::render::add_heading_ids;
        let html = r#"<h2 id="custom-id">Title</h2>"#;
        let result = add_heading_ids(html);
        assert!(result.contains(r#"<h2 id="custom-id">"#));
        assert!(!result.contains("custom-id-"));
    }

    #[test]
    fn test_add_heading_ids_deduplicates() {
        use crate::render::add_heading_ids;
        let html = r#"<h2>Same</h2><h2>Same</h2>"#;
        let result = add_heading_ids(html);
        assert!(result.contains(r#"<h2 id="same">"#));
        assert!(result.contains(r#"<h2 id="same-2">"#));
    }

    #[test]
    fn test_inline_toc_two_headings() {
        use crate::render::{add_heading_ids, extract_inline_toc, render_inline_toc};
        let html = r#"<h2>Alpha</h2><p>text</p><h3>Beta</h3><p>more</p>"#;
        let html = add_heading_ids(html);
        let toc = extract_inline_toc(&html);
        assert_eq!(toc.len(), 2);
        let rendered = render_inline_toc(&toc);
        assert!(rendered.contains(r#"<nav class="toc">"#));
        assert!(rendered.contains(r#"#alpha"#));
        assert!(rendered.contains(r#"#beta"#));
        assert!(rendered.contains(r#"toc-h2"#));
        assert!(rendered.contains(r#"toc-h3"#));
    }

    #[test]
    fn test_inline_toc_hidden_when_single_heading() {
        use crate::render::{add_heading_ids, extract_inline_toc, render_inline_toc};
        let html = r#"<h2>Only One</h2><p>text</p>"#;
        let html = add_heading_ids(html);
        let toc = extract_inline_toc(&html);
        assert_eq!(toc.len(), 1);
        let rendered = render_inline_toc(&toc);
        assert!(rendered.is_empty());
    }

    #[test]
    fn test_inline_toc_hidden_when_no_headings() {
        use crate::render::render_inline_toc;
        let rendered = render_inline_toc(&[]);
        assert!(rendered.is_empty());
    }

    #[test]
    fn test_add_copy_buttons() {
        use crate::render::add_copy_buttons;
        let html = r#"<pre><code class="language-bash">echo hello
</code></pre>"#;
        let result = add_copy_buttons(html);
        assert!(result.contains(r#"class="code-block-wrapper""#));
        assert!(result.contains(r#"class="code-copy-btn""#));
        assert!(result.contains("navigator.clipboard.writeText"));
        assert!(result.contains("Copied!"));
        assert!(result.contains(r#"class="language-bash""#));
        assert!(result.contains("echo hello"));
    }

    #[test]
    fn test_add_copy_buttons_multiple_blocks() {
        use crate::render::add_copy_buttons;
        let html = r#"<pre><code>block1</code></pre><p>text</p><pre><code>block2</code></pre>"#;
        let result = add_copy_buttons(html);
        assert_eq!(result.matches("code-block-wrapper").count(), 2);
        assert_eq!(result.matches("code-copy-btn").count(), 2);
    }

    #[test]
    fn test_render_markdown_includes_toc_and_copy() {
        use crate::render::render_markdown;
        let md = r#"# Title

## Section One

Some text with `code`.

```rust
fn main() {}
```

## Section Two

More content.

### Subsection

Details.
"#;
        let html = render_markdown(md);
        assert!(
            html.contains(r#"<nav class="toc">"#),
            "inline TOC should be present"
        );
        assert!(
            html.contains(r#"#section-one"#),
            "TOC should link to section-one"
        );
        assert!(
            html.contains(r#"#section-two"#),
            "TOC should link to section-two"
        );
        assert!(
            html.contains(r#"#subsection"#),
            "TOC should link to subsection"
        );
        assert!(
            html.contains(r#"class="code-copy-btn""#),
            "copy button should be present"
        );
    }

    #[test]
    fn test_render_markdown_no_toc_for_single_heading() {
        use crate::render::render_markdown;
        let md = "# Title\n\n## Only Section\n\nSome text.\n";
        let html = render_markdown(md);
        assert!(
            !html.contains(r#"<nav class="toc">"#),
            "TOC should not appear with single h2/h3"
        );
    }

    #[test]
    fn test_breadcrumbs_structured_data() {
        let config = SiteConfig::default();
        let generator = SiteGenerator::new(config);
        let docs = vec![SsgDocument {
            slug: "docs/guides/advanced".to_string(),
            title: "Advanced Guide".to_string(),
            content: "# Advanced Guide\n\nContent.".to_string(),
            description: None,
            author: None,
            tags: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            order: 0,
            language: "en".to_string(),
        }];

        let tmp = std::env::temp_dir().join("tachyon-ssg-breadcrumb-test");
        let _ = std::fs::remove_dir_all(&tmp);
        generator.build_to_dir(&docs, &tmp).unwrap();

        let html =
            std::fs::read_to_string(tmp.join("docs").join("guides").join("advanced.html")).unwrap();
        assert!(
            html.contains(r#"itemscope itemtype="https://schema.org/BreadcrumbList""#),
            "should have BreadcrumbList schema"
        );
        assert!(
            html.contains(r#"itemprop="itemListElement""#),
            "should have itemListElement"
        );
        assert!(
            html.contains(r#"class="breadcrumbs""#),
            "should have breadcrumbs class"
        );
        assert!(
            html.contains(r#"aria-current="page""#),
            "last breadcrumb should have aria-current"
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_pagefind_attributes_in_output() {
        let config = SiteConfig::default();
        let generator = SiteGenerator::new(config);
        let docs = vec![SsgDocument {
            slug: "test-pagefind".to_string(),
            title: "Pagefind Test".to_string(),
            content: "# Pagefind Test\n\nContent.".to_string(),
            description: None,
            author: None,
            tags: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            order: 0,
            language: "en".to_string(),
        }];

        let tmp = std::env::temp_dir().join("tachyon-ssg-pagefind-test");
        let _ = std::fs::remove_dir_all(&tmp);
        generator.build_to_dir(&docs, &tmp).unwrap();

        let html = std::fs::read_to_string(tmp.join("test-pagefind.html")).unwrap();
        assert!(
            html.contains(r#"data-pagefind-body"#),
            "should have data-pagefind-body on content"
        );
        assert!(
            html.contains(r#"data-pagefind-ignore"#),
            "should have data-pagefind-ignore on nav/sidebar/footer"
        );
        assert!(html.contains(r#"id="search""#), "should have search div");
        assert!(
            html.contains("pagefind-ui.css"),
            "should include pagefind CSS"
        );
        assert!(
            html.contains("pagefind-ui.js"),
            "should include pagefind JS"
        );
        assert!(
            html.contains("npx pagefind"),
            "should have pagefind build comment"
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_pagefind_disabled() {
        let config = SiteConfig {
            pagefind_enabled: false,
            ..Default::default()
        };
        let generator = SiteGenerator::new(config);
        let docs = vec![SsgDocument {
            slug: "no-pagefind".to_string(),
            title: "No Pagefind".to_string(),
            content: "# No Pagefind\n\nContent.".to_string(),
            description: None,
            author: None,
            tags: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            order: 0,
            language: "en".to_string(),
        }];

        let tmp = std::env::temp_dir().join("tachyon-ssg-no-pagefind-test");
        let _ = std::fs::remove_dir_all(&tmp);
        generator.build_to_dir(&docs, &tmp).unwrap();

        let html = std::fs::read_to_string(tmp.join("no-pagefind.html")).unwrap();
        assert!(
            !html.contains("pagefind-ui.css"),
            "should not include pagefind CSS when disabled"
        );
        assert!(
            !html.contains("pagefind-ui.js"),
            "should not include pagefind JS when disabled"
        );
        assert!(
            !html.contains(r#"id="search""#),
            "should not have search div when disabled"
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_mermaid_blocks() {
        use crate::render::process_mermaid_blocks;
        let input = r#"<pre><code class="language-mermaid">graph TD
  A --&gt; B
  B --&gt; C
</code></pre>"#;
        let output = process_mermaid_blocks(input);
        assert!(
            output.contains(r#"<div class="mermaid">"#),
            "mermaid block should become div: {}",
            output
        );
        assert!(
            output.contains("graph TD"),
            "mermaid content should be preserved"
        );
        assert!(
            output.contains("A --> B"),
            "HTML entities should be unescaped in mermaid content"
        );
        assert!(
            !output.contains("<pre>"),
            "pre tag should be removed for mermaid blocks"
        );
    }

    #[test]
    fn test_mermaid_blocks_non_mermaid_untouched() {
        use crate::render::process_mermaid_blocks;
        let input = r#"<pre><code class="language-bash">echo hello</code></pre>"#;
        let output = process_mermaid_blocks(input);
        assert!(
            output.contains("<pre>"),
            "non-mermaid blocks should be untouched"
        );
        assert!(
            output.contains("language-bash"),
            "non-mermaid language class should be preserved"
        );
    }

    #[test]
    fn test_mermaid_blocks_mixed() {
        use crate::render::process_mermaid_blocks;
        let input = r#"<pre><code class="language-bash">echo hello</code></pre>
<p>text</p>
<pre><code class="language-mermaid">graph TD
  A --&gt; B
</code></pre>
<pre><code class="language-rust">fn main() {}</code></pre>"#;
        let output = process_mermaid_blocks(input);
        assert!(output.contains(r#"<div class="mermaid">"#));
        assert!(output.contains("language-bash"));
        assert!(output.contains("language-rust"));
        assert!(output.contains("echo hello"));
        assert!(output.contains("fn main() {}"));
    }

    #[test]
    fn test_mermaid_in_rendered_output() {
        let config = SiteConfig::default();
        let generator = SiteGenerator::new(config);
        let docs = vec![SsgDocument {
            slug: "test-mermaid".to_string(),
            title: "Mermaid Test".to_string(),
            content: "# Mermaid Test\n\n```mermaid\ngraph TD\n  A-->B\n```\n".to_string(),
            description: None,
            author: None,
            tags: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            order: 0,
            language: "en".to_string(),
        }];

        let tmp = std::env::temp_dir().join("tachyon-ssg-mermaid-test");
        let _ = std::fs::remove_dir_all(&tmp);
        generator.build_to_dir(&docs, &tmp).unwrap();

        let html = std::fs::read_to_string(tmp.join("test-mermaid.html")).unwrap();
        assert!(
            html.contains(r#"<div class="mermaid">"#),
            "mermaid div should be in output"
        );
        assert!(
            html.contains("mermaid.min.js"),
            "mermaid JS should be included"
        );
        assert!(
            html.contains("mermaid.initialize"),
            "mermaid init should be included"
        );
        assert!(
            html.contains("graph TD"),
            "mermaid content should be in output"
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_mermaid_disabled() {
        let config = SiteConfig {
            mermaid_enabled: false,
            ..Default::default()
        };
        let generator = SiteGenerator::new(config);
        let docs = vec![SsgDocument {
            slug: "no-mermaid".to_string(),
            title: "No Mermaid".to_string(),
            content: "# No Mermaid\n\n```mermaid\ngraph TD\n  A-->B\n```\n".to_string(),
            description: None,
            author: None,
            tags: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            order: 0,
            language: "en".to_string(),
        }];

        let tmp = std::env::temp_dir().join("tachyon-ssg-no-mermaid-test");
        let _ = std::fs::remove_dir_all(&tmp);
        generator.build_to_dir(&docs, &tmp).unwrap();

        let html = std::fs::read_to_string(tmp.join("no-mermaid.html")).unwrap();
        assert!(
            !html.contains("mermaid.min.js"),
            "mermaid JS should not be included when disabled"
        );
        assert!(
            !html.contains("mermaid.initialize"),
            "mermaid init should not be included when disabled"
        );
        assert!(
            html.contains(r#"<div class="mermaid">"#),
            "mermaid blocks should still be converted to divs"
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_mermaid_not_wrapped_by_copy_button() {
        use crate::render::render_markdown;
        let md = "# Test\n\n```mermaid\ngraph TD\n  A-->B\n```\n\n```bash\necho hello\n```\n";
        let html = render_markdown(md);
        let mermaid_pos = html.find(r#"<div class="mermaid">"#);
        let copy_pos = html.find(r#"class="code-copy-btn""#);
        assert!(mermaid_pos.is_some(), "mermaid div should exist");
        assert!(
            copy_pos.is_some(),
            "copy button should exist for non-mermaid blocks"
        );
        if let (Some(m), Some(c)) = (mermaid_pos, copy_pos) {
            let mermaid_end = html[m..].find("</div>").map(|i| m + i).unwrap_or(m);
            assert!(
                c > mermaid_end || c < m,
                "copy button should not be inside mermaid block"
            );
        }
    }

    #[test]
    fn test_syntax_highlighting_enabled_by_default() {
        let config = SiteConfig::default();
        assert!(
            config.syntax_highlighting_enabled,
            "syntax_highlighting_enabled should default to true"
        );
    }

    #[test]
    fn test_syntax_highlighting_cdn_in_output() {
        let config = SiteConfig::default();
        let generator = SiteGenerator::new(config);
        let docs = vec![SsgDocument {
            slug: "code-example".to_string(),
            title: "Code Example".to_string(),
            content: "# Code\n\n```rust\nfn main() {}\n```\n".to_string(),
            description: None,
            author: None,
            tags: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            order: 0,
            language: "en".to_string(),
        }];

        let tmp = std::env::temp_dir().join("tachyon-ssg-highlight-test");
        let _ = std::fs::remove_dir_all(&tmp);
        generator.build_to_dir(&docs, &tmp).unwrap();

        let html = std::fs::read_to_string(tmp.join("code-example.html")).unwrap();
        assert!(
            html.contains("highlight.min.js"),
            "highlight.js CDN should be included"
        );
        assert!(
            html.contains("hljs.highlightAll"),
            "hljs.highlightAll should be called"
        );
        assert!(
            html.contains("github-dark.min.css"),
            "highlight.js theme CSS should be included"
        );
        assert!(
            html.contains("language-mermaid"),
            "mermaid exclusion in hljs selector should be present"
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_syntax_highlighting_custom_theme() {
        let config = SiteConfig {
            code_theme: "monokai".to_string(),
            ..SiteConfig::default()
        };
        let generator = SiteGenerator::new(config);
        let docs = vec![SsgDocument {
            slug: "theme-test".to_string(),
            title: "Theme Test".to_string(),
            content: "# Code\n\n```rust\nfn main() {}\n```\n".to_string(),
            description: None,
            author: None,
            tags: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            order: 0,
            language: "en".to_string(),
        }];

        let tmp = std::env::temp_dir().join("tachyon-ssg-theme-test");
        let _ = std::fs::remove_dir_all(&tmp);
        generator.build_to_dir(&docs, &tmp).unwrap();

        let html = std::fs::read_to_string(tmp.join("theme-test.html")).unwrap();
        assert!(
            html.contains("monokai.min.css"),
            "custom theme CSS should be included"
        );
        assert!(
            !html.contains("github-dark.min.css"),
            "default theme CSS should NOT be included"
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_syntax_highlighting_disabled() {
        let config = SiteConfig {
            syntax_highlighting_enabled: false,
            ..SiteConfig::default()
        };
        let generator = SiteGenerator::new(config);
        let docs = vec![SsgDocument {
            slug: "no-highlight".to_string(),
            title: "No Highlight".to_string(),
            content: "# Code\n\n```rust\nfn main() {}\n```\n".to_string(),
            description: None,
            author: None,
            tags: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            order: 0,
            language: "en".to_string(),
        }];

        let tmp = std::env::temp_dir().join("tachyon-ssg-no-highlight-test");
        let _ = std::fs::remove_dir_all(&tmp);
        generator.build_to_dir(&docs, &tmp).unwrap();

        let html = std::fs::read_to_string(tmp.join("no-highlight.html")).unwrap();
        assert!(
            !html.contains("highlight.min.js"),
            "highlight.js should NOT be included when disabled"
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_site_config_new_defaults() {
        let config = SiteConfig::default();
        assert!(
            config.pagefind_enabled,
            "pagefind_enabled should default to true"
        );
        assert!(
            config.mermaid_enabled,
            "mermaid_enabled should default to true"
        );
        assert!(
            config.syntax_highlighting_enabled,
            "syntax_highlighting_enabled should default to true"
        );
        assert_eq!(
            config.code_theme, "github-dark",
            "code_theme should default to github-dark"
        );
        assert!(config.robots_txt, "robots_txt should default to true");
    }

    #[test]
    fn test_code_groups_two_adjacent_blocks() {
        use crate::render::process_code_groups;
        let input = r#"<div class="code-block-wrapper"><pre><code class="language-rust">fn main() {}</code></pre><button class="code-copy-btn" onclick="x">Copy</button></div><div class="code-block-wrapper"><pre><code class="language-typescript">console.log("hi")</code></pre><button class="code-copy-btn" onclick="x">Copy</button></div>"#;
        let result = process_code_groups(input);
        assert!(
            result.contains(r#"class="code-group""#),
            "should contain code-group div"
        );
        assert!(
            result.contains(r#"class="tab active""#),
            "should contain active tab"
        );
        assert!(
            result.contains(r#"data-lang="rust""#),
            "should have rust tab"
        );
        assert!(
            result.contains(r#"data-lang="typescript""#),
            "should have typescript tab"
        );
        assert!(
            result.contains("fn main()"),
            "rust code should be preserved"
        );
        assert!(
            result.contains("console.log"),
            "typescript code should be preserved"
        );
    }

    #[test]
    fn test_code_groups_no_grouping_single_block() {
        use crate::render::process_code_groups;
        let input = r#"<div class="code-block-wrapper"><pre><code class="language-rust">fn main() {}</code></pre><button class="code-copy-btn" onclick="x">Copy</button></div>"#;
        let result = process_code_groups(input);
        assert!(
            !result.contains(r#"class="code-group""#),
            "single block should not be grouped"
        );
    }

    #[test]
    fn test_code_groups_non_adjacent_not_grouped() {
        use crate::render::process_code_groups;
        let input = r#"<pre><code class="language-rust">fn main() {}</code></pre><p>Some text between</p><pre><code class="language-typescript">console.log("hi")</code></pre>"#;
        let result = process_code_groups(input);
        assert!(
            !result.contains(r#"class="code-group""#),
            "non-adjacent blocks should not be grouped"
        );
    }

    #[test]
    fn test_robots_txt_generation() {
        let config = SiteConfig {
            base_url: "https://docs.example.com".to_string(),
            ..Default::default()
        };
        let generator = SiteGenerator::new(config);
        let docs = vec![SsgDocument {
            slug: "test-robots".to_string(),
            title: "Test".to_string(),
            content: "# Test\n\nContent.".to_string(),
            description: None,
            author: None,
            tags: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            order: 0,
            language: "en".to_string(),
        }];

        let tmp = std::env::temp_dir().join("tachyon-ssg-robots-test");
        let _ = std::fs::remove_dir_all(&tmp);
        generator.build_to_dir(&docs, &tmp).unwrap();

        let robots = std::fs::read_to_string(tmp.join("robots.txt")).unwrap();
        assert!(
            robots.contains("User-agent: *"),
            "should contain User-agent"
        );
        assert!(robots.contains("Allow: /"), "should contain Allow: /");
        assert!(
            robots.contains("Sitemap: https://docs.example.com/sitemap.xml"),
            "should contain sitemap URL"
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_canonical_url_in_output() {
        let config = SiteConfig {
            base_url: "https://mydocs.example.com".to_string(),
            ..Default::default()
        };
        let generator = SiteGenerator::new(config);
        let docs = vec![SsgDocument {
            slug: "canonical-test".to_string(),
            title: "Canonical Test".to_string(),
            content: "# Test\n\nContent.".to_string(),
            description: None,
            author: None,
            tags: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            order: 0,
            language: "en".to_string(),
        }];

        let tmp = std::env::temp_dir().join("tachyon-ssg-canonical-test");
        let _ = std::fs::remove_dir_all(&tmp);
        generator.build_to_dir(&docs, &tmp).unwrap();

        let html = std::fs::read_to_string(tmp.join("canonical-test.html")).unwrap();
        assert!(
            html.contains(
                r#"<link rel="canonical" href="https://mydocs.example.com/canonical-test.html">"#
            ),
            "should contain canonical URL"
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_hreflang_multilingual() {
        let config = SiteConfig {
            base_url: "https://docs.example.com".to_string(),
            language: "en".to_string(),
            translations: vec![TranslationConfig {
                language: "zh".to_string(),
                name: "中文".to_string(),
                base_url: "https://docs.example.com/zh".to_string(),
            }],
            ..Default::default()
        };
        let generator = SiteGenerator::new(config);
        let docs = vec![
            SsgDocument {
                slug: "hreflang-test".to_string(),
                title: "Hreflang Test".to_string(),
                content: "# Test\n\nContent.".to_string(),
                description: None,
                author: None,
                tags: vec![],
                created_at: Utc::now(),
                updated_at: Utc::now(),
                order: 0,
                language: "en".to_string(),
            },
            SsgDocument {
                slug: "hreflang-test".to_string(),
                title: "Hreflang 测试".to_string(),
                content: "# 测试\n\n内容.".to_string(),
                description: None,
                author: None,
                tags: vec![],
                created_at: Utc::now(),
                updated_at: Utc::now(),
                order: 0,
                language: "zh".to_string(),
            },
        ];

        let tmp = std::env::temp_dir().join("tachyon-ssg-hreflang-test");
        let _ = std::fs::remove_dir_all(&tmp);
        generator.build_to_dir(&docs, &tmp).unwrap();

        let en_html = std::fs::read_to_string(tmp.join("en").join("hreflang-test.html")).unwrap();
        assert!(
            en_html.contains(r#"hreflang="en""#),
            "should contain en hreflang"
        );
        assert!(
            en_html.contains(r#"hreflang="zh""#),
            "should contain zh hreflang"
        );
        assert!(
            en_html.contains(r#"https://docs.example.com/en/hreflang-test.html"#),
            "should contain en alternate URL"
        );
        assert!(
            en_html.contains(r#"https://docs.example.com/zh/hreflang-test.html"#),
            "should contain zh alternate URL"
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
