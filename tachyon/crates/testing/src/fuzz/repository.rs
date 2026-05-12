//! Fuzzing tests for repository operations
//!
//! Property-based tests using random input to verify repository types
//! don't panic and handle edge cases gracefully.

#[allow(unused_imports)]
use std::path::PathBuf;
#[allow(unused_imports)]
use tachyon_core::types::repository::{
    Repository, RepositoryConfig, RepositoryStatus, RepositoryType, RepositoryVisibility,
};
#[allow(unused_imports)]
use tachyon_core::{generate_repository_id, generate_user_id};

#[test]
fn test_repository_random_names_no_panic() {
    let long_name = "a".repeat(200);
    let names = vec![
        "",
        "a",
        "Valid Name",
        long_name.as_str(),
        "name with\nnewlines",
        "name with\ttabs",
        "🔥 emoji name",
        "<script>alert(1)</script>",
        "trailing space ",
        " leading space",
        "name/with/slashes",
        "name\\with\\backslashes",
    ];

    for name in &names {
        let repo = Repository::new(
            generate_repository_id(),
            name.to_string(),
            RepositoryType::Personal,
            generate_user_id(),
        );
        let _ = repo.validate();
    }
}

#[test]
fn test_repository_random_descriptions_no_panic() {
    let descriptions = vec![
        None,
        Some("".to_string()),
        Some("Normal description".to_string()),
        Some("a".repeat(10000).to_string()),
        Some("desc\nwith\nnewlines".to_string()),
        Some("<b>html</b> description".to_string()),
    ];

    for desc in &descriptions {
        let mut repo = Repository::new(
            generate_repository_id(),
            "Test Repo".to_string(),
            RepositoryType::Personal,
            generate_user_id(),
        );
        repo.metadata.description = desc.clone();
        let _ = repo.validate();
    }
}

#[test]
fn test_repository_random_paths_no_panic() {
    let paths = vec![
        None,
        Some(PathBuf::from("/")),
        Some(PathBuf::from("/tmp")),
        Some(PathBuf::from("/very/deep/nested/path/that/goes/on/and/on")),
        Some(PathBuf::from("")),
        Some(PathBuf::from("relative/path")),
        Some(PathBuf::from("../traversal")),
        Some(PathBuf::from("/path/with spaces")),
        Some(PathBuf::from("/path/with\0null")),
    ];

    for path in &paths {
        let mut repo = Repository::new(
            generate_repository_id(),
            "Test Repo".to_string(),
            RepositoryType::Personal,
            generate_user_id(),
        );
        repo.local_path = path.clone();
        let _ = repo.validate();
    }
}

#[test]
fn test_repository_all_status_transitions() {
    let statuses = vec![
        RepositoryStatus::Init,
        RepositoryStatus::Cloned,
        RepositoryStatus::Dirty,
        RepositoryStatus::Synced,
        RepositoryStatus::Conflict,
    ];

    for status in statuses {
        let repo = Repository::new(
            generate_repository_id(),
            "Test".to_string(),
            RepositoryType::Personal,
            generate_user_id(),
        )
        .with_status(status);

        match status {
            RepositoryStatus::Dirty | RepositoryStatus::Conflict => {
                assert!(repo.needs_sync());
            }
            _ => {
                assert!(!repo.needs_sync());
            }
        }
    }
}

#[test]
fn test_repository_config_random_values_no_panic() {
    let configs = vec![
        RepositoryConfig::new()
            .with_default_branch("main".to_string())
            .with_default_branch("".to_string())
            .with_default_branch("a".repeat(1000).to_string())
            .with_remote_url("https://github.com/test/repo".to_string())
            .with_remote_url("not-a-url".to_string())
            .with_remote_url("".to_string())
            .with_auto_sync(true)
            .with_auto_sync(false),
    ];

    for config in &configs {
        let json = serde_json::to_string(config).unwrap();
        let de: RepositoryConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.default_branch, de.default_branch);
    }
}

#[test]
fn test_repository_random_types_no_panic() {
    let types = vec![
        RepositoryType::Personal,
        RepositoryType::Team,
        RepositoryType::Organization,
        RepositoryType::Public,
    ];

    for rt in types {
        let repo = Repository::new(
            generate_repository_id(),
            "Test".to_string(),
            rt,
            generate_user_id(),
        );
        assert_eq!(repo.metadata.repository_type, rt);
    }
}

#[test]
fn test_repository_random_visibilities_no_panic() {
    let visibilities = vec![RepositoryVisibility::Public, RepositoryVisibility::Private];

    for vis in visibilities {
        let repo = Repository::new(
            generate_repository_id(),
            "Test".to_string(),
            RepositoryType::Personal,
            generate_user_id(),
        )
        .with_visibility(vis);
        assert_eq!(repo.visibility, Some(vis));
    }
}

#[test]
fn test_repository_serde_roundtrip_random_fields() {
    let repos = vec![
        Repository::new(
            generate_repository_id(),
            "Test Repo".to_string(),
            RepositoryType::Personal,
            generate_user_id(),
        ),
        Repository::new(
            generate_repository_id(),
            "Team Repo".to_string(),
            RepositoryType::Team,
            generate_user_id(),
        )
        .with_visibility(RepositoryVisibility::Public),
        Repository::new(
            generate_repository_id(),
            "".to_string(),
            RepositoryType::Personal,
            generate_user_id(),
        ),
    ];

    for repo in &repos {
        let json = serde_json::to_string(repo).unwrap();
        let de: Repository = serde_json::from_str(&json).unwrap();
        assert_eq!(repo.id, de.id);
    }
}

#[test]
fn test_repository_mark_operations_sequence() {
    let mut repo = Repository::new(
        generate_repository_id(),
        "Test".to_string(),
        RepositoryType::Personal,
        generate_user_id(),
    )
    .with_status(RepositoryStatus::Cloned);

    repo.mark_dirty();
    assert_eq!(repo.status, RepositoryStatus::Dirty);

    repo.mark_synced();
    assert_eq!(repo.status, RepositoryStatus::Synced);

    repo.mark_conflict();
    assert_eq!(repo.status, RepositoryStatus::Conflict);
}
