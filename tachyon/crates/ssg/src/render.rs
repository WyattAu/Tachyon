use crate::error::SsgResult;
use crate::manifest::{SiteConfig, SsgDocument};

pub(crate) struct PageContext<'a> {
    pub(crate) site: &'a SiteConfig,
    pub(crate) title: &'a str,
    pub(crate) description: &'a str,
    pub(crate) body: &'a str,
    pub(crate) page_url: &'a str,
    pub(crate) author: Option<&'a str>,
    pub(crate) created_at: &'a str,
    pub(crate) updated_at: &'a str,
    pub(crate) tags: &'a [String],
    pub(crate) nav_items: &'a [NavItem],
    pub(crate) current_slug: Option<&'a str>,
    pub(crate) language: &'a str,
    pub(crate) language_switcher: &'a str,
}

pub(crate) struct IndexContext<'a> {
    pub(crate) site: &'a SiteConfig,
    pub(crate) nav_items: &'a [NavItem],
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

pub(crate) struct NavItem {
    pub(crate) title: String,
    pub(crate) href: String,
}

pub(crate) struct DocCard {
    pub(crate) title: String,
    pub(crate) slug: String,
    pub(crate) description: String,
    pub(crate) tags: Vec<String>,
    pub(crate) updated_at: String,
    pub(crate) author: Option<String>,
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

        let nav_items = all_docs
            .iter()
            .map(|d| NavItem {
                title: d.title.clone(),
                href: format!("{}.html", d.slug),
            })
            .collect::<Vec<_>>();

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

        let ctx = PageContext {
            site: &self.config,
            title: &doc.title,
            description: &description,
            body: &body_html,
            page_url: &page_url,
            author: doc.author.as_deref(),
            created_at: &doc.created_at.to_rfc3339(),
            updated_at: &doc.updated_at.to_rfc3339(),
            tags: &doc.tags,
            nav_items: &nav_items,
            current_slug: Some(&doc.slug),
            language: lang,
            language_switcher: &language_switcher,
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
        let nav_items = docs
            .iter()
            .map(|d| NavItem {
                title: d.title.clone(),
                href: format!("{}.html", d.slug),
            })
            .collect::<Vec<_>>();

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
                    author: d.author.clone(),
                }
            })
            .collect();

        let language_switcher = self.build_language_switcher(all_languages, "index", lang_prefix);

        let ctx = IndexContext {
            site: &self.config,
            nav_items: &nav_items,
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
                    author: d.author.clone(),
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

pub(crate) fn render_markdown(content: &str) -> String {
    use tachyon_renderer::{RenderConfig, Renderer};

    let config = RenderConfig::default();
    match Renderer::new(config).render(content, None) {
        Ok(result) => result.content,
        Err(_) => {
            format!("<div>{}</div>", content)
        }
    }
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
