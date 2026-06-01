use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;

use crate::build_cache::BuildCache;
use crate::error::{SsgError, SsgResult};
use crate::manifest::{BuildResult, SiteConfig, SsgDocument};

pub struct SiteGenerator {
    pub(crate) config: SiteConfig,
}

struct VersionContext<'a> {
    all_languages: &'a [String],
    version_prefix: Option<&'a str>,
    current_version: &'a str,
}

impl SiteGenerator {
    pub fn new(config: SiteConfig) -> Self {
        Self { config }
    }

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
        let mut versions_built = 0usize;

        let cache_path = output_dir.join(".build-cache.json");
        let mut cache = BuildCache::load(&cache_path);

        let is_versioned = self.config.versions.len() > 1;
        let languages = self.collect_languages(documents);
        let is_multi_lang = languages.len() > 1;

        if is_versioned {
            tracing::info!(
                "Building versioned site: {} versions, {} languages",
                self.config.versions.len(),
                languages.len(),
            );

            for version_config in &self.config.versions {
                let version = &version_config.version;
                let version_dir = output_dir.join(version);
                std::fs::create_dir_all(&version_dir).map_err(|e| SsgError::Io(e.to_string()))?;

                let version_docs: Vec<&SsgDocument> =
                    documents.iter().filter(|d| d.version == *version).collect();

                if is_multi_lang {
                    for lang in &languages {
                        let lang_docs: Vec<&SsgDocument> = version_docs
                            .iter()
                            .filter(|d| d.language == *lang)
                            .cloned()
                            .collect();

                        let (pages, categories, generated) = self.build_versioned_language_dir(
                            &lang_docs,
                            lang,
                            &version_dir,
                            &languages,
                            version,
                            &mut cache,
                        )?;

                        total_pages += pages;
                        total_categories += categories;
                        total_files += pages + categories + 3;
                        all_generated_pages.extend(generated);
                    }

                    let redirect =
                        self.render_version_root_redirect(version, &self.config.language)?;
                    let index_path = version_dir.join("index.html");
                    std::fs::write(&index_path, redirect).map_err(|e| {
                        SsgError::Io(format!("Failed to write version index.html: {}", e))
                    })?;
                    total_files += 1;
                } else {
                    let lang = languages.first().map(|s| s.as_str()).unwrap_or("en");
                    let (pages, categories, generated) = self.build_versioned_language_dir(
                        &version_docs,
                        lang,
                        &version_dir,
                        &languages,
                        version,
                        &mut cache,
                    )?;

                    total_pages += pages;
                    total_categories += categories;
                    total_files += pages + categories + 3;
                    all_generated_pages.extend(generated);
                }

                versions_built += 1;
            }

            let redirect = self.render_version_redirect(&self.config.default_version)?;
            let index_path = output_dir.join("index.html");
            std::fs::write(&index_path, redirect)
                .map_err(|e| SsgError::Io(format!("Failed to write root index.html: {}", e)))?;
            total_files += 1;

            let latest_version = self
                .config
                .versions
                .iter()
                .find(|v| v.is_latest)
                .map(|v| v.version.clone())
                .unwrap_or_else(|| self.config.default_version.clone());
            let latest_dir = output_dir.join("latest");
            let redirect = self.render_version_redirect(&latest_version)?;
            let latest_path = latest_dir.join("index.html");
            std::fs::create_dir_all(&latest_dir).map_err(|e| SsgError::Io(e.to_string()))?;
            std::fs::write(&latest_path, redirect)
                .map_err(|e| SsgError::Io(format!("Failed to write latest/index.html: {}", e)))?;
            total_files += 1;
        } else if is_multi_lang {
            for lang in &languages {
                let lang_docs: Vec<&SsgDocument> =
                    documents.iter().filter(|d| d.language == *lang).collect();

                let version_ctx = VersionContext {
                    all_languages: &languages,
                    version_prefix: None,
                    current_version: &self.config.default_version,
                };
                let (pages, categories, generated) = self.build_language_dir(
                    &lang_docs,
                    lang,
                    output_dir,
                    &version_ctx,
                    &mut cache,
                )?;

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
            let version_ctx = VersionContext {
                all_languages: &languages,
                version_prefix: None,
                current_version: &self.config.default_version,
            };
            let (pages, categories, generated) =
                self.build_language_dir(&all_docs, lang, output_dir, &version_ctx, &mut cache)?;

            total_pages += pages;
            total_categories += categories;
            total_files += pages + categories + 3;
            all_generated_pages.extend(generated);
        }

        let output_size_bytes = dir_size(output_dir).map_err(|e| SsgError::Io(e.to_string()))?;
        let build_time_ms = start.elapsed().as_millis() as u64;

        cache.save(&cache_path);

        if self.config.robots_txt {
            let base = self.config.base_url.trim_end_matches('/');
            let robots_content =
                format!("User-agent: *\nAllow: /\nSitemap: {}/sitemap.xml\n", base);
            let robots_path = output_dir.join("robots.txt");
            std::fs::write(&robots_path, robots_content)
                .map_err(|e| SsgError::Io(format!("Failed to write robots.txt: {}", e)))?;
            total_files += 1;
        }

        tracing::info!(
            "SSG build complete: {} pages, {} categories, {} languages, {} versions, {} files, {:.1}KB in {}ms",
            total_pages,
            total_categories,
            languages.len(),
            versions_built,
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
            versions_built,
        })
    }

    pub fn build_to_zip(&self, documents: &[SsgDocument]) -> SsgResult<(Vec<u8>, BuildResult)> {
        let start = std::time::Instant::now();

        let mut total_pages = 0usize;
        let mut total_categories = 0usize;
        let mut all_generated_pages = Vec::new();

        let languages = self.collect_languages(documents);
        let is_multi_lang = languages.len() > 1;

        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        let options: zip::write::FileOptions<()> =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        if is_multi_lang {
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
                versions_built: 0,
            },
        ))
    }

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

    fn build_language_dir(
        &self,
        docs: &[&SsgDocument],
        lang: &str,
        output_dir: &Path,
        version_ctx: &VersionContext<'_>,
        cache: &mut BuildCache,
    ) -> SsgResult<(usize, usize, Vec<String>)> {
        let mut sorted_docs: Vec<&SsgDocument> = docs.to_vec();
        sorted_docs.sort_by_key(|d| (d.order, d.title.clone()));

        let is_multi = version_ctx.all_languages.len() > 1;
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

        self.build_pages_to_dir(
            &sorted_docs,
            lang,
            lang_prefix.as_deref(),
            &write_dir,
            version_ctx,
            cache,
        )
    }

    fn build_versioned_language_dir(
        &self,
        docs: &[&SsgDocument],
        lang: &str,
        version_dir: &Path,
        all_languages: &[String],
        version: &str,
        cache: &mut BuildCache,
    ) -> SsgResult<(usize, usize, Vec<String>)> {
        let mut sorted_docs: Vec<&SsgDocument> = docs.to_vec();
        sorted_docs.sort_by_key(|d| (d.order, d.title.clone()));

        let is_multi = all_languages.len() > 1;
        let write_dir = if is_multi {
            let lang_dir = version_dir.join(lang);
            std::fs::create_dir_all(&lang_dir).map_err(|e| SsgError::Io(e.to_string()))?;
            lang_dir
        } else {
            version_dir.to_path_buf()
        };

        let lang_prefix = if is_multi {
            Some(format!("{}/", lang))
        } else {
            None
        };

        let version_prefix = Some(format!("{}/", version));
        let version_ctx = VersionContext {
            all_languages,
            version_prefix: version_prefix.as_deref(),
            current_version: version,
        };

        self.build_pages_to_dir(
            &sorted_docs,
            lang,
            lang_prefix.as_deref(),
            &write_dir,
            &version_ctx,
            cache,
        )
    }

    fn build_pages_to_dir(
        &self,
        sorted_docs: &[&SsgDocument],
        lang: &str,
        lang_prefix: Option<&str>,
        write_dir: &Path,
        version_ctx: &VersionContext<'_>,
        cache: &mut BuildCache,
    ) -> SsgResult<(usize, usize, Vec<String>)> {
        let render_ctx = crate::render::RenderContext {
            lang,
            lang_prefix,
            all_languages: version_ctx.all_languages,
            current_version: version_ctx.current_version,
            version_prefix: version_ctx.version_prefix,
        };
        let mut generated_pages = Vec::new();

        let active_slugs: Vec<&str> = sorted_docs.iter().map(|d| d.slug.as_str()).collect();
        let stale = cache.prune_stale(&active_slugs);
        for slug in &stale {
            let stale_path = write_dir.join(format!("{}.html", slug));
            if stale_path.exists() {
                let _ = std::fs::remove_file(&stale_path);
            }
        }

        let tag_groups: BTreeMap<String, Vec<&SsgDocument>> = if self.config.group_by_tag {
            let mut map: BTreeMap<String, Vec<&SsgDocument>> = BTreeMap::new();
            for doc in sorted_docs {
                for tag in &doc.tags {
                    map.entry(tag.clone()).or_default().push(doc);
                }
            }
            map
        } else {
            BTreeMap::new()
        };

        for doc in sorted_docs {
            let filename = format!("{}.html", doc.slug);
            let path = write_dir.join(&filename);

            if !cache.needs_rebuild(doc, &path) {
                tracing::debug!("Skipping unchanged: {}", doc.slug);
                generated_pages.push(doc.slug.clone());
                continue;
            }

            let html = self.render_document_page(doc, sorted_docs, &render_ctx)?;
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    SsgError::Io(format!("Failed to create dir {:?}: {}", parent, e))
                })?;
            }
            std::fs::write(&path, html).map_err(|e| {
                SsgError::Io(format!("Failed to write {}/{}: {}", lang, filename, e))
            })?;
            cache.record(doc);
            generated_pages.push(doc.slug.clone());
        }

        let mut category_count = 0usize;
        if self.config.group_by_tag {
            for (tag, tag_docs) in &tag_groups {
                let html = self.render_category_page(tag, tag_docs, &render_ctx)?;
                let filename = format!("category-{}.html", crate::slug::slugify(tag));
                let path = write_dir.join(&filename);
                std::fs::write(&path, html)
                    .map_err(|e| SsgError::Io(format!("Failed to write category page: {}", e)))?;
                category_count += 1;
            }
        }

        let index_html = self.render_index_page(sorted_docs, &render_ctx)?;
        let index_path = write_dir.join("index.html");
        std::fs::write(&index_path, index_html)
            .map_err(|e| SsgError::Io(format!("Failed to write {}/index.html: {}", lang, e)))?;

        let sitemap = if let Some(vp) = version_ctx.version_prefix {
            self.render_versioned_sitemap(sorted_docs, lang, lang_prefix, vp)?
        } else {
            self.render_sitemap(sorted_docs, lang, lang_prefix)?
        };
        let sitemap_path = write_dir.join("sitemap.xml");
        std::fs::write(&sitemap_path, sitemap)
            .map_err(|e| SsgError::Io(format!("Failed to write {}/sitemap.xml: {}", lang, e)))?;

        let rss = self.render_rss(sorted_docs, lang, lang_prefix)?;
        let rss_path = write_dir.join("feed.xml");
        std::fs::write(&rss_path, rss)
            .map_err(|e| SsgError::Io(format!("Failed to write {}/feed.xml: {}", lang, e)))?;

        Ok((sorted_docs.len(), category_count, generated_pages))
    }

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

        let render_ctx = crate::render::RenderContext {
            lang,
            lang_prefix: lang_prefix.as_deref(),
            all_languages,
            current_version: &self.config.default_version,
            version_prefix: None,
        };

        for doc in &sorted_docs {
            let html = self.render_document_page(doc, &sorted_docs, &render_ctx)?;
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
                let html = self.render_category_page(tag, tag_docs, &render_ctx)?;
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

        let index_html = self.render_index_page(&sorted_docs, &render_ctx)?;
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

    fn render_version_redirect(&self, version: &str) -> SsgResult<String> {
        Ok(format!(
            r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <meta http-equiv="refresh" content="0;url={version}/">
  <link rel="canonical" href="{version}/">
</head>
<body>
  <p>Redirecting to <a href="{version}/">{version}</a>...</p>
</body>
</html>"#,
            version = version,
        ))
    }

    fn render_version_root_redirect(&self, version: &str, default_lang: &str) -> SsgResult<String> {
        if self.config.translations.is_empty() {
            Ok(format!(
                r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <meta http-equiv="refresh" content="0;url={version}/">
  <link rel="canonical" href="{version}/">
</head>
<body>
  <p>Redirecting to <a href="{version}/">{version}</a>...</p>
</body>
</html>"#,
                version = version,
            ))
        } else {
            Ok(format!(
                r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <meta http-equiv="refresh" content="0;url={version}/{lang}/">
  <link rel="canonical" href="{version}/{lang}/">
</head>
<body>
  <p>Redirecting to <a href="{version}/{lang}/">{version}/{lang}</a>...</p>
</body>
</html>"#,
                version = version,
                lang = default_lang,
            ))
        }
    }
}

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
