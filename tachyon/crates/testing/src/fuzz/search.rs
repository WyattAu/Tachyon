//! Fuzzing tests for search functionality
//!
//! Property-based tests using random input to verify search types
//! don't panic and handle edge cases gracefully.

#[allow(unused_imports)]
use tachyon_core::{generate_document_id, generate_user_id};
#[allow(unused_imports)]
use tachyon_search::{
    SearchDocument, SearchRequest,
    types::{FieldDefinition, FieldType, QueryType, RangeValue, SortOrder, Suggestion, SuggestionCategory},
};

#[test]
fn test_search_request_random_query_no_panic() {
    let long_query = "very long query string that exceeds normal length".to_string() + &"word ".repeat(1000);
    let repeated_a = "a".repeat(10000);
    let inputs = vec![
        "",
        "a",
        "    ",
        "normal query",
        "query with special chars: @#$%^&*()",
        long_query.as_str(),
        "unicode: 你好世界 🚀",
        "null\x00bytes",
        "newlines\nand\ttabs",
        repeated_a.as_str(),
        "\"quoted\" 'single' `backtick`",
    ];

    for query in &inputs {
        let req = SearchRequest::new(*query);
        let _ = req.validate();
    }
}

#[test]
fn test_search_document_random_titles_no_panic() {
    let repeated_300 = "a".repeat(300);
    let repeated_200 = "a".repeat(200);
    let titles = vec![
        "",
        "a",
        "Valid Title",
        repeated_300.as_str(),
        "title with\nnewlines",
        "title with\ttabs",
        "🔥 unicode emoji title",
        "<script>alert(1)</script>",
        repeated_200.as_str(),
    ];

    for title in &titles {
        let doc = SearchDocument::new(
            generate_document_id(),
            title.to_string(),
            "content".to_string(),
            generate_user_id(),
        );
        let _ = doc.validate();
    }
}

#[test]
fn test_search_document_random_content_no_panic() {
    let repeated_100k = "a".repeat(100000);
    let contents = vec![
        "",
        "x",
        "normal content",
        repeated_100k.as_str(),
        "binary \x00\x01\x02 content",
        "unicode 你好 🚀",
        "markdown **bold** _italic_ `code`",
        "html <b>bold</b> <script>alert(1)</script>",
    ];

    for content in &contents {
        let doc = SearchDocument::new(
            generate_document_id(),
            "Title".to_string(),
            content.to_string(),
            generate_user_id(),
        );
        let _ = doc.validate();
    }
}

#[test]
fn test_field_definition_random_names_no_panic() {
    let repeated_500 = "a".repeat(500);
    let names = vec![
        "",
        "a",
        "field-name",
        "field_name",
        "field.name",
        "field with spaces",
        repeated_500.as_str(),
        "fieldName123",
        "UPPERCASE",
    ];

    for name in &names {
        let field = FieldDefinition::new(*name, FieldType::Text);
        assert_eq!(field.name, *name);
    }
}

#[test]
fn test_query_type_serialization_roundtrip_no_panic() {
    let queries = vec![
        QueryType::Term {
            field: "title".to_string(),
            value: "test".to_string(),
        },
        QueryType::Phrase {
            field: "content".to_string(),
            value: "hello world".to_string(),
            slop: 0,
        },
        QueryType::Phrase {
            field: "content".to_string(),
            value: "".to_string(),
            slop: 1000,
        },
        QueryType::Boolean {
            operator: tachyon_search::types::BooleanOperator::And,
            queries: vec![],
        },
        QueryType::Range {
            field: "date".to_string(),
            from: Some(RangeValue::DateTime("2024-01-01".to_string())),
            to: Some(RangeValue::DateTime("2024-12-31".to_string())),
        },
        QueryType::Range {
            field: "count".to_string(),
            from: Some(RangeValue::Integer(0)),
            to: Some(RangeValue::Integer(i64::MAX)),
        },
        QueryType::Fuzzy {
            field: "title".to_string(),
            value: "tset".to_string(),
            distance: 2,
            prefix_length: 0,
        },
    ];

    for query in &queries {
        let json = serde_json::to_string(query).unwrap();
        let de: QueryType = serde_json::from_str(&json).unwrap();
        let _ = serde_json::to_string(&de).unwrap();
    }
}

#[test]
fn test_search_request_serde_random_values_no_panic() {
    let requests = vec![
        SearchRequest::new("").with_page_size(0),
        SearchRequest::new("").with_page_size(101),
        SearchRequest::new("test").with_pagination(0, 10),
        SearchRequest::new("test").with_pagination(1, 0),
    ];

    for req in &requests {
        let json = serde_json::to_string(req).unwrap();
        let de: SearchRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(req.query, de.query);
    }
}

#[test]
fn test_suggestion_creation_no_panic() {
    let repeated_500 = "a".repeat(500);
    let texts = vec![
        "",
        "a",
        "normal suggestion",
        "🔥 emoji suggestion",
        repeated_500.as_str(),
        "suggestion with\nnewlines",
    ];

    for text in &texts {
        let _ = Suggestion::document(*text, "doc-1");
        let _ = Suggestion::tag(*text);
    }
}

#[test]
fn test_sort_order_all_variants() {
    let orders = vec![
        SortOrder::Score,
        SortOrder::DateDesc,
        SortOrder::DateAsc,
        SortOrder::TitleAsc,
        SortOrder::TitleDesc,
    ];

    for order in &orders {
        let json = serde_json::to_string(order).unwrap();
        let de: SortOrder = serde_json::from_str(&json).unwrap();
        assert_eq!(*order, de);
    }
}
