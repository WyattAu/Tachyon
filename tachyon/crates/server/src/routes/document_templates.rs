//! Document template engine — variable substitution in templates.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub content: String,
    pub variables: Vec<TemplateVariable>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    pub name: String,
    pub label: String,
    pub default_value: Option<String>,
    pub required: bool,
}

/// Apply template variable substitution.
pub fn render_template(template: &str, variables: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in variables {
        result = result.replace(&format!("{{{{{}}}}}", key), value);
    }
    let re = regex::Regex::new(r"\{\{[^}]+\}\}").unwrap();
    re.replace_all(&result, "").to_string()
}

#[derive(Debug, Serialize)]
pub struct RenderedTemplate {
    pub content: String,
    pub variables_used: usize,
    pub variables_remaining: usize,
}

pub fn render_template_with_stats(template: &str, variables: &HashMap<String, String>) -> RenderedTemplate {
    let original_count = count_placeholders(template);
    let mut result = template.to_string();
    for (key, value) in variables {
        result = result.replace(&format!("{{{{{}}}}}", key), value);
    }
    let remaining = count_placeholders(&result);
    let re = regex::Regex::new(r"\{\{[^}]+\}\}").unwrap();
    let content = re.replace_all(&result, "").to_string();
    RenderedTemplate {
        content,
        variables_used: original_count - remaining,
        variables_remaining: remaining,
    }
}

fn count_placeholders(s: &str) -> usize {
    let re = regex::Regex::new(r"\{\{[^}]+\}\}").unwrap();
    re.find_iter(s).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_rendering() {
        let template = "Dear {{name}},\n\nYour order {{order_id}} is ready.";
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "Alice".to_string());
        vars.insert("order_id".to_string(), "12345".to_string());
        let rendered = render_template(template, &vars);
        assert!(rendered.contains("Alice"));
        assert!(rendered.contains("12345"));
        assert!(!rendered.contains("{{"));
    }

    #[test]
    fn test_template_missing_variable() {
        let template = "Hello {{name}} {{missing}}";
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "World".to_string());
        let rendered = render_template(template, &vars);
        assert!(rendered.contains("World"));
        assert!(!rendered.contains("{{missing}}"));
    }

    #[test]
    fn test_render_with_stats() {
        let template = "Hi {{name}}, ref {{ref}}, id {{id}}";
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "A".to_string());
        let result = render_template_with_stats(template, &vars);
        assert_eq!(result.variables_used, 1);
        assert_eq!(result.variables_remaining, 2);
    }

    #[test]
    fn test_template_serialization() {
        let t = DocumentTemplate {
            id: "1".to_string(),
            name: "Bug Report".to_string(),
            description: "A bug report template".to_string(),
            content: "Bug: {{title}}\nSteps: {{steps}}".to_string(),
            variables: vec![
                TemplateVariable {
                    name: "title".to_string(),
                    label: "Bug Title".to_string(),
                    default_value: None,
                    required: true,
                },
            ],
        };
        let json = serde_json::to_string(&t).unwrap();
        assert!(json.contains("Bug Report"));
    }
}
