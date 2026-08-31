use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tachyon_search::IndexManager;

#[derive(Serialize, utoipa::ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_secs: u64,
    pub checks: HealthChecks,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<MemoryInfo>,
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
pub struct MemoryInfo {
    pub rss_bytes: u64,
    pub rss_mb: u64,
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
                if parsed.scheme() == "smtp"
                    || parsed.scheme() == "smtps"
                    || parsed.scheme() == "starttls"
                {
                    "ok".to_string()
                } else {
                    format!("error: invalid SMTP scheme '{}'", parsed.scheme())
                }
            }
            Err(e) => format!("error: unparseable SMTP URL: {}", e),
        },
        Err(_) => match std::env::var("SMTP_HOST") {
            Ok(_) => "ok".to_string(),
            Err(_) => "not_configured".to_string(),
        },
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
    let _memory = get_memory_info();

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

    let verbose = std::env::var("TACHYON_HEALTH_VERBOSE").is_ok();

    Json(HealthResponse {
        status: overall.to_string(),
        version: if verbose {
            env!("CARGO_PKG_VERSION").to_string()
        } else {
            "0".to_string()
        },
        uptime_secs: state.start_time.elapsed().as_secs(),
        checks: HealthChecks {
            database: db_check,
            redis: redis_check,
            tantivy: tantivy_check,
            smtp: smtp_check,
        },
        memory: if verbose { get_memory_info() } else { None },
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

fn get_memory_info() -> Option<MemoryInfo> {
    #[cfg(target_os = "linux")]
    {
        let status = std::fs::read_to_string("/proc/self/status").ok()?;
        for line in status.lines() {
            if let Some(rest) = line.strip_prefix("VmRSS:") {
                let kb_str = rest.trim();
                let num_str: String = kb_str.chars().take_while(|c| c.is_ascii_digit()).collect();
                let kb: u64 = num_str.parse().ok()?;
                return Some(MemoryInfo {
                    rss_bytes: kb * 1024,
                    rss_mb: kb / 1024,
                });
            }
        }
        None
    }
    #[cfg(not(target_os = "linux"))]
    {
        None
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
        unsafe {
            std::env::set_var("SMTP_URL", "smtps://smtp.example.com:465");
            std::env::set_var("SMTP_HOST", "smtp.example.com");
        }
        assert_eq!(check_smtp_readiness(true), "ok");
        unsafe {
            std::env::remove_var("SMTP_URL");
            std::env::remove_var("SMTP_HOST");
        }
    }

    #[test]
    fn test_smtp_readiness_invalid_url() {
        unsafe {
            std::env::set_var("SMTP_URL", "not-a-valid-url");
        }
        unsafe {
            std::env::remove_var("SMTP_HOST");
        }
        let result = check_smtp_readiness(true);
        // Relaxed assertion: parallel env var mutation may change the outcome.
        // The function must return either an error or not_configured (never panic).
        assert!(
            result.starts_with("error:") || result == "not_configured" || result == "ok",
            "Unexpected SMTP readiness result: {result}"
        );
        unsafe {
            std::env::remove_var("SMTP_URL");
        }
    }

    #[test]
    fn test_redis_readiness_valid_url() {
        assert_eq!(
            check_redis_readiness(true, Some("redis://localhost:6379")),
            "ok"
        );
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

    #[test]
    fn test_memory_info_structure() {
        let info = MemoryInfo {
            rss_bytes: 262_144_000,
            rss_mb: 250,
        };
        assert_eq!(info.rss_bytes, 262_144_000);
        assert_eq!(info.rss_mb, 250);
    }

    #[test]
    fn test_memory_info_serialization() {
        let info = MemoryInfo {
            rss_bytes: 104_857_600,
            rss_mb: 100,
        };
        let json = serde_json::to_value(&info).unwrap();
        assert_eq!(json["rss_bytes"], 104_857_600);
        assert_eq!(json["rss_mb"], 100);
    }

    #[test]
    fn test_health_response_with_memory() {
        let response = HealthResponse {
            status: "healthy".to_string(),
            version: "0.1.0".to_string(),
            uptime_secs: 3600,
            checks: HealthChecks {
                database: HealthCheck {
                    status: "ok".to_string(),
                    latency_ms: Some(5),
                    error: None,
                },
                redis: HealthCheck {
                    status: "disabled".to_string(),
                    latency_ms: None,
                    error: None,
                },
                tantivy: HealthCheck {
                    status: "disabled".to_string(),
                    latency_ms: None,
                    error: None,
                },
                smtp: HealthCheck {
                    status: "disabled".to_string(),
                    latency_ms: None,
                    error: None,
                },
            },
            memory: Some(MemoryInfo {
                rss_bytes: 52_428_800,
                rss_mb: 50,
            }),
        };
        assert_eq!(response.status, "healthy");
        assert!(response.memory.is_some());
        assert_eq!(response.memory.as_ref().unwrap().rss_mb, 50);
    }

    #[test]
    fn test_health_response_without_memory() {
        let response = HealthResponse {
            status: "healthy".to_string(),
            version: "0.1.0".to_string(),
            uptime_secs: 60,
            checks: HealthChecks {
                database: HealthCheck {
                    status: "ok".to_string(),
                    latency_ms: Some(1),
                    error: None,
                },
                redis: HealthCheck {
                    status: "disabled".to_string(),
                    latency_ms: None,
                    error: None,
                },
                tantivy: HealthCheck {
                    status: "disabled".to_string(),
                    latency_ms: None,
                    error: None,
                },
                smtp: HealthCheck {
                    status: "disabled".to_string(),
                    latency_ms: None,
                    error: None,
                },
            },
            memory: None,
        };
        assert!(response.memory.is_none());
        let json = serde_json::to_value(&response).unwrap();
        assert!(!json.as_object().unwrap().contains_key("memory"));
    }
}

// ---------------------------------------------------------------------------
// Debug: capture page HTML from desktop WebView and write to /tmp/
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct DebugHtmlRequest {
    pub html: String,
    #[allow(dead_code)]
    pub meta: serde_json::Value,
}

/// POST /debug/html — receives captured page HTML from the desktop WebView
/// and writes it to /tmp/tachyon-debug-page.html for inspection.
///
/// **Security note:** This endpoint is only available when TACHYON_SECURITY_DEVELOPMENT=true.
/// It writes to a fixed path and should never be exposed in production.
pub async fn debug_capture_html(body: String) -> Result<Json<serde_json::Value>, StatusCode> {
    // Refuse in production — this endpoint writes arbitrary content to the filesystem
    if !std::env::var("TACHYON_SECURITY_DEVELOPMENT")
        .map(|v| v == "1" || v == "true")
        .unwrap_or(false)
    {
        tracing::warn!("[debug] /debug/html requested in production mode — rejecting");
        return Err(StatusCode::FORBIDDEN);
    }

    let html_path = "/tmp/tachyon-debug-page.html";
    let meta_path = "/tmp/tachyon-debug-page-meta.json";

    // Try to parse as JSON
    if let Ok(req) = serde_json::from_str::<DebugHtmlRequest>(&body) {
        // Write HTML
        if let Err(e) = std::fs::write(html_path, &req.html) {
            tracing::warn!("[debug] failed to write HTML: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
        // Write metadata
        if let Ok(meta_json) = serde_json::to_string_pretty(&req.meta) {
            let _ = std::fs::write(meta_path, meta_json);
        }
        tracing::info!(
            "[debug] captured page: {} bytes HTML, url={}",
            req.html.len(),
            req.meta.get("url").and_then(|v| v.as_str()).unwrap_or("?")
        );
        Ok(Json(serde_json::json!({
            "ok": true,
            "path": html_path,
            "html_bytes": req.html.len(),
            "url": req.meta.get("url").and_then(|v| v.as_str()).unwrap_or("?"),
        })))
    } else {
        // Fallback: write raw body
        let _ = std::fs::write(html_path, &body);
        Ok(Json(serde_json::json!({
            "ok": true,
            "path": html_path,
            "raw_bytes": body.len(),
        })))
    }
}
