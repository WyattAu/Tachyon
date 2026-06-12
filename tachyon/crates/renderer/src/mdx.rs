//! MDX Parser
//!
//! Parses MDX syntax (Markdown + JSX) and converts it to HTML.
//! Handles:
//! - JSX components: `<Component prop="value">children</Component>`
//! - JSX expressions: `{expression}` (simplified to inline content)
//! - MDX imports: `import Component from './Component'` (stripped, not rendered)
//! - Standard Markdown via pulldown-cmark

use crate::error::RendererResult;
use crate::mdx_components::ComponentRegistry;
use crate::types::{MarkdownOptions, OutputFormat, RenderMetadata, RenderResult, RenderStats};
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, html};
use regex::Regex;
use std::cell::Cell;
use std::collections::HashMap;
use std::sync::LazyLock;
use tracing::{debug, instrument};

static MDX_IMPORT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"^import\s+\w+\s+from\s+['\"].*?['\"];\s*$"#).unwrap());

static MDX_COMPONENT_OPEN_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^<(\w+)\s*(.*?)>(.*?)</\1>\s*$").unwrap()
});

static MDX_COMPONENT_SELF_CLOSING_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^<(\w+)\s*(.*?)/>\s*$").unwrap()
});

static MDX_PROP_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(\w+)=(?:"([^"]*)"|'([^']*)')"#).unwrap());

/// MDX Parser for parsing MDX content to HTML
pub struct MdxParser {
    component_registry: ComponentRegistry,
    cmark_options: Options,
}

impl MdxParser {
    /// Create a new MDX parser
    pub fn new() -> Self {
        Self::with_options(MarkdownOptions::default())
    }

    /// Create a new MDX parser with custom options
    pub fn with_options(options: MarkdownOptions) -> Self {
        let cmark_options = Self::build_cmark_options(&options);
        Self {
            component_registry: ComponentRegistry::new(),
            cmark_options,
        }
    }

    fn build_cmark_options(opts: &MarkdownOptions) -> Options {
        let mut options = Options::empty();
        if opts.enable_gfm {
            options.insert(Options::ENABLE_STRIKETHROUGH);
            options.insert(Options::ENABLE_TABLES);
            options.insert(Options::ENABLE_TASKLISTS);
        }
        if opts.enable_footnotes {
            options.insert(Options::ENABLE_FOOTNOTES);
        }
        if opts.enable_strikethrough && !opts.enable_gfm {
            options.insert(Options::ENABLE_STRIKETHROUGH);
        }
        if opts.enable_tables && !opts.enable_gfm {
            options.insert(Options::ENABLE_TABLES);
        }
        if opts.enable_task_lists && !opts.enable_gfm {
            options.insert(Options::ENABLE_TASKLISTS);
        }
        if opts.enable_smart_punctuation {
            options.insert(Options::ENABLE_SMART_PUNCTUATION);
        }
        if opts.enable_heading_attributes {
            options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
        }
        options
    }

    /// Check if content contains MDX syntax (JSX components or expressions)
    pub fn is_mdx(content: &str) -> bool {
        for line in content.lines() {
            let trimmed = line.trim();
            if MDX_IMPORT_RE.is_match(trimmed) {
                return true;
            }
            if trimmed.starts_with('<')
                && !trimmed.starts_with("<!")
                && !trimmed.starts_with("<br")
                && !trimmed.starts_with("<hr")
                && !trimmed.starts_with("<img")
                && !trimmed.starts_with("<a ")
                && !trimmed.starts_with("<p")
                && !trimmed.starts_with("<h")
                && !trimmed.starts_with("<pre")
                && !trimmed.starts_with("<code")
                && !trimmed.starts_with("<div")
                && !trimmed.starts_with("<span")
                && !trimmed.starts_with("<table")
                && !trimmed.starts_with("<ul")
                && !trimmed.starts_with("<ol")
                && !trimmed.starts_with("<li")
                && !trimmed.starts_with("<blockquote")
            {
                if MDX_COMPONENT_OPEN_RE.is_match(trimmed)
                    || MDX_COMPONENT_SELF_CLOSING_RE.is_match(trimmed)
                {
                    return true;
                }
            }
        }
        false
    }

    /// Parse MDX content to HTML
    #[instrument(skip(self, mdx), fields(format = ?format))]
    pub fn parse<S: AsRef<str>>(
        &self,
        mdx: S,
        format: OutputFormat,
    ) -> RendererResult<RenderResult> {
        let mdx = mdx.as_ref();
        let start_time = std::time::Instant::now();

        debug!("Parsing MDX content ({} bytes)", mdx.len());

        // Strip imports
        let cleaned = self.strip_imports(mdx);

        // Process MDX components to placeholders, then render markdown, then replace
        let (with_placeholders, placeholders) = self.extract_components(&cleaned);

        let parser = Parser::new_ext(&with_placeholders, self.cmark_options);
        let mut html_output = String::with_capacity(mdx.len() * 2);

        let code_block_count = Cell::new(0u32);
        let parser_with_count = parser.inspect(|event| {
            if matches!(event, Event::Start(Tag::CodeBlock(_))) {
                code_block_count.set(code_block_count.get() + 1);
            }
        });

        html::push_html(&mut html_output, parser_with_count);

        // Sanitize
        html_output = ammonia::Builder::default()
            .add_tags([
                "img", "pre", "code", "span", "div", "a", "figure", "figcaption", "iframe",
            ])
            .add_generic_attributes(&["class"])
            .add_tag_attributes("img", ["src", "alt", "title", "width", "height", "loading"])
            .add_tag_attributes("a", ["href", "title"])
            .add_tag_attributes("iframe", ["src", "loading"])
            .clean(&html_output)
            .to_string();

        // Replace placeholders with rendered component HTML
        let html_output = self.replace_placeholders(&html_output, &placeholders);

        let metadata = self.extract_metadata(mdx);
        let mut stats = RenderStats::new();
        for _ in 0..code_block_count.get() {
            stats.increment_code_blocks();
        }

        let render_time = start_time.elapsed();
        stats = stats
            .with_render_time(render_time)
            .with_output_size(html_output.len());

        Ok(RenderResult::new(html_output, format)
            .with_metadata(metadata)
            .with_stats(stats))
    }

    /// Strip MDX import statements
    fn strip_imports(&self, content: &str) -> String {
        content
            .lines()
            .filter(|line| !MDX_IMPORT_RE.is_match(line.trim()))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Extract MDX components and replace with placeholders
    fn extract_components(&self, content: &str) -> (String, Vec<String>) {
        let mut placeholders = Vec::new();
        let mut result = content.to_string();

        // Process self-closing components first
        loop {
            let mut found = false;
            if let Some(caps) = MDX_COMPONENT_SELF_CLOSING_RE.captures(&result.clone()) {
                let tag = caps[1].to_string();
                let props_str = caps[2].to_string();
                let full_match = caps[0].to_string();
                let props = self.parse_props(&props_str);
                let html = if let Some(component) = self.component_registry.get(&tag) {
                    component.render(&props, "")
                } else {
                    format!(
                        r#"<div class="mdx-component mdx-{}">Unknown component: {}</div>"#,
                        tag.to_lowercase(),
                        tag,
                    )
                };
                let idx = placeholders.len();
                placeholders.push(html);
                result = result.replace(&full_match, &format!("MDXPLACEHOLDER{}", idx));
                found = true;
            }
            if !found {
                break;
            }
        }

        // Process open/close component pairs (may be nested, but we handle simple cases)
        loop {
            let mut found = false;
            if let Some(caps) = MDX_COMPONENT_OPEN_RE.captures(&result.clone()) {
                let tag = caps[1].to_string();
                let props_str = caps[2].to_string();
                let children = caps[3].to_string();
                let full_match = caps[0].to_string();
                let props = self.parse_props(&props_str);
                let html = if let Some(component) = self.component_registry.get(&tag) {
                    component.render(&props, &children)
                } else {
                    format!(
                        r#"<div class="mdx-component mdx-{}">{}</div>"#,
                        tag.to_lowercase(),
                        children,
                    )
                };
                let idx = placeholders.len();
                placeholders.push(html);
                result = result.replace(&full_match, &format!("MDXPLACEHOLDER{}", idx));
                found = true;
            }
            if !found {
                break;
            }
        }

        // Handle JSX expressions {expression} - replace with placeholder
        let expr_re = Regex::new(r"\{([^{}]+)\}").unwrap();
        let result = expr_re.replace_all(&result, |caps: &regex::Captures| {
            let expr = caps[1].trim();
            // Simple string literals
            if (expr.starts_with('"') && expr.ends_with('"'))
                || (expr.starts_with('\'') && expr.ends_with('\''))
            {
                let inner = &expr[1..expr.len() - 1];
                let idx = placeholders.len();
                placeholders.push(inner.to_string());
                format!("MDXPLACEHOLDER{}", idx)
            } else {
                // For non-string expressions, render as escaped text
                let idx = placeholders.len();
                placeholders.push(format!(
                    r#"<span class="mdx-expression">{}</span>"#,
                    crate::mdx_components::escape_html(expr)
                ));
                format!("MDXPLACEHOLDER{}", idx)
            }
        });

        (result.to_string(), placeholders)
    }

    /// Parse JSX props from a string like `prop="value" other='value'`
    fn parse_props(&self, props_str: &str) -> HashMap<String, String> {
        let mut props = HashMap::new();
        for caps in MDX_PROP_RE.captures_iter(props_str) {
            let key = caps[1].to_string();
            let value = caps
                .get(2)
                .or_else(|| caps.get(3))
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            props.insert(key, value);
        }
        props
    }

    /// Replace MDXPLACEHOLDERn with actual component HTML
    fn replace_placeholders(&self, html: &str, placeholders: &[String]) -> String {
        let mut result = html.to_string();
        for (i, component_html) in placeholders.iter().enumerate() {
            let placeholder = format!("MDXPLACEHOLDER{}", i);
            result = result.replace(&placeholder, component_html);
        }
        result
    }

    /// Extract metadata from MDX content
    fn extract_metadata(&self, mdx: &str) -> RenderMetadata {
        let mut metadata = RenderMetadata::new();
        let mut word_count = 0;
        let mut char_count = 0;
        let mut heading_count = 0;
        let mut code_block_count = 0;
        let mut first_heading: Option<String> = None;

        for event in Parser::new_ext(mdx, self.cmark_options) {
            match &event {
                Event::Start(Tag::CodeBlock(_)) => {
                    code_block_count += 1;
                }
                Event::Start(Tag::Heading(level, _, _)) => {
                    heading_count += 1;
                    if first_heading.is_none() && *level == HeadingLevel::H1 {}
                }
                Event::Text(text) => {
                    char_count += text.len();
                    word_count += text.split_whitespace().count();
                    if first_heading.is_none() && !text.trim().is_empty() {
                        first_heading = Some(text.chars().take(100).collect());
                    }
                }
                _ => {}
            }
        }

        metadata.title = first_heading;
        metadata.word_count = word_count;
        metadata.char_count = char_count;
        metadata.heading_count = heading_count;
        metadata.code_block_count = code_block_count;

        metadata
    }
}

impl Default for MdxParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_mdx_with_import() {
        assert!(MdxParser::is_mdx(
            "import Callout from './Callout';"
        ));
    }

    #[test]
    fn test_is_mdx_with_component() {
        assert!(MdxParser::is_mdx("<Callout type=\"note\">Text</Callout>"));
    }

    #[test]
    fn test_is_mdx_self_closing() {
        assert!(MdxParser::is_mdx("<Badge text=\"Stable\" />"));
    }

    #[test]
    fn test_is_not_mdx_plain_markdown() {
        assert!(!MdxParser::is_mdx("# Hello World\n\nPlain markdown."));
    }

    #[test]
    fn test_is_not_mdx_html_tags() {
        assert!(!MdxParser::is_mdx("<div>Hello</div>"));
        assert!(!MdxParser::is_mdx("<p>Text</p>"));
        assert!(!MdxParser::is_mdx("<img src=\"url\">"));
    }

    #[test]
    fn test_strip_imports() {
        let parser = MdxParser::new();
        let content = "import Foo from './Foo';\n\n# Title\n\nContent.";
        let result = parser.strip_imports(content);
        assert!(!result.contains("import"));
        assert!(result.contains("# Title"));
    }

    #[test]
    fn test_parse_simple_mdx() {
        let parser = MdxParser::new();
        let mdx = "# Hello MDX\n\nThis is **MDX** content.";
        let result = parser.parse(mdx, OutputFormat::Html).unwrap();
        assert!(result.content.contains("Hello MDX"));
        assert!(result.content.contains("<strong>MDX</strong>"));
    }

    #[test]
    fn test_parse_mdx_with_callout() {
        let parser = MdxParser::new();
        let mdx = r#"import Callout from './Callout';

# Guide

<Callout type="warning" title="Caution">
<p>Be careful with this!</p>
</Callout>"#;
        let result = parser.parse(mdx, OutputFormat::Html).unwrap();
        assert!(result.content.contains("admonition-warning"));
        assert!(result.content.contains("Caution"));
        assert!(result.content.contains("Be careful with this!"));
        assert!(!result.content.contains("import"));
    }

    #[test]
    fn test_parse_mdx_with_badge() {
        let parser = MdxParser::new();
        let mdx = "# API\n\nStatus: <Badge color=\"green\" text=\"Stable\" />";
        let result = parser.parse(mdx, OutputFormat::Html).unwrap();
        assert!(result.content.contains("badge-green"));
        assert!(result.content.contains("Stable"));
    }

    #[test]
    fn test_parse_mdx_with_code_block() {
        let parser = MdxParser::new();
        let mdx = r#"<CodeBlock lang="rust" title="main.rs">fn main() {}</CodeBlock>"#;
        let result = parser.parse(mdx, OutputFormat::Html).unwrap();
        assert!(result.content.contains("main.rs"));
        assert!(result.content.contains("language-rust"));
    }

    #[test]
    fn test_parse_mdx_with_frame() {
        let parser = MdxParser::new();
        let mdx = r#"<Frame src="https://example.com" caption="Example" />"#;
        let result = parser.parse(mdx, OutputFormat::Html).unwrap();
        assert!(result.content.contains("iframe"));
        assert!(result.content.contains("Example"));
    }

    #[test]
    fn test_parse_mdx_with_steps() {
        let parser = MdxParser::new();
        let mdx = r#"<Steps>
<p>Step 1: Install</p>
<p>Step 2: Configure</p>
</Steps>"#;
        let result = parser.parse(mdx, OutputFormat::Html).unwrap();
        assert!(result.content.contains("class=\"steps\""));
        assert!(result.content.contains("Step 1"));
    }

    #[test]
    fn test_parse_mdx_unknown_component() {
        let parser = MdxParser::new();
        let mdx = "<Unknown>Content</Unknown>";
        let result = parser.parse(mdx, OutputFormat::Html).unwrap();
        assert!(result.content.contains("Unknown component"));
    }

    #[test]
    fn test_parse_mdx_with_expression() {
        let parser = MdxParser::new();
        let mdx = "# Title\n\nValue: {\"hello\"}";
        let result = parser.parse(mdx, OutputFormat::Html).unwrap();
        assert!(result.content.contains("hello"));
    }

    #[test]
    fn test_parse_metadata() {
        let parser = MdxParser::new();
        let mdx = "# Document Title\n\nSome content with multiple words.";
        let result = parser.parse(mdx, OutputFormat::Html).unwrap();
        assert!(result.metadata.title.is_some());
        assert!(result.metadata.word_count > 0);
    }

    #[test]
    fn test_fallback_to_markdown() {
        let parser = MdxParser::new();
        let mdx = "# Hello\n\nJust **markdown** content.";
        let result = parser.parse(mdx, OutputFormat::Html).unwrap();
        assert!(result.content.contains("<h1>"));
        assert!(result.content.contains("<strong>markdown</strong>"));
    }

    #[test]
    fn test_extract_components_multi() {
        let parser = MdxParser::new();
        let content = "<Badge text=\"A\" /><Badge text=\"B\" />";
        let (cleaned, placeholders) = parser.extract_components(content);
        assert_eq!(placeholders.len(), 2);
        assert!(cleaned.contains("MDXPLACEHOLDER0"));
        assert!(cleaned.contains("MDXPLACEHOLDER1"));
    }
}
