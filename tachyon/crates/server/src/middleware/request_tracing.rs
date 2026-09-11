use axum::{extract::Request, middleware::Next, response::Response};
use std::sync::Arc;
use std::time::Instant;

#[derive(Clone)]
pub struct RequestTracingState {
    pub metrics: Arc<crate::middleware::metrics::RequestMetrics>,
}

/// Query parameter keys whose values must never appear in logs.
///
/// Checked case-insensitively as substring matches so variants like
/// `api_key`, `sessionId`, `reset_token`, `magic-link` are all covered.
const SENSITIVE_QUERY_KEYS: &[&str] = &[
    "token",
    "api_key",
    "apikey",
    "session",
    "password",
    "passwd",
    "secret",
    "auth",
    "magic",
    "otp",
    "code",
    "sessionid",
];

/// Redact sensitive query-string values before logging.
///
/// Returns `None` when there is no query string; otherwise returns the query
/// with sensitive values replaced by `[REDACTED]`, preserving structure so
/// logs remain useful for debugging (`q=docs&token=abc` →
/// `q=docs&token=[REDACTED]`).
pub fn redact_query(query: Option<&str>) -> Option<String> {
    let raw = query?;
    if raw.is_empty() {
        return None;
    }
    let redacted: Vec<String> = raw
        .split('&')
        .map(|pair| match pair.split_once('=') {
            Some((key, _value)) if SENSITIVE_QUERY_KEYS.iter().any(|s| key.to_lowercase().contains(s)) => {
                format!("{}=[REDACTED]", key)
            }
            _ => pair.to_string(),
        })
        .collect();
    Some(redacted.join("&"))
}

/// Normalize a request path for use as a metric label.
///
/// Dynamic segments (UUIDs, numeric IDs, long opaque tokens) are collapsed so
/// label cardinality stays bounded: `/api/v1/documents/019e...` →
/// `/api/v1/documents/:id`, `/page/123` → `/page/:n`.
fn normalize_route(path: &str) -> String {
    path.split('/')
        .map(|seg| {
            if seg.chars().all(|c| c.is_ascii_digit()) && !seg.is_empty() {
                ":n".to_string()
            } else if is_uuid_like(seg) {
                ":id".to_string()
            } else {
                seg.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("/")
}

/// Heuristic UUID detection (avoids a uuid dependency in middleware).
fn is_uuid_like(seg: &str) -> bool {
    seg.len() == 36
        && seg.chars().filter(|&c| c == '-').count() == 4
        && seg.chars().all(|c| c.is_ascii_hexdigit() || c == '-')
}

pub async fn request_logging_middleware(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let query = redact_query(request.uri().query());
    let user_agent = request
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();
    let client_ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next().map(|s| s.trim().to_string()))
        .or_else(|| {
            request
                .headers()
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "unknown".to_string());

    let start = Instant::now();
    let response = next.run(request).await;
    let duration = start.elapsed();

    let status = response.status().as_u16();
    let response_request_id = response
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("?");

    tracing::info!(
        request_id = %response_request_id,
        method = %method,
        path = %path,
        query = ?query,
        status = status,
        duration_ms = duration.as_millis() as u64,
        client_ip = %client_ip,
        user_agent = %user_agent,
        "request completed"
    );

    if duration.as_secs() >= 1 {
        tracing::warn!(
            request_id = %response_request_id,
            method = %method,
            path = %path,
            duration_ms = duration.as_millis() as u64,
            "slow request detected"
        );
    }

    if status >= 500 {
        tracing::error!(
            request_id = %response_request_id,
            method = %method,
            path = %path,
            status = status,
            duration_ms = duration.as_millis() as u64,
            "server error"
        );
    } else if status >= 400 {
        tracing::warn!(
            request_id = %response_request_id,
            method = %method,
            path = %path,
            status = status,
            "client error"
        );
    }

    response
}

pub async fn request_logging_with_metrics(
    axum::extract::State(state): axum::extract::State<RequestTracingState>,
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let query = redact_query(request.uri().query());
    let route = normalize_route(&path);
    let user_agent = request
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();
    let client_ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next().map(|s| s.trim().to_string()))
        .or_else(|| {
            request
                .headers()
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "unknown".to_string());

    let start = Instant::now();
    let response = next.run(request).await;
    let duration = start.elapsed();

    let status = response.status().as_u16();
    let response_request_id = response
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("?");

    state
        .metrics
        .record_request(duration.as_millis() as u64, status, method.as_str(), &route);

    tracing::info!(
        request_id = %response_request_id,
        method = %method,
        path = %path,
        query = ?query,
        status = status,
        duration_ms = duration.as_millis() as u64,
        client_ip = %client_ip,
        user_agent = %user_agent,
        "request completed"
    );

    if duration.as_secs() >= 1 {
        tracing::warn!(
            request_id = %response_request_id,
            method = %method,
            path = %path,
            duration_ms = duration.as_millis() as u64,
            "slow request detected"
        );
    }

    if status >= 500 {
        tracing::error!(
            request_id = %response_request_id,
            method = %method,
            path = %path,
            status = status,
            duration_ms = duration.as_millis() as u64,
            "server error"
        );
    } else if status >= 400 {
        tracing::warn!(
            request_id = %response_request_id,
            method = %method,
            path = %path,
            status = status,
            "client error"
        );
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_query_none_and_empty() {
        assert_eq!(redact_query(None), None);
        assert_eq!(redact_query(Some("")), None);
    }

    #[test]
    fn test_redact_query_preserves_benign_params() {
        assert_eq!(
            redact_query(Some("q=docs&page=2")),
            Some("q=docs&page=2".to_string())
        );
    }

    #[test]
    fn test_redact_query_masks_tokens() {
        assert_eq!(
            redact_query(Some("q=docs&token=abc123")),
            Some("q=docs&token=[REDACTED]".to_string())
        );
        assert_eq!(
            redact_query(Some("api_key=xyz&sessionId=s1&password=hunter2&q=x")),
            Some("api_key=[REDACTED]&sessionId=[REDACTED]&password=[REDACTED]&q=x".to_string())
        );
    }

    #[test]
    fn test_redact_query_case_insensitive() {
        assert_eq!(
            redact_query(Some("Reset_Token=abc")),
            Some("Reset_Token=[REDACTED]".to_string())
        );
    }

    #[test]
    fn test_redact_query_flag_without_value() {
        assert_eq!(
            redact_query(Some("verbose&token")),
            Some("verbose&token".to_string())
        );
    }

    #[test]
    fn test_normalize_route_static() {
        assert_eq!(normalize_route("/health"), "/health");
        assert_eq!(normalize_route("/api/v1/documents"), "/api/v1/documents");
        // v1 must NOT collapse — only UUID/numeric segments do
        assert_eq!(normalize_route("/api/v2/search"), "/api/v2/search");
    }

    #[test]
    fn test_normalize_route_dynamic_segments() {
        assert_eq!(
            normalize_route("/api/v1/documents/019ecb40-776a-7d01-991c-180ba31c9406"),
            "/api/v1/documents/:id"
        );
        assert_eq!(normalize_route("/ws/rooms/123"), "/ws/rooms/:n");
    }

    #[test]
    fn test_normalize_route_non_uuid_kept() {
        assert_eq!(normalize_route("/documents/my-notes"), "/documents/my-notes");
    }
}
