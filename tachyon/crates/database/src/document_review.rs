// Document Review Repository
// Review workflow management for document versions

use crate::error::{DatabaseError, DatabaseResult};
use crate::schema::DatabasePool;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, FromRow, Row};
use tracing::{debug, info, instrument};

// ============================================================================
// Review Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReviewStatus {
    Pending,
    Approved,
    Rejected,
    ChangesRequested,
    Cancelled,
}

impl ReviewStatus {
    /// Valid transitions from this status
    pub fn can_transition_to(&self, target: &ReviewStatus) -> bool {
        match (self, target) {
            (ReviewStatus::Pending, ReviewStatus::Approved) => true,
            (ReviewStatus::Pending, ReviewStatus::Rejected) => true,
            (ReviewStatus::Pending, ReviewStatus::ChangesRequested) => true,
            (ReviewStatus::Pending, ReviewStatus::Cancelled) => true,
            (ReviewStatus::ChangesRequested, ReviewStatus::Approved) => true,
            (ReviewStatus::ChangesRequested, ReviewStatus::Rejected) => true,
            (ReviewStatus::ChangesRequested, ReviewStatus::Cancelled) => true,
            // Same-state or invalid transitions
            _ => false,
        }
    }
}

impl std::fmt::Display for ReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReviewStatus::Pending => write!(f, "pending"),
            ReviewStatus::Approved => write!(f, "approved"),
            ReviewStatus::Rejected => write!(f, "rejected"),
            ReviewStatus::ChangesRequested => write!(f, "changes_requested"),
            ReviewStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

const REVIEW_SELECT_SQL: &str = r#"
    SELECT
        id::text as id,
        document_id::text as document_id,
        version_number,
        status,
        reviewer_id::text as reviewer_id,
        summary,
        created_at,
        resolved_at
    FROM document_reviews
"#;

/// A review request for a specific document version.
///
/// Reviews follow a state machine: `Pending` can transition to
/// `Approved`, `Rejected`, `ChangesRequested`, or `Cancelled`.
/// `ChangesRequested` can additionally transition to `Approved`/`Rejected`/`Cancelled`.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DocumentReview {
    pub id: String,
    pub document_id: String,
    pub version_number: i32,
    pub status: String,
    pub reviewer_id: String,
    pub summary: Option<String>,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreateReviewRequest {
    pub document_id: String,
    pub version_number: i32,
    pub reviewer_id: String,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct UpdateReviewRequest {
    pub status: ReviewStatus,
    pub summary: Option<String>,
}

// ============================================================================
// Comment Types
// ============================================================================

const COMMENT_SELECT_SQL: &str = r#"
    SELECT
        id::text as id,
        review_id::text as review_id,
        author_id::text as author_id,
        content,
        created_at
    FROM review_comments
"#;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReviewComment {
    pub id: String,
    pub review_id: String,
    pub author_id: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCommentRequest {
    pub review_id: String,
    pub author_id: String,
    pub content: String,
}

// ============================================================================
// Repository
// ============================================================================

/// Repository for managing document review workflows and review comments.
///
/// Enforces status-transition rules and supports idempotent review creation
/// (returns the existing pending review if one already exists for the same
/// reviewer and document).
#[derive(Clone)]
pub struct DocumentReviewRepository {
    pool: DatabasePool,
}

impl DocumentReviewRepository {
    /// Create a new review repository backed by the given connection pool.
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    // --- Reviews ---

    /// Create a new review request for a document version.
    ///
    /// Idempotent: if a pending review already exists for the same
    /// reviewer and document, the existing review is returned without
    /// creating a duplicate.
    ///
    /// # Arguments
    /// * `req` - Review creation parameters
    ///
    /// # Returns
    /// The created or existing `DocumentReview`.
    ///
    /// # Errors
    /// Returns `DatabaseError::QueryError` on SQL failures.
    #[instrument(skip(self))]
    pub async fn create_review(&self, req: CreateReviewRequest) -> DatabaseResult<DocumentReview> {
        let mut conn = self.pool.acquire().await?;

        // Check for existing pending review by this reviewer on this document
        let existing: Option<DocumentReview> = query_as(&format!(
            "{} WHERE document_id = $1::uuid AND reviewer_id = $2::uuid AND status = 'pending'",
            REVIEW_SELECT_SQL
        ))
        .bind(&req.document_id)
        .bind(&req.reviewer_id)
        .fetch_optional(&mut *conn)
        .await
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if let Some(existing) = existing {
            return Ok(existing);
        }

        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        let review: DocumentReview = query_as(
            r#"
            INSERT INTO document_reviews (
                id, document_id, version_number, status, reviewer_id, summary, created_at
            ) VALUES ($1::uuid, $2::uuid, $3, 'pending', $4::uuid, $5, $6)
            RETURNING id::text as id, document_id::text as document_id, version_number,
                      status, reviewer_id::text as reviewer_id, summary, created_at, resolved_at
            "#,
        )
        .bind(&id)
        .bind(&req.document_id)
        .bind(req.version_number)
        .bind(&req.reviewer_id)
        .bind(&req.summary)
        .bind(now)
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        info!(
            "Review created: {} for document {}",
            review.id, req.document_id
        );
        Ok(review)
    }

    #[instrument(skip(self))]
    pub async fn get_review(&self, id: &str) -> DatabaseResult<DocumentReview> {
        let sql = format!("{} WHERE id = $1::uuid", REVIEW_SELECT_SQL);
        let mut conn = self.pool.acquire().await?;

        let review: Option<DocumentReview> = query_as(&sql)
            .bind(id)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        review.ok_or_else(|| DatabaseError::not_found("document_review", id))
    }

    #[instrument(skip(self))]
    pub async fn list_by_document(&self, document_id: &str) -> DatabaseResult<Vec<DocumentReview>> {
        let sql = format!(
            "{} WHERE document_id = $1::uuid ORDER BY created_at DESC",
            REVIEW_SELECT_SQL
        );
        let mut conn = self.pool.acquire().await?;

        let reviews: Vec<DocumentReview> = query_as(&sql)
            .bind(document_id)
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        debug!(
            "Found {} reviews for document {}",
            reviews.len(),
            document_id
        );
        Ok(reviews)
    }

    /// Transition a review to a new status.
    ///
    /// Validates the transition against the [`ReviewStatus::can_transition_to`]
    /// state machine before applying. Terminal states (`Approved`, `Rejected`,
    /// `Cancelled`) automatically set `resolved_at`.
    ///
    /// # Arguments
    /// * `id` - UUID of the review
    /// * `req` - New status and optional summary
    ///
    /// # Returns
    /// The updated `DocumentReview`.
    ///
    /// # Errors
    /// Returns `DatabaseError::ValidationError` if the transition is invalid
    /// or the current status is unknown.
    #[instrument(skip(self))]
    pub async fn update_review_status(
        &self,
        id: &str,
        req: UpdateReviewRequest,
    ) -> DatabaseResult<DocumentReview> {
        let mut conn = self.pool.acquire().await?;

        // Fetch current review
        let current = self.get_review(id).await?;
        let current_status = match current.status.as_str() {
            "pending" => ReviewStatus::Pending,
            "approved" => ReviewStatus::Approved,
            "rejected" => ReviewStatus::Rejected,
            "changes_requested" => ReviewStatus::ChangesRequested,
            "cancelled" => ReviewStatus::Cancelled,
            _ => {
                return Err(DatabaseError::ValidationError(format!(
                    "Unknown review status: {}",
                    current.status
                )))
            }
        };

        if !current_status.can_transition_to(&req.status) {
            return Err(DatabaseError::ValidationError(format!(
                "Cannot transition review from {} to {}",
                current_status, req.status
            )));
        }

        let now = Utc::now();
        let is_resolved = matches!(
            req.status,
            ReviewStatus::Approved | ReviewStatus::Rejected | ReviewStatus::Cancelled
        );

        let resolved_at: Option<DateTime<Utc>> = if is_resolved { Some(now) } else { None };

        let review: DocumentReview = query_as(
            r#"
            UPDATE document_reviews
            SET status = $2, summary = COALESCE($3, summary), resolved_at = $4
            WHERE id = $1::uuid
            RETURNING id::text as id, document_id::text as document_id, version_number,
                      status, reviewer_id::text as reviewer_id, summary, created_at, resolved_at
            "#,
        )
        .bind(id)
        .bind(req.status.to_string())
        .bind(&req.summary)
        .bind(resolved_at)
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        info!("Review {} updated to status: {}", id, req.status);
        Ok(review)
    }

    /// Count pending reviews for a document.
    ///
    /// # Arguments
    /// * `document_id` - UUID of the document
    ///
    /// # Returns
    /// The number of reviews currently in "pending" status.
    #[instrument(skip(self))]
    pub async fn get_pending_count(&self, document_id: &str) -> DatabaseResult<i64> {
        let mut conn = self.pool.acquire().await?;
        let row = query(
            "SELECT COUNT(*) as count FROM document_reviews WHERE document_id = $1::uuid AND status = 'pending'"
        )
        .bind(document_id)
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(row.get("count"))
    }

    #[instrument(skip(self))]
    pub async fn get_latest_status(&self, document_id: &str) -> DatabaseResult<Option<String>> {
        let sql = format!(
            "{} WHERE document_id = $1::uuid AND status IN ('approved', 'rejected', 'changes_requested') ORDER BY resolved_at DESC NULLS LAST LIMIT 1",
            REVIEW_SELECT_SQL
        );
        let mut conn = self.pool.acquire().await?;

        let review: Option<DocumentReview> = query_as(&sql)
            .bind(document_id)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(review.map(|r| r.status))
    }

    // --- Comments ---

    #[instrument(skip(self))]
    pub async fn create_comment(&self, req: CreateCommentRequest) -> DatabaseResult<ReviewComment> {
        let mut conn = self.pool.acquire().await?;

        // Verify review exists
        self.get_review(&req.review_id).await?;

        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        let comment: ReviewComment = query_as(
            r#"
            INSERT INTO review_comments (id, review_id, author_id, content, created_at)
            VALUES ($1::uuid, $2::uuid, $3::uuid, $4, $5)
            RETURNING id::text as id, review_id::text as review_id,
                      author_id::text as author_id, content, created_at
            "#,
        )
        .bind(&id)
        .bind(&req.review_id)
        .bind(&req.author_id)
        .bind(&req.content)
        .bind(now)
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        info!("Comment created on review {}", req.review_id);
        Ok(comment)
    }

    #[instrument(skip(self))]
    pub async fn list_comments(&self, review_id: &str) -> DatabaseResult<Vec<ReviewComment>> {
        let sql = format!(
            "{} WHERE review_id = $1::uuid ORDER BY created_at ASC",
            COMMENT_SELECT_SQL
        );
        let mut conn = self.pool.acquire().await?;

        let comments: Vec<ReviewComment> = query_as(&sql)
            .bind(review_id)
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(comments)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_review_status_transitions() {
        // Pending can transition to all terminal and changes_requested
        assert!(ReviewStatus::Pending.can_transition_to(&ReviewStatus::Approved));
        assert!(ReviewStatus::Pending.can_transition_to(&ReviewStatus::Rejected));
        assert!(ReviewStatus::Pending.can_transition_to(&ReviewStatus::ChangesRequested));
        assert!(ReviewStatus::Pending.can_transition_to(&ReviewStatus::Cancelled));

        // Changes_requested can re-transition
        assert!(ReviewStatus::ChangesRequested.can_transition_to(&ReviewStatus::Approved));
        assert!(ReviewStatus::ChangesRequested.can_transition_to(&ReviewStatus::Rejected));
        assert!(ReviewStatus::ChangesRequested.can_transition_to(&ReviewStatus::Cancelled));

        // Terminal states cannot transition
        assert!(!ReviewStatus::Approved.can_transition_to(&ReviewStatus::Rejected));
        assert!(!ReviewStatus::Rejected.can_transition_to(&ReviewStatus::Approved));
        assert!(!ReviewStatus::Cancelled.can_transition_to(&ReviewStatus::Pending));
    }

    #[test]
    fn test_review_status_display() {
        assert_eq!(ReviewStatus::Pending.to_string(), "pending");
        assert_eq!(ReviewStatus::Approved.to_string(), "approved");
        assert_eq!(
            ReviewStatus::ChangesRequested.to_string(),
            "changes_requested"
        );
    }

    #[test]
    fn test_review_status_serialization() {
        assert_eq!(
            serde_json::to_string(&ReviewStatus::ChangesRequested).unwrap(),
            "\"changes_requested\""
        );
        assert_eq!(
            serde_json::from_str::<ReviewStatus>("\"approved\"").unwrap(),
            ReviewStatus::Approved
        );
    }
}
