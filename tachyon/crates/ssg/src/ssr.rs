//! Server-side rendering for hybrid dynamic/SSG pages.

use crate::manifest::SsgDocument;

pub fn render_document(document: &SsgDocument) -> String {
    crate::render::render_markdown(&document.content, "client", "github-dark")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_document() -> SsgDocument {
        SsgDocument {
            slug: "test".to_string(),
            title: "Test".to_string(),
            content: "# Hello\n\nWorld".to_string(),
            description: Some("A test".to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn test_render_document_returns_html() {
        let html = render_document(&test_document());
        assert!(html.contains("Hello"));
    }
}
