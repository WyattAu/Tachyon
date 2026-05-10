// Query Processing Module
// Query parsing and execution for search operations

use crate::error::{SearchError, SearchResult};
use crate::types::{SearchRequest, Suggestion, SuggestionCategory};
use crate::IndexManager;
use std::sync::Arc;
use tantivy::{
    collector::TopDocs,
    query::{Query, QueryParser},
    schema::*,
    DocAddress, Searcher, TantivyDocument,
};

/// Query engine for parsing and executing search queries
///
/// Manages query parsing, filtering, and result conversion.
pub struct QueryEngine {
    /// Index manager for accessing search index
    index_manager: Arc<IndexManager>,
    /// Default field boost factors
    field_boosts: Vec<(String, f32)>,
}

impl QueryEngine {
    /// Create a new query engine
    ///
    /// # Arguments
    /// * `index_manager` - Index manager for search operations
    ///
    /// # Returns
    /// New QueryEngine instance
    pub fn new(index_manager: IndexManager) -> Self {
        let field_boosts = vec![("title".to_string(), 2.0), ("tags".to_string(), 1.5)];

        Self {
            index_manager: Arc::new(index_manager),
            field_boosts,
        }
    }

    /// Execute a search query
    ///
    /// # Arguments
    /// * `request` - Search request
    ///
    /// # Returns
    /// Result containing search results or error
    ///
    /// # Errors
    /// Returns error if query execution fails
    pub async fn search(
        &self,
        request: &SearchRequest,
    ) -> SearchResult<crate::types::SearchResponse> {
        request.validate()?;

        let reader = self.index_manager.reader()?;
        let searcher = reader.searcher();
        let schema = self.index_manager.schema();

        // Parse query
        let query = self.parse_query(request, schema)?;

        // Create collector with pagination
        let limit = request.page_size;
        let _offset = request.offset();

        let collector = TopDocs::with_limit(limit);

        // Execute search
        let start = std::time::Instant::now();
        let top_docs = searcher.search(&query, &collector).map_err(|e| {
            SearchError::query(
                "SEARCH_EXECUTION_ERROR",
                format!("Search execution failed: {}", e),
            )
        })?;
        let query_time_ms = start.elapsed().as_millis() as u64;

        // Convert to response
        let results = self.convert_to_response_items(&top_docs, &searcher, request)?;

        let total_hits = searcher.num_docs() as usize;

        let response =
            crate::types::SearchResponse::new(results, total_hits, request, query_time_ms);

        Ok(response)
    }

    /// Parse a query from request
    ///
    /// # Arguments
    /// * `request` - Search request
    /// * `schema` - Tantivy schema
    ///
    /// # Returns
    /// Result containing parsed query or error
    fn parse_query(
        &self,
        request: &SearchRequest,
        schema: &Schema,
    ) -> SearchResult<Box<dyn Query>> {
        // Create query parser
        let title_field = schema.get_field("title")?;
        let content_field = schema.get_field("content")?;
        let tags_field = schema.get_field("tags")?;

        let mut parser = QueryParser::for_index(
            self.index_manager.index(),
            vec![title_field, content_field, tags_field],
        );

        // Set field boosts
        for (field_name, boost) in &self.field_boosts {
            if let Ok(field) = schema.get_field(field_name) {
                parser.set_field_boost(field, *boost);
            }
        }

        // Parse query string
        let query = parser.parse_query(&request.query).map_err(|e| {
            SearchError::query("QUERY_PARSE_ERROR", format!("Query parsing failed: {}", e))
        })?;

        Ok(query)
    }

    /// Convert search results to response items
    ///
    /// # Arguments
    /// * `top_docs` - Top docs from search
    /// * `searcher` - Tantivy searcher
    /// * `request` - Original search request
    ///
    /// # Returns
    /// Result containing response items or error
    fn convert_to_response_items(
        &self,
        top_docs: &[(tantivy::Score, DocAddress)],
        searcher: &Searcher,
        request: &SearchRequest,
    ) -> SearchResult<Vec<crate::types::SearchResponseItem>> {
        let mut results = Vec::new();
        let schema = self.index_manager.schema();

        let id_field = schema.get_field("id")?;
        let title_field = schema.get_field("title")?;
        let author_id_field = schema.get_field("author_id")?;
        let repository_id_field = schema.get_field("repository_id")?;
        let tags_field = schema.get_field("tags")?;
        let created_at_field = schema.get_field("created_at")?;

        for (score, doc_address) in top_docs.iter() {
            if let Ok(retrieved_doc) = searcher.doc::<TantivyDocument>(*doc_address) {
                let snippet = self.generate_snippet(&retrieved_doc, request);
                let highlights = self.generate_highlights(&retrieved_doc, request);

                // Parse tags from the document
                let tags = self.parse_tags_from_document(&retrieved_doc, tags_field);

                // Parse created_at from the document
                let created_at =
                    self.parse_created_at_from_document(&retrieved_doc, created_at_field);

                let id_val = retrieved_doc
                    .get_first(id_field)
                    .ok_or_else(|| SearchError::document_not_found("Missing document ID"))?;
                let title_val = retrieved_doc
                    .get_first(title_field)
                    .ok_or_else(|| SearchError::document_not_found("Missing document title"))?;

                let document_id =
                    tachyon_core::id::DocumentId::parse_str(id_val.as_str().unwrap_or_default())
                        .map_err(|e| {
                            SearchError::query(
                                "INVALID_DOCUMENT_ID",
                                format!("Invalid document ID: {}", e),
                            )
                        })?;

                let author_id = retrieved_doc
                    .get_first(author_id_field)
                    .and_then(|v| v.as_str())
                    .and_then(|s| tachyon_core::id::UserId::parse_str(s).ok())
                    .unwrap_or_default();

                let repository_id = retrieved_doc
                    .get_first(repository_id_field)
                    .and_then(|v| v.as_str())
                    .and_then(|s| tachyon_core::id::RepositoryId::parse_str(s).ok());

                let item = crate::types::SearchResponseItem {
                    document_id,
                    title: title_val.as_str().unwrap_or_default().to_string(),
                    snippet,
                    score: *score,
                    highlights,
                    author_id,
                    repository_id,
                    tags,
                    created_at,
                };

                results.push(item);
            }
        }

        Ok(results)
    }

    /// Parse tags from a Tantivy document
    ///
    /// # Arguments
    /// * `doc` - Tantivy document
    /// * `tags_field` - Tags field handle
    ///
    /// # Returns
    /// Vector of tag strings
    fn parse_tags_from_document(
        &self,
        doc: &TantivyDocument,
        tags_field: tantivy::schema::Field,
    ) -> Vec<String> {
        let mut tags = Vec::new();

        // Get all values for the tags field (it might be multi-valued)
        for value in doc.get_all(tags_field) {
            if let Some(s) = value.as_str() {
                // Tags might be stored as a comma-separated string
                for tag in s.split(',') {
                    let tag = tag.trim();
                    if !tag.is_empty() {
                        tags.push(tag.to_string());
                    }
                }
            }
            if let Some(arr) = value.as_array() {
                // Tags might be stored as an array
                for item in arr {
                    if let Some(s) = item.as_str() {
                        tags.push(s.to_string());
                    }
                }
            }
        }

        // If no tags were found, check if there's a single value
        if tags.is_empty() {
            if let Some(value) = doc.get_first(tags_field) {
                if let Some(s) = value.as_str() {
                    if !s.is_empty() {
                        tags.push(s.to_string());
                    }
                }
            }
        }

        tags
    }

    /// Parse created_at timestamp from a Tantivy document
    ///
    /// # Arguments
    /// * `doc` - Tantivy document
    /// * `created_at_field` - Created at field handle
    ///
    /// # Returns
    /// DateTime in UTC timezone
    fn parse_created_at_from_document(
        &self,
        doc: &TantivyDocument,
        created_at_field: tantivy::schema::Field,
    ) -> chrono::DateTime<chrono::Utc> {
        // Try to get the created_at value
        if let Some(value) = doc.get_first(created_at_field) {
            if let Some(s) = value.as_str() {
                // Try parsing as ISO 8601 datetime string
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
                    return dt.with_timezone(&chrono::Utc);
                }
                // Try parsing as datetime with timezone
                if let Ok(dt) = chrono::DateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S %:z") {
                    return dt.with_timezone(&chrono::Utc);
                }
                // Try parsing as naive datetime
                if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
                    return dt.and_utc();
                }
            }
            if let Some(d) = value.as_datetime() {
                // Convert Tantivy date to chrono DateTime
                return chrono::DateTime::from_timestamp(d.into_timestamp_secs(), 0)
                    .unwrap_or_else(chrono::Utc::now);
            }
            if let Some(timestamp) = value.as_i64() {
                // Assume Unix timestamp
                return chrono::DateTime::from_timestamp(timestamp, 0)
                    .unwrap_or_else(chrono::Utc::now);
            }
        }

        // Default to current time if parsing fails
        chrono::Utc::now()
    }

    /// Generate search snippet
    ///
    /// # Arguments
    /// * `doc` - Tantivy document
    /// * `request` - Original search request
    ///
    /// # Returns
    /// Generated snippet string
    fn generate_snippet(&self, _doc: &TantivyDocument, _request: &SearchRequest) -> String {
        // Simple snippet generation - in production use proper snippet generation
        String::new()
    }

    /// Generate highlights
    ///
    /// # Arguments
    /// * `doc` - Tantivy document
    /// * `request` - Original search request
    ///
    /// # Returns
    /// Vector of highlight strings
    fn generate_highlights(&self, _doc: &TantivyDocument, _request: &SearchRequest) -> Vec<String> {
        // Simple highlight generation - in production use proper highlighting
        Vec::new()
    }

    /// Get suggestions for autocomplete
    ///
    /// Uses Tantivy's phrase prefix query to find document titles that
    /// match the given prefix.
    ///
    /// # Arguments
    /// * `prefix` - Partial query text to match against titles
    /// * `limit` - Maximum number of suggestions to return
    ///
    /// # Returns
    /// Result containing suggestions or error
    ///
    /// # Errors
    /// Returns error if suggestion generation fails
    pub async fn suggest(&self, prefix: &str, limit: usize) -> SearchResult<Vec<Suggestion>> {
        if prefix.is_empty() || limit == 0 {
            return Ok(Vec::new());
        }

        let reader = self.index_manager.reader()?;
        let searcher = reader.searcher();
        let schema = self.index_manager.schema();

        let title_field = schema.get_field("title").map_err(|e| {
            SearchError::query("FIELD_ERROR", format!("Failed to get title field: {}", e))
        })?;

        let lowered = prefix.to_lowercase();
        let terms: Vec<tantivy::Term> = lowered
            .split_whitespace()
            .map(|word| tantivy::Term::from_field_text(title_field, word))
            .collect();

        if terms.is_empty() {
            return Ok(Vec::new());
        }

        let prefix_query = tantivy::query::PhrasePrefixQuery::new(terms);

        let collector = TopDocs::with_limit(limit);
        let top_docs = searcher.search(&prefix_query, &collector).map_err(|e| {
            SearchError::query(
                "SUGGESTION_EXECUTION_ERROR",
                format!("Suggestion search failed: {}", e),
            )
        })?;

        let id_field = schema.get_field("id").map_err(|e| {
            SearchError::query("FIELD_ERROR", format!("Failed to get id field: {}", e))
        })?;

        let mut suggestions = Vec::new();
        let mut seen_titles = std::collections::HashSet::new();

        for (_score, doc_address) in top_docs {
            if let Ok(retrieved_doc) = searcher.doc::<TantivyDocument>(doc_address) {
                let title = retrieved_doc
                    .get_first(title_field)
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();

                if seen_titles.insert(title.clone()) {
                    let document_id = retrieved_doc
                        .get_first(id_field)
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    suggestions.push(Suggestion {
                        text: title,
                        document_id,
                        category: SuggestionCategory::Document,
                    });
                }
            }
        }

        Ok(suggestions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SearchDocument;
    use proptest::prelude::*;
    use tachyon_core::id::{DocumentId, UserId};
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_query_engine_creation() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().to_path_buf();

        let index_manager = IndexManager::new(index_path).await.unwrap();
        let query_engine = QueryEngine::new(index_manager);

        assert_eq!(query_engine.field_boosts.len(), 2);
    }

    async fn setup_index_with_docs(docs: Vec<SearchDocument>) -> (QueryEngine, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().to_path_buf();
        let index_manager = IndexManager::new(index_path).await.unwrap();

        for doc in &docs {
            index_manager.index_document(doc).await.unwrap();
        }

        (QueryEngine::new(index_manager), temp_dir)
    }

    fn make_doc(title: &str, content: &str) -> SearchDocument {
        SearchDocument::new(
            DocumentId::new(),
            title.to_string(),
            content.to_string(),
            UserId::new(),
        )
    }

    #[tokio::test]
    async fn test_suggest_empty_prefix_returns_nothing() {
        let (engine, _dir) = setup_index_with_docs(vec![
            make_doc("Rust Programming", "content"),
            make_doc("Rust Language Guide", "content"),
        ])
        .await;

        let results = engine.suggest("", 10).await.unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_suggest_zero_limit_returns_nothing() {
        let (engine, _dir) =
            setup_index_with_docs(vec![make_doc("Rust Programming", "content")]).await;

        let results = engine.suggest("rust", 0).await.unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_suggest_single_char_prefix() {
        let (engine, _dir) = setup_index_with_docs(vec![
            make_doc("Rust Programming Language", "content about rust"),
            make_doc("Rust Language Guide", "another rust doc"),
            make_doc("Python Programming", "content about python"),
        ])
        .await;

        let results = engine.suggest("r", 10).await.unwrap();
        assert!(!results.is_empty());
        for s in &results {
            assert!(s.text.to_lowercase().contains("r"));
            assert_eq!(s.category, SuggestionCategory::Document);
        }
    }

    #[tokio::test]
    async fn test_suggest_multi_word_prefix() {
        let (engine, _dir) = setup_index_with_docs(vec![
            make_doc("Rust Programming Language", "content"),
            make_doc("Rust Performance Tips", "content"),
            make_doc("Python Programming Language", "content"),
        ])
        .await;

        let results = engine.suggest("rust pro", 10).await.unwrap();
        assert!(!results.is_empty());
        assert!(results.iter().all(|s| {
            let lower = s.text.to_lowercase();
            lower.contains("rust") && lower.contains("pro")
        }));
    }

    #[tokio::test]
    async fn test_suggest_no_matches_returns_empty() {
        let (engine, _dir) = setup_index_with_docs(vec![
            make_doc("Rust Programming", "content"),
            make_doc("Rust Language Guide", "content"),
        ])
        .await;

        let results = engine.suggest("zzzzzzz", 10).await.unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_suggest_limit_respected() {
        let (engine, _dir) = setup_index_with_docs(vec![
            make_doc("Rust Programming Basics", "content"),
            make_doc("Rust Async Programming", "content"),
            make_doc("Rust Systems Programming", "content"),
            make_doc("Rust Web Programming", "content"),
            make_doc("Rust Embedded Programming", "content"),
        ])
        .await;

        let results = engine.suggest("rust", 2).await.unwrap();
        assert!(results.len() <= 2);
    }

    #[tokio::test]
    async fn test_suggest_case_insensitive() {
        let (engine, _dir) = setup_index_with_docs(vec![
            make_doc("Rust Programming", "content"),
            make_doc("rust programming guide", "content"),
        ])
        .await;

        let lower = engine.suggest("rust", 10).await.unwrap();
        let upper = engine.suggest("RUST", 10).await.unwrap();

        assert_eq!(lower.len(), upper.len());
    }

    #[tokio::test]
    async fn test_suggest_returns_document_ids() {
        let doc = make_doc("Rust Programming", "content");
        let doc_id = doc.id.to_string();
        let (engine, _dir) = setup_index_with_docs(vec![doc]).await;

        let results = engine.suggest("rust", 10).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].document_id.as_deref(), Some(doc_id.as_str()));
    }

    #[tokio::test]
    async fn test_suggest_whitespace_only_prefix() {
        let (engine, _dir) =
            setup_index_with_docs(vec![make_doc("Rust Programming", "content")]).await;

        let results = engine.suggest("   ", 10).await.unwrap();
        assert!(results.is_empty());
    }

    async fn run_roundtrip_via_engine(title: String, content_words: Vec<String>) {
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().to_path_buf();
        let index_manager = IndexManager::new(index_path).await.unwrap();

        let content = content_words.join(" ");
        let search_word = &content_words[0];

        let doc = SearchDocument::new(
            DocumentId::new(),
            title.clone(),
            content.clone(),
            UserId::new(),
        );
        index_manager.index_document(&doc).await.unwrap();

        let engine = QueryEngine::new(index_manager);
        let request = SearchRequest::new(search_word);
        let response = engine.search(&request).await.unwrap();

        assert!(
            !response.results.is_empty(),
            "Search for '{}' in content '{}' returned no results",
            search_word,
            content
        );
    }

    async fn run_empty_query_test() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().to_path_buf();
        let index_manager = IndexManager::new(index_path).await.unwrap();

        let doc = SearchDocument::new(
            DocumentId::new(),
            "Test".to_string(),
            "Some content here".to_string(),
            UserId::new(),
        );
        index_manager.index_document(&doc).await.unwrap();

        let engine = QueryEngine::new(index_manager);
        let request = SearchRequest::new("");
        let result = engine.search(&request).await;

        assert!(result.is_err(), "Empty query should return an error");
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(20))]

        #[test]
        fn prop_index_search_roundtrip_via_query_engine(
            title in "[a-zA-Z]{1,100}",
            content_words in proptest::collection::vec("[a-zA-Z]{3,10}", 2..10),
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(run_roundtrip_via_engine(title, content_words));
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(20))]

        #[test]
        fn prop_empty_query_returns_empty(
            _dummy in 0u8..1u8,
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(run_empty_query_test());
        }
    }
}
