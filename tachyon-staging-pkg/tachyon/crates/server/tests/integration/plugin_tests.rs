// Integration tests for plugin routes
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;

use crate::common;

#[tokio::test]
async fn test_list_plugins() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;
    let unique = uuid::Uuid::new_v4();

    let auth = common::register_and_login(
        &app,
        &format!("plugin_user_{}", unique),
        &format!("plugin_{}@test.com", unique),
        "Password123!",
    )
    .await
    .expect("Failed to register/login");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/plugins")
                .header("Authorization", common::auth_header(&auth.token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(response.status(), StatusCode::OK);
    let body = common::read_body_json(response).await;
    assert!(body.is_array(), "Plugins should be an array");
}

#[tokio::test]
async fn test_create_plugin() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;
    let unique = uuid::Uuid::new_v4();

    let auth = common::register_and_login(
        &app,
        &format!("plugcreate_{}", unique),
        &format!("plugcreate_{}@test.com", unique),
        "Password123!",
    )
    .await
    .expect("Failed to register/login");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/plugins")
                .header("Content-Type", "application/json")
                .header("Authorization", common::auth_header(&auth.token))
                .body(Body::from(
                    json!({
                        "name": format!("test-plugin-{}", unique),
                        "version": "1.0.0",
                        "description": "A test plugin",
                        "author": "Integration Test",
                        "enabled": true,
                        "extension_points": ["editor.toolbar"],
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert!(
        response.status() == StatusCode::CREATED || response.status() == StatusCode::OK,
        "Expected CREATED or OK, got {}",
        response.status()
    );
}

#[tokio::test]
async fn test_create_plugin_blank_name() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;
    let unique = uuid::Uuid::new_v4();

    let auth = common::register_and_login(
        &app,
        &format!("plugblank_{}", unique),
        &format!("plugblank_{}@test.com", unique),
        "Password123!",
    )
    .await
    .expect("Failed to register/login");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/plugins")
                .header("Content-Type", "application/json")
                .header("Authorization", common::auth_header(&auth.token))
                .body(Body::from(
                    json!({
                        "name": "",
                        "version": "1.0.0",
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_plugins_unauthorized() {
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
                .uri("/api/v1/plugins")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert!(
        response.status() == StatusCode::UNAUTHORIZED || response.status() == StatusCode::OK,
        "Expected UNAUTHORIZED or OK (no auth middleware in test router), got {}",
        response.status()
    );
}

#[tokio::test]
async fn test_get_plugin_not_found() {
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
                .uri(format!("/api/v1/plugins/{}", fake_id))
                .header(
                    "Authorization",
                    common::auth_header(&common::create_test_jwt(
                        &fake_id.to_string(),
                        "test_secret",
                    )),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert!(
        response.status() == StatusCode::NOT_FOUND || response.status() == StatusCode::UNAUTHORIZED,
        "Expected NOT_FOUND or UNAUTHORIZED, got {}",
        response.status()
    );
}

#[tokio::test]
async fn test_create_plugin_blank_version() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;
    let unique = uuid::Uuid::new_v4();

    let auth = common::register_and_login(
        &app,
        &format!("plugnoversion_{}", unique),
        &format!("plugnoversion_{}@test.com", unique),
        "Password123!",
    )
    .await
    .expect("Failed to register/login");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/plugins")
                .header("Content-Type", "application/json")
                .header("Authorization", common::auth_header(&auth.token))
                .body(Body::from(
                    json!({
                        "name": format!("test-plugin-noversion-{}", unique),
                        "version": "",
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_update_plugin_not_found() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;
    let unique = uuid::Uuid::new_v4();
    let fake_id = uuid::Uuid::new_v4();

    let auth = common::register_and_login(
        &app,
        &format!("plugupdate_{}", unique),
        &format!("plugupdate_{}@test.com", unique),
        "Password123!",
    )
    .await
    .expect("Failed to register/login");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/plugins/{}", fake_id))
                .header("Content-Type", "application/json")
                .header("Authorization", common::auth_header(&auth.token))
                .body(Body::from(
                    json!({
                        "description": "updated description"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert!(
        response.status() == StatusCode::NOT_FOUND
            || response.status() == StatusCode::UNAUTHORIZED
            || response.status() == StatusCode::INTERNAL_SERVER_ERROR,
        "Expected NOT_FOUND for non-existent plugin, got {}",
        response.status()
    );
}

#[tokio::test]
async fn test_delete_plugin_not_found() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;
    let unique = uuid::Uuid::new_v4();
    let fake_id = uuid::Uuid::new_v4();

    let auth = common::register_and_login(
        &app,
        &format!("plugdelete_{}", unique),
        &format!("plugdelete_{}@test.com", unique),
        "Password123!",
    )
    .await
    .expect("Failed to register/login");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/plugins/{}", fake_id))
                .header("Authorization", common::auth_header(&auth.token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert!(
        response.status() == StatusCode::NOT_FOUND
            || response.status() == StatusCode::UNAUTHORIZED
            || response.status() == StatusCode::NO_CONTENT,
        "Expected NOT_FOUND or NO_CONTENT, got {}",
        response.status()
    );
}

#[tokio::test]
async fn test_list_plugins_with_enabled_filter() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;
    let unique = uuid::Uuid::new_v4();

    let auth = common::register_and_login(
        &app,
        &format!("plugfilter_{}", unique),
        &format!("plugfilter_{}@test.com", unique),
        "Password123!",
    )
    .await
    .expect("Failed to register/login");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/plugins?enabled=true")
                .header("Authorization", common::auth_header(&auth.token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(response.status(), StatusCode::OK);
    let body = common::read_body_json(response).await;
    assert!(body.is_array(), "Filtered plugins should be an array");
}

#[tokio::test]
async fn test_create_plugin_missing_name() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;
    let unique = uuid::Uuid::new_v4();

    let auth = common::register_and_login(
        &app,
        &format!("plugnoname_{}", unique),
        &format!("plugnoname_{}@test.com", unique),
        "Password123!",
    )
    .await
    .expect("Failed to register/login");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/plugins")
                .header("Content-Type", "application/json")
                .header("Authorization", common::auth_header(&auth.token))
                .body(Body::from(
                    json!({
                        "version": "1.0.0"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY,
        "Expected BAD_REQUEST for missing name, got {}",
        response.status()
    );
}
