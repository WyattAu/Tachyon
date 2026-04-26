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
        let base = self.config.base_url.trim_end_matches('/');
        let now = Utc::now().to_rfc3339();

        let index_loc = if let Some(prefix) = lang_prefix {
            format!("{}/{}index.html", base, prefix)
        } else {
            format!("{}/index.html", base)
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
            let doc_loc = if let Some(prefix) = lang_prefix {
                format!("{}/{}{}.html", base, prefix, doc.slug)
            } else {
                format!("{}/{}.html", base, doc.slug)
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
