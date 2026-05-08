use tachyon_core::compute_content_hash;
use tachyon_import_export::DocusaurusImporter;

fn create_test_dir() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();

    let infra_linux = dir.path().join("docs_infrastructure").join("linux");
    std::fs::create_dir_all(&infra_linux).unwrap();

    std::fs::write(
        infra_linux.join("bash-scripting.md"),
        r#"---
id: bash-scripting
title: Bash Scripting
description: Bash scripting guide
slug: bash-scripting
sidebar_position: 4
---

## Conditional Expressions

### if / elif / else / fi

```bash
if [[ condition ]]; then
    commands
fi
```

### Mermaid Diagram

```mermaid
graph LR
    A[Input] --> B[Process]
    B --> C[Output]
```
"#,
    )
    .unwrap();

    let infra_git = dir.path().join("docs_infrastructure").join("git");
    std::fs::create_dir_all(&infra_git).unwrap();

    std::fs::write(
        infra_git.join("commits.md"),
        r#"---
title: Git Commits
tags:
  - git
  - vcs
---

# Commits

Basic commit workflow.
"#,
    )
    .unwrap();

    let cpp = dir.path().join("docs_cpp").join("algorithms");
    std::fs::create_dir_all(&cpp).unwrap();

    std::fs::write(
        cpp.join("sorting.md"),
        "# Sorting Algorithms\n\nNo frontmatter here.\n",
    )
    .unwrap();

    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    std::fs::write(dir.path().join(".git").join("hidden.md"), "hidden").unwrap();

    std::fs::create_dir_all(dir.path().join("node_modules")).unwrap();
    std::fs::write(dir.path().join("node_modules").join("pkg.md"), "hidden").unwrap();

    std::fs::write(dir.path().join("image.png"), "PNG_DATA").unwrap();
    std::fs::write(dir.path().join("empty.md"), "").unwrap();

    dir
}

#[test]
fn test_docusaurus_import_frontmatter_parsing() {
    let dir = create_test_dir();
    let (docs, _summary) = DocusaurusImporter::import_from_dir(dir.path()).unwrap();

    let bash = docs
        .iter()
        .find(|d| d.source_path.contains("bash-scripting"))
        .unwrap();
    assert_eq!(bash.title, "Bash Scripting");
    assert_eq!(bash.slug.as_deref(), Some("bash-scripting"));
    assert!(bash
        .frontmatter
        .description
        .as_ref()
        .unwrap()
        .contains("Bash"));
    assert!(!bash.content.contains("---"));
    assert!(bash.content.contains("## Conditional Expressions"));
}

#[test]
fn test_docusaurus_import_tag_extraction_from_paths() {
    let dir = create_test_dir();
    let (docs, _summary) = DocusaurusImporter::import_from_dir(dir.path()).unwrap();

    let bash = docs
        .iter()
        .find(|d| d.source_path.contains("bash-scripting"))
        .unwrap();
    assert!(bash.tags.contains(&"infrastructure".to_string()));
    assert!(bash.tags.contains(&"linux".to_string()));

    let git = docs
        .iter()
        .find(|d| d.source_path.contains("commits"))
        .unwrap();
    assert!(git.tags.contains(&"infrastructure".to_string()));
    assert!(git.tags.contains(&"git".to_string()));
    assert!(git.tags.contains(&"vcs".to_string()));

    let sorting = docs
        .iter()
        .find(|d| d.source_path.contains("sorting"))
        .unwrap();
    assert!(sorting.tags.contains(&"cpp".to_string()));
    assert!(sorting.tags.contains(&"algorithms".to_string()));
}

#[test]
fn test_docusaurus_import_mermaid_preserved() {
    let dir = create_test_dir();
    let (docs, _) = DocusaurusImporter::import_from_dir(dir.path()).unwrap();

    let bash = docs
        .iter()
        .find(|d| d.source_path.contains("bash-scripting"))
        .unwrap();
    assert!(
        bash.content.contains("```mermaid"),
        "Mermaid blocks should be preserved in imported content"
    );
    assert!(bash.content.contains("graph LR"));
    assert!(bash.content.contains("Input"));
}

#[test]
fn test_docusaurus_import_duplicate_handling() {
    let dir = create_test_dir();

    let content = "---\ntitle: Duplicate\n---\n\nSame content.\n";
    std::fs::write(dir.path().join("dup-a.md"), content).unwrap();
    std::fs::write(dir.path().join("dup-b.md"), content).unwrap();

    let (docs, _summary) = DocusaurusImporter::import_from_dir(dir.path()).unwrap();

    let dups: Vec<_> = docs.iter().filter(|d| d.title == "Duplicate").collect();
    assert_eq!(dups.len(), 2, "both files should be imported");

    let hash_a = compute_content_hash(&dups[0].content);
    let hash_b = compute_content_hash(&dups[1].content);
    assert_eq!(hash_a, hash_b, "duplicate content should have same hash");
}

#[test]
fn test_docusaurus_import_skip_non_markdown() {
    let dir = create_test_dir();
    let (_, summary) = DocusaurusImporter::import_from_dir(dir.path()).unwrap();

    assert!(
        summary.skipped >= 2,
        "expected at least 2 skipped, got {} (total={})",
        summary.skipped,
        summary.total_files
    );
    assert_eq!(summary.failed, 0);
}

#[test]
fn test_docusaurus_import_summary_tags() {
    let dir = create_test_dir();
    let (_, summary) = DocusaurusImporter::import_from_dir(dir.path()).unwrap();

    assert!(summary.all_tags.contains(&"infrastructure".to_string()));
    assert!(summary.all_tags.contains(&"linux".to_string()));
    assert!(summary.all_tags.contains(&"git".to_string()));
    assert!(summary.all_tags.contains(&"cpp".to_string()));
}

#[test]
fn test_docusaurus_import_extra_fields() {
    let dir = create_test_dir();
    let (docs, _) = DocusaurusImporter::import_from_dir(dir.path()).unwrap();

    let bash = docs
        .iter()
        .find(|d| d.source_path.contains("bash-scripting"))
        .unwrap();
    assert_eq!(
        bash.extra
            .get("sidebar_position")
            .and_then(|v: &serde_json::Value| v.as_i64()),
        Some(4)
    );
    assert_eq!(
        bash.extra
            .get("id")
            .and_then(|v: &serde_json::Value| v.as_str()),
        Some("bash-scripting")
    );
}
