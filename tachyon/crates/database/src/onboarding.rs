use crate::error::{DatabaseError, DatabaseResult};
use crate::schema::DatabasePool;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tracing::instrument;

/// A single step in the onboarding flow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingStep {
    /// Machine-readable step identifier (e.g. `create_first_document`).
    pub id: String,
    /// Human-readable step label.
    pub name: String,
    /// Whether the user has completed this step.
    pub completed: bool,
}

/// Snapshot of a user's current onboarding progress.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingStatus {
    /// `true` when all steps are complete (or the shortcut condition is met).
    pub completed: bool,
    /// Ordered list of all onboarding steps with their completion state.
    pub steps: Vec<OnboardingStep>,
    /// Index of the first incomplete step (equal to `steps.len()` when all done).
    pub current_step: usize,
}

const ONBOARDING_STEPS: &[(&str, &str)] = &[
    ("create_first_document", "Create Your First Document"),
    ("invite_team", "Invite Your Team"),
    ("configure_profile", "Configure Your Profile"),
    ("explore_features", "Explore Features"),
];

/// Repository for tracking new-user onboarding progress.
#[derive(Clone)]
pub struct OnboardingRepository {
    pool: DatabasePool,
}

impl OnboardingRepository {
    /// Create a new onboarding repository backed by `pool`.
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    /// Count documents authored by a user (used to determine auto-completion).
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

    /// Look up when a user's account was created.
    #[instrument(skip(self))]
    pub async fn get_user_created_at(
        &self,
        user_id: &str,
    ) -> DatabaseResult<Option<chrono::DateTime<chrono::Utc>>> {
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

    /// Build the full [`OnboardingStatus`] for a user.
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

    /// Mark a single onboarding step as completed.
    ///
    /// Silently returns `Ok(())` if onboarding is already complete or the step
    /// is already recorded.
    #[instrument(skip(self))]
    pub async fn complete_step(&self, user_id: &str, step_id: &str) -> DatabaseResult<()> {
        let valid_step = ONBOARDING_STEPS.iter().any(|(id, _)| *id == step_id);
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
            .ok_or_else(|| {
                DatabaseError::SerializationError("preferences is not a JSON object".to_string())
            })?
            .entry("onboarding")
            .or_insert_with(|| serde_json::json!({}));

        let completed_steps = onboarding
            .as_object_mut()
            .ok_or_else(|| {
                DatabaseError::SerializationError("onboarding is not a JSON object".to_string())
            })?
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

    /// Returns `true` if the user has completed all onboarding steps.
    #[instrument(skip(self))]
    pub async fn is_onboarded(&self, user_id: &str) -> DatabaseResult<bool> {
        let status = self.get_onboarding_status(user_id).await?;
        Ok(status.completed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_onboarding_steps_count() {
        assert_eq!(ONBOARDING_STEPS.len(), 4);
    }

    #[test]
    fn test_onboarding_step_ids() {
        let ids: Vec<&str> = ONBOARDING_STEPS.iter().map(|(id, _)| *id).collect();
        assert!(ids.contains(&"create_first_document"));
        assert!(ids.contains(&"invite_team"));
        assert!(ids.contains(&"configure_profile"));
        assert!(ids.contains(&"explore_features"));
    }

    #[test]
    fn test_onboarding_step_names() {
        let step = ONBOARDING_STEPS
            .iter()
            .find(|(id, _)| *id == "create_first_document");
        assert_eq!(
            step.map(|(_, name)| *name),
            Some("Create Your First Document")
        );

        let step = ONBOARDING_STEPS.iter().find(|(id, _)| *id == "invite_team");
        assert_eq!(step.map(|(_, name)| *name), Some("Invite Your Team"));
    }

    #[test]
    fn test_onboarding_status_incomplete() {
        let status = OnboardingStatus {
            completed: false,
            steps: vec![
                OnboardingStep {
                    id: "s1".into(),
                    name: "Step 1".into(),
                    completed: false,
                },
                OnboardingStep {
                    id: "s2".into(),
                    name: "Step 2".into(),
                    completed: true,
                },
            ],
            current_step: 0,
        };
        assert!(!status.completed);
        assert_eq!(status.current_step, 0);
        assert_eq!(status.steps.len(), 2);
    }

    #[test]
    fn test_onboarding_status_all_completed() {
        let status = OnboardingStatus {
            completed: true,
            steps: vec![
                OnboardingStep {
                    id: "s1".into(),
                    name: "Step 1".into(),
                    completed: true,
                },
                OnboardingStep {
                    id: "s2".into(),
                    name: "Step 2".into(),
                    completed: true,
                },
            ],
            current_step: 2,
        };
        assert!(status.completed);
        assert_eq!(status.current_step, 2);
    }

    #[test]
    fn test_onboarding_step_struct() {
        let step = OnboardingStep {
            id: "create_first_document".into(),
            name: "Create Your First Document".into(),
            completed: true,
        };
        assert_eq!(step.id, "create_first_document");
        assert!(step.completed);
    }

    #[test]
    fn test_onboarding_invalid_step_id() {
        let invalid_id = "nonexistent_step";
        let valid = ONBOARDING_STEPS.iter().any(|(id, _)| *id == invalid_id);
        assert!(!valid);
    }
}
