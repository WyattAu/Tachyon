use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    routing::get,
};
use serde::{Deserialize, Serialize};
use tachyon_database::DatabasePool;



#[derive(Clone)]
pub struct AnalyticsState {
    pub pool: DatabasePool,
}

impl AnalyticsState {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

// ============================================================================
// Query Parameters
// ============================================================================

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct DateRangeQuery {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub granularity: Option<String>,
    pub days: Option<i32>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct ActivityQuery {
    pub days: Option<i32>,
}

// ============================================================================
// Response Types
// ============================================================================

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct AnalyticsOverview {
    pub total_documents: i64,
    pub total_users: i64,
    pub storage_bytes: i64,
    pub active_spaces: i64,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct DailyActivity {
    pub date: String,
    pub created: i64,
    pub updated: i64,
    pub deleted: i64,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct DailyActivityResponse {
    pub entries: Vec<DailyActivity>,
    pub total: usize,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct DailyUserActivity {
    pub date: String,
    pub active_users: i64,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UserActivityResponse {
    pub entries: Vec<DailyUserActivity>,
    pub total: usize,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct DailySearchCount {
    pub date: String,
    pub query_count: i64,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SearchActivityResponse {
    pub entries: Vec<DailySearchCount>,
    pub total: usize,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ApiRequestVolume {
    pub date: String,
    pub total_requests: i64,
    pub successful: i64,
    pub failed: i64,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ApiActivityResponse {
    pub entries: Vec<ApiRequestVolume>,
    pub total: usize,
}

// ============================================================================
// Handlers
// ============================================================================

#[utoipa::path(
    get,
    path = "/analytics/overview",
    responses(
        (status = 200, description = "Analytics overview", body = AnalyticsOverview),
        (status = 500, description = "Internal server error"),
    ),
    tag = "analytics",
    security(("bearer_auth" = [])),
)]
pub async fn get_overview(
    State(state): State<AnalyticsState>,
) -> Result<Json<AnalyticsOverview>, (StatusCode, String)> {
    let total_documents: (i64,) = sqlx::query_as(
        "SELECT COUNT(*)::bigint FROM documents WHERE deleted_at IS NULL",
    )
    .fetch_one(state.pool.inner())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let total_users: (i64,) = sqlx::query_as(
        "SELECT COUNT(*)::bigint FROM users",
    )
    .fetch_one(state.pool.inner())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let storage: (Option<i64>,) = sqlx::query_as(
        "SELECT COALESCE(SUM(LENGTH(content)), 0)::bigint FROM documents WHERE deleted_at IS NULL",
    )
    .fetch_one(state.pool.inner())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let active_spaces: (i64,) = sqlx::query_as(
        "SELECT COUNT(*)::bigint FROM spaces",
    )
    .fetch_one(state.pool.inner())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(AnalyticsOverview {
        total_documents: total_documents.0,
        total_users: total_users.0,
        storage_bytes: storage.0.unwrap_or(0),
        active_spaces: active_spaces.0,
    }))
}

#[utoipa::path(
    get,
    path = "/analytics/activity",
    params(DateRangeQuery),
    responses(
        (status = 200, description = "Document activity per day", body = DailyActivityResponse),
        (status = 500, description = "Internal server error"),
    ),
    tag = "analytics",
    security(("bearer_auth" = [])),
)]
pub async fn get_document_activity(
    State(state): State<AnalyticsState>,
    Query(query): Query<DateRangeQuery>,
) -> Result<Json<DailyActivityResponse>, (StatusCode, String)> {
    let days = query.days.unwrap_or(30);
    let start_date = query.start_date.unwrap_or_else(|| {
        (chrono::Utc::now() - chrono::Duration::days(days as i64))
            .format("%Y-%m-%d")
            .to_string()
    });
    let end_date = query.end_date.unwrap_or_else(|| {
        chrono::Utc::now().format("%Y-%m-%d").to_string()
    });

    let rows: Vec<(String, i64, i64, i64)> = sqlx::query_as(
        r#"
        WITH date_series AS (
            SELECT generate_series(
                $1::date,
                $2::date,
                '1 day'::interval
            )::date AS day
        ),
        creates AS (
            SELECT DATE(created_at) AS day, COUNT(*)::bigint AS cnt
            FROM documents
            WHERE created_at >= $1::date AND created_at <= ($2::date + interval '1 day')
              AND deleted_at IS NULL
            GROUP BY DATE(created_at)
        ),
        updates AS (
            SELECT DATE(updated_at) AS day, COUNT(*)::bigint AS cnt
            FROM documents
            WHERE updated_at >= $1::date AND updated_at <= ($2::date + interval '1 day')
              AND deleted_at IS NULL
            GROUP BY DATE(updated_at)
        ),
        deletes AS (
            SELECT DATE(deleted_at) AS day, COUNT(*)::bigint AS cnt
            FROM documents
            WHERE deleted_at >= $1::date AND deleted_at <= ($2::date + interval '1 day')
              AND deleted_at IS NOT NULL
            GROUP BY DATE(deleted_at)
        )
        SELECT
            ds.day::text,
            COALESCE(creates.cnt, 0),
            COALESCE(updates.cnt, 0),
            COALESCE(deletes.cnt, 0)
        FROM date_series ds
        LEFT JOIN creates ON creates.day = ds.day
        LEFT JOIN updates ON updates.day = ds.day
        LEFT JOIN deletes ON deletes.day = ds.day
        ORDER BY ds.day
        "#,
    )
    .bind(&start_date)
    .bind(&end_date)
    .fetch_all(state.pool.inner())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let entries: Vec<DailyActivity> = rows
        .into_iter()
        .map(|(date, created, updated, deleted)| DailyActivity {
            date,
            created,
            updated,
            deleted,
        })
        .collect();

    let total = entries.len();
    Ok(Json(DailyActivityResponse { entries, total }))
}

#[utoipa::path(
    get,
    path = "/analytics/users",
    params(DateRangeQuery),
    responses(
        (status = 200, description = "Active users per day", body = UserActivityResponse),
        (status = 500, description = "Internal server error"),
    ),
    tag = "analytics",
    security(("bearer_auth" = [])),
)]
pub async fn get_user_activity(
    State(state): State<AnalyticsState>,
    Query(query): Query<DateRangeQuery>,
) -> Result<Json<UserActivityResponse>, (StatusCode, String)> {
    let days = query.days.unwrap_or(30);
    let start_date = query.start_date.unwrap_or_else(|| {
        (chrono::Utc::now() - chrono::Duration::days(days as i64))
            .format("%Y-%m-%d")
            .to_string()
    });
    let end_date = query.end_date.unwrap_or_else(|| {
        chrono::Utc::now().format("%Y-%m-%d").to_string()
    });

    let rows: Vec<(String, i64)> = sqlx::query_as(
        r#"
        WITH date_series AS (
            SELECT generate_series(
                $1::date,
                $2::date,
                '1 day'::interval
            )::date AS day
        ),
        active AS (
            SELECT DATE(created_at) AS day, COUNT(DISTINCT actor_id)::bigint AS cnt
            FROM activity_events
            WHERE created_at >= $1::date AND created_at <= ($2::date + interval '1 day')
            GROUP BY DATE(created_at)
        )
        SELECT
            ds.day::text,
            COALESCE(active.cnt, 0)
        FROM date_series ds
        LEFT JOIN active ON active.day = ds.day
        ORDER BY ds.day
        "#,
    )
    .bind(&start_date)
    .bind(&end_date)
    .fetch_all(state.pool.inner())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let entries: Vec<DailyUserActivity> = rows
        .into_iter()
        .map(|(date, active_users)| DailyUserActivity { date, active_users })
        .collect();

    let total = entries.len();
    Ok(Json(UserActivityResponse { entries, total }))
}

#[utoipa::path(
    get,
    path = "/analytics/search",
    params(DateRangeQuery),
    responses(
        (status = 200, description = "Search queries per day", body = SearchActivityResponse),
        (status = 500, description = "Internal server error"),
    ),
    tag = "analytics",
    security(("bearer_auth" = [])),
)]
pub async fn get_search_activity(
    State(state): State<AnalyticsState>,
    Query(query): Query<DateRangeQuery>,
) -> Result<Json<SearchActivityResponse>, (StatusCode, String)> {
    let days = query.days.unwrap_or(30);
    let start_date = query.start_date.unwrap_or_else(|| {
        (chrono::Utc::now() - chrono::Duration::days(days as i64))
            .format("%Y-%m-%d")
            .to_string()
    });
    let end_date = query.end_date.unwrap_or_else(|| {
        chrono::Utc::now().format("%Y-%m-%d").to_string()
    });

    let rows: Vec<(String, i64)> = sqlx::query_as(
        r#"
        WITH date_series AS (
            SELECT generate_series(
                $1::date,
                $2::date,
                '1 day'::interval
            )::date AS day
        ),
        searches AS (
            SELECT DATE(created_at) AS day, COUNT(*)::bigint AS cnt
            FROM saved_searches
            WHERE created_at >= $1::date AND created_at <= ($2::date + interval '1 day')
            GROUP BY DATE(created_at)
        )
        SELECT
            ds.day::text,
            COALESCE(searches.cnt, 0)
        FROM date_series ds
        LEFT JOIN searches ON searches.day = ds.day
        ORDER BY ds.day
        "#,
    )
    .bind(&start_date)
    .bind(&end_date)
    .fetch_all(state.pool.inner())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let entries: Vec<DailySearchCount> = rows
        .into_iter()
        .map(|(date, query_count)| DailySearchCount { date, query_count })
        .collect();

    let total = entries.len();
    Ok(Json(SearchActivityResponse { entries, total }))
}

#[utoipa::path(
    get,
    path = "/analytics/api",
    params(DateRangeQuery),
    responses(
        (status = 200, description = "API request volume per day", body = ApiActivityResponse),
        (status = 500, description = "Internal server error"),
    ),
    tag = "analytics",
    security(("bearer_auth" = [])),
)]
pub async fn get_api_activity(
    State(state): State<AnalyticsState>,
    Query(query): Query<DateRangeQuery>,
) -> Result<Json<ApiActivityResponse>, (StatusCode, String)> {
    let days = query.days.unwrap_or(30);
    let start_date = query.start_date.unwrap_or_else(|| {
        (chrono::Utc::now() - chrono::Duration::days(days as i64))
            .format("%Y-%m-%d")
            .to_string()
    });
    let end_date = query.end_date.unwrap_or_else(|| {
        chrono::Utc::now().format("%Y-%m-%d").to_string()
    });

    let rows: Vec<(String, i64, i64, i64)> = sqlx::query_as(
        r#"
        WITH date_series AS (
            SELECT generate_series(
                $1::date,
                $2::date,
                '1 day'::interval
            )::date AS day
        ),
        requests AS (
            SELECT DATE(created_at) AS day, COUNT(*)::bigint AS total,
                   SUM(CASE WHEN event_type != 'error' THEN 1 ELSE 0 END)::bigint AS successful,
                   SUM(CASE WHEN event_type = 'error' THEN 1 ELSE 0 END)::bigint AS failed
            FROM activity_events
            WHERE created_at >= $1::date AND created_at <= ($2::date + interval '1 day')
            GROUP BY DATE(created_at)
        )
        SELECT
            ds.day::text,
            COALESCE(requests.total, 0),
            COALESCE(requests.successful, 0),
            COALESCE(requests.failed, 0)
        FROM date_series ds
        LEFT JOIN requests ON requests.day = ds.day
        ORDER BY ds.day
        "#,
    )
    .bind(&start_date)
    .bind(&end_date)
    .fetch_all(state.pool.inner())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let entries: Vec<ApiRequestVolume> = rows
        .into_iter()
        .map(|(date, total_requests, successful, failed)| ApiRequestVolume {
            date,
            total_requests,
            successful,
            failed,
        })
        .collect();

    let total = entries.len();
    Ok(Json(ApiActivityResponse { entries, total }))
}

pub fn create_analytics_router() -> axum::Router<AnalyticsState> {
    axum::Router::new()
        .route("/analytics/overview", get(get_overview))
        .route("/analytics/activity", get(get_document_activity))
        .route("/analytics/users", get(get_user_activity))
        .route("/analytics/search", get(get_search_activity))
        .route("/analytics/api", get(get_api_activity))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_date_range_query_default() {
        let json = r#"{}"#;
        let query: DateRangeQuery = serde_json::from_str(json).unwrap();
        assert!(query.start_date.is_none());
        assert!(query.end_date.is_none());
        assert!(query.granularity.is_none());
    }

    #[test]
    fn test_date_range_query_with_values() {
        let json = r#"{"start_date": "2025-01-01", "end_date": "2025-01-31", "granularity": "day"}"#;
        let query: DateRangeQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.start_date.as_deref(), Some("2025-01-01"));
        assert_eq!(query.end_date.as_deref(), Some("2025-01-31"));
        assert_eq!(query.granularity.as_deref(), Some("day"));
    }

    #[test]
    fn test_analytics_overview_serialization() {
        let overview = AnalyticsOverview {
            total_documents: 42,
            total_users: 5,
            storage_bytes: 1024000,
            active_spaces: 3,
        };
        let json = serde_json::to_string(&overview).unwrap();
        assert!(json.contains("\"total_documents\":42"));
        assert!(json.contains("\"total_users\":5"));
    }

    #[test]
    fn test_daily_activity_response_serialization() {
        let response = DailyActivityResponse {
            entries: vec![
                DailyActivity {
                    date: "2025-01-01".to_string(),
                    created: 5,
                    updated: 10,
                    deleted: 1,
                },
            ],
            total: 1,
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"created\":5"));
        assert!(json.contains("\"updated\":10"));
        assert!(json.contains("\"deleted\":1"));
    }

    #[test]
    fn test_daily_user_activity_serialization() {
        let entry = DailyUserActivity {
            date: "2025-01-01".to_string(),
            active_users: 8,
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"active_users\":8"));
    }

    #[test]
    fn test_api_request_volume_serialization() {
        let entry = ApiRequestVolume {
            date: "2025-01-01".to_string(),
            total_requests: 150,
            successful: 145,
            failed: 5,
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"total_requests\":150"));
        assert!(json.contains("\"successful\":145"));
        assert!(json.contains("\"failed\":5"));
    }
}
