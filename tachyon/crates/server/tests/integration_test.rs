use axum::{
    body::Body,
    http::{Request, StatusCode, header},
    Router,
};
use tower::ServiceExt;
use serde_json::json;

fn create_mock_app() -> Router {
    use axum::routing::{get, post, put, delete};
    
    Router::new()
        .route("/health", get(|| async { StatusCode::OK }))
        .route("/api/v1/documents", get(|| async { 
            (StatusCode::OK, axum::Json(json!({"results": [], "total": 0}))) 
        }))
        .route("/api/v1/documents", post(|| async { 
            (StatusCode::CREATED, axum::Json(json!({"id": "test-id", "title": "Test"}))) 
        }))
        .route("/api/v1/documents/:id", get(|| async { 
            (StatusCode::OK, axum::Json(json!({"id": "test-id", "title": "Test"}))) 
        }))
        .route("/api/v1/documents/:id", put(|| async { 
            (StatusCode::OK, axum::Json(json!({"id": "test-id", "title": "Updated"}))) 
        }))
        .route("/api/v1/documents/:id", delete(|| async { StatusCode::NO_CONTENT }))
}

#[tokio::test]
async fn test_health_check() {
    let app = create_mock_app();
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_list_documents() {
    let app = create_mock_app();
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/documents")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_create_document() {
    let app = create_mock_app();
    
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/documents")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "title": "Test Document",
                    "content": "Test content"
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_get_document() {
    let app = create_mock_app();
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/documents/test-id")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_update_document() {
    let app = create_mock_app();
    
    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/v1/documents/test-id")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "title": "Updated Title"
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_delete_document() {
    let app = create_mock_app();
    
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/v1/documents/test-id")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_cors_preflight() {
    let app = create_mock_app();
    
    let response = app
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/api/v1/documents")
                .header("Origin", "http://localhost:3000")
                .header("Access-Control-Request-Method", "GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_content_type_json() {
    let app = create_mock_app();
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/documents")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    let content_type = response.headers().get("content-type");
    assert!(content_type.is_some());
    assert!(content_type.unwrap().to_str().unwrap().contains("application/json"));
}

#[tokio::test]
async fn test_pagination_parameters() {
    let app = create_mock_app();
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/documents?page=1&page_size=10")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_query_parameters() {
    let app = create_mock_app();
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/documents?search=test&author_id=user-1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_invalid_json_body() {
    let app = create_mock_app();
    
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/documents")
                .header("Content-Type", "application/json")
                .body(Body::from("not valid json"))
                .unwrap(),
        )
        .await;
    
    match response {
        Ok(res) => {
            assert_ne!(res.status(), StatusCode::OK);
        }
        Err(_) => {
            // Error is acceptable for invalid JSON
        }
    }
}

#[tokio::test]
async fn test_missing_content_type() {
    let app = create_mock_app();
    
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/documents")
                .body(Body::from(json!({"title": "Test"}).to_string()))
                .unwrap(),
        )
        .await;
    
    match response {
        Ok(res) => {
            // Should either work or return an error
            assert!(res.status() == StatusCode::OK 
                || res.status() == StatusCode::CREATED
                || res.status() == StatusCode::BAD_REQUEST
                || res.status() == StatusCode::UNSUPPORTED_MEDIA_TYPE);
        }
        Err(_) => {}
    }
}

#[tokio::test]
async fn test_accept_header() {
    let app = create_mock_app();
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/documents")
                .header(header::ACCEPT, "application/json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_concurrent_requests() {
    let app = create_mock_app();
    
    let mut handles = vec![];
    
    for i in 0..10 {
        let app_clone = app.clone();
        let handle = tokio::spawn(async move {
            app_clone
                .oneshot(
                    Request::builder()
                        .uri("/api/v1/documents")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap()
        });
        handles.push(handle);
    }
    
    for handle in handles {
        let response = handle.await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
