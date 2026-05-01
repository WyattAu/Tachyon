use axum::{extract::State, http::StatusCode, response::Json};
use serde::Serialize;
use sqlx::FromRow;
use tachyon_database::DatabasePool;

#[derive(Clone)]
pub struct TagsState {
    pub pool: DatabasePool,
}

#[derive(Debug, Serialize, FromRow)]
pub struct TagInfo {
    pub tag: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct TagsResponse {
    pub tags: Vec<TagInfo>,
    pub total: usize,
}

/// List all tags with document counts.
///
/// `GET /api/v1/tags`
///
/// Returns up to 100 tags ordered by usage count (descending).
pub async fn list_tags(
    State(state): State<TagsState>,
) -> Result<Json<TagsResponse>, (StatusCode, String)> {
    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let rows: Vec<TagInfo> = sqlx::query_as(
        r#"SELECT value AS tag, COUNT(*) as count
           FROM documents, jsonb_array_elements_text(tags) AS value
           WHERE tags IS NOT NULL AND jsonb_array_length(tags) > 0
           GROUP BY value
           ORDER BY count DESC
           LIMIT 100"#,
    )
    .fetch_all(&mut *conn)
    .await
    .map_err(|e: sqlx::Error| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let total = rows.len();
    Ok(Json(TagsResponse { tags: rows, total }))
}

pub fn create_tags_router() -> axum::Router<TagsState> {
    axum::Router::new().route("/tags", axum::routing::get(list_tags))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_info_serialization() {
        let tag = TagInfo {
            tag: "rust".to_string(),
            count: 42,
        };
        let json = serde_json::to_string(&tag).unwrap();
        assert!(json.contains("rust"));
        assert!(json.contains("42"));
    }

    #[test]
    fn test_tags_response_serialization() {
        let response = TagsResponse {
            tags: vec![
                TagInfo {
                    tag: "rust".to_string(),
                    count: 42,
                },
                TagInfo {
                    tag: "web".to_string(),
                    count: 10,
                },
            ],
            total: 2,
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("rust"));
        assert!(json.contains("web"));
    }
}
