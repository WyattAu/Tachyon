use crate::commands::Command;
use crate::error::{CliError, CliResult};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tachyon_core::compute_content_hash;
use tachyon_database::{DocumentRepository, init_with_migrations};
use tachyon_renderer::context::{RenderContext, RenderMetadata};
use tachyon_renderer::page::{render_full_page, SiteConfig};
use tachyon_renderer::{RenderConfig, Renderer};

#[derive(Debug, Clone)]
pub struct BuildCommand {
    pub repo_path: PathBuf,
    pub output_dir: PathBuf,
    pub database_url: Option<String>,
    pub site_title: String,
    pub site_description: String,
    pub base_url: String,
    pub published_only: bool,
    pub clean: bool,
    pub verbose: bool,
    pub template: Option<PathBuf>,
}

impl BuildCommand {
    pub fn new(
        repo_path: PathBuf,
        output_dir: PathBuf,
        database_url: Option<String>,
        site_title: String,
        site_description: String,
        base_url: String,
        published_only: bool,
        clean: bool,
        verbose: bool,
        template: Option<PathBuf>,
    ) -> Self {
        Self {
            repo_path,
            output_dir,
            database_url,
            site_title,
            site_description,
            base_url,
            published_only,
            clean,
            verbose,
            template,
        }
    }

    pub fn from_args(
        repo_path: Option<PathBuf>,
        output_dir: Option<PathBuf>,
        database_url: Option<String>,
        site_title: Option<String>,
        site_description: Option<String>,
        base_url: Option<String>,
        published_only: bool,
        clean: bool,
        verbose: bool,
        template: Option<PathBuf>,
    ) -> Self {
        Self::new(
            repo_path.unwrap_or_else(|| PathBuf::from(".")),
            output_dir.unwrap_or_else(|| PathBuf::from("dist")),
            database_url,
            site_title.unwrap_or_else(|| "Tachyon Docs".to_string()),
            site_description.unwrap_or_else(|| "Knowledge Management System".to_string()),
            base_url.unwrap_or_else(|| "/".to_string()),
            published_only,
            clean,
            verbose,
            template,
        )
    }
}

impl Command for BuildCommand {
    fn execute(&self) -> CliResult<()> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| CliError::generic(format!("Failed to create async runtime: {}", e)))?;
        rt.block_on(self.run())
    }

    fn name(&self) -> &str {
        "build"
    }

    fn description(&self) -> &str {
        "Build static site from database documents"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BuildManifest {
    build_time: String,
    commit_hash: Option<String>,
    documents: BTreeMap<String, DocumentEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DocumentEntry {
    content_hash: String,
    output_file: String,
    built_at: String,
}

struct DocInfo {
    slug: String,
    title: String,
    content: String,
    description: Option<String>,
    tags: Vec<String>,
    updated_at: String,
    created_at: String,
}

impl BuildCommand {
    async fn run(&self) -> CliResult<()> {
        let start = Instant::now();

        println!("");
        println!("Tachyon Build");
        println!("===============");
        println!("Repository: {}", self.repo_path.display());
        println!("Output:     {}", self.output_dir.display());
        println!("");

        let database_url = self.resolve_database_url()?;

        if self.verbose {
            println!("Database URL: {}", database_url);
        }

        println!("Initializing database...");
        let pool = init_with_migrations(&database_url)
            .await
            .map_err(|e| CliError::database(format!("Failed to initialize database: {}", e)))?;

        let prev_manifest = load_manifest(&self.output_dir);

        println!("Fetching documents...");
        let repo = DocumentRepository::new(pool);
        let documents = repo
            .list_all(Some(100_000), None)
            .await
            .map_err(|e| CliError::database(format!("Failed to fetch documents: {}", e)))?;

        let mut docs: Vec<DocInfo> = documents
            .into_iter()
            .filter(|d| {
                if self.published_only {
                    d.status == "published" && d.visibility == "public"
                } else {
                    true
                }
            })
            .filter_map(|d| {
                let slug = d.slug.clone()?;
                let tags = d.parse_tags().ok().unwrap_or_default();
                let content = d.content.clone()?;
                Some(DocInfo {
                    slug,
                    title: d.title,
                    content,
                    description: d.description,
                    tags,
                    updated_at: d.updated_at.to_rfc3339(),
                    created_at: d.created_at.to_rfc3339(),
                })
            })
            .collect();

        docs.sort_by(|a, b| a.slug.as_str().cmp(b.slug.as_str()));
        println!("Found {} document(s).", docs.len());

        if self.clean && self.output_dir.exists() {
            println!("Cleaning output directory...");
            fs::remove_dir_all(&self.output_dir).map_err(|e| {
                CliError::io(&self.output_dir, format!("Failed to clean: {}", e))
            })?;
        }

        fs::create_dir_all(self.output_dir.join("docs")).map_err(|e| {
            CliError::io(
                &self.output_dir,
                format!("Failed to create output dir: {}", e),
            )
        })?;
        fs::create_dir_all(self.output_dir.join("static")).map_err(|e| {
            CliError::io(
                &self.output_dir,
                format!("Failed to create static dir: {}", e),
            )
        })?;

        let renderer = Renderer::new(RenderConfig::default());
        let site_config = SiteConfig {
            site_title: self.site_title.clone(),
            site_description: self.site_description.clone(),
            base_url: self.base_url.clone(),
            theme_color: "#2563eb".to_string(),
            og_image: None,
            template_dir: self.template.as_ref().map(|p| p.to_string_lossy().to_string()),
        };

        let mut new_manifest = BuildManifest {
            build_time: chrono::Utc::now().to_rfc3339(),
            commit_hash: get_commit_hash(&self.repo_path),
            documents: BTreeMap::new(),
        };

        let mut built: usize = 0;
        let mut skipped: usize = 0;
        let mut errors: Vec<(String, String)> = Vec::new();

        println!("Rendering documents...");

        for doc in &docs {
            let content_hash = compute_content_hash(&doc.content);
            let output_file = format!("docs/{}/index.html", doc.slug);
            let output_path = self.output_dir.join(&output_file);

            if let Some(ref prev) = prev_manifest {
                if let Some(entry) = prev.documents.get(&doc.slug) {
                    if entry.content_hash == content_hash && output_path.exists() {
                        if self.verbose {
                            println!("  [cached] {}", doc.slug);
                        }
                        skipped += 1;
                        new_manifest.documents.insert(
                            doc.slug.clone(),
                            DocumentEntry {
                                content_hash,
                                output_file,
                                built_at: entry.built_at.clone(),
                            },
                        );
                        continue;
                    }
                }
            }

            match render_document(&renderer, &site_config, doc) {
                Ok(html) => {
                    if let Some(parent) = output_path.parent() {
                        fs::create_dir_all(parent).map_err(|e| {
                            CliError::io(parent, format!("Failed to create dir: {}", e))
                        })?;
                    }
                    fs::write(&output_path, html).map_err(|e| {
                        CliError::io(&output_path, format!("Failed to write: {}", e))
                    })?;

                    if self.verbose {
                        println!("  [built]  {}", doc.slug);
                    }
                    built += 1;
                    new_manifest.documents.insert(
                        doc.slug.clone(),
                        DocumentEntry {
                            content_hash,
                            output_file,
                            built_at: new_manifest.build_time.clone(),
                        },
                    );
                }
                Err(e) => {
                    errors.push((doc.slug.clone(), e.to_string()));
                    eprintln!("  [error]  {}: {}", doc.slug, e);
                }
            }
        }

        println!("Generating index page...");
        generate_index(&self.output_dir, &docs, &site_config)?;

        println!("Generating sitemap...");
        generate_sitemap(&self.output_dir, &docs, &self.base_url)?;

        println!("Writing build manifest...");
        write_manifest(&self.output_dir, &new_manifest)?;

        let removed = remove_stale(&self.output_dir, &prev_manifest, &new_manifest, self.verbose)?;

        let elapsed = start.elapsed();

        println!("");
        println!("Build completed in {:.2}s", elapsed.as_secs_f64());
        println!("  Built:   {}", built);
        println!("  Cached:  {}", skipped);
        println!("  Removed: {}", removed);
        println!("  Errors:  {}", errors.len());

        if !errors.is_empty() {
            println!("");
            println!("Failed documents:");
            for (slug, err) in &errors {
                println!("  {}: {}", slug, err);
            }
            return Err(CliError::build(format!(
                "{} document(s) failed to render",
                errors.len()
            )));
        }

        Ok(())
    }

    fn resolve_database_url(&self) -> CliResult<String> {
        if let Some(ref url) = self.database_url {
            Ok(url.clone())
        } else {
            let db_path = self.repo_path.join(".tachyon").join("db").join("tachyon.db");
            Err(CliError::database(format!(
                "No database URL provided. SQLite is not yet supported.\n\
                 Use --database-url to specify a PostgreSQL connection string.\n\
                 Example: --database-url postgres://user:pass@localhost/db\n\
                 Expected SQLite path would be: {}",
                db_path.display()
            )))
        }
    }
}

fn render_document(
    renderer: &Renderer,
    site: &SiteConfig,
    doc: &DocInfo,
) -> Result<String, CliError> {
    let result = renderer
        .render(&doc.content, None)
        .map_err(|e| CliError::build(format!("Render failed for '{}': {}", doc.slug, e)))?;

    let ctx = RenderContext {
        title: doc.title.clone(),
        content: result.content,
        author: None,
        metadata: Some(RenderMetadata {
            created_at: doc.created_at.clone(),
            updated_at: doc.updated_at.clone(),
            tags: doc.tags.clone(),
            read_time: None,
        }),
        navigation: None,
    };

    Ok(render_full_page(&ctx, site))
}

fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            _ => out.push(c),
        }
    }
    out
}

fn generate_index(
    output_dir: &Path,
    docs: &[DocInfo],
    site: &SiteConfig,
) -> CliResult<()> {
    let mut items = String::new();
    for doc in docs {
        let title = escape_html(&doc.title);
        let desc = escape_html(doc.description.as_deref().unwrap_or("No description"));
        let slug = escape_html(&doc.slug);
        let tags_csv = escape_html(&doc.tags.join(","));
        let tags_html = if doc.tags.is_empty() {
            String::new()
        } else {
            let tags: Vec<String> = doc
                .tags
                .iter()
                .map(|t| {
                    format!(
                        r#"<span class="inline-block px-2 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200">{}</span>"#,
                        escape_html(t)
                    )
                })
                .collect();
            format!(r#"<div class="flex flex-wrap gap-1 mt-1">{}</div>"#, tags.join(""))
        };

        items.push_str(&format!(
            r#"<div class="doc-item border-b border-gray-200 dark:border-gray-700 py-4" data-title="{title}" data-tags="{tags}">
                    <a href="docs/{slug}/" class="text-xl font-semibold text-blue-600 dark:text-blue-400 hover:underline">{title}</a>
                    <p class="text-gray-600 dark:text-gray-400 mt-1">{desc}</p>
                    {tags_html}
                </div>"#,
            title = title,
            slug = slug,
            desc = desc,
            tags = tags_csv,
        ));
    }

    let search_js = r#"<script>
document.getElementById('search').addEventListener('input', function() {
    var query = this.value.toLowerCase();
    var items = document.querySelectorAll('.doc-item');
    var visible = 0;
    items.forEach(function(item) {
        var title = item.getAttribute('data-title').toLowerCase();
        var tags = item.getAttribute('data-tags').toLowerCase();
        var match = title.indexOf(query) !== -1 || tags.indexOf(query) !== -1;
        item.style.display = match ? '' : 'none';
        if (match) visible++;
    });
    document.getElementById('no-results').style.display = visible > 0 ? 'none' : 'block';
});
</script>"#;

    let content = format!(
        r#"<div class="mb-6">
                <input type="text" id="search" placeholder="Search documents..." 
                    class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-blue-500 focus:border-transparent outline-none">
            </div>
            <div id="doc-list">{items}</div>
            <p id="no-results" class="text-gray-500 dark:text-gray-400 text-center py-8" style="display:none">No documents found.</p>
            {search_js}"#,
        items = items,
        search_js = search_js,
    );

    let ctx = RenderContext::new(format!("{} - Index", site.site_title), content);
    let html = render_full_page(&ctx, site);

    let index_path = output_dir.join("index.html");
    fs::write(&index_path, html)
        .map_err(|e| CliError::io(&index_path, format!("Failed to write index.html: {}", e)))?;

    Ok(())
}

fn generate_sitemap(output_dir: &Path, docs: &[DocInfo], base_url: &str) -> CliResult<()> {
    let mut urls = String::new();
    let base = base_url.trim_end_matches('/');
    for doc in docs {
        urls.push_str(&format!(
            r#"  <url>
    <loc>{base}/docs/{slug}/</loc>
    <lastmod>{updated}</lastmod>
  </url>
"#,
            base = base,
            slug = doc.slug,
            updated = doc.updated_at,
        ));
    }

    let sitemap = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
{urls}
</urlset>"#,
        urls = urls,
    );

    let sitemap_path = output_dir.join("sitemap.xml");
    fs::write(&sitemap_path, sitemap)
        .map_err(|e| CliError::io(&sitemap_path, format!("Failed to write sitemap.xml: {}", e)))?;

    Ok(())
}

fn load_manifest(output_dir: &Path) -> Option<BuildManifest> {
    let path = output_dir.join(".build-manifest.json");
    if !path.exists() {
        return None;
    }
    let data = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&data).ok()
}

fn write_manifest(output_dir: &Path, manifest: &BuildManifest) -> CliResult<()> {
    let path = output_dir.join(".build-manifest.json");
    let json = serde_json::to_string_pretty(manifest)
        .map_err(|e| CliError::generic(format!("Failed to serialize manifest: {}", e)))?;
    fs::write(&path, json)
        .map_err(|e| CliError::io(&path, format!("Failed to write manifest: {}", e)))?;
    Ok(())
}

fn remove_stale(
    output_dir: &Path,
    prev: &Option<BuildManifest>,
    current: &BuildManifest,
    verbose: bool,
) -> CliResult<usize> {
    let prev = match prev {
        Some(p) => p,
        None => return Ok(0),
    };

    let mut removed: usize = 0;
    for (slug, entry) in &prev.documents {
        if !current.documents.contains_key(slug) {
            let path = output_dir.join(&entry.output_file);
            if path.exists() {
                if let Some(parent) = path.parent() {
                    match fs::remove_dir_all(parent) {
                        Ok(()) => {
                            removed += 1;
                            if verbose {
                                println!("  [removed] {}", slug);
                            }
                        }
                        Err(e) => {
                            eprintln!(
                                "Warning: failed to remove stale directory {}: {}",
                                parent.display(),
                                e
                            );
                        }
                    }
                }
            }
        }
    }
    Ok(removed)
}

fn get_commit_hash(repo_path: &Path) -> Option<String> {
    let repo = git2::Repository::discover(repo_path).ok()?;
    let head = repo.head().ok()?;
    let commit = head.peel_to_commit().ok()?;
    let hash = commit.id().to_string();
    Some(hash[..8].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_build_command_from_args_defaults() {
        let cmd = BuildCommand::from_args(None, None, None, None, None, None, false, false, false, None);
        assert_eq!(cmd.repo_path, PathBuf::from("."));
        assert_eq!(cmd.output_dir, PathBuf::from("dist"));
        assert!(cmd.database_url.is_none());
        assert_eq!(cmd.site_title, "Tachyon Docs");
        assert_eq!(cmd.site_description, "Knowledge Management System");
        assert_eq!(cmd.base_url, "/");
        assert!(!cmd.published_only);
        assert!(!cmd.clean);
        assert!(!cmd.verbose);
        assert!(cmd.template.is_none());
    }

    #[test]
    fn test_build_command_from_args_custom() {
        let cmd = BuildCommand::from_args(
            Some(PathBuf::from("/repo")),
            Some(PathBuf::from("/out")),
            Some("postgres://localhost/db".to_string()),
            Some("My Site".to_string()),
            Some("My Desc".to_string()),
            Some("https://example.com".to_string()),
            true,
            true,
            true,
            Some(PathBuf::from("/templates")),
        );
        assert_eq!(cmd.repo_path, PathBuf::from("/repo"));
        assert_eq!(cmd.output_dir, PathBuf::from("/out"));
        assert_eq!(cmd.database_url, Some("postgres://localhost/db".to_string()));
        assert_eq!(cmd.site_title, "My Site");
        assert_eq!(cmd.site_description, "My Desc");
        assert_eq!(cmd.base_url, "https://example.com");
        assert!(cmd.published_only);
        assert!(cmd.clean);
        assert!(cmd.verbose);
        assert_eq!(cmd.template, Some(PathBuf::from("/templates")));
    }

    #[test]
    fn test_resolve_database_url_with_flag() {
        let cmd = BuildCommand::from_args(
            None,
            None,
            Some("postgres://user:pass@host/db".to_string()),
            None,
            None,
            None,
            false,
            false,
            false,
            None,
        );
        let url = cmd.resolve_database_url().unwrap();
        assert_eq!(url, "postgres://user:pass@host/db");
    }

    #[test]
    fn test_resolve_database_url_missing() {
        let cmd = BuildCommand::from_args(None, None, None, None, None, None, false, false, false, None);
        let result = cmd.resolve_database_url();
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("No database URL provided"));
        assert!(err.contains("SQLite is not yet supported"));
    }

    #[test]
    fn test_manifest_round_trip() {
        let dir = tempdir().unwrap();
        let output = dir.path().join("dist");
        fs::create_dir_all(&output).unwrap();

        let manifest = BuildManifest {
            build_time: "2026-04-07T00:00:00+00:00".to_string(),
            commit_hash: Some("abc12345".to_string()),
            documents: {
                let mut map = BTreeMap::new();
                map.insert(
                    "getting-started".to_string(),
                    DocumentEntry {
                        content_hash: "deadbeef".to_string(),
                        output_file: "docs/getting-started/index.html".to_string(),
                        built_at: "2026-04-07T00:00:00+00:00".to_string(),
                    },
                );
                map
            },
        };

        write_manifest(&output, &manifest).unwrap();
        let loaded = load_manifest(&output).unwrap();
        assert_eq!(loaded.build_time, manifest.build_time);
        assert_eq!(loaded.commit_hash, manifest.commit_hash);
        assert_eq!(loaded.documents.len(), 1);
        assert_eq!(
            loaded.documents["getting-started"].content_hash,
            "deadbeef"
        );
    }

    #[test]
    fn test_load_manifest_missing() {
        let dir = tempdir().unwrap();
        let output = dir.path().join("nonexistent");
        assert!(load_manifest(&output).is_none());
    }

    #[test]
    fn test_escape_html() {
        assert_eq!(escape_html("<script>alert('xss')</script>"), "&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;");
        assert_eq!(escape_html("Hello & <World>"), "Hello &amp; &lt;World&gt;");
        assert_eq!(escape_html("plain text"), "plain text");
    }

    #[test]
    fn test_remove_stale() {
        let dir = tempdir().unwrap();
        let output = dir.path().join("dist");
        fs::create_dir_all(output.join("docs").join("old-doc")).unwrap();
        fs::write(output.join("docs").join("old-doc").join("index.html"), "old").unwrap();

        let prev = BuildManifest {
            build_time: "2026-01-01T00:00:00+00:00".to_string(),
            commit_hash: None,
            documents: {
                let mut map = BTreeMap::new();
                map.insert(
                    "old-doc".to_string(),
                    DocumentEntry {
                        content_hash: "aaa".to_string(),
                        output_file: "docs/old-doc/index.html".to_string(),
                        built_at: "2026-01-01T00:00:00+00:00".to_string(),
                    },
                );
                map
            },
        };

        let current = BuildManifest {
            build_time: "2026-04-07T00:00:00+00:00".to_string(),
            commit_hash: None,
            documents: BTreeMap::new(),
        };

        let removed = remove_stale(&output, &Some(prev), &current, false).unwrap();
        assert_eq!(removed, 1);
        assert!(!output.join("docs").join("old-doc").exists());
    }

    #[test]
    fn test_command_name_and_description() {
        let cmd = BuildCommand::from_args(None, None, None, None, None, None, false, false, false, None);
        assert_eq!(cmd.name(), "build");
        assert!(cmd.description().contains("static site"));
    }
}
