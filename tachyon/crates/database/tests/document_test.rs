use tachyon_database::{DocumentRepository, DocumentMetadata, init_with_migrations};
use chrono::Utc;
use serde_json::json;
use sqlx::PgPool;

async fn setup_test_db() -> PgPool {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://tachyon:tachyon@localhost:5432/tachyon_test".to_string());
    
    let pool = init_with_migrations(&database_url)
        .await
        .expect("Failed to setup test database");
    
    pool
}

async fn cleanup_test_data(pool: &PgPool) {
    sqlx::query("DELETE FROM documents WHERE title LIKE 'TEST_%'")
        .execute(pool)
        .await
        .ok();
}

fn create_test_document() -> DocumentMetadata {
    DocumentMetadata {
        id: uuid::Uuid::new_v4().to_string(),
        title: format!("TEST_Document_{}", Utc::now().timestamp()),
        slug: format!("test-doc-{}", Utc::now().timestamp()),
        author_id: uuid::Uuid::new_v4().to_string(),
        description: Some("Test document description".to_string()),
        tags: vec!["test".to_string(), "automated".to_string()],
        frontmatter: json!({"category": "test"}),
        project_id: None,
        visibility: "Private".to_string(),
        status: "Draft".to_string(),
        content_type: "Markdown".to_string(),
        word_count: 100,
        character_count: 500,
        read_count: 0,
        edit_count: 0,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        published_at: None,
    }
}

#[tokio::test]
async fn test_create_document() {
    let pool = setup_test_db().await;
    cleanup_test_data(&pool).await;
    
    let repo = DocumentRepository::new(pool.clone());
    let doc = create_test_document();
    
    let result = repo.create_document(&doc).await;
    assert!(result.is_ok(), "Failed to create document: {:?}", result.err());
    
    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_document_by_id() {
    let pool = setup_test_db().await;
    cleanup_test_data(&pool).await;
    
    let repo = DocumentRepository::new(pool.clone());
    let doc = create_test_document();
    
    repo.create_document(&doc).await.expect("Failed to create document");
    
    let retrieved = repo.get_document_by_id(&doc.id).await
        .expect("Failed to get document");
    
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.id, doc.id);
    assert_eq!(retrieved.title, doc.title);
    assert_eq!(retrieved.slug, doc.slug);
    
    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_document_by_slug() {
    let pool = setup_test_db().await;
    cleanup_test_data(&pool).await;
    
    let repo = DocumentRepository::new(pool.clone());
    let doc = create_test_document();
    
    repo.create_document(&doc).await.expect("Failed to create document");
    
    let retrieved = repo.get_document_by_slug(&doc.slug).await
        .expect("Failed to get document by slug");
    
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.slug, doc.slug);
    
    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_update_document() {
    let pool = setup_test_db().await;
    cleanup_test_data(&pool).await;
    
    let repo = DocumentRepository::new(pool.clone());
    let mut doc = create_test_document();
    
    repo.create_document(&doc).await.expect("Failed to create document");
    
    doc.title = format!("TEST_Updated_{}", Utc::now().timestamp());
    doc.word_count = 200;
    doc.updated_at = Utc::now();
    
    let result = repo.update_document(&doc).await;
    assert!(result.is_ok(), "Failed to update document: {:?}", result.err());
    
    let updated = repo.get_document_by_id(&doc.id).await
        .expect("Failed to get updated document")
        .expect("Document not found");
    
    assert_eq!(updated.title, doc.title);
    assert_eq!(updated.word_count, 200);
    
    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_delete_document() {
    let pool = setup_test_db().await;
    cleanup_test_data(&pool).await;
    
    let repo = DocumentRepository::new(pool.clone());
    let doc = create_test_document();
    
    repo.create_document(&doc).await.expect("Failed to create document");
    
    let result = repo.delete_document(&doc.id).await;
    assert!(result.is_ok(), "Failed to delete document: {:?}", result.err());
    
    let retrieved = repo.get_document_by_id(&doc.id).await
        .expect("Failed to query document");
    assert!(retrieved.is_none(), "Document should be deleted");
    
    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_list_documents() {
    let pool = setup_test_db().await;
    cleanup_test_data(&pool).await;
    
    let repo = DocumentRepository::new(pool.clone());
    
    for i in 0..5 {
        let mut doc = create_test_document();
        doc.title = format!("TEST_List_{}_{}", i, Utc::now().timestamp());
        doc.slug = format!("test-list-{}-{}", i, Utc::now().timestamp());
        repo.create_document(&doc).await.expect("Failed to create document");
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }
    
    let docs = repo.list_documents(None, Some(10), Some(0)).await
        .expect("Failed to list documents");
    
    assert!(docs.len() >= 5, "Should have at least 5 test documents");
    
    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_search_documents_by_tags() {
    let pool = setup_test_db().await;
    cleanup_test_data(&pool).await;
    
    let repo = DocumentRepository::new(pool.clone());
    let mut doc = create_test_document();
    doc.tags = vec!["unique-test-tag".to_string()];
    
    repo.create_document(&doc).await.expect("Failed to create document");
    
    let results = repo.search_by_tags(&["unique-test-tag"]).await
        .expect("Failed to search by tags");
    
    assert!(!results.is_empty(), "Should find document by tag");
    
    cleanup_test_data(&pool).await;
}
