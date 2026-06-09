#![allow(dead_code)]

use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag};

#[derive(Clone)]
pub struct MarkdownHeading {
    pub level: u8,
    pub text: String,
    pub slug: String,
}

fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn slugify_wiki_link(text: &str) -> String {
    text.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c.to_lowercase().next().unwrap()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Placeholder prefix used to smuggle wikilink HTML through markdown rendering
/// without exposing raw HTML (XSS protection).
const WIKILINK_PREFIX: &str = "\u{200B}WIKILINK";

/// Escape any content that looks like our placeholder prefix to prevent collisions.
fn escape_wikilink_placeholders(input: &str) -> String {
    input.replace(WIKILINK_PREFIX, &format!("{}\u{FFFD}", WIKILINK_PREFIX))
}

fn preprocess_wikilinks(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.char_indices().peekable();
    let mut in_code_block = false;
    let mut link_index = 0u32;

    while let Some(&(i, c)) = chars.peek() {
        if c == '`' {
            // Count consecutive backticks
            let mut tick_count = 0u32;
            while let Some(&(_, next)) = chars.peek() {
                if next == '`' {
                    result.push(next);
                    chars.next();
                    tick_count += 1;
                } else {
                    break;
                }
            }
            if tick_count >= 3 {
                in_code_block = !in_code_block;
            }
            continue;
        }

        if in_code_block {
            result.push(c);
            chars.next();
            continue;
        }

        if c == '[' {
            let _start = i;
            chars.next();

            if let Some(&(_, next)) = chars.peek() {
                if next == '[' {
                    chars.next();

                    let mut link_content = String::new();
                    let mut found_end = false;

                    while let Some(&(_, ch)) = chars.peek() {
                        if ch == ']' {
                            chars.next();
                            if let Some(&(_, ch2)) = chars.peek() {
                                if ch2 == ']' {
                                    chars.next();
                                    found_end = true;
                                    break;
                                } else {
                                    link_content.push(']');
                                }
                            } else {
                                link_content.push(']');
                            }
                        } else {
                            link_content.push(ch);
                            chars.next();
                        }
                    }

                    if found_end {
                        let (target, display) = if let Some(pipe_pos) = link_content.find('|') {
                            let (t, d) = link_content.split_at(pipe_pos);
                            (t.trim(), d[1..].trim())
                        } else {
                            (link_content.as_str(), link_content.as_str())
                        };

                        let href_target = if let Some(hash_pos) = target.find('#') {
                            let (t, h) = target.split_at(hash_pos);
                            format!("/documents/{}#{}", slugify_wiki_link(t), slugify(&h[1..]))
                        } else {
                            format!("/documents/{}", slugify_wiki_link(target))
                        };

                        // Use zero-width space + placeholder to avoid markdown processing
                        // The ZWSP prevents markdown from interpreting this as HTML
                        result.push_str(&format!(
                            "{}{}|{}|{}{}",
                            WIKILINK_PREFIX, link_index, href_target, display, WIKILINK_PREFIX,
                        ));
                        link_index += 1;
                        continue;
                    } else {
                        result.push_str("[[");
                        result.push_str(&link_content);
                        continue;
                    }
                } else {
                    result.push('[');
                    continue;
                }
            } else {
                result.push('[');
                continue;
            }
        }

        result.push(c);
        chars.next();
    }

    result
}

fn restore_wikilinks(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut search_from = 0;

    while let Some(start) = html[search_from..].find(WIKILINK_PREFIX) {
        let abs_start = search_from + start;
        let after_open = abs_start + WIKILINK_PREFIX.len();

        // Find the closing WIKILINK_PREFIX after the opening
        if let Some(end_rel) = html[after_open..].find(WIKILINK_PREFIX) {
            let abs_end = after_open + end_rel;
            let inner = &html[after_open..abs_end];

            // Parse: index|href|display
            if let Some(pipe1) = inner.find('|') {
                let _idx = &inner[..pipe1];
                let after_idx = &inner[pipe1 + 1..];
                if let Some(pipe2) = after_idx.find('|') {
                    let href = &after_idx[..pipe2];
                    let display = &after_idx[pipe2 + 1..];
                    result.push_str(&html[search_from..abs_start]);
                    result.push_str(&format!(
                        r#"<a class="wikilink" href="{}">{}</a>"#,
                        href, display
                    ));
                    search_from = abs_end + WIKILINK_PREFIX.len();
                    continue;
                }
            }
        }

        // Not a valid wikilink placeholder — copy the char and move on
        result.push_str(&html[search_from..abs_start + 1]);
        search_from = abs_start + 1;
    }

    result.push_str(&html[search_from..]);
    result
}

pub fn render_markdown_to_html(markdown: &str) -> String {
    let escaped = escape_wikilink_placeholders(markdown);
    let preprocessed = preprocess_wikilinks(&escaped);

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);

    let parser = Parser::new_ext(&preprocessed, options);

    let mut html = String::with_capacity(markdown.len() * 2);
    let mut in_table_head = false;

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Heading(level, _fragment, _classes) => {
                    let lvl = match level {
                        HeadingLevel::H1 => "1",
                        HeadingLevel::H2 => "2",
                        HeadingLevel::H3 => "3",
                        HeadingLevel::H4 => "4",
                        HeadingLevel::H5 => "5",
                        HeadingLevel::H6 => "6",
                    };
                    html.push_str(&format!("<h{}>", lvl));
                }
                Tag::Paragraph => html.push_str("<p>"),
                Tag::BlockQuote => html.push_str("<blockquote>"),
                Tag::CodeBlock(kind) => {
                    let lang_class = match kind {
                        CodeBlockKind::Fenced(lang) => {
                            let lang_str = lang.as_ref();
                            if lang_str.is_empty() {
                                String::new()
                            } else {
                                format!(" class=\"language-{}\"", lang_str)
                            }
                        }
                        CodeBlockKind::Indented => String::new(),
                    };
                    html.push_str(&format!("<pre class=\"code-block\"><code{}>", lang_class));
                }
                Tag::List(Some(1)) => html.push_str("<ol>"),
                Tag::List(Some(start)) => html.push_str(&format!("<ol start=\"{}\">", start)),
                Tag::List(None) => html.push_str("<ul>"),
                Tag::Item => html.push_str("<li>"),
                Tag::Emphasis => html.push_str("<em>"),
                Tag::Strong => html.push_str("<strong>"),
                Tag::Strikethrough => html.push_str("<del>"),
                Tag::Link(_link_type, url, title) => {
                    html.push_str("<a href=\"");
                    html.push_str(url.as_ref());
                    if !title.is_empty() {
                        html.push_str("\" title=\"");
                        html.push_str(title.as_ref());
                    }
                    html.push_str("\">");
                }
                Tag::Image(_link_type, url, title) => {
                    html.push_str("<img src=\"");
                    html.push_str(url.as_ref());
                    html.push_str("\" alt=\"");
                    html.push_str(title.as_ref());
                    html.push_str("\" />");
                }
                Tag::Table(_alignment) => {
                    html.push_str("<div class=\"overflow-x-auto\"><table class=\"min-w-full\">")
                }
                Tag::TableHead => {
                    in_table_head = true;
                    html.push_str("<thead><tr>");
                }
                Tag::TableRow => html.push_str("<tr>"),
                Tag::TableCell => {
                    if in_table_head {
                        html.push_str("<th>");
                    } else {
                        html.push_str("<td>");
                    }
                }
                Tag::FootnoteDefinition(label) => {
                    html.push_str(&format!(
                        "<div class=\"footnote-definition\" id=\"fn-{}\">",
                        label
                    ));
                }
            },
            Event::End(tag) => match tag {
                Tag::Heading(level, _, _) => {
                    let lvl = match level {
                        HeadingLevel::H1 => "1",
                        HeadingLevel::H2 => "2",
                        HeadingLevel::H3 => "3",
                        HeadingLevel::H4 => "4",
                        HeadingLevel::H5 => "5",
                        HeadingLevel::H6 => "6",
                    };
                    html.push_str(&format!("</h{}>", lvl));
                }
                Tag::Paragraph => html.push_str("</p>"),
                Tag::BlockQuote => html.push_str("</blockquote>"),
                Tag::CodeBlock(_) => html.push_str("</code></pre>"),
                Tag::List(Some(_)) => html.push_str("</ol>"),
                Tag::List(None) => html.push_str("</ul>"),
                Tag::Item => html.push_str("</li>"),
                Tag::Emphasis => html.push_str("</em>"),
                Tag::Strong => html.push_str("</strong>"),
                Tag::Strikethrough => html.push_str("</del>"),
                Tag::Link(..) => html.push_str("</a>"),
                Tag::Image(..) => {}
                Tag::Table(_) => html.push_str("</table></div>"),
                Tag::TableHead => {
                    in_table_head = false;
                    html.push_str("</tr></thead>");
                }
                Tag::TableRow => html.push_str("</tr>"),
                Tag::TableCell => {
                    if in_table_head {
                        html.push_str("</th>");
                    } else {
                        html.push_str("</td>");
                    }
                }
                Tag::FootnoteDefinition(_) => {
                    html.push_str("</div>");
                }
            },
            Event::Text(text) => {
                html.push_str(text.as_ref());
            }
            Event::Code(text) => {
                html.push_str("<code>");
                html.push_str(text.as_ref());
                html.push_str("</code>");
            }
            Event::Html(text) => {
                // Strip raw HTML for XSS protection.
                // Wikilinks use ZWSP placeholders that survive as text events.
                let _ = text;
            }
            Event::SoftBreak => html.push('\n'),
            Event::HardBreak => html.push_str("<br />"),
            Event::Rule => html.push_str("<hr />"),
            Event::FootnoteReference(label) => {
                html.push_str(&format!(
                    "<sup class=\"footnote-reference\"><a href=\"#fn-{}\">[{}]</a></sup>",
                    label, label
                ));
            }
            Event::TaskListMarker(checked) => {
                if checked {
                    html.push_str("<input type=\"checkbox\" disabled checked /> ");
                } else {
                    html.push_str("<input type=\"checkbox\" disabled /> ");
                }
            }
        }
    }

    restore_wikilinks(&html)
}

pub fn extract_headings(markdown: &str) -> Vec<MarkdownHeading> {
    let options = Options::empty();
    let parser = Parser::new_ext(markdown, options);

    let mut headings = Vec::new();
    let mut current_heading_level: Option<u8> = None;
    let mut current_text = String::new();

    for event in parser {
        match event {
            Event::Start(Tag::Heading(level, _, _)) => {
                current_heading_level = Some(match level {
                    HeadingLevel::H1 => 1,
                    HeadingLevel::H2 => 2,
                    HeadingLevel::H3 => 3,
                    HeadingLevel::H4 => 4,
                    HeadingLevel::H5 => 5,
                    HeadingLevel::H6 => 6,
                });
                current_text.clear();
            }
            Event::End(Tag::Heading(..)) => {
                if let Some(level) = current_heading_level.take() {
                    if !current_text.is_empty() {
                        let slug = slugify(&current_text);
                        headings.push(MarkdownHeading {
                            level,
                            text: current_text.trim().to_string(),
                            slug,
                        });
                    }
                }
                current_text.clear();
            }
            Event::Text(text) =>
            {
                #[allow(clippy::collapsible_match)]
                if current_heading_level.is_some() {
                    current_text.push_str(text.as_ref());
                }
            }
            Event::Code(text) =>
            {
                #[allow(clippy::collapsible_match)]
                if current_heading_level.is_some() {
                    current_text.push_str(text.as_ref());
                }
            }
            _ => {}
        }
    }

    headings
}

pub fn count_words(markdown: &str) -> usize {
    let preprocessed = preprocess_wikilinks(markdown);
    let mut count = 0usize;
    let mut in_word = false;

    for c in preprocessed.chars() {
        if c.is_whitespace() || c == '\n' || c == '\r' {
            if in_word {
                count += 1;
                in_word = false;
            }
        } else if c.is_alphanumeric() || c == '_' || c == '-' || c == '\'' {
            in_word = true;
        } else if c == '`' {
            // Skip code fences for word counting
            continue;
        }
    }

    if in_word {
        count += 1;
    }

    count
}

pub fn count_characters(markdown: &str) -> usize {
    markdown.chars().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_markdown() {
        let md = "# Hello\n\nWorld";
        let html = render_markdown_to_html(md);
        assert!(html.contains("<h1>"));
        assert!(html.contains("Hello"));
        assert!(html.contains("<p>"));
        assert!(html.contains("World"));
    }

    #[test]
    fn test_code_block_with_language() {
        let md = "```rust\nfn main() {}\n```";
        let html = render_markdown_to_html(md);
        assert!(html.contains("language-rust"));
        assert!(html.contains("code-block"));
    }

    #[test]
    fn test_wikilink_simple() {
        let md = "See [[My Page]] for details";
        let html = render_markdown_to_html(md);
        assert!(html.contains("wikilink"));
        assert!(html.contains("my-page"));
    }

    #[test]
    fn test_wikilink_with_display() {
        let md = "See [[My Page|Custom Text]] for details";
        let html = render_markdown_to_html(md);
        assert!(html.contains("Custom Text"));
        assert!(html.contains("my-page"));
    }

    #[test]
    fn test_wikilink_with_heading() {
        let md = "See [[My Page#Section]]";
        let html = render_markdown_to_html(md);
        assert!(html.contains("#section"));
    }

    #[test]
    fn test_wikilink_not_in_code_block() {
        let md = "```\n[[Not a link]]\n```\n[[Real Link]]";
        let html = render_markdown_to_html(md);
        assert!(!html.contains("not-a-link"));
        assert!(html.contains("real-link"));
    }

    #[test]
    fn test_strikethrough() {
        let md = "~~deleted~~";
        let html = render_markdown_to_html(md);
        assert!(html.contains("<del>"));
        assert!(html.contains("deleted"));
    }

    #[test]
    fn test_task_list() {
        let md = "- [x] done\n- [ ] todo";
        let html = render_markdown_to_html(md);
        assert!(html.contains("type=\"checkbox\""));
        assert!(html.contains("checked"));
    }

    #[test]
    fn test_extract_headings() {
        let md = "# Title\n## Subtitle\n### Section";
        let headings = extract_headings(md);
        assert_eq!(headings.len(), 3);
        assert_eq!(headings[0].level, 1);
        assert_eq!(headings[1].level, 2);
        assert_eq!(headings[2].level, 3);
    }

    #[test]
    fn test_count_words() {
        let md = "Hello world this is a test";
        assert_eq!(count_words(md), 6);
    }

    #[test]
    fn test_count_characters() {
        let md = "abc";
        assert_eq!(count_characters(md), 3);
    }

    #[test]
    fn test_raw_html_stripped() {
        let md = "# Hello\n<script>alert('xss')</script>\nWorld";
        let html = render_markdown_to_html(md);
        assert!(!html.contains("<script>"));
        assert!(html.contains("World"));
    }

    #[test]
    fn test_table() {
        let md = "| a | b |\n|---|---|\n| 1 | 2 |";
        let html = render_markdown_to_html(md);
        let bytes = html.as_bytes();
        let table_pos = html.find("<table");
        assert!(
            table_pos.is_some(),
            "No <table> found. Bytes around pos 25-40: {:?}",
            &bytes[25..40.min(bytes.len())]
        );
        let table_str = &html[table_pos.unwrap()..];
        eprintln!("Table tag: {:?}", &table_str[..table_str.len().min(20)]);
        assert!(html.contains("<th>"));
        assert!(html.contains("<td>"));
    }

    #[test]
    fn test_footnotes() {
        let md = "Text with a footnote[^1].\n\n[^1]: The footnote text.";
        let html = render_markdown_to_html(md);
        assert!(html.contains("footnote"));
        assert!(html.contains("fn-1"));
    }

    #[test]
    fn test_inline_code() {
        let md = "Use `println!` for debugging";
        let html = render_markdown_to_html(md);
        assert!(html.contains("<code>println!</code>"));
    }

    #[test]
    fn test_horizontal_rule() {
        let md = "---";
        let html = render_markdown_to_html(md);
        assert!(html.contains("<hr"));
    }

    #[test]
    fn test_blockquote() {
        let md = "> This is a quote";
        let html = render_markdown_to_html(md);
        assert!(html.contains("<blockquote>"));
        assert!(html.contains("This is a quote"));
    }

    #[test]
    fn test_numbered_list() {
        let md = "1. First\n2. Second\n3. Third";
        let html = render_markdown_to_html(md);
        assert!(html.contains("<ol>"));
        assert!(html.contains("<li>First</li>"));
    }

    #[test]
    fn test_bullet_list() {
        let md = "- Item 1\n- Item 2";
        let html = render_markdown_to_html(md);
        assert!(html.contains("<ul>"));
        assert!(html.contains("<li>"));
    }

    #[test]
    fn test_image_tag() {
        let md = "![alt text](https://example.com/image.png)";
        let html = render_markdown_to_html(md);
        assert!(html.contains("src=\"https://example.com/image.png\""));
        assert!(html.contains("<img"));
    }

    #[test]
    fn test_autolink() {
        let md = "Visit [my site](https://example.com)";
        let html = render_markdown_to_html(md);
        assert!(html.contains("<a href=\"https://example.com\">"));
        assert!(html.contains("my site"));
    }

    #[test]
    fn test_wikilink_with_pipe_in_code_block() {
        let md = "```\nSome text | with pipe\n```\n[[Real Link|Display]]";
        let html = render_markdown_to_html(md);
        assert!(html.contains("Display"));
        assert!(html.contains("real-link"));
    }

    #[test]
    fn test_multiple_wikilinks() {
        let md = "See [[Page A]] and [[Page B|Custom B]] for info";
        let html = render_markdown_to_html(md);
        assert!(html.contains("page-a"));
        assert!(html.contains("page-b"));
        assert!(html.contains("Custom B"));
    }

    #[test]
    fn test_unclosed_wikilink_preserved() {
        let md = "This [[has no closing";
        let html = render_markdown_to_html(md);
        assert!(html.contains("[["));
    }

    #[test]
    fn test_extract_headings_with_code_in_heading() {
        let md = "# Heading with `code` inside";
        let headings = extract_headings(md);
        assert_eq!(headings.len(), 1);
        assert_eq!(headings[0].text, "Heading with code inside");
    }

    #[test]
    fn test_extract_headings_nested_heading_levels() {
        let md = "#### H4\n###### H6\n## H2\n##### H5";
        let headings = extract_headings(md);
        assert_eq!(headings.len(), 4);
        assert_eq!(headings[0].level, 4);
        assert_eq!(headings[1].level, 6);
    }

    #[test]
    fn test_count_words_with_wikilinks() {
        let md = "Hello [[My Page]] world";
        assert_eq!(count_words(md), 4);
    }

    #[test]
    fn test_count_words_empty() {
        assert_eq!(count_words(""), 0);
    }

    #[test]
    fn test_count_words_single_word() {
        assert_eq!(count_words("hello"), 1);
    }

    #[test]
    fn test_count_characters_unicode() {
        let md = "hello世界";
        assert_eq!(count_characters(md), 7);
    }

    #[test]
    fn test_empty_markdown() {
        let html = render_markdown_to_html("");
        assert!(html.is_empty());
    }
}
