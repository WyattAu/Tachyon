use axum::{extract::Request, http::HeaderValue, middleware::Next, response::Response};
use tracing::info_span;

#[derive(Debug, Clone)]
pub struct RequestId(pub String);

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub async fn request_id_middleware(request: Request, next: Next) -> Response {
    let request_id = request
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let request_id_header =
        HeaderValue::from_str(&request_id).unwrap_or_else(|_| HeaderValue::from_static("unknown"));

    let span = info_span!("http_request", request_id = %request_id);

    let mut request = request;
    request
        .extensions_mut()
        .insert(RequestId(request_id.clone()));

    let response = next.run(request).await;

    let mut response = response;
    response
        .headers_mut()
        .insert("X-Request-Id", request_id_header);

    span.in_scope(|| response)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_request_id_format() {
        let id = uuid::Uuid::new_v4().to_string();
        assert_eq!(id.len(), 36);
        assert_eq!(id.chars().filter(|&c| c == '-').count(), 4);
    }
}
