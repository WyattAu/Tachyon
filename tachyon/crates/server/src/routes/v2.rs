//! API v2 route placeholders.
//! Currently mirrors v1 with version headers.

use axum::{http::StatusCode, response::Json, routing::get};
use serde_json::{Value, json};

/// Health check for v2 API.
pub async fn v2_health() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "version": "2.0.0",
        "message": "Tachyon API v2"
    }))
}

/// 404 handler for unimplemented v2 endpoints.
pub async fn v2_not_implemented() -> (StatusCode, Json<Value>) {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({
            "error": "not_implemented",
            "message": "This endpoint is not yet available in API v2"
        })),
    )
}

pub fn v2_routes() -> axum::Router<()> {
    axum::Router::new().route("/health", get(v2_health))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_v2_health_response() {
        let Json(body) = v2_health().await;
        assert_eq!(body["status"], "ok");
        assert_eq!(body["version"], "2.0.0");
        assert_eq!(body["message"], "Tachyon API v2");
    }

    #[tokio::test]
    async fn test_v2_not_implemented() {
        let (status, Json(body)) = v2_not_implemented().await;
        assert_eq!(status, StatusCode::NOT_IMPLEMENTED);
        assert_eq!(body["error"], "not_implemented");
    }
}
