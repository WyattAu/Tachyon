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

#[derive(Serialize, utoipa::ToSchema)]
pub struct ReadinessResponse {
    pub status: String,
    pub checks: ReadinessChecks,
    pub timestamp: String,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct ReadinessChecks {
    pub database: String,
    pub smtp: String,
    pub redis: String,
}

fn check_smtp_readiness(smtp_configured: bool) -> String {
    if !smtp_configured {
        return "not_configured".to_string();
    }

    match std::env::var("SMTP_URL") {
        Ok(url) => match url.parse::<url::Url>() {
            Ok(parsed) => {
                if parsed.scheme() == "smtp" || parsed.scheme() == "smtps" || parsed.scheme() == "starttls" {
                    "ok".to_string()
                } else {
                    format!("error: invalid SMTP scheme '{}'", parsed.scheme())
                }
            }
            Err(e) => format!("error: unparseable SMTP URL: {}", e),
        },
        Err(_) => {
            match std::env::var("SMTP_HOST") {
                Ok(_) => "ok".to_string(),
                Err(_) => "not_configured".to_string(),
            }
        }
    }
}

fn check_redis_readiness(redis_enabled: bool, redis_url: Option<&str>) -> String {
    if !redis_enabled {
        return "not_configured".to_string();
    }

    match redis_url {
        Some(url) => match url.parse::<url::Url>() {
            Ok(parsed) => {
                if parsed.scheme() == "redis" || parsed.scheme() == "rediss" {
                    "ok".to_string()
                } else {
                    format!("error: invalid Redis scheme '{}'", parsed.scheme())
                }
            }
            Err(e) => format!("error: unparseable Redis URL: {}", e),
        },
        None => "not_configured".to_string(),
    }
}

async fn check_database_readiness(pool: &tachyon_database::DatabasePool) -> String {
    match pool.execute("SELECT 1").await {
        Ok(_) => "ok".to_string(),
        Err(e) => format!("error: {}", e),
    }
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
        (status = 200, description = "Service is ready", body = ReadinessResponse),
        (status = 503, description = "Database unreachable"),
    ),
    tag = "system",
)]
pub(crate) async fn readiness_check(
    State(state): State<crate::HealthState>,
) -> (StatusCode, Json<ReadinessResponse>) {
    let db_status = check_database_readiness(&state.pool).await;
    let smtp_status = check_smtp_readiness(state.smtp_configured);
    let redis_status = check_redis_readiness(state.redis_enabled, state.redis_url.as_deref());

    let overall = if db_status == "ok"
        && (smtp_status == "ok" || smtp_status == "not_configured")
        && (redis_status == "ok" || redis_status == "not_configured")
    {
        "ok"
    } else if db_status == "ok" {
        "degraded"
    } else {
        "error"
    };

    let status_code = if overall == "ok" {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    let response = ReadinessResponse {
        status: overall.to_string(),
        checks: ReadinessChecks {
            database: db_status,
            smtp: smtp_status,
            redis: redis_status,
        },
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    (status_code, Json(response))
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

    #[test]
    fn test_readiness_response_structure() {
        let response = ReadinessResponse {
            status: "ok".to_string(),
            checks: ReadinessChecks {
                database: "ok".to_string(),
                smtp: "not_configured".to_string(),
                redis: "ok".to_string(),
            },
            timestamp: "2025-01-01T00:00:00Z".to_string(),
        };
        assert_eq!(response.status, "ok");
        assert_eq!(response.checks.smtp, "not_configured");
        assert_eq!(response.checks.redis, "ok");
        assert_eq!(response.checks.database, "ok");
        assert!(!response.timestamp.is_empty());
    }

    #[test]
    fn test_readiness_response_serialization() {
        let response = ReadinessResponse {
            status: "ok".to_string(),
            checks: ReadinessChecks {
                database: "ok".to_string(),
                smtp: "not_configured".to_string(),
                redis: "ok".to_string(),
            },
            timestamp: "2025-01-01T00:00:00Z".to_string(),
        };
        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["status"], "ok");
        assert_eq!(json["checks"]["database"], "ok");
        assert_eq!(json["checks"]["smtp"], "not_configured");
        assert_eq!(json["checks"]["redis"], "ok");
        assert_eq!(json["timestamp"], "2025-01-01T00:00:00Z");
    }

    #[test]
    fn test_smtp_readiness_not_configured() {
        assert_eq!(check_smtp_readiness(false), "not_configured");
    }

    #[test]
    fn test_redis_readiness_not_configured() {
        assert_eq!(check_redis_readiness(false, None), "not_configured");
    }

    #[test]
    fn test_smtp_readiness_valid_url() {
        std::env::set_var("SMTP_URL", "smtps://smtp.example.com:465");
        std::env::set_var("SMTP_HOST", "smtp.example.com");
        assert_eq!(check_smtp_readiness(true), "ok");
        std::env::remove_var("SMTP_URL");
        std::env::remove_var("SMTP_HOST");
    }

    #[test]
    fn test_smtp_readiness_invalid_url() {
        std::env::set_var("SMTP_URL", "not-a-valid-url");
        std::env::remove_var("SMTP_HOST");
        let result = check_smtp_readiness(true);
        assert!(result.starts_with("error:"));
    }

    #[test]
    fn test_redis_readiness_valid_url() {
        assert_eq!(check_redis_readiness(true, Some("redis://localhost:6379")), "ok");
    }

    #[test]
    fn test_redis_readiness_invalid_url() {
        let result = check_redis_readiness(true, Some("not-a-url"));
        assert!(result.starts_with("error:"));
    }

    #[test]
    fn test_redis_readiness_invalid_scheme() {
        let result = check_redis_readiness(true, Some("http://localhost:6379"));
        assert!(result.contains("invalid Redis scheme"));
    }
}
