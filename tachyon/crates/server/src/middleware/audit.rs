use axum::{extract::Request, middleware::Next, response::Response};
use tracing;

use crate::middleware::auth::AuthContext;
use crate::middleware::request_id::RequestId;

pub async fn audit_middleware(request: Request, next: Next) -> Response {
    let path = request.uri().path().to_string();
    let method = request.method().to_string();

    let skip_paths = ["/health", "/ready", "/metrics", "/metrics/prometheus"];

    if skip_paths.iter().any(|p| path == *p || path.starts_with(p))
        || path.starts_with("/static/")
        || path.starts_with("/assets/")
    {
        return next.run(request).await;
    }

    let auth_context = request.extensions().get::<AuthContext>().cloned();
    let request_id = request
        .extensions()
        .get::<RequestId>()
        .map(|rid| rid.0.clone());

    let ip_address = extract_client_ip(request.headers());
    let resource_type = extract_resource_type(&path);
    let action = format!("{} {}", method, path);

    let response = next.run(request).await;
    let status = response.status().as_u16();

    if let Some(ref ctx) = auth_context {
        tracing::info!(
            target: "audit",
            user_id = %ctx.user_id,
            action = %action,
            resource_type = %resource_type,
            ip_address = %ip_address,
            status = status,
            request_id = ?request_id,
            "Authenticated request"
        );
    } else if status == 401 || status == 403 {
        tracing::warn!(
            target: "audit",
            action = %action,
            resource_type = %resource_type,
            ip_address = %ip_address,
            status = status,
            request_id = ?request_id,
            "Unauthenticated request - auth failure"
        );
    } else {
        tracing::info!(
            target: "audit",
            action = %action,
            resource_type = %resource_type,
            ip_address = %ip_address,
            status = status,
            request_id = ?request_id,
            "Unauthenticated request"
        );
    }

    response
}

fn extract_client_ip(headers: &axum::http::HeaderMap) -> String {
    if let Some(forwarded) = headers.get("x-forwarded-for")
        && let Ok(forwarded_str) = forwarded.to_str()
            && let Some(first_ip) = forwarded_str.split(',').next() {
                return first_ip.trim().to_string();
            }
    if let Some(real_ip) = headers.get("x-real-ip")
        && let Ok(ip_str) = real_ip.to_str() {
            return ip_str.to_string();
        }
    "unknown".to_string()
}

fn extract_resource_type(path: &str) -> String {
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    if segments.len() >= 3 {
        return segments[2].to_string();
    }
    if !segments.is_empty() {
        return segments[0].to_string();
    }
    "unknown".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_resource_type() {
        assert_eq!(extract_resource_type("/api/v1/documents"), "documents");
        assert_eq!(extract_resource_type("/api/v1/users/123"), "users");
        assert_eq!(extract_resource_type("/health"), "health");
        assert_eq!(extract_resource_type("/"), "unknown");
    }

    #[test]
    fn test_extract_client_ip() {
        let mut headers = axum::http::HeaderMap::new();
        headers.insert("x-forwarded-for", "10.0.0.1, 192.168.1.1".parse().unwrap());
        assert_eq!(extract_client_ip(&headers), "10.0.0.1");
    }

    #[test]
    fn test_extract_client_ip_real_ip() {
        let mut headers = axum::http::HeaderMap::new();
        headers.insert("x-real-ip", "10.0.0.2".parse().unwrap());
        assert_eq!(extract_client_ip(&headers), "10.0.0.2");
    }

    #[test]
    fn test_extract_client_ip_unknown() {
        let headers = axum::http::HeaderMap::new();
        assert_eq!(extract_client_ip(&headers), "unknown");
    }
}
