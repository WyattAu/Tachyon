use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use serde_json::json;
use tachyon_server::routes::create_router;
use tower::ServiceExt;

/// Returns true when no PostgreSQL instance is reachable (CI / local dev without Docker).
fn skip_without_db() -> bool {
    std::env::var("DATABASE_URL").is_err() && std::env::var("TEST_DATABASE_URL").is_err()
}

async fn create_test_app() -> Router {
    create_router().await
}

#[tokio::test]
async fn test_health_endpoint() {
    if skip_without_db() {
        return;
    }
    let app = create_test_app().await;

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
async fn test_api_version_endpoint() {
    if skip_without_db() {
        return;
    }
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/version")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_list_documents_endpoint() {
    if skip_without_db() {
        return;
    }
    let app = create_test_app().await;

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
async fn test_create_document_endpoint() {
    if skip_without_db() {
        return;
    }
    let app = create_test_app().await;

    let doc_data = json!({
        "title": "Test Document",
        "slug": "test-document",
        "description": "Test document description",
        "content_type": "Markdown",
        "visibility": "Private",
        "status": "Draft"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/documents")
                .header("Content-Type", "application/json")
                .body(Body::from(doc_data.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status() == StatusCode::CREATED || response.status() == StatusCode::OK);
}

#[tokio::test]
async fn test_get_document_endpoint() {
    if skip_without_db() {
        return;
    }
    let app = create_test_app().await;

    let doc_id = uuid::Uuid::new_v4();

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/documents/{}", doc_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_update_document_endpoint() {
    if skip_without_db() {
        return;
    }
    let app = create_test_app().await;

    let doc_id = uuid::Uuid::new_v4();
    let update_data = json!({
        "title": "Updated Title",
        "description": "Updated description"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/documents/{}", doc_id))
                .header("Content-Type", "application/json")
                .body(Body::from(update_data.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_document_endpoint() {
    if skip_without_db() {
        return;
    }
    let app = create_test_app().await;

    let doc_id = uuid::Uuid::new_v4();

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/documents/{}", doc_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        response.status() == StatusCode::NO_CONTENT || response.status() == StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn test_list_projects_endpoint() {
    if skip_without_db() {
        return;
    }
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/catalog/projects")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_create_project_endpoint() {
    if skip_without_db() {
        return;
    }
    let app = create_test_app().await;

    let project_data = json!({
        "name": "Test Project",
        "slug": "test-project",
        "description": "Test project description",
        "project_type": "service",
        "lifecycle": "development"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/catalog/projects")
                .header("Content-Type", "application/json")
                .body(Body::from(project_data.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status() == StatusCode::CREATED || response.status() == StatusCode::OK);
}

#[tokio::test]
async fn test_404_endpoint() {
    if skip_without_db() {
        return;
    }
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/nonexistent")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_cors_headers() {
    if skip_without_db() {
        return;
    }
    let app = create_test_app().await;

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
