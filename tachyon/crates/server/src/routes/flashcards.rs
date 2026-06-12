use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
};
use serde::Serialize;
use tachyon_database::flashcard::{
    CreateFlashcardRequest, Flashcard, FlashcardRepository, FlashcardSrsState, Rating,
    ReviewFlashcardRequest, UpdateFlashcardRequest,
};

use crate::flashcards::srs;

#[derive(Clone)]
pub struct FlashcardState {
    pub repo: FlashcardRepository,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct FlashcardResponse {
    pub flashcard: Flashcard,
    pub srs_state: Option<FlashcardSrsState>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct FlashcardListResponse {
    pub flashcards: Vec<Flashcard>,
    pub total: usize,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct FlashcardReviewResponse {
    pub flashcard: Flashcard,
    pub srs_state: FlashcardSrsState,
    pub message: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ReviewQueueResponse {
    pub cards: Vec<Flashcard>,
    pub total_due: usize,
}

/// Create a new flashcard.
///
/// `POST /api/v1/flashcards`
pub async fn create_flashcard(
    State(state): State<FlashcardState>,
    Json(req): Json<CreateFlashcardRequest>,
) -> Result<(StatusCode, Json<FlashcardResponse>), (StatusCode, String)> {
    let flashcard = state
        .repo
        .create(req)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let srs_state = state.repo.get_srs_state(flashcard.id).await.ok();

    Ok((
        StatusCode::CREATED,
        Json(FlashcardResponse {
            flashcard,
            srs_state,
        }),
    ))
}

/// List all flashcards.
///
/// `GET /api/v1/flashcards`
pub async fn list_flashcards(
    State(state): State<FlashcardState>,
) -> Result<Json<FlashcardListResponse>, (StatusCode, String)> {
    let cards = state
        .repo
        .list_all()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let total = cards.len();
    Ok(Json(FlashcardListResponse {
        flashcards: cards,
        total,
    }))
}

/// Get a single flashcard by ID.
///
/// `GET /api/v1/flashcards/:id`
pub async fn get_flashcard(
    State(state): State<FlashcardState>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<FlashcardResponse>, (StatusCode, String)> {
    let flashcard = state.repo.get(id).await.map_err(|e| match e {
        tachyon_database::DatabaseError::NotFound { .. } => (StatusCode::NOT_FOUND, e.to_string()),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    })?;

    let srs_state = state.repo.get_srs_state(id).await.ok();
    Ok(Json(FlashcardResponse {
        flashcard,
        srs_state,
    }))
}

/// Update a flashcard.
///
/// `PUT /api/v1/flashcards/:id`
pub async fn update_flashcard(
    State(state): State<FlashcardState>,
    Path(id): Path<uuid::Uuid>,
    Json(req): Json<UpdateFlashcardRequest>,
) -> Result<Json<FlashcardResponse>, (StatusCode, String)> {
    let flashcard = state.repo.update(id, req).await.map_err(|e| match e {
        tachyon_database::DatabaseError::NotFound { .. } => (StatusCode::NOT_FOUND, e.to_string()),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    })?;

    let srs_state = state.repo.get_srs_state(id).await.ok();
    Ok(Json(FlashcardResponse {
        flashcard,
        srs_state,
    }))
}

/// Delete a flashcard.
///
/// `DELETE /api/v1/flashcards/:id`
pub async fn delete_flashcard(
    State(state): State<FlashcardState>,
    Path(id): Path<uuid::Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    state.repo.delete(id).await.map_err(|e| match e {
        tachyon_database::DatabaseError::NotFound { .. } => (StatusCode::NOT_FOUND, e.to_string()),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    })?;
    Ok(StatusCode::NO_CONTENT)
}

/// Submit a review for a flashcard.
///
/// `POST /api/v1/flashcards/:id/review`
pub async fn review_flashcard(
    State(state): State<FlashcardState>,
    Path(id): Path<uuid::Uuid>,
    Json(req): Json<ReviewFlashcardRequest>,
) -> Result<Json<FlashcardReviewResponse>, (StatusCode, String)> {
    let srs_state = state
        .repo
        .get_srs_state(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let rating = Rating::from_i16(req.rating);
    let now = chrono::Utc::now();
    let updated = srs::process_review(&srs_state, rating, now);

    state
        .repo
        .update_srs_state(
            id,
            updated.state,
            updated.step,
            updated.stability,
            updated.difficulty,
            updated.due,
            updated.reps,
            updated.lapses,
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    state
        .repo
        .log_review(id, req.rating)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let flashcard = state
        .repo
        .get(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let final_srs = state
        .repo
        .get_srs_state(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(FlashcardReviewResponse {
        flashcard,
        srs_state: final_srs,
        message: format!("Review recorded: {:?}", rating),
    }))
}

/// Get the review queue (cards due for review).
///
/// `GET /api/v1/flashcards/review`
pub async fn get_review_queue(
    State(state): State<FlashcardState>,
) -> Result<Json<ReviewQueueResponse>, (StatusCode, String)> {
    let cards = state
        .repo
        .get_due_cards(50)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let total_due = cards.len();
    Ok(Json(ReviewQueueResponse { cards, total_due }))
}

pub fn create_flashcard_router() -> Router<FlashcardState> {
    Router::new()
        .route("/flashcards", post(create_flashcard).get(list_flashcards))
        .route("/flashcards/review", get(get_review_queue))
        .route(
            "/flashcards/{id}",
            get(get_flashcard)
                .put(update_flashcard)
                .delete(delete_flashcard),
        )
        .route("/flashcards/{id}/review", post(review_flashcard))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rating_from_i16() {
        assert_eq!(Rating::from_i16(0), Rating::Again);
        assert_eq!(Rating::from_i16(1), Rating::Hard);
        assert_eq!(Rating::from_i16(2), Rating::Good);
        assert_eq!(Rating::from_i16(3), Rating::Easy);
        assert_eq!(Rating::from_i16(99), Rating::Again);
    }
}
