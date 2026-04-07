// Search ranking module
// Provides BM25 ranking algorithm for search results

use crate::types::{SearchRequest, SearchResponseItem, SortOrder};
use chrono::Utc;
use tracing::{debug, instrument};

// ============================================================================
// BM25 Ranker
// ============================================================================

/// BM25 ranking algorithm implementation
pub struct BM25Ranker {
    /// k1 parameter - term frequency saturation
    k1: f32,
    /// b parameter - length normalization
    b: f32,
    /// Average document length
    avg_doc_length: f32,
    /// Number of documents in corpus
    doc_count: u64,
}

impl BM25Ranker {
    /// Create a new BM25 ranker with default parameters
    ///
    /// # Returns
    /// New BM25Ranker instance
    pub fn new() -> Self {
        Self {
            k1: 1.2,
            b: 0.75,
            avg_doc_length: 0.0,
            doc_count: 0,
        }
    }

    /// Create a new BM25 ranker with custom parameters
    ///
    /// # Arguments
    /// * `k1` - k1 parameter (default: 1.2)
    /// * `b` - b parameter (default: 0.75)
    ///
    /// # Returns
    /// New BM25Ranker instance
    pub fn with_params(k1: f32, b: f32) -> Self {
        Self {
            k1,
            b,
            avg_doc_length: 0.0,
            doc_count: 0,
        }
    }

    /// Set corpus statistics for ranking
    ///
    /// # Arguments
    /// * `avg_doc_length` - Average document length
    /// * `doc_count` - Number of documents
    pub fn set_corpus_stats(&mut self, avg_doc_length: f32, doc_count: u64) {
        self.avg_doc_length = avg_doc_length;
        self.doc_count = doc_count;
    }

    /// Calculate BM25 score for a document
    ///
    /// # Arguments
    /// * `term_freq` - Term frequency in document
    /// * `doc_length` - Document length
    /// * `term_doc_freq` - Number of documents containing term
    ///
    /// # Returns
    /// BM25 score
    ///
    /// # Formula
    /// score = IDF * ((TF * (k1 + 1)) / (TF + k1 * (1 - b + b * (doc_length / avg_doc_length))))
    /// IDF = log((doc_count - term_doc_freq + 0.5) / (term_doc_freq + 0.5))
    #[instrument(skip_all)]
    pub fn calculate_bm25(&self, term_freq: u64, doc_length: u64, term_doc_freq: u64) -> f32 {
        if term_doc_freq == 0 {
            return 0.0;
        }

        let tf = term_freq as f32;
        let doc_len = doc_length as f32;

        // Calculate IDF
        let idf = ((self.doc_count - term_doc_freq + 1) as f32 / (term_doc_freq as f32 + 1.0)).ln();

        // Calculate length normalization
        let length_norm = 1.0 - self.b + self.b * (doc_len / self.avg_doc_length);

        // Calculate term frequency component
        let tf_component = (tf * (self.k1 + 1.0)) / (tf + self.k1 * length_norm);

        // Calculate final score
        idf * tf_component
    }

    /// Rank search results by sort order
    ///
    /// # Arguments
    /// * `results` - Search results to rank
    /// * `sort_order` - Sort order to apply
    ///
    /// # Returns
    /// Ranked search results
    #[instrument(skip_all)]
    pub fn rank_results(
        &self,
        mut results: Vec<SearchResponseItem>,
        sort_order: SortOrder,
    ) -> Vec<SearchResponseItem> {
        match sort_order {
            SortOrder::Score => {
                results.sort_by(|a, b| {
                    b.score
                        .partial_cmp(&a.score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                debug!("Sorted results by score");
            }
            SortOrder::DateDesc => {
                results.sort_by(|a, b| b.created_at.cmp(&a.created_at));
                debug!("Sorted results by date (newest first)");
            }
            SortOrder::DateAsc => {
                results.sort_by(|a, b| a.created_at.cmp(&b.created_at));
                debug!("Sorted results by date (oldest first)");
            }
            SortOrder::TitleAsc => {
                results.sort_by(|a, b| a.title.cmp(&b.title));
                debug!("Sorted results by title (A-Z)");
            }
            SortOrder::TitleDesc => {
                results.sort_by(|a, b| b.title.cmp(&a.title));
                debug!("Sorted results by title (Z-A)");
            }
        }
        results
    }

    /// Calculate relevance score based on multiple factors
    ///
    /// # Arguments
    /// * `result` - Search result item
    /// * `query_terms` - Query terms used
    /// * `request` - Original search request
    ///
    /// # Returns
    /// Relevance score
    #[instrument(skip_all)]
    pub fn calculate_relevance(
        &self,
        result: &SearchResponseItem,
        query_terms: &[String],
        request: &SearchRequest,
    ) -> f32 {
        let mut score = result.score;

        // Bonus for title matches
        for term in query_terms {
            if result.title.to_lowercase().contains(&term.to_lowercase()) {
                score *= 2.0;
            }
        }

        // Bonus for tag matches
        if let Some(ref tags) = request.tags {
            for tag in tags {
                if result.tags.contains(tag) {
                    score *= 1.5;
                }
            }
        }

        // Bonus for recent documents (recency boost)
        let days_since_update = (Utc::now() - result.created_at).num_days().abs();
        if days_since_update < 30 {
            score *= 1.2; // 20% boost for recent documents
        } else if days_since_update < 90 {
            score *= 1.1; // 10% boost for documents updated within 90 days
        }

        score
    }

    /// Re-rank search results with custom scoring
    ///
    /// # Arguments
    /// * `results` - Search results to re-rank
    /// * `query_terms` - Query terms
    /// * `request` - Original search request
    ///
    /// # Returns
    /// Re-ranked search results
    #[instrument(skip_all)]
    pub fn rerank_results(
        &self,
        mut results: Vec<SearchResponseItem>,
        query_terms: &[String],
        request: &SearchRequest,
    ) -> Vec<SearchResponseItem> {
        for result in &mut results {
            let relevance_score = self.calculate_relevance(result, query_terms, request);
            result.score = relevance_score;
        }

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        debug!("Re-ranked {} results with custom scoring", results.len());
        results
    }
}

impl Default for BM25Ranker {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Query Statistics
// ============================================================================

/// Statistics for query performance analysis
#[derive(Debug, Clone)]
pub struct QueryStats {
    /// Number of terms in query
    pub term_count: usize,
    /// Number of documents matched
    pub matched_docs: usize,
    /// Average term frequency
    pub avg_term_freq: f32,
    /// Query execution time in milliseconds
    pub execution_time_ms: u64,
}

impl QueryStats {
    /// Create new query statistics
    ///
    /// # Returns
    /// New QueryStats instance
    pub fn new() -> Self {
        Self {
            term_count: 0,
            matched_docs: 0,
            avg_term_freq: 0.0,
            execution_time_ms: 0,
        }
    }

    /// Record term frequency
    ///
    /// # Arguments
    /// * `freq` - Term frequency
    pub fn record_term_freq(&mut self, freq: u64) {
        self.term_count += 1;
        self.avg_term_freq = (self.avg_term_freq * (self.term_count - 1) as f32 + freq as f32)
            / self.term_count as f32;
    }

    /// Record execution time
    ///
    /// # Arguments
    /// * `time_ms` - Execution time in milliseconds
    pub fn record_execution_time(&mut self, time_ms: u64) {
        self.execution_time_ms = time_ms;
    }
}

impl Default for QueryStats {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Field Weights
// ============================================================================

/// Field weights for search ranking
#[derive(Debug, Clone)]
pub struct FieldWeights {
    /// Title field weight
    pub title: f32,
    /// Content field weight
    pub content: f32,
    /// Tags field weight
    pub tags: f32,
    /// Author field weight
    pub author: f32,
}

impl FieldWeights {
    /// Create new field weights with default values
    ///
    /// # Returns
    /// New FieldWeights instance
    pub fn new() -> Self {
        Self {
            title: 2.0,
            content: 1.0,
            tags: 1.5,
            author: 0.5,
        }
    }

    /// Set title weight
    ///
    /// # Arguments
    /// * `weight` - Title weight
    pub fn with_title_weight(mut self, weight: f32) -> Self {
        self.title = weight;
        self
    }

    /// Set content weight
    ///
    /// # Arguments
    /// * `weight` - Content weight
    pub fn with_content_weight(mut self, weight: f32) -> Self {
        self.content = weight;
        self
    }

    /// Set tags weight
    ///
    /// # Arguments
    /// * `weight` - Tags weight
    pub fn with_tags_weight(mut self, weight: f32) -> Self {
        self.tags = weight;
        self
    }

    /// Set author weight
    ///
    /// # Arguments
    /// * `weight` - Author weight
    pub fn with_author_weight(mut self, weight: f32) -> Self {
        self.author = weight;
        self
    }
}

impl Default for FieldWeights {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Result Aggregator
// ============================================================================

/// Aggregates multiple search results from different queries
pub struct ResultAggregator {
    /// Field weights for scoring
    #[allow(dead_code)]
    weights: FieldWeights,
}

impl ResultAggregator {
    /// Create new result aggregator
    ///
    /// # Arguments
    /// * `weights` - Field weights
    pub fn new(weights: FieldWeights) -> Self {
        Self { weights }
    }

    /// Create new result aggregator with default weights
    ///
    /// # Returns
    /// New ResultAggregator instance
    pub fn default() -> Self {
        Self::new(FieldWeights::default())
    }

    /// Aggregate results from multiple queries
    ///
    /// # Arguments
    /// * `results_sets` - Multiple result sets to aggregate
    ///
    /// # Returns
    /// Aggregated and ranked results
    #[instrument(skip_all)]
    pub fn aggregate(&self, results_sets: Vec<Vec<SearchResponseItem>>) -> Vec<SearchResponseItem> {
        let mut combined: Vec<SearchResponseItem> = Vec::new();
        let mut seen_ids = std::collections::HashSet::new();
        let num_sets = results_sets.len();

        for results in results_sets {
            for result in results {
                if seen_ids.insert(result.document_id.clone()) {
                    combined.push(result.clone());
                }
            }
        }

        combined.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        debug!(
            "Aggregated {} unique results from {} result sets",
            combined.len(),
            num_sets
        );
        combined
    }

    /// Fuse results from multiple sources with score averaging
    ///
    /// # Arguments
    /// * `results_sets` - Multiple result sets to fuse
    ///
    /// # Returns
    /// Fused results
    #[instrument(skip_all)]
    pub fn fuse_results(
        &self,
        results_sets: Vec<Vec<SearchResponseItem>>,
    ) -> Vec<SearchResponseItem> {
        let mut fused_map: std::collections::HashMap<
            tachyon_core::id::DocumentId,
            (Vec<f32>, SearchResponseItem),
        > = std::collections::HashMap::new();
        let num_sets = results_sets.len();

        for results in results_sets {
            for result in results {
                fused_map
                    .entry(result.document_id.clone())
                    .or_insert_with(|| (Vec::new(), result.clone()))
                    .0
                    .push(result.score);
            }
        }

        let mut fused: Vec<SearchResponseItem> = fused_map
            .into_iter()
            .map(|(_id, (scores, mut result))| {
                let avg_score = scores.iter().sum::<f32>() / scores.len() as f32;
                result.score = avg_score;
                result
            })
            .collect();

        fused.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        debug!(
            "Fused {} results from {} result sets",
            fused.len(),
            num_sets
        );
        fused
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SearchDocument;
    use tachyon_core::id::{DocumentId, UserId};

    #[test]
    fn test_bm25_ranker_creation() {
        let ranker = BM25Ranker::new();
        assert_eq!(ranker.k1, 1.2);
        assert_eq!(ranker.b, 0.75);
    }

    #[test]
    fn test_bm25_calculation() {
        let mut ranker = BM25Ranker::new();
        ranker.set_corpus_stats(100.0, 1000);

        let score = ranker.calculate_bm25(5, 80, 10);
        assert!(score > 0.0);
    }

    #[test]
    fn test_rank_results_by_score() {
        let ranker = BM25Ranker::new();
        let mut results = vec![
            SearchResponseItem {
                document_id: DocumentId::new(),
                title: "Doc 1".to_string(),
                snippet: "Snippet 1".to_string(),
                score: 0.5,
                highlights: vec![],
                author_id: UserId::new(),
                repository_id: None,
                tags: vec![],
                created_at: chrono::Utc::now(),
            },
            SearchResponseItem {
                document_id: DocumentId::new(),
                title: "Doc 2".to_string(),
                snippet: "Snippet 2".to_string(),
                score: 1.0,
                highlights: vec![],
                author_id: UserId::new(),
                repository_id: None,
                tags: vec![],
                created_at: chrono::Utc::now(),
            },
        ];

        let ranked = ranker.rank_results(results, SortOrder::Score);
        assert_eq!(ranked[0].score, 1.0);
        assert_eq!(ranked[1].score, 0.5);
    }

    #[test]
    fn test_rank_results_by_date() {
        let ranker = BM25Ranker::new();
        let mut results = vec![
            SearchResponseItem {
                document_id: DocumentId::new(),
                title: "Doc 1".to_string(),
                snippet: "Snippet 1".to_string(),
                score: 0.5,
                highlights: vec![],
                author_id: UserId::new(),
                repository_id: None,
                tags: vec![],
                created_at: chrono::Utc::now() - chrono::Duration::days(10),
            },
            SearchResponseItem {
                document_id: DocumentId::new(),
                title: "Doc 2".to_string(),
                snippet: "Snippet 2".to_string(),
                score: 1.0,
                highlights: vec![],
                author_id: UserId::new(),
                repository_id: None,
                tags: vec![],
                created_at: chrono::Utc::now(),
            },
        ];

        let ranked = ranker.rank_results(results, SortOrder::DateDesc);
        assert!(ranked[0].created_at >= ranked[1].created_at);
    }

    #[test]
    fn test_relevance_calculation() {
        let ranker = BM25Ranker::new();
        let result = SearchResponseItem {
            document_id: DocumentId::new(),
            title: "Test Document".to_string(),
            snippet: "Test content".to_string(),
            score: 1.0,
            highlights: vec![],
            author_id: UserId::new(),
            repository_id: None,
            tags: vec!["test".to_string()],
            created_at: chrono::Utc::now(),
        };

        let request = SearchRequest::new("test").with_tags(vec!["test".to_string()]);
        let query_terms = vec!["test".to_string()];
        let relevance = ranker.calculate_relevance(&result, &query_terms, &request);
        assert!(relevance > 1.0); // Should be boosted due to title and tag match
    }

    #[test]
    fn test_field_weights() {
        let weights = FieldWeights::new()
            .with_title_weight(3.0)
            .with_content_weight(1.5);

        assert_eq!(weights.title, 3.0);
        assert_eq!(weights.content, 1.5);
        assert_eq!(weights.tags, 1.5);
        assert_eq!(weights.author, 0.5);
    }

    #[test]
    fn test_aggregate_results() {
        let aggregator = ResultAggregator::default();

        let set1 = vec![SearchResponseItem {
            document_id: DocumentId::new(),
            title: "Doc 1".to_string(),
            snippet: "Snippet 1".to_string(),
            score: 1.0,
            highlights: vec![],
            author_id: UserId::new(),
            repository_id: None,
            tags: vec![],
            created_at: chrono::Utc::now(),
        }];

        let set2 = vec![SearchResponseItem {
            document_id: DocumentId::new(),
            title: "Doc 2".to_string(),
            snippet: "Snippet 2".to_string(),
            score: 0.8,
            highlights: vec![],
            author_id: UserId::new(),
            repository_id: None,
            tags: vec![],
            created_at: chrono::Utc::now(),
        }];

        let aggregated = aggregator.aggregate(vec![set1, set2]);
        assert_eq!(aggregated.len(), 2);
    }

    #[test]
    fn test_fuse_results() {
        let aggregator = ResultAggregator::default();

        let doc_id = DocumentId::new();
        let doc = SearchResponseItem {
            document_id: doc_id.clone(),
            title: "Test Doc".to_string(),
            snippet: "Test".to_string(),
            score: 1.0,
            highlights: vec![],
            author_id: UserId::new(),
            repository_id: None,
            tags: vec![],
            created_at: chrono::Utc::now(),
        };

        let set1 = vec![doc.clone()];
        let set2 = vec![doc.clone()];

        let fused = aggregator.fuse_results(vec![set1, set2]);
        assert_eq!(fused.len(), 1);
        assert_eq!(fused[0].document_id, doc_id);
    }
}
