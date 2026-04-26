use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
