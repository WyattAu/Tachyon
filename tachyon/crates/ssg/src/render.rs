use crate::error::SsgResult;
use crate::manifest::{SiteConfig, SsgDocument, Translations};
use std::sync::LazyLock;

static TOC_HEADING_REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r#"<h([1-6])[^>]*id="([^"]*)"[^>]*>(.*?)</h[1-6]>"#).unwrap()
});

static HTML_STRIP_REGEX: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"<[^>]+>").unwrap());

static HEADING_ID_REGEX: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"<(h[23])(\s[^>]*)?>([\s\S]*?)</h[23]>"#).unwrap());

static INLINE_TOC_REGEX: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"<h([23])[^>]*id="([^"]*)"[^>]*>(.*?)</h[23]>"#).unwrap());

static CODE_BLOCK_REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r#"<pre([^>]*)>\s*<code([^>]*)>([\s\S]*?)</code>\s*</pre>"#).unwrap()
});

static CODE_TITLE_WRAPPER_REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(
        r#"(<div class="code-block-wrapper">)<pre([^>]*)><code([^>]*)>([\s\S]*?)</code></pre>"#,
    )
    .unwrap()
});

static CODE_TITLE_REGEX: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"^([\s]*)(?://|#|--|;)\s*title=(.+?)\s*$"#).unwrap());

static IMG_TAG_REGEX: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"<img([^>]*)>"#).unwrap());

static CONTENT_TAB_REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r#"<!--\s*tab:(.+?)\s*-->([\s\S]*?)<!--\s*/tab\s*-->"#).unwrap()
});

static MERMAID_BLOCK_REGEX: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"```mermaid\s*\n([\s\S]*?)```"#).unwrap());

static ADMONITION_REGEX: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"^>\s*\[!([\w-]+)\]\s*(.*)$"#).unwrap());

static CODE_GROUP_WRAPPER_REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(
        r#"<div class="code-block-wrapper"><pre[^>]*><code class="language-([^"]*)"[^>]*>([\s\S]*?)</code></pre><button[^>]*>Copy</button></div>"#,
    )
    .unwrap()
});

static CODE_GROUP_BARE_REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r#"<pre[^>]*><code class="language-([^"]*)"[^>]*>([\s\S]*?)</code></pre>"#)
        .unwrap()
});

static MERMAID_CODE_BLOCK_REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(
        r#"<pre[^>]*>\s*<code[^>]*class="language-mermaid"[^>]*>([\s\S]*?)</code>\s*</pre>"#,
    )
    .unwrap()
});

static ADMONITION_BLOCKQUOTE_REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(
        r#"(?si)<blockquote>\s*<p>\s*\[!(NOTE|WARNING|TIP|DANGER|INFO|SUCCESS)([^\]]*)\]\s*</p>\s*(.*?)</blockquote>"#,
    )
    .unwrap()
});

#[derive(Debug, Clone, serde::Serialize)]
pub struct TocEntry {
    pub level: u8,
    pub id: String,
    pub title: String,
}

pub fn extract_toc(html: &str) -> Vec<TocEntry> {
    TOC_HEADING_REGEX
        .captures_iter(html)
        .map(|cap| TocEntry {
            level: cap[1].parse().unwrap_or(2),
            id: cap[2].to_string(),
            title: strip_html_tags(&cap[3]),
        })
        .collect()
}

fn strip_html_tags(html: &str) -> String {
    HTML_STRIP_REGEX.replace_all(html, "").to_string()
}

pub(crate) fn add_heading_ids(html: &str) -> String {
    let mut id_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    HEADING_ID_REGEX
        .replace_all(html, |caps: &regex::Captures| {
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
    INLINE_TOC_REGEX
        .captures_iter(html)
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
    CODE_BLOCK_REGEX.replace_all(html, |caps: &regex::Captures| {
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

pub(crate) fn extract_code_titles(html: &str) -> String {
    CODE_TITLE_WRAPPER_REGEX
        .replace_all(html, |caps: &regex::Captures| {
            let wrapper = &caps[1];
            let pre_attrs = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            let code_attrs = caps.get(3).map(|m| m.as_str()).unwrap_or("");
            let code = caps.get(4).map(|m| m.as_str()).unwrap_or("");

            if let Some(tc) = CODE_TITLE_REGEX.captures(code) {
                let title = tc[2].trim();
                let remaining_code = &code[tc[0].len()..];
                format!(
                    r#"<div class="code-title">{}</div>{}<pre{}><code{}>{}</code></pre>"#,
                    escape_for_html(title),
                    wrapper,
                    pre_attrs,
                    code_attrs,
                    remaining_code
                )
            } else {
                caps[0].to_string()
            }
        })
        .to_string()
}

pub(crate) fn enhance_images(html: &str) -> String {
    IMG_TAG_REGEX
        .replace_all(html, |caps: &regex::Captures| {
            let attrs = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            if attrs.contains("loading=") {
                return caps[0].to_string();
            }
            format!("<img loading=\"lazy\"{}>", attrs)
        })
        .to_string()
}

pub(crate) fn process_content_tabs(html: &str) -> String {
    let mut groups: Vec<(String, String, usize, usize)> = Vec::new();
    for cap in CONTENT_TAB_REGEX.captures_iter(html) {
        let name = cap[1].trim().to_string();
        let content = cap[2].trim().to_string();
        let start = cap.get(0).unwrap().start();
        let end = cap.get(0).unwrap().end();
        groups.push((name, content, start, end));
    }

    if groups.len() < 2 {
        return html.to_string();
    }

    groups.sort_by_key(|g| g.2);

    let first_start = groups[0].2;
    let last_end = groups[groups.len() - 1].3;

    let tabs: String = groups
        .iter()
        .enumerate()
        .map(|(i, (name, _, _, _))| {
            let active = if i == 0 { " active" } else { "" };
            format!(
                r#"<button class="content-tab{active}" data-tab="{name}" onclick="this.parentElement.querySelectorAll('.content-tab,.content-tab-panel').forEach(function(e){{e.classList.remove('active')}});this.classList.add('active');this.closest('.content-group').querySelectorAll('.content-tab-panel[data-tab=&quot;'+this.dataset.tab+'&quot;]').forEach(function(e){{e.classList.add('active')}})">{name}</button>"#,
                active = active,
                name = escape_for_html(name),
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let panels: String = groups
        .iter()
        .enumerate()
        .map(|(i, (_, content, _, _))| {
            let active = if i == 0 { " active" } else { "" };
            format!(
                r#"<div class="content-tab-panel{active}" data-tab="{}">{}</div>"#,
                escape_for_html(&groups[i].0),
                content
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let replacement = format!(
        r#"<div class="content-group"><div class="content-tabs">{}</div>{}</div>"#,
        tabs, panels
    );

    let result = html.to_string();
    format!(
        "{}{}{}",
        &result[..first_start],
        replacement,
        &result[last_end..]
    )
}

pub(crate) struct RenderContext<'a> {
    pub(crate) lang: &'a str,
    pub(crate) lang_prefix: Option<&'a str>,
    pub(crate) all_languages: &'a [String],
    pub(crate) current_version: &'a str,
    pub(crate) version_prefix: Option<&'a str>,
}

pub(crate) struct PageContext<'a> {
    pub(crate) site: &'a SiteConfig,
    pub(crate) translations: &'a Translations,
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
    pub(crate) version_switcher: &'a str,
    pub(crate) current_version: &'a str,
    pub(crate) toc: &'a [TocEntry],
    pub(crate) breadcrumbs: &'a [(String, String)],
    pub(crate) prev_link: Option<&'a (String, String)>,
    pub(crate) next_link: Option<&'a (String, String)>,
    pub(crate) base_url: &'a str,
    pub(crate) slug: &'a str,
    pub(crate) date: &'a str,
    pub(crate) json_ld: String,
    pub(crate) hreflang_tags: String,
    pub(crate) og_image: Option<&'a str>,
    pub(crate) sidebar_auto_items: &'a [(String, String)],
}

pub(crate) struct IndexContext<'a> {
    pub(crate) site: &'a SiteConfig,
    pub(crate) translations: &'a Translations,
    pub(crate) documents: &'a [DocCard],
    pub(crate) language: &'a str,
    pub(crate) language_switcher: &'a str,
    pub(crate) version_switcher: &'a str,
    pub(crate) current_version: &'a str,
}

pub(crate) struct CategoryContext<'a> {
    pub(crate) site: &'a SiteConfig,
    pub(crate) translations: &'a Translations,
    pub(crate) category_name: &'a str,
    pub(crate) documents: &'a [DocCard],
    pub(crate) language: &'a str,
    pub(crate) language_switcher: &'a str,
    pub(crate) version_switcher: &'a str,
    pub(crate) current_version: &'a str,
}

pub(crate) struct DocCard {
    pub(crate) title: String,
    pub(crate) slug: String,
    pub(crate) description: String,
    pub(crate) tags: Vec<String>,
    pub(crate) updated_at: String,
}

impl crate::build::SiteGenerator {
    pub(crate) fn sidebar_ordered_slugs(&self, all_docs: &[&SsgDocument]) -> Vec<String> {
        let mut slugs = Vec::new();
        fn collect(items: &[crate::manifest::SidebarItem], slugs: &mut Vec<String>) {
            for item in items {
                let slug = item
                    .href
                    .trim_start_matches('/')
                    .trim_end_matches(".html")
                    .to_string();
                if !slug.is_empty() && slug != "#" {
                    slugs.push(slug);
                }
                collect(&item.children, slugs);
            }
        }
        collect(&self.config.menu_items, &mut slugs);
        // Auto-generate sidebar from documents when menu_items is empty
        if slugs.is_empty() {
            slugs = all_docs
                .iter()
                .map(|d| d.slug.clone())
                .filter(|s| !s.is_empty())
                .collect();
        }
        slugs
    }

    pub(crate) fn render_document_page(
        &self,
        doc: &SsgDocument,
        all_docs: &[&SsgDocument],
        ctx: &RenderContext<'_>,
    ) -> SsgResult<String> {
        let RenderContext {
            lang,
            lang_prefix,
            all_languages,
            current_version,
            version_prefix,
        } = *ctx;

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

        let current_page = format!("{}.html", doc.slug);
        let version_switcher =
            self.build_version_switcher(current_version, &current_page, version_prefix);

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

        let breadcrumbs: Vec<(String, String)> = if doc.hide_breadcrumbs {
            vec![]
        } else {
            doc.slug
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
                .collect()
        };

        let sidebar_slugs = self.sidebar_ordered_slugs(all_docs);
        // Build auto sidebar items (slug, title) when menu_items is empty
        let sidebar_auto_items: Vec<(String, String)> = if self.config.menu_items.is_empty() {
            sidebar_slugs
                .iter()
                .filter_map(|slug| {
                    all_docs
                        .iter()
                        .find(|d| d.slug == *slug)
                        .map(|d| (d.slug.clone(), d.title.clone()))
                })
                .collect()
        } else {
            Vec::new()
        };
        let sidebar_idx = sidebar_slugs.iter().position(|s| *s == doc.slug);
        let prev_link = sidebar_idx.and_then(|idx| {
            if idx > 0 {
                let prev_slug = &sidebar_slugs[idx - 1];
                all_docs.iter().find(|d| d.slug == *prev_slug).map(|prev| {
                    (
                        prev.title.clone(),
                        format!("{}{}.html", root_prefix, prev.slug),
                    )
                })
            } else {
                None
            }
        });
        let next_link = sidebar_idx.and_then(|idx| {
            if idx + 1 < sidebar_slugs.len() {
                let next_slug = &sidebar_slugs[idx + 1];
                all_docs.iter().find(|d| d.slug == *next_slug).map(|next| {
                    (
                        next.title.clone(),
                        format!("{}{}.html", root_prefix, next.slug),
                    )
                })
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

        let hreflang_tags =
            build_hreflang_tags(&self.config, &doc.slug, lang_prefix, all_languages);

        let ctx = PageContext {
            site: &self.config,
            translations: &Translations::default(),
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
            version_switcher: &version_switcher,
            current_version,
            toc: &toc,
            breadcrumbs: &breadcrumbs,
            prev_link: prev_link.as_ref(),
            next_link: next_link.as_ref(),
            base_url: base,
            slug: &doc.slug,
            date: &doc.created_at.to_rfc3339(),
            json_ld,
            hreflang_tags,
            og_image: self.config.og_image.as_deref(),
            sidebar_auto_items: &sidebar_auto_items,
        };

        Ok(crate::templates::render_doc_page(&ctx))
    }

    pub(crate) fn render_index_page(
        &self,
        docs: &[&SsgDocument],
        ctx: &RenderContext<'_>,
    ) -> SsgResult<String> {
        let RenderContext {
            lang,
            lang_prefix,
            all_languages,
            current_version,
            version_prefix,
        } = *ctx;
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

        let version_switcher =
            self.build_version_switcher(current_version, "index.html", version_prefix);

        let ctx = IndexContext {
            site: &self.config,
            translations: &Translations::default(),
            documents: &doc_cards,
            language: lang,
            language_switcher: &language_switcher,
            version_switcher: &version_switcher,
            current_version,
        };

        Ok(crate::templates::render_index_page(&ctx))
    }

    pub(crate) fn render_category_page(
        &self,
        tag: &str,
        docs: &[&SsgDocument],
        ctx: &RenderContext<'_>,
    ) -> SsgResult<String> {
        let RenderContext {
            lang,
            lang_prefix,
            all_languages,
            current_version,
            version_prefix,
        } = *ctx;
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

        let version_switcher =
            self.build_version_switcher(current_version, "index.html", version_prefix);

        let ctx = CategoryContext {
            site: &self.config,
            translations: &Translations::default(),
            category_name: tag,
            documents: &doc_cards,
            language: lang,
            language_switcher: &language_switcher,
            version_switcher: &version_switcher,
            current_version,
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

    pub(crate) fn build_version_switcher(
        &self,
        current_version: &str,
        current_page: &str,
        version_prefix: Option<&str>,
    ) -> String {
        if self.config.versions.len() <= 1 {
            return String::new();
        }

        let options: Vec<String> = self
            .config
            .versions
            .iter()
            .map(|vc| {
                let href = match version_prefix {
                    Some(_) => format!("../../{}/{}", vc.version, current_page),
                    None => format!("../{}/{}", vc.version, current_page),
                };
                let selected = if vc.version == current_version {
                    r#" selected"#
                } else {
                    ""
                };
                let label = if vc.is_latest {
                    format!("{} (Latest)", vc.title)
                } else {
                    vc.title.clone()
                };
                format!(
                    r#"<option value="{href}"{selected}>{label}</option>"#,
                    href = href,
                    selected = selected,
                    label = label,
                )
            })
            .collect();

        format!(
            r#"<select id="version-select" class="text-sm border border-gray-300 dark:border-gray-600 rounded px-2 py-1 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300">
  {}
</select>"#,
            options.join("\n  ")
        )
    }
}

fn build_hreflang_tags(
    config: &SiteConfig,
    slug: &str,
    _current_lang_prefix: Option<&str>,
    all_languages: &[String],
) -> String {
    if all_languages.len() <= 1 {
        return String::new();
    }

    let base = config.base_url.trim_end_matches('/');

    let tags: Vec<String> = all_languages
        .iter()
        .map(|lang| {
            let href = format!("{}/{}/{}.html", base, lang, slug);
            format!(
                r#"<link rel="alternate" hreflang="{}" href="{}">"#,
                lang, href
            )
        })
        .collect();

    tags.join("\n  ")
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

pub fn process_code_groups(html: &str) -> String {
    struct CodeBlock {
        lang: String,
        code: String,
        start: usize,
        end: usize,
    }

    let mut all_blocks: Vec<CodeBlock> = Vec::new();

    for cap in CODE_GROUP_WRAPPER_REGEX.captures_iter(html) {
        let m = cap.get(0).unwrap();
        all_blocks.push(CodeBlock {
            lang: cap[1].to_string(),
            code: cap[2].to_string(),
            start: m.start(),
            end: m.end(),
        });
    }

    for cap in CODE_GROUP_BARE_REGEX.captures_iter(html) {
        let m = cap.get(0).unwrap();
        let overlaps = all_blocks
            .iter()
            .any(|b| m.start() >= b.start && m.start() < b.end);
        if !overlaps {
            all_blocks.push(CodeBlock {
                lang: cap[1].to_string(),
                code: cap[2].to_string(),
                start: m.start(),
                end: m.end(),
            });
        }
    }

    all_blocks.sort_by_key(|b| b.start);

    if all_blocks.len() < 2 {
        return html.to_string();
    }

    let mut adjacent_groups: Vec<Vec<usize>> = Vec::new();
    let mut current_group: Vec<usize> = vec![0];

    for i in 1..all_blocks.len() {
        let prev_end = all_blocks[i - 1].end;
        let curr_start = all_blocks[i].start;
        let between = html[prev_end..curr_start].trim();
        if between.is_empty() {
            current_group.push(i);
        } else {
            if current_group.len() >= 2 {
                adjacent_groups.push(current_group.clone());
            }
            current_group = vec![i];
        }
    }
    if current_group.len() >= 2 {
        adjacent_groups.push(current_group);
    }

    if adjacent_groups.is_empty() {
        return html.to_string();
    }

    let mut result = html.to_string();

    for group_indices in adjacent_groups.iter().rev() {
        let first = &all_blocks[group_indices[0]];
        let last = &all_blocks[*group_indices.last().unwrap()];

        let mut seen_langs = std::collections::HashSet::new();
        let mut unique_blocks: Vec<(String, String)> = Vec::new();
        for &idx in group_indices {
            let b = &all_blocks[idx];
            if seen_langs.insert(b.lang.clone()) {
                unique_blocks.push((b.lang.clone(), b.code.clone()));
            }
        }

        if unique_blocks.len() < 2 {
            continue;
        }

        let first_start = first.start;
        let last_end = last.end;

        let tabs: String = unique_blocks
            .iter()
            .enumerate()
            .map(|(i, (lang, _))| {
                let active = if i == 0 { " active" } else { "" };
                let display = capitalize_render(lang);
                format!(
                    r#"<button class="tab{active}" data-lang="{lang}" onclick="this.parentElement.querySelectorAll('.tab,.tab-content').forEach(function(e){{e.classList.remove('active')}});this.classList.add('active');this.closest('.code-group').querySelectorAll('.tab-content[data-lang='+this.dataset.lang+']').forEach(function(e){{e.classList.add('active')}})">{display}</button>"#,
                    active = active,
                    lang = lang,
                    display = display,
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let contents: String = unique_blocks
            .iter()
            .enumerate()
            .map(|(i, (lang, code))| {
                let active = if i == 0 { " active" } else { "" };
                format!(
                    r#"<div class="tab-content{active}" data-lang="{lang}"><pre><code class="language-{lang}">{code}</code></pre></div>"#,
                    active = active,
                    lang = lang,
                    code = code,
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let replacement = format!(
            r#"<div class="code-group"><div class="code-tabs">{tabs}</div>{contents}</div>"#,
            tabs = tabs,
            contents = contents,
        );

        result = format!(
            "{}{}{}",
            &result[..first_start],
            replacement,
            &result[last_end..]
        );
    }

    result
}

fn capitalize_render(s: &str) -> String {
    match s {
        "js" => "JavaScript".to_string(),
        "ts" => "TypeScript".to_string(),
        "rust" => "Rust".to_string(),
        "python" | "py" => "Python".to_string(),
        "go" => "Go".to_string(),
        "bash" | "sh" | "shell" => "Shell".to_string(),
        "json" => "JSON".to_string(),
        "yaml" | "yml" => "YAML".to_string(),
        "toml" => "TOML".to_string(),
        "html" => "HTML".to_string(),
        "css" => "CSS".to_string(),
        "sql" => "SQL".to_string(),
        "java" => "Java".to_string(),
        "c" => "C".to_string(),
        "cpp" | "c++" => "C++".to_string(),
        _ => {
            let mut c = s.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().chain(c).collect(),
            }
        }
    }
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
    let result = extract_code_titles(&result);
    let result = process_code_groups(&result);
    let result = enhance_images(&result);
    let result = process_content_tabs(&result);

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
    MERMAID_CODE_BLOCK_REGEX
        .replace_all(html, |caps: &regex::Captures| {
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
    ADMONITION_BLOCKQUOTE_REGEX
        .replace_all(html, |caps: &regex::Captures| {
            let admonition_type = &caps[1];
            let custom_title = caps[2].trim();
            let title = if custom_title.is_empty() {
                capitalize_first(admonition_type)
            } else {
                custom_title.to_string()
            };
            let content = caps[3].trim();
            format!(
                r#"<div class="admonition admonition-{type}">
<p class="admonition-title">{title}</p>
{content}
</div>"#,
                type = admonition_type.to_lowercase(),
                title = escape_for_html(&title),
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
