//! MDX Component Registry
//!
//! Defines the `MdxComponent` trait and built-in components for rendering
//! MDX content to HTML.

use std::collections::HashMap;

/// Trait for MDX components that can render themselves to HTML
pub trait MdxComponent: Send + Sync {
    /// Render the component with given props and children to HTML
    fn render(&self, props: &HashMap<String, String>, children: &str) -> String;
}

/// A registry of MDX components
pub struct ComponentRegistry {
    components: HashMap<String, Box<dyn MdxComponent>>,
}

impl ComponentRegistry {
    /// Create a new component registry with built-in components
    pub fn new() -> Self {
        let mut registry = Self {
            components: HashMap::new(),
        };
        registry.register_builtins();
        registry
    }

    /// Register a custom component
    pub fn register(&mut self, name: impl Into<String>, component: Box<dyn MdxComponent>) {
        self.components.insert(name.into(), component);
    }

    /// Look up a component by name
    pub fn get(&self, name: &str) -> Option<&dyn MdxComponent> {
        self.components.get(name).map(|c| c.as_ref())
    }

    /// Check if a component is registered
    pub fn has(&self, name: &str) -> bool {
        self.components.contains_key(name)
    }

    fn register_builtins(&mut self) {
        self.register("Callout", Box::new(CalloutComponent));
        self.register("Tabs", Box::new(TabsComponent));
        self.register("CodeBlock", Box::new(CodeBlockComponent));
        self.register("Badge", Box::new(BadgeComponent));
        self.register("Frame", Box::new(FrameComponent));
        self.register("Steps", Box::new(StepsComponent));
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ── Callout ──────────────────────────────────────────────────────────────────

struct CalloutComponent;

impl MdxComponent for CalloutComponent {
    fn render(&self, props: &HashMap<String, String>, children: &str) -> String {
        let callout_type = props.get("type").map(|s| s.as_str()).unwrap_or("note");
        let title = props
            .get("title")
            .cloned()
            .unwrap_or_else(|| capitalize_first(callout_type));

        format!(
            r#"<div class="admonition admonition-{type}">
<p class="admonition-title">{title}</p>
{children}
</div>"#,
            type = callout_type.to_lowercase(),
            title = escape_html(&title),
            children = children,
        )
    }
}

// ── Tabs ─────────────────────────────────────────────────────────────────────

struct TabsComponent;

impl MdxComponent for TabsComponent {
    fn render(&self, props: &HashMap<String, String>, children: &str) -> String {
        let labels_raw = props.get("labels").map(|s| s.as_str()).unwrap_or("[]");
        let labels = parse_json_string_array(labels_raw);

        if labels.is_empty() {
            return children.to_string();
        }

        let tabs_html: String = labels
            .iter()
            .enumerate()
            .map(|(i, label)| {
                let active = if i == 0 { " active" } else { "" };
                format!(
                    r#"<button class="content-tab{active}" data-tab="tab-{i}" onclick="this.parentElement.querySelectorAll('.content-tab,.content-tab-panel').forEach(function(e){{e.classList.remove('active')}});this.classList.add('active');this.closest('.content-group').querySelectorAll('.content-tab-panel[data-tab=&quot;tab-{i}&quot;]').forEach(function(e){{e.classList.add('active')}})">{label}</button>"#,
                    active = active,
                    i = i,
                    label = escape_html(label),
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let panels = split_tab_panels(children, &labels);

        let panels_html: String = panels
            .iter()
            .enumerate()
            .map(|(i, content)| {
                let active = if i == 0 { " active" } else { "" };
                format!(
                    r#"<div class="content-tab-panel{active}" data-tab="tab-{i}">{content}</div>"#,
                    active = active,
                    i = i,
                    content = content,
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"<div class="content-group"><div class="content-tabs">{tabs}</div>{panels}</div>"#,
            tabs = tabs_html,
            panels = panels_html,
        )
    }
}

// ── CodeBlock ────────────────────────────────────────────────────────────────

struct CodeBlockComponent;

impl MdxComponent for CodeBlockComponent {
    fn render(&self, props: &HashMap<String, String>, children: &str) -> String {
        let lang = props.get("lang").map(|s| s.as_str()).unwrap_or("");
        let title = props.get("title").map(|s| s.as_str());

        let title_html = match title {
            Some(t) => format!(r#"<div class="code-title">{}</div>"#, escape_html(t)),
            None => String::new(),
        };

        let lang_attr = if lang.is_empty() {
            String::new()
        } else {
            format!(" language-{}", lang)
        };

        let code = if children.starts_with('\n') {
            children[1..].trim_end_matches('\n')
        } else {
            children.trim()
        };

        format!(
            r#"{title}<div class="code-block-wrapper"><pre class="syntax-highlight" data-language="{lang}"><code class="{lang_attr}">{code}</code></pre><button class="code-copy-btn" onclick="(function(b){{var c=b.parentElement.querySelector('code');navigator.clipboard.writeText(c.textContent).then(function(){{b.textContent='Copied!';setTimeout(function(){{b.textContent='Copy'}},2000)}})}})(this)" aria-label="Copy code to clipboard">Copy</button></div>"#,
            title = title_html,
            lang = lang,
            lang_attr = lang_attr,
            code = escape_html(code),
        )
    }
}

// ── Badge ────────────────────────────────────────────────────────────────────

struct BadgeComponent;

impl MdxComponent for BadgeComponent {
    fn render(&self, props: &HashMap<String, String>, _children: &str) -> String {
        let color = props.get("color").map(|s| s.as_str()).unwrap_or("blue");
        let text = props.get("text").map(|s| s.as_str()).unwrap_or("");
        let text = if text.is_empty() {
            _children.trim()
        } else {
            text
        };

        format!(
            r#"<span class="badge badge-{color}">{text}</span>"#,
            color = color,
            text = escape_html(text),
        )
    }
}

// ── Frame ────────────────────────────────────────────────────────────────────

struct FrameComponent;

impl MdxComponent for FrameComponent {
    fn render(&self, props: &HashMap<String, String>, _children: &str) -> String {
        let src = props.get("src").map(|s| s.as_str()).unwrap_or("");
        let caption = props.get("caption").map(|s| s.as_str());

        let caption_html = match caption {
            Some(c) if !c.is_empty() => {
                format!(
                    r#"<figcaption class="frame-caption">{}</figcaption>"#,
                    escape_html(c)
                )
            }
            _ => String::new(),
        };

        format!(
            r#"<figure class="frame">
<iframe src="{src}" loading="lazy"></iframe>
{caption}
</figure>"#,
            src = escape_html(src),
            caption = caption_html,
        )
    }
}

// ── Steps ────────────────────────────────────────────────────────────────────

struct StepsComponent;

impl MdxComponent for StepsComponent {
    fn render(&self, _props: &HashMap<String, String>, children: &str) -> String {
        format!(
            r#"<div class="steps">{children}</div>"#,
            children = children,
        )
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Escape HTML special characters
pub fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().chain(chars).collect(),
    }
}

fn parse_json_string_array(raw: &str) -> Vec<String> {
    let trimmed = raw.trim();
    if !trimmed.starts_with('[') || !trimmed.ends_with(']') {
        return vec![];
    }
    let inner = &trimmed[1..trimmed.len() - 1];
    inner
        .split(',')
        .map(|s| {
            let s = s.trim().trim_matches('"').trim_matches('\'');
            s.to_string()
        })
        .filter(|s| !s.is_empty())
        .collect()
}

fn split_tab_panels(children: &str, labels: &[String]) -> Vec<String> {
    let panels: Vec<&str> = children.split("---").collect();
    let mut result: Vec<String> = Vec::new();

    for (i, panel) in panels.iter().enumerate() {
        let content = panel.trim();
        if i < labels.len() {
            result.push(content.to_string());
        }
    }

    while result.len() < labels.len() {
        result.push(String::new());
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_callout_component() {
        let reg = ComponentRegistry::new();
        let callout = reg.get("Callout").unwrap();
        let mut props = HashMap::new();
        props.insert("type".to_string(), "warning".to_string());
        props.insert("title".to_string(), "Caution".to_string());
        let html = callout.render(&props, "<p>Be careful!</p>");
        assert!(html.contains("admonition-warning"));
        assert!(html.contains("Caution"));
        assert!(html.contains("Be careful!"));
    }

    #[test]
    fn test_callout_default_title() {
        let reg = ComponentRegistry::new();
        let callout = reg.get("Callout").unwrap();
        let props = HashMap::new();
        let html = callout.render(&props, "<p>Note content</p>");
        assert!(html.contains("admonition-title\">Note</p>"));
    }

    #[test]
    fn test_badge_component() {
        let reg = ComponentRegistry::new();
        let badge = reg.get("Badge").unwrap();
        let mut props = HashMap::new();
        props.insert("color".to_string(), "green".to_string());
        props.insert("text".to_string(), "Stable".to_string());
        let html = badge.render(&props, "");
        assert!(html.contains("badge-green"));
        assert!(html.contains("Stable"));
    }

    #[test]
    fn test_badge_from_children() {
        let reg = ComponentRegistry::new();
        let badge = reg.get("Badge").unwrap();
        let props = HashMap::new();
        let html = badge.render(&props, "Beta");
        assert!(html.contains("Beta"));
    }

    #[test]
    fn test_frame_component() {
        let reg = ComponentRegistry::new();
        let frame = reg.get("Frame").unwrap();
        let mut props = HashMap::new();
        props.insert("src".to_string(), "https://example.com".to_string());
        props.insert("caption".to_string(), "Example Site".to_string());
        let html = frame.render(&props, "");
        assert!(html.contains("iframe"));
        assert!(html.contains("https://example.com"));
        assert!(html.contains("Example Site"));
    }

    #[test]
    fn test_code_block_component() {
        let reg = ComponentRegistry::new();
        let code = reg.get("CodeBlock").unwrap();
        let mut props = HashMap::new();
        props.insert("lang".to_string(), "rust".to_string());
        props.insert("title".to_string(), "main.rs".to_string());
        let html = code.render(&props, "fn main() {}");
        assert!(html.contains("main.rs"));
        assert!(html.contains("language-rust"));
        assert!(html.contains("fn main() {}"));
    }

    #[test]
    fn test_steps_component() {
        let reg = ComponentRegistry::new();
        let steps = reg.get("Steps").unwrap();
        let props = HashMap::new();
        let html = steps.render(&props, "<p>Step 1</p><p>Step 2</p>");
        assert!(html.contains("class=\"steps\""));
        assert!(html.contains("Step 1"));
    }

    #[test]
    fn test_parse_json_string_array() {
        let arr = parse_json_string_array(r#"["Tab1", "Tab2"]"#);
        assert_eq!(arr, vec!["Tab1", "Tab2"]);
    }

    #[test]
    fn test_parse_json_string_array_empty() {
        let arr = parse_json_string_array("[]");
        assert!(arr.is_empty());
    }

    #[test]
    fn test_component_registry_has_builtins() {
        let reg = ComponentRegistry::new();
        assert!(reg.has("Callout"));
        assert!(reg.has("Tabs"));
        assert!(reg.has("CodeBlock"));
        assert!(reg.has("Badge"));
        assert!(reg.has("Frame"));
        assert!(reg.has("Steps"));
    }

    #[test]
    fn test_custom_component() {
        let mut reg = ComponentRegistry::new();
        struct Custom;
        impl MdxComponent for Custom {
            fn render(&self, _props: &HashMap<String, String>, _children: &str) -> String {
                "custom".to_string()
            }
        }
        reg.register("Custom", Box::new(Custom));
        assert!(reg.has("Custom"));
        let custom = reg.get("Custom").unwrap();
        assert_eq!(custom.render(&HashMap::new(), ""), "custom");
    }
}
