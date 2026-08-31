//! Tantivy full-text search integration.
//!
//! Provides helper functions for initializing and querying the Tantivy
//! search index, used as an alternative/fallback to PostgreSQL tsvector.

use std::path::Path;
use tachyon_search::{IndexManager, SearchRequest};
use tracing::info;

/// Create and initialize a Tantivy search index at the given path.
///
/// Returns an `IndexManager` ready for indexing and querying.
pub async fn init_search_index(
    index_path: &Path,
) -> Result<IndexManager, Box<dyn std::error::Error>> {
    let manager = IndexManager::new(index_path.to_path_buf()).await?;
    info!(
        "Tantivy search index initialized at: {}",
        index_path.display()
    );
    Ok(manager)
}

/// Open an existing Tantivy search index at the given path.
pub async fn open_search_index(
    index_path: &Path,
) -> Result<IndexManager, Box<dyn std::error::Error>> {
    let manager = IndexManager::open(index_path.to_path_buf()).await?;
    info!("Tantivy search index opened at: {}", index_path.display());
    Ok(manager)
}

/// Search using a Tantivy index via the query engine.
///
/// Returns matching documents up to the specified limit.
pub fn search_tantivy(
    manager: &IndexManager,
    query: &str,
    limit: usize,
    offset: usize,
) -> Result<Vec<tachyon_search::SearchResponseItem>, Box<dyn std::error::Error>> {
    let engine = tachyon_search::QueryEngine::new(manager.clone());
    let request = SearchRequest::new(query).with_pagination(offset, limit);

    let rt = tokio::runtime::Handle::current();
    let results = rt.block_on(engine.search(&request))?;

    Ok(results.results)
}
