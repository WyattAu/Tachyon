//! Tachyon SSG — Static Site Generator
//!
//! Generates a complete static site from Tachyon documents.

#![allow(dead_code)]
//! Produces HTML pages with navigation, index, sitemap, and RSS feed.
//! Designed to compete with Docusaurus as the best SSG for markdown content.

mod build;
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
        assert_eq!(result.total_files, 5);
        assert!(result.build_time_ms > 0);
        assert!(result.output_size_bytes > 0);
        assert_eq!(result.languages, 1);

        assert!(tmp.join("index.html").exists());
        assert!(tmp.join("getting-started.html").exists());
        assert!(tmp.join("configuration.html").exists());
        assert!(tmp.join("sitemap.xml").exists());
        assert!(tmp.join("feed.xml").exists());

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
}
