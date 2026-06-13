//! Confluence macro-to-Markdown conversion.
//!
//! Maps Confluence structured macros to their Markdown equivalents:
//! - `code` -> fenced code block
//! - `info`/`warning`/`note`/`tip`/`caution` -> admonition block
//! - `expand` -> `<details><summary>` HTML
//! - `toc` -> auto-generated TOC placeholder
//! - `jira` -> Jira link
//! - `include` -> transclusion placeholder
//! - `panel` -> blockquote with title
//!
//! Confluence storage format uses `<ac:structured-macro ac:name="...">` elements
//! containing `<ac:parameter>` for parameters and `<ac:plain-text-body>` or
//! `<ac:rich-text-body>` for content.

use quick_xml::Reader;
use quick_xml::events::Event;
use std::collections::HashMap;

/// A parsed Confluence macro with its parameters and body content.
#[derive(Debug, Clone)]
pub struct ParsedMacro {
    pub name: String,
    pub parameters: HashMap<String, String>,
    pub body: String,
    pub body_is_rich: bool,
}

/// Convert Confluence storage format XHTML to Markdown, with enhanced
/// macro handling for structured macros.
///
/// This extends the basic conversion in `confluence_storage_to_markdown`
/// by properly rendering macro bodies into their markdown equivalents.
#[allow(clippy::collapsible_match, clippy::collapsible_if)]
pub fn convert_xhtml_to_markdown(html: &str) -> String {
    let mut md = String::with_capacity(html.len());
    let mut reader = Reader::from_str(html);
    let mut buf = Vec::new();

    let mut list_stack: Vec<char> = Vec::new();
    let mut in_pre = false;
    let mut in_code_block = false;
    let mut code_content = String::new();
    let mut code_language = String::new();
    let mut tag_stack: Vec<String> = Vec::new();
    let mut macro_stack: Vec<ParsedMacro> = Vec::new();
    let mut in_macro_param = false;
    let mut current_param_name = String::new();
    let mut in_plain_text_body = false;
    let mut in_rich_text_body = false;
    let mut rich_text_body_content = String::new();
    #[allow(unused_variables)]
    let in_table_header = false;
    #[allow(unused_variables)]
    let table_row_count = 0;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();

                match tag_name.as_str() {
                    "p" | "div" | "section" => {
                        if !in_pre && !in_code_block && !in_rich_text_body {
                            md.push('\n');
                        }
                    }
                    "br" => md.push('\n'),
                    "hr" => md.push_str("\n---\n"),
                    "h1" => {
                        md.push_str("\n# ");
                        tag_stack.push("h1".into());
                    }
                    "h2" => {
                        md.push_str("\n## ");
                        tag_stack.push("h2".into());
                    }
                    "h3" => {
                        md.push_str("\n### ");
                        tag_stack.push("h3".into());
                    }
                    "h4" => {
                        md.push_str("\n#### ");
                        tag_stack.push("h4".into());
                    }
                    "h5" => {
                        md.push_str("\n##### ");
                        tag_stack.push("h5".into());
                    }
                    "h6" => {
                        md.push_str("\n###### ");
                        tag_stack.push("h6".into());
                    }
                    "strong" | "b" => {
                        md.push_str("**");
                        tag_stack.push("strong".into());
                    }
                    "em" | "i" => {
                        md.push('*');
                        tag_stack.push("em".into());
                    }
                    "u" => {
                        md.push_str("<u>");
                        tag_stack.push("u".into());
                    }
                    "s" | "strike" => {
                        md.push_str("~~");
                        tag_stack.push("s".into());
                    }
                    "code" => {
                        if in_pre {
                            code_content.clear();
                            in_code_block = true;
                        } else {
                            md.push('`');
                            tag_stack.push("code".into());
                        }
                    }
                    "pre" => {
                        in_pre = true;
                        tag_stack.push("pre".into());
                    }
                    "a" => {
                        if let Some(href) = get_xml_attr(e, "href") {
                            md.push('[');
                            tag_stack.push(format!("a|{}", href));
                        }
                    }
                    "img" => {
                        let src = get_xml_attr(e, "src");
                        let alt = get_xml_attr(e, "alt").unwrap_or_default();
                        if let Some(src) = src {
                            md.push_str(&format!("![{}]({})", alt, src));
                        }
                    }
                    "ul" => {
                        list_stack.push('u');
                        tag_stack.push("ul".into());
                    }
                    "ol" => {
                        list_stack.push('o');
                        tag_stack.push("ol".into());
                    }
                    "li" => {
                        let prefix = match list_stack.last() {
                            Some('o') => "1. ",
                            _ => "- ",
                        };
                        let indent = "  ".repeat(list_stack.len().saturating_sub(1));
                        md.push_str(&format!("\n{}{}", indent, prefix));
                    }
                    "table" => {
                        tag_stack.push("table".into());
                    }
                    "thead" => {}
                    "tbody" => {}
                    "tr" => {
                        md.push('\n');
                    }
                    "td" | "th" => {
                        md.push_str("| ");
                        tag_stack.push("td".into());
                    }
                    "blockquote" => {
                        tag_stack.push("blockquote".into());
                    }
                    "ac:structured-macro" => {
                        let name = get_xml_attr(e, "ac:name").unwrap_or_default();
                        let mut mac = ParsedMacro {
                            name,
                            parameters: HashMap::new(),
                            body: String::new(),
                            body_is_rich: false,
                        };
                        // Extract language parameter if present
                        if let Some(lang) = get_xml_attr(e, "ac:parameter") {
                            // Some macros put language as a child parameter
                            mac.parameters.insert("language".into(), lang);
                        }
                        macro_stack.push(mac);
                    }
                    "ac:parameter" => {
                        if let Some(_mac) = macro_stack.last_mut()
                            && let Some(param_name) = get_xml_attr(e, "ac:name")
                        {
                            current_param_name = param_name;
                            in_macro_param = true;
                        }
                    }
                    "ac:plain-text-body" => {
                        if macro_stack.last().is_some() {
                            in_plain_text_body = true;
                        }
                    }
                    "ac:rich-text-body" => {
                        if macro_stack.last().is_some() {
                            in_rich_text_body = true;
                            rich_text_body_content.clear();
                        }
                    }
                    "ac:link" => {
                        // Handle <ac:link><ri:page ri:content-title="..."/></ac:link>
                        tag_stack.push("ac:link".into());
                    }
                    "ri:page" => {
                        if tag_stack.last().map(|s| s.as_str()) == Some("ac:link")
                            && let Some(title) = get_xml_attr(e, "ri:content-title")
                        {
                            md.push_str(&format!("[{}]", title));
                        }
                    }
                    "ac:emoticon" => {
                        let name = get_xml_attr(e, "ac:name").unwrap_or_default();
                        match name.as_str() {
                            "smile" => md.push_str(":)"),
                            "laugh" => md.push_str(":D"),
                            "tongue" => md.push_str(":P"),
                            "wink" => md.push_str(";)"),
                            "thumbs-up" => md.push_str(":thumbsup:"),
                            "thumbs-down" => md.push_str(":thumbsdown:"),
                            _ => md.push_str(&format!(":{name}:")),
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(ref e)) => {
                let text = e.unescape().unwrap_or_default();

                if in_code_block {
                    code_content.push_str(&text);
                } else if in_macro_param {
                    if let Some(ref mut mac) = macro_stack.last_mut() {
                        mac.parameters
                            .entry(current_param_name.clone())
                            .and_modify(|v| v.push_str(&text))
                            .or_insert_with(|| text.to_string());
                    }
                } else if in_plain_text_body {
                    if let Some(ref mut mac) = macro_stack.last_mut() {
                        mac.body.push_str(&text);
                    }
                } else if in_rich_text_body {
                    rich_text_body_content.push_str(&text);
                } else if let Some(ref mut mac) = macro_stack.last_mut() {
                    mac.body.push_str(&text);
                } else {
                    md.push_str(&text);
                }
            }
            Ok(Event::CData(ref e)) => {
                let text = String::from_utf8_lossy(e.as_ref()).to_string();
                if in_code_block {
                    code_content.push_str(&text);
                } else if in_plain_text_body {
                    if let Some(ref mut mac) = macro_stack.last_mut() {
                        mac.body.push_str(&text);
                    }
                } else if in_rich_text_body {
                    rich_text_body_content.push_str(&text);
                }
            }
            Ok(Event::End(ref e)) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match tag_name.as_str() {
                    "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                        md.push('\n');
                        tag_stack.pop();
                    }
                    "strong" | "b" => {
                        md.push_str("**");
                        tag_stack.pop();
                    }
                    "em" | "i" => {
                        md.push('*');
                        tag_stack.pop();
                    }
                    "u" => {
                        md.push_str("</u>");
                        tag_stack.pop();
                    }
                    "s" | "strike" => {
                        md.push_str("~~");
                        tag_stack.pop();
                    }
                    "code" => {
                        if in_code_block {
                            let lang = if code_language.is_empty() {
                                String::new()
                            } else {
                                code_language.clone()
                            };
                            md.push_str(&format!("```{}\n{}\n```\n", lang, code_content));
                            in_code_block = false;
                            code_language.clear();
                        } else {
                            md.push('`');
                            tag_stack.pop();
                        }
                    }
                    "pre" => {
                        in_pre = false;
                        tag_stack.pop();
                    }
                    "a" => {
                        if let Some(last) = tag_stack.pop()
                            && let Some(href) = last.strip_prefix("a|")
                        {
                            md.push_str(&format!("]({})", href));
                        }
                    }
                    "ul" | "ol" => {
                        list_stack.pop();
                        md.push('\n');
                        tag_stack.pop();
                    }
                    "table" => {
                        md.push('\n');
                        tag_stack.pop();
                    }
                    "td" | "th" => {
                        md.push(' ');
                        tag_stack.pop();
                    }
                    "blockquote" => {
                        tag_stack.pop();
                    }
                    "ac:structured-macro" => {
                        if let Some(mac) = macro_stack.pop() {
                            let rendered = render_macro(&mac);
                            md.push_str(&rendered);
                            md.push('\n');
                        }
                    }
                    "ac:parameter" => {
                        in_macro_param = false;
                        current_param_name.clear();
                    }
                    "ac:plain-text-body" => {
                        in_plain_text_body = false;
                    }
                    "ac:rich-text-body" => {
                        in_rich_text_body = false;
                        if let Some(ref mut mac) = macro_stack.last_mut() {
                            mac.body = rich_text_body_content.clone();
                            mac.body_is_rich = true;
                        }
                        rich_text_body_content.clear();
                    }
                    "ac:link" => {
                        tag_stack.pop();
                        // If we have a URL parameter, append it
                        if let Some(mac) = macro_stack.last()
                            && let Some(url) = mac.parameters.get("href")
                        {
                            md.push_str(&format!("({})", url));
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                tracing::warn!("XML parse error in macro conversion: {}", e);
                break;
            }
            _ => {}
        }
        buf.clear();
    }

    // Clean up excessive newlines
    let mut result = md.trim().to_string();
    while result.contains("\n\n\n") {
        result = result.replace("\n\n\n", "\n\n");
    }
    result
}

/// Render a parsed Confluence macro to Markdown.
fn render_macro(mac: &ParsedMacro) -> String {
    match mac.name.as_str() {
        "code" => render_code_macro(mac),
        "info" => render_admonition_macro(mac, "info"),
        "warning" => render_admonition_macro(mac, "warning"),
        "note" => render_admonition_macro(mac, "note"),
        "tip" => render_admonition_macro(mac, "tip"),
        "caution" => render_admonition_macro(mac, "caution"),
        "expand" => render_expand_macro(mac),
        "toc" | "toc-zone" => render_toc_macro(mac),
        "jira" => render_jira_macro(mac),
        "include" => render_include_macro(mac),
        "panel" => render_panel_macro(mac),
        "noformat" => render_noformat_macro(mac),
        "quote" => render_quote_macro(mac),
        "color" => render_color_macro(mac),
        "status" => render_status_macro(mac),
        "section" => render_section_macro(mac),
        "page-tree" => render_page_tree_macro(mac),
        _ => {
            // Unknown macro: render as blockquote with macro name
            let mut out = format!("> **[{}]**\n", mac.name);
            if !mac.body.is_empty() {
                for line in mac.body.lines() {
                    out.push_str("> ");
                    out.push_str(line);
                    out.push('\n');
                }
            }
            out
        }
    }
}

/// Render a code macro as a fenced code block.
fn render_code_macro(mac: &ParsedMacro) -> String {
    let lang = mac
        .parameters
        .get("language")
        .or_else(|| mac.parameters.get("lang"))
        .cloned()
        .unwrap_or_default();

    let title = mac.parameters.get("title").cloned();

    let mut out = String::new();
    if let Some(title) = title {
        out.push_str(&format!("**{}**\n", title));
    }
    out.push_str(&format!("```{}\n", lang));
    out.push_str(&mac.body);
    if !mac.body.ends_with('\n') {
        out.push('\n');
    }
    out.push_str("```\n");
    out
}

/// Render an info/warning/note/tip/caution macro as an admonition.
fn render_admonition_macro(mac: &ParsedMacro, severity: &str) -> String {
    let title = mac
        .parameters
        .get("title")
        .cloned()
        .unwrap_or_else(|| match severity {
            "info" => "Info".to_string(),
            "warning" => "Warning".to_string(),
            "note" => "Note".to_string(),
            "tip" => "Tip".to_string(),
            "caution" => "Caution".to_string(),
            _ => severity.to_string(),
        });

    let icon = match severity {
        "info" => "ℹ️",
        "warning" => "⚠️",
        "note" => "📝",
        "tip" => "💡",
        "caution" => "🔥",
        _ => "📌",
    };

    let mut out = format!("> {} **{}**\n", icon, title);
    if !mac.body.is_empty() {
        for line in mac.body.lines() {
            out.push_str("> ");
            out.push_str(line);
            out.push('\n');
        }
    }
    out.push('\n');
    out
}

/// Render an expand macro as HTML details/summary.
fn render_expand_macro(mac: &ParsedMacro) -> String {
    let title = mac
        .parameters
        .get("title")
        .or_else(|| mac.parameters.get("head"))
        .cloned()
        .unwrap_or_else(|| "Details".to_string());

    let mut out = format!("<details>\n<summary>{}</summary>\n\n", title);
    if !mac.body.is_empty() {
        out.push_str(&mac.body);
        if !mac.body.ends_with('\n') {
            out.push('\n');
        }
    }
    out.push_str("\n</details>\n");
    out
}

/// Render a toc macro as a placeholder.
fn render_toc_macro(mac: &ParsedMacro) -> String {
    let max_depth = mac.parameters.get("maxLevel").cloned().unwrap_or_else(|| {
        mac.parameters
            .get("max-depth")
            .cloned()
            .unwrap_or_else(|| "5".to_string())
    });

    format!("> [TOC]\n> Max depth: {}\n\n", max_depth)
}

/// Render a jira macro as a link.
fn render_jira_macro(mac: &ParsedMacro) -> String {
    let key = mac
        .parameters
        .get("key")
        .cloned()
        .unwrap_or_else(|| mac.body.trim().to_string());

    if key.is_empty() {
        return "> [JIRA: no key specified]\n\n".to_string();
    }

    // Try to extract base URL from parameters or use a placeholder
    let base_url = mac
        .parameters
        .get("server")
        .or_else(|| mac.parameters.get("url"))
        .cloned()
        .unwrap_or_else(|| "https://jira.example.com".to_string());

    format!("[{}: {}]({}/browse/{})\n", "JIRA", key, base_url, key)
}

/// Render an include macro as a transclusion placeholder.
fn render_include_macro(mac: &ParsedMacro) -> String {
    let page = mac
        .parameters
        .get("page")
        .or_else(|| mac.parameters.get("content"))
        .cloned()
        .unwrap_or_else(|| "unknown".to_string());

    let rev = mac.parameters.get("rev").cloned();

    let mut out = format!("> **Included page:** {}", page);
    if let Some(rev) = rev {
        out.push_str(&format!(" (rev: {})", rev));
    }
    out.push_str("\n\n");
    out
}

/// Render a panel macro as a blockquote with title.
fn render_panel_macro(mac: &ParsedMacro) -> String {
    let title = mac.parameters.get("title").cloned().unwrap_or_default();

    let bg = mac.parameters.get("bgColor").cloned();

    let mut out = String::new();
    if !title.is_empty() {
        out.push_str(&format!("> **{}**\n", title));
    }
    if !mac.body.is_empty() {
        for line in mac.body.lines() {
            out.push_str("> ");
            out.push_str(line);
            out.push('\n');
        }
    }
    if bg.is_some() {
        // Note the background color as metadata
        out.push_str(&format!("> _Background: {}_\n", bg.unwrap_or_default()));
    }
    out.push('\n');
    out
}

/// Render a noformat macro as a code block.
fn render_noformat_macro(mac: &ParsedMacro) -> String {
    format!("```\n{}\n```\n", mac.body)
}

/// Render a quote macro as a blockquote.
fn render_quote_macro(mac: &ParsedMacro) -> String {
    let mut out = String::new();
    if !mac.body.is_empty() {
        for line in mac.body.lines() {
            out.push_str("> ");
            out.push_str(line);
            out.push('\n');
        }
    }
    out.push('\n');
    out
}

/// Render a color macro as inline HTML.
fn render_color_macro(mac: &ParsedMacro) -> String {
    let color = mac
        .parameters
        .get("colour")
        .or_else(|| mac.parameters.get("color"))
        .cloned()
        .unwrap_or_else(|| "black".to_string());

    format!("<span style=\"color:{}\">{}</span>", color, mac.body)
}

/// Render a status macro as inline text.
fn render_status_macro(mac: &ParsedMacro) -> String {
    let title = mac
        .parameters
        .get("title")
        .cloned()
        .unwrap_or_else(|| mac.body.trim().to_string());
    let color = mac
        .parameters
        .get("colour")
        .or_else(|| mac.parameters.get("color"))
        .cloned()
        .unwrap_or_else(|| "Gray".to_string());

    format!("`{}` ({})", title, color)
}

/// Render a section macro (used for page sections).
fn render_section_macro(mac: &ParsedMacro) -> String {
    // Section macro wraps content; render body directly
    if !mac.body.is_empty() {
        format!("{}\n", mac.body)
    } else {
        String::new()
    }
}

/// Render a page-tree macro as a placeholder.
fn render_page_tree_macro(mac: &ParsedMacro) -> String {
    let root = mac
        .parameters
        .get("root")
        .or_else(|| mac.parameters.get("pageId"))
        .cloned()
        .unwrap_or_else(|| "current".to_string());

    format!("> **Page tree** (root: {})\n\n", root)
}

/// Extract an attribute value from an XML element by name.
fn get_xml_attr(e: &quick_xml::events::BytesStart<'_>, name: &str) -> Option<String> {
    for attr in e.attributes().flatten() {
        if attr.key.as_ref() == name.as_bytes() {
            return Some(String::from_utf8_lossy(&attr.value).to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_macro() {
        let html = r#"<ac:structured-macro ac:name="code"><ac:parameter ac:name="language">rust</ac:parameter><ac:plain-text-body>fn main() {}</ac:plain-text-body></ac:structured-macro>"#;
        let md = convert_xhtml_to_markdown(html);
        assert!(md.contains("```rust"));
        assert!(md.contains("fn main() {}"));
    }

    #[test]
    fn test_info_macro() {
        let html = r#"<ac:structured-macro ac:name="info"><ac:parameter ac:name="title">Heads up</ac:parameter><ac:plain-text-body>This is important.</ac:plain-text-body></ac:structured-macro>"#;
        let md = convert_xhtml_to_markdown(html);
        assert!(md.contains("**Heads up**"));
        assert!(md.contains("This is important."));
    }

    #[test]
    fn test_warning_macro() {
        let html = r#"<ac:structured-macro ac:name="warning"><ac:plain-text-body>Be careful!</ac:plain-text-body></ac:structured-macro>"#;
        let md = convert_xhtml_to_markdown(html);
        assert!(md.contains("Warning"));
        assert!(md.contains("Be careful!"));
    }

    #[test]
    fn test_expand_macro() {
        let html = r#"<ac:structured-macro ac:name="expand"><ac:parameter ac:name="title">Click to expand</ac:parameter><ac:plain-text-body>Hidden content here.</ac:plain-text-body></ac:structured-macro>"#;
        let md = convert_xhtml_to_markdown(html);
        assert!(md.contains("<details>"));
        assert!(md.contains("Click to expand"));
        assert!(md.contains("Hidden content here."));
        assert!(md.contains("</details>"));
    }

    #[test]
    fn test_toc_macro() {
        let html = r#"<ac:structured-macro ac:name="toc"><ac:parameter ac:name="maxLevel">3</ac:parameter></ac:structured-macro>"#;
        let md = convert_xhtml_to_markdown(html);
        assert!(md.contains("[TOC]"));
        assert!(md.contains("Max depth: 3"));
    }

    #[test]
    fn test_jira_macro() {
        let html = r#"<ac:structured-macro ac:name="jira"><ac:parameter ac:name="key">PROJ-123</ac:parameter></ac:structured-macro>"#;
        let md = convert_xhtml_to_markdown(html);
        assert!(md.contains("PROJ-123"));
        assert!(md.contains("/browse/PROJ-123"));
    }

    #[test]
    fn test_include_macro() {
        let html = r#"<ac:structured-macro ac:name="include"><ac:parameter ac:name="page">My Other Page</ac:parameter></ac:structured-macro>"#;
        let md = convert_xhtml_to_markdown(html);
        assert!(
            md.contains("My Other Page"),
            "Expected 'My Other Page' in output: {}",
            md
        );
    }

    #[test]
    fn test_panel_macro() {
        let html = r#"<ac:structured-macro ac:name="panel"><ac:parameter ac:name="title">Panel Title</ac:parameter><ac:plain-text-body>Panel body text.</ac:plain-text-body></ac:structured-macro>"#;
        let md = convert_xhtml_to_markdown(html);
        assert!(md.contains("**Panel Title**"));
        assert!(md.contains("Panel body text."));
    }

    #[test]
    fn test_noformat_macro() {
        let html = r#"<ac:structured-macro ac:name="noformat"><ac:plain-text-body>raw text here</ac:plain-text-body></ac:structured-macro>"#;
        let md = convert_xhtml_to_markdown(html);
        assert!(md.contains("```\nraw text here\n```"));
    }

    #[test]
    fn test_quote_macro() {
        let html = r#"<ac:structured-macro ac:name="quote"><ac:plain-text-body>To be or not to be.</ac:plain-text-body></ac:structured-macro>"#;
        let md = convert_xhtml_to_markdown(html);
        assert!(md.contains("> To be or not to be."));
    }

    #[test]
    fn test_color_macro() {
        let html = r#"<ac:structured-macro ac:name="color"><ac:parameter ac:name="colour">red</ac:parameter><ac:plain-text-body>Important text</ac:plain-text-body></ac:structured-macro>"#;
        let md = convert_xhtml_to_markdown(html);
        assert!(md.contains("color:red"));
        assert!(md.contains("Important text"));
    }

    #[test]
    fn test_status_macro() {
        let html = r#"<ac:structured-macro ac:name="status"><ac:parameter ac:name="title">Done</ac:parameter><ac:parameter ac:name="colour">Green</ac:parameter></ac:structured-macro>"#;
        let md = convert_xhtml_to_markdown(html);
        assert!(md.contains("`Done` (Green)"));
    }

    #[test]
    fn test_basic_xhtml_elements() {
        let html = r#"<p>Hello <strong>world</strong> and <em>italic</em>.</p><h2>Heading</h2><a href="https://example.com">Link</a>"#;
        let md = convert_xhtml_to_markdown(html);
        assert!(md.contains("Hello **world**"));
        assert!(md.contains("*italic*"));
        assert!(md.contains("## Heading"));
        assert!(md.contains("[Link](https://example.com)"));
    }

    #[test]
    fn test_nested_lists() {
        let html =
            r#"<ul><li>Item 1</li><li>Item 2<ol><li>Nested 1</li><li>Nested 2</li></ol></li></ul>"#;
        let md = convert_xhtml_to_markdown(html);
        assert!(md.contains("- Item 1"));
        assert!(md.contains("- Item 2"));
        assert!(md.contains("1. Nested 1"));
    }

    #[test]
    fn test_table() {
        let html = r#"<table><thead><tr><th>Name</th><th>Value</th></tr></thead><tbody><tr><td>foo</td><td>bar</td></tr></tbody></table>"#;
        let md = convert_xhtml_to_markdown(html);
        assert!(md.contains("| Name"));
        assert!(md.contains("| foo"));
    }

    #[test]
    fn test_unknown_macro_fallback() {
        let html = r#"<ac:structured-macro ac:name="customWidget"><ac:plain-text-body>widget content</ac:plain-text-body></ac:structured-macro>"#;
        let md = convert_xhtml_to_markdown(html);
        assert!(md.contains("**[customWidget]**"));
        assert!(md.contains("widget content"));
    }

    #[test]
    fn test_inline_code() {
        let html = r#"<p>Use <code>println!()</code> for output.</p>"#;
        let md = convert_xhtml_to_markdown(html);
        assert!(md.contains("`println!()`"));
    }

    #[test]
    fn test_bold_and_italic_combined() {
        let html = r#"<p><strong><em>Bold italic</em></strong></p>"#;
        let md = convert_xhtml_to_markdown(html);
        assert!(md.contains("**"));
        assert!(md.contains("*Bold italic*"));
    }
}
