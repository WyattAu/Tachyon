//! Markdown parsing with pulldown-cmark
//!
//! This module provides markdown parsing capabilities using pulldown-cmark
//! library, supporting CommonMark and GitHub Flavored Markdown (GFM).
//!
//! ## Why no streaming/chunked rendering?
//!
//! pulldown-cmark is already an incremental, event-driven parser (it yields
//! `Event` items via a standard Rust `Iterator`). The `html::push_html` call
//! consumes this stream in a single pass with no intermediate buffering.
//! Benchmark data shows pulldown-cmark renders 1 MB of markdown in <100 ms on
//! modern hardware, so streaming only matters for documents >10 MB — far beyond
//! typical knowledge-base notes. Additionally, the `ammonia` XSS sanitizer
//! operates on the complete HTML string, making true chunked output incorrect
//! (tags can span chunk boundaries). Streaming would add complexity without
//! measurable benefit for this use case.

use crate::error::{RendererError, RendererResult};
use crate::types::{MarkdownOptions, OutputFormat, RenderMetadata, RenderResult, RenderStats};
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, html};
use regex::Regex;
use std::cell::Cell;
use std::sync::LazyLock;
use std::time::Instant;
use tracing::{debug, instrument};

// ============================================================================
// Static Regex Patterns (compiled once, zero per-call overhead)
// ============================================================================

static EMBED_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"!\{(\w+):\s*([^}]+)\}").unwrap());

static WIKILINK_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[\[([^\]|]+)(?:\|([^\]]+))?\]\]").unwrap());

static ADMONITION_HEADER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^>\s*\[!(\w+)\]").unwrap());

static ADMONITION_BODY_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^>\s?(.*)").unwrap());

// ============================================================================
// TOC & Embed Types
// ============================================================================

/// A single entry in the table of contents.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct TocEntry {
    /// Heading level (1-6).
    pub level: usize,
    /// Slugified heading ID for anchor links.
    pub slug: String,
    /// Heading text.
    pub text: String,
}

/// An embed block extracted from content.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct EmbedBlock {
    /// Embed type (youtube, vimeo, figma, mermaid, plantuml, codepen, github).
    pub kind: String,
    /// Embed identifier (video ID, file hash, diagram code, etc).
    pub id: String,
}

/// A block reference (transclusion) parsed from markdown.
/// Syntax: `![[doc-id]]` or `![[doc-id#heading]]` or `![[doc-id#^block-id]]`
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct BlockReference {
    /// The target document slug or ID.
    pub target: String,
    /// Optional heading to transclude within the target document.
    pub heading: Option<String>,
    /// Optional block-level reference (e.g., `^block-id`).
    pub block_id: Option<String>,
    /// Whether this is a "reference only" (`![[doc-id#^block-id]]`) vs. full embed.
    pub reference_only: bool,
}

// ============================================================================
// MarkdownParser
// ============================================================================

/// Markdown parser for parsing and rendering markdown documents
pub struct MarkdownParser {
    /// Parsing options
    #[cfg(feature = "staging")]
    _options: MarkdownOptions,
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
            _options: options,
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

    /// Extract table of contents headings from markdown content.
    ///
    /// Returns heading level (1-6), slug, and text for each heading found
    /// outside of code blocks.
    pub fn extract_toc(content: &str) -> Vec<TocEntry> {
        let mut entries = Vec::new();
        let mut in_code_block = false;

        for line in content.lines() {
            if line.trim_start().starts_with("```") {
                in_code_block = !in_code_block;
                continue;
            }
            if in_code_block {
                continue;
            }
            let trimmed = line.trim_start();
            if let Some(rest) = trimmed.strip_prefix('#') {
                let level = rest.len().min(6);
                if level > 0 && !rest.starts_with(' ') {
                    continue;
                }
                let text = rest.trim().to_string();
                if text.is_empty() {
                    continue;
                }
                let slug = text
                    .to_lowercase()
                    .chars()
                    .map(|c| {
                        if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                            c
                        } else {
                            '-'
                        }
                    })
                    .collect::<String>();
                entries.push(TocEntry { level, slug, text });
            }
        }
        entries
    }

    /// Extract embed blocks `!{type id}` from content.
    ///
    /// Recognized types: youtube, vimeo, figma, mermaid, plantuml, codepen, github.
    /// Skips content inside code blocks.
    pub fn extract_embeds(content: &str) -> Vec<EmbedBlock> {
        let mut embeds = Vec::new();
        let mut in_code_block = false;

        for line in content.lines() {
            if line.trim_start().starts_with("```") {
                in_code_block = !in_code_block;
                continue;
            }
            if !in_code_block {
                for caps in EMBED_RE.captures_iter(line) {
                    let kind = caps[1].to_lowercase();
                    let id = caps[2].trim().to_string();
                    if !id.is_empty() {
                        embeds.push(EmbedBlock { kind, id });
                    }
                }
            }
        }
        embeds
    }

    /// Pre-process admonition blocks `> [!type]` into HTML divs.
    ///
    /// Converts blocks like:
    /// ```markdown
    /// > [!note]
    /// > This is a note
    /// ```
    ///
    /// Into:
    /// ```html
    /// <div class="admonition admonition-note"><div class="admonition-title">Note</div><div class="admonition-content">
    /// This is a note
    /// </div></div>
    /// ```
    fn preprocess_admonitions(content: &str) -> String {
        let mut result = String::with_capacity(content.len());
        let mut in_code_block = false;
        let mut in_admonition = false;
        let mut admonition_type = String::new();
        let mut admonition_lines: Vec<String> = Vec::new();

        for line in content.lines() {
            if line.trim_start().starts_with("```") {
                if in_admonition {
                    result.push_str(&format_admonition_html(&admonition_type, &admonition_lines));
                    in_admonition = false;
                    admonition_lines.clear();
                }
                in_code_block = !in_code_block;
                result.push_str(line);
                result.push('\n');
                continue;
            }

            if in_code_block {
                result.push_str(line);
                result.push('\n');
                continue;
            }

            if !in_admonition {
                if let Some(caps) = ADMONITION_HEADER_RE.captures(line) {
                    in_admonition = true;
                    admonition_type = caps.get(1).unwrap().as_str().to_lowercase();
                    continue;
                }
            }

            if in_admonition {
                if let Some(caps) = ADMONITION_BODY_RE.captures(line) {
                    admonition_lines.push(caps.get(1).unwrap().as_str().to_string());
                    continue;
                } else {
                    result.push_str(&format_admonition_html(&admonition_type, &admonition_lines));
                    in_admonition = false;
                    admonition_lines.clear();
                }
            }

            result.push_str(line);
            result.push('\n');
        }

        if in_admonition {
            result.push_str(&format_admonition_html(&admonition_type, &admonition_lines));
        }

        if result.ends_with('\n') {
            result.pop();
        }

        result
    }

    /// Pre-process wikilinks [[target]] and [[target|display]] into HTML anchors.
    ///
    /// Converts to `<a href="/documents/{slug}" class="wikilink">{text}</a>`.
    /// Skips wikilinks inside code blocks.
    fn preprocess_wikilinks(content: &str) -> String {
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
                let replaced = WIKILINK_RE.replace_all(line, |caps: &regex::Captures| {
                    let target: &str = &caps[1];
                    let display: &str = match caps.get(2) {
                        Some(m) => m.as_str(),
                        None => target,
                    };
                    let slug = target
                        .to_lowercase()
                        .chars()
                        .map(|c| {
                            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                                c
                            } else {
                                '-'
                            }
                        })
                        .collect::<String>();
                    format!(
                        "<a href=\"/documents/{}\" class=\"wikilink\">{}</a>",
                        slug, display
                    )
                });
                result.push_str(&replaced);
                result.push('\n');
            }
        }

        result.pop();
        result
    }

    /// Extract block references (transclusions) from markdown content.
    ///
    /// Parses `![[target]]`, `![[target#heading]]`, `![[target#^block-id]]` syntax.
    /// Skips references inside code blocks and inline code.
    ///
    /// Returns the block references found and their positions.
    pub fn extract_block_references(&self, content: &str) -> Vec<(usize, BlockReference)> {
        let mut references = Vec::new();
        let mut in_code_block = false;
        let mut code_fence_marker = String::new();

        for (line_idx, line) in content.lines().enumerate() {
            if line.trim().starts_with("```") || line.trim().starts_with("~~~") {
                if !in_code_block {
                    in_code_block = true;
                    code_fence_marker = line.trim().chars().take(3).collect();
                } else if line.trim().starts_with(&code_fence_marker) {
                    in_code_block = false;
                    code_fence_marker.clear();
                }
                continue;
            }

            if in_code_block {
                continue;
            }

            let mut search_start = 0;
            let chars: Vec<char> = line.chars().collect();

            while search_start < chars.len() {
                if chars.get(search_start) == Some(&'`') {
                    let tick_count = count_consecutive(&chars[search_start..], '`');
                    let close_pos =
                        find_closing_backtick(&chars, search_start + tick_count, tick_count);
                    if let Some(pos) = close_pos {
                        search_start = pos + tick_count;
                    } else {
                        search_start = chars.len();
                    }
                    continue;
                }

                if search_start + 2 < chars.len()
                    && chars[search_start] == '!'
                    && chars[search_start + 1] == '['
                    && chars[search_start + 2] == '['
                {
                    let close = find_closing_brackets(&chars, search_start + 3, '[', ']');
                    if let Some(end_pos) = close {
                        let inner: String = chars[search_start + 3..end_pos].iter().collect();
                        if let Some(reference) = parse_block_reference(&inner) {
                            let offset = content[..]
                                .lines()
                                .take(line_idx)
                                .map(|l| l.len() + 1)
                                .sum::<usize>()
                                + search_start;
                            references.push((offset, reference));
                        }
                        search_start = end_pos + 2;
                    } else {
                        search_start += 1;
                    }
                } else {
                    search_start += 1;
                }
            }
        }

        references
    }

    /// Extract all wikilink targets from content (without converting)
    pub fn extract_wikilinks(content: &str) -> Vec<String> {
        let mut in_code_block = false;
        let mut targets = Vec::new();

        for line in content.lines() {
            if line.trim_start().starts_with("```") {
                in_code_block = !in_code_block;
                continue;
            }

            if !in_code_block {
                for caps in WIKILINK_RE.captures_iter(line) {
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
        let markdown = Self::preprocess_wikilinks(markdown);
        let markdown = Self::preprocess_admonitions(&markdown);
        let parser = Parser::new_ext(&markdown, self.cmark_options);

        let metadata = self.extract_metadata(&markdown);
        let mut stats = RenderStats::new();

        let code_block_count = Cell::new(0u32);
        let parser_with_count = parser.inspect(|event| {
            if matches!(event, Event::Start(Tag::CodeBlock(_))) {
                code_block_count.set(code_block_count.get() + 1);
            }
        });

        let mut html_output = String::with_capacity(markdown.len() * 2);
        html::push_html(&mut html_output, parser_with_count);

        html_output = ammonia::Builder::default()
            .add_tags(["img", "pre", "code", "span", "div", "a"])
            .add_generic_attributes(&["class"])
            .add_tag_attributes("img", ["src", "alt", "title", "width", "height", "loading"])
            .add_tag_attributes("a", ["href", "title"])
            .clean(&html_output)
            .to_string();

        for _ in 0..code_block_count.get() {
            stats.increment_code_blocks();
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

/// Format an admonition block as HTML.
fn format_admonition_html(admonition_type: &str, lines: &[String]) -> String {
    let title = match admonition_type {
        "note" => "Note",
        "tip" => "Tip",
        "info" => "Info",
        "warning" => "Warning",
        "danger" => "Danger",
        "caution" => "Caution",
        _ => admonition_type,
    };

    let content = lines.join("\n");

    format!(
        "<div class=\"admonition admonition-{type}\">\
         <div class=\"admonition-title\">{title}</div>\
         <div class=\"admonition-content\">{content}</div>\
         </div>",
        type = admonition_type,
        title = title,
        content = content
    )
}

/// Parse the inner content of a block reference `[[target]]`, `[[target#heading]]`, `[[target#^block-id]]`.
fn parse_block_reference(inner: &str) -> Option<BlockReference> {
    let inner = inner.trim();
    if inner.is_empty() {
        return None;
    }

    let (inner, reference_only) = if let Some(stripped) = inner.strip_prefix('!') {
        (stripped, true)
    } else {
        (inner, false)
    };

    let target;
    let mut heading = None;
    let mut block_id = None;

    if let Some(hash_pos) = inner.find('#') {
        target = inner[..hash_pos].trim().to_string();
        let fragment = &inner[hash_pos + 1..];

        if let Some(stripped) = fragment.strip_prefix('^') {
            block_id = Some(stripped.trim().to_string());
        } else {
            heading = Some(fragment.trim().to_string());
        }
    } else {
        target = inner.trim().to_string();
    }

    if target.is_empty() {
        return None;
    }

    Some(BlockReference {
        target,
        heading,
        block_id,
        reference_only,
    })
}

/// Count consecutive occurrences of a character at the start of a slice.
fn count_consecutive(chars: &[char], target: char) -> usize {
    chars.iter().take_while(|c| **c == target).count()
}

/// Find the closing backtick matching the opening tick count.
fn find_closing_backtick(chars: &[char], start: usize, tick_count: usize) -> Option<usize> {
    let mut pos = start;
    while pos + tick_count <= chars.len() {
        if chars[pos] == '`' && count_consecutive(&chars[pos..], '`') >= tick_count {
            return Some(pos);
        }
        pos += 1;
    }
    None
}

/// Find closing `]]` bracket pair.
fn find_closing_brackets(chars: &[char], start: usize, open: char, close: char) -> Option<usize> {
    let mut depth = 1i32;
    let mut pos = start;
    while pos < chars.len() {
        if chars[pos] == open {
            depth += 1;
        } else if chars[pos] == close {
            depth -= 1;
            if depth == 0 {
                return Some(pos);
            }
        }
        pos += 1;
    }
    None
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
        assert_eq!(
            result,
            r#"See <a href="/documents/hello" class="wikilink">Hello</a> for details"#
        );
    }

    #[test]
    fn test_preprocess_wikilinks_with_display() {
        let result = MarkdownParser::preprocess_wikilinks("Click [[Hello|Click here]] now");
        assert_eq!(
            result,
            r#"Click <a href="/documents/hello" class="wikilink">Click here</a> now"#
        );
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
            result.contains(r#"<a href="/documents/world" class="wikilink">World</a>"#),
            "wikilink outside code block should be converted to HTML anchor"
        );
    }

    #[test]
    fn test_preprocess_wikilinks_multiple() {
        let input = "Check [[Alpha]], [[Beta|the beta doc]], and [[Gamma]]";
        let result = MarkdownParser::preprocess_wikilinks(input);
        assert_eq!(
            result,
            concat!(
                r#"Check <a href="/documents/alpha" class="wikilink">Alpha</a>, "#,
                r#"<a href="/documents/beta" class="wikilink">the beta doc</a>, "#,
                r#"and <a href="/documents/gamma" class="wikilink">Gamma</a>"#
            )
        );
    }

    #[test]
    fn test_preprocess_wikilinks_slug_with_special_chars() {
        let result = MarkdownParser::preprocess_wikilinks("[[My Document Title]]");
        assert_eq!(
            result,
            r#"<a href="/documents/my-document-title" class="wikilink">My Document Title</a>"#
        );
    }

    #[test]
    fn test_extract_wikilinks() {
        let input = "Link to [[Foo]] and [[Bar|display]]\n```\n[[Ignored]]\n```\n[[After]]";
        let targets = MarkdownParser::extract_wikilinks(input);
        assert_eq!(targets, vec!["Foo", "Bar", "After"]);
    }

    // ── XSS Sanitization Tests ──────────────────────────────────────────

    #[test]
    fn test_xss_script_tag_stripped() {
        let parser = MarkdownParser::new();
        let markdown = r#"<script>alert("xss")</script>"#;
        let result = parser.parse(markdown, OutputFormat::Html).unwrap();

        assert!(
            !result.content.contains("<script"),
            "Script tags must be stripped by ammonia sanitization, got: {}",
            result.content
        );
        assert!(
            !result.content.contains("alert"),
            "Script content must be stripped, got: {}",
            result.content
        );
    }

    #[test]
    fn test_xss_event_handler_stripped() {
        let parser = MarkdownParser::new();
        // pulldown-cmark treats raw HTML as Event::Html, which ammonia then sanitizes
        let markdown = r#"<img src=x onerror="alert('xss')">"#;
        let result = parser.parse(markdown, OutputFormat::Html).unwrap();

        assert!(
            !result.content.contains("onerror"),
            "Event handlers must be stripped by ammonia, got: {}",
            result.content
        );
    }

    #[test]
    fn test_xss_javascript_uri_stripped() {
        let parser = MarkdownParser::new();
        let markdown = r#"[click me](javascript:alert('xss'))"#;
        let result = parser.parse(markdown, OutputFormat::Html).unwrap();

        assert!(
            !result.content.contains("javascript:"),
            "javascript: URIs must be stripped by ammonia, got: {}",
            result.content
        );
    }

    #[test]
    fn test_xss_iframe_stripped() {
        let parser = MarkdownParser::new();
        let markdown = r#"<iframe src="https://evil.com"></iframe>"#;
        let result = parser.parse(markdown, OutputFormat::Html).unwrap();

        assert!(
            !result.content.contains("<iframe"),
            "iframe tags must be stripped by ammonia, got: {}",
            result.content
        );
    }

    #[test]
    fn test_xss_svg_onload_stripped() {
        let parser = MarkdownParser::new();
        let markdown = r#"<svg onload="alert('xss')"><circle r="40"/></svg>"#;
        let result = parser.parse(markdown, OutputFormat::Html).unwrap();

        assert!(
            !result.content.contains("onload"),
            "SVG onload handlers must be stripped by ammonia, got: {}",
            result.content
        );
    }

    #[test]
    fn test_safe_content_preserved_after_sanitization() {
        let parser = MarkdownParser::new();
        let markdown = "# Hello\n\nParagraph with **bold** and *italic*.\n\n```rust\nfn main() {}\n```\n\n[link](https://example.com)";
        let result = parser.parse(markdown, OutputFormat::Html).unwrap();

        assert!(result.content.contains("<h1>"));
        assert!(result.content.contains("<strong>bold</strong>"));
        assert!(result.content.contains("<em>italic</em>"));
        assert!(result.content.contains("<code"));
        assert!(result.content.contains("href=\"https://example.com\""));
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

    // ── Block Reference Tests ───────────────────────────────────────────

    #[test]
    fn test_extract_block_references_basic() {
        let parser = MarkdownParser::new();
        let content = "See ![[design-specs]] for details.";
        let refs = parser.extract_block_references(content);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].1.target, "design-specs");
        assert_eq!(refs[0].1.heading, None);
        assert_eq!(refs[0].1.block_id, None);
        assert!(!refs[0].1.reference_only);
    }

    #[test]
    fn test_extract_block_references_with_heading() {
        let parser = MarkdownParser::new();
        let content = "Embed ![[api-docs#authentication]] here.";
        let refs = parser.extract_block_references(content);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].1.target, "api-docs");
        assert_eq!(refs[0].1.heading, Some("authentication".to_string()));
        assert_eq!(refs[0].1.block_id, None);
    }

    #[test]
    fn test_extract_block_references_with_block_id() {
        let parser = MarkdownParser::new();
        let content = "Reference ![[notes#^important-quote]] inline.";
        let refs = parser.extract_block_references(content);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].1.target, "notes");
        assert_eq!(refs[0].1.heading, None);
        assert_eq!(refs[0].1.block_id, Some("important-quote".to_string()));
    }

    #[test]
    fn test_extract_block_references_reference_only() {
        let parser = MarkdownParser::new();
        let content = "See ![[!design-specs]] for the original.";
        let refs = parser.extract_block_references(content);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].1.target, "design-specs");
        assert!(refs[0].1.reference_only);
    }

    #[test]
    fn test_extract_block_references_skips_code_blocks() {
        let parser = MarkdownParser::new();
        let content = "Before.\n```\n![[should-not-parse]]\n```\nAfter ![[real-ref]].";
        let refs = parser.extract_block_references(content);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].1.target, "real-ref");
    }

    #[test]
    fn test_extract_block_references_skips_inline_code() {
        let parser = MarkdownParser::new();
        let content = "Use `![[not-a-ref]]` literally, but ![[actual-ref]] embeds.";
        let refs = parser.extract_block_references(content);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].1.target, "actual-ref");
    }

    #[test]
    fn test_extract_block_references_multiple() {
        let parser = MarkdownParser::new();
        let content = "![[doc-a]] and ![[doc-b#intro]] and ![[doc-c#^key]]";
        let refs = parser.extract_block_references(content);
        assert_eq!(refs.len(), 3);
        assert_eq!(refs[0].1.target, "doc-a");
        assert_eq!(refs[1].1.target, "doc-b");
        assert_eq!(refs[1].1.heading, Some("intro".to_string()));
        assert_eq!(refs[2].1.target, "doc-c");
        assert_eq!(refs[2].1.block_id, Some("key".to_string()));
    }

    #[test]
    fn test_extract_block_references_empty() {
        let parser = MarkdownParser::new();
        let content = "No references here.";
        let refs = parser.extract_block_references(content);
        assert!(refs.is_empty());
    }

    #[test]
    fn test_parse_block_reference_invalid() {
        assert!(parse_block_reference("").is_none());
        assert!(parse_block_reference("#").is_none());
    }
}

#[cfg(test)]
mod test_ammonia_class {
    use crate::{RenderConfig, Renderer};

    #[test]
    fn test_ammonia_allows_class_on_code() {
        let html = r#"<pre><code class="language-json">{"key": "value"}</code></pre>"#;
        let cleaned = ammonia::Builder::default()
            .add_tags(["img", "pre", "code", "span", "div"])
            .add_generic_attributes(&["class"])
            .add_tag_attributes("img", ["src", "alt", "title", "width", "height", "loading"])
            .clean(html)
            .to_string();
        assert!(
            cleaned.contains(r#"class="language-json""#),
            "Expected class preserved, got: {}",
            cleaned
        );
    }

    #[test]
    fn test_code_block_preserves_class_attribute() {
        let md = r#"```json
{"key": "value"}
```"#;
        let config = RenderConfig::default();
        let renderer = Renderer::new(config);
        let result = renderer.render(md, None).unwrap();
        assert!(
            result.content.contains(r#"class="language-json""#),
            "Expected code block to have class=\"language-json\", got: {}",
            &result.content
        );
    }
}
