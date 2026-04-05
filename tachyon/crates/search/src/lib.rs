// Tachyon Search Library
// Full-text search engine for Tachyon knowledge management system using Tantivy

pub mod api;
pub mod error;
pub mod indexer;
pub mod query;
pub mod ranking;
pub mod types;

// Re-export common types for convenience
pub use api::{SearchApiState, create_router};
pub use error::{ErrorCategory, SearchError, SearchResult};
pub use indexer::IndexManager;
pub use query::QueryEngine;
pub use ranking::{BM25Ranker, FieldWeights, QueryStats, ResultAggregator};
pub use types::{
    BM25Config, BatchIndexRequest, BatchIndexResponse, BooleanOperator, FieldDefinition, FieldType,
    IndexConfig, QueryType, RangeValue, SearchDocument, SearchRequest, SearchResponse,
    SearchResponseItem, SortOrder,
};

/// Search library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize search index manager with default configuration
///
/// # Returns
/// Result containing initialized IndexManager or error
///
/// # Errors
/// Returns error if initialization fails
pub async fn init() -> SearchResult<IndexManager> {
    IndexManager::new(std::env::temp_dir().join("tachyon_search")).await
}

/// Initialize search index manager with custom configuration
///
/// # Arguments
/// * `config` - Index configuration
///
/// # Returns
/// Result containing initialized IndexManager or error
///
/// # Errors
/// Returns error if initialization fails
pub async fn init_with_config(config: types::IndexConfig) -> SearchResult<IndexManager> {
    let index_path = config
        .index_path
        .as_ref()
        .map(|p| std::path::PathBuf::from(p))
        .unwrap_or_else(|| std::env::temp_dir().join("tachyon_search"));
    IndexManager::with_config(index_path, config).await
}

/// Create a new BM25 ranker with default parameters
///
/// # Returns
/// New BM25Ranker instance
pub fn create_ranker() -> BM25Ranker {
    BM25Ranker::new()
}

/// Create a new BM25 ranker with custom parameters
///
/// # Arguments
/// * `k1` - k1 parameter (default: 1.2)
/// * `b` - b parameter (default: 0.75)
///
/// # Returns
/// New BM25Ranker instance
pub fn create_ranker_with_params(k1: f32, b: f32) -> BM25Ranker {
    BM25Ranker::with_params(k1, b)
}

/// Create a new query engine
///
/// # Arguments
/// * `index_manager` - Index manager
///
/// # Returns
/// New QueryEngine instance
pub fn create_query_engine(index_manager: IndexManager) -> QueryEngine {
    QueryEngine::new(index_manager)
}

/// Create field weights with default values
///
/// # Returns
/// New FieldWeights instance
pub fn create_field_weights() -> FieldWeights {
    FieldWeights::new()
}

/// Create field weights with custom values
///
/// # Arguments
/// * `title_weight` - Title field weight
/// * `content_weight` - Content field weight
/// * `tags_weight` - Tags field weight
/// * `author_weight` - Author field weight
///
/// # Returns
/// New FieldWeights instance
pub fn create_field_weights_with_custom(
    title_weight: f32,
    content_weight: f32,
    tags_weight: f32,
    author_weight: f32,
) -> FieldWeights {
    FieldWeights::new()
        .with_title_weight(title_weight)
        .with_content_weight(content_weight)
        .with_tags_weight(tags_weight)
        .with_author_weight(author_weight)
}

/// Create result aggregator for combining multiple query results
///
/// # Returns
/// New ResultAggregator instance
pub fn create_aggregator() -> ResultAggregator {
    ResultAggregator::new(FieldWeights::default())
}
