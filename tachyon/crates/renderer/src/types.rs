//! Renderer types and structures
//!
//! Core render types (options, results, metadata, stats, themes, languages)
//! come from the published `docs-pipeline` crate and are re-exported here.
//! This module keeps the Tachyon-specific cache and template types plus the
//! aggregate [`RenderConfig`].

pub use docs_pipeline::types::{
    Language, MarkdownOptions, OutputFormat, RenderMetadata, RenderResult, RenderStats, SyntaxTheme,
};

use crate::error::RendererResult;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tachyon_core::id::DocumentId;

/// Cache key for rendered documents
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CacheKey {
    /// Document ID
    pub document_id: DocumentId,

    /// Content hash
    pub content_hash: String,

    /// Render options hash
    pub options_hash: String,
}

impl CacheKey {
    /// Create a new cache key
    pub fn new(document_id: DocumentId, content_hash: String, options_hash: String) -> Self {
        Self {
            document_id,
            content_hash,
            options_hash,
        }
    }

    /// Generate a cache key from components
    pub fn generate<S1, S2>(
        document_id: DocumentId,
        content: S1,
        options: S2,
    ) -> RendererResult<Self>
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
    {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(content.as_ref().as_bytes());
        let content_hash = format!("{:x}", hasher.finalize());

        let mut hasher = Sha256::new();
        hasher.update(options.as_ref().as_bytes());
        let options_hash = format!("{:x}", hasher.finalize());

        Ok(Self::new(document_id, content_hash, options_hash))
    }
}

/// Cache entry storing rendered documents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// Cache key
    pub key: CacheKey,

    /// Rendered content
    pub content: String,

    /// Render metadata
    pub metadata: RenderMetadata,

    /// Render statistics
    pub stats: RenderStats,

    /// Timestamp when entry was created
    pub created_at: time::OffsetDateTime,

    /// Timestamp when entry was last accessed
    pub last_accessed: time::OffsetDateTime,

    /// Time to live in seconds (None means no expiration)
    pub ttl_seconds: Option<u64>,
}

impl CacheEntry {
    /// Create a new cache entry
    pub fn new(
        key: CacheKey,
        content: String,
        metadata: RenderMetadata,
        stats: RenderStats,
        ttl_seconds: Option<u64>,
    ) -> Self {
        let now = time::OffsetDateTime::now_utc();
        Self {
            key,
            content,
            metadata,
            stats,
            created_at: now,
            last_accessed: now,
            ttl_seconds,
        }
    }

    /// Check if entry has expired
    pub fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl_seconds {
            let expiration = self.created_at + time::Duration::seconds(ttl as i64);
            time::OffsetDateTime::now_utc() > expiration
        } else {
            false
        }
    }

    /// Update last accessed timestamp
    pub fn touch(&mut self) {
        self.last_accessed = time::OffsetDateTime::now_utc();
    }

    /// Get age in seconds
    pub fn age_seconds(&self) -> i64 {
        let now = time::OffsetDateTime::now_utc();
        (now - self.created_at).whole_seconds()
    }

    /// Get time since last access in seconds
    pub fn idle_seconds(&self) -> i64 {
        let now = time::OffsetDateTime::now_utc();
        (now - self.last_accessed).whole_seconds()
    }
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Maximum number of entries in cache
    pub max_entries: usize,

    /// Default TTL for cache entries in seconds (None means no default TTL)
    pub default_ttl_seconds: Option<u64>,

    /// Enable cache statistics
    pub enable_stats: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            default_ttl_seconds: Some(3600), // 1 hour
            enable_stats: true,
        }
    }
}

impl CacheConfig {
    /// Create a new cache config
    pub fn new(max_entries: usize, default_ttl_seconds: Option<u64>) -> Self {
        Self {
            max_entries,
            default_ttl_seconds,
            enable_stats: true,
        }
    }

    /// Create config without TTL
    pub fn without_ttl(max_entries: usize) -> Self {
        Self::new(max_entries, None)
    }

    /// Set statistics enabled
    pub fn with_stats(mut self, enabled: bool) -> Self {
        self.enable_stats = enabled;
        self
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheStats {
    /// Total number of cache hits
    pub hits: u64,

    /// Total number of cache misses
    pub misses: u64,

    /// Current number of entries
    pub current_entries: usize,

    /// Total evictions
    pub evictions: u64,

    /// Total size in bytes
    pub total_bytes: u64,
}

impl CacheStats {
    /// Create a new cache stats
    pub fn new() -> Self {
        Self::default()
    }

    /// Increment hit count
    pub fn increment_hit(&mut self) {
        self.hits += 1;
    }

    /// Increment miss count
    pub fn increment_miss(&mut self) {
        self.misses += 1;
    }

    /// Increment eviction count
    pub fn increment_eviction(&mut self) {
        self.evictions += 1;
    }

    /// Calculate hit rate as percentage
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits as f64 / total as f64) * 100.0
        }
    }
}

/// Template context for rendering
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TemplateContext {
    /// Variables available in the template
    pub variables: BTreeMap<String, serde_json::Value>,

    /// Global functions available in the template
    #[serde(skip)]
    pub functions: Vec<String>,
}

impl TemplateContext {
    /// Create a new template context
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a variable
    pub fn set<V: Into<serde_json::Value>>(&mut self, key: String, value: V) {
        self.variables.insert(key, value.into());
    }

    /// Get a variable
    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.variables.get(key)
    }

    /// Check if a variable exists
    pub fn has(&self, key: &str) -> bool {
        self.variables.contains_key(key)
    }

    /// Remove a variable
    pub fn remove(&mut self, key: &str) -> Option<serde_json::Value> {
        self.variables.remove(key)
    }

    /// Clear all variables
    pub fn clear(&mut self) {
        self.variables.clear();
    }

    /// Convert variables to a minijinja-compatible value
    pub fn as_map(&self) -> minijinja::value::Value {
        let mut map: std::collections::HashMap<String, minijinja::value::Value> =
            std::collections::HashMap::new();
        for (key, value) in &self.variables {
            let val = convert_json_to_minijinja(value);
            map.insert(key.clone(), val);
        }
        minijinja::value::Value::from(map)
    }
}

/// Convert serde_json::Value to minijinja::value::Value
fn convert_json_to_minijinja(value: &serde_json::Value) -> minijinja::value::Value {
    match value {
        serde_json::Value::Null => minijinja::value::Value::from(()),
        serde_json::Value::Bool(b) => minijinja::value::Value::from(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                minijinja::value::Value::from(i)
            } else if let Some(f) = n.as_f64() {
                minijinja::value::Value::from(f)
            } else {
                minijinja::value::Value::from(())
            }
        }
        serde_json::Value::String(s) => minijinja::value::Value::from(s.as_str()),
        serde_json::Value::Array(arr) => {
            let vec: Vec<minijinja::value::Value> =
                arr.iter().map(convert_json_to_minijinja).collect();
            minijinja::value::Value::from(vec)
        }
        serde_json::Value::Object(obj) => {
            let mut map: std::collections::HashMap<String, minijinja::value::Value> =
                std::collections::HashMap::new();
            for (k, v) in obj {
                map.insert(k.clone(), convert_json_to_minijinja(v));
            }
            minijinja::value::Value::from(map)
        }
    }
}

/// Render configuration combining all options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderConfig {
    /// Markdown parsing options
    pub markdown: MarkdownOptions,

    /// Output format
    pub format: OutputFormat,

    /// Cache configuration
    pub cache: CacheConfig,

    /// Syntax highlighting theme
    pub syntax_theme: SyntaxTheme,

    /// Enable LaTeX rendering
    pub enable_latex: bool,

    /// Enable syntax highlighting
    pub enable_syntax: bool,

    /// Enable template rendering
    pub enable_templates: bool,

    /// Custom CSS classes
    pub custom_css_classes: Option<String>,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            markdown: MarkdownOptions::default(),
            format: OutputFormat::default(),
            cache: CacheConfig::default(),
            syntax_theme: SyntaxTheme::default(),
            enable_latex: true,
            enable_syntax: true,
            enable_templates: true,
            custom_css_classes: None,
        }
    }
}

impl RenderConfig {
    /// Create a new render config
    pub fn new() -> Self {
        Self::default()
    }

    /// Set output format
    pub fn with_format(mut self, format: OutputFormat) -> Self {
        self.format = format;
        self
    }

    /// Set syntax theme
    pub fn with_syntax_theme(mut self, theme: SyntaxTheme) -> Self {
        self.syntax_theme = theme;
        self
    }

    /// Enable or disable LaTeX
    pub fn with_latex(mut self, enabled: bool) -> Self {
        self.enable_latex = enabled;
        self
    }

    /// Enable or disable syntax highlighting
    pub fn with_syntax(mut self, enabled: bool) -> Self {
        self.enable_syntax = enabled;
        self
    }

    /// Enable or disable templates
    pub fn with_templates(mut self, enabled: bool) -> Self {
        self.enable_templates = enabled;
        self
    }

    /// Set cache configuration
    pub fn with_cache(mut self, config: CacheConfig) -> Self {
        self.cache = config;
        self
    }

    /// Set custom CSS classes
    pub fn with_css_classes<S: Into<String>>(mut self, classes: S) -> Self {
        self.custom_css_classes = Some(classes.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_generation() {
        use tachyon_core::id::DocumentId;

        let id = DocumentId::new();
        let key = CacheKey::generate(id, "content", "options").unwrap();
        assert_eq!(key.document_id, id);
        assert!(!key.content_hash.is_empty());
        assert!(!key.options_hash.is_empty());
    }

    #[test]
    fn test_cache_entry_expiration() {
        let entry = CacheEntry::new(
            CacheKey::generate(tachyon_core::id::DocumentId::new(), "content", "options").unwrap(),
            "rendered".to_string(),
            RenderMetadata::new(),
            RenderStats::new(),
            Some(1),
        );
        // Entry should not be expired immediately
        assert!(!entry.is_expired());
    }

    #[test]
    fn test_template_context() {
        let mut ctx = TemplateContext::new();
        ctx.set("title".to_string(), "Hello");
        assert_eq!(ctx.get("title"), Some(&serde_json::json!("Hello")));
        assert!(ctx.has("title"));
    }
}
