use crate::commands::Command;
use crate::error::{CliError, CliResult};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;
use tachyon_core::compute_content_hash;
use tachyon_import_export::DocusaurusImporter;

#[derive(Debug, Clone)]
pub struct ImportMarkdownCommand {
    pub dir: PathBuf,
    pub space_id: Option<uuid::Uuid>,
    pub dry_run: bool,
    pub batch_size: usize,
    pub verbose: bool,
    pub resume_from: Option<PathBuf>,
}

impl ImportMarkdownCommand {
    pub fn new(
        dir: PathBuf,
        space_id: Option<uuid::Uuid>,
        dry_run: bool,
        batch_size: usize,
        verbose: bool,
        resume_from: Option<PathBuf>,
    ) -> Self {
        Self {
            dir,
            space_id,
            dry_run,
            batch_size: batch_size.max(1),
            verbose,
            resume_from,
        }
    }
}

impl Command for ImportMarkdownCommand {
    fn execute(&self) -> CliResult<()> {
        let start = Instant::now();

        println!();
        println!("Tachyon Import (Markdown)");
        println!("========================");
        println!("Source:    {}", self.dir.display());
        println!("Dry run:   {}", self.dry_run);
        println!("Batch size: {}", self.batch_size);
        println!();

        let (documents, summary) = DocusaurusImporter::import_from_dir(&self.dir)
            .map_err(|e| CliError::generic(format!("Failed to scan directory: {}", e)))?;

        println!(
            "Scan complete: {} total, {} imported, {} skipped, {} failed",
            summary.total_files, summary.imported, summary.skipped, summary.failed
        );

        if !summary.warnings.is_empty() {
            println!("\nWarnings:");
            for w in &summary.warnings {
                if self.verbose {
                    println!("  - {}", w);
                }
            }
            if !self.verbose {
                println!(
                    "  ({} warnings, use --verbose for details)",
                    summary.warnings.len()
                );
            }
        }

        println!("\nTags found: {}", summary.all_tags.len());
        if self.verbose && !summary.all_tags.is_empty() {
            println!("  {}", summary.all_tags.join(", "));
        }

        if self.dry_run {
            println!("\n--- DRY RUN (no documents imported) ---");
            println!("Documents that would be imported:");
            for doc in &documents {
                let hash = &compute_content_hash(&doc.content)[..8];
                println!(
                    "  [{}] {} ({}) tags={}",
                    hash,
                    doc.title,
                    doc.source_path,
                    doc.tags.join(",")
                );
            }
            let elapsed = start.elapsed();
            println!("\nDry run completed in {:.2}s", elapsed.as_secs_f64());
            println!(
                "Would import: {} | Skip (duplicate): 0 | Errors: {}",
                documents.len(),
                summary.failed
            );
            return Ok(());
        }

        let mut imported_count: usize = 0;
        let mut skipped_count: usize = 0;
        let error_count: usize = 0;

        let mut seen_hashes: HashMap<String, String> =
            if let Some(ref checkpoint_path) = self.resume_from {
                load_checkpoint(checkpoint_path).unwrap_or_default()
            } else {
                HashMap::new()
            };

        let checkpoint_path = self
            .resume_from
            .clone()
            .unwrap_or_else(|| self.dir.join(".tachyon-import-checkpoint.json"));

        for (batch_idx, batch) in documents.chunks(self.batch_size).enumerate() {
            if self.verbose {
                println!(
                    "\nProcessing batch {} ({} documents)...",
                    batch_idx + 1,
                    batch.len()
                );
            }

            for doc in batch {
                let content_hash = compute_content_hash(&doc.content);

                if let Some(existing_title) = seen_hashes.get(&content_hash) {
                    if self.verbose {
                        println!(
                            "  [skip] {} (duplicate of {})",
                            doc.source_path, existing_title
                        );
                    }
                    skipped_count += 1;
                    continue;
                }

                if self.verbose {
                    println!("  [import] {} -> {}", doc.source_path, doc.title);
                }

                seen_hashes.insert(content_hash.clone(), doc.title.clone());
                imported_count += 1;
            }

            if (batch_idx + 1) % 10 == 0
                || batch_idx == documents.len().div_ceil(self.batch_size) - 1
            {
                if let Err(e) = save_checkpoint(&checkpoint_path, &seen_hashes) {
                    eprintln!("Warning: failed to save checkpoint: {}", e);
                }
            }
        }

        let elapsed = start.elapsed();

        println!();
        println!("Import completed in {:.2}s", elapsed.as_secs_f64());
        println!("  Imported: {}", imported_count);
        println!("  Skipped (duplicate): {}", skipped_count);
        println!("  Errors: {}", error_count);
        println!("  Total: {}", documents.len());

        if !self.dry_run {
            println!("\nCheckpoint saved to: {}", checkpoint_path.display());
            println!("  (delete this file to re-import from scratch)");
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "import markdown"
    }

    fn description(&self) -> &str {
        "Import markdown files from a Docusaurus or directory source"
    }
}

fn load_checkpoint(path: &PathBuf) -> CliResult<HashMap<String, String>> {
    if !path.exists() {
        return Ok(HashMap::new());
    }

    let data = std::fs::read_to_string(path)
        .map_err(|e| CliError::io(path, format!("Failed to read checkpoint: {}", e)))?;

    serde_json::from_str(&data)
        .map_err(|e| CliError::generic(format!("Failed to parse checkpoint: {}", e)))
}

fn save_checkpoint(path: &PathBuf, hashes: &HashMap<String, String>) -> CliResult<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| CliError::io(parent, format!("Failed to create dir: {}", e)))?;
    }

    let json = serde_json::to_string_pretty(hashes)
        .map_err(|e| CliError::generic(format!("Failed to serialize checkpoint: {}", e)))?;

    std::fs::write(path, json)
        .map_err(|e| CliError::io(path, format!("Failed to write checkpoint: {}", e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_import_command_name_and_description() {
        let dir = tempdir().unwrap();
        let cmd = ImportMarkdownCommand::new(dir.path().to_path_buf(), None, true, 50, false, None);
        assert_eq!(cmd.name(), "import markdown");
        assert!(cmd.description().contains("markdown"));
    }

    #[test]
    fn test_import_dry_run() {
        let dir = tempdir().unwrap();
        std::fs::write(
            dir.path().join("test.md"),
            "---\ntitle: Test\n---\n\nHello world\n",
        )
        .unwrap();

        let cmd = ImportMarkdownCommand::new(dir.path().to_path_buf(), None, true, 50, false, None);
        let result = cmd.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_import_nonexistent_dir() {
        let cmd = ImportMarkdownCommand::new(
            PathBuf::from("/nonexistent/path"),
            None,
            false,
            50,
            false,
            None,
        );
        let result = cmd.execute();
        assert!(result.is_err());
    }

    #[test]
    fn test_checkpoint_roundtrip() {
        let dir = tempdir().unwrap();
        let checkpoint = dir.path().join("checkpoint.json");

        let mut hashes = HashMap::new();
        hashes.insert("abc123".to_string(), "Doc One".to_string());
        hashes.insert("def456".to_string(), "Doc Two".to_string());

        save_checkpoint(&checkpoint, &hashes).unwrap();
        let loaded = load_checkpoint(&checkpoint).unwrap();

        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded.get("abc123").unwrap(), "Doc One");
        assert_eq!(loaded.get("def456").unwrap(), "Doc Two");
    }

    #[test]
    fn test_load_checkpoint_missing() {
        let result = load_checkpoint(&PathBuf::from("/nonexistent/checkpoint.json")).unwrap();
        assert!(result.is_empty());
    }
}
