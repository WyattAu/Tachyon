use crate::error::SsgResult;
use crate::manifest::{SiteConfig, SsgDocument};

#[derive(Debug, Clone, serde::Serialize)]
pub struct TocEntry {
    pub level: u8,
    pub id: String,
    pub title: String,
}

pub fn extract_toc(html: &str) -> Vec<TocEntry> {
    let re = regex::Regex::new(r#"<h([1-6])[^>]*id="([^"]*)"[^>]*>(.*?)</h[1-6]>"#).unwrap();
    re.captures_iter(html)
        .map(|cap| TocEntry {
            level: cap[1].parse().unwrap_or(2),
            id: cap[2].to_string(),
            title: strip_html_tags(&cap[3]),
        })
        .collect()
}

fn strip_html_tags(html: &str) -> String {
    let re = regex::Regex::new(r"<[^>]+>").unwrap();
    re.replace_all(html, "").to_string()
}

pub(crate) fn add_heading_ids(html: &str) -> String {
    let re = regex::Regex::new(r#"<(h[23])(\s[^>]*)?>([\s\S]*?)</h[23]>"#).unwrap();
    let mut id_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    re.replace_all(html, |caps: &regex::Captures| {
        let tag = &caps[1];
        let attrs = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        let content = &caps[3];

        if attrs.contains(r#"id=""#) {
            return caps[0].to_string();
        }

        let text = strip_html_tags(content);
        let base_id = crate::slug::slugify(&text);
        let count = id_counts.entry(base_id.clone()).or_insert(0);
        *count += 1;
        let id = if *count == 1 {
            base_id
        } else {
            format!("{}-{}", base_id, count)
        };

        format!(r#"<{} id="{}"{}>{}</{}>"#, tag, id, attrs, content, tag)
    })
    .to_string()
}

pub(crate) fn extract_inline_toc(html: &str) -> Vec<TocEntry> {
    let re = regex::Regex::new(r#"<h([23])[^>]*id="([^"]*)"[^>]*>(.*?)</h[23]>"#).unwrap();
    re.captures_iter(html)
        .map(|cap| TocEntry {
            level: cap[1].parse().unwrap_or(2),
            id: cap[2].to_string(),
            title: strip_html_tags(&cap[3]),
        })
        .collect()
}

pub(crate) fn render_inline_toc(toc: &[TocEntry]) -> String {
    if toc.len() < 2 {
        return String::new();
    }
    let items: String = toc
        .iter()
        .map(|entry| {
            let class = if entry.level == 3 { "toc-h3" } else { "toc-h2" };
            format!(
                r##"<li class="{}"><a href="#{}">{}</a></li>"##,
                class,
                entry.id,
                escape_for_html(&entry.title),
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(r#"<nav class="toc"><ul>{}</ul></nav>"#, items)
}

fn escape_for_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub(crate) fn add_copy_buttons(html: &str) -> String {
    let re =
        regex::Regex::new(r#"<pre([^>]*)>\s*<code([^>]*)>([\s\S]*?)</code>\s*</pre>"#).unwrap();

    re.replace_all(html, |caps: &regex::Captures| {
        let pre_attrs = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let code_attrs = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        let code_content = &caps[3];

        format!(
            r#"<div class="code-block-wrapper"><pre{}><code{}>{}</code></pre><button class="code-copy-btn" onclick="(function(b){{var c=b.parentElement.querySelector('code');navigator.clipboard.writeText(c.textContent).then(function(){{b.textContent='Copied!';setTimeout(function(){{b.textContent='Copy'}},2000)}})}})(this)" aria-label="Copy code to clipboard">Copy</button></div>"#,
            pre_attrs, code_attrs, code_content
        )
    })
    .to_string()
}

pub(crate) struct PageContext<'a> {
    pub(crate) site: &'a SiteConfig,
    pub(crate) title: &'a str,
    pub(crate) description: &'a str,
    pub(crate) body: &'a str,
    pub(crate) page_url: &'a str,
    pub(crate) root_prefix: &'a str,
    pub(crate) author: Option<&'a str>,
    pub(crate) updated_at: &'a str,
    pub(crate) tags: &'a [String],
    pub(crate) current_slug: Option<&'a str>,
    pub(crate) language: &'a str,
    pub(crate) language_switcher: &'a str,
    pub(crate) toc: &'a [TocEntry],
    pub(crate) breadcrumbs: &'a [(String, String)],
    pub(crate) prev_link: Option<&'a (String, String)>,
    pub(crate) next_link: Option<&'a (String, String)>,
    pub(crate) base_url: &'a str,
    pub(crate) slug: &'a str,
    pub(crate) date: &'a str,
    pub(crate) json_ld: String,
}

pub(crate) struct IndexContext<'a> {
    pub(crate) site: &'a SiteConfig,
    pub(crate) documents: &'a [DocCard],
    pub(crate) language: &'a str,
    pub(crate) language_switcher: &'a str,
}

pub(crate) struct CategoryContext<'a> {
    pub(crate) site: &'a SiteConfig,
    pub(crate) category_name: &'a str,
    pub(crate) documents: &'a [DocCard],
    pub(crate) language: &'a str,
    pub(crate) language_switcher: &'a str,
}

pub(crate) struct DocCard {
    pub(crate) title: String,
    pub(crate) slug: String,
    pub(crate) description: String,
    pub(crate) tags: Vec<String>,
    pub(crate) updated_at: String,
}

impl crate::build::SiteGenerator {
    pub(crate) fn render_document_page(
        &self,
        doc: &SsgDocument,
        all_docs: &[&SsgDocument],
        lang: &str,
        lang_prefix: Option<&str>,
        all_languages: &[String],
    ) -> SsgResult<String> {
        let body_html = render_markdown(&doc.content);
        let toc = extract_toc(&body_html);

        let description = doc
            .description
            .clone()
            .unwrap_or_else(|| truncate_text(&body_html, 160));

        let base = self.config.base_url.trim_end_matches('/');
        let page_url = if let Some(prefix) = lang_prefix {
            format!("{}/{}{}.html", base, prefix, doc.slug)
        } else {
            format!("{}/{}.html", base, doc.slug)
        };

        let language_switcher = self.build_language_switcher(all_languages, &doc.slug, lang_prefix);

        // Compute relative path prefix to site root from the page's directory.
        // e.g., slug "docs_university/computing/algo" → depth 2 → "../../"
        let root_prefix = if doc.slug.contains('/') {
            let depth = doc.slug.matches('/').count();
            std::iter::repeat_n("..", depth)
                .collect::<Vec<_>>()
                .join("/")
                + "/"
        } else {
            String::new()
        };

        let breadcrumbs: Vec<(String, String)> = doc
            .slug
            .split('/')
            .enumerate()
            .map(|(i, part)| {
                let href = if i == 0 {
                    format!("{}index.html", root_prefix)
                } else {
                    format!("{}{}.html", root_prefix, part)
                };
                (part.to_string(), href)
            })
            .collect();

        let current_idx = all_docs.iter().position(|d| d.slug == doc.slug);
        let prev_link = current_idx.and_then(|idx| {
            if idx > 0 {
                let prev = all_docs[idx - 1];
                Some((prev.title.clone(), format!("{}.html", prev.slug)))
            } else {
                None
            }
        });
        let next_link = current_idx.and_then(|idx| {
            if idx + 1 < all_docs.len() {
                let next = all_docs[idx + 1];
                Some((next.title.clone(), format!("{}.html", next.slug)))
            } else {
                None
            }
        });

        let base = self.config.base_url.trim_end_matches('/');
        let json_ld = generate_json_ld(
            &doc.title,
            &description,
            &doc.created_at.to_rfc3339(),
            &doc.updated_at.to_rfc3339(),
            doc.author.as_deref(),
            &page_url,
        );

        let ctx = PageContext {
            site: &self.config,
            title: &doc.title,
            description: &description,
            body: &body_html,
            page_url: &page_url,
            root_prefix: &root_prefix,
            author: doc.author.as_deref(),
            updated_at: &doc.updated_at.to_rfc3339(),
            tags: &doc.tags,
            current_slug: Some(&doc.slug),
            language: lang,
            language_switcher: &language_switcher,
            toc: &toc,
            breadcrumbs: &breadcrumbs,
            prev_link: prev_link.as_ref(),
            next_link: next_link.as_ref(),
            base_url: base,
            slug: &doc.slug,
            date: &doc.created_at.to_rfc3339(),
            json_ld,
        };

        Ok(crate::templates::render_doc_page(&ctx))
    }

    pub(crate) fn render_index_page(
        &self,
        docs: &[&SsgDocument],
        lang: &str,
        lang_prefix: Option<&str>,
        all_languages: &[String],
    ) -> SsgResult<String> {
        let doc_cards: Vec<DocCard> = docs
            .iter()
            .map(|d| {
                let body = render_markdown(&d.content);
                DocCard {
                    title: d.title.clone(),
                    slug: d.slug.clone(),
                    description: d
                        .description
                        .clone()
                        .unwrap_or_else(|| truncate_text(&body, 200)),
                    tags: d.tags.clone(),
                    updated_at: d.updated_at.to_rfc3339(),
                }
            })
            .collect();

        let language_switcher = self.build_language_switcher(all_languages, "index", lang_prefix);

        let ctx = IndexContext {
            site: &self.config,
            documents: &doc_cards,
            language: lang,
            language_switcher: &language_switcher,
        };

        Ok(crate::templates::render_index_page(&ctx))
    }

    pub(crate) fn render_category_page(
        &self,
        tag: &str,
        docs: &[&SsgDocument],
        lang: &str,
        lang_prefix: Option<&str>,
        all_languages: &[String],
    ) -> SsgResult<String> {
        let doc_cards: Vec<DocCard> = docs
            .iter()
            .map(|d| {
                let body = render_markdown(&d.content);
                DocCard {
                    title: d.title.clone(),
                    slug: d.slug.clone(),
                    description: d
                        .description
                        .clone()
                        .unwrap_or_else(|| truncate_text(&body, 200)),
                    tags: d.tags.clone(),
                    updated_at: d.updated_at.to_rfc3339(),
                }
            })
            .collect();

        let language_switcher = self.build_language_switcher(all_languages, "index", lang_prefix);

        let ctx = CategoryContext {
            site: &self.config,
            category_name: tag,
            documents: &doc_cards,
            language: lang,
            language_switcher: &language_switcher,
        };

        Ok(crate::templates::render_category_page(&ctx))
    }

    pub(crate) fn build_language_switcher(
        &self,
        all_languages: &[String],
        current_page: &str,
        lang_prefix: Option<&str>,
    ) -> String {
        if all_languages.len() <= 1 {
            return String::new();
        }

        let prefix_to = |target_lang: &str| -> String {
            match lang_prefix {
                Some(_) => format!("../{}/", target_lang),
                None => format!("{}/", target_lang),
            }
        };

        all_languages
            .iter()
            .map(|lang| {
                let name = crate::i18n::language_display_name(lang);
                let href = format!("{}{}", prefix_to(lang), current_page);
                let is_current = lang_prefix
                    .map(|p| p.trim_end_matches('/') == *lang)
                    .unwrap_or(false);
                let active_class = if is_current {
                    " font-bold underline"
                } else {
                    ""
                };
                format!(
                    r#"<a href="{href}" class="text-sm{active_class}">{name}</a>"#,
                    href = href,
                    name = name,
                    active_class = active_class,
                )
            })
            .collect::<Vec<_>>()
            .join("\n          ")
    }
}

pub(crate) fn generate_json_ld(
    title: &str,
    description: &str,
    date_published: &str,
    date_modified: &str,
    author: Option<&str>,
    url: &str,
) -> String {
    let json = serde_json::json!({
        "@context": "https://schema.org",
        "@type": "Article",
        "headline": title,
        "description": description,
        "datePublished": date_published,
        "dateModified": date_modified,
        "author": author.unwrap_or("Unknown"),
        "url": url,
    });
    format!(r#"<script type="application/ld+json">{}</script>"#, json)
}

/// Sanitize content for safe embedding in format! output.
/// Replaces `{` and `}` to prevent format! macro interpretation.
fn html_escape_content(s: &str) -> String {
    s.replace('{', "&#123;").replace('}', "&#125;")
}

pub(crate) fn render_markdown(content: &str) -> String {
    use tachyon_renderer::{RenderConfig, Renderer};

    let (content, mermaid_blocks) = extract_mermaid_blocks(content);

    let config = RenderConfig::default();
    let renderer = Renderer::new(config);

    let result = match renderer.render(&content, None) {
        Ok(result) => result.content,
        Err(_) => {
            format!("<div>{}</div>", html_escape_content(&content))
        }
    };

    let result = if let Ok(latex) = renderer.latex().render_from_text(&result) {
        latex
    } else {
        result
    };

    let result = render_admonitions(&result);
    let result = add_heading_ids(&result);
    let toc_entries = extract_inline_toc(&result);
    let toc_html = render_inline_toc(&toc_entries);
    let result = replace_mermaid_placeholders(&result, &mermaid_blocks);
    let result = add_copy_buttons(&result);

    if toc_html.is_empty() {
        result
    } else {
        format!("{}\n{}", toc_html, result)
    }
}

pub(crate) fn extract_mermaid_blocks(content: &str) -> (String, Vec<String>) {
    let mut blocks = Vec::new();
    let mut result = String::with_capacity(content.len());
    let mut in_code_block = false;
    let mut in_mermaid = false;
    let mut mermaid_content = String::new();

    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") && !in_code_block {
            let lang = trimmed.trim_start_matches('`').trim();
            if lang == "mermaid" {
                in_mermaid = true;
                mermaid_content.clear();
            } else {
                result.push_str(line);
                result.push('\n');
            }
            in_code_block = true;
        } else if trimmed.starts_with("```") && in_code_block {
            if in_mermaid {
                blocks.push(mermaid_content.trim().to_string());
                result.push_str(&format!("\n\nMERMPLACEHOLDER{}END\n\n", blocks.len() - 1));
                in_mermaid = false;
            } else {
                result.push_str(line);
                result.push('\n');
            }
            in_code_block = false;
        } else if in_mermaid {
            mermaid_content.push_str(line);
            mermaid_content.push('\n');
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }

    (result, blocks)
}

pub(crate) fn replace_mermaid_placeholders(html: &str, blocks: &[String]) -> String {
    let mut result = html.to_string();
    for (i, block) in blocks.iter().enumerate() {
        let placeholder = format!("MERMPLACEHOLDER{}END", i);
        let escaped = block
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;");
        let replacement = format!(r#"<div class="mermaid">{}</div>"#, escaped);
        result = result.replace(&placeholder, &replacement);
    }
    result
}

pub(crate) fn process_mermaid_blocks(html: &str) -> String {
    let re = regex::Regex::new(
        r#"<pre[^>]*>\s*<code[^>]*class="language-mermaid"[^>]*>([\s\S]*?)</code>\s*</pre>"#,
    )
    .unwrap();

    re.replace_all(html, |caps: &regex::Captures| {
        let content = &caps[1];
        let unescaped = content
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&amp;", "&")
            .replace("&quot;", "\"")
            .replace("&#39;", "'");
        format!(r#"<div class="mermaid">{}</div>"#, unescaped.trim())
    })
    .to_string()
}

pub(crate) fn render_admonitions(html: &str) -> String {
    let re = regex::Regex::new(
        r#"(?si)<blockquote>\s*<p>\s*\[!(NOTE|WARNING|TIP|DANGER|INFO|SUCCESS)\]\s*</p>\s*(.*?)</blockquote>"#,
    )
    .unwrap();

    re.replace_all(html, |caps: &regex::Captures| {
        let admonition_type = &caps[1];
        let title = capitalize_first(admonition_type);
        let content = caps[2].trim();
        format!(
            r#"<div class="admonition admonition-{type}">
<p class="admonition-title">{title}</p>
{content}
</div>"#,
            type = admonition_type.to_lowercase(),
            title = title,
            content = content,
        )
    })
    .to_string()
}

fn capitalize_first(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    if let Some(first) = chars.next() {
        for c in first.to_uppercase() {
            result.push(c);
        }
    }
    for c in chars {
        for lc in c.to_lowercase() {
            result.push(lc);
        }
    }
    result
}

pub(crate) fn truncate_text(html: &str, max_chars: usize) -> String {
    let plain = html_to_plain(html);
    if plain.len() <= max_chars {
        plain
    } else {
        let truncated = &plain[..max_chars];
        if let Some(pos) = truncated.rfind(' ') {
            format!("{}...", &truncated[..pos])
        } else {
            format!("{}...", truncated)
        }
    }
}

pub(crate) fn html_to_plain(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut prev_was_tag_end = false;
    for ch in html.chars() {
        match ch {
            '<' => {
                in_tag = true;
            }
            '>' => {
                in_tag = false;
                prev_was_tag_end = true;
            }
            _ if !in_tag => {
                if prev_was_tag_end && !ch.is_whitespace() && !result.is_empty() {
                    result.push(' ');
                }
                result.push(ch);
                prev_was_tag_end = false;
            }
            _ => {}
        }
    }
    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub(crate) fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
