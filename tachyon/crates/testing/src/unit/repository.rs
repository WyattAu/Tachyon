//! Unit tests for repository types
//!
//! Tests for repository creation, status transitions, configuration,
//! validation, and builder pattern.

#[allow(unused_imports)]
use std::path::PathBuf;
use tachyon_core::{generate_repository_id, generate_user_id};
#[allow(unused_imports)]
use tachyon_core::types::repository::{
    GitOperations, Repository, RepositoryBuilder, RepositoryConfig, RepositoryMetadata,
    RepositoryStatus, RepositoryType, RepositoryVisibility,
};

#[allow(dead_code)]
fn make_repo() -> Repository {
    Repository::new(
        generate_repository_id(),
        "Test Repo".to_string(),
        RepositoryType::Personal,
        generate_user_id(),
    )
}

#[test]
fn test_repository_creation() {
    let repo = make_repo();
    assert_eq!(repo.metadata.name, "Test Repo");
    assert_eq!(repo.status, RepositoryStatus::Init);
    assert_eq!(repo.metadata.repository_type, RepositoryType::Personal);
    assert!(!repo.is_ready());
    assert!(!repo.needs_sync());
}

#[test]
fn test_repository_default_visibility() {
    let repo = make_repo();
    assert_eq!(repo.visibility, Some(RepositoryVisibility::Private));
}

#[test]
fn test_repository_with_visibility() {
    let repo = make_repo().with_visibility(RepositoryVisibility::Public);
    assert_eq!(repo.visibility, Some(RepositoryVisibility::Public));
}

#[test]
fn test_repository_with_local_path() {
    let path = PathBuf::from("/tmp/test/repo");
    let repo = make_repo().with_local_path(path.clone());
    assert_eq!(repo.local_path, Some(path));
}

#[test]
fn test_repository_with_status() {
    let repo = make_repo().with_status(RepositoryStatus::Cloned);
    assert_eq!(repo.status, RepositoryStatus::Cloned);
    assert!(repo.is_ready());
}

#[test]
fn test_repository_mark_dirty() {
    let mut repo = make_repo().with_status(RepositoryStatus::Synced);
    repo.mark_dirty();
    assert_eq!(repo.status, RepositoryStatus::Dirty);
    assert!(repo.needs_sync());
}

#[test]
fn test_repository_mark_synced() {
    let mut repo = make_repo().with_status(RepositoryStatus::Dirty);
    repo.mark_synced();
    assert_eq!(repo.status, RepositoryStatus::Synced);
    assert!(!repo.needs_sync());
}

#[test]
fn test_repository_mark_conflict() {
    let mut repo = make_repo().with_status(RepositoryStatus::Synced);
    repo.mark_conflict();
    assert_eq!(repo.status, RepositoryStatus::Conflict);
    assert!(repo.needs_sync());
}

#[test]
fn test_repository_validation_ok() {
    let repo = make_repo();
    assert!(repo.validate().is_ok());
}

#[test]
fn test_repository_validation_empty_name() {
    let repo = Repository::new(
        generate_repository_id(),
        "".to_string(),
        RepositoryType::Personal,
        generate_user_id(),
    );
    assert!(repo.validate().is_err());
}

#[test]
fn test_repository_validation_name_too_long() {
    let repo = Repository::new(
        generate_repository_id(),
        "a".repeat(101),
        RepositoryType::Personal,
        generate_user_id(),
    );
    assert!(repo.validate().is_err());
}

#[test]
fn test_repository_update_metadata() {
    let mut repo = make_repo();
    repo.update_metadata("New Name".to_string(), Some("New desc".to_string()));
    assert_eq!(repo.metadata.name, "New Name");
    assert_eq!(repo.metadata.description.as_deref(), Some("New desc"));
    assert_eq!(repo.metadata.slug.as_deref(), Some("new-name"));
}

#[test]
fn test_repository_config_defaults() {
    let config = RepositoryConfig::new();
    assert_eq!(config.default_branch, Some("main".to_string()));
    assert_eq!(config.auto_sync, Some(false));
    assert_eq!(config.sync_interval_seconds, Some(300));
    assert_eq!(config.file_watching_enabled, Some(true));
    assert!(config.remote_url.is_none());
}

#[test]
fn test_repository_config_builder() {
    let config = RepositoryConfig::new()
        .with_default_branch("develop".to_string())
        .with_remote_url("https://github.com/test/repo".to_string())
        .with_auto_sync(true);

    assert_eq!(config.default_branch, Some("develop".to_string()));
    assert_eq!(config.remote_url.as_deref(), Some("https://github.com/test/repo"));
    assert_eq!(config.auto_sync, Some(true));
}

#[test]
fn test_git_operations_defaults() {
    let ops = GitOperations::new();
    assert!(ops.last_commit_hash.is_none());
    assert!(ops.current_branch.is_none());
    assert!(ops.commits_ahead.is_none());
    assert!(ops.commits_behind.is_none());
}

#[test]
fn test_repository_status_needs_sync() {
    assert!(RepositoryStatus::Dirty.needs_sync());
    assert!(RepositoryStatus::Conflict.needs_sync());
    assert!(!RepositoryStatus::Synced.needs_sync());
    assert!(!RepositoryStatus::Init.needs_sync());
    assert!(!RepositoryStatus::Cloned.needs_sync());
}

#[test]
fn test_repository_status_is_ready() {
    assert!(RepositoryStatus::Cloned.is_ready());
    assert!(RepositoryStatus::Synced.is_ready());
    assert!(RepositoryStatus::Dirty.is_ready());
    assert!(!RepositoryStatus::Init.is_ready());
    assert!(!RepositoryStatus::Conflict.is_ready());
}

#[test]
fn test_repository_builder() {
    let user_id = generate_user_id();
    let repo = RepositoryBuilder::new("My Repo".to_string(), RepositoryType::Team, user_id)
        .description("A test repo".to_string())
        .visibility(RepositoryVisibility::Public)
        .local_path(PathBuf::from("/tmp/repo"))
        .remote_url("https://github.com/org/repo".to_string())
        .default_branch("develop".to_string())
        .build()
        .expect("should build");

    assert_eq!(repo.metadata.name, "My Repo");
    assert_eq!(repo.visibility, Some(RepositoryVisibility::Public));
    assert_eq!(repo.metadata.description.as_deref(), Some("A test repo"));
}

#[test]
fn test_repository_metadata_slug_generation() {
    let repo = Repository::new(
        generate_repository_id(),
        "My Test Repo".to_string(),
        RepositoryType::Personal,
        generate_user_id(),
    );
    assert_eq!(repo.metadata.slug.as_deref(), Some("my-test-repo"));
}

#[test]
fn test_repository_serde_roundtrip() {
    let repo = make_repo().with_visibility(RepositoryVisibility::Public);
    let json = serde_json::to_string(&repo).expect("serialize");
    let de: Repository = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(repo.id, de.id);
    assert_eq!(repo.metadata.name, de.metadata.name);
}

#[test]
fn test_all_repository_types() {
    for rt in [
        RepositoryType::Personal,
        RepositoryType::Team,
        RepositoryType::Organization,
        RepositoryType::Public,
    ] {
        let repo = Repository::new(
            generate_repository_id(),
            "Test".to_string(),
            rt,
            generate_user_id(),
        );
        assert_eq!(repo.metadata.repository_type, rt);
    }
}
