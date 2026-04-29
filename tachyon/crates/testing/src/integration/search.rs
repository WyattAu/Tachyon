//! Integration tests for search operations
//!
//! Tests search document indexing and query creation using the search crate types.
//! These tests exercise the type system without requiring a running Tantivy index.

#[allow(unused_imports)]
use tachyon_core::{generate_document_id, generate_user_id};
#[allow(unused_imports)]
use tachyon_search::{
    types::BooleanOperator, types::QueryType, FieldDefinition, FieldType, IndexConfig,
    SearchDocument, SearchRequest, SearchResponse, SortOrder,
};

#[test]
fn test_full_search_document_workflow() {
    let doc = SearchDocument::new(
        generate_document_id(),
        "Integration Test Doc".to_string(),
        "This is the content for integration testing".to_string(),
        generate_user_id(),
    )
    .with_tags(vec!["integration".to_string(), "test".to_string()])
    .with_repository_id(tachyon_core::generate_repository_id())
    .with_custom_field("priority", serde_json::json!("high"));

    assert!(doc.validate().is_ok());
    assert_eq!(doc.tags.len(), 2);
    assert!(doc.repository_id.is_some());
    assert_eq!(doc.custom_fields.get("priority").unwrap(), "high");
}

#[test]
fn test_search_request_response_workflow() {
    let request = SearchRequest::new("integration test")
        .with_tags(vec!["test".to_string()])
        .with_sort(SortOrder::DateDesc)
        .with_page_size(10);

    assert!(request.validate().is_ok());
    assert_eq!(request.offset(), 0);

    let response = SearchResponse::new(vec![], 0, &request, 42);
    assert_eq!(response.total_pages, 0);
    assert_eq!(response.page, 1);
    assert_eq!(response.page_size, 10);
    assert_eq!(response.query_time_ms, 42);
}

#[test]
fn test_structured_query_construction() {
    let _term_query = QueryType::Term {
        field: "title".to_string(),
        value: "rust".to_string(),
    };

    let bool_query = QueryType::Boolean {
        operator: BooleanOperator::And,
        queries: vec![
            QueryType::Term {
                field: "title".to_string(),
                value: "rust".to_string(),
            },
            QueryType::Term {
                field: "content".to_string(),
                value: "programming".to_string(),
            },
        ],
    };

    let request = SearchRequest::new("").with_structured_query(bool_query);

    assert!(request.validate().is_ok());
    assert!(request.structured_query.is_some());
}

#[test]
fn test_index_config_with_schema() {
    let config = IndexConfig::new("documents")
        .with_num_shards(1)
        .add_field(
            FieldDefinition::new("id", FieldType::String)
                .with_stored(true)
                .with_indexed(true),
        )
        .add_field(
            FieldDefinition::new("title", FieldType::Text)
                .with_stored(true)
                .with_indexed(true)
                .with_boost(2.0),
        )
        .add_field(
            FieldDefinition::new("content", FieldType::Text)
                .with_stored(true)
                .with_indexed(true)
                .with_boost(1.0),
        )
        .add_field(
            FieldDefinition::new("tags", FieldType::Text)
                .with_stored(true)
                .with_indexed(true)
                .with_boost(1.5),
        )
        .add_field(
            FieldDefinition::new("created_at", FieldType::DateTime)
                .with_stored(true)
                .with_indexed(true),
        );

    assert_eq!(config.fields.len(), 5);
    assert_eq!(config.fields[1].boost, 2.0);
    assert_eq!(config.fields[3].boost, 1.5);
}

#[test]
fn test_search_document_serde_workflow() {
    let doc = SearchDocument::new(
        generate_document_id(),
        "Serde Test".to_string(),
        "Content for serde".to_string(),
        generate_user_id(),
    )
    .with_tags(vec!["serde".to_string()]);

    let json = serde_json::to_string(&doc).unwrap();
    let de: SearchDocument = serde_json::from_str(&json).unwrap();
    assert_eq!(doc.title, de.title);
    assert_eq!(doc.content, de.content);
    assert_eq!(doc.tags, de.tags);
}

#[test]
fn test_batch_index_multiple_documents() {
    let docs: Vec<SearchDocument> = (0..10)
        .map(|i| {
            SearchDocument::new(
                generate_document_id(),
                format!("Document {}", i),
                format!("Content for document {}", i),
                generate_user_id(),
            )
        })
        .collect();

    assert!(docs.iter().all(|d| d.validate().is_ok()));
    assert_eq!(docs.len(), 10);
}

#[test]
fn test_search_request_with_repository_filter() {
    let repo_id = tachyon_core::generate_repository_id();
    let author_id = generate_user_id();

    let request = SearchRequest::new("test")
        .with_repository_id(repo_id)
        .with_author_id(author_id);

    assert!(request.repository_id.is_some());
    assert!(request.author_id.is_some());
}

#[test]
fn test_search_request_pagination_boundary() {
    let req = SearchRequest::new("test").with_page_size(100);
    assert!(req.validate().is_ok());

    let req = SearchRequest::new("test").with_page_size(101);
    assert!(req.validate().is_err());
}
