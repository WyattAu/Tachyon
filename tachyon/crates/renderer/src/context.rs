// Rendering context module
// Provides context data for rendering templates

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Rendering context with template data
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderContext {
    /// Document title
    pub title: String,
    /// Document content (HTML)
    pub content: String,
    /// Author information
    pub author: Option<AuthorInfo>,
    /// Metadata
    pub metadata: Option<RenderMetadata>,
    /// Navigation
    pub navigation: Option<NavigationInfo>,
}

/// Author information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct AuthorInfo {
    pub name: String,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
}

/// Rendering metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderMetadata {
    pub created_at: String,
    pub updated_at: String,
    pub tags: Vec<String>,
    pub read_time: Option<usize>,
}

/// Navigation information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct NavigationInfo {
    pub current_page: String,
    pub breadcrumbs: Vec<Breadcrumb>,
    pub related_pages: Vec<RelatedPage>,
}

/// Breadcrumb item
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Breadcrumb {
    pub title: String,
    pub url: String,
}

/// Related page
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct RelatedPage {
    pub title: String,
    pub url: String,
    pub snippet: Option<String>,
}

#[allow(dead_code)]
impl RenderContext {
    /// Create a new render context
    ///
    /// # Arguments
    /// * `title` - Document title
    /// * `content` - HTML content
    pub fn new(title: String, content: String) -> Self {
        Self {
            title,
            content,
            author: None,
            metadata: None,
            navigation: None,
        }
    }

    /// Convert to JSON Value
    pub(crate) fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
}

impl Default for RenderContext {
    fn default() -> Self {
        Self::new(String::new(), String::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_context_new() {
        let ctx = RenderContext::new("Test".to_string(), "<p>Content</p>".to_string());
        assert_eq!(ctx.title, "Test");
        assert_eq!(ctx.content, "<p>Content</p>");
    }

    #[test]
    fn test_render_context_to_json() {
        let ctx = RenderContext::new("Test".to_string(), "<p>Content</p>".to_string());
        let json = ctx.to_json();
        assert!(json.is_object());
    }
}
