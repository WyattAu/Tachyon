//! Tachyon SSG — Static Site Generator
//!
//! Generates a complete static site from Tachyon documents.

#![allow(dead_code)]
//! Produces HTML pages with navigation, index, sitemap, and RSS feed.
//! Designed to compete with Docusaurus as the best SSG for markdown content.

mod error;
mod templates;
pub mod i18n;

pub use error::{SsgError, SsgResult};
pub use templates::{DEFAULT_BASE_TEMPLATE, DEFAULT_DOC_TEMPLATE, DEFAULT_INDEX_TEMPLATE};
pub use i18n::{language_display_name, text_direction};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;

// ============================================================================
// Configuration
// ============================================================================

/// Site-wide configuration for static site generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteConfig {
    /// Site title (displayed in header, title tags, RSS feed)
    pub title: String,
    /// Site description (used in meta tags, OG, RSS)
    pub description: String,
    /// Base URL for canonical links and sitemap (e.g., "https://docs.example.com")
    pub base_url: String,
    /// Optional site logo URL (used in header)
    pub logo_url: Option<String>,
    /// Optional favicon URL
    pub favicon_url: Option<String>,
    /// Theme variant: "light", "dark", or "auto"
    #[serde(default = "default_theme")]
    pub theme: String,
    /// Optional custom CSS (appended after built-in styles)
    pub custom_css: Option<String>,
    /// Optional Google Analytics / Plausible tracking ID
    pub tracking_id: Option<String>,
    /// Navigation bar links
    #[serde(default)]
    pub nav_links: Vec<NavLink>,
    /// Footer text
    #[serde(default = "default_footer")]
    pub footer: String,
    /// Include author metadata in rendered pages
    #[serde(default)]
    pub show_author: bool,
    /// Include "last updated" timestamps
    #[serde(default = "default_true")]
    pub show_updated_at: bool,
    /// Group documents by their first tag (creates category pages)
    #[serde(default)]
    pub group_by_tag: bool,
    /// Site language code (ISO 639-1, e.g., "en", "zh", "ja")
    #[serde(default = "default_language")]
    pub language: String,
    /// Available translations (language codes)
    #[serde(default)]
    pub translations: Vec<TranslationConfig>,
    /// Custom color theme
    #[serde(default)]
    pub color_theme: Option<ColorTheme>,
}

fn default_theme() -> String {
    "auto".to_string()
}
fn default_true() -> bool {
    true
}
fn default_footer() -> String {
    "Built with Tachyon".to_string()
}
fn default_language() -> String {
    "en".to_string()
}

impl Default for SiteConfig {
    fn default() -> Self {
        Self {
            title: "Tachyon Docs".to_string(),
            description: "A knowledge base built with Tachyon".to_string(),
            base_url: "https://docs.example.com".to_string(),
            logo_url: None,
            favicon_url: None,
            theme: "auto".to_string(),
            custom_css: None,
            tracking_id: None,
            nav_links: vec![],
            footer: "Built with Tachyon".to_string(),
            show_author: false,
            show_updated_at: true,
            group_by_tag: false,
            language: "en".to_string(),
            translations: vec![],
            color_theme: None,
        }
    }
}

/// A navigation link in the site header.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavLink {
    pub label: String,
    pub href: String,
}

/// Configuration for a translated version of the site.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationConfig {
    /// ISO 639-1 language code (e.g., "zh", "ja", "de")
    pub language: String,
    /// Display name (e.g., "中文", "日本語")
    pub name: String,
    /// Base URL for this language version
    pub base_url: String,
}

/// Predefined color themes for the generated site.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorTheme {
    /// Primary color (hex, e.g., "#2563eb")
    pub primary: String,
    /// Secondary color (hex)
    pub secondary: String,
    /// Accent color (hex)
    pub accent: String,
    /// Background color for code blocks
    pub code_bg: String,
    /// Font family for body text
    pub font_family: Option<String>,
    /// Font family for headings
    pub heading_font_family: Option<String>,
}

impl Default for ColorTheme {
    fn default() -> Self {
        Self {
            primary: "#2563eb".to_string(),
            secondary: "#7c3aed".to_string(),
            accent: "#06b6d4".to_string(),
            code_bg: "#1f2937".to_string(),
            font_family: None,
            heading_font_family: None,
        }
    }
}

// ============================================================================
// Input: Document to render
// ============================================================================

/// A document to be included in the static site.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsgDocument {
    /// URL slug (used as filename: `{slug}.html`)
    pub slug: String,
    /// Document title
    pub title: String,
    /// Raw markdown content
    pub content: String,
    /// Optional description (for meta tag, falls back to first 160 chars)
    pub description: Option<String>,
    /// Author display name
    pub author: Option<String>,
    /// Tags
    #[serde(default)]
    pub tags: Vec<String>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
    /// Sort order (lower = earlier in listings)
    #[serde(default)]
    pub order: i32,
    /// Document language code (for i18n filtering)
    #[serde(default = "default_language")]
    pub language: String,
}

// ============================================================================
// Output: Build result
// ============================================================================

/// Result of a site build.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildResult {
    /// Number of document pages generated
    pub pages: usize,
    /// Number of category index pages (if group_by_tag)
    pub category_pages: usize,
    /// Total files written (pages + index + sitemap + rss + assets)
    pub total_files: usize,
    /// Build duration in milliseconds
    pub build_time_ms: u64,
    /// Output size in bytes (if written to disk)
    pub output_size_bytes: u64,
    /// List of generated page slugs
    pub generated_pages: Vec<String>,
    /// Number of languages generated
    pub languages: usize,
}

// ============================================================================
// SSG Engine
// ============================================================================

/// Static site generator engine.
///
/// Takes a `SiteConfig` and a list of `SsgDocument`s, renders each document
/// to HTML using `tachyon-renderer`, and produces a complete static site.
pub struct SiteGenerator {
    config: SiteConfig,
}

impl SiteGenerator {
    /// Create a new site generator with the given configuration.
    pub fn new(config: SiteConfig) -> Self {
        Self { config }
    }

    /// Generate the complete static site, writing files to the given directory.
    ///
    /// Creates the output directory if it doesn't exist. Returns a `BuildResult`
    /// with statistics about what was generated.
    ///
    /// When `translations` is configured in the `SiteConfig`, generates per-language
    /// subdirectories (e.g., `output/en/`, `output/zh/`). Otherwise, generates
    /// a flat site in the output directory (backward compatible).
    pub fn build_to_dir<P: AsRef<Path>>(
        &self,
        documents: &[SsgDocument],
        output_dir: P,
    ) -> SsgResult<BuildResult> {
        let start = std::time::Instant::now();
        let output_dir = output_dir.as_ref();

        std::fs::create_dir_all(output_dir).map_err(|e| SsgError::Io(e.to_string()))?;

        let mut total_files = 0usize;
        let mut total_pages = 0usize;
        let mut total_categories = 0usize;
        let mut all_generated_pages = Vec::new();

        // Collect all language codes to generate
        let languages = self.collect_languages(documents);

        if languages.len() > 1 {
            // Multi-language build: generate per-language subdirectories
            for lang in &languages {
                let lang_docs: Vec<&SsgDocument> = documents
                    .iter()
                    .filter(|d| d.language == *lang)
                    .collect();

                let (pages, categories, generated) =
                    self.build_language_dir(&lang_docs, lang, output_dir, &languages)?;

                total_pages += pages;
                total_categories += categories;
                total_files += pages + categories + 3; // +3 for index + sitemap + rss per language
                all_generated_pages.extend(generated);
            }

            // Root index.html: redirect to default language
            let redirect = self.render_root_redirect(&self.config.language)?;
            let index_path = output_dir.join("index.html");
            std::fs::write(&index_path, redirect)
                .map_err(|e| SsgError::Io(format!("Failed to write root index.html: {}", e)))?;
            total_files += 1;
        } else {
            // Single-language build: flat output (backward compatible)
            let lang = languages.first().map(|s| s.as_str()).unwrap_or("en");
            let all_docs: Vec<&SsgDocument> = documents.iter().collect();
            let (pages, categories, generated) =
                self.build_language_dir(&all_docs, lang, output_dir, &languages)?;

            total_pages += pages;
            total_categories += categories;
            total_files += pages + categories + 3; // +3 for index + sitemap + rss
            all_generated_pages.extend(generated);
        }

        // Calculate output size
        let output_size_bytes = dir_size(output_dir).map_err(|e| SsgError::Io(e.to_string()))?;
        let build_time_ms = start.elapsed().as_millis() as u64;

        tracing::info!(
            "SSG build complete: {} pages, {} categories, {} languages, {} files, {:.1}KB in {}ms",
            total_pages,
            total_categories,
            languages.len(),
            total_files,
            output_size_bytes as f64 / 1024.0,
            build_time_ms,
        );

        Ok(BuildResult {
            pages: total_pages,
            category_pages: total_categories,
            total_files,
            build_time_ms,
            output_size_bytes,
            generated_pages: all_generated_pages,
            languages: languages.len(),
        })
    }

    /// Generate the complete static site as a ZIP archive in memory.
    ///
    /// Returns the ZIP bytes and a `BuildResult`.
    /// Multi-language sites use per-language subdirectories in the ZIP.
    pub fn build_to_zip(&self, documents: &[SsgDocument]) -> SsgResult<(Vec<u8>, BuildResult)> {
        let start = std::time::Instant::now();

        let mut total_pages = 0usize;
        let mut total_categories = 0usize;
        let mut all_generated_pages = Vec::new();

        let languages = self.collect_languages(documents);

        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        let options: zip::write::FileOptions<()> =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        if languages.len() > 1 {
            for lang in &languages {
                let lang_docs: Vec<&SsgDocument> = documents
                    .iter()
                    .filter(|d| d.language == *lang)
                    .collect();

                let (pages, categories, generated) =
                    self.build_language_zip(&lang_docs, lang, &mut zip, options, &languages)?;

                total_pages += pages;
                total_categories += categories;
                all_generated_pages.extend(generated);
            }

            // Root index.html redirect
            let redirect = self.render_root_redirect(&self.config.language)?;
            zip.start_file("index.html", options)
                .map_err(|e| SsgError::Zip(e.to_string()))?;
            zip.write_all(redirect.as_bytes())
                .map_err(|e| SsgError::Zip(e.to_string()))?;
        } else {
            let lang = languages.first().map(|s| s.as_str()).unwrap_or("en");
            let all_docs: Vec<&SsgDocument> = documents.iter().collect();
            let (pages, categories, generated) =
                self.build_language_zip(&all_docs, lang, &mut zip, options, &languages)?;

            total_pages += pages;
            total_categories += categories;
            all_generated_pages.extend(generated);
        }

        let cursor = zip.finish().map_err(|e| SsgError::Zip(e.to_string()))?;
        let zip_bytes = cursor.into_inner();
        let zip_len = zip_bytes.len() as u64;
        let build_time_ms = start.elapsed().as_millis() as u64;

        Ok((
            zip_bytes,
            BuildResult {
                pages: total_pages,
                category_pages: total_categories,
                total_files: 0, // ZIP doesn't count individual files the same way
                build_time_ms,
                output_size_bytes: zip_len,
                generated_pages: all_generated_pages,
                languages: languages.len(),
            },
        ))
    }

    /// Collect all unique language codes from config + documents.
    fn collect_languages(&self, documents: &[SsgDocument]) -> Vec<String> {
        let mut langs: std::collections::HashSet<String> = std::collections::HashSet::new();

        // Always include the default language
        langs.insert(self.config.language.clone());

        // Include all configured translations
        for t in &self.config.translations {
            langs.insert(t.language.clone());
        }

        // Include any languages found in documents
        for doc in documents {
            if !doc.language.is_empty() {
                langs.insert(doc.language.clone());
            }
        }

        let mut langs: Vec<String> = langs.into_iter().collect();
        langs.sort();
        langs
    }

    /// Build a single language's output to a directory.
    /// Returns (pages, categories, generated_page_slugs).
    fn build_language_dir(
        &self,
        docs: &[&SsgDocument],
        lang: &str,
        output_dir: &Path,
        all_languages: &[String],
    ) -> SsgResult<(usize, usize, Vec<String>)> {
        let mut sorted_docs: Vec<&SsgDocument> = docs.to_vec();
        sorted_docs.sort_by_key(|d| (d.order, d.title.clone()));

        // Single language → flat output. Multi-language → subdirectory per language.
        let is_multi = all_languages.len() > 1;
        let write_dir = if is_multi {
            let lang_dir = output_dir.join(lang);
            std::fs::create_dir_all(&lang_dir).map_err(|e| SsgError::Io(e.to_string()))?;
            lang_dir
        } else {
            output_dir.to_path_buf()
        };

        let lang_prefix = if is_multi {
            Some(format!("{}/", lang))
        } else {
            None
        };

        let mut generated_pages = Vec::new();

        // Tag groups
        let tag_groups: BTreeMap<String, Vec<&SsgDocument>> = if self.config.group_by_tag {
            let mut map: BTreeMap<String, Vec<&SsgDocument>> = BTreeMap::new();
            for doc in &sorted_docs {
                for tag in &doc.tags {
                    map.entry(tag.clone()).or_default().push(doc);
                }
            }
            map
        } else {
            BTreeMap::new()
        };

        // Document pages
        for doc in &sorted_docs {
            let html = self.render_document_page(doc, &sorted_docs, lang, lang_prefix.as_deref(), all_languages)?;
            let filename = format!("{}.html", doc.slug);
            let path = write_dir.join(&filename);
            std::fs::write(&path, html)
                .map_err(|e| SsgError::Io(format!("Failed to write {}/{}: {}", lang, filename, e)))?;
            generated_pages.push(doc.slug.clone());
        }

        // Category pages
        let mut category_count = 0usize;
        if self.config.group_by_tag {
            for (tag, tag_docs) in &tag_groups {
                let html = self.render_category_page(tag, tag_docs, lang, lang_prefix.as_deref(), all_languages)?;
                let filename = format!("category-{}.html", slug::slugify(tag));
                let path = write_dir.join(&filename);
                std::fs::write(&path, html)
                    .map_err(|e| SsgError::Io(format!("Failed to write category page: {}", e)))?;
                category_count += 1;
            }
        }

        // Language index page
        let index_html = self.render_index_page(&sorted_docs, lang, lang_prefix.as_deref(), all_languages)?;
        let index_path = write_dir.join("index.html");
        std::fs::write(&index_path, index_html)
            .map_err(|e| SsgError::Io(format!("Failed to write {}/index.html: {}", lang, e)))?;

        // Language sitemap
        let sitemap = self.render_sitemap(&sorted_docs, lang, lang_prefix.as_deref())?;
        let sitemap_path = write_dir.join("sitemap.xml");
        std::fs::write(&sitemap_path, sitemap)
            .map_err(|e| SsgError::Io(format!("Failed to write {}/sitemap.xml: {}", lang, e)))?;

        // Language RSS feed
        let rss = self.render_rss(&sorted_docs, lang, lang_prefix.as_deref())?;
        let rss_path = write_dir.join("feed.xml");
        std::fs::write(&rss_path, rss)
            .map_err(|e| SsgError::Io(format!("Failed to write {}/feed.xml: {}", lang, e)))?;

        Ok((sorted_docs.len(), category_count, generated_pages))
    }

    /// Build a single language's output into a ZIP.
    fn build_language_zip(
        &self,
        docs: &[&SsgDocument],
        lang: &str,
        zip: &mut zip::ZipWriter<std::io::Cursor<Vec<u8>>>,
        options: zip::write::FileOptions<()>,
        all_languages: &[String],
    ) -> SsgResult<(usize, usize, Vec<String>)> {
        let mut sorted_docs: Vec<&SsgDocument> = docs.to_vec();
        sorted_docs.sort_by_key(|d| (d.order, d.title.clone()));

        let is_multi = all_languages.len() > 1;
        let lang_prefix = if is_multi {
            Some(format!("{}/", lang))
        } else {
            None
        };

        let mut generated_pages = Vec::new();

        let tag_groups: BTreeMap<String, Vec<&SsgDocument>> = if self.config.group_by_tag {
            let mut map: BTreeMap<String, Vec<&SsgDocument>> = BTreeMap::new();
            for doc in &sorted_docs {
                for tag in &doc.tags {
                    map.entry(tag.clone()).or_default().push(doc);
                }
            }
            map
        } else {
            BTreeMap::new()
        };

        for doc in &sorted_docs {
            let html = self.render_document_page(doc, &sorted_docs, lang, lang_prefix.as_deref(), all_languages)?;
            let path = if is_multi {
                format!("{}/{}.html", lang, doc.slug)
            } else {
                format!("{}.html", doc.slug)
            };
            zip.start_file(&path, options)
                .map_err(|e| SsgError::Zip(e.to_string()))?;
            zip.write_all(html.as_bytes())
                .map_err(|e| SsgError::Zip(e.to_string()))?;
            generated_pages.push(path);
        }

        let mut category_count = 0usize;
        if self.config.group_by_tag {
            for (tag, tag_docs) in &tag_groups {
                let html = self.render_category_page(tag, tag_docs, lang, lang_prefix.as_deref(), all_languages)?;
                let path = if is_multi {
                    format!("{}/category-{}.html", lang, slug::slugify(tag))
                } else {
                    format!("category-{}.html", slug::slugify(tag))
                };
                zip.start_file(&path, options)
                    .map_err(|e| SsgError::Zip(e.to_string()))?;
                zip.write_all(html.as_bytes())
                    .map_err(|e| SsgError::Zip(e.to_string()))?;
                category_count += 1;
            }
        }

        // Index
        let index_html = self.render_index_page(&sorted_docs, lang, lang_prefix.as_deref(), all_languages)?;
        let index_path = if is_multi {
            format!("{}/index.html", lang)
        } else {
            "index.html".to_string()
        };
        zip.start_file(&index_path, options)
            .map_err(|e| SsgError::Zip(e.to_string()))?;
        zip.write_all(index_html.as_bytes())
            .map_err(|e| SsgError::Zip(e.to_string()))?;

        Ok((sorted_docs.len(), category_count, generated_pages))
    }

    /// Render a root index.html that redirects to the default language.
    fn render_root_redirect(&self, default_lang: &str) -> SsgResult<String> {
        Ok(format!(
            r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <meta http-equiv="refresh" content="0;url={lang}/">
  <link rel="canonical" href="{lang}/">
</head>
<body>
  <p>Redirecting to <a href="{lang}/">{lang}</a>...</p>
</body>
</html>"#,
            lang = default_lang,
        ))
    }

    // ========================================================================
    // Rendering methods (private)
    // ========================================================================

    /// Render a single document to a full HTML page.
    fn render_document_page(
        &self,
        doc: &SsgDocument,
        all_docs: &[&SsgDocument],
        lang: &str,
        lang_prefix: Option<&str>,
        all_languages: &[String],
    ) -> SsgResult<String> {
        // Render markdown to HTML body
        let body_html = render_markdown(&doc.content);

        // Build navigation items — use lang_prefix-relative paths
        let nav_items = all_docs
            .iter()
            .map(|d| NavItem {
                title: d.title.clone(),
                href: format!("{}.html", d.slug),
            })
            .collect::<Vec<_>>();

        let description = doc
            .description
            .clone()
            .unwrap_or_else(|| truncate_text(&body_html, 160));

        let base = self.config.base_url.trim_end_matches('/');
        let page_url = if let Some(prefix) = lang_prefix {
            format!("{}/{}{}.html", base, prefix, doc.slug)
        } else {
            format!("{}/{}.html", base, doc.slug)
        };

        // Build language switcher links
        let language_switcher = self.build_language_switcher(all_languages, &doc.slug, lang_prefix);

        let ctx = PageContext {
            site: &self.config,
            title: &doc.title,
            description: &description,
            body: &body_html,
            page_url: &page_url,
            author: doc.author.as_deref(),
            created_at: &doc.created_at.to_rfc3339(),
            updated_at: &doc.updated_at.to_rfc3339(),
            tags: &doc.tags,
            nav_items: &nav_items,
            current_slug: Some(&doc.slug),
            language: lang,
            language_switcher: &language_switcher,
        };

        Ok(templates::render_doc_page(&ctx))
    }

    /// Render the site index page.
    fn render_index_page(
        &self,
        docs: &[&SsgDocument],
        lang: &str,
        lang_prefix: Option<&str>,
        all_languages: &[String],
    ) -> SsgResult<String> {
        let nav_items = docs
            .iter()
            .map(|d| NavItem {
                title: d.title.clone(),
                href: format!("{}.html", d.slug),
            })
            .collect::<Vec<_>>();

        let doc_cards: Vec<DocCard> = docs
            .iter()
            .map(|d| {
                let body = render_markdown(&d.content);
                DocCard {
                    title: d.title.clone(),
                    slug: d.slug.clone(),
                    description: d
                        .description
                        .clone()
                        .unwrap_or_else(|| truncate_text(&body, 200)),
                    tags: d.tags.clone(),
                    updated_at: d.updated_at.to_rfc3339(),
                    author: d.author.clone(),
                }
            })
            .collect();

        let language_switcher = self.build_language_switcher(all_languages, "index", lang_prefix);

        let ctx = IndexContext {
            site: &self.config,
            nav_items: &nav_items,
            documents: &doc_cards,
            language: lang,
            language_switcher: &language_switcher,
        };

        Ok(templates::render_index_page(&ctx))
    }

    /// Render a category index page.
    fn render_category_page(
        &self,
        tag: &str,
        docs: &[&SsgDocument],
        lang: &str,
        lang_prefix: Option<&str>,
        all_languages: &[String],
    ) -> SsgResult<String> {
        let doc_cards: Vec<DocCard> = docs
            .iter()
            .map(|d| {
                let body = render_markdown(&d.content);
                DocCard {
                    title: d.title.clone(),
                    slug: d.slug.clone(),
                    description: d
                        .description
                        .clone()
                        .unwrap_or_else(|| truncate_text(&body, 200)),
                    tags: d.tags.clone(),
                    updated_at: d.updated_at.to_rfc3339(),
                    author: d.author.clone(),
                }
            })
            .collect();

        let language_switcher = self.build_language_switcher(all_languages, "index", lang_prefix);

        let ctx = CategoryContext {
            site: &self.config,
            category_name: tag,
            documents: &doc_cards,
            language: lang,
            language_switcher: &language_switcher,
        };

        Ok(templates::render_category_page(&ctx))
    }

    /// Render a sitemap.xml.
    fn render_sitemap(
        &self,
        docs: &[&SsgDocument],
        lang: &str,
        lang_prefix: Option<&str>,
    ) -> SsgResult<String> {
        let base = self.config.base_url.trim_end_matches('/');
        let now = Utc::now().to_rfc3339();

        let index_loc = if let Some(prefix) = lang_prefix {
            format!("{}/{}index.html", base, prefix)
        } else {
            format!("{}/index.html", base)
        };

        let mut urls = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:xhtml="http://www.w3.org/1999/xhtml">
  <url>
    <loc>{index_loc}</loc>
    <lastmod>{now}</lastmod>
    <changefreq>daily</changefreq>
    <priority>1.0</priority>
    <xhtml:link rel="alternate" hreflang="{lang}" href="{index_loc}"/>
  </url>"#
        );

        for doc in docs {
            let doc_loc = if let Some(prefix) = lang_prefix {
                format!("{}/{}{}.html", base, prefix, doc.slug)
            } else {
                format!("{}/{}.html", base, doc.slug)
            };
            urls.push_str(&format!(
                r#"
  <url>
    <loc>{doc_loc}</loc>
    <lastmod>{updated}</lastmod>
    <changefreq>weekly</changefreq>
    <priority>0.8</priority>
    <xhtml:link rel="alternate" hreflang="{lang}" href="{doc_loc}"/>
  </url>"#,
                doc_loc = doc_loc,
                lang = lang,
                updated = doc.updated_at.to_rfc3339(),
            ));
        }

        urls.push_str("\n</urlset>");
        Ok(urls)
    }

    /// Render an RSS 2.0 feed.
    fn render_rss(
        &self,
        docs: &[&SsgDocument],
        lang: &str,
        lang_prefix: Option<&str>,
    ) -> SsgResult<String> {
        let base = self.config.base_url.trim_end_matches('/');
        // RSS feeds typically show the 20 most recent items
        let recent: Vec<_> = docs.iter().take(20).collect();
        let now = Utc::now().to_rfc3339();

        let base_with_prefix = if let Some(prefix) = lang_prefix {
            format!("{}/{}", base, prefix)
        } else {
            base.to_string()
        };

        let mut items = String::new();
        for doc in &recent {
            let description = doc
                .description
                .clone()
                .unwrap_or_else(|| "No description".to_string());
            items.push_str(&format!(
                r#"
    <item>
      <title>{}</title>
      <link>{}/{}.html</link>
      <description>{}</description>
      <pubDate>{}</pubDate>
      <guid>{}/{}.html</guid>
    </item>"#,
                escape_xml(&doc.title),
                base_with_prefix,
                doc.slug,
                escape_xml(&description),
                doc.updated_at.to_rfc3339(),
                base_with_prefix,
                doc.slug,
            ));
        }

        Ok(format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">
  <channel>
    <title>{}</title>
    <link>{}</link>
    <description>{}</description>
    <language>{}</language>
    <lastBuildDate>{}</lastBuildDate>
    <atom:link href="{}/feed.xml" rel="self" type="application/rss+xml"/>
    <generator>Tachyon SSG</generator>
    {}
  </channel>
</rss>"#,
            escape_xml(&self.config.title),
            base_with_prefix,
            escape_xml(&self.config.description),
            lang,
            now,
            base_with_prefix,
            items,
        ))
    }

    /// Build a language switcher HTML snippet for the nav bar.
    fn build_language_switcher(
        &self,
        all_languages: &[String],
        current_page: &str,
        lang_prefix: Option<&str>,
    ) -> String {
        if all_languages.len() <= 1 {
            return String::new();
        }

        let prefix_to = |target_lang: &str| -> String {
            // Navigate from current lang dir to target lang dir
            // e.g., from /en/ to /zh/ → "../zh/"
            // from /zh/ to /zh/ → ""
            // For root pages (single lang): just "{target_lang}/"
            match lang_prefix {
                Some(_) => format!("../{}/", target_lang),
                None => format!("{}/", target_lang),
            }
        };

        all_languages
            .iter()
            .map(|lang| {
                let name = self::i18n::language_display_name(lang);
                let href = format!("{}{}", prefix_to(lang), current_page);
                let is_current = lang_prefix
                    .map(|p| p.trim_end_matches('/') == *lang)
                    .unwrap_or(false);
                let active_class = if is_current {
                    " font-bold underline"
                } else {
                    ""
                };
                format!(
                    r#"<a href="{href}" class="text-sm{active_class}">{name}</a>"#,
                    href = href,
                    name = name,
                    active_class = active_class,
                )
            })
            .collect::<Vec<_>>()
            .join("\n          ")
    }
}

// ============================================================================
// Template context types (private)
// ============================================================================

struct PageContext<'a> {
    site: &'a SiteConfig,
    title: &'a str,
    description: &'a str,
    body: &'a str,
    page_url: &'a str,
    author: Option<&'a str>,
    created_at: &'a str,
    updated_at: &'a str,
    tags: &'a [String],
    nav_items: &'a [NavItem],
    current_slug: Option<&'a str>,
    language: &'a str,
    language_switcher: &'a str,
}

struct IndexContext<'a> {
    site: &'a SiteConfig,
    nav_items: &'a [NavItem],
    documents: &'a [DocCard],
    language: &'a str,
    language_switcher: &'a str,
}

struct CategoryContext<'a> {
    site: &'a SiteConfig,
    category_name: &'a str,
    documents: &'a [DocCard],
    language: &'a str,
    language_switcher: &'a str,
}

struct NavItem {
    title: String,
    href: String,
}

struct DocCard {
    title: String,
    slug: String,
    description: String,
    tags: Vec<String>,
    updated_at: String,
    author: Option<String>,
}

// ============================================================================
// Utility functions
// ============================================================================

/// Render markdown to HTML using tachyon-renderer.
fn render_markdown(content: &str) -> String {
    use tachyon_renderer::{RenderConfig, Renderer};

    let config = RenderConfig::default();
    match Renderer::new(config).render(content, None) {
        Ok(result) => result.content,
        Err(_) => {
            // Fallback: if renderer fails, return the raw content wrapped in a div
            format!("<div>{}</div>", content)
        }
    }
}

/// Truncate text to approximately `max_chars` characters, breaking at word boundaries.
fn truncate_text(html: &str, max_chars: usize) -> String {
    // Strip HTML tags for the description
    let plain = html_to_plain(html);
    if plain.len() <= max_chars {
        plain
    } else {
        let truncated = &plain[..max_chars];
        // Find last space to break cleanly
        if let Some(pos) = truncated.rfind(' ') {
            format!("{}...", &truncated[..pos])
        } else {
            format!("{}...", truncated)
        }
    }
}

/// Strip HTML tags to get plain text.
fn html_to_plain(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut prev_was_tag_end = false;
    for ch in html.chars() {
        match ch {
            '<' => {
                in_tag = true;
            }
            '>' => {
                in_tag = false;
                prev_was_tag_end = true;
            }
            _ if !in_tag => {
                if prev_was_tag_end && !ch.is_whitespace() && !result.is_empty() {
                    result.push(' ');
                }
                result.push(ch);
                prev_was_tag_end = false;
            }
            _ => {}
        }
    }
    // Collapse whitespace
    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Escape XML special characters.
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Calculate total directory size in bytes.
fn dir_size(path: &Path) -> std::io::Result<u64> {
    let mut total = 0u64;
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            if metadata.is_file() {
                total += metadata.len();
            } else if metadata.is_dir() {
                total += dir_size(&entry.path())?;
            }
        }
    }
    Ok(total)
}

// ============================================================================
// Slug utility
// ============================================================================

pub mod slug {
    /// Convert a string to a URL-safe slug.
    pub fn slugify(s: &str) -> String {
        s.to_lowercase()
            .chars()
            .map(|ch| {
                if ch.is_alphanumeric() || ch == '-' || ch == '_' {
                    ch
                } else if ch.is_whitespace() {
                    '-'
                } else {
                    '\0' // sentinel
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

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(result.total_files, 5); // 2 docs + index + sitemap + rss
        assert!(result.build_time_ms > 0);
        assert!(result.output_size_bytes > 0);
        assert_eq!(result.languages, 1);

        // Verify files exist (single-language = flat output)
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
        let mut config = SiteConfig::default();
        config.group_by_tag = true;

        let generator = SiteGenerator::new(config);
        let docs = sample_documents();

        let tmp = std::env::temp_dir().join("tachyon-ssg-tag-test");
        let _ = std::fs::remove_dir_all(&tmp);

        let result = generator.build_to_dir(&docs, &tmp).unwrap();

        assert_eq!(result.category_pages, 2); // "guide" and "beginner"
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
            translations: vec![
                TranslationConfig {
                    language: "zh".to_string(),
                    name: "中文".to_string(),
                    base_url: "https://docs.example.com/zh".to_string(),
                },
            ],
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

        // Should have 2 languages
        assert_eq!(result.languages, 2);
        assert_eq!(result.pages, 2); // 1 per language

        // Root index.html should redirect
        let root_index = std::fs::read_to_string(tmp.join("index.html")).unwrap();
        assert!(root_index.contains("en/"));

        // English subdirectory
        assert!(tmp.join("en/index.html").exists());
        assert!(tmp.join("en/getting-started.html").exists());
        let en_html = std::fs::read_to_string(tmp.join("en/getting-started.html")).unwrap();
        assert!(en_html.contains("Getting Started"));
        assert!(en_html.contains(r#"lang="en""#));
        assert!(en_html.contains(r#"dir="ltr""#));
        // Should have language switcher
        assert!(en_html.contains("English"));
        assert!(en_html.contains("中文"));

        // Chinese subdirectory
        assert!(tmp.join("zh/index.html").exists());
        assert!(tmp.join("zh/getting-started.html").exists());
        let zh_html = std::fs::read_to_string(tmp.join("zh/getting-started.html")).unwrap();
        assert!(zh_html.contains("快速入门"));
        assert!(zh_html.contains(r#"lang="zh""#));

        // Sitemap and RSS per language
        assert!(tmp.join("en/sitemap.xml").exists());
        assert!(tmp.join("en/feed.xml").exists());
        assert!(tmp.join("zh/sitemap.xml").exists());
        assert!(tmp.join("zh/feed.xml").exists());

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
