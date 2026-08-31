use tachyon_database::DocumentRepository;

use super::common::skip_without_db;
use crate::common::setup::{
    create_test_document, create_test_pool, create_test_user, setup_database, teardown_test_user,
};

#[tokio::test]
async fn test_create_document() {
    if skip_without_db() {
        println!("Skipping: DATABASE_URL not set");
        return;
    }

    let pool = create_test_pool().await;
    setup_database(&pool).await;
    let user = create_test_user(&pool).await;
    let doc = create_test_document(&pool, &user.id.as_str()).await;

    assert!(!doc.id.is_empty());
    assert!(!doc.title.is_empty());
    assert!(doc.slug.is_some());
    assert_eq!(doc.author_id, user.id.as_str());
    assert_eq!(doc.status, "draft");
    assert_eq!(doc.visibility, "private");

    teardown_test_user(&pool, &user.username).await;
}

#[tokio::test]
async fn test_get_document_by_id() {
    if skip_without_db() {
        println!("Skipping: DATABASE_URL not set");
        return;
    }

    let pool = create_test_pool().await;
    setup_database(&pool).await;
    let user = create_test_user(&pool).await;
    let doc = create_test_document(&pool, &user.id.as_str()).await;
    let repo = DocumentRepository::new(pool.clone());

    let doc_id = tachyon_core::id::DocumentId::parse_str(&doc.id).unwrap();
    let fetched = repo
        .get_by_id(&doc_id)
        .await
        .expect("Failed to get document by ID");
    assert_eq!(fetched.title, doc.title);
    assert_eq!(fetched.author_id, doc.author_id);

    teardown_test_user(&pool, &user.username).await;
}

#[tokio::test]
async fn test_update_document_title_and_content() {
    if skip_without_db() {
        println!("Skipping: DATABASE_URL not set");
        return;
    }

    let pool = create_test_pool().await;
    setup_database(&pool).await;
    let user = create_test_user(&pool).await;
    let mut doc = create_test_document(&pool, &user.id.as_str()).await;
    let repo = DocumentRepository::new(pool.clone());

    doc.title = "Updated Title".to_string();
    doc.content = Some("# Updated\n\nNew content here.".to_string());
    doc.status = "published".to_string();
    doc.updated_at = chrono::Utc::now();

    repo.update(doc.clone())
        .await
        .expect("Failed to update document");

    let doc_id = tachyon_core::id::DocumentId::parse_str(&doc.id).unwrap();
    let fetched = repo
        .get_by_id(&doc_id)
        .await
        .expect("Failed to refetch document");
    assert_eq!(fetched.title, "Updated Title");
    assert_eq!(fetched.status, "published");

    teardown_test_user(&pool, &user.username).await;
}

#[tokio::test]
async fn test_update_document_metadata() {
    if skip_without_db() {
        println!("Skipping: DATABASE_URL not set");
        return;
    }

    let pool = create_test_pool().await;
    setup_database(&pool).await;
    let user = create_test_user(&pool).await;
    let mut doc = create_test_document(&pool, &user.id.as_str()).await;
    let repo = DocumentRepository::new(pool.clone());

    doc.description = Some("New description".to_string());
    doc.visibility = "public".to_string();
    doc.word_count = 100;
    doc.character_count = 500;
    doc.updated_at = chrono::Utc::now();

    repo.update(doc.clone())
        .await
        .expect("Failed to update document metadata");

    let doc_id = tachyon_core::id::DocumentId::parse_str(&doc.id).unwrap();
    let fetched = repo
        .get_by_id(&doc_id)
        .await
        .expect("Failed to refetch document");
    assert_eq!(fetched.description.as_deref(), Some("New description"));
    assert_eq!(fetched.visibility, "public");
    assert_eq!(fetched.word_count, 100);

    teardown_test_user(&pool, &user.username).await;
}

#[tokio::test]
async fn test_list_documents() {
    if skip_without_db() {
        println!("Skipping: DATABASE_URL not set");
        return;
    }

    let pool = create_test_pool().await;
    setup_database(&pool).await;
    let user = create_test_user(&pool).await;
    create_test_document(&pool, &user.id.as_str()).await;
    create_test_document(&pool, &user.id.as_str()).await;

    let repo = DocumentRepository::new(pool.clone());
    let all = repo
        .list_all(Some(10), None)
        .await
        .expect("Failed to list documents");
    assert!(all.len() >= 2);

    let paged = repo
        .list_all(Some(1), None)
        .await
        .expect("Failed to list documents page 1");
    assert_eq!(paged.len(), 1);

    teardown_test_user(&pool, &user.username).await;
}

#[tokio::test]
async fn test_list_documents_with_pagination() {
    if skip_without_db() {
        println!("Skipping: DATABASE_URL not set");
        return;
    }

    let pool = create_test_pool().await;
    setup_database(&pool).await;
    let user = create_test_user(&pool).await;
    for _ in 0..5 {
        create_test_document(&pool, &user.id.as_str()).await;
    }

    let repo = DocumentRepository::new(pool.clone());

    let page1 = repo
        .list_all(Some(2), Some(0))
        .await
        .expect("Failed to list page 1");
    assert_eq!(page1.len(), 2);

    let page2 = repo
        .list_all(Some(2), Some(2))
        .await
        .expect("Failed to list page 2");
    assert_eq!(page2.len(), 2);

    let page3 = repo
        .list_all(Some(2), Some(4))
        .await
        .expect("Failed to list page 3");
    assert!(!page3.is_empty());

    teardown_test_user(&pool, &user.username).await;
}

#[tokio::test]
async fn test_list_documents_by_author() {
    if skip_without_db() {
        println!("Skipping: DATABASE_URL not set");
        return;
    }

    let pool = create_test_pool().await;
    setup_database(&pool).await;
    let user = create_test_user(&pool).await;
    create_test_document(&pool, &user.id.as_str()).await;
    create_test_document(&pool, &user.id.as_str()).await;

    let repo = DocumentRepository::new(pool.clone());
    let docs = repo
        .list_by_author(&user.id.as_str(), Some(10), None)
        .await
        .expect("Failed to list documents by author");

    assert!(docs.len() >= 2);
    for doc in &docs {
        assert_eq!(doc.author_id, user.id.as_str());
    }

    teardown_test_user(&pool, &user.username).await;
}

#[tokio::test]
async fn test_delete_document() {
    if skip_without_db() {
        println!("Skipping: DATABASE_URL not set");
        return;
    }

    let pool = create_test_pool().await;
    setup_database(&pool).await;
    let user = create_test_user(&pool).await;
    let doc = create_test_document(&pool, &user.id.as_str()).await;
    let repo = DocumentRepository::new(pool.clone());

    let doc_id = tachyon_core::id::DocumentId::parse_str(&doc.id).unwrap();
    repo.delete(&doc_id)
        .await
        .expect("Failed to delete document");

    let result = repo.get_by_id(&doc_id).await;
    assert!(result.is_err(), "Deleted document should not be found");

    teardown_test_user(&pool, &user.username).await;
}

#[tokio::test]
async fn test_document_tags() {
    if skip_without_db() {
        println!("Skipping: DATABASE_URL not set");
        return;
    }

    let pool = create_test_pool().await;
    setup_database(&pool).await;
    let user = create_test_user(&pool).await;
    let doc = create_test_document(&pool, &user.id.as_str()).await;

    let tags: Vec<String> = doc.parse_tags().expect("Failed to parse tags");
    assert!(tags.contains(&"test".to_string()));

    teardown_test_user(&pool, &user.username).await;
}
