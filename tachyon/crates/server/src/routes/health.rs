use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;
use std::time::Instant;
use tachyon_search::IndexManager;

#[derive(Serialize, utoipa::ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_secs: u64,
    pub checks: HealthChecks,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct HealthChecks {
    pub database: HealthCheck,
    pub redis: HealthCheck,
    pub tantivy: HealthCheck,
    pub smtp: HealthCheck,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct HealthCheck {
    pub status: String,
    pub latency_ms: Option<u64>,
    pub error: Option<String>,
}

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Health check result", body = HealthResponse),
    ),
    tag = "system",
)]
pub(crate) async fn health_check(State(state): State<crate::HealthState>) -> Json<HealthResponse> {
    let db_check = check_database(&state.pool).await;
    let redis_check = check_redis(&state).await;
    let tantivy_check = check_tantivy().await;
    let smtp_check = check_smtp(&state).await;

    let overall = if db_check.status == "ok"
        && (redis_check.status == "ok" || redis_check.status == "disabled")
        && (tantivy_check.status == "ok" || tantivy_check.status == "disabled")
        && (smtp_check.status == "ok" || smtp_check.status == "disabled")
    {
        "healthy"
    } else if db_check.status == "ok" {
        "degraded"
    } else {
        "unhealthy"
    };

    Json(HealthResponse {
        status: overall.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_secs: state.start_time.elapsed().as_secs(),
        checks: HealthChecks {
            database: db_check,
            redis: redis_check,
            tantivy: tantivy_check,
            smtp: smtp_check,
        },
    })
}

#[utoipa::path(
    get,
    path = "/ready",
    responses(
        (status = 200, description = "Service is ready"),
        (status = 503, description = "Database unreachable"),
    ),
    tag = "system",
)]
pub(crate) async fn readiness_check(
    State(state): State<crate::HealthState>,
) -> (StatusCode, Json<serde_json::Value>) {
    let db_ok = state.pool.execute("SELECT 1").await.is_ok();

    if db_ok {
        (StatusCode::OK, Json(serde_json::json!({"status": "ready"})))
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"status": "not ready", "error": "database unreachable"})),
        )
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

async fn check_redis(state: &crate::HealthState) -> HealthCheck {
    if !state.redis_enabled {
        return HealthCheck {
            status: "disabled".to_string(),
            latency_ms: None,
            error: None,
        };
    }

    // Extract host:port from redis URL (format: redis://host:port or redis://:password@host:port)
    let target = state
        .redis_url
        .as_deref()
        .and_then(extract_host_port)
        .unwrap_or_else(|| "127.0.0.1:6379".to_string());

    let start = Instant::now();
    match tokio::net::TcpStream::connect(&target).await {
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

fn extract_host_port(redis_url: &str) -> Option<String> {
    // Parse redis://[[username:]password@]host:port[/database]
    let without_scheme = redis_url.strip_prefix("redis://")?;
    let host_port_part = if let Some(at_pos) = without_scheme.rfind('@') {
        &without_scheme[at_pos + 1..]
    } else {
        without_scheme
    };
    let host_port = host_port_part.split('/').next()?;
    if host_port.contains(':') {
        Some(host_port.to_string())
    } else {
        Some(format!("{}:6379", host_port))
    }
}

async fn check_tantivy() -> HealthCheck {
    let index_path = std::path::Path::new(".tachyon/search_index");

    if !index_path.exists() {
        return HealthCheck {
            status: "disabled".to_string(),
            latency_ms: None,
            error: None,
        };
    }

    let start = Instant::now();
    match IndexManager::open(index_path.to_path_buf()).await {
        Ok(_mgr) => {
            let latency = start.elapsed().as_millis() as u64;
            HealthCheck {
                status: "ok".to_string(),
                latency_ms: Some(latency),
                error: None,
            }
        }
        Err(e) => HealthCheck {
            status: "error".to_string(),
            latency_ms: Some(start.elapsed().as_millis() as u64),
            error: Some(e.to_string()),
        },
    }
}

async fn check_smtp(state: &crate::HealthState) -> HealthCheck {
    if !state.smtp_configured {
        return HealthCheck {
            status: "disabled".to_string(),
            latency_ms: None,
            error: None,
        };
    }

    let host = std::env::var("SMTP_HOST").unwrap_or_else(|_| "localhost".to_string());
    let target = format!("{}:25", host);
    let start = Instant::now();

    match tokio::net::TcpStream::connect(&target).await {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_check_structure() {
        let check = HealthCheck {
            status: "ok".to_string(),
            latency_ms: Some(5),
            error: None,
        };
        assert_eq!(check.status, "ok");
        assert_eq!(check.latency_ms, Some(5));
        assert!(check.error.is_none());
    }

    #[test]
    fn test_extract_host_port() {
        assert_eq!(
            extract_host_port("redis://localhost:6379"),
            Some("localhost:6379".to_string())
        );
        assert_eq!(
            extract_host_port("redis://:password@redis.example.com:6380/0"),
            Some("redis.example.com:6380".to_string())
        );
        assert_eq!(
            extract_host_port("redis://localhost"),
            Some("localhost:6379".to_string())
        );
        assert_eq!(extract_host_port("not-redis://x"), None);
    }
}
