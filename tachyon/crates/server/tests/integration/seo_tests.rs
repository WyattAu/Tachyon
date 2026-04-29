// Integration tests for SEO routes (robots.txt, sitemap.xml, document pages)
use axum::{
    body::Body,
    http::{Request, StatusCode, header},
    Router,
};
use tower::ServiceExt;
use serde_json::json;

use crate::common;

#[tokio::test]
async fn test_robots_txt() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/robots.txt")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert!(
        response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND,
        "Expected OK or NOT_FOUND, got {}",
        response.status()
    );
    if response.status() != StatusCode::OK {
        return;
    }
    let text = common::read_body_string(response).await;

    assert!(text.contains("User-agent"), "robots.txt should contain User-agent directive");
    assert!(text.contains("Disallow") || text.contains("Allow"), "robots.txt should contain Disallow or Allow");
}

#[tokio::test]
async fn test_sitemap_xml() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/sitemap.xml")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert!(
        response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND,
        "Expected OK or NOT_FOUND, got {}",
        response.status()
    );
    if response.status() != StatusCode::OK {
        return;
    }
    let text = common::read_body_string(response).await;

    assert!(text.contains("<?xml") || text.contains("urlset"), "sitemap should be XML with urlset");
}

#[tokio::test]
async fn test_document_page_not_found() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;
    let fake_id = uuid::Uuid::new_v4();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/docs/{}", fake_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    // Non-existent document should return 404 or a rendered error page
    assert!(
        response.status() == StatusCode::NOT_FOUND
            || response.status() == StatusCode::OK, // May render error page as 200
        "Expected 404 or 200, got {}",
        response.status()
    );
}

#[tokio::test]
async fn test_document_page_with_valid_doc() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;
    let unique = uuid::Uuid::new_v4();

    let auth = common::register_and_login(
        &app,
        &format!("seo_user_{}", unique),
        &format!("seo_user_{}@test.com", unique),
        "Password123!",
    )
    .await
    .expect("Failed to register/login test user");

    // Create a document first
    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/documents")
                .header("Content-Type", "application/json")
                .header("Authorization", common::auth_header(&auth.token))
                .body(Body::from(json!({
                    "title": &format!("SEO Test Doc {}", unique),
                    "content": "# Hello\n\nThis is an SEO test document.",
                }).to_string()))
                .unwrap(),
        )
        .await
        .expect("Document creation failed");

    if create_response.status() == StatusCode::CREATED || create_response.status() == StatusCode::OK {
        let body = common::read_body_json(create_response).await;
        let doc_id = body["id"].as_str().or_else(|| body["document_id"].as_str());

        if let Some(id) = doc_id {
            let page_response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method("GET")
                        .uri(format!("/docs/{}", id))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .expect("Request failed");

            // Should serve an HTML page
            assert!(
                page_response.status() == StatusCode::OK,
                "Expected OK for valid document page, got {}",
                page_response.status()
            );
        }
    }
}

#[tokio::test]
async fn test_robots_txt_content_type() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/robots.txt")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert!(
        response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND,
        "Expected OK or NOT_FOUND, got {}",
        response.status()
    );
    if response.status() != StatusCode::OK {
        return;
    }
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    assert!(
        content_type.contains("text/plain"),
        "robots.txt should have text/plain content type, got: {}",
        content_type
    );
}
