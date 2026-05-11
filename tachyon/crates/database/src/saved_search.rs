// Saved Search
// Persistent search queries and filters for users

use crate::error::{DatabaseError, DatabaseResult};
use crate::schema::DatabasePool;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, FromRow, Row};
use tracing::{debug, info, instrument};

/// A user's persistent search query stored in `saved_searches`.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SavedSearch {
    /// Primary key (UUID).
    pub id: String,
    /// Owning user ID (UUID string).
    pub user_id: String,
    /// User-given label for the saved search.
    pub name: String,
    /// The raw search query text.
    pub query: String,
    /// JSON-encoded [`super::search::SearchFilters`], if any.
    pub filters: Option<String>,
    /// Row-creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Last-update timestamp.
    pub updated_at: DateTime<Utc>,
}

impl SavedSearch {
    /// Deserialize the `filters` JSON column into [`super::search::SearchFilters`].
    pub fn parse_filters(&self) -> DatabaseResult<Option<super::search::SearchFilters>> {
        match &self.filters {
            Some(f) => {
                Ok(Some(serde_json::from_str(f).map_err(|e| {
                    DatabaseError::SerializationError(e.to_string())
                })?))
            }
            None => Ok(None),
        }
    }

    /// Serialize [`super::search::SearchFilters`] to a JSON string for storage.
    pub fn serialize_filters(
        filters: &Option<super::search::SearchFilters>,
    ) -> DatabaseResult<Option<String>> {
        match filters {
            Some(f) => {
                Ok(Some(serde_json::to_string(f).map_err(|e| {
                    DatabaseError::SerializationError(e.to_string())
                })?))
            }
            None => Ok(None),
        }
    }
}

/// Request body for creating a new saved search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSavedSearchRequest {
    pub user_id: String,
    pub name: String,
    pub query: String,
    pub filters: Option<super::search::SearchFilters>,
}

/// Partial update payload for an existing saved search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSavedSearchRequest {
    pub name: Option<String>,
    pub query: Option<String>,
    pub filters: Option<super::search::SearchFilters>,
}

/// Repository for persisting and querying saved searches.
#[derive(Clone)]
pub struct SavedSearchRepository {
    pool: DatabasePool,
}

impl SavedSearchRepository {
    /// Create a new saved search repository backed by `pool`.
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    /// Insert a new saved search.
    #[instrument(skip(self))]
    pub async fn create(&self, request: CreateSavedSearchRequest) -> DatabaseResult<SavedSearch> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();
        let filters_json = SavedSearch::serialize_filters(&request.filters)?;

        let sql = r#"
            INSERT INTO saved_searches (id, user_id, name, query, filters, created_at, updated_at)
            VALUES ($1::uuid, $2::uuid, $3, $4, $5::jsonb, $6, $7)
            RETURNING id::text as id, user_id::text as user_id, name, query, filters::text as filters, created_at, updated_at
        "#;

        let mut conn = self.pool.acquire().await?;
        let saved_search: SavedSearch = query_as(sql)
            .bind(&id)
            .bind(&request.user_id)
            .bind(&request.name)
            .bind(&request.query)
            .bind(&filters_json)
            .bind(now)
            .bind(now)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| {
                if e.to_string().contains("duplicate key") {
                    DatabaseError::duplicate("saved_search", &request.name)
                } else {
                    DatabaseError::QueryError(e.to_string())
                }
            })?;

        info!(
            "Saved search created: {} for user {}",
            saved_search.id, saved_search.user_id
        );
        Ok(saved_search)
    }

    /// Retrieve a saved search by UUID.
    #[instrument(skip(self))]
    pub async fn get_by_id(&self, id: &str) -> DatabaseResult<SavedSearch> {
        let sql = r#"
            SELECT id::text as id, user_id::text as user_id, name, query, filters::text as filters, created_at, updated_at
            FROM saved_searches
            WHERE id = $1::uuid
        "#;

        let mut conn = self.pool.acquire().await?;
        let saved_search = query_as(sql)
            .bind(id)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        match saved_search {
            Some(s) => Ok(s),
            None => Err(DatabaseError::not_found("saved_search", id)),
        }
    }

    /// List all saved searches for a user, newest first.
    #[instrument(skip(self))]
    pub async fn list_by_user(&self, user_id: &str) -> DatabaseResult<Vec<SavedSearch>> {
        let sql = r#"
            SELECT id::text as id, user_id::text as user_id, name, query, filters::text as filters, created_at, updated_at
            FROM saved_searches
            WHERE user_id = $1::uuid
            ORDER BY created_at DESC
        "#;

        let mut conn = self.pool.acquire().await?;
        let saved_searches = query_as(sql)
            .bind(user_id)
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        debug!(
            "Found {} saved searches for user {}",
            saved_searches.len(),
            user_id
        );
        Ok(saved_searches)
    }

    /// Apply a partial update to an existing saved search.
    #[instrument(skip(self))]
    pub async fn update(
        &self,
        id: &str,
        request: UpdateSavedSearchRequest,
    ) -> DatabaseResult<SavedSearch> {
        let current = self.get_by_id(id).await?;
        let filters_json = SavedSearch::serialize_filters(&request.filters)?;

        let name = request.name.unwrap_or(current.name);
        let query = request.query.unwrap_or(current.query);
        let filters = filters_json.or(current.filters);
        let now = Utc::now();

        let sql = r#"
            UPDATE saved_searches
            SET name = $2, query = $3, filters = $4::jsonb, updated_at = $5
            WHERE id = $1::uuid
            RETURNING id::text as id, user_id::text as user_id, name, query, filters::text as filters, created_at, updated_at
        "#;

        let mut conn = self.pool.acquire().await?;
        let saved_search: SavedSearch = query_as(sql)
            .bind(id)
            .bind(&name)
            .bind(&query)
            .bind(&filters)
            .bind(now)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        info!("Saved search updated: {}", id);
        Ok(saved_search)
    }

    /// Permanently delete a saved search by UUID.
    #[instrument(skip(self))]
    pub async fn delete(&self, id: &str) -> DatabaseResult<()> {
        let sql = "DELETE FROM saved_searches WHERE id = $1::uuid";

        let mut conn = self.pool.acquire().await?;
        let result = query(sql)
            .bind(id)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::not_found("saved_search", id));
        }

        info!("Saved search deleted: {}", id);
        Ok(())
    }

    /// Count saved searches for a user.
    #[instrument(skip(self))]
    pub async fn count_by_user(&self, user_id: &str) -> DatabaseResult<i64> {
        let sql = "SELECT COUNT(*) as count FROM saved_searches WHERE user_id = $1::uuid";

        let mut conn = self.pool.acquire().await?;
        let row = query(sql)
            .bind(user_id)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        let count: i64 = row.get("count");
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::super::search::SearchFilters;
    use super::*;
    use pretty_assertions::assert_eq;

    fn make_saved_search(filters: Option<&str>) -> SavedSearch {
        SavedSearch {
            id: "1".into(),
            user_id: "user-1".into(),
            name: "My Search".into(),
            query: "test query".into(),
            filters: filters.map(String::from),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn test_serialize_filters_none() {
        let result = SavedSearch::serialize_filters(&None).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_serialize_filters_some() {
        let filters = SearchFilters {
            content_type: Some("markdown".into()),
            status: Some("published".into()),
            ..Default::default()
        };
        let result = SavedSearch::serialize_filters(&Some(filters)).unwrap();
        assert!(result.is_some());
        let json = result.unwrap();
        assert!(json.contains("markdown"));
        assert!(json.contains("published"));
    }

    #[test]
    fn test_parse_filters_none() {
        let search = make_saved_search(None);
        let result = search.parse_filters().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_filters_valid() {
        let filters = SearchFilters::default();
        let json = serde_json::to_string(&filters).unwrap();
        let search = make_saved_search(Some(&json));
        let result = search.parse_filters().unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_parse_filters_invalid_json() {
        let search = make_saved_search(Some("not json"));
        assert!(search.parse_filters().is_err());
    }

    #[test]
    fn test_saved_search_struct_fields() {
        let search = make_saved_search(None);
        assert_eq!(search.name, "My Search");
        assert_eq!(search.query, "test query");
        assert_eq!(search.user_id, "user-1");
    }

    #[test]
    fn test_create_saved_search_request_fields() {
        let req = CreateSavedSearchRequest {
            user_id: "user-1".into(),
            name: "Search".into(),
            query: "hello".into(),
            filters: None,
        };
        assert_eq!(req.name, "Search");
        assert!(req.filters.is_none());
    }

    #[test]
    fn test_update_saved_search_request_all_none() {
        let req = UpdateSavedSearchRequest {
            name: None,
            query: None,
            filters: None,
        };
        assert!(req.name.is_none());
        assert!(req.query.is_none());
    }
}
