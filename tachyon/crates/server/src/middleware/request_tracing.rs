use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use std::time::Instant;

#[derive(Clone)]
pub struct RequestTracingState {
    pub metrics: Arc<crate::middleware::metrics::RequestMetrics>,
}

pub async fn request_logging_middleware(
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let query = request.uri().query().map(|q| q.to_string());
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
    let query = request.uri().query().map(|q| q.to_string());
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

    state.metrics.record_request(duration.as_millis() as u64, status);

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
