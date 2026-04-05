// Query Processing Module
// Query parsing and execution for search operations

use crate::IndexManager;
use crate::error::{SearchError, SearchResult};
use crate::types::SearchRequest;
use std::sync::Arc;
use tantivy::{
    DocAddress, Searcher, TantivyDocument,
    collector::TopDocs,
    query::{Query, QueryParser},
    schema::*,
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
        let limit = request.page_size as usize;
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
    /// # Arguments
    /// * `query` - Partial query
    /// * `limit` - Maximum number of suggestions
    ///
    /// # Returns
    /// Result containing suggestions or error
    ///
    /// # Errors
    /// Returns error if suggestion generation fails
    pub async fn suggest(&self, _query: &str, _limit: usize) -> SearchResult<Vec<String>> {
        // Simple suggestion implementation
        let suggestions = Vec::new();
        Ok(suggestions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_query_engine_creation() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().to_path_buf();

        let index_manager = IndexManager::new(index_path).await.unwrap();
        let query_engine = QueryEngine::new(index_manager);

        assert_eq!(query_engine.field_boosts.len(), 2);
    }
}
