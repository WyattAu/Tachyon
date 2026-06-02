use super::common::skip_without_db;
use tachyon_core::id::{DocumentId, UserId};
use tachyon_search::{IndexManager, QueryEngine, SearchDocument, SearchRequest, SortOrder};

fn skip_search_tests() -> bool {
    // Search tests require tantivy index infrastructure.
    std::env::var("RUN_SEARCH_TESTS").is_err()
}

async fn create_index_manager() -> IndexManager {
    let index_dir =
        std::env::temp_dir().join(format!("tachyon_search_test_{}", uuid::Uuid::new_v4()));
    IndexManager::new(index_dir)
        .await
        .expect("Failed to create index manager")
}

#[tokio::test]
async fn test_index_document() {
    if skip_without_db() || skip_search_tests() {
        println!("Skipping: DATABASE_URL or RUN_SEARCH_TESTS not set");
        return;
    }

    let manager = create_index_manager().await;
    let doc = SearchDocument::new(
        DocumentId::new(),
        "Test Document".to_string(),
        "This is test content for indexing.".to_string(),
        UserId::new(),
    )
    .with_tags(vec!["test".to_string(), "search".to_string()]);

    assert!(doc.validate().is_ok());

    let result = manager.index_document(&doc).await;
    assert!(
        result.is_ok(),
        "Indexing document should succeed: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_index_and_search_document() {
    if skip_without_db() || skip_search_tests() {
        println!("Skipping: DATABASE_URL or RUN_SEARCH_TESTS not set");
        return;
    }

    let manager = create_index_manager().await;

    let doc = SearchDocument::new(
        DocumentId::new(),
        "Rust Programming Guide".to_string(),
        "Rust is a systems programming language focused on safety and performance.".to_string(),
        UserId::new(),
    )
    .with_tags(vec!["rust".to_string(), "programming".to_string()]);

    manager
        .index_document(&doc)
        .await
        .expect("Failed to index document");

    let query_engine = QueryEngine::new(manager);
    let request = SearchRequest::new("Rust programming")
        .with_page_size(10)
        .with_sort(SortOrder::Score);

    let response = query_engine.search(&request).await.expect("Search failed");
    assert!(
        response.total_hits >= 1,
        "Should find at least one result for 'Rust programming'"
    );
}

#[tokio::test]
async fn test_search_no_results() {
    if skip_without_db() || skip_search_tests() {
        println!("Skipping: DATABASE_URL or RUN_SEARCH_TESTS not set");
        return;
    }

    let manager = create_index_manager().await;
    let query_engine = QueryEngine::new(manager);

    let request = SearchRequest::new("nonexistent_unique_query_xyz123");
    let response = query_engine.search(&request).await.expect("Search failed");
    assert_eq!(response.total_hits, 0);
}

#[tokio::test]
async fn test_search_suggestions() {
    if skip_without_db() || skip_search_tests() {
        println!("Skipping: DATABASE_URL or RUN_SEARCH_TESTS not set");
        return;
    }

    let manager = create_index_manager().await;

    let doc = SearchDocument::new(
        DocumentId::new(),
        "Tachyon Knowledge Base".to_string(),
        "A knowledge management system for teams.".to_string(),
        UserId::new(),
    )
    .with_tags(vec!["knowledge".to_string()]);

    manager.index_document(&doc).await.expect("Failed to index");

    let query_engine = QueryEngine::new(manager);
    let suggestions = query_engine
        .suggest("tach", 5)
        .await
        .expect("Suggestions failed");
    assert!(
        !suggestions.is_empty(),
        "Should return suggestions for 'tach'"
    );
}

#[tokio::test]
async fn test_delete_document_from_index() {
    if skip_without_db() || skip_search_tests() {
        println!("Skipping: DATABASE_URL or RUN_SEARCH_TESTS not set");
        return;
    }

    let manager = create_index_manager().await;
    let doc_id = DocumentId::new();

    let doc = SearchDocument::new(
        doc_id,
        "Document To Delete".to_string(),
        "Content that will be deleted.".to_string(),
        UserId::new(),
    );

    manager.index_document(&doc).await.expect("Failed to index");

    let query_engine = QueryEngine::new(manager.clone());
    let request = SearchRequest::new("Document To Delete");
    let before = query_engine
        .search(&request)
        .await
        .expect("Search before delete failed");
    assert!(before.total_hits >= 1);

    manager
        .delete_document(&doc_id.as_str())
        .await
        .expect("Failed to delete from index");

    let query_engine2 = QueryEngine::new(manager);
    let after = query_engine2
        .search(&request)
        .await
        .expect("Search after delete failed");
    assert_eq!(
        after.total_hits, 0,
        "Document should not be found after deletion"
    );
}

#[tokio::test]
async fn test_batch_index_documents() {
    if skip_without_db() || skip_search_tests() {
        println!("Skipping: DATABASE_URL or RUN_SEARCH_TESTS not set");
        return;
    }

    let manager = create_index_manager().await;

    let docs: Vec<SearchDocument> = (0..5)
        .map(|i| {
            SearchDocument::new(
                DocumentId::new(),
                format!("Batch Document {}", i),
                format!("Content of batch document number {}.", i),
                UserId::new(),
            )
        })
        .collect();

    let count = manager
        .batch_index(&docs)
        .await
        .expect("Batch index failed");
    assert_eq!(count, 5);
}

#[tokio::test]
async fn test_search_with_pagination() {
    if skip_without_db() || skip_search_tests() {
        println!("Skipping: DATABASE_URL or RUN_SEARCH_TESTS not set");
        return;
    }

    let manager = create_index_manager().await;

    for i in 0..10 {
        let doc = SearchDocument::new(
            DocumentId::new(),
            format!("Pagination Test Doc {}", i),
            format!("Content for pagination test document number {}.", i),
            UserId::new(),
        );
        manager.index_document(&doc).await.expect("Failed to index");
    }

    let query_engine = QueryEngine::new(manager);
    let page1 = SearchRequest::new("Pagination Test Doc").with_pagination(1, 3);
    let response1 = query_engine
        .search(&page1)
        .await
        .expect("Search page 1 failed");
    assert_eq!(response1.results.len(), 3);

    let page2 = SearchRequest::new("Pagination Test Doc").with_pagination(2, 3);
    let response2 = query_engine
        .search(&page2)
        .await
        .expect("Search page 2 failed");
    assert_eq!(response2.results.len(), 3);
}
