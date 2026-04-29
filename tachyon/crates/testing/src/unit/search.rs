//! Unit tests for search types and operations
//!
//! Tests for search document creation, search request validation,
//! search response construction, field definitions, and index configuration.

use tachyon_core::{generate_document_id, generate_user_id};
#[allow(unused_imports)]
use tachyon_search::{
    types::{BooleanOperator, QueryType, Suggestion, SuggestionCategory},
    BatchIndexRequest, FieldDefinition, FieldType, IndexConfig, SearchDocument, SearchRequest,
    SearchResponse, SortOrder,
};

#[allow(dead_code)]
fn make_search_doc(title: &str, content: &str) -> SearchDocument {
    SearchDocument::new(
        generate_document_id(),
        title.to_string(),
        content.to_string(),
        generate_user_id(),
    )
}

#[test]
fn test_search_document_creation() {
    let doc = make_search_doc("Test Title", "Some content here");
    assert_eq!(doc.title, "Test Title");
    assert_eq!(doc.content, "Some content here");
    assert!(doc.tags.is_empty());
    assert!(doc.repository_id.is_none());
}

#[test]
fn test_search_document_with_tags() {
    let doc =
        make_search_doc("Title", "Content").with_tags(vec!["rust".to_string(), "web".to_string()]);
    assert_eq!(doc.tags.len(), 2);
    assert_eq!(doc.tags[0], "rust");
}

#[test]
fn test_search_document_with_repository() {
    let doc = make_search_doc("Title", "Content")
        .with_repository_id(tachyon_core::generate_repository_id());
    assert!(doc.repository_id.is_some());
}

#[test]
fn test_search_document_with_custom_field() {
    let doc = make_search_doc("Title", "Content")
        .with_custom_field("priority", serde_json::json!("high"));
    assert_eq!(doc.custom_fields.get("priority").unwrap(), "high");
}

#[test]
fn test_search_document_validation_ok() {
    let doc = make_search_doc("Valid Title", "Valid content");
    assert!(doc.validate().is_ok());
}

#[test]
fn test_search_document_validation_empty_title() {
    let doc = make_search_doc("", "Content");
    assert!(doc.validate().is_err());
}

#[test]
fn test_search_document_validation_empty_content() {
    let doc = make_search_doc("Title", "");
    assert!(doc.validate().is_err());
}

#[test]
fn test_search_document_validation_title_too_long() {
    let doc = make_search_doc(&"a".repeat(201), "content");
    assert!(doc.validate().is_err());
}

#[test]
fn test_search_request_basic() {
    let req = SearchRequest::new("test query");
    assert_eq!(req.query, "test query");
    assert_eq!(req.page, 1);
    assert_eq!(req.page_size, 20);
    assert_eq!(req.sort, SortOrder::Score);
    assert!(req.highlight);
}

#[test]
fn test_search_request_with_filters() {
    let req = SearchRequest::new("test")
        .with_filter("status", serde_json::json!("published"))
        .with_sort(SortOrder::DateDesc)
        .with_page_size(10)
        .with_pagination(2, 10);

    assert_eq!(req.filters.get("status").unwrap(), "published");
    assert_eq!(req.sort, SortOrder::DateDesc);
    assert_eq!(req.page_size, 10);
    assert_eq!(req.page, 2);
    assert_eq!(req.offset(), 10);
}

#[test]
fn test_search_request_with_tags() {
    let req = SearchRequest::new("test").with_tags(vec!["rust".to_string()]);
    assert_eq!(req.tags.as_deref(), Some(&["rust".to_string()][..]));
}

#[test]
fn test_search_request_with_date_range() {
    let from = chrono::Utc::now() - chrono::Duration::days(7);
    let to = chrono::Utc::now();
    let req = SearchRequest::new("test").with_date_range(Some(from), Some(to));
    assert!(req.date_from.is_some());
    assert!(req.date_to.is_some());
}

#[test]
fn test_search_request_validation_ok() {
    let req = SearchRequest::new("query");
    assert!(req.validate().is_ok());
}

#[test]
fn test_search_request_validation_empty() {
    let req = SearchRequest::new("");
    assert!(req.validate().is_err());
}

#[test]
fn test_search_request_validation_page_size_zero() {
    let req = SearchRequest::new("test").with_page_size(0);
    assert!(req.validate().is_err());
}

#[test]
fn test_search_request_validation_page_size_too_large() {
    let req = SearchRequest::new("test").with_page_size(101);
    assert!(req.validate().is_err());
}

#[test]
fn test_search_request_offset() {
    let req = SearchRequest::new("test").with_pagination(3, 25);
    assert_eq!(req.offset(), 50);
}

#[test]
fn test_search_request_offset_page_zero() {
    let req = SearchRequest::new("test").with_pagination(0, 25);
    assert_eq!(req.offset(), 0);
}

#[test]
fn test_search_response_new() {
    let req = SearchRequest::new("test").with_page_size(10);
    let resp = SearchResponse::new(vec![], 25, &req, 50);
    assert_eq!(resp.total_hits, 25);
    assert_eq!(resp.total_pages, 3);
    assert_eq!(resp.page, 1);
    assert_eq!(resp.query_time_ms, 50);
}

#[test]
fn test_search_response_empty() {
    let req = SearchRequest::new("test").with_page_size(10);
    let resp = SearchResponse::empty(&req, 10);
    assert_eq!(resp.total_hits, 0);
    assert_eq!(resp.total_pages, 0);
    assert!(resp.results.is_empty());
}

#[test]
fn test_field_definition_builder() {
    let field = FieldDefinition::new("title", FieldType::Text)
        .with_boost(2.0)
        .with_required(true)
        .with_indexed(true)
        .with_stored(true);

    assert_eq!(field.name, "title");
    assert_eq!(field.field_type, FieldType::Text);
    assert_eq!(field.boost, 2.0);
    assert!(field.required);
}

#[test]
fn test_index_config_builder() {
    let config = IndexConfig::new("test-index")
        .with_num_shards(2)
        .with_index_path("/tmp/index")
        .add_field(FieldDefinition::new("title", FieldType::Text))
        .add_field(FieldDefinition::new("content", FieldType::Text));

    assert_eq!(config.name, "test-index");
    assert_eq!(config.num_shards, 2);
    assert_eq!(config.index_path.as_deref(), Some("/tmp/index"));
    assert_eq!(config.fields.len(), 2);
}

#[test]
fn test_suggestion_document() {
    let s = Suggestion::document("Rust Programming", "doc-123");
    assert_eq!(s.text, "Rust Programming");
    assert_eq!(s.document_id.as_deref(), Some("doc-123"));
    assert_eq!(s.category, SuggestionCategory::Document);
}

#[test]
fn test_suggestion_tag() {
    let s = Suggestion::tag("rust");
    assert_eq!(s.text, "rust");
    assert!(s.document_id.is_none());
    assert_eq!(s.category, SuggestionCategory::Tag);
}

#[test]
fn test_batch_index_request() {
    let docs = vec![
        make_search_doc("A", "Content A"),
        make_search_doc("B", "Content B"),
    ];
    let req = BatchIndexRequest::new(docs).with_clear_before_index(true);
    assert_eq!(req.documents.len(), 2);
    assert!(req.clear_before_index);
}

#[test]
fn test_search_document_serde() {
    let doc = make_search_doc("Title", "Content");
    let json = serde_json::to_string(&doc).expect("serialize");
    let de: SearchDocument = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(doc.title, de.title);
    assert_eq!(doc.content, de.content);
}

#[test]
fn test_search_request_serde() {
    let req = SearchRequest::new("test").with_page_size(10);
    let json = serde_json::to_string(&req).expect("serialize");
    let de: SearchRequest = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(req.query, de.query);
    assert_eq!(req.page_size, de.page_size);
}

#[test]
fn test_query_type_variants() {
    let term = QueryType::Term {
        field: "title".to_string(),
        value: "test".to_string(),
    };
    let json = serde_json::to_string(&term).expect("serialize");
    let de: QueryType = serde_json::from_str(&json).expect("deserialize");
    match de {
        QueryType::Term { field, value } => {
            assert_eq!(field, "title");
            assert_eq!(value, "test");
        }
        _ => panic!("Expected Term variant"),
    }
}
