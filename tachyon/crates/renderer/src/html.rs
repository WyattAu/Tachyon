// HTML rendering module
// Handles HTML template rendering and minification

use anyhow::Result;

/// Render HTML with template data
/// 
/// # Arguments
/// * `template` - The HTML template string
/// * `data` - The data to insert into the template
/// 
/// # Returns
/// The rendered HTML string
/// 
/// # Errors
/// Returns an error if template rendering fails
pub fn render_html(template: &str, data: &serde_json::Value) -> Result<String> {
    // Simple template substitution (for production, use handlebars)
    let mut result = template.to_string();
    
    if let Some(obj) = data.as_object() {
        for (key, value) in obj {
            let placeholder = format!("{{{{{}}}}}", key);
            let replacement = match value {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => b.to_string(),
                _ => String::new(),
            };
            result = result.replace(&placeholder, &replacement);
        }
    }
    
    Ok(result)
}

/// Minify HTML by removing whitespace
/// 
/// # Arguments
/// * `html` - The HTML string to minify
/// 
/// # Returns
/// The minified HTML string
pub fn minify_html(html: &str) -> String {
    // Remove whitespace between tags
    let result = regex::Regex::new(r">\s+<")
        .map(|re| re.replace_all(html, "><"))
        .unwrap_or_else(|_| html.to_string());
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_render_html() {
        let template = "<div>{{title}}</div>";
        let data = serde_json::json!({"title": "Hello"});
        let result = render_html(template, &data).unwrap();
        assert_eq!(result, "<div>Hello</div>");
    }
    
    #[test]
    fn test_minify_html() {
        let html = "<div>\n  <p>Text</p>\n</div>";
        let result = minify_html(html);
        assert!(!result.contains('\n'));
    }
}
