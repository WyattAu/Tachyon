// Document Review API Routes
// Review workflow endpoints for document approval/rejection

use crate::audit::{AuditEvent, AuditEventType, AuditLogger, AuditSeverity};
use crate::error::ServerError;
use axum::{
    extract::{Path, State},
    response::Json,
};
use serde::{Deserialize, Serialize};
use tachyon_database::{
    CreateCommentRequest, CreateNotification, DatabasePool, DocumentReviewRepository,
    NotificationRepository, ReviewStatus,
};
pub use tachyon_database::{CreateReviewRequest, UpdateReviewRequest};
use tracing::info;

// ============================================================================
// State
// ============================================================================

#[derive(Clone)]
pub struct ReviewState {
    pub pool: DatabasePool,
    pub http_client: reqwest::Client,
    pub audit_logger: AuditLogger,
}

impl ReviewState {
    pub fn new(pool: DatabasePool, http_client: reqwest::Client) -> Self {
        Self {
            pool,
            http_client,
            audit_logger: AuditLogger::disabled(),
        }
    }

    pub fn with_audit_logger(mut self, logger: AuditLogger) -> Self {
        self.audit_logger = logger;
        self
    }
}

// ============================================================================
// Response Types
// ============================================================================

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ReviewResponse {
    pub id: String,
    pub document_id: String,
    pub version_number: i32,
    pub status: String,
    pub reviewer_id: String,
    pub summary: Option<String>,
    pub created_at: String,
    pub resolved_at: Option<String>,
}

impl From<tachyon_database::DocumentReview> for ReviewResponse {
    fn from(r: tachyon_database::DocumentReview) -> Self {
        Self {
            id: r.id,
            document_id: r.document_id,
            version_number: r.version_number,
            status: r.status,
            reviewer_id: r.reviewer_id,
            summary: r.summary,
            created_at: r.created_at.to_rfc3339(),
            resolved_at: r.resolved_at.map(|t| t.to_rfc3339()),
        }
    }
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CommentResponse {
    pub id: String,
    pub review_id: String,
    pub author_id: String,
    pub content: String,
    pub created_at: String,
}

impl From<tachyon_database::ReviewComment> for CommentResponse {
    fn from(c: tachyon_database::ReviewComment) -> Self {
        Self {
            id: c.id,
            review_id: c.review_id,
            author_id: c.author_id,
            content: c.content,
            created_at: c.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateReviewBody {
    pub version_number: Option<i32>,
    pub reviewer_id: String,
    pub summary: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateReviewBody {
    pub status: String,
    pub summary: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateCommentBody {
    pub author_id: String,
    pub content: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ReviewStatusResponse {
    pub pending_count: i64,
    pub latest_status: Option<String>,
}

// ============================================================================
// Handlers
// ============================================================================

/// Create a new review request for a document.
///
/// `POST /api/v1/documents/{document_id}/reviews`
///
/// Sends a notification to the assigned reviewer and triggers a `review_created` webhook.
#[utoipa::path(
    post,
    path = "/documents/{document_id}/reviews",
    params(
        ("document_id" = String, Path, description = "Document ID"),
    ),
    request_body(content = CreateReviewBody, description = "Review creation request"),
    responses(
        (status = 200, description = "Review created", body = ReviewResponse),
        (status = 500, description = "Internal server error"),
    ),
    tag = "reviews",
    security(("bearer_auth" = [])),
)]
pub async fn create_review(
    Path(document_id): Path<String>,
    State(state): State<ReviewState>,
    Json(body): Json<CreateReviewBody>,
) -> Result<Json<ReviewResponse>, ServerError> {
    let repo = DocumentReviewRepository::new(state.pool.clone());

    let review = repo
        .create_review(CreateReviewRequest {
            document_id: document_id.clone(),
            version_number: body.version_number.unwrap_or(1),
            reviewer_id: body.reviewer_id,
            summary: body.summary,
        })
        .await
        .map_err(|e| ServerError::database(format!("Failed to create review: {}", e)))?;

    info!("Review created for document {}: {}", document_id, review.id);

    let _ = state
        .audit_logger
        .log(
            AuditEvent::new(
                AuditEventType::ReviewCreated,
                AuditSeverity::Low,
                "review_create",
                format!(
                    "Review '{}' created for document '{}'",
                    review.id, document_id
                ),
            )
            .with_target(&review.id, "review"),
        )
        .await;

    {
        let pool = state.pool.clone();
        let client = state.http_client.clone();
        let payload = serde_json::json!({
            "review_id": review.id.clone(),
            "document_id": document_id.clone(),
        });
        tokio::spawn(async move {
            crate::webhook_delivery::deliver_event(pool, client, "review_created", &payload).await;
        });
    }

    {
        let pool = state.pool.clone();
        let reviewer_id = review.reviewer_id.clone();
        let document_id = document_id.clone();
        let review_id = review.id.clone();
        tokio::spawn(async move {
            let _ = NotificationRepository::create(
                &pool,
                CreateNotification {
                    user_id: reviewer_id.parse().unwrap_or(uuid::Uuid::nil()),
                    notification_type: "review_requested".to_string(),
                    title: format!("Review requested on {}", document_id),
                    body: None,
                    link: Some(format!("/documents/{}/reviews/{}", document_id, review_id)),
                    metadata: Some(serde_json::json!({
                        "document_id": document_id,
                        "review_id": review_id,
                    })),
                },
            )
            .await;
        });
    }

    Ok(Json(ReviewResponse::from(review)))
}

/// List all reviews for a document.
///
/// `GET /api/v1/documents/{document_id}/reviews`
#[utoipa::path(
    get,
    path = "/documents/{document_id}/reviews",
    params(
        ("document_id" = String, Path, description = "Document ID"),
    ),
    responses(
        (status = 200, description = "List of reviews", body = Vec<ReviewResponse>),
        (status = 500, description = "Internal server error"),
    ),
    tag = "reviews",
    security(("bearer_auth" = [])),
)]
pub async fn list_reviews(
    Path(document_id): Path<String>,
    State(state): State<ReviewState>,
) -> Result<Json<Vec<ReviewResponse>>, ServerError> {
    let repo = DocumentReviewRepository::new(state.pool.clone());
    let reviews = repo
        .list_by_document(&document_id)
        .await
        .map_err(|e| ServerError::database(format!("Failed to list reviews: {}", e)))?;

    Ok(Json(
        reviews.into_iter().map(ReviewResponse::from).collect(),
    ))
}

/// Update a review's status.
///
/// `PUT /api/v1/documents/{document_id}/reviews/{review_id}`
///
/// Valid statuses: `approved`, `rejected`, `changes_requested`, `cancelled`.
/// Triggers a webhook event and notification on status change.
#[utoipa::path(
    put,
    path = "/documents/{document_id}/reviews/{review_id}",
    params(
        ("review_id" = String, Path, description = "Review ID"),
    ),
    request_body(content = UpdateReviewBody, description = "Review status update"),
    responses(
        (status = 200, description = "Review updated", body = ReviewResponse),
        (status = 400, description = "Invalid review status"),
        (status = 404, description = "Review not found"),
        (status = 409, description = "Invalid status transition"),
    ),
    tag = "reviews",
    security(("bearer_auth" = [])),
)]
pub async fn update_review(
    Path(review_id): Path<String>,
    State(state): State<ReviewState>,
    Json(body): Json<UpdateReviewBody>,
) -> Result<Json<ReviewResponse>, ServerError> {
    let repo = DocumentReviewRepository::new(state.pool.clone());

    let status = match body.status.as_str() {
        "approved" => ReviewStatus::Approved,
        "rejected" => ReviewStatus::Rejected,
        "changes_requested" => ReviewStatus::ChangesRequested,
        "cancelled" => ReviewStatus::Cancelled,
        _ => {
            return Err(ServerError::bad_request(format!(
                "Invalid review status: {}",
                body.status
            )));
        }
    };

    let summary = body.summary.clone();
    let review = repo
        .update_review_status(
            &review_id,
            UpdateReviewRequest {
                status,
                summary: body.summary,
            },
        )
        .await
        .map_err(|e| {
            if matches!(e, tachyon_database::DatabaseError::ValidationError(_)) {
                ServerError::conflict(format!("Failed to update review: {}", e))
            } else {
                ServerError::not_found("Review", &review_id)
            }
        })?;

    info!("Review {} updated to {}", review_id, body.status);

    let _ = state
        .audit_logger
        .log(
            AuditEvent::new(
                AuditEventType::ReviewUpdated,
                AuditSeverity::Low,
                "review_update",
                format!("Review '{}' updated to '{}'", review_id, body.status),
            )
            .with_target(&review_id, "review"),
        )
        .await;

    let notification_type = match body.status.as_str() {
        "approved" => "review_approved",
        "rejected" => "review_rejected",
        _ => "review_updated",
    };

    {
        let pool = state.pool.clone();
        let client = state.http_client.clone();
        let event_type = notification_type.to_string();
        let payload = serde_json::json!({
            "review_id": review_id.clone(),
            "status": body.status.clone(),
        });
        tokio::spawn(async move {
            crate::webhook_delivery::deliver_event(pool, client, &event_type, &payload).await;
        });
    }

    {
        let pool = state.pool.clone();
        let reviewer_id = review.reviewer_id.clone();
        let notification_type = notification_type.to_string();
        let review_status = body.status.clone();
        let review_id = review_id.clone();
        tokio::spawn(async move {
            let _ = NotificationRepository::create(
                &pool,
                CreateNotification {
                    user_id: reviewer_id.parse().unwrap_or(uuid::Uuid::nil()),
                    notification_type,
                    title: format!("Review {} for document", review_status),
                    body: summary,
                    link: Some(format!("/reviews/{}", review_id)),
                    metadata: Some(serde_json::json!({
                        "review_id": review_id,
                        "status": review_status,
                    })),
                },
            )
            .await;
        });
    }

    Ok(Json(ReviewResponse::from(review)))
}

/// Add a comment to a review.
///
/// `POST /api/v1/documents/{document_id}/reviews/{review_id}/comments`
///
/// Sends a `review_commented` notification to the review author.
#[utoipa::path(
    post,
    path = "/documents/{document_id}/reviews/{review_id}/comments",
    params(
        ("review_id" = String, Path, description = "Review ID"),
    ),
    request_body(content = CreateCommentBody, description = "Comment to create"),
    responses(
        (status = 200, description = "Comment created", body = CommentResponse),
        (status = 500, description = "Internal server error"),
    ),
    tag = "reviews",
    security(("bearer_auth" = [])),
)]
pub async fn create_comment(
    Path(review_id): Path<String>,
    State(state): State<ReviewState>,
    Json(body): Json<CreateCommentBody>,
) -> Result<Json<CommentResponse>, ServerError> {
    let repo = DocumentReviewRepository::new(state.pool.clone());

    let author_id = body.author_id.clone();
    let content = body.content.clone();
    let comment = repo
        .create_comment(CreateCommentRequest {
            review_id: review_id.clone(),
            author_id: body.author_id,
            content: body.content,
        })
        .await
        .map_err(|e| ServerError::database(format!("Failed to create comment: {}", e)))?;

    {
        let pool = state.pool.clone();
        let review_id = review_id.clone();
        let comment_id = comment.id.clone();
        tokio::spawn(async move {
            let _ = NotificationRepository::create(
                &pool,
                CreateNotification {
                    user_id: author_id.parse().unwrap_or(uuid::Uuid::nil()),
                    notification_type: "review_commented".to_string(),
                    title: format!("New comment on review {}", review_id),
                    body: Some(content),
                    link: Some(format!("/reviews/{}/comments", review_id)),
                    metadata: Some(serde_json::json!({
                        "review_id": review_id,
                        "comment_id": comment_id,
                    })),
                },
            )
            .await;
        });
    }

    let _ = state
        .audit_logger
        .log(
            AuditEvent::new(
                AuditEventType::ReviewCommentAdded,
                AuditSeverity::Low,
                "review_comment",
                format!("Comment added to review '{}'", review_id),
            )
            .with_target(&review_id, "review"),
        )
        .await;

    Ok(Json(CommentResponse::from(comment)))
}

/// List comments on a review.
///
/// `GET /api/v1/documents/{document_id}/reviews/{review_id}/comments`
#[utoipa::path(
    get,
    path = "/documents/{document_id}/reviews/{review_id}/comments",
    params(
        ("review_id" = String, Path, description = "Review ID"),
    ),
    responses(
        (status = 200, description = "List of comments", body = Vec<CommentResponse>),
        (status = 500, description = "Internal server error"),
    ),
    tag = "reviews",
    security(("bearer_auth" = [])),
)]
pub async fn list_comments(
    Path(review_id): Path<String>,
    State(state): State<ReviewState>,
) -> Result<Json<Vec<CommentResponse>>, ServerError> {
    let repo = DocumentReviewRepository::new(state.pool.clone());
    let comments = repo
        .list_comments(&review_id)
        .await
        .map_err(|e| ServerError::database(format!("Failed to list comments: {}", e)))?;

    Ok(Json(
        comments.into_iter().map(CommentResponse::from).collect(),
    ))
}

/// Get review status summary for a document.
///
/// `GET /api/v1/documents/{document_id}/reviews/status`
///
/// Returns the pending review count and the latest review status.
#[utoipa::path(
    get,
    path = "/documents/{document_id}/reviews/status",
    params(
        ("document_id" = String, Path, description = "Document ID"),
    ),
    responses(
        (status = 200, description = "Review status summary", body = ReviewStatusResponse),
        (status = 500, description = "Internal server error"),
    ),
    tag = "reviews",
    security(("bearer_auth" = [])),
)]
pub async fn get_review_status(
    Path(document_id): Path<String>,
    State(state): State<ReviewState>,
) -> Result<Json<ReviewStatusResponse>, ServerError> {
    let repo = DocumentReviewRepository::new(state.pool.clone());

    let pending_count = repo
        .get_pending_count(&document_id)
        .await
        .map_err(|e| ServerError::database(format!("Failed to get review status: {}", e)))?;

    let latest_status = repo
        .get_latest_status(&document_id)
        .await
        .map_err(|e| ServerError::database(format!("Failed to get latest review status: {}", e)))?;

    Ok(Json(ReviewStatusResponse {
        pending_count,
        latest_status,
    }))
}

// ============================================================================
// Router
// ============================================================================

pub fn create_review_router() -> axum::Router<ReviewState> {
    use axum::routing::{get, post, put};

    axum::Router::new()
        .route("/documents/{document_id}/reviews", post(create_review))
        .route("/documents/{document_id}/reviews", get(list_reviews))
        .route(
            "/documents/{document_id}/reviews/status",
            get(get_review_status),
        )
        .route(
            "/documents/{document_id}/reviews/{review_id}",
            put(update_review),
        )
        .route(
            "/documents/{document_id}/reviews/{review_id}/comments",
            post(create_comment),
        )
        .route(
            "/documents/{document_id}/reviews/{review_id}/comments",
            get(list_comments),
        )
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_review_body_deserialization() {
        let body = CreateReviewBody {
            version_number: Some(3),
            reviewer_id: "user-123".to_string(),
            summary: Some("Please review".to_string()),
        };
        assert_eq!(body.version_number, Some(3));
        assert_eq!(body.reviewer_id, "user-123");
    }

    #[test]
    fn test_update_review_body_deserialization() {
        let body = UpdateReviewBody {
            status: "approved".to_string(),
            summary: Some("Looks good".to_string()),
        };
        assert_eq!(body.status, "approved");
    }

    #[test]
    fn test_create_comment_body_deserialization() {
        let body = CreateCommentBody {
            author_id: "user-456".to_string(),
            content: "Nit: fix typo on line 3".to_string(),
        };
        assert_eq!(body.content, "Nit: fix typo on line 3");
    }
}
