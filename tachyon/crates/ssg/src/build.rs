use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;

use crate::error::{SsgError, SsgResult};
use crate::manifest::{BuildResult, SiteConfig, SsgDocument};

/// Static site generator engine.
///
/// Takes a `SiteConfig` and a list of `SsgDocument`s, renders each document
/// to HTML using `tachyon-renderer`, and produces a complete static site.
pub struct SiteGenerator {
    pub(crate) config: SiteConfig,
}

impl SiteGenerator {
    /// Create a new site generator with the given configuration.
    pub fn new(config: SiteConfig) -> Self {
        Self { config }
    }

    /// Generate the complete static site, writing files to the given directory.
    ///
    /// Creates the output directory if it doesn't exist. Returns a `BuildResult`
    /// with statistics about what was generated.
    ///
    /// When `translations` is configured in the `SiteConfig`, generates per-language
    /// subdirectories (e.g., `output/en/`, `output/zh/`). Otherwise, generates
    /// a flat site in the output directory (backward compatible).
    pub fn build_to_dir<P: AsRef<Path>>(
        &self,
        documents: &[SsgDocument],
        output_dir: P,
    ) -> SsgResult<BuildResult> {
        let start = std::time::Instant::now();
        let output_dir = output_dir.as_ref();

        std::fs::create_dir_all(output_dir).map_err(|e| SsgError::Io(e.to_string()))?;

        let mut total_files = 0usize;
        let mut total_pages = 0usize;
        let mut total_categories = 0usize;
        let mut all_generated_pages = Vec::new();

        let languages = self.collect_languages(documents);

        if languages.len() > 1 {
            for lang in &languages {
                let lang_docs: Vec<&SsgDocument> =
                    documents.iter().filter(|d| d.language == *lang).collect();

                let (pages, categories, generated) =
                    self.build_language_dir(&lang_docs, lang, output_dir, &languages)?;

                total_pages += pages;
                total_categories += categories;
                total_files += pages + categories + 3;
                all_generated_pages.extend(generated);
            }

            let redirect = self.render_root_redirect(&self.config.language)?;
            let index_path = output_dir.join("index.html");
            std::fs::write(&index_path, redirect)
                .map_err(|e| SsgError::Io(format!("Failed to write root index.html: {}", e)))?;
            total_files += 1;
        } else {
            let lang = languages.first().map(|s| s.as_str()).unwrap_or("en");
            let all_docs: Vec<&SsgDocument> = documents.iter().collect();
            let (pages, categories, generated) =
                self.build_language_dir(&all_docs, lang, output_dir, &languages)?;

            total_pages += pages;
            total_categories += categories;
            total_files += pages + categories + 3;
            all_generated_pages.extend(generated);
        }

        let output_size_bytes = dir_size(output_dir).map_err(|e| SsgError::Io(e.to_string()))?;
        let build_time_ms = start.elapsed().as_millis() as u64;

        tracing::info!(
            "SSG build complete: {} pages, {} categories, {} languages, {} files, {:.1}KB in {}ms",
            total_pages,
            total_categories,
            languages.len(),
            total_files,
            output_size_bytes as f64 / 1024.0,
            build_time_ms,
        );

        Ok(BuildResult {
            pages: total_pages,
            category_pages: total_categories,
            total_files,
            build_time_ms,
            output_size_bytes,
            generated_pages: all_generated_pages,
            languages: languages.len(),
        })
    }

    /// Generate the complete static site as a ZIP archive in memory.
    ///
    /// Returns the ZIP bytes and a `BuildResult`.
    /// Multi-language sites use per-language subdirectories in the ZIP.
    pub fn build_to_zip(&self, documents: &[SsgDocument]) -> SsgResult<(Vec<u8>, BuildResult)> {
        let start = std::time::Instant::now();

        let mut total_pages = 0usize;
        let mut total_categories = 0usize;
        let mut all_generated_pages = Vec::new();

        let languages = self.collect_languages(documents);

        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        let options: zip::write::FileOptions<()> =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        if languages.len() > 1 {
            for lang in &languages {
                let lang_docs: Vec<&SsgDocument> =
                    documents.iter().filter(|d| d.language == *lang).collect();

                let (pages, categories, generated) =
                    self.build_language_zip(&lang_docs, lang, &mut zip, options, &languages)?;

                total_pages += pages;
                total_categories += categories;
                all_generated_pages.extend(generated);
            }

            let redirect = self.render_root_redirect(&self.config.language)?;
            zip.start_file("index.html", options)
                .map_err(|e| SsgError::Zip(e.to_string()))?;
            zip.write_all(redirect.as_bytes())
                .map_err(|e| SsgError::Zip(e.to_string()))?;
        } else {
            let lang = languages.first().map(|s| s.as_str()).unwrap_or("en");
            let all_docs: Vec<&SsgDocument> = documents.iter().collect();
            let (pages, categories, generated) =
                self.build_language_zip(&all_docs, lang, &mut zip, options, &languages)?;

            total_pages += pages;
            total_categories += categories;
            all_generated_pages.extend(generated);
        }

        let cursor = zip.finish().map_err(|e| SsgError::Zip(e.to_string()))?;
        let zip_bytes = cursor.into_inner();
        let zip_len = zip_bytes.len() as u64;
        let build_time_ms = start.elapsed().as_millis() as u64;

        Ok((
            zip_bytes,
            BuildResult {
                pages: total_pages,
                category_pages: total_categories,
                total_files: 0,
                build_time_ms,
                output_size_bytes: zip_len,
                generated_pages: all_generated_pages,
                languages: languages.len(),
            },
        ))
    }

    /// Collect all unique language codes from config + documents.
    fn collect_languages(&self, documents: &[SsgDocument]) -> Vec<String> {
        let mut langs: std::collections::HashSet<String> = std::collections::HashSet::new();

        langs.insert(self.config.language.clone());

        for t in &self.config.translations {
            langs.insert(t.language.clone());
        }

        for doc in documents {
            if !doc.language.is_empty() {
                langs.insert(doc.language.clone());
            }
        }

        let mut langs: Vec<String> = langs.into_iter().collect();
        langs.sort();
        langs
    }

    /// Build a single language's output to a directory.
    /// Returns (pages, categories, generated_page_slugs).
    fn build_language_dir(
        &self,
        docs: &[&SsgDocument],
        lang: &str,
        output_dir: &Path,
        all_languages: &[String],
    ) -> SsgResult<(usize, usize, Vec<String>)> {
        let mut sorted_docs: Vec<&SsgDocument> = docs.to_vec();
        sorted_docs.sort_by_key(|d| (d.order, d.title.clone()));

        let is_multi = all_languages.len() > 1;
        let write_dir = if is_multi {
            let lang_dir = output_dir.join(lang);
            std::fs::create_dir_all(&lang_dir).map_err(|e| SsgError::Io(e.to_string()))?;
            lang_dir
        } else {
            output_dir.to_path_buf()
        };

        let lang_prefix = if is_multi {
            Some(format!("{}/", lang))
        } else {
            None
        };

        let mut generated_pages = Vec::new();

        let tag_groups: BTreeMap<String, Vec<&SsgDocument>> = if self.config.group_by_tag {
            let mut map: BTreeMap<String, Vec<&SsgDocument>> = BTreeMap::new();
            for doc in &sorted_docs {
                for tag in &doc.tags {
                    map.entry(tag.clone()).or_default().push(doc);
                }
            }
            map
        } else {
            BTreeMap::new()
        };

        for doc in &sorted_docs {
            let html = self.render_document_page(
                doc,
                &sorted_docs,
                lang,
                lang_prefix.as_deref(),
                all_languages,
            )?;
            let filename = format!("{}.html", doc.slug);
            let path = write_dir.join(&filename);
            std::fs::write(&path, html).map_err(|e| {
                SsgError::Io(format!("Failed to write {}/{}: {}", lang, filename, e))
            })?;
            generated_pages.push(doc.slug.clone());
        }

        let mut category_count = 0usize;
        if self.config.group_by_tag {
            for (tag, tag_docs) in &tag_groups {
                let html = self.render_category_page(
                    tag,
                    tag_docs,
                    lang,
                    lang_prefix.as_deref(),
                    all_languages,
                )?;
                let filename = format!("category-{}.html", crate::slug::slugify(tag));
                let path = write_dir.join(&filename);
                std::fs::write(&path, html)
                    .map_err(|e| SsgError::Io(format!("Failed to write category page: {}", e)))?;
                category_count += 1;
            }
        }

        let index_html =
            self.render_index_page(&sorted_docs, lang, lang_prefix.as_deref(), all_languages)?;
        let index_path = write_dir.join("index.html");
        std::fs::write(&index_path, index_html)
            .map_err(|e| SsgError::Io(format!("Failed to write {}/index.html: {}", lang, e)))?;

        let sitemap = self.render_sitemap(&sorted_docs, lang, lang_prefix.as_deref())?;
        let sitemap_path = write_dir.join("sitemap.xml");
        std::fs::write(&sitemap_path, sitemap)
            .map_err(|e| SsgError::Io(format!("Failed to write {}/sitemap.xml: {}", lang, e)))?;

        let rss = self.render_rss(&sorted_docs, lang, lang_prefix.as_deref())?;
        let rss_path = write_dir.join("feed.xml");
        std::fs::write(&rss_path, rss)
            .map_err(|e| SsgError::Io(format!("Failed to write {}/feed.xml: {}", lang, e)))?;

        Ok((sorted_docs.len(), category_count, generated_pages))
    }

    /// Build a single language's output into a ZIP.
    fn build_language_zip(
        &self,
        docs: &[&SsgDocument],
        lang: &str,
        zip: &mut zip::ZipWriter<std::io::Cursor<Vec<u8>>>,
        options: zip::write::FileOptions<()>,
        all_languages: &[String],
    ) -> SsgResult<(usize, usize, Vec<String>)> {
        let mut sorted_docs: Vec<&SsgDocument> = docs.to_vec();
        sorted_docs.sort_by_key(|d| (d.order, d.title.clone()));

        let is_multi = all_languages.len() > 1;
        let lang_prefix = if is_multi {
            Some(format!("{}/", lang))
        } else {
            None
        };

        let mut generated_pages = Vec::new();

        let tag_groups: BTreeMap<String, Vec<&SsgDocument>> = if self.config.group_by_tag {
            let mut map: BTreeMap<String, Vec<&SsgDocument>> = BTreeMap::new();
            for doc in &sorted_docs {
                for tag in &doc.tags {
                    map.entry(tag.clone()).or_default().push(doc);
                }
            }
            map
        } else {
            BTreeMap::new()
        };

        for doc in &sorted_docs {
            let html = self.render_document_page(
                doc,
                &sorted_docs,
                lang,
                lang_prefix.as_deref(),
                all_languages,
            )?;
            let path = if is_multi {
                format!("{}/{}.html", lang, doc.slug)
            } else {
                format!("{}.html", doc.slug)
            };
            zip.start_file(&path, options)
                .map_err(|e| SsgError::Zip(e.to_string()))?;
            zip.write_all(html.as_bytes())
                .map_err(|e| SsgError::Zip(e.to_string()))?;
            generated_pages.push(path);
        }

        let mut category_count = 0usize;
        if self.config.group_by_tag {
            for (tag, tag_docs) in &tag_groups {
                let html = self.render_category_page(
                    tag,
                    tag_docs,
                    lang,
                    lang_prefix.as_deref(),
                    all_languages,
                )?;
                let path = if is_multi {
                    format!("{}/category-{}.html", lang, crate::slug::slugify(tag))
                } else {
                    format!("category-{}.html", crate::slug::slugify(tag))
                };
                zip.start_file(&path, options)
                    .map_err(|e| SsgError::Zip(e.to_string()))?;
                zip.write_all(html.as_bytes())
                    .map_err(|e| SsgError::Zip(e.to_string()))?;
                category_count += 1;
            }
        }

        let index_html =
            self.render_index_page(&sorted_docs, lang, lang_prefix.as_deref(), all_languages)?;
        let index_path = if is_multi {
            format!("{}/index.html", lang)
        } else {
            "index.html".to_string()
        };
        zip.start_file(&index_path, options)
            .map_err(|e| SsgError::Zip(e.to_string()))?;
        zip.write_all(index_html.as_bytes())
            .map_err(|e| SsgError::Zip(e.to_string()))?;

        Ok((sorted_docs.len(), category_count, generated_pages))
    }

    /// Render a root index.html that redirects to the default language.
    fn render_root_redirect(&self, default_lang: &str) -> SsgResult<String> {
        Ok(format!(
            r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <meta http-equiv="refresh" content="0;url={lang}/">
  <link rel="canonical" href="{lang}/">
</head>
<body>
  <p>Redirecting to <a href="{lang}/">{lang}</a>...</p>
</body>
</html>"#,
            lang = default_lang,
        ))
    }
}

/// Calculate total directory size in bytes.
fn dir_size(path: &Path) -> std::io::Result<u64> {
    let mut total = 0u64;
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            if metadata.is_file() {
                total += metadata.len();
            } else if metadata.is_dir() {
                total += dir_size(&entry.path())?;
            }
        }
    }
    Ok(total)
}
