//! Markdown parsing with pulldown-cmark
//!
//! This module provides markdown parsing capabilities using pulldown-cmark
//! library, supporting CommonMark and GitHub Flavored Markdown (GFM).

use crate::error::{RendererError, RendererResult};
use crate::types::{MarkdownOptions, OutputFormat, RenderMetadata, RenderResult, RenderStats};
use pulldown_cmark::{html, Event, HeadingLevel, Options, Parser, Tag};
use regex::Regex;
use std::time::Instant;
use tracing::{debug, instrument};

/// Markdown parser for parsing and rendering markdown documents
pub struct MarkdownParser {
    /// Parsing options
    #[cfg(feature = "staging")]
    options: MarkdownOptions,
    /// Compiled pulldown-cmark options
    cmark_options: Options,
}

impl MarkdownParser {
    /// Create a new markdown parser with default options
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new markdown parser with custom options
    pub fn with_options(options: MarkdownOptions) -> Self {
        let cmark_options = Self::build_cmark_options(&options);
        Self {
            #[cfg(feature = "staging")]
            options,
            cmark_options,
        }
    }

    /// Build pulldown-cmark options from our MarkdownOptions
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

    /// Pre-process wikilinks [[target]] and [[target|display]] into markdown links
    fn preprocess_wikilinks(content: &str) -> String {
        let re = Regex::new(r"\[\[([^\]|]+)(?:\|([^\]]+))?\]\]").unwrap();
        let mut result = String::with_capacity(content.len());
        let mut in_code_block = false;

        for line in content.lines() {
            if line.trim_start().starts_with("```") {
                in_code_block = !in_code_block;
                result.push_str(line);
                result.push('\n');
                continue;
            }

            if in_code_block {
                result.push_str(line);
                result.push('\n');
            } else {
                let replaced = re.replace_all(line, |caps: &regex::Captures| {
                    let target: &str = &caps[1];
                    let display: &str = match caps.get(2) {
                        Some(m) => m.as_str(),
                        None => target,
                    };
                    let slug = target.to_lowercase().replace(' ', "-");
                    format!("[{}]({})", display, slug)
                });
                result.push_str(&replaced);
                result.push('\n');
            }
        }

        result.pop();
        result
    }

    /// Extract all wikilink targets from content (without converting)
    pub fn extract_wikilinks(content: &str) -> Vec<String> {
        let re = Regex::new(r"\[\[([^\]|]+)(?:\|([^\]]+))?\]\]").unwrap();
        let mut in_code_block = false;
        let mut targets = Vec::new();

        for line in content.lines() {
            if line.trim_start().starts_with("```") {
                in_code_block = !in_code_block;
                continue;
            }

            if !in_code_block {
                for caps in re.captures_iter(line) {
                    targets.push(caps[1].to_string());
                }
            }
        }

        targets
    }

    /// Parse markdown content
    #[instrument(skip(self, markdown), fields(format = ?format))]
    pub fn parse<S: AsRef<str>>(
        &self,
        markdown: S,
        format: OutputFormat,
    ) -> RendererResult<RenderResult> {
        let markdown = markdown.as_ref();
        let markdown_str = Self::preprocess_wikilinks(markdown);
        let start_time = Instant::now();

        debug!("Parsing markdown content ({} bytes)", markdown_str.len());

        let (content, metadata, stats) = match format {
            OutputFormat::Html => self.parse_to_html(&markdown_str)?,
            OutputFormat::PlainText => self.parse_to_plain_text(&markdown_str)?,
            OutputFormat::Ast => self.parse_to_ast(&markdown_str)?,
            OutputFormat::Markdown => {
                let metadata = self.extract_metadata(&markdown_str);
                let stats = RenderStats::new()
                    .with_render_time(start_time.elapsed())
                    .with_output_size(markdown_str.len());
                (markdown_str.to_string(), metadata, stats)
            }
        };

        let render_time = start_time.elapsed();
        let stats = stats
            .with_render_time(render_time)
            .with_output_size(content.len());

        debug!(
            "Parsed markdown in {}ms, output {} bytes",
            render_time.as_millis(),
            content.len()
        );

        Ok(RenderResult::new(content, format)
            .with_metadata(metadata)
            .with_stats(stats))
    }

    /// Parse markdown to HTML
    fn parse_to_html(
        &self,
        markdown: &str,
    ) -> RendererResult<(String, RenderMetadata, RenderStats)> {
        let parser = Parser::new_ext(markdown, self.cmark_options);

        // Extract metadata while parsing
        let metadata = self.extract_metadata(markdown);
        let mut stats = RenderStats::new();

        // Count code blocks during parsing
        let parser_with_count = parser.inspect(|event| {
            if matches!(event, Event::Start(Tag::CodeBlock(_))) {
                // We can't mutate stats here directly, but we can count later
            }
        });

        // Render to HTML
        let mut html_output = String::with_capacity(markdown.len() * 2);
        html::push_html(&mut html_output, parser_with_count);

        // Sanitize HTML output to prevent XSS attacks.
        // Allows safe elements (headings, paragraphs, lists, tables, code blocks, links, images)
        // while stripping script tags, event handlers, and other dangerous content.
        html_output = ammonia::Builder::default()
            .add_tags(["img"])
            .add_tag_attributes("img", ["src", "alt", "title", "width", "height", "loading"])
            .clean(&html_output)
            .to_string();

        // Count code blocks from original parse
        let parser_for_count = Parser::new_ext(markdown, self.cmark_options);
        for event in parser_for_count {
            if matches!(event, Event::Start(Tag::CodeBlock(_))) {
                stats.increment_code_blocks();
            }
        }

        Ok((html_output, metadata, stats))
    }

    /// Parse markdown to plain text
    fn parse_to_plain_text(
        &self,
        markdown: &str,
    ) -> RendererResult<(String, RenderMetadata, RenderStats)> {
        let parser = Parser::new_ext(markdown, self.cmark_options);
        let metadata = self.extract_metadata(markdown);
        let stats = RenderStats::new();

        let mut plain_text = String::with_capacity(markdown.len());

        for event in parser {
            match event {
                Event::Text(text) => {
                    plain_text.push_str(&text);
                }
                Event::Code(code) => {
                    plain_text.push_str(&code);
                }
                Event::SoftBreak | Event::HardBreak => {
                    plain_text.push('\n');
                }
                Event::End(Tag::Paragraph) | Event::End(Tag::Heading(..)) => {
                    plain_text.push_str("\n\n");
                }
                _ => {}
            }
        }

        // Clean up extra whitespace
        let plain_text = plain_text
            .lines()
            .map(|line| line.trim())
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string();

        Ok((plain_text, metadata, stats))
    }

    /// Parse markdown to AST representation (JSON)
    fn parse_to_ast(
        &self,
        markdown: &str,
    ) -> RendererResult<(String, RenderMetadata, RenderStats)> {
        let parser = Parser::new_ext(markdown, self.cmark_options);
        let metadata = self.extract_metadata(markdown);
        let stats = RenderStats::new();

        let mut events = Vec::new();
        for event in parser {
            let event_str = match &event {
                Event::Start(tag) => format!("Start: {:?}", tag),
                Event::End(tag) => format!("End: {:?}", tag),
                Event::Text(text) => format!("Text: {}", text),
                Event::Code(code) => format!("Code: {}", code),
                Event::Html(html) => format!("Html: {}", html),
                Event::FootnoteReference(name) => format!("FootnoteReference: {}", name),
                Event::SoftBreak => "SoftBreak".to_string(),
                Event::HardBreak => "HardBreak".to_string(),
                Event::Rule => "Rule".to_string(),
                Event::TaskListMarker(checked) => format!("TaskListMarker: {}", checked),
            };
            events.push(event_str);
        }

        let ast = serde_json::to_string_pretty(&events)
            .map_err(|e| RendererError::serialization(e.to_string()))?;

        Ok((ast, metadata, stats))
    }

    /// Extract metadata from markdown content
    fn extract_metadata(&self, markdown: &str) -> RenderMetadata {
        let mut metadata = RenderMetadata::new();
        let _parser = Parser::new_ext(markdown, self.cmark_options);

        let mut word_count = 0;
        let mut char_count = 0;
        let mut heading_count = 0;
        let mut code_block_count = 0;
        let mut _in_code_block = false;
        let mut first_heading: Option<String> = None;

        for event in Parser::new_ext(markdown, self.cmark_options) {
            match &event {
                Event::Start(Tag::CodeBlock(_)) => {
                    code_block_count += 1;
                    _in_code_block = true;
                }
                Event::End(Tag::CodeBlock(_)) => {
                    _in_code_block = false;
                }
                Event::Start(Tag::Heading(level, _, _)) => {
                    heading_count += 1;
                    if first_heading.is_none() && *level == HeadingLevel::H1 {
                        // Next text event will be the title
                    }
                }
                Event::Text(text) => {
                    char_count += text.len();
                    word_count += text.split_whitespace().count();

                    // Use first H1 as title
                    if first_heading.is_none() {
                        // Simple heuristic: first text in document is often the title
                        if !text.trim().is_empty() {
                            first_heading = Some(text.chars().take(100).collect());
                        }
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

impl Default for MarkdownParser {
    fn default() -> Self {
        Self::with_options(MarkdownOptions::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_markdown() {
        let parser = MarkdownParser::new();
        let result = parser
            .parse("# Hello World\n\nThis is a test.", OutputFormat::Html)
            .unwrap();

        assert!(result.content.contains("<h1>"));
        assert!(result.content.contains("Hello World"));
        assert_eq!(result.format, OutputFormat::Html);
    }

    #[test]
    fn test_parse_to_plain_text() {
        let parser = MarkdownParser::new();
        let result = parser
            .parse("# Hello World\n\nThis is a test.", OutputFormat::PlainText)
            .unwrap();

        assert!(result.content.contains("Hello World"));
        assert!(result.content.contains("This is a test"));
        assert!(!result.content.contains("<"));
    }

    #[test]
    fn test_parse_to_ast() {
        let parser = MarkdownParser::new();
        let result = parser.parse("# Hello", OutputFormat::Ast).unwrap();

        assert!(result.content.contains("Start"));
        assert!(result.content.contains("Heading"));
    }

    #[test]
    fn test_metadata_extraction() {
        let parser = MarkdownParser::new();
        let result = parser
            .parse("# Document Title\n\nSome content here.", OutputFormat::Html)
            .unwrap();

        assert!(result.metadata.title.is_some());
        assert_eq!(result.metadata.heading_count, 1);
        assert!(result.metadata.word_count > 0);
    }

    #[test]
    fn test_code_block_counting() {
        let parser = MarkdownParser::new();
        let markdown = r#"
# Code Example

```rust
fn main() {
    println!("Hello");
}
```

Some more text.
"#;
        let result = parser.parse(markdown, OutputFormat::Html).unwrap();

        assert_eq!(result.metadata.code_block_count, 1);
    }

    #[test]
    fn test_gfm_features() {
        let parser = MarkdownParser::new();
        let markdown = "| Col1 | Col2 |\n|------|------|\n| A | B |";
        let result = parser.parse(markdown, OutputFormat::Html).unwrap();

        assert!(result.content.contains("<table"));
    }

    #[test]
    fn test_pass_through() {
        let parser = MarkdownParser::new();
        let markdown = "# Hello";
        let result = parser.parse(markdown, OutputFormat::Markdown).unwrap();

        assert_eq!(result.content, markdown);
    }

    #[test]
    fn test_preprocess_wikilinks_basic() {
        let result = MarkdownParser::preprocess_wikilinks("See [[Hello]] for details");
        assert_eq!(result, "See [Hello](hello) for details");
    }

    #[test]
    fn test_preprocess_wikilinks_with_display() {
        let result = MarkdownParser::preprocess_wikilinks("Click [[Hello|Click here]] now");
        assert_eq!(result, "Click [Click here](hello) now");
    }

    #[test]
    fn test_preprocess_wikilinks_in_code_block() {
        let input = "Before\n```\n[[Hello]]\n```\nAfter [[World]]";
        let result = MarkdownParser::preprocess_wikilinks(input);
        assert!(
            result.contains("[[Hello]]"),
            "wikilink inside code block should NOT be converted"
        );
        assert!(
            result.contains("[World](world)"),
            "wikilink outside code block should be converted"
        );
    }

    #[test]
    fn test_preprocess_wikilinks_multiple() {
        let input = "Check [[Alpha]], [[Beta|the beta doc]], and [[Gamma]]";
        let result = MarkdownParser::preprocess_wikilinks(input);
        assert_eq!(
            result,
            "Check [Alpha](alpha), [the beta doc](beta), and [Gamma](gamma)"
        );
    }

    #[test]
    fn test_extract_wikilinks() {
        let input = "Link to [[Foo]] and [[Bar|display]]\n```\n[[Ignored]]\n```\n[[After]]";
        let targets = MarkdownParser::extract_wikilinks(input);
        assert_eq!(targets, vec!["Foo", "Bar", "After"]);
    }

    #[test]
    fn test_image_rendering() {
        let parser = MarkdownParser::new();
        let markdown = "![alt text](https://example.com/image.png)";
        let result = parser.parse(markdown, OutputFormat::Html).unwrap();

        assert!(
            result.content.contains("<img"),
            "Expected HTML to contain <img tag, got: {}",
            result.content
        );
        assert!(
            result.content.contains("alt=\"alt text\""),
            "Expected img to preserve alt attribute"
        );
        assert!(
            result
                .content
                .contains("src=\"https://example.com/image.png\""),
            "Expected img to preserve src attribute"
        );
    }
}
