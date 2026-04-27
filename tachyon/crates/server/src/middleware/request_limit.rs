use axum::{
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Response},
    http::StatusCode,
};

pub async fn request_size_limit(
    request: Request,
    next: Next,
) -> Response {
    const MAX_SIZE: usize = 10 * 1024 * 1024;

    if let Some(content_length) = request.headers().get("content-length") {
        if let Ok(length_str) = content_length.to_str() {
            if let Ok(length) = length_str.parse::<usize>() {
                if length > MAX_SIZE {
                    return (
                        StatusCode::PAYLOAD_TOO_LARGE,
                        axum::Json(serde_json::json!({
                            "error": "request_too_large",
                            "message": format!("Request body exceeds {} bytes", MAX_SIZE)
                        })),
                    ).into_response();
                }
            }
        }
    }

    next.run(request).await
}
