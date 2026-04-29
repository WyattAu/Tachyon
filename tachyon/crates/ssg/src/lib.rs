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
pub use manifest::{BuildResult, ColorTheme, NavLink, SiteConfig, SsgDocument, TranslationConfig};
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
}
