use axum::{Json, extract::State};
use serde::Serialize;
use std::sync::Arc;

#[derive(Clone)]
pub struct QueryStatsState {
    pub pool: tachyon_database::DatabasePool,
    pub query_logger: Arc<tachyon_database::SlowQueryLogger>,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct QueryStatsResponse {
    pub pool: tachyon_database::PoolMetrics,
    pub slow_query_count: u64,
    pub total_queries: u64,
    pub average_query_time_ms: f64,
    pub slow_query_threshold_ms: u64,
}

#[utoipa::path(
    get,
    path = "/admin/query-stats",
    responses(
        (status = 200, description = "Query statistics", body = QueryStatsResponse),
    ),
    tag = "admin",
    security(("bearer_auth" = [])),
)]
pub async fn query_stats_handler(State(state): State<QueryStatsState>) -> Json<QueryStatsResponse> {
    let pool = state.pool.pool_metrics();
    Json(QueryStatsResponse {
        pool,
        slow_query_count: state.query_logger.slow_count(),
        total_queries: state.query_logger.total_queries(),
        average_query_time_ms: state.query_logger.average_query_time_ms(),
        slow_query_threshold_ms: state.query_logger.threshold_ms(),
    })
}

pub fn create_query_stats_router() -> axum::Router<QueryStatsState> {
    axum::Router::new().route(
        "/admin/query-stats",
        axum::routing::get(query_stats_handler),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_stats_response_serialization() {
        let metrics = tachyon_database::PoolMetrics {
            size: 5,
            idle: 3,
            active: 2,
            max_connections: 10,
        };
        let response = QueryStatsResponse {
            pool: metrics,
            slow_query_count: 3,
            total_queries: 100,
            average_query_time_ms: 12.5,
            slow_query_threshold_ms: 100,
        };
        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["pool"]["size"], 5);
        assert_eq!(json["pool"]["idle"], 3);
        assert_eq!(json["pool"]["active"], 2);
        assert_eq!(json["pool"]["max_connections"], 10);
        assert_eq!(json["slow_query_count"], 3);
        assert_eq!(json["total_queries"], 100);
        assert_eq!(json["average_query_time_ms"], 12.5);
        assert_eq!(json["slow_query_threshold_ms"], 100);
    }

    #[test]
    fn test_pool_metrics_fields() {
        let metrics = tachyon_database::PoolMetrics {
            size: 10,
            idle: 7,
            active: 3,
            max_connections: 20,
        };
        assert_eq!(metrics.active, metrics.size - metrics.idle);
    }
}
