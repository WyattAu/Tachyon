use crate::error::{DatabaseError, DatabaseResult};
use crate::schema::DatabasePool;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tracing::instrument;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingStep {
    pub id: String,
    pub name: String,
    pub completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingStatus {
    pub completed: bool,
    pub steps: Vec<OnboardingStep>,
    pub current_step: usize,
}

const ONBOARDING_STEPS: &[(&str, &str)] = &[
    ("create_first_document", "Create Your First Document"),
    ("invite_team", "Invite Your Team"),
    ("configure_profile", "Configure Your Profile"),
    ("explore_features", "Explore Features"),
];

#[derive(Clone)]
pub struct OnboardingRepository {
    pool: DatabasePool,
}

impl OnboardingRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    #[instrument(skip(self))]
    pub async fn get_user_document_count(&self, user_id: &str) -> DatabaseResult<i64> {
        let sql = "SELECT COUNT(*) as count FROM documents WHERE author_id::text = $1";
        let mut conn = self.pool.acquire().await?;
        let row = sqlx::query(sql)
            .bind(user_id)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| DatabaseError::query_error(e.to_string()))?;
        Ok(row.get("count"))
    }

    #[instrument(skip(self))]
    pub async fn get_user_created_at(&self, user_id: &str) -> DatabaseResult<Option<chrono::DateTime<chrono::Utc>>> {
        let sql = "SELECT created_at FROM users WHERE id::text = $1";
        let mut conn = self.pool.acquire().await?;
        let row = sqlx::query(sql)
            .bind(user_id)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::query_error(e.to_string()))?;
        match row {
            Some(row) => Ok(row.get("created_at")),
            None => Ok(None),
        }
    }

    #[instrument(skip(self))]
    pub async fn get_onboarding_status(&self, user_id: &str) -> DatabaseResult<OnboardingStatus> {
        let doc_count = self.get_user_document_count(user_id).await?;
        let completed = doc_count >= 3;

        let mut conn = self.pool.acquire().await?;
        let row = sqlx::query("SELECT preferences FROM user_preferences WHERE user_id::text = $1")
            .bind(user_id)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::query_error(e.to_string()))?;

        let completed_steps: Vec<String> = match row {
            Some(r) => {
                let prefs: serde_json::Value = r.get("preferences");
                prefs
                    .get("onboarding")
                    .and_then(|o| o.get("completed_steps"))
                    .and_then(|cs| cs.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default()
            }
            None => vec![],
        };

        let steps: Vec<OnboardingStep> = ONBOARDING_STEPS
            .iter()
            .map(|(id, name)| OnboardingStep {
                id: id.to_string(),
                name: name.to_string(),
                completed: completed || completed_steps.contains(&id.to_string()),
            })
            .collect();

        let current_step = steps
            .iter()
            .position(|s| !s.completed)
            .unwrap_or(steps.len());

        Ok(OnboardingStatus {
            completed,
            steps,
            current_step,
        })
    }

    #[instrument(skip(self))]
    pub async fn complete_step(&self, user_id: &str, step_id: &str) -> DatabaseResult<()> {
        let valid_step = ONBOARDING_STEPS
            .iter()
            .any(|(id, _)| *id == step_id);
        if !valid_step {
            return Err(DatabaseError::validation_error(format!(
                "Invalid step_id: {}",
                step_id
            )));
        }

        let status = self.get_onboarding_status(user_id).await?;
        if status.completed {
            return Ok(());
        }

        let mut conn = self.pool.acquire().await?;
        let row = sqlx::query("SELECT preferences FROM user_preferences WHERE user_id::text = $1")
            .bind(user_id)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::query_error(e.to_string()))?;

        let mut prefs: serde_json::Value = match row {
            Some(r) => {
                let p: serde_json::Value = r.get("preferences");
                p
            }
            None => serde_json::json!({}),
        };

        let onboarding = prefs
            .as_object_mut()
            .unwrap()
            .entry("onboarding")
            .or_insert_with(|| serde_json::json!({}));

        let completed_steps = onboarding
            .as_object_mut()
            .unwrap()
            .entry("completed_steps")
            .or_insert_with(|| serde_json::json!([]));

        if let Some(arr) = completed_steps.as_array_mut() {
            if !arr.iter().any(|v| v.as_str() == Some(step_id)) {
                arr.push(serde_json::json!(step_id));
            }
        }

        let sql = r#"
            INSERT INTO user_preferences (user_id, preferences, updated_at)
            VALUES ($1, $2, NOW())
            ON CONFLICT (user_id) DO UPDATE SET preferences = $2, updated_at = NOW()
        "#;
        sqlx::query(sql)
            .bind(user_id)
            .bind(&prefs)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::query_error(e.to_string()))?;

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn is_onboarded(&self, user_id: &str) -> DatabaseResult<bool> {
        let status = self.get_onboarding_status(user_id).await?;
        Ok(status.completed)
    }
}
