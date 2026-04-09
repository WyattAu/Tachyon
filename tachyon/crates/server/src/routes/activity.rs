use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    routing::get,
};
use serde::{Deserialize, Serialize};
use tachyon_database::{ActivityEvent, ActivityRepository, CreateActivityEvent, DatabasePool};

#[derive(Clone)]
pub struct ActivityState {
    pub pool: DatabasePool,
}

impl ActivityState {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListActivityQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub target_type: Option<String>,
    pub target_id: Option<String>,
    pub actor_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ActivityListResponse {
    pub events: Vec<ActivityEvent>,
    pub count: usize,
}

pub async fn list_activity(
    State(state): State<ActivityState>,
    Query(query): Query<ListActivityQuery>,
) -> Result<Json<ActivityListResponse>, (StatusCode, String)> {
    let limit = query.limit.unwrap_or(50).min(100);
    let offset = query.offset.unwrap_or(0);

    let events = if let (Some(target_type), Some(target_id)) = (&query.target_type, &query.target_id) {
        let target_uuid = uuid::Uuid::parse_str(target_id)
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid target_id: {}", e)))?;
        ActivityRepository::list_by_target(&state.pool, target_type, target_uuid, limit)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    } else if let Some(actor_id) = &query.actor_id {
        let actor_uuid = uuid::Uuid::parse_str(actor_id)
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid actor_id: {}", e)))?;
        ActivityRepository::list_by_actor(&state.pool, actor_uuid, limit)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    } else {
        ActivityRepository::list_recent(&state.pool, limit, offset)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    };

    let count = events.len();
    Ok(Json(ActivityListResponse { events, count }))
}

pub async fn create_activity(
    State(state): State<ActivityState>,
    Json(event): Json<CreateActivityEvent>,
) -> Result<Json<ActivityEvent>, (StatusCode, String)> {
    let created = ActivityRepository::create(&state.pool, event)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(created))
}

pub fn create_activity_router() -> axum::Router<ActivityState> {
    axum::Router::new()
        .route("/activity", get(list_activity).post(create_activity))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_activity_query_deserialization() {
        let json = r#"{"limit": 25, "offset": 10, "actor_id": "550e8400-e29b-41d4-a716-446655440000"}"#;
        let query: ListActivityQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.limit, Some(25));
        assert_eq!(query.offset, Some(10));
        assert_eq!(query.actor_id.as_deref(), Some("550e8400-e29b-41d4-a716-446655440000"));
        assert!(query.target_type.is_none());
    }

    #[test]
    fn test_activity_list_response_construction() {
        let response = ActivityListResponse {
            events: vec![],
            count: 0,
        };
        assert_eq!(response.count, 0);
        assert!(response.events.is_empty());

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"count\":0"));
        assert!(json.contains("\"events\":[]"));
    }
}
