// Integration tests for team management routes
use axum::{
    body::Body,
    http::{Request, StatusCode, header},
    Router,
};
use tower::ServiceExt;
use serde_json::{json, Value};

use crate::common;

#[tokio::test]
async fn test_create_team() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;
    let unique = uuid::Uuid::new_v4();

    let auth = common::register_and_login(
        &app,
        &format!("teamuser_{}", unique),
        &format!("teamuser_{}@test.com", unique),
        "Password123!",
    )
    .await
    .expect("Failed to register/login test user");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/teams")
                .header("Content-Type", "application/json")
                .header("Authorization", common::auth_header(&auth.token))
                .body(Body::from(
                    json!({
                        "name": &format!("Test Team {}", unique),
                        "description": "A test team",
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(response.status(), StatusCode::CREATED);
    let body: Value = common::read_body_json(response).await;
    assert_eq!(body["name"].as_str().unwrap(), &format!("Test Team {}", unique));
    assert!(body["id"].is_string());
}

#[tokio::test]
async fn test_list_teams() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;
    let auth = common::create_test_user(&app, "listuser", "listuser@test.com", "Password123!")
        .await
        .expect("Failed to create test user");

    // Create two teams
    for i in 0..2 {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/teams")
                    .header("Content-Type", "application/json")
                    .header("Authorization", common::auth_header(&auth.token))
                    .body(Body::from(
                        json!({
                            "name": &format!("Team {}", i),
                            "description": &format!("Team number {}", i),
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .expect("Request failed");
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/teams")
                .header("Authorization", common::auth_header(&auth.token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = common::read_body_json(response).await;
    let results = body["teams"].as_array().expect("teams should be array");
    assert!(results.len() >= 2, "Should have at least 2 teams");
}

#[tokio::test]
async fn test_add_team_member() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;
    let unique = uuid::Uuid::new_v4();

    let owner_auth = common::register_and_login(
        &app,
        &format!("owner_{}", unique),
        &format!("owner_{}@test.com", unique),
        "Password123!",
    )
    .await
    .expect("Failed to create owner");

    // Create team
    let team_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/teams")
                .header("Content-Type", "application/json")
                .header("Authorization", common::auth_header(&owner_auth.token))
                .body(Body::from(
                    json!({
                        "name": &format!("Member Team {}", unique),
                        "description": "Team for member test",
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("Team creation failed");

    let body: Value = common::read_body_json(team_response).await;
    let team_id = body["id"].as_str().expect("team id missing").to_string();

    // Register a second user as member
    let member_auth = common::register_and_login(
        &app,
        &format!("member_{}", unique),
        &format!("member_{}@test.com", unique),
        "Password123!",
    )
    .await
    .expect("Failed to create member user");

    // Add member to team
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/teams/{}/members", team_id))
                .header("Content-Type", "application/json")
                .header("Authorization", common::auth_header(&owner_auth.token))
                .body(Body::from(
                    json!({
                        "user_id": &member_auth.user_id,
                        "role": "editor",
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("Add member request failed");

    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_unauthorized_team_access() {
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
                .uri("/api/v1/teams")
                .header("Authorization", "Bearer invalid_token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
