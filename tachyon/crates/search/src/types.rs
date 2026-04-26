// Search types and structures
// Defines core data structures for search operations

use crate::error::{SearchError, SearchResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tachyon_core::id::{DocumentId, RepositoryId, UserId};

// ============================================================================
// Document Field Types
// ============================================================================

/// Field types supported in the search index
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FieldType {
    /// Text field for full-text search
    #[serde(rename = "text")]
    Text,
    /// String field for exact matching
    #[serde(rename = "string")]
    String,
    /// Integer field
    #[serde(rename = "integer")]
    Integer,
    /// Date/time field
    #[serde(rename = "datetime")]
    DateTime,
    /// Boolean field
    #[serde(rename = "boolean")]
    Boolean,
}

/// Field definition for index schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FieldDefinition {
    /// Field name
    pub name: String,
    /// Field type
    pub field_type: FieldType,
    /// Whether field is stored for retrieval
    pub stored: bool,
    /// Whether field is indexed for search
    pub indexed: bool,
    /// Boost factor for field weighting
    pub boost: f32,
    /// Whether field is required
    pub required: bool,
}

impl FieldDefinition {
    /// Create a new field definition
    ///
    /// # Arguments
    /// * `name` - Field name
    /// * `field_type` - Field type
    pub fn new(name: impl Into<String>, field_type: FieldType) -> Self {
        Self {
            name: name.into(),
            field_type,
            stored: true,
            indexed: true,
            boost: 1.0,
            required: false,
        }
    }

    /// Set whether field is stored
    ///
    /// # Arguments
    /// * `stored` - Store flag
    pub fn with_stored(mut self, stored: bool) -> Self {
        self.stored = stored;
        self
    }

    /// Set whether field is indexed
    ///
    /// # Arguments
    /// * `indexed` - Index flag
    pub fn with_indexed(mut self, indexed: bool) -> Self {
        self.indexed = indexed;
        self
    }

    /// Set boost factor for field
    ///
    /// # Arguments
    /// * `boost` - Boost factor
    pub fn with_boost(mut self, boost: f32) -> Self {
        self.boost = boost;
        self
    }

    /// Set whether field is required
    ///
    /// # Arguments
    /// * `required` - Required flag
    pub fn with_required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }
}

// ============================================================================
// Index Configuration
// ============================================================================

/// Configuration for search index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexConfig {
    /// Index name
    pub name: String,
    /// Number of shards for parallel processing
    pub num_shards: usize,
    /// Path for index storage
    pub index_path: Option<String>,
    /// Field definitions
    pub fields: Vec<FieldDefinition>,
    /// BM25 parameters
    pub bm25: BM25Config,
}

/// BM25 ranking configuration
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BM25Config {
    /// k1 parameter - term frequency saturation
    pub k1: f32,
    /// b parameter - length normalization
    pub b: f32,
}

impl Default for BM25Config {
    fn default() -> Self {
        Self { k1: 1.2, b: 0.75 }
    }
}

impl IndexConfig {
    /// Create a new index configuration
    ///
    /// # Arguments
    /// * `name` - Index name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            num_shards: 1,
            index_path: None,
            fields: Vec::new(),
            bm25: BM25Config::default(),
        }
    }

    /// Set number of shards
    ///
    /// # Arguments
    /// * `num_shards` - Number of shards
    pub fn with_num_shards(mut self, num_shards: usize) -> Self {
        self.num_shards = num_shards;
        self
    }

    /// Set index storage path
    ///
    /// # Arguments
    /// * `path` - Storage path
    pub fn with_index_path(mut self, path: impl Into<String>) -> Self {
        self.index_path = Some(path.into());
        self
    }

    /// Add a field to the configuration
    ///
    /// # Arguments
    /// * `field` - Field definition
    pub fn add_field(mut self, field: FieldDefinition) -> Self {
        self.fields.push(field);
        self
    }

    /// Set BM25 configuration
    ///
    /// # Arguments
    /// * `bm25` - BM25 configuration
    pub fn with_bm25(mut self, bm25: BM25Config) -> Self {
        self.bm25 = bm25;
        self
    }
}

// ============================================================================
// Search Document
// ============================================================================

/// Document to be indexed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchDocument {
    /// Document ID
    pub id: DocumentId,
    /// Title
    pub title: String,
    /// Content text
    pub content: String,
    /// Author ID
    pub author_id: UserId,
    /// Repository ID (optional)
    pub repository_id: Option<RepositoryId>,
    /// Tags
    pub tags: Vec<String>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// Additional custom fields
    #[serde(flatten)]
    pub custom_fields: BTreeMap<String, serde_json::Value>,
}

impl SearchDocument {
    /// Create a new search document
    ///
    /// # Arguments
    /// * `id` - Document ID
    /// * `title` - Document title
    /// * `content` - Document content
    /// * `author_id` - Author user ID
    pub fn new(id: DocumentId, title: String, content: String, author_id: UserId) -> Self {
        let now = Utc::now();
        Self {
            id,
            title,
            content,
            author_id,
            repository_id: None,
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
            custom_fields: BTreeMap::new(),
        }
    }

    /// Set repository ID
    ///
    /// # Arguments
    /// * `repository_id` - Repository ID
    pub fn with_repository_id(mut self, repository_id: RepositoryId) -> Self {
        self.repository_id = Some(repository_id);
        self
    }

    /// Add tags to document
    ///
    /// # Arguments
    /// * `tags` - Tags to add
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Add a custom field
    ///
    /// # Arguments
    /// * `key` - Field name
    /// * `value` - Field value
    pub fn with_custom_field(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.custom_fields.insert(key.into(), value);
        self
    }

    /// Validate document
    ///
    /// # Returns
    /// Result indicating validation success or error
    pub fn validate(&self) -> SearchResult<()> {
        if self.title.is_empty() {
            return Err(SearchError::field_validation(
                "title",
                "Title cannot be empty",
            ));
        }

        if self.title.len() > 200 {
            return Err(SearchError::field_validation(
                "title",
                "Title too long (max 200 characters)",
            ));
        }

        if self.content.is_empty() {
            return Err(SearchError::field_validation(
                "content",
                "Content cannot be empty",
            ));
        }

        Ok(())
    }
}

// ============================================================================
// Query Types
// ============================================================================

/// Boolean operator for query combination
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BooleanOperator {
    /// AND - all terms must match
    #[serde(rename = "and")]
    And,
    /// OR - any term must match
    #[serde(rename = "or")]
    Or,
    /// NOT - term must not match
    #[serde(rename = "not")]
    Not,
}

/// Query type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryType {
    /// Term query
    #[serde(rename = "term")]
    Term { field: String, value: String },
    /// Phrase query
    #[serde(rename = "phrase")]
    Phrase {
        field: String,
        value: String,
        slop: usize,
    },
    /// Boolean query
    #[serde(rename = "boolean")]
    Boolean {
        operator: BooleanOperator,
        queries: Vec<QueryType>,
    },
    /// Range query
    #[serde(rename = "range")]
    Range {
        field: String,
        from: Option<RangeValue>,
        to: Option<RangeValue>,
    },
    /// Fuzzy query
    #[serde(rename = "fuzzy")]
    Fuzzy {
        field: String,
        value: String,
        distance: usize,
        prefix_length: usize,
    },
}

/// Range value for date/numeric ranges
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RangeValue {
    /// Integer value
    Integer(i64),
    /// DateTime value
    DateTime(String),
    /// String value
    String(String),
}

/// Sort order for results
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SortOrder {
    /// Sort by relevance score (descending)
    #[serde(rename = "score")]
    Score,
    /// Sort by date (newest first)
    #[serde(rename = "date_desc")]
    DateDesc,
    /// Sort by date (oldest first)
    #[serde(rename = "date_asc")]
    DateAsc,
    /// Sort by title (A-Z)
    #[serde(rename = "title_asc")]
    TitleAsc,
    /// Sort by title (Z-A)
    #[serde(rename = "title_desc")]
    TitleDesc,
}

// ============================================================================
// Search Request
// ============================================================================

/// Search request with all query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    /// Query string
    pub query: String,
    /// Structured query (alternative to query string)
    pub structured_query: Option<QueryType>,
    /// Field filters
    pub filters: BTreeMap<String, serde_json::Value>,
    /// Tags to filter by
    pub tags: Option<Vec<String>>,
    /// Repository ID to filter by
    pub repository_id: Option<RepositoryId>,
    /// Author ID to filter by
    pub author_id: Option<UserId>,
    /// Date range filter
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    /// Sort order
    pub sort: SortOrder,
    /// Page number (1-indexed)
    pub page: usize,
    /// Page size
    pub page_size: usize,
    /// Enable highlighting
    pub highlight: bool,
    /// Highlight snippet length
    pub snippet_length: usize,
}

impl Default for SearchRequest {
    fn default() -> Self {
        Self {
            query: String::new(),
            structured_query: None,
            filters: BTreeMap::new(),
            tags: None,
            repository_id: None,
            author_id: None,
            date_from: None,
            date_to: None,
            sort: SortOrder::Score,
            page: 1,
            page_size: 20,
            highlight: true,
            snippet_length: 200,
        }
    }
}

impl SearchRequest {
    /// Create a new search request
    ///
    /// # Arguments
    /// * `query` - Query string
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            ..Default::default()
        }
    }

    /// Set structured query
    ///
    /// # Arguments
    /// * `query` - Structured query
    pub fn with_structured_query(mut self, query: QueryType) -> Self {
        self.structured_query = Some(query);
        self
    }

    /// Add a filter
    ///
    /// # Arguments
    /// * `field` - Field name
    /// * `value` - Filter value
    pub fn with_filter(mut self, field: impl Into<String>, value: serde_json::Value) -> Self {
        self.filters.insert(field.into(), value);
        self
    }

    /// Set tags filter
    ///
    /// # Arguments
    /// * `tags` - Tags to filter by
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    /// Set repository filter
    ///
    /// # Arguments
    /// * `repository_id` - Repository ID
    pub fn with_repository_id(mut self, repository_id: RepositoryId) -> Self {
        self.repository_id = Some(repository_id);
        self
    }

    /// Set author filter
    ///
    /// # Arguments
    /// * `author_id` - Author ID
    pub fn with_author_id(mut self, author_id: UserId) -> Self {
        self.author_id = Some(author_id);
        self
    }

    /// Set date range filter
    ///
    /// # Arguments
    /// * `from` - Start date
    /// * `to` - End date
    pub fn with_date_range(
        mut self,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
    ) -> Self {
        self.date_from = from;
        self.date_to = to;
        self
    }

    /// Set sort order
    ///
    /// # Arguments
    /// * `sort` - Sort order
    pub fn with_sort(mut self, sort: SortOrder) -> Self {
        self.sort = sort;
        self
    }

    /// Set pagination
    ///
    /// # Arguments
    /// * `page` - Page number (1-indexed)
    /// * `page_size` - Results per page
    pub fn with_pagination(mut self, page: usize, page_size: usize) -> Self {
        self.page = page;
        self.page_size = page_size;
        self
    }

    /// Set page size
    ///
    /// # Arguments
    /// * `page_size` - Results per page
    pub fn with_page_size(mut self, page_size: usize) -> Self {
        self.page_size = page_size;
        self
    }

    /// Enable or disable highlighting
    ///
    /// # Arguments
    /// * `highlight` - Highlight flag
    pub fn with_highlight(mut self, highlight: bool) -> Self {
        self.highlight = highlight;
        self
    }

    /// Set snippet length
    ///
    /// # Arguments
    /// * `length` - Snippet length in characters
    pub fn with_snippet_length(mut self, length: usize) -> Self {
        self.snippet_length = length;
        self
    }

    /// Calculate offset for pagination
    ///
    /// # Returns
    /// Offset value
    pub fn offset(&self) -> usize {
        if self.page == 0 {
            return 0;
        }
        (self.page - 1) * self.page_size
    }

    /// Validate search request
    ///
    /// # Returns
    /// Result indicating validation success or error
    pub fn validate(&self) -> SearchResult<()> {
        if self.query.is_empty() && self.structured_query.is_none() {
            return Err(SearchError::invalid_query(
                "Query string or structured query required",
            ));
        }

        if self.page_size == 0 {
            return Err(SearchError::invalid_query(
                "Page size must be greater than 0",
            ));
        }

        if self.page_size > 100 {
            return Err(SearchError::invalid_query("Page size cannot exceed 100"));
        }

        Ok(())
    }
}

// ============================================================================
// Search Response
// ============================================================================

/// Search result item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponseItem {
    /// Document ID
    pub document_id: DocumentId,
    /// Document title
    pub title: String,
    /// Content snippet
    pub snippet: String,
    /// Relevance score
    pub score: f32,
    /// Highlighted terms
    pub highlights: Vec<String>,
    /// Author ID
    pub author_id: UserId,
    /// Repository ID (optional)
    pub repository_id: Option<RepositoryId>,
    /// Tags
    pub tags: Vec<String>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

/// Complete search response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    /// Search results
    pub results: Vec<SearchResponseItem>,
    /// Total number of results
    pub total_hits: usize,
    /// Current page number
    pub page: usize,
    /// Page size
    pub page_size: usize,
    /// Total number of pages
    pub total_pages: usize,
    /// Query execution time in milliseconds
    pub query_time_ms: u64,
}

impl SearchResponse {
    /// Create a new search response
    ///
    /// # Arguments
    /// * `results` - Search results
    /// * `total_hits` - Total number of hits
    /// * `request` - Original search request
    /// * `query_time_ms` - Query execution time
    pub fn new(
        results: Vec<SearchResponseItem>,
        total_hits: usize,
        request: &SearchRequest,
        query_time_ms: u64,
    ) -> Self {
        let total_pages = if request.page_size > 0 {
            total_hits.div_ceil(request.page_size)
        } else {
            0
        };

        Self {
            results,
            total_hits,
            page: request.page,
            page_size: request.page_size,
            total_pages,
            query_time_ms,
        }
    }

    /// Create empty response
    ///
    /// # Arguments
    /// * `request` - Original search request
    /// * `query_time_ms` - Query execution time
    pub fn empty(request: &SearchRequest, query_time_ms: u64) -> Self {
        Self {
            results: Vec::new(),
            total_hits: 0,
            page: request.page,
            page_size: request.page_size,
            total_pages: 0,
            query_time_ms,
        }
    }
}

// ============================================================================
// Suggestions
// ============================================================================

/// Category of a search suggestion
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SuggestionCategory {
    /// Suggestion from document titles
    #[serde(rename = "document")]
    Document,
    /// Suggestion from tags
    #[serde(rename = "tag")]
    Tag,
    /// Suggestion from spaces
    #[serde(rename = "space")]
    Space,
    /// Suggestion from recent searches
    #[serde(rename = "recent")]
    Recent,
}

/// A single search suggestion for autocomplete
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    /// Suggestion text
    pub text: String,
    /// Associated document ID (if applicable)
    pub document_id: Option<String>,
    /// Category of the suggestion
    pub category: SuggestionCategory,
}

impl Suggestion {
    /// Create a new document suggestion
    pub fn document(text: impl Into<String>, document_id: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            document_id: Some(document_id.into()),
            category: SuggestionCategory::Document,
        }
    }

    /// Create a new tag suggestion
    pub fn tag(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            document_id: None,
            category: SuggestionCategory::Tag,
        }
    }
}

// ============================================================================
// Batch Operations
// ============================================================================

/// Batch index operation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchIndexRequest {
    /// Documents to index
    pub documents: Vec<SearchDocument>,
    /// Whether to delete existing documents before indexing
    pub clear_before_index: bool,
    /// Index configuration (optional, for new indices)
    pub config: Option<IndexConfig>,
}

impl BatchIndexRequest {
    /// Create a new batch index request
    ///
    /// # Arguments
    /// * `documents` - Documents to index
    pub fn new(documents: Vec<SearchDocument>) -> Self {
        Self {
            documents,
            clear_before_index: false,
            config: None,
        }
    }

    /// Set clear before index flag
    ///
    /// # Arguments
    /// * `clear` - Clear flag
    pub fn with_clear_before_index(mut self, clear: bool) -> Self {
        self.clear_before_index = clear;
        self
    }

    /// Set index configuration
    ///
    /// # Arguments
    /// * `config` - Index configuration
    pub fn with_config(mut self, config: IndexConfig) -> Self {
        self.config = Some(config);
        self
    }
}

/// Batch index response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchIndexResponse {
    /// Number of documents indexed
    pub indexed_count: usize,
    /// Number of documents that failed
    pub failed_count: usize,
    /// Documents that failed to index
    pub failed_documents: Vec<DocumentId>,
    /// Operation time in milliseconds
    pub operation_time_ms: u64,
}

impl BatchIndexResponse {
    /// Create a new batch index response
    ///
    /// # Arguments
    /// * `indexed_count` - Number of documents indexed
    /// * `failed_documents` - Documents that failed
    /// * `operation_time_ms` - Operation time
    pub fn new(
        indexed_count: usize,
        failed_documents: Vec<DocumentId>,
        operation_time_ms: u64,
    ) -> Self {
        Self {
            indexed_count,
            failed_count: failed_documents.len(),
            failed_documents,
            operation_time_ms,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_definition() {
        let field = FieldDefinition::new("title", FieldType::Text)
            .with_boost(2.0)
            .with_required(true);

        assert_eq!(field.name, "title");
        assert_eq!(field.field_type, FieldType::Text);
        assert_eq!(field.boost, 2.0);
        assert!(field.required);
    }

    #[test]
    fn test_index_config() {
        let config = IndexConfig::new("test_index")
            .with_num_shards(2)
            .add_field(FieldDefinition::new("title", FieldType::Text));

        assert_eq!(config.name, "test_index");
        assert_eq!(config.num_shards, 2);
        assert_eq!(config.fields.len(), 1);
    }

    #[test]
    fn test_search_document() {
        let doc = SearchDocument::new(
            DocumentId::new(),
            "Test Title".to_string(),
            "Test content".to_string(),
            UserId::new(),
        )
        .with_tags(vec!["test".to_string()]);

        assert_eq!(doc.title, "Test Title");
        assert_eq!(doc.tags, vec!["test"]);
    }

    #[test]
    fn test_search_document_validation() {
        let doc = SearchDocument::new(
            DocumentId::new(),
            "Test Title".to_string(),
            "Test content".to_string(),
            UserId::new(),
        );
        assert!(doc.validate().is_ok());

        let invalid_doc = SearchDocument::new(
            DocumentId::new(),
            "".to_string(),
            "Test content".to_string(),
            UserId::new(),
        );
        assert!(invalid_doc.validate().is_err());
    }

    #[test]
    fn test_search_request() {
        let request = SearchRequest::new("test query")
            .with_page_size(10)
            .with_sort(SortOrder::DateDesc);

        assert_eq!(request.query, "test query");
        assert_eq!(request.page_size, 10);
        assert_eq!(request.sort, SortOrder::DateDesc);
        assert_eq!(request.offset(), 0);
    }

    #[test]
    fn test_search_request_pagination() {
        let request = SearchRequest::new("test")
            .with_page_size(20)
            .with_pagination(2, 20);

        assert_eq!(request.page, 2);
        assert_eq!(request.page_size, 20);
        assert_eq!(request.offset(), 20);
    }

    #[test]
    fn test_search_request_validation() {
        let valid = SearchRequest::new("test query");
        assert!(valid.validate().is_ok());

        let invalid = SearchRequest::new("");
        assert!(invalid.validate().is_err());

        let invalid_size = SearchRequest::new("test").with_page_size(0);
        assert!(invalid_size.validate().is_err());
    }

    #[test]
    fn test_search_response() {
        let request = SearchRequest::new("test").with_page_size(10);
        let response = SearchResponse::new(vec![], 25, &request, 50);

        assert_eq!(response.total_hits, 25);
        assert_eq!(response.page, 1);
        assert_eq!(response.page_size, 10);
        assert_eq!(response.total_pages, 3);
    }

    #[test]
    fn test_batch_index_request() {
        let doc = SearchDocument::new(
            DocumentId::new(),
            "Test".to_string(),
            "Content".to_string(),
            UserId::new(),
        );
        let request = BatchIndexRequest::new(vec![doc]).with_clear_before_index(true);

        assert_eq!(request.documents.len(), 1);
        assert!(request.clear_before_index);
    }
}
