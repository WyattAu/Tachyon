use tachyon_database::DocumentRepository;

use crate::common::setup::{
    create_test_document, create_test_pool, create_test_user, setup_database, teardown_database,
};

fn skip_without_db() -> bool {
    std::env::var("DATABASE_URL").is_err()
        && std::env::var("TEST_DATABASE_URL").is_err()
}

#[tokio::test]
async fn test_list_tags_from_document() {
    if skip_without_db() {
        println!("Skipping: DATABASE_URL not set");
        return;
    }

    let pool = create_test_pool().await;
    setup_database(&pool).await;
    let _ = teardown_database(&pool).await;

    let user = create_test_user(&pool).await;

    let mut doc = create_test_document(&pool, &user.id.as_str()).await;
    let repo = DocumentRepository::new(pool.clone());

    let mut tags: Vec<String> = doc.parse_tags().expect("Failed to parse initial tags");
    tags.push("integration".to_string());
    tags.push("tag-test".to_string());
    tags.sort();
    tags.dedup();

    let tags_json = serde_json::to_string(&tags).expect("Failed to serialize tags");
    doc.tags = tags_json;
    doc.updated_at = chrono::Utc::now();

    repo.update(doc.clone()).await.expect("Failed to update document tags");

    let doc_id = tachyon_core::id::DocumentId::parse_str(&doc.id).unwrap();
    let fetched = repo.get_by_id(&doc_id).await.expect("Failed to refetch document");
    let fetched_tags: Vec<String> = fetched.parse_tags().expect("Failed to parse fetched tags");
    assert!(fetched_tags.contains(&"test".to_string()));
    assert!(fetched_tags.contains(&"integration".to_string()));
    assert!(fetched_tags.contains(&"tag-test".to_string()));

    teardown_database(&pool).await;
}

#[tokio::test]
async fn test_tag_creation_on_document_create() {
    if skip_without_db() {
        println!("Skipping: DATABASE_URL not set");
        return;
    }

    let pool = create_test_pool().await;
    setup_database(&pool).await;
    let _ = teardown_database(&pool).await;

    let user = create_test_user(&pool).await;
    let _doc = create_test_document(&pool, &user.id.as_str()).await;
    let repo = DocumentRepository::new(pool.clone());

    let results = repo
        .search_by_tags(&["test".to_string()], Some(10))
        .await
        .expect("Failed to search by tag");
    assert!(results.len() >= 1, "Should find at least one document with 'test' tag");

    let no_results = repo
        .search_by_tags(&["nonexistent_tag_xyz_123".to_string()], Some(10))
        .await
        .expect("Failed to search by nonexistent tag");
    assert!(no_results.is_empty());

    teardown_database(&pool).await;
}

#[tokio::test]
async fn test_multiple_tags_per_document() {
    if skip_without_db() {
        println!("Skipping: DATABASE_URL not set");
        return;
    }

    let pool = create_test_pool().await;
    setup_database(&pool).await;
    let _ = teardown_database(&pool).await;

    let user = create_test_user(&pool).await;
    let doc = create_test_document(&pool, &user.id.as_str()).await;
    let repo = DocumentRepository::new(pool.clone());

    let new_tags = vec!["rust".to_string(), "database".to_string(), "testing".to_string(), "integration".to_string()];
    let mut updated_doc = doc.clone();
    updated_doc.tags = serde_json::to_string(&new_tags).expect("Failed to serialize tags");
    updated_doc.updated_at = chrono::Utc::now();

    repo.update(updated_doc).await.expect("Failed to update document");

    let doc_id = tachyon_core::id::DocumentId::parse_str(&doc.id).unwrap();
    let fetched = repo.get_by_id(&doc_id).await.expect("Failed to refetch");
    let fetched_tags: Vec<String> = fetched.parse_tags().expect("Failed to parse tags");
    assert_eq!(fetched_tags.len(), 4);
    assert!(fetched_tags.contains(&"rust".to_string()));
    assert!(fetched_tags.contains(&"database".to_string()));

    teardown_database(&pool).await;
}
