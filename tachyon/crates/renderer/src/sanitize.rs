//! HTML sanitization module.
//!
//! Wraps `ammonia::clean()` to strip XSS vectors from rendered HTML output.

use ammonia::clean;

/// Sanitize HTML content by removing potentially dangerous elements and attributes.
///
/// This strips:
/// - `<script>`, `<style>`, `<iframe>`, `<object>`, `<embed>`, `<form>` tags
/// - `on*` event handler attributes
/// - `javascript:` and `data:` URLs
/// - SVG-based XSS vectors
pub fn sanitize_html(html: &str) -> String {
    clean(html)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strips_script_tags() {
        let input = r#"<p>Hello</p><script>alert('xss')</script><p>World</p>"#;
        let output = sanitize_html(input);
        assert!(!output.contains("<script>"));
        assert!(output.contains("Hello"));
        assert!(output.contains("World"));
    }

    #[test]
    fn test_strips_onerror_attribute() {
        let input = r#"<img src="x" onerror="alert(1)">"#;
        let output = sanitize_html(input);
        assert!(!output.contains("onerror"));
    }

    #[test]
    fn test_strips_javascript_uri() {
        let input = r#"<a href="javascript:alert(1)">click</a>"#;
        let output = sanitize_html(input);
        assert!(!output.contains("javascript:"));
    }

    #[test]
    fn test_preserves_safe_html() {
        let input = r#"<h1>Title</h1><p><strong>Bold</strong> and <em>italic</em></p><a href="https://example.com">link</a>"#;
        let output = sanitize_html(input);
        assert!(output.contains("<h1>"));
        assert!(output.contains("<strong>"));
        assert!(output.contains("href"));
    }

    #[test]
    fn test_strips_svg_xss() {
        let input = r#"<svg onload="alert(1)"><circle r="10"/></svg>"#;
        let output = sanitize_html(input);
        assert!(!output.contains("onload"));
    }

    #[test]
    fn test_strips_iframe() {
        let input = r#"<iframe src="https://evil.com"></iframe>"#;
        let output = sanitize_html(input);
        assert!(!output.contains("<iframe"));
    }
}
