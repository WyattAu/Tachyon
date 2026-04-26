use chrono::Utc;

use crate::error::SsgResult;
use crate::manifest::SsgDocument;
use crate::render::escape_xml;

impl crate::build::SiteGenerator {
    pub(crate) fn render_rss(
        &self,
        docs: &[&SsgDocument],
        lang: &str,
        lang_prefix: Option<&str>,
    ) -> SsgResult<String> {
        let base = self.config.base_url.trim_end_matches('/');
        let recent: Vec<_> = docs.iter().take(20).collect();
        let now = Utc::now().to_rfc3339();

        let base_with_prefix = if let Some(prefix) = lang_prefix {
            format!("{}/{}", base, prefix)
        } else {
            base.to_string()
        };

        let mut items = String::new();
        for doc in &recent {
            let description = doc
                .description
                .clone()
                .unwrap_or_else(|| "No description".to_string());
            items.push_str(&format!(
                r#"
    <item>
      <title>{}</title>
      <link>{}/{}.html</link>
      <description>{}</description>
      <pubDate>{}</pubDate>
      <guid>{}/{}.html</guid>
    </item>"#,
                escape_xml(&doc.title),
                base_with_prefix,
                doc.slug,
                escape_xml(&description),
                doc.updated_at.to_rfc3339(),
                base_with_prefix,
                doc.slug,
            ));
        }

        Ok(format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">
  <channel>
    <title>{}</title>
    <link>{}</link>
    <description>{}</description>
    <language>{}</language>
    <lastBuildDate>{}</lastBuildDate>
    <atom:link href="{}/feed.xml" rel="self" type="application/rss+xml"/>
    <generator>Tachyon SSG</generator>
    {}
  </channel>
</rss>"#,
            escape_xml(&self.config.title),
            base_with_prefix,
            escape_xml(&self.config.description),
            lang,
            now,
            base_with_prefix,
            items,
        ))
    }
}
