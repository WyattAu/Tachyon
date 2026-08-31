use chrono::Utc;

use crate::error::SsgResult;
use crate::manifest::SsgDocument;

impl crate::build::SiteGenerator {
    pub(crate) fn render_sitemap(
        &self,
        docs: &[&SsgDocument],
        lang: &str,
        lang_prefix: Option<&str>,
    ) -> SsgResult<String> {
        self.render_sitemap_inner(docs, lang, lang_prefix, None)
    }

    pub(crate) fn render_versioned_sitemap(
        &self,
        docs: &[&SsgDocument],
        lang: &str,
        lang_prefix: Option<&str>,
        version_prefix: &str,
    ) -> SsgResult<String> {
        self.render_sitemap_inner(docs, lang, lang_prefix, Some(version_prefix))
    }

    fn render_sitemap_inner(
        &self,
        docs: &[&SsgDocument],
        lang: &str,
        lang_prefix: Option<&str>,
        version_prefix: Option<&str>,
    ) -> SsgResult<String> {
        let base = self.config.base_url.trim_end_matches('/');
        let now = Utc::now().to_rfc3339();

        let mut url_base = String::new();
        if let Some(prefix) = version_prefix {
            url_base.push_str(prefix);
        }
        if let Some(prefix) = lang_prefix {
            url_base.push_str(prefix);
        }

        let index_loc = if url_base.is_empty() {
            format!("{}/index.html", base)
        } else {
            format!("{}/{}/index.html", base, url_base.trim_end_matches('/'))
        };

        let mut urls = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:xhtml="http://www.w3.org/1999/xhtml">
  <url>
    <loc>{index_loc}</loc>
    <lastmod>{now}</lastmod>
    <changefreq>daily</changefreq>
    <priority>1.0</priority>
    <xhtml:link rel="alternate" hreflang="{lang}" href="{index_loc}"/>
  </url>"#
        );

        for doc in docs {
            let doc_loc = if url_base.is_empty() {
                format!("{}/{}.html", base, doc.slug)
            } else {
                format!(
                    "{}/{}/{}.html",
                    base,
                    url_base.trim_end_matches('/'),
                    doc.slug
                )
            };
            urls.push_str(&format!(
                r#"
  <url>
    <loc>{doc_loc}</loc>
    <lastmod>{updated}</lastmod>
    <changefreq>weekly</changefreq>
    <priority>0.8</priority>
    <xhtml:link rel="alternate" hreflang="{lang}" href="{doc_loc}"/>
  </url>"#,
                doc_loc = doc_loc,
                lang = lang,
                updated = doc.updated_at.to_rfc3339(),
            ));
        }

        urls.push_str("\n</urlset>");
        Ok(urls)
    }
}
