use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::Serialize;
use std::time::Instant;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_secs: u64,
    pub checks: HealthChecks,
}

#[derive(Serialize)]
pub struct HealthChecks {
    pub database: HealthCheck,
    pub redis: HealthCheck,
}

#[derive(Serialize)]
pub struct HealthCheck {
    pub status: String,
    pub latency_ms: Option<u64>,
    pub error: Option<String>,
}

pub(crate) async fn health_check(State(state): State<crate::HealthState>) -> Json<HealthResponse> {
    let _start = Instant::now();

    let db_check = check_database(&state.pool).await;
    let redis_check = check_redis(&state.redis_enabled).await;

    Json(HealthResponse {
        status: if db_check.status == "ok" && (redis_check.status == "ok" || redis_check.status == "disabled") {
            "healthy"
        } else {
            "degraded"
        }.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_secs: state.start_time.elapsed().as_secs(),
        checks: HealthChecks {
            database: db_check,
            redis: redis_check,
        },
    })
}

pub(crate) async fn readiness_check(State(state): State<crate::HealthState>) -> (StatusCode, Json<serde_json::Value>) {
    let db_ok = state.pool.execute("SELECT 1").await.is_ok();

    if db_ok {
        (StatusCode::OK, Json(serde_json::json!({"status": "ready"})))
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({"status": "not ready", "error": "database unreachable"})))
    }
}

async fn check_database(pool: &tachyon_database::DatabasePool) -> HealthCheck {
    let start = Instant::now();
    match pool.execute("SELECT 1").await {
        Ok(_) => HealthCheck {
            status: "ok".to_string(),
            latency_ms: Some(start.elapsed().as_millis() as u64),
            error: None,
        },
        Err(e) => HealthCheck {
            status: "error".to_string(),
            latency_ms: Some(start.elapsed().as_millis() as u64),
            error: Some(e.to_string()),
        },
    }
}

async fn check_redis(redis_enabled: &bool) -> HealthCheck {
    if *redis_enabled {
        HealthCheck {
            status: "ok".to_string(),
            latency_ms: None,
            error: None,
        }
    } else {
        HealthCheck {
            status: "disabled".to_string(),
            latency_ms: None,
            error: None,
        }
    }
}
