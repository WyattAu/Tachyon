//! Synchronization commands for external editors.

use crate::commands::Command;
use crate::error::{CliError, CliResult};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use tachyon_core::compute_content_hash;
use tachyon_database::{DocumentRepository, init_with_migrations};
use tachyon_import_export::{MarkdownZipExporter, ObsidianImporter};

const MANIFEST_NAME: &str = ".tachyon-sync.json";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct SyncManifest {
    version: u32,
    generated_at: String,
    documents: BTreeMap<String, SyncEntry>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SyncEntry {
    title: String,
    path: String,
    content_hash: String,
    updated_at: String,
}

#[derive(Debug, Clone)]
pub struct PullCommand {
    pub output: PathBuf,
    pub database_url: Option<String>,
    pub force: bool,
}
impl PullCommand {
    pub fn new(output: PathBuf, database_url: Option<String>, force: bool) -> Self {
        Self {
            output,
            database_url,
            force,
        }
    }
    fn database_url(&self) -> CliResult<String> {
        self.database_url
            .clone()
            .or_else(|| std::env::var("DATABASE_URL").ok())
            .ok_or_else(|| CliError::invalid_argument("provide --database-url or DATABASE_URL"))
    }
}
impl Command for PullCommand {
    fn execute(&self) -> CliResult<()> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| CliError::generic(format!("Failed to create runtime: {e}")))?;
        rt.block_on(async {
            if self.output.exists()
                && !self.force
                && fs::read_dir(&self.output)
                    .map_err(|e| CliError::io(&self.output, e.to_string()))?
                    .next()
                    .is_some()
            {
                return Err(CliError::invalid_argument(format!(
                    "output directory '{}' is not empty; use --force",
                    self.output.display()
                )));
            }
            fs::create_dir_all(&self.output)
                .map_err(|e| CliError::io(&self.output, e.to_string()))?;
            let pool = init_with_migrations(&self.database_url()?)
                .await
                .map_err(|e| CliError::database(format!("Failed to connect: {e}")))?;
            let docs = DocumentRepository::new(pool)
                .list_all(Some(100_000), None)
                .await
                .map_err(|e| CliError::database(format!("Failed to list documents: {e}")))?;
            let mut manifest = SyncManifest {
                version: 1,
                generated_at: chrono::Utc::now().to_rfc3339(),
                ..Default::default()
            };
            let mut written = 0;
            for doc in docs {
                let slug = match doc.slug.clone() {
                    Some(slug) if !slug.is_empty() => slug,
                    _ => continue,
                };
                let content = doc.content.clone().unwrap_or_default();
                let markdown = MarkdownZipExporter::document_to_markdown(
                    &doc.title,
                    &content,
                    &doc.parse_tags().unwrap_or_default(),
                    doc.description.as_deref(),
                    Some(doc.created_at),
                    Some(doc.updated_at),
                );
                let relative = safe_markdown_path(&slug);
                let path = self.output.join(&relative);
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent).map_err(|e| CliError::io(parent, e.to_string()))?;
                }
                fs::write(&path, markdown).map_err(|e| CliError::io(&path, e.to_string()))?;
                manifest.documents.insert(
                    slug,
                    SyncEntry {
                        title: doc.title,
                        path: relative.to_string_lossy().to_string(),
                        content_hash: doc
                            .content_hash
                            .unwrap_or_else(|| compute_content_hash(&content)),
                        updated_at: doc.updated_at.to_rfc3339(),
                    },
                );
                written += 1;
            }
            write_manifest(&self.output, &manifest)?;
            println!(
                "Pulled {written} document(s) into {}",
                self.output.display()
            );
            Ok(())
        })
    }
    fn name(&self) -> &str {
        "pull"
    }
    fn description(&self) -> &str {
        "Pull Tachyon documents into an editor-friendly markdown vault"
    }
}

#[derive(Debug, Clone)]
pub struct PushCommand {
    pub input: PathBuf,
    pub database_url: Option<String>,
    pub dry_run: bool,
    pub force: bool,
}
impl PushCommand {
    pub fn new(input: PathBuf, database_url: Option<String>, dry_run: bool) -> Self {
        Self {
            input,
            database_url,
            dry_run,
            force: false,
        }
    }
    pub fn with_force(mut self, force: bool) -> Self {
        self.force = force;
        self
    }
    fn database_url(&self) -> CliResult<String> {
        self.database_url
            .clone()
            .or_else(|| std::env::var("DATABASE_URL").ok())
            .ok_or_else(|| CliError::invalid_argument("provide --database-url or DATABASE_URL"))
    }
}
impl Command for PushCommand {
    fn execute(&self) -> CliResult<()> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| CliError::generic(format!("Failed to create runtime: {e}")))?;
        rt.block_on(async {
            if !self.input.is_dir() { return Err(CliError::io(&self.input, "not a directory")); }
            let manifest = read_manifest(&self.input)?;
            let (docs, summary) = ObsidianImporter::import_from_dir(&self.input).map_err(|e| CliError::generic(format!("Failed to scan vault: {e}")))?;
            println!("Found {} document(s): {} imported, {} skipped, {} failed", summary.total_files, summary.imported, summary.skipped, summary.failed);
            if self.dry_run { for doc in &docs { println!("  {} -> {}", doc.source_path, doc.title); } println!("Dry run: no documents changed."); return Ok(()); }
            let pool = init_with_migrations(&self.database_url()?).await.map_err(|e| CliError::database(format!("Failed to connect: {e}")))?; let repo = DocumentRepository::new(pool); let mut updated = 0;
            for doc in docs {
                let slug = doc.effective_slug(); let content = doc.content.clone(); let hash = compute_content_hash(&content); let existing = repo.get_by_slug(&slug).await.map_err(|e| CliError::database(format!("Failed to find '{slug}': {e}")))?;
                if let Some(mut current) = existing {
                    if current.content_hash.as_deref() == Some(hash.as_str()) { continue; }
                    if !self.force { if let Some(entry) = manifest.documents.get(&slug) { let server_hash = current.content_hash.clone().unwrap_or_else(|| compute_content_hash(current.content.as_deref().unwrap_or(""))); if server_hash != entry.content_hash { return Err(CliError::invalid_argument(format!("conflict on '{slug}': server changed since pull; review and use --force to overwrite"))); } } }
                    current.title = doc.title; current.slug = Some(slug); current.description = doc.frontmatter.description.clone(); current.tags = serde_json::to_string(&doc.tags).unwrap_or_else(|_| "[]".into()); current.content = Some(content); current.content_hash = Some(hash); current.updated_at = chrono::Utc::now(); repo.update(current).await.map_err(|e| CliError::database(format!("Failed to update document: {e}")))?; updated += 1;
                } else { println!("  Skipping new document '{}' (author/project context required)", doc.title); }
            }
            println!("Push complete: 0 created, {updated} updated"); Ok(())
        })
    }
    fn name(&self) -> &str {
        "push"
    }
    fn description(&self) -> &str {
        "Push an editor-friendly markdown vault into Tachyon"
    }
}

fn safe_markdown_path(slug: &str) -> PathBuf {
    PathBuf::from(format!("{}.md", slug.replace(['\\', '/'], "-")))
}
fn read_manifest(root: &Path) -> CliResult<SyncManifest> {
    let path = root.join(MANIFEST_NAME);
    if !path.exists() {
        return Ok(SyncManifest::default());
    }
    let bytes = fs::read(&path).map_err(|e| CliError::io(&path, e.to_string()))?;
    serde_json::from_slice(&bytes)
        .map_err(|e| CliError::invalid_argument(format!("invalid sync manifest: {e}")))
}
fn write_manifest(root: &Path, manifest: &SyncManifest) -> CliResult<()> {
    let path = root.join(MANIFEST_NAME);
    let bytes = serde_json::to_vec_pretty(manifest)
        .map_err(|e| CliError::generic(format!("Failed to serialize sync manifest: {e}")))?;
    fs::write(&path, bytes).map_err(|e| CliError::io(&path, e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn paths_cannot_escape_vault() {
        assert_eq!(
            safe_markdown_path("../secret"),
            PathBuf::from("..-secret.md")
        );
        assert_eq!(
            safe_markdown_path("notes/my-page"),
            PathBuf::from("notes-my-page.md")
        );
    }
    #[test]
    fn missing_manifest_is_allowed() {
        let dir = tempfile::tempdir().unwrap();
        assert!(read_manifest(dir.path()).unwrap().documents.is_empty());
    }
}
