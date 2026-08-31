use crate::error::{DatabaseError, DatabaseResult};
use crate::schema::DatabasePool;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Flashcard state in the SRS system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "smallint", rename_all = "lowercase")]
pub enum CardState {
    New = 0,
    Learning = 1,
    Review = 2,
    Relearning = 3,
}

impl CardState {
    pub fn from_i16(v: i16) -> Self {
        match v {
            0 => Self::New,
            1 => Self::Learning,
            2 => Self::Review,
            3 => Self::Relearning,
            _ => Self::New,
        }
    }
}

/// Review rating: Again=0, Hard=1, Good=2, Easy=3
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "smallint", rename_all = "lowercase")]
pub enum Rating {
    Again = 0,
    Hard = 1,
    Good = 2,
    Easy = 3,
}

impl Rating {
    pub fn from_i16(v: i16) -> Self {
        match v {
            0 => Self::Again,
            1 => Self::Hard,
            2 => Self::Good,
            3 => Self::Easy,
            _ => Self::Again,
        }
    }
}

/// A flashcard with front/back content
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct Flashcard {
    pub id: uuid::Uuid,
    pub document_id: uuid::Uuid,
    pub front: String,
    pub back: String,
    pub tags: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// SRS state for a flashcard
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct FlashcardSrsState {
    pub flashcard_id: uuid::Uuid,
    pub state: i16,
    pub step: i16,
    pub stability: f64,
    pub difficulty: f64,
    pub due: DateTime<Utc>,
    pub reps: i32,
    pub lapses: i32,
    pub last_review: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

/// A single review entry in the log
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct ReviewLog {
    pub id: uuid::Uuid,
    pub flashcard_id: uuid::Uuid,
    pub rating: i16,
    pub reviewed_at: DateTime<Utc>,
}

/// Request to create a flashcard
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct CreateFlashcardRequest {
    pub document_id: uuid::Uuid,
    pub front: String,
    pub back: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Request to update a flashcard
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct UpdateFlashcardRequest {
    pub front: Option<String>,
    pub back: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// Request to submit a review
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct ReviewFlashcardRequest {
    pub rating: i16,
}

/// Flashcard repository for database operations
#[derive(Clone)]
pub struct FlashcardRepository {
    pool: DatabasePool,
}

impl FlashcardRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, req: CreateFlashcardRequest) -> DatabaseResult<Flashcard> {
        let id = uuid::Uuid::new_v4();
        let tags = serde_json::to_value(&req.tags).unwrap_or_default();
        let now = Utc::now();

        let card: Flashcard = sqlx::query_as::<_, Flashcard>(
            r#"INSERT INTO flashcards (id, document_id, front, back, tags, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               RETURNING *"#,
        )
        .bind(id)
        .bind(req.document_id)
        .bind(&req.front)
        .bind(&req.back)
        .bind(&tags)
        .bind(now)
        .bind(now)
        .fetch_one(self.pool.inner())
        .await?;

        sqlx::query(
            r#"INSERT INTO flashcard_srs_state (flashcard_id, state, step, stability, difficulty, due, reps, lapses, updated_at)
               VALUES ($1, 0, 0, 1.0, 0.3, $2, 0, 0, $2)"#,
        )
        .bind(id)
        .bind(now)
        .execute(self.pool.inner())
        .await?;

        Ok(card)
    }

    pub async fn get(&self, id: uuid::Uuid) -> DatabaseResult<Flashcard> {
        sqlx::query_as::<_, Flashcard>("SELECT * FROM flashcards WHERE id = $1")
            .bind(id)
            .fetch_optional(self.pool.inner())
            .await?
            .ok_or(DatabaseError::NotFound {
                entity_type: "Flashcard".into(),
                id: id.to_string(),
            })
    }

    pub async fn list_by_document(
        &self,
        document_id: uuid::Uuid,
    ) -> DatabaseResult<Vec<Flashcard>> {
        Ok(sqlx::query_as::<_, Flashcard>(
            "SELECT * FROM flashcards WHERE document_id = $1 ORDER BY created_at DESC",
        )
        .bind(document_id)
        .fetch_all(self.pool.inner())
        .await?)
    }

    pub async fn list_all(&self) -> DatabaseResult<Vec<Flashcard>> {
        Ok(
            sqlx::query_as::<_, Flashcard>("SELECT * FROM flashcards ORDER BY created_at DESC")
                .fetch_all(self.pool.inner())
                .await?,
        )
    }

    pub async fn update(
        &self,
        id: uuid::Uuid,
        req: UpdateFlashcardRequest,
    ) -> DatabaseResult<Flashcard> {
        let existing = self.get(id).await?;
        let front = req.front.unwrap_or(existing.front);
        let back = req.back.unwrap_or(existing.back);
        let tags = match req.tags {
            Some(t) => serde_json::to_value(&t).unwrap_or_default(),
            None => existing.tags,
        };
        let now = Utc::now();

        sqlx::query_as::<_, Flashcard>(
            r#"UPDATE flashcards SET front = $2, back = $3, tags = $4, updated_at = $5
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id)
        .bind(&front)
        .bind(&back)
        .bind(&tags)
        .bind(now)
        .fetch_optional(self.pool.inner())
        .await?
        .ok_or(DatabaseError::NotFound {
            entity_type: "Flashcard".into(),
            id: id.to_string(),
        })
    }

    pub async fn delete(&self, id: uuid::Uuid) -> DatabaseResult<()> {
        let result = sqlx::query("DELETE FROM flashcards WHERE id = $1")
            .bind(id)
            .execute(self.pool.inner())
            .await?;
        if result.rows_affected() == 0 {
            return Err(DatabaseError::NotFound {
                entity_type: "Flashcard".into(),
                id: id.to_string(),
            });
        }
        Ok(())
    }

    pub async fn get_srs_state(
        &self,
        flashcard_id: uuid::Uuid,
    ) -> DatabaseResult<FlashcardSrsState> {
        sqlx::query_as::<_, FlashcardSrsState>(
            "SELECT * FROM flashcard_srs_state WHERE flashcard_id = $1",
        )
        .bind(flashcard_id)
        .fetch_optional(self.pool.inner())
        .await?
        .ok_or(DatabaseError::NotFound {
            entity_type: "FlashcardSrsState".into(),
            id: flashcard_id.to_string(),
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn update_srs_state(
        &self,
        flashcard_id: uuid::Uuid,
        state: i16,
        step: i16,
        stability: f64,
        difficulty: f64,
        due: DateTime<Utc>,
        reps: i32,
        lapses: i32,
    ) -> DatabaseResult<FlashcardSrsState> {
        let now = Utc::now();
        sqlx::query_as::<_, FlashcardSrsState>(
            r#"UPDATE flashcard_srs_state
               SET state = $2, step = $3, stability = $4, difficulty = $5, due = $6,
                   reps = $7, lapses = $8, last_review = $9, updated_at = $9
               WHERE flashcard_id = $1
               RETURNING *"#,
        )
        .bind(flashcard_id)
        .bind(state)
        .bind(step)
        .bind(stability)
        .bind(difficulty)
        .bind(due)
        .bind(reps)
        .bind(lapses)
        .bind(now)
        .fetch_optional(self.pool.inner())
        .await?
        .ok_or(DatabaseError::NotFound {
            entity_type: "FlashcardSrsState".into(),
            id: flashcard_id.to_string(),
        })
    }

    pub async fn log_review(
        &self,
        flashcard_id: uuid::Uuid,
        rating: i16,
    ) -> DatabaseResult<ReviewLog> {
        let id = uuid::Uuid::new_v4();
        let now = Utc::now();

        Ok(sqlx::query_as::<_, ReviewLog>(
            r#"INSERT INTO flashcard_review_log (id, flashcard_id, rating, reviewed_at)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(id)
        .bind(flashcard_id)
        .bind(rating)
        .bind(now)
        .fetch_one(self.pool.inner())
        .await?)
    }

    pub async fn get_due_cards(&self, limit: i64) -> DatabaseResult<Vec<Flashcard>> {
        let now = Utc::now();
        Ok(sqlx::query_as::<_, Flashcard>(
            r#"SELECT f.* FROM flashcards f
               INNER JOIN flashcard_srs_state s ON f.id = s.flashcard_id
               WHERE s.due <= $1
               ORDER BY s.due ASC
               LIMIT $2"#,
        )
        .bind(now)
        .bind(limit)
        .fetch_all(self.pool.inner())
        .await?)
    }

    pub async fn get_review_history(
        &self,
        flashcard_id: uuid::Uuid,
    ) -> DatabaseResult<Vec<ReviewLog>> {
        Ok(sqlx::query_as::<_, ReviewLog>(
            r#"SELECT * FROM flashcard_review_log
               WHERE flashcard_id = $1
               ORDER BY reviewed_at DESC"#,
        )
        .bind(flashcard_id)
        .fetch_all(self.pool.inner())
        .await?)
    }
}
